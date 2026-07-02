use iced_widget::container::{Catalog, Style, StyleFn};
use iced_widget::core::{Background, Border, border};

use super::Theme;
use crate::tokens;
use crate::utils::shadow_from_level;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(transparent)
    }

    fn style(&self, class: &Self::Class<'_>) -> Style {
        class(self)
    }
}

pub fn transparent(_theme: &Theme) -> Style {
    Style {
        border: border::rounded(tokens::shape::CORNER_EXTRA_SMALL),
        ..Style::default()
    }
}

pub fn primary(theme: &Theme) -> Style {
    let primary = theme.colors().primary;

    Style {
        background: Some(Background::Color(primary.color)),
        text_color: Some(primary.text),
        border: border::rounded(tokens::shape::CORNER_EXTRA_SMALL),
        ..Style::default()
    }
}

pub fn primary_container(theme: &Theme) -> Style {
    let primary = theme.colors().primary;

    Style {
        background: Some(Background::Color(primary.container)),
        text_color: Some(primary.container_text),
        border: border::rounded(tokens::shape::CORNER_SMALL),
        ..Style::default()
    }
}

pub fn secondary(theme: &Theme) -> Style {
    let secondary = theme.colors().secondary;

    Style {
        background: Some(Background::Color(secondary.color)),
        text_color: Some(secondary.text),
        border: border::rounded(tokens::shape::CORNER_EXTRA_SMALL),
        ..Style::default()
    }
}

pub fn secondary_container(theme: &Theme) -> Style {
    let secondary = theme.colors().secondary;

    Style {
        background: Some(Background::Color(secondary.container)),
        text_color: Some(secondary.container_text),
        border: border::rounded(tokens::shape::CORNER_SMALL),
        ..Style::default()
    }
}

pub fn tertiary(theme: &Theme) -> Style {
    let tertiary = theme.colors().tertiary;

    Style {
        background: Some(Background::Color(tertiary.color)),
        text_color: Some(tertiary.text),
        border: border::rounded(tokens::shape::CORNER_EXTRA_SMALL),
        ..Style::default()
    }
}

pub fn tertiary_container(theme: &Theme) -> Style {
    let tertiary = theme.colors().tertiary;

    Style {
        background: Some(Background::Color(tertiary.container)),
        text_color: Some(tertiary.container_text),
        border: border::rounded(tokens::shape::CORNER_SMALL),
        ..Style::default()
    }
}

pub fn error(theme: &Theme) -> Style {
    let error = theme.colors().error;

    Style {
        background: Some(Background::Color(error.color)),
        text_color: Some(error.text),
        border: border::rounded(tokens::shape::CORNER_EXTRA_SMALL),
        ..Style::default()
    }
}

pub fn error_container(theme: &Theme) -> Style {
    let error = theme.colors().error;

    Style {
        background: Some(Background::Color(error.container)),
        text_color: Some(error.container_text),
        border: border::rounded(tokens::shape::CORNER_SMALL),
        ..Style::default()
    }
}

pub fn surface(theme: &Theme) -> Style {
    let surface = theme.colors().surface;

    Style {
        background: Some(Background::Color(surface.color)),
        text_color: Some(surface.text),
        border: border::rounded(tokens::shape::CORNER_EXTRA_SMALL),
        ..Style::default()
    }
}

pub fn surface_container_lowest(theme: &Theme) -> Style {
    let surface = theme.colors().surface;

    Style {
        background: Some(Background::Color(surface.container.lowest)),
        text_color: Some(surface.text),
        border: border::rounded(tokens::shape::CORNER_SMALL),
        ..Style::default()
    }
}

pub fn surface_container_low(theme: &Theme) -> Style {
    let surface = theme.colors().surface;

    Style {
        background: Some(Background::Color(surface.container.low)),
        text_color: Some(surface.text),
        border: border::rounded(tokens::shape::CORNER_SMALL),
        ..Style::default()
    }
}

pub fn surface_container(theme: &Theme) -> Style {
    let surface = theme.colors().surface;

    Style {
        background: Some(Background::Color(surface.container.base)),
        text_color: Some(surface.text),
        border: border::rounded(tokens::shape::CORNER_SMALL),
        ..Style::default()
    }
}

pub fn surface_container_high(theme: &Theme) -> Style {
    let surface = theme.colors().surface;

    Style {
        background: Some(Background::Color(surface.container.high)),
        text_color: Some(surface.text),
        border: border::rounded(tokens::shape::CORNER_SMALL),
        ..Style::default()
    }
}

pub fn surface_container_highest(theme: &Theme) -> Style {
    let surface = theme.colors().surface;

    Style {
        background: Some(Background::Color(surface.container.highest)),
        text_color: Some(surface.text),
        border: border::rounded(tokens::shape::CORNER_SMALL),
        ..Style::default()
    }
}

pub fn inverse_surface(theme: &Theme) -> Style {
    let inverse = theme.colors().inverse;

    Style {
        background: Some(Background::Color(inverse.inverse_surface)),
        text_color: Some(inverse.inverse_surface_text),
        border: border::rounded(tokens::shape::CORNER_EXTRA_SMALL),
        ..Style::default()
    }
}

pub fn outlined(theme: &Theme) -> Style {
    let base = transparent(theme);

    Style {
        border: Border {
            color: theme.colors().outline.color,
            width: tokens::component::button::OUTLINED_OUTLINE_WIDTH,
            ..base.border
        },
        ..base
    }
}

pub fn elevated_card(theme: &Theme) -> Style {
    let colors = theme.colors();

    Style {
        background: Some(Background::Color(colors.surface.container.low)),
        text_color: Some(colors.surface.text),
        border: border::rounded(tokens::component::card::CONTAINER_SHAPE),
        shadow: shadow_from_level(
            tokens::component::card::ELEVATED_ELEVATION.active,
            colors.shadow,
        ),
        ..Style::default()
    }
}

pub fn filled_card(theme: &Theme) -> Style {
    let colors = theme.colors();

    Style {
        background: Some(Background::Color(colors.surface.container.highest)),
        text_color: Some(colors.surface.text),
        border: border::rounded(tokens::component::card::CONTAINER_SHAPE),
        shadow: shadow_from_level(
            tokens::component::card::FILLED_ELEVATION.active,
            colors.shadow,
        ),
        ..Style::default()
    }
}

pub fn outlined_card(theme: &Theme) -> Style {
    let colors = theme.colors();

    Style {
        background: Some(Background::Color(colors.surface.color)),
        text_color: Some(colors.surface.text),
        border: Border {
            color: colors.outline.variant,
            width: tokens::component::card::OUTLINED_OUTLINE_WIDTH,
            radius: tokens::component::card::CONTAINER_SHAPE.into(),
        },
        shadow: shadow_from_level(
            tokens::component::card::OUTLINED_ELEVATION.active,
            colors.shadow,
        ),
        ..Style::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn container_helpers_use_m3_shape_tokens() {
        assert_eq!(
            transparent(&Theme::Light).border.radius.top_left,
            tokens::shape::CORNER_EXTRA_SMALL
        );
        assert_eq!(
            surface_container(&Theme::Light).border.radius.top_left,
            tokens::shape::CORNER_SMALL
        );
    }

    #[test]
    fn outlined_container_uses_m3_one_pixel_outline() {
        let style = outlined(&Theme::Light);

        assert_eq!(style.border.color, Theme::Light.colors().outline.color);
        assert_eq!(
            style.border.width,
            tokens::component::button::OUTLINED_OUTLINE_WIDTH
        );
    }

    #[test]
    fn card_helpers_use_m3_container_tokens() {
        let theme = Theme::Light;
        let colors = theme.colors();

        let elevated = elevated_card(&theme);
        assert_eq!(
            elevated.background,
            Some(Background::Color(colors.surface.container.low))
        );
        assert_eq!(
            elevated.border.radius.top_left,
            tokens::component::card::CONTAINER_SHAPE
        );
        assert_eq!(elevated.shadow.offset.y, 1.0);
        assert_eq!(elevated.shadow.blur_radius, 3.0);

        let filled = filled_card(&theme);
        assert_eq!(
            filled.background,
            Some(Background::Color(colors.surface.container.highest))
        );
        assert_eq!(filled.shadow.offset.y, 0.0);

        let outlined = outlined_card(&theme);
        assert_eq!(
            outlined.background,
            Some(Background::Color(colors.surface.color))
        );
        assert_eq!(outlined.border.color, colors.outline.variant);
        assert_eq!(
            outlined.border.width,
            tokens::component::card::OUTLINED_OUTLINE_WIDTH
        );
    }
}
