use iced_widget::container::Style;
use iced_widget::core::{Background, Shadow, border};

use crate::Theme;
use crate::tokens;
use crate::utils::shadow_from_level;

pub fn plain(theme: &Theme) -> Style {
    let inverse = theme.colors().inverse;

    Style {
        text_color: Some(inverse.inverse_surface_text),
        background: Some(Background::Color(inverse.inverse_surface)),
        border: border::rounded(tokens::component::tooltip::PLAIN_CONTAINER_SHAPE),
        shadow: Shadow::default(),
        snap: cfg!(feature = "crisp"),
    }
}

pub fn rich(theme: &Theme) -> Style {
    let colors = theme.colors();

    Style {
        text_color: Some(colors.surface.text_variant),
        background: Some(Background::Color(colors.surface.container.base)),
        border: border::rounded(tokens::component::tooltip::RICH_CONTAINER_SHAPE),
        shadow: shadow_from_level(
            tokens::component::tooltip::RICH_CONTAINER_ELEVATION_LEVEL,
            colors.shadow,
        ),
        snap: cfg!(feature = "crisp"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_tooltip_uses_m3_plain_tooltip_tokens() {
        let theme = Theme::Light;
        let colors = theme.colors();
        let style = plain(&theme);

        assert_eq!(
            style.background,
            Some(Background::Color(colors.inverse.inverse_surface))
        );
        assert_eq!(style.text_color, Some(colors.inverse.inverse_surface_text));
        assert_eq!(
            style.border.radius.top_left,
            tokens::component::tooltip::PLAIN_CONTAINER_SHAPE
        );
        assert_eq!(style.shadow, Shadow::default());
    }

    #[test]
    fn rich_tooltip_uses_m3_rich_tooltip_tokens() {
        let theme = Theme::Light;
        let colors = theme.colors();
        let style = rich(&theme);

        assert_eq!(
            style.background,
            Some(Background::Color(colors.surface.container.base))
        );
        assert_eq!(style.text_color, Some(colors.surface.text_variant));
        assert_eq!(
            style.border.radius.top_left,
            tokens::component::tooltip::RICH_CONTAINER_SHAPE
        );
        assert_eq!(style.shadow.offset.y, 2.0);
        assert_eq!(style.shadow.blur_radius, 6.0);
    }
}
