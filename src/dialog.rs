use iced_dialog::dialog::{Catalog, Style, StyleFn};
use iced_widget::container;
use iced_widget::core::{Background, border};

use super::{Theme, text};
use crate::tokens;
use crate::utils::shadow_from_level;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> <Self as Catalog>::Class<'a> {
        Box::new(default)
    }

    fn default_container<'a>() -> <Self as container::Catalog>::Class<'a> {
        Box::new(default_container)
    }

    fn default_title<'a>() -> <Self as iced_widget::text::Catalog>::Class<'a> {
        Box::new(text::surface)
    }

    fn style(&self, class: &<Self as Catalog>::Class<'_>) -> Style {
        class(self)
    }
}

pub fn default_container(theme: &Theme) -> container::Style {
    let scheme = theme.colors();
    let colors = scheme.surface;
    container::Style {
        background: Some(Background::Color(colors.container.high)),
        text_color: Some(colors.text_variant),
        border: border::rounded(tokens::component::dialog::CONTAINER_SHAPE),
        shadow: shadow_from_level(
            tokens::component::dialog::CONTAINER_ELEVATION_LEVEL,
            scheme.shadow,
        ),
        ..container::Style::default()
    }
}

pub fn default(theme: &Theme) -> Style {
    Style {
        backdrop_color: theme.colors().scrim,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_dialog_container_uses_m3_container_tokens() {
        let theme = Theme::Light;
        let colors = theme.colors();
        let style = default_container(&theme);

        assert_eq!(
            style.background,
            Some(Background::Color(colors.surface.container.high))
        );
        assert_eq!(
            style.border.radius.top_left,
            tokens::component::dialog::CONTAINER_SHAPE
        );
        assert_eq!(style.shadow.offset.y, 4.0);
        assert_eq!(style.shadow.blur_radius, 8.0);
    }
}
