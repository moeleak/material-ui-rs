use iced_widget::container::Style;
use iced_widget::core::{border, Background};

use crate::{tokens, Theme};

pub fn item(theme: &Theme) -> Style {
    let surface = theme.colors().surface;

    Style {
        background: Some(Background::Color(surface.color)),
        text_color: Some(surface.text),
        border: border::rounded(tokens::component::list::CONTAINER_SHAPE),
        ..Style::default()
    }
}

pub fn disabled_item(theme: &Theme) -> Style {
    let surface = theme.colors().surface;

    Style {
        background: Some(Background::Color(surface.color)),
        text_color: Some(iced_widget::core::Color {
            a: tokens::component::list::DISABLED_LABEL_TEXT_OPACITY,
            ..surface.text
        }),
        border: border::rounded(tokens::component::list::CONTAINER_SHAPE),
        ..Style::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_item_uses_m3_surface_and_shape_tokens() {
        let theme = Theme::Light;
        let colors = theme.colors();
        let style = item(&theme);

        assert_eq!(style.background, Some(Background::Color(colors.surface.color)));
        assert_eq!(style.text_color, Some(colors.surface.text));
        assert_eq!(
            style.border.radius,
            tokens::component::list::CONTAINER_SHAPE.into()
        );
    }

    #[test]
    fn disabled_list_item_uses_m3_disabled_label_opacity() {
        let theme = Theme::Light;
        let style = disabled_item(&theme);

        assert_eq!(
            style.text_color.map(|color| color.a),
            Some(tokens::component::list::DISABLED_LABEL_TEXT_OPACITY)
        );
    }
}
