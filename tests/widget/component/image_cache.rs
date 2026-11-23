use super::*;

fn png(width: u32, height: u32) -> Arc<[u8]> {
    let image = image::RgbaImage::from_pixel(width, height, image::Rgba([200, 100, 50, 255]));
    let mut bytes = std::io::Cursor::new(Vec::new());
    image.write_to(&mut bytes, image::ImageFormat::Png).unwrap();

    bytes.into_inner().into()
}

fn fitted(max: u32) -> Processing {
    Processing {
        max_dimension: Some(max),
        ..Processing::default()
    }
}

/// A figure of `color` at `(x, y, width, height)` on white.
fn figure(size: (u32, u32), at: (u32, u32, u32, u32), color: [u8; 4]) -> image::RgbaImage {
    let mut image = image::RgbaImage::from_pixel(size.0, size.1, image::Rgba([255, 255, 255, 255]));

    for y in at.1..at.1 + at.3 {
        for x in at.0..at.0 + at.2 {
            image.put_pixel(x, y, image::Rgba(color));
        }
    }

    image
}

fn loaded(key: u32, result: Result<Handle, Arc<str>>) -> Loaded<u32> {
    Loaded { key, result }
}

fn ready(key: u32) -> Loaded<u32> {
    loaded(key, Ok(Handle::from_rgba(1, 1, vec![0; 4])))
}

#[test]
fn requests_load_once_until_removed() {
    let mut cache = Cache::new(4);

    let _ = cache.request(1, Source::Bytes(png(1, 1)), |_| ());
    assert!(matches!(cache.status(&1), Some(Status::Loading)));

    cache.insert(ready(1));
    assert!(cache.handle(&1).is_some());

    // Asking again keeps the decoded image.
    let _ = cache.request(1, Source::Bytes(png(1, 1)), |_| ());
    assert!(cache.handle(&1).is_some());

    cache.remove(&1);
    assert!(cache.status(&1).is_none());
    assert!(cache.is_empty());
}

#[test]
fn failures_are_kept_with_their_reason() {
    let mut cache = Cache::new(4);
    let _ = cache.request(7, Source::Path("/nonexistent.png".into()), |_| ());

    cache.insert(loaded(7, Err("no such file".into())));

    assert!(
        matches!(cache.status(&7), Some(Status::Failed(reason)) if &**reason == "no such file")
    );
    assert!(cache.handle(&7).is_none());
}

#[test]
fn results_for_forgotten_requests_are_dropped() {
    let mut cache = Cache::new(4);
    let _ = cache.request(1, Source::Bytes(png(1, 1)), |_| ());
    cache.remove(&1);

    cache.insert(ready(1));
    // Never requested at all.
    cache.insert(ready(2));

    assert!(cache.is_empty());
}

#[test]
fn the_least_recently_requested_images_go_first() {
    let mut cache = Cache::new(2);

    for key in [1, 2] {
        let _ = cache.request(key, Source::Bytes(png(1, 1)), |_| ());
        cache.insert(ready(key));
    }

    // Requesting 1 again makes 2 the least recent.
    let _ = cache.request(1, Source::Bytes(png(1, 1)), |_| ());
    let _ = cache.request(3, Source::Bytes(png(1, 1)), |_| ());
    cache.insert(ready(3));

    assert!(cache.handle(&1).is_some());
    assert!(cache.status(&2).is_none());
    assert!(cache.handle(&3).is_some());
    assert_eq!(cache.len(), 2);
}

#[test]
fn loading_images_are_not_evicted() {
    let mut cache = Cache::new(1);

    let _ = cache.request(1, Source::Bytes(png(1, 1)), |_| ());
    let _ = cache.request(2, Source::Bytes(png(1, 1)), |_| ());
    cache.insert(ready(1));

    assert!(cache.handle(&1).is_some());
    assert!(matches!(cache.status(&2), Some(Status::Loading)));
}

#[test]
fn large_images_are_fitted_keeping_their_aspect_ratio() {
    assert_eq!(fitted_size(1200, 1800, Some(600)), (400, 600));
    assert_eq!(fitted_size(1800, 1200, Some(600)), (600, 400));
    assert_eq!(fitted_size(300, 200, Some(600)), (300, 200));
    assert_eq!(fitted_size(3000, 1, Some(600)), (600, 1));
    assert_eq!(fitted_size(1200, 1800, None), (1200, 1800));
}

#[test]
fn decoding_reads_and_fits_encoded_images() {
    let handle = decode(&Source::Bytes(png(40, 20)), &fitted(10)).unwrap();

    match handle {
        Handle::Rgba {
            width,
            height,
            pixels,
            ..
        } => {
            assert_eq!((width, height), (10, 5));
            assert_eq!(&pixels[..4], &[200, 100, 50, 255]);
        }
        other => panic!("expected decoded pixels, got {other:?}"),
    }

    assert!(
        decode(
            &Source::Bytes(Arc::from(&b"not an image"[..])),
            &Processing::default()
        )
        .is_err()
    );
    assert!(
        decode(
            &Source::Path("/nonexistent/cover.png".into()),
            &Processing::default()
        )
        .is_err()
    );
}

#[test]
fn cover_crops_keep_the_center_at_the_cell_ratio() {
    // A landscape image in a 3:4 cell keeps its middle.
    assert_eq!(cover_crop(1000, 700, 3.0 / 4.0), (237, 0, 525, 700));
    // A tall image in a square cell keeps its middle rows.
    assert_eq!(cover_crop(300, 500, 1.0), (0, 100, 300, 300));
    // An image at the ratio, or a ratio that is none, stays whole.
    assert_eq!(cover_crop(300, 400, 3.0 / 4.0), (0, 0, 300, 400));
    assert_eq!(cover_crop(300, 400, 0.0), (0, 0, 300, 400));
    assert_eq!(cover_crop(0, 400, 1.0), (0, 0, 0, 400));
}

#[test]
fn files_decode_by_their_contents_whatever_their_name() {
    let directory = std::env::temp_dir().join(format!("material-cache-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join("05cb788083949e4ccd54c1fda698ca4d");
    std::fs::write(&path, &*png(8, 4)).unwrap();

    let handle = decode(&Source::Path(path), &Processing::default());
    std::fs::remove_dir_all(&directory).unwrap();

    match handle.unwrap() {
        Handle::Rgba { width, height, .. } => assert_eq!((width, height), (8, 4)),
        other => panic!("expected decoded pixels, got {other:?}"),
    }
}

#[test]
fn decoding_crops_before_fitting() {
    let processing = Processing {
        aspect_ratio: Some(1.0),
        max_dimension: Some(100),
        ..Processing::default()
    };
    let handle = decode(&Source::Bytes(png(400, 200)), &processing).unwrap();

    match handle {
        Handle::Rgba { width, height, .. } => assert_eq!((width, height), (100, 100)),
        other => panic!("expected decoded pixels, got {other:?}"),
    }
}

#[test]
fn borders_of_the_background_color_are_trimmed() {
    let portrait = figure((300, 600), (110, 40, 80, 520), [30, 40, 50, 255]);

    assert_eq!(content_bounds(&portrait), (110, 40, 80, 520));
    // Near-white compression noise is background too.
    let mut noisy = portrait.clone();
    noisy.put_pixel(5, 5, image::Rgba([245, 250, 240, 255]));
    assert_eq!(content_bounds(&noisy), (110, 40, 80, 520));
    // A blank image, or one without a margin, stays whole.
    assert_eq!(
        content_bounds(&figure((10, 10), (0, 0, 0, 0), [0; 4])),
        (0, 0, 10, 10)
    );
    assert_eq!(
        content_bounds(&figure((10, 10), (0, 0, 10, 10), [0, 0, 0, 255])),
        (0, 0, 10, 10)
    );
}

#[test]
fn portraits_crop_from_the_top_after_trimming() {
    assert_eq!(cover_crop_at(120, 600, 3.0 / 4.0, true), (0, 0, 120, 160));
    assert_eq!(
        cover_crop_at(120, 600, 3.0 / 4.0, false),
        (0, 220, 120, 160)
    );
    // Wide images still crop around their middle.
    assert_eq!(
        cover_crop_at(400, 300, 3.0 / 4.0, true),
        cover_crop(400, 300, 3.0 / 4.0)
    );

    let mut bytes = std::io::Cursor::new(Vec::new());
    figure((300, 600), (110, 40, 80, 520), [30, 40, 50, 255])
        .write_to(&mut bytes, image::ImageFormat::Png)
        .unwrap();
    let processing = Processing {
        trim_borders: true,
        aspect_ratio: Some(3.0 / 4.0),
        from_top: true,
        max_dimension: None,
    };

    match decode(&Source::Bytes(bytes.into_inner().into()), &processing).unwrap() {
        Handle::Rgba {
            width,
            height,
            pixels,
            ..
        } => {
            // The figure, from its top, fills the whole cell.
            assert_eq!((width, height), (80, 107));
            assert!(pixels.chunks(4).all(|pixel| pixel == [30, 40, 50, 255]));
        }
        other => panic!("expected decoded pixels, got {other:?}"),
    }
}
