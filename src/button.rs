use iced_widget::button::{Catalog, Status, Style, StyleFn};
use iced_widget::core::{Background, Border, Color, border};

use crate::Theme;
use crate::tokens;
use crate::tokens::component::button::ElevationLevels;
use crate::utils::{
    HOVERED_LAYER_OPACITY, PRESSED_LAYER_OPACITY, disabled_container, disabled_text, mix,
    shadow_from_level, state_layer,
};

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(filled)
    }

    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style {
        class(self, status)
    }
}

pub fn styled(
    background: Color,
    foreground: Color,
    disabled: Color,
    shadow_color: Color,
    elevation_level: u8,
    status: Status,
) -> Style {
    styled_with_elevations(
        background,
        foreground,
        disabled,
        shadow_color,
        ElevationLevels {
            active: elevation_level,
            hovered: elevation_level + 1,
            pressed: elevation_level,
            disabled: 0,
        },
        status,
    )
}

fn styled_with_elevations(
    background: Color,
    foreground: Color,
    disabled: Color,
    shadow_color: Color,
    elevations: ElevationLevels,
    status: Status,
) -> Style {
    styled_with_elevations_and_shape(
        background,
        foreground,
        disabled,
        shadow_color,
        elevations,
        tokens::component::button::CONTAINER_SHAPE,
        status,
    )
}

fn styled_with_elevations_and_shape(
    background: Color,
    foreground: Color,
    disabled: Color,
    shadow_color: Color,
    elevations: ElevationLevels,
    shape: f32,
    status: Status,
) -> Style {
    let active = Style {
        background: Some(Background::Color(background)),
        text_color: foreground,
        border: border::rounded(shape),
        shadow: shadow_from_level(elevations.active, shadow_color),
        snap: cfg!(feature = "crisp"),
    };

    match status {
        Status::Active => active,
        Status::Pressed => Style {
            background: Some(Background::Color(mix(
                background,
                foreground,
                PRESSED_LAYER_OPACITY,
            ))),
            shadow: shadow_from_level(elevations.pressed, shadow_color),
            ..active
        },
        Status::Hovered => Style {
            background: Some(Background::Color(mix(
                background,
                foreground,
                HOVERED_LAYER_OPACITY,
            ))),
            shadow: shadow_from_level(elevations.hovered, shadow_color),
            ..active
        },
        Status::Disabled => Style {
            background: Some(Background::Color(disabled_container(disabled))),
            text_color: disabled_text(disabled),
            border: border::rounded(shape),
            shadow: shadow_from_level(elevations.disabled, shadow_color),
            ..Default::default()
        },
    }
}

pub fn elevated(theme: &Theme, status: Status) -> Style {
    let surface = theme.colors().surface;

    let foreground = theme.colors().primary.color;
    let background = surface.container.low;
    let disabled = surface.text;

    let shadow_color = theme.colors().shadow;

    styled_with_elevations(
        background,
        foreground,
        disabled,
        shadow_color,
        tokens::component::button::ELEVATED_ELEVATION,
        status,
    )
}

pub fn filled(theme: &Theme, status: Status) -> Style {
    let primary = theme.colors().primary;

    let foreground = primary.text;
    let background = primary.color;
    let disabled = theme.colors().surface.text;

    let shadow_color = theme.colors().shadow;

    styled_with_elevations(
        background,
        foreground,
        disabled,
        shadow_color,
        tokens::component::button::FILLED_ELEVATION,
        status,
    )
}

pub fn filled_tonal(theme: &Theme, status: Status) -> Style {
    let secondary = theme.colors().secondary;

    let foreground = secondary.container_text;
    let background = secondary.container;
    let disabled = theme.colors().surface.text;
    let shadow_color = theme.colors().shadow;

    styled_with_elevations(
        background,
        foreground,
        disabled,
        shadow_color,
        tokens::component::button::FILLED_TONAL_ELEVATION,
        status,
    )
}

pub fn outlined(theme: &Theme, status: Status) -> Style {
    let foreground = theme.colors().primary.color;
    let background = Color::TRANSPARENT;
    let disabled = theme.colors().surface.text;

    let outline = theme.colors().outline.color;

    let border = match status {
        Status::Active | Status::Pressed | Status::Hovered => Border {
            color: outline,
            width: tokens::component::button::OUTLINED_OUTLINE_WIDTH,
            radius: tokens::component::button::CONTAINER_SHAPE.into(),
        },
        Status::Disabled => Border {
            color: disabled_container(disabled),
            width: tokens::component::button::OUTLINED_OUTLINE_WIDTH,
            radius: tokens::component::button::CONTAINER_SHAPE.into(),
        },
    };

    let mut style = styled_with_elevations(
        background,
        foreground,
        disabled,
        Color::TRANSPARENT,
        tokens::component::button::FLAT_ELEVATION,
        status,
    );

    if matches!(status, Status::Disabled) {
        style.background = None;
    }

    Style { border, ..style }
}

pub fn text(theme: &Theme, status: Status) -> Style {
    let foreground = theme.colors().primary.color;
    let background = Color::TRANSPARENT;
    let disabled = theme.colors().surface.text;

    let style = styled_with_elevations(
        background,
        foreground,
        disabled,
        Color::TRANSPARENT,
        tokens::component::button::FLAT_ELEVATION,
        status,
    );

    match status {
        Status::Hovered | Status::Pressed => style,
        Status::Active | Status::Disabled => Style {
            background: None,
            ..style
        },
    }
}

fn fab_style(theme: &Theme, background: Color, foreground: Color, status: Status) -> Style {
    styled_with_elevations_and_shape(
        background,
        foreground,
        theme.colors().surface.text,
        theme.colors().shadow,
        tokens::component::fab::ELEVATION,
        tokens::component::fab::CONTAINER_SHAPE,
        status,
    )
}

pub fn fab_primary(theme: &Theme, status: Status) -> Style {
    let primary = theme.colors().primary;

    fab_style(theme, primary.container, primary.container_text, status)
}

pub fn fab_secondary(theme: &Theme, status: Status) -> Style {
    let secondary = theme.colors().secondary;

    fab_style(theme, secondary.container, secondary.container_text, status)
}

pub fn fab_surface(theme: &Theme, status: Status) -> Style {
    let colors = theme.colors();

    fab_style(
        theme,
        colors.surface.container.high,
        colors.primary.color,
        status,
    )
}

pub fn icon(theme: &Theme, status: Status) -> Style {
    let surface = theme.colors().surface;

    let active = Style {
        background: None,
        text_color: surface.text_variant,
        border: border::rounded(tokens::component::icon_button::CONTAINER_SHAPE),
        shadow: shadow_from_level(0, Color::TRANSPARENT),
        snap: cfg!(feature = "crisp"),
    };

    match status {
        Status::Active => active,
        Status::Hovered => Style {
            background: Some(Background::Color(mix(
                Color::TRANSPARENT,
                surface.text_variant,
                HOVERED_LAYER_OPACITY,
            ))),
            ..active
        },
        Status::Pressed => Style {
            background: Some(Background::Color(mix(
                Color::TRANSPARENT,
                surface.text_variant,
                PRESSED_LAYER_OPACITY,
            ))),
            ..active
        },
        Status::Disabled => Style {
            text_color: Color {
                a: tokens::component::icon_button::DISABLED_ICON_OPACITY,
                ..surface.text
            },
            ..active
        },
    }
}

pub fn filled_icon(theme: &Theme, status: Status) -> Style {
    let colors = theme.colors();

    styled_with_elevations_and_shape(
        colors.primary.color,
        colors.primary.text,
        colors.surface.text,
        Color::TRANSPARENT,
        tokens::component::button::FLAT_ELEVATION,
        tokens::component::icon_button::CONTAINER_SHAPE,
        status,
    )
}

pub fn filled_tonal_icon(theme: &Theme, status: Status) -> Style {
    let colors = theme.colors();

    styled_with_elevations_and_shape(
        colors.secondary.container,
        colors.secondary.container_text,
        colors.surface.text,
        Color::TRANSPARENT,
        tokens::component::button::FLAT_ELEVATION,
        tokens::component::icon_button::CONTAINER_SHAPE,
        status,
    )
}

pub fn outlined_icon(theme: &Theme, status: Status) -> Style {
    let colors = theme.colors();
    let surface = colors.surface;

    let active = Style {
        background: None,
        text_color: surface.text_variant,
        border: Border {
            color: colors.outline.color,
            width: tokens::component::icon_button::OUTLINED_OUTLINE_WIDTH,
            radius: tokens::component::icon_button::CONTAINER_SHAPE.into(),
        },
        shadow: shadow_from_level(0, Color::TRANSPARENT),
        snap: cfg!(feature = "crisp"),
    };

    match status {
        Status::Active => active,
        Status::Hovered => Style {
            background: Some(Background::Color(mix(
                Color::TRANSPARENT,
                surface.text_variant,
                HOVERED_LAYER_OPACITY,
            ))),
            ..active
        },
        Status::Pressed => Style {
            background: Some(Background::Color(mix(
                Color::TRANSPARENT,
                surface.text,
                PRESSED_LAYER_OPACITY,
            ))),
            text_color: surface.text,
            ..active
        },
        Status::Disabled => Style {
            text_color: Color {
                a: tokens::component::icon_button::DISABLED_ICON_OPACITY,
                ..surface.text
            },
            border: Border {
                color: Color {
                    a: tokens::component::icon_button::OUTLINED_DISABLED_OUTLINE_OPACITY,
                    ..surface.text
                },
                ..active.border
            },
            ..active
        },
    }
}

#[derive(Debug, Clone, Copy)]
struct ChipSpec {
    background: Option<Color>,
    foreground: Color,
    outline: Option<Color>,
    disabled_background: Option<Color>,
    disabled_outline: Option<Color>,
    hover_layer: Color,
    pressed_layer: Color,
    elevations: ElevationLevels,
    shadow_color: Color,
}

fn chip_style(spec: ChipSpec, status: Status) -> Style {
    let border = Border {
        color: spec.outline.unwrap_or(Color::TRANSPARENT),
        width: spec
            .outline
            .map_or(tokens::component::chip::SELECTED_OUTLINE_WIDTH, |_| {
                tokens::component::chip::OUTLINE_WIDTH
            }),
        radius: tokens::component::chip::CONTAINER_SHAPE.into(),
    };

    let active = Style {
        background: spec.background.map(Background::Color),
        text_color: spec.foreground,
        border,
        shadow: shadow_from_level(spec.elevations.active, spec.shadow_color),
        snap: cfg!(feature = "crisp"),
    };

    match status {
        Status::Active => active,
        Status::Hovered => Style {
            background: Some(Background::Color(chip_state_background(
                spec.background,
                spec.hover_layer,
                HOVERED_LAYER_OPACITY,
            ))),
            shadow: shadow_from_level(spec.elevations.hovered, spec.shadow_color),
            ..active
        },
        Status::Pressed => Style {
            background: Some(Background::Color(chip_state_background(
                spec.background,
                spec.pressed_layer,
                PRESSED_LAYER_OPACITY,
            ))),
            shadow: shadow_from_level(spec.elevations.pressed, spec.shadow_color),
            ..active
        },
        Status::Disabled => Style {
            background: spec.disabled_background.map(Background::Color),
            text_color: Color {
                a: tokens::component::chip::DISABLED_LABEL_TEXT_OPACITY,
                ..spec.foreground
            },
            border: Border {
                color: spec.disabled_outline.unwrap_or(Color::TRANSPARENT),
                ..border
            },
            shadow: shadow_from_level(spec.elevations.disabled, spec.shadow_color),
            snap: cfg!(feature = "crisp"),
        },
    }
}

fn chip_state_background(background: Option<Color>, layer: Color, opacity: f32) -> Color {
    background.map_or_else(
        || state_layer(layer, opacity),
        |background| mix(background, layer, opacity),
    )
}

fn outlined_chip_spec(
    foreground: Color,
    outline: Color,
    disabled_color: Color,
    pressed_layer: Color,
) -> ChipSpec {
    ChipSpec {
        background: None,
        foreground,
        outline: Some(outline),
        disabled_background: None,
        disabled_outline: Some(Color {
            a: tokens::component::chip::DISABLED_OUTLINE_OPACITY,
            ..disabled_color
        }),
        hover_layer: foreground,
        pressed_layer,
        elevations: tokens::component::chip::FLAT_ELEVATION,
        shadow_color: Color::TRANSPARENT,
    }
}

fn elevated_chip_spec(
    background: Color,
    foreground: Color,
    disabled_color: Color,
    shadow_color: Color,
) -> ChipSpec {
    ChipSpec {
        background: Some(background),
        foreground,
        outline: None,
        disabled_background: Some(Color {
            a: tokens::component::chip::DISABLED_CONTAINER_OPACITY,
            ..disabled_color
        }),
        disabled_outline: None,
        hover_layer: foreground,
        pressed_layer: foreground,
        elevations: tokens::component::chip::ELEVATED_ELEVATION,
        shadow_color,
    }
}

pub fn assist_chip(theme: &Theme, status: Status) -> Style {
    let colors = theme.colors();

    chip_style(
        outlined_chip_spec(
            colors.surface.text,
            colors.outline.color,
            colors.surface.text,
            colors.surface.text,
        ),
        status,
    )
}

pub fn elevated_assist_chip(theme: &Theme, status: Status) -> Style {
    let colors = theme.colors();

    chip_style(
        elevated_chip_spec(
            colors.surface.container.low,
            colors.surface.text,
            colors.surface.text,
            colors.shadow,
        ),
        status,
    )
}

pub fn suggestion_chip(theme: &Theme, status: Status) -> Style {
    let colors = theme.colors();

    chip_style(
        outlined_chip_spec(
            colors.surface.text_variant,
            colors.outline.color,
            colors.surface.text,
            colors.surface.text_variant,
        ),
        status,
    )
}

pub fn elevated_suggestion_chip(theme: &Theme, status: Status) -> Style {
    let colors = theme.colors();

    chip_style(
        elevated_chip_spec(
            colors.surface.container.low,
            colors.surface.text_variant,
            colors.surface.text,
            colors.shadow,
        ),
        status,
    )
}

pub fn filter_chip(theme: &Theme, status: Status) -> Style {
    let colors = theme.colors();

    chip_style(
        outlined_chip_spec(
            colors.surface.text_variant,
            colors.outline.color,
            colors.surface.text,
            colors.secondary.container_text,
        ),
        status,
    )
}

pub fn selected_filter_chip(theme: &Theme, status: Status) -> Style {
    let colors = theme.colors();

    chip_style(
        ChipSpec {
            background: Some(colors.secondary.container),
            foreground: colors.secondary.container_text,
            outline: None,
            disabled_background: Some(Color {
                a: tokens::component::chip::DISABLED_CONTAINER_OPACITY,
                ..colors.surface.text
            }),
            disabled_outline: None,
            hover_layer: colors.secondary.container_text,
            pressed_layer: colors.surface.text_variant,
            elevations: tokens::component::chip::SELECTED_FLAT_ELEVATION,
            shadow_color: colors.shadow,
        },
        status,
    )
}

pub fn input_chip(theme: &Theme, status: Status) -> Style {
    let colors = theme.colors();

    chip_style(
        outlined_chip_spec(
            colors.surface.text_variant,
            colors.outline.color,
            colors.surface.text,
            colors.surface.text_variant,
        ),
        status,
    )
}

pub fn selected_input_chip(theme: &Theme, status: Status) -> Style {
    let colors = theme.colors();

    chip_style(
        ChipSpec {
            background: Some(colors.secondary.container),
            foreground: colors.secondary.container_text,
            outline: None,
            disabled_background: Some(Color {
                a: tokens::component::chip::DISABLED_CONTAINER_OPACITY,
                ..colors.surface.text
            }),
            disabled_outline: None,
            hover_layer: colors.secondary.container_text,
            pressed_layer: colors.secondary.container_text,
            elevations: tokens::component::chip::FLAT_ELEVATION,
            shadow_color: Color::TRANSPARENT,
        },
        status,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filled_button_uses_m3_elevation_tokens() {
        let theme = Theme::Light;

        let active = filled(&theme, Status::Active);
        assert_eq!(active.shadow.offset.y, 0.0);
        assert_eq!(active.shadow.blur_radius, 0.0);

        let hovered = filled(&theme, Status::Hovered);
        assert_eq!(hovered.shadow.offset.y, 1.0);
        assert_eq!(hovered.shadow.blur_radius, 3.0);

        let pressed = filled(&theme, Status::Pressed);
        assert_eq!(pressed.shadow.offset.y, 0.0);
        assert_eq!(pressed.shadow.blur_radius, 0.0);
    }

    #[test]
    fn elevated_button_uses_m3_elevation_tokens() {
        let theme = Theme::Light;

        let active = elevated(&theme, Status::Active);
        assert_eq!(active.shadow.offset.y, 1.0);
        assert_eq!(active.shadow.blur_radius, 3.0);

        let hovered = elevated(&theme, Status::Hovered);
        assert_eq!(hovered.shadow.offset.y, 2.0);
        assert_eq!(hovered.shadow.blur_radius, 6.0);

        let pressed = elevated(&theme, Status::Pressed);
        assert_eq!(pressed.shadow.offset.y, 1.0);
        assert_eq!(pressed.shadow.blur_radius, 3.0);
    }

    #[test]
    fn outlined_disabled_button_has_no_container_fill() {
        let theme = Theme::Light;
        let style = outlined(&theme, Status::Disabled);

        assert_eq!(style.background, None);
        assert_eq!(
            style.border.color.a,
            tokens::state::DISABLED_CONTAINER_OPACITY
        );
        assert_eq!(
            style.text_color.a,
            tokens::state::DISABLED_LABEL_TEXT_OPACITY
        );
    }

    #[test]
    fn fab_primary_uses_m3_container_shape_and_elevation_tokens() {
        let theme = Theme::Light;
        let colors = theme.colors();

        let active = fab_primary(&theme, Status::Active);
        assert_eq!(
            active.background,
            Some(Background::Color(colors.primary.container))
        );
        assert_eq!(active.text_color, colors.primary.container_text);
        assert_eq!(
            active.border.radius.top_left,
            tokens::component::fab::CONTAINER_SHAPE
        );
        assert_eq!(active.shadow.offset.y, 4.0);
        assert_eq!(active.shadow.blur_radius, 8.0);

        let hovered = fab_primary(&theme, Status::Hovered);
        assert_eq!(hovered.shadow.offset.y, 6.0);
        assert_eq!(hovered.shadow.blur_radius, 10.0);

        let pressed = fab_primary(&theme, Status::Pressed);
        assert_eq!(pressed.shadow.offset.y, 4.0);
        assert_eq!(pressed.shadow.blur_radius, 8.0);
    }

    #[test]
    fn fab_surface_uses_m3_surface_container_high_and_primary_icon() {
        let theme = Theme::Light;
        let colors = theme.colors();
        let style = fab_surface(&theme, Status::Active);

        assert_eq!(
            style.background,
            Some(Background::Color(colors.surface.container.high))
        );
        assert_eq!(style.text_color, colors.primary.color);
    }

    #[test]
    fn standard_icon_button_uses_m3_state_layer_tokens() {
        let theme = Theme::Light;
        let colors = theme.colors();

        let active = icon(&theme, Status::Active);
        assert_eq!(active.background, None);
        assert_eq!(active.text_color, colors.surface.text_variant);
        assert_eq!(
            active.border.radius.top_left,
            tokens::component::icon_button::CONTAINER_SHAPE
        );

        let hovered = icon(&theme, Status::Hovered);
        assert_eq!(
            hovered.background,
            Some(Background::Color(mix(
                Color::TRANSPARENT,
                colors.surface.text_variant,
                HOVERED_LAYER_OPACITY
            )))
        );

        let disabled = icon(&theme, Status::Disabled);
        assert_eq!(
            disabled.text_color.a,
            tokens::component::icon_button::DISABLED_ICON_OPACITY
        );
        assert_eq!(disabled.background, None);
    }

    #[test]
    fn filled_icon_buttons_use_m3_container_tokens() {
        let theme = Theme::Light;
        let colors = theme.colors();

        let filled = filled_icon(&theme, Status::Active);
        assert_eq!(
            filled.background,
            Some(Background::Color(colors.primary.color))
        );
        assert_eq!(filled.text_color, colors.primary.text);
        assert_eq!(
            filled.border.radius.top_left,
            tokens::component::icon_button::CONTAINER_SHAPE
        );

        let tonal = filled_tonal_icon(&theme, Status::Active);
        assert_eq!(
            tonal.background,
            Some(Background::Color(colors.secondary.container))
        );
        assert_eq!(tonal.text_color, colors.secondary.container_text);
    }

    #[test]
    fn chip_helpers_use_m3_chip_tokens() {
        let theme = Theme::Light;
        let colors = theme.colors();

        let assist = assist_chip(&theme, Status::Active);
        assert_eq!(assist.background, None);
        assert_eq!(assist.text_color, colors.surface.text);
        assert_eq!(assist.border.color, colors.outline.color);
        assert_eq!(assist.border.width, tokens::component::chip::OUTLINE_WIDTH);
        assert_eq!(
            assist.border.radius.top_left,
            tokens::component::chip::CONTAINER_SHAPE
        );

        let elevated = elevated_assist_chip(&theme, Status::Active);
        assert_eq!(
            elevated.background,
            Some(Background::Color(colors.surface.container.low))
        );
        assert_eq!(elevated.shadow.offset.y, 1.0);
        assert_eq!(elevated.shadow.blur_radius, 3.0);

        let hovered_elevated = elevated_assist_chip(&theme, Status::Hovered);
        assert_eq!(hovered_elevated.shadow.offset.y, 2.0);
        assert_eq!(hovered_elevated.shadow.blur_radius, 6.0);

        let suggestion = suggestion_chip(&theme, Status::Active);
        assert_eq!(suggestion.text_color, colors.surface.text_variant);

        let disabled = assist_chip(&theme, Status::Disabled);
        assert_eq!(
            disabled.text_color.a,
            tokens::component::chip::DISABLED_LABEL_TEXT_OPACITY
        );
        assert_eq!(
            disabled.border.color.a,
            tokens::component::chip::DISABLED_OUTLINE_OPACITY
        );
    }

    #[test]
    fn selectable_chip_helpers_use_m3_selected_and_unselected_tokens() {
        let theme = Theme::Light;
        let colors = theme.colors();

        let unselected = filter_chip(&theme, Status::Active);
        assert_eq!(unselected.background, None);
        assert_eq!(unselected.text_color, colors.surface.text_variant);
        assert_eq!(unselected.border.color, colors.outline.color);

        let selected = selected_filter_chip(&theme, Status::Active);
        assert_eq!(
            selected.background,
            Some(Background::Color(colors.secondary.container))
        );
        assert_eq!(selected.text_color, colors.secondary.container_text);
        assert_eq!(
            selected.border.width,
            tokens::component::chip::SELECTED_OUTLINE_WIDTH
        );

        let hovered = selected_filter_chip(&theme, Status::Hovered);
        assert_eq!(hovered.shadow.offset.y, 1.0);
        assert_eq!(hovered.shadow.blur_radius, 3.0);

        let input = input_chip(&theme, Status::Active);
        assert_eq!(input.text_color, colors.surface.text_variant);

        let selected_input = selected_input_chip(&theme, Status::Active);
        assert_eq!(
            selected_input.background,
            Some(Background::Color(colors.secondary.container))
        );
        assert_eq!(selected_input.shadow.blur_radius, 0.0);
    }

    #[test]
    fn outlined_icon_button_uses_m3_outline_and_pressed_icon_tokens() {
        let theme = Theme::Light;
        let colors = theme.colors();

        let active = outlined_icon(&theme, Status::Active);
        assert_eq!(active.border.color, colors.outline.color);
        assert_eq!(
            active.border.width,
            tokens::component::icon_button::OUTLINED_OUTLINE_WIDTH
        );
        assert_eq!(active.text_color, colors.surface.text_variant);

        let pressed = outlined_icon(&theme, Status::Pressed);
        assert_eq!(pressed.text_color, colors.surface.text);

        let disabled = outlined_icon(&theme, Status::Disabled);
        assert_eq!(
            disabled.border.color.a,
            tokens::component::icon_button::OUTLINED_DISABLED_OUTLINE_OPACITY
        );
        assert_eq!(
            disabled.text_color.a,
            tokens::component::icon_button::DISABLED_ICON_OPACITY
        );
    }
}
