fn date_picker_container_style(theme: &Theme) -> iced_widget::container::Style {
    let colors = theme.colors();

    iced_widget::container::Style {
        background: Some(Background::Color(colors.surface.container.high)),
        text_color: Some(colors.surface.text),
        border: border::rounded(tokens::component::date_picker::CONTAINER_SHAPE),
        shadow: shadow_from_level(
            tokens::component::date_picker::CONTAINER_ELEVATION_LEVEL,
            colors.shadow,
        ),
        ..Default::default()
    }
}

fn date_header_style(theme: &Theme) -> iced_widget::container::Style {
    let colors = theme.colors();

    iced_widget::container::Style {
        background: Some(Background::Color(colors.surface.container.high)),
        text_color: Some(colors.surface.text_variant),
        ..Default::default()
    }
}

fn date_input_panel_style(theme: &Theme, content_alpha: f32) -> iced_widget::container::Style {
    let colors = theme.colors();

    iced_widget::container::Style {
        background: None,
        text_color: Some(alpha_color(colors.surface.text, content_alpha)),
        ..Default::default()
    }
}

fn date_range_scrollable_style(
    theme: &Theme,
    status: iced_widget::scrollable::Status,
    content_alpha: f32,
) -> iced_widget::scrollable::Style {
    let content_alpha = content_alpha.clamp(0.0, 1.0);
    let mut style = crate::style::scrollable::default(theme, status);

    style.container.background = None;
    style.container.text_color = style
        .container
        .text_color
        .map(|color| alpha_color(color, content_alpha));
    style.container.border.color = alpha_color(style.container.border.color, content_alpha);
    style.container.shadow.color = alpha_color(style.container.shadow.color, content_alpha);
    style.vertical_rail = scrollable_rail_alpha(style.vertical_rail, content_alpha);
    style.horizontal_rail = scrollable_rail_alpha(style.horizontal_rail, content_alpha);
    style.gap = style
        .gap
        .map(|background| background.scale_alpha(content_alpha));
    style.auto_scroll.background = style.auto_scroll.background.scale_alpha(content_alpha);
    style.auto_scroll.border.color =
        alpha_color(style.auto_scroll.border.color, content_alpha);
    style.auto_scroll.shadow.color =
        alpha_color(style.auto_scroll.shadow.color, content_alpha);
    style.auto_scroll.icon = alpha_color(style.auto_scroll.icon, content_alpha);

    style
}

fn scrollable_rail_alpha(
    mut rail: iced_widget::scrollable::Rail,
    content_alpha: f32,
) -> iced_widget::scrollable::Rail {
    rail.background = rail
        .background
        .map(|background| background.scale_alpha(content_alpha));
    rail.border.color = alpha_color(rail.border.color, content_alpha);
    rail.scroller.background = rail.scroller.background.scale_alpha(content_alpha);
    rail.scroller.border.color = alpha_color(rail.scroller.border.color, content_alpha);
    rail
}

fn year_picker_panel_style(theme: &Theme, content_alpha: f32) -> iced_widget::container::Style {
    let colors = theme.colors();
    let content_alpha = content_alpha.clamp(0.0, 1.0);

    iced_widget::container::Style {
        background: (content_alpha > 0.0).then_some(Background::Color(alpha_color(
            colors.surface.container.high,
            content_alpha,
        ))),
        text_color: Some(alpha_color(colors.surface.text, content_alpha)),
        ..Default::default()
    }
}

fn time_picker_container_style(theme: &Theme) -> iced_widget::container::Style {
    let colors = theme.colors();

    iced_widget::container::Style {
        background: Some(Background::Color(colors.surface.container.high)),
        text_color: Some(colors.surface.text),
        border: border::rounded(tokens::component::time_picker::CONTAINER_SHAPE),
        shadow: shadow_from_level(
            tokens::component::time_picker::CONTAINER_ELEVATION_LEVEL,
            colors.shadow,
        ),
        ..Default::default()
    }
}

fn time_picker_dialog_container_style(theme: &Theme) -> iced_widget::container::Style {
    let colors = theme.colors();

    iced_widget::container::Style {
        background: Some(Background::Color(colors.surface.container.high)),
        text_color: Some(colors.surface.text),
        border: border::rounded(tokens::component::time_picker_dialog::CONTAINER_SHAPE),
        shadow: shadow_from_level(
            tokens::component::time_picker_dialog::CONTAINER_ELEVATION_LEVEL,
            colors.shadow,
        ),
        ..Default::default()
    }
}

fn rich_time_picker_dialog_container_style(theme: &Theme) -> iced_widget::container::Style {
    let colors = theme.colors();

    iced_widget::container::Style {
        background: Some(Background::Color(colors.surface.container.base)),
        text_color: Some(colors.surface.text),
        border: border::rounded(tokens::component::time_picker_dialog::CONTAINER_SHAPE),
        shadow: shadow_from_level(
            tokens::component::time_picker_dialog::CONTAINER_ELEVATION_LEVEL,
            colors.shadow,
        ),
        ..Default::default()
    }
}

fn day_button_style(
    theme: &Theme,
    status: Status,
    selected: bool,
    selected_progress: f32,
    enabled: bool,
    range_position: DateRangePosition,
    content_alpha: f32,
) -> Style {
    let colors = theme.colors();
    let mut background = Color::TRANSPARENT;
    let text_color = Color::TRANSPARENT;

    if matches!(status, Status::Hovered | Status::Pressed) && enabled {
        let layer = if selected || selected_progress > 0.0 {
            colors.primary.text
        } else if range_position.draws_range_background() {
            colors.secondary.container_text
        } else {
            colors.surface.text
        };
        background = mix(background, layer, 0.08);
    }

    if matches!(status, Status::Disabled) {
        background = Color::TRANSPARENT;
    }

    Style {
        background: (background.a > 0.0)
            .then_some(Background::Color(alpha_color(background, content_alpha))),
        text_color,
        border: Border {
            radius: tokens::component::date_picker::DATE_CONTAINER_SHAPE.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

fn year_button_style(
    theme: &Theme,
    status: Status,
    selected: bool,
    current_year: bool,
    enabled: bool,
    content_alpha: f32,
) -> Style {
    let colors = theme.colors();
    let disabled = colors.surface.text;
    let mut background = if selected {
        colors.primary.color
    } else {
        Color::TRANSPARENT
    };
    let mut text_color = if selected {
        colors.primary.text
    } else if current_year {
        colors.primary.color
    } else {
        colors.surface.text_variant
    };

    if matches!(status, Status::Hovered | Status::Pressed) {
        let layer = if selected {
            colors.primary.text
        } else {
            colors.surface.text
        };
        background = mix(background, layer, 0.08);
    }

    if matches!(status, Status::Disabled) || !enabled {
        background = if selected {
            disabled_container(disabled)
        } else {
            Color::TRANSPARENT
        };
        text_color = disabled_text(disabled);
    }

    Style {
        background: (background.a > 0.0)
            .then_some(Background::Color(alpha_color(background, content_alpha))),
        text_color: alpha_color(text_color, content_alpha),
        border: Border {
            color: if current_year && !selected {
                alpha_color(colors.primary.color, content_alpha)
            } else {
                Color::TRANSPARENT
            },
            width: if current_year && !selected {
                tokens::component::date_picker::DATE_TODAY_OUTLINE_WIDTH
            } else {
                0.0
            },
            radius: tokens::shape::CORNER_FULL.into(),
        },
        ..Default::default()
    }
}

fn time_selector_style(theme: &Theme, status: Status, selected_progress: f32) -> Style {
    let colors = theme.colors();
    let selected_progress = selected_progress.clamp(0.0, 1.0);
    let mut background = mix(
        colors.surface.container.highest,
        colors.primary.container,
        selected_progress,
    );
    let text_color = mix(
        colors.surface.text,
        colors.primary.container_text,
        selected_progress,
    );

    if matches!(status, Status::Hovered | Status::Pressed) {
        background = mix(background, text_color, 0.08);
    }

    Style {
        background: Some(Background::Color(background)),
        text_color,
        border: border::rounded(tokens::component::time_picker::TIME_SELECTOR_SHAPE),
        ..Default::default()
    }
}

fn time_scroll_item_style(theme: &Theme, status: Status, selected: bool) -> Style {
    let colors = theme.colors();
    let mut background = Color::TRANSPARENT;
    let text_color = if selected {
        colors.primary.container_text
    } else {
        colors.surface.text
    };

    if matches!(status, Status::Hovered | Status::Pressed) {
        let layer = if selected {
            colors.primary.container_text
        } else {
            colors.surface.text
        };
        background = mix(background, layer, 0.08);
    }

    Style {
        background: (background.a > 0.0).then_some(Background::Color(background)),
        text_color,
        border: Border::default(),
        ..Default::default()
    }
}

fn time_scroll_field_container_style(theme: &Theme) -> iced_widget::container::Style {
    let colors = theme.colors();

    iced_widget::container::Style {
        background: Some(Background::Color(colors.surface.container.highest)),
        text_color: Some(colors.surface.text),
        border: border::rounded(tokens::component::time_picker::TIME_SELECTOR_SHAPE),
        ..Default::default()
    }
}

fn time_scroll_selection_layer_style(theme: &Theme) -> iced_widget::container::Style {
    let colors = theme.colors();

    iced_widget::container::Style {
        background: Some(Background::Color(colors.primary.container)),
        text_color: Some(colors.primary.container_text),
        border: border::rounded(tokens::component::time_picker::TIME_SELECTOR_SHAPE),
        ..Default::default()
    }
}

fn time_input_selector_style(theme: &Theme, status: Status, valid: bool) -> Style {
    let colors = theme.colors();
    let mut background = colors.surface.container.highest;
    let text_color = if valid {
        colors.surface.text
    } else {
        colors.error.color
    };

    if matches!(status, Status::Hovered | Status::Pressed) {
        background = mix(background, text_color, 0.08);
    }

    Style {
        background: Some(Background::Color(background)),
        text_color,
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: tokens::component::time_input::TIME_FIELD_CONTAINER_SHAPE.into(),
        },
        ..Default::default()
    }
}

fn period_toggle_container_style(theme: &Theme) -> iced_widget::container::Style {
    let colors = theme.colors();

    iced_widget::container::Style {
        border: Border {
            color: colors.outline.color,
            width: tokens::component::time_picker::PERIOD_SELECTOR_OUTLINE_WIDTH,
            radius: tokens::component::time_picker::PERIOD_SELECTOR_SHAPE.into(),
        },
        ..Default::default()
    }
}

fn period_toggle_separator_style(theme: &Theme) -> iced_widget::container::Style {
    iced_widget::container::Style {
        background: Some(Background::Color(theme.colors().outline.color)),
        ..Default::default()
    }
}

fn period_button_style(
    theme: &Theme,
    status: Status,
    selected_progress: f32,
    radius: border::Radius,
) -> Style {
    let colors = theme.colors();
    let selected_progress = selected_progress.clamp(0.0, 1.0);
    let selected_container = colors.tertiary.container;
    let selected_content = colors.tertiary.container_text;
    let mut background = if selected_progress > 0.0 {
        Color {
            a: selected_container.a * selected_progress,
            ..selected_container
        }
    } else {
        Color::TRANSPARENT
    };
    let text_color = mix(
        colors.surface.text_variant,
        selected_content,
        selected_progress,
    );

    if matches!(status, Status::Hovered | Status::Pressed) {
        background = mix(
            background,
            text_color,
            tokens::state::HOVER_STATE_LAYER_OPACITY,
        );
    }

    Style {
        background: (background.a > 0.0).then_some(Background::Color(background)),
        text_color,
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius,
        },
        ..Default::default()
    }
}
