//! A cache of decoded images that loads them in the background.
//!
//! Decoding an image file on the frame that first draws it stalls that frame.
//! A [`Cache`] decodes images on the executor instead, optionally down to a
//! bounded size, and keeps them for the frames after: request an image when it
//! comes into view, for example from the `on_show` of a
//! [`sensor`](fn@iced::widget::sensor), and hand the [`Loaded`] result back to
//! [`Cache::insert`].
//!
//! ```ignore
//! Message::Show(id) => self.covers.request(id, Source::Path(path), Message::Loaded),
//! Message::Loaded(loaded) => {
//!     self.covers.insert(loaded);
//!     Task::none()
//! }
//! ```

use std::collections::{HashMap, VecDeque};
use std::hash::Hash;
use std::path::PathBuf;
use std::sync::Arc;

use iced::Task;

use super::Handle;

/// Where an image is decoded from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Source {
    /// An image file.
    Path(PathBuf),
    /// An encoded image in memory.
    Bytes(Arc<[u8]>),
}

/// The state of an image in a [`Cache`].
#[derive(Debug, Clone)]
pub enum Status {
    /// It is being decoded.
    Loading,
    /// It is decoded.
    Ready(Handle),
    /// It could not be decoded, for the given reason.
    Failed(Arc<str>),
}

/// An image a [`Cache`] decoded, or failed to, for [`Cache::insert`].
#[derive(Debug, Clone)]
pub struct Loaded<K> {
    key: K,
    result: Result<Handle, Arc<str>>,
}

impl<K> Loaded<K> {
    /// The key the image was requested for.
    pub fn key(&self) -> &K {
        &self.key
    }

    /// Whether the image was decoded.
    pub fn is_ok(&self) -> bool {
        self.result.is_ok()
    }
}

/// Decoded images by key, of which at most `capacity` are kept, the least
/// recently requested going first.
///
/// Keep the capacity above the number of images shown at once: an image
/// dropped while it is shown is not requested again until it comes back into
/// view, and shows its placeholder until then.
#[derive(Debug, Clone)]
pub struct Cache<K> {
    entries: HashMap<K, Status>,
    /// The keys of the decoded and failed images, least recently requested first.
    recent: VecDeque<K>,
    capacity: usize,
    max_dimension: Option<u32>,
}

impl<K> Cache<K>
where
    K: Eq + Hash + Clone,
{
    /// A cache keeping at most `capacity` images.
    pub fn new(capacity: usize) -> Self {
        Self {
            entries: HashMap::new(),
            recent: VecDeque::new(),
            capacity: capacity.max(1),
            max_dimension: None,
        }
    }

    /// Decodes images down to at most `max` pixels on their longer side,
    /// keeping their aspect ratio, as covers in a grid need.
    pub fn max_dimension(mut self, max: u32) -> Self {
        self.max_dimension = Some(max.max(1));
        self
    }

    /// The state of the image of `key`, if it was requested.
    pub fn status(&self, key: &K) -> Option<&Status> {
        self.entries.get(key)
    }

    /// The decoded image of `key`, if it is ready.
    pub fn handle(&self, key: &K) -> Option<&Handle> {
        match self.entries.get(key) {
            Some(Status::Ready(handle)) => Some(handle),
            _ => None,
        }
    }

    /// How many images are loading, decoded or failed.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether no image was requested.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Starts decoding the image of `key` from `source`, unless it was
    /// requested already, whether it is loading, decoded or failed to decode:
    /// the task decodes it in the background and produces `on_loaded` of the
    /// result, for [`Cache::insert`]. [`Cache::remove`] it to decode it again.
    pub fn request<Message>(
        &mut self,
        key: K,
        source: Source,
        on_loaded: impl FnOnce(Loaded<K>) -> Message + Send + 'static,
    ) -> Task<Message>
    where
        K: Send + 'static,
        Message: Send + 'static,
    {
        if self.entries.contains_key(&key) {
            self.touch(&key);
            return Task::none();
        }

        let _ = self.entries.insert(key.clone(), Status::Loading);
        let max_dimension = self.max_dimension;

        Task::perform(
            async move { decode(&source, max_dimension) },
            move |result| on_loaded(Loaded { key, result }),
        )
    }

    /// Keeps an image a request decoded, or its failure, dropping the least
    /// recently requested images past the capacity. Images that are no longer
    /// requested, because they were removed meanwhile, are dropped.
    pub fn insert(&mut self, loaded: Loaded<K>) {
        let Loaded { key, result } = loaded;

        if !matches!(self.entries.get(&key), Some(Status::Loading)) {
            return;
        }

        let status = match result {
            Ok(handle) => Status::Ready(handle),
            Err(reason) => Status::Failed(reason),
        };

        let _ = self.entries.insert(key.clone(), status);
        self.recent.push_back(key);
        self.evict();
    }

    /// Forgets the image of `key`, for example because its file changed, so
    /// that the next request decodes it again.
    pub fn remove(&mut self, key: &K) {
        let _ = self.entries.remove(key);
        self.recent.retain(|recent| recent != key);
    }

    /// Forgets every image.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.recent.clear();
    }

    fn touch(&mut self, key: &K) {
        if let Some(index) = self.recent.iter().position(|recent| recent == key)
            && let Some(key) = self.recent.remove(index)
        {
            self.recent.push_back(key);
        }
    }

    fn evict(&mut self) {
        while self.recent.len() > self.capacity {
            if let Some(key) = self.recent.pop_front() {
                let _ = self.entries.remove(&key);
            }
        }
    }
}

/// The size an image of `width` by `height` is decoded to, at most `max` on
/// its longer side and keeping its aspect ratio; images that fit keep their size.
pub fn fitted_size(width: u32, height: u32, max: Option<u32>) -> (u32, u32) {
    let Some(max) = max else {
        return (width, height);
    };

    if width <= max && height <= max {
        return (width, height);
    }

    let scale = f64::from(max) / f64::from(width.max(height));
    let fit = |side: u32| ((f64::from(side) * scale).round() as u32).max(1);

    (fit(width), fit(height))
}

fn decode(source: &Source, max_dimension: Option<u32>) -> Result<Handle, Arc<str>> {
    let image = match source {
        Source::Path(path) => image::open(path),
        Source::Bytes(bytes) => image::load_from_memory(bytes),
    }
    .map_err(|error| Arc::from(error.to_string()))?;

    let (width, height) = fitted_size(image.width(), image.height(), max_dimension);
    let image = if (width, height) == (image.width(), image.height()) {
        image
    } else {
        image.resize_exact(width, height, image::imageops::FilterType::Triangle)
    };

    let rgba = image.into_rgba8();
    let (width, height) = rgba.dimensions();

    Ok(Handle::from_rgba(width, height, rgba.into_raw()))
}

#[cfg(test)]
#[path = "../../../../tests/widget/component/image_cache.rs"]
mod tests;
