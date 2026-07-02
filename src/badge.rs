use iced_widget::container::Style;
use iced_widget::core::{Background, border};

use crate::{Theme, tokens};

pub fn default(theme: &Theme) -> Style {
    let error = theme.colors().error;

    Style {
        background: Some(Background::Color(error.color)),
        text_color: Some(error.text),
        border: border::rounded(tokens::component::badge::LARGE_CONTAINER_SHAPE),
        ..Style::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn badge_uses_m3_error_container_and_on_error_text() {
        let theme = Theme::Light;
        let colors = theme.colors();
        let style = default(&theme);

        assert_eq!(
            style.background,
            Some(Background::Color(colors.error.color))
        );
        assert_eq!(style.text_color, Some(colors.error.text));
        assert_eq!(
            style.border.radius,
            tokens::component::badge::LARGE_CONTAINER_SHAPE.into()
        );
    }
}
