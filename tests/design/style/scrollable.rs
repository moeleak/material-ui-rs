use super::*;
use iced_widget::core::{
    Color, Element, Layout, Length, Pixels, Rectangle, Size, layout, mouse, renderer, widget::Tree,
};
use iced_widget::{Container, Scrollable, Space};

fn viewport_pixels(
    theme: &Theme,
    content_height: f32,
    explicit_surface: bool,
) -> tiny_skia::Pixmap {
    let mut renderer = iced_tiny_skia::Renderer::new(crate::fonts::ROBOTO, Pixels(16.0));
    let scrollable = Scrollable::new(Space::new().width(Length::Fill).height(content_height))
        .width(Length::Fill)
        .height(64);
    let scrollable = if explicit_surface {
        scrollable.style(|theme, status| Style {
            container: crate::style::container::surface_container(theme),
            ..default(theme, status)
        })
    } else {
        scrollable
    };
    let mut element: Element<'_, (), Theme, iced_tiny_skia::Renderer> = Container::new(scrollable)
        .width(Length::Fill)
        .height(80)
        .padding(8)
        .style(|_| iced_widget::container::Style {
            background: Some(Color::from_rgb8(37, 67, 97).into()),
            ..iced_widget::container::Style::default()
        })
        .into();
    let mut tree = Tree::new(element.as_widget());
    let size = Size::new(120.0, 80.0);
    let node = element.as_widget_mut().layout(
        &mut tree,
        &renderer,
        &layout::Limits::new(Size::ZERO, size),
    );
    let viewport = Rectangle::with_size(size);
    element.as_widget().draw(
        &tree,
        &mut renderer,
        theme,
        &renderer::Style::default(),
        Layout::new(&node),
        mouse::Cursor::Unavailable,
        &viewport,
    );
    let mut pixels = tiny_skia::Pixmap::new(120, 80).unwrap();
    renderer.draw(
        &mut pixels.as_mut(),
        &mut tiny_skia::Mask::new(120, 80).unwrap(),
        &iced_widget::graphics::Viewport::with_physical_size(Size::new(120, 80), 1.0),
        &[viewport],
        Color::BLACK,
    );
    pixels
}

#[test]
fn default_viewport_preserves_the_enclosing_surface_with_and_without_overflow() {
    for theme in [Theme::Light, Theme::Dark] {
        for height in [24.0, 200.0] {
            let pixels = viewport_pixels(&theme, height, false);
            assert_eq!(
                pixels.pixel(20, 24),
                pixels.pixel(4, 24),
                "scrolling content painted a separate surface: theme={theme}, height={height}"
            );
        }
    }
}

#[test]
fn callers_can_still_opt_into_a_separate_scrollable_surface() {
    for theme in [Theme::Light, Theme::Dark] {
        let pixels = viewport_pixels(&theme, 200.0, true);
        assert_ne!(pixels.pixel(20, 24), pixels.pixel(4, 24));
    }
}
