use iced_widget::core::{Background, Color, border};
use iced_widget::slider::{Catalog, Handle, HandleShape, Rail, Status, Style, StyleFn};

use super::Theme;
use crate::tokens;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> <Self as Catalog>::Class<'a> {
        Box::new(default)
    }

    fn style(&self, class: &<Self as Catalog>::Class<'_>, status: Status) -> Style {
        class(self, status)
    }
}

pub fn styled(left: Color, right: Color, handle_radius: f32) -> Style {
    Style {
        rail: Rail {
            backgrounds: (left.into(), right.into()),
            width: tokens::component::slider::ACTIVE_TRACK_HEIGHT,
            border: border::rounded(tokens::component::slider::TRACK_SHAPE),
        },
        handle: Handle {
            shape: HandleShape::Circle {
                radius: handle_radius,
            },
            background: Background::Color(left),
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
        },
    }
}

pub fn default(theme: &Theme, status: Status) -> Style {
    let surface = theme.colors().surface;
    let primary = theme.colors().primary;
    let active = primary.color;
    let inactive = surface.container.highest;

    match status {
        Status::Active | Status::Hovered | Status::Dragged => {
            styled(active, inactive, tokens::component::slider::HANDLE_RADIUS)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced_widget::core::Background;

    #[test]
    fn default_slider_uses_m3_track_and_handle_tokens() {
        let theme = Theme::Light;
        let colors = theme.colors();
        let style = default(&theme, Status::Active);

        assert_eq!(
            style.rail.width,
            tokens::component::slider::ACTIVE_TRACK_HEIGHT
        );
        assert_eq!(
            style.rail.backgrounds,
            (
                Background::Color(colors.primary.color),
                Background::Color(colors.surface.container.highest)
            )
        );
        assert_eq!(
            style.handle.shape,
            HandleShape::Circle {
                radius: tokens::component::slider::HANDLE_RADIUS
            }
        );
        assert_eq!(
            style.handle.background,
            Background::Color(colors.primary.color)
        );
    }
}
