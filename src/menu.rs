use iced_widget::core::{Background, border};
use iced_widget::overlay::menu::{Catalog, Style, StyleFn};

use super::Theme;
use crate::tokens;
use crate::utils::shadow_from_level;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> <Self as Catalog>::Class<'a> {
        Box::new(default)
    }

    fn style(&self, class: &<Self as Catalog>::Class<'_>) -> Style {
        class(self)
    }
}

pub fn default(theme: &Theme) -> Style {
    let colors = theme.colors();
    let surface = colors.surface;

    Style {
        border: border::rounded(tokens::component::menu::CONTAINER_SHAPE),
        background: Background::Color(surface.container.base),
        text_color: surface.text,
        selected_background: Background::Color(colors.secondary.container),
        selected_text_color: colors.secondary.container_text,
        shadow: shadow_from_level(
            tokens::component::menu::CONTAINER_ELEVATION_LEVEL,
            colors.shadow,
        ),
    }
}

pub fn outlined_select(theme: &Theme) -> Style {
    let colors = theme.colors();
    let surface = colors.surface;

    Style {
        border: border::rounded(tokens::component::select::MENU_CONTAINER_SHAPE),
        background: Background::Color(surface.container.base),
        text_color: surface.text,
        selected_background: Background::Color(surface.container.highest),
        selected_text_color: surface.text,
        shadow: shadow_from_level(
            tokens::component::select::MENU_CONTAINER_ELEVATION_LEVEL,
            colors.shadow,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_menu_uses_m3_container_tokens() {
        let theme = Theme::Light;
        let colors = theme.colors();
        let style = default(&theme);

        assert_eq!(
            style.background,
            Background::Color(colors.surface.container.base)
        );
        assert_eq!(
            style.border.radius.top_left,
            tokens::component::menu::CONTAINER_SHAPE
        );
        assert_eq!(
            style.selected_background,
            Background::Color(colors.secondary.container)
        );
        assert_eq!(style.selected_text_color, colors.secondary.container_text);
        assert_eq!(style.shadow.offset.y, 2.0);
        assert_eq!(style.shadow.blur_radius, 6.0);
    }

    #[test]
    fn outlined_select_menu_uses_m3_outlined_select_tokens() {
        let theme = Theme::Light;
        let colors = theme.colors();
        let style = outlined_select(&theme);

        assert_eq!(
            style.background,
            Background::Color(colors.surface.container.base)
        );
        assert_eq!(
            style.border.radius.top_left,
            tokens::component::select::MENU_CONTAINER_SHAPE
        );
        assert_eq!(
            style.selected_background,
            Background::Color(colors.surface.container.highest)
        );
        assert_eq!(style.selected_text_color, colors.surface.text);
        assert_eq!(style.shadow.offset.y, 2.0);
        assert_eq!(style.shadow.blur_radius, 6.0);
    }
}
