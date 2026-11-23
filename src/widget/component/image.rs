//! Material 3 images: rounded corners from the shape scale, and placeholders
//! standing in for images that are loading, failed to load, or are missing.
//!
//! Images fill their bounds and are cropped to cover them, as cover art in a
//! [grid](super::grid) is. [`cache`] decodes images in the background and keeps
//! them for the next frame.

use iced_widget::core::text as core_text;
use iced_widget::core::{Background, Element, Length, alignment, border};
use iced_widget::{Container, Space, container};

use crate::{Theme, fonts, text as text_style, tokens};

pub use iced_widget::core::ContentFit;
pub use iced_widget::image::{FilterMethod, Handle, Image};

#[path = "image/cache.rs"]
pub mod cache;

pub use cache::{Cache, Loaded, Source, Status};

/// The size of the icon in a [`placeholder`].
pub const PLACEHOLDER_ICON_SIZE: f32 = tokens::component::icon_button::ICON_SIZE;

/// An image with medium corners, filling its bounds and cropped to cover them.
pub fn image(handle: impl Into<Handle>) -> Image<Handle> {
    shaped(handle, tokens::shape::CORNER_MEDIUM)
}

/// An image with corners of `corner`, one of [`tokens::shape`], filling its
/// bounds and cropped to cover them.
pub fn shaped(handle: impl Into<Handle>, corner: f32) -> Image<Handle> {
    Image::new(handle)
        .width(Length::Fill)
        .height(Length::Fill)
        .content_fit(ContentFit::Cover)
        .border_radius(corner)
}

/// What a [`placeholder`] stands for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Placeholder {
    /// An image that is being loaded: a plain surface.
    Loading,
    /// An image that failed to load.
    Failed,
    /// An image that doesn't exist.
    Missing,
}

impl Placeholder {
    /// The Material Symbols icon the placeholder shows, if any.
    pub fn icon(self) -> Option<&'static str> {
        match self {
            Self::Loading => None,
            Self::Failed => Some("broken_image"),
            Self::Missing => Some("image"),
        }
    }
}

impl From<&Status> for Placeholder {
    fn from(status: &Status) -> Self {
        match status {
            Status::Loading | Status::Ready(_) => Self::Loading,
            Status::Failed(_) => Self::Failed,
        }
    }
}

/// A surface the size of the image it stands for, with corners of `corner`,
/// and the icon of `kind` in its center.
pub fn placeholder<'a, Message, Renderer>(
    kind: Placeholder,
    corner: f32,
) -> Container<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    faded_placeholder(kind, corner, 1.0)
}

/// A [`placeholder`] faded by `alpha`, as a dialog transition fades its content.
pub fn faded_placeholder<'a, Message, Renderer>(
    kind: Placeholder,
    corner: f32,
    alpha: f32,
) -> Container<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    let alpha = alpha.clamp(0.0, 1.0);
    let content: Element<'a, Message, Theme, Renderer> = match kind.icon() {
        Some(name) => fonts::icon(name, PLACEHOLDER_ICON_SIZE)
            .width(Length::Fixed(PLACEHOLDER_ICON_SIZE))
            .height(Length::Fixed(PLACEHOLDER_ICON_SIZE))
            .center()
            .style(move |theme: &Theme| iced_widget::text::Style {
                color: text_style::surface_variant(theme)
                    .color
                    .map(|color| color.scale_alpha(alpha)),
            })
            .into(),
        None => Space::new().into(),
    };

    Container::new(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(alignment::Horizontal::Center)
        .align_y(alignment::Vertical::Center)
        .style(move |theme: &Theme| placeholder_style(theme, corner, alpha))
}

fn placeholder_style(theme: &Theme, corner: f32, alpha: f32) -> container::Style {
    container::Style {
        background: Some(Background::Color(
            theme.colors().surface.container.highest.scale_alpha(alpha),
        )),
        border: border::rounded(corner),
        ..container::Style::default()
    }
}

/// The image of `key` in `cache` with corners of `corner`, or a placeholder
/// while it is loading or if it failed to.
pub fn cached<'a, K, Message, Renderer>(
    cache: &Cache<K>,
    key: &K,
    corner: f32,
) -> Element<'a, Message, Theme, Renderer>
where
    K: Eq + std::hash::Hash + Clone,
    Message: 'a,
    Renderer: core_text::Renderer + iced_widget::core::image::Renderer<Handle = Handle> + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    match cache.status(key) {
        Some(Status::Ready(handle)) => shaped(handle.clone(), corner).into(),
        Some(status) => placeholder(status.into(), corner).into(),
        None => placeholder(Placeholder::Loading, corner).into(),
    }
}

#[cfg(test)]
#[path = "../../../tests/widget/component/image.rs"]
mod tests;
