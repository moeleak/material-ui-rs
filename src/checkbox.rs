use iced_widget::checkbox::{Catalog, Status, Style, StyleFn};
use iced_widget::core::{Background, Border, Color, border};

use super::Theme;
use crate::tokens;
use crate::utils::{HOVERED_LAYER_OPACITY, state_layer};

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(default)
    }

    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style {
        class(self, status)
    }
}

pub fn styled(
    background_color: Color,
    background_unchecked: Option<Color>,
    icon_color: Color,
    border_color: Color,
    text_color: Option<Color>,
    is_checked: bool,
) -> Style {
    Style {
        background: Background::Color(if is_checked {
            background_color
        } else {
            background_unchecked.unwrap_or(Color::TRANSPARENT)
        }),
        icon_color,
        border: if is_checked {
            border::rounded(tokens::component::checkbox::CONTAINER_SHAPE)
        } else {
            Border {
                color: border_color,
                width: tokens::component::checkbox::UNSELECTED_OUTLINE_WIDTH,
                radius: tokens::component::checkbox::CONTAINER_SHAPE.into(),
            }
        },
        text_color,
    }
}

pub fn default(theme: &Theme, status: Status) -> Style {
    let surface = theme.colors().surface;
    let primary = theme.colors().primary;

    match status {
        Status::Active { is_checked } => styled(
            primary.color,
            None,
            primary.text,
            surface.text_variant,
            Some(surface.text),
            is_checked,
        ),
        Status::Hovered { is_checked } => styled(
            primary.color,
            Some(state_layer(surface.text, HOVERED_LAYER_OPACITY)),
            primary.text,
            surface.text,
            Some(surface.text),
            is_checked,
        ),
        Status::Disabled { is_checked } => styled(
            state_layer(
                surface.text,
                tokens::component::checkbox::SELECTED_DISABLED_CONTAINER_OPACITY,
            ),
            None,
            surface.color,
            state_layer(
                surface.text,
                tokens::component::checkbox::UNSELECTED_DISABLED_CONTAINER_OPACITY,
            ),
            Some(state_layer(
                surface.text,
                tokens::state::DISABLED_LABEL_TEXT_OPACITY,
            )),
            is_checked,
        ),
    }
}

pub fn error(theme: &Theme, status: Status) -> Style {
    let surface = theme.colors().surface;
    let error = theme.colors().error;

    match status {
        Status::Active { is_checked } => styled(
            error.color,
            None,
            error.text,
            error.color,
            Some(error.color),
            is_checked,
        ),
        Status::Hovered { is_checked } => styled(
            error.color,
            Some(state_layer(error.color, HOVERED_LAYER_OPACITY)),
            error.text,
            error.color,
            Some(error.color),
            is_checked,
        ),
        Status::Disabled { is_checked } => styled(
            state_layer(
                surface.text,
                tokens::component::checkbox::SELECTED_DISABLED_CONTAINER_OPACITY,
            ),
            None,
            surface.color,
            state_layer(
                surface.text,
                tokens::component::checkbox::UNSELECTED_DISABLED_CONTAINER_OPACITY,
            ),
            Some(state_layer(
                surface.text,
                tokens::state::DISABLED_LABEL_TEXT_OPACITY,
            )),
            is_checked,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn background_color(background: Background) -> Color {
        match background {
            Background::Color(color) => color,
            Background::Gradient(_) => panic!("expected solid color"),
        }
    }

    #[test]
    fn disabled_checkbox_uses_m3_component_opacity_tokens() {
        let checked = default(&Theme::Light, Status::Disabled { is_checked: true });
        assert_eq!(
            background_color(checked.background).a,
            tokens::component::checkbox::SELECTED_DISABLED_CONTAINER_OPACITY
        );
        assert_eq!(
            checked.text_color.unwrap().a,
            tokens::state::DISABLED_LABEL_TEXT_OPACITY
        );

        let unchecked = default(&Theme::Light, Status::Disabled { is_checked: false });
        assert_eq!(
            unchecked.border.color.a,
            tokens::component::checkbox::UNSELECTED_DISABLED_CONTAINER_OPACITY
        );
        assert_eq!(
            unchecked.border.width,
            tokens::component::checkbox::UNSELECTED_DISABLED_OUTLINE_WIDTH
        );
        assert_eq!(
            unchecked.text_color.unwrap().a,
            tokens::state::DISABLED_LABEL_TEXT_OPACITY
        );
    }
}
