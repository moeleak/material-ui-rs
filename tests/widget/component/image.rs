use iced_widget::core::{Length, Size, Widget};

use super::*;

type Renderer = iced_tiny_skia::Renderer;

#[test]
fn images_cover_their_bounds_with_shape_corners() {
    let handle = Handle::from_rgba(2, 2, vec![0; 16]);
    let image = image(handle.clone());

    assert_eq!(
        Widget::<(), Theme, Renderer>::size(&image),
        Size::new(Length::Fill, Length::Fill)
    );

    let shaped = shaped(handle, tokens::shape::CORNER_LARGE);
    assert_eq!(
        Widget::<(), Theme, Renderer>::size(&shaped),
        Size::new(Length::Fill, Length::Fill)
    );
}

#[test]
fn placeholders_show_what_happened_to_the_image() {
    assert_eq!(Placeholder::Loading.icon(), None);
    assert_eq!(Placeholder::Failed.icon(), Some("broken_image"));
    assert_eq!(Placeholder::Missing.icon(), Some("image"));
    assert_eq!(Placeholder::from(&Status::Loading), Placeholder::Loading);
    assert_eq!(
        Placeholder::from(&Status::Failed("no such file".into())),
        Placeholder::Failed
    );
}

#[test]
fn placeholders_fill_the_image_bounds_on_the_highest_surface() {
    let placeholder: Container<'_, (), Theme, Renderer> =
        placeholder(Placeholder::Failed, tokens::shape::CORNER_MEDIUM);

    assert_eq!(
        Widget::<(), Theme, Renderer>::size(&placeholder),
        Size::new(Length::Fill, Length::Fill)
    );

    let style = placeholder_style(&Theme::Dark, tokens::shape::CORNER_MEDIUM, 1.0);
    assert_eq!(
        style.background,
        Some(Background::Color(
            Theme::Dark.colors().surface.container.highest
        ))
    );
    assert_eq!(
        style.border.radius,
        border::radius(tokens::shape::CORNER_MEDIUM)
    );
}

#[test]
fn faded_placeholders_scale_their_surface() {
    let faded = placeholder_style(&Theme::Dark, tokens::shape::CORNER_MEDIUM, 0.5);

    assert_eq!(
        faded.background,
        Some(Background::Color(
            Theme::Dark
                .colors()
                .surface
                .container
                .highest
                .scale_alpha(0.5)
        ))
    );
}
