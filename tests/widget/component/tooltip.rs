use super::*;

#[test]
fn plain_tooltip_text_shrinks_under_material_max_width() {
    let type_scale = tokens::component::tooltip::PLAIN_SUPPORTING_TEXT;
    let text: Text<'_, Theme, iced_widget::Renderer> =
        plain_supporting_text("Material 3 plain tooltip", type_scale);

    assert_eq!(
        Widget::<(), Theme, iced_widget::Renderer>::size(&text).width,
        Length::Shrink
    );
    assert_eq!(plain_tooltip_inner_horizontal_padding(), 4.0);
    assert_eq!(plain_tooltip_inner_max_width(), 192.0);
    assert_eq!(
        plain_tooltip_inner_max_width() + tokens::component::tooltip::PLAIN_VERTICAL_SPACE * 2.0,
        tokens::component::tooltip::PLAIN_MAX_WIDTH
    );
}

#[test]
fn rich_tooltip_padding_matches_androidx_material_layout_constants() {
    assert_eq!(rich_title_top_padding(), 8.0);
    assert_eq!(rich_tooltip_shadow_padding(), 8.0);
    assert_eq!(
        rich_supporting_text_padding(false, false),
        Padding {
            top: 4.0,
            right: 0.0,
            bottom: 4.0,
            left: 0.0,
        }
    );
    assert_eq!(
        rich_supporting_text_padding(true, true),
        Padding {
            top: 4.0,
            right: 0.0,
            bottom: 16.0,
            left: 0.0,
        }
    );
}

#[test]
fn rich_tooltip_surface_keeps_material_gap_from_anchor() {
    let content = Rectangle {
        x: 120.0,
        y: 160.0,
        width: 80.0,
        height: 32.0,
    };
    let clip_padding = rich_tooltip_shadow_padding();
    let tooltip_node = layout::Node::new(Size::new(180.0, 96.0));
    let tooltip = rich_tooltip_surface_bounds(
        &tooltip_node,
        RichTooltipPlacement {
            content_bounds: content,
            viewport: Rectangle::new(Point::ORIGIN, Size::new(400.0, 400.0)),
            cursor_position: Point::ORIGIN,
            position: Position::Top,
            gap: tokens::component::tooltip::SPACING_BETWEEN_TOOLTIP_AND_ANCHOR,
            padding: 0.0,
            clip_padding,
            snap_within_viewport: true,
        },
    );
    let surface = rich_tooltip_visual_bounds(tooltip, clip_padding);

    assert_eq!(
        content.y - (surface.y + surface.height),
        tokens::component::tooltip::SPACING_BETWEEN_TOOLTIP_AND_ANCHOR
    );
    assert_eq!(tooltip.width, 180.0 + clip_padding * 2.0);
    assert_eq!(surface.width, 180.0);
}

#[test]
fn rich_tooltip_corridor_spans_anchor_surface_gap() {
    let content = Rectangle {
        x: 120.0,
        y: 160.0,
        width: 80.0,
        height: 32.0,
    };
    let tooltip = Rectangle {
        x: 70.0,
        y: 60.0,
        width: 180.0,
        height: 96.0,
    };
    let corridor = rich_tooltip_corridor_bounds(content, tooltip, Position::Top)
        .expect("top tooltip should have a corridor to its anchor");

    assert!(corridor.contains(Point::new(160.0, 158.0)));
    assert!(!corridor.contains(Point::new(160.0, 120.0)));
    assert!(!corridor.contains(Point::new(80.0, 158.0)));
}

#[test]
fn rich_tooltip_keep_alive_does_not_cover_adjacent_anchor() {
    let rich_anchor = Rectangle {
        x: 289.0,
        y: 290.0,
        width: 118.0,
        height: 62.0,
    };
    let rich_tooltip = Rectangle {
        x: 28.0,
        y: 9.0,
        width: 640.0,
        height: 272.0,
    };

    assert!(rich_tooltip_keep_alive_contains(
        rich_anchor,
        rich_tooltip,
        Position::Top,
        mouse::Cursor::Available(Point::new(348.0, 286.0)),
    ));
    assert!(!rich_tooltip_keep_alive_contains(
        rich_anchor,
        rich_tooltip,
        Position::Top,
        mouse::Cursor::Available(Point::new(200.0, 322.0)),
    ));
}

#[test]
fn rich_tooltip_anchor_exit_defers_dismissal_to_overlay() {
    let start = Instant::now();
    let mut state = RichTooltipState::default();

    state.show(start, Point::new(10.0, 20.0));

    assert!(!rich_tooltip_anchor_exit_dismisses(true, &state));
    assert!(rich_tooltip_anchor_exit_dismisses(false, &state));
}

#[test]
fn rich_tooltip_transition_matches_androidx_compose_motion() {
    let start = Instant::now();
    let mut state = RichTooltipState::default();

    state.show(start, Point::new(10.0, 20.0));
    assert_eq!(state.phase, TooltipPhase::Showing);
    assert_eq!(
        state.animation_frame(start),
        TooltipAnimationFrame {
            scale: tokens::component::tooltip::SCALE_START,
            alpha: 0.0,
        }
    );

    let halfway = start + duration_ms(tokens::component::tooltip::FADE_IN_DURATION_MS / 2);
    let half_frame = state.animation_frame(halfway);
    assert!((half_frame.alpha - 0.5).abs() < 0.001);
    assert!(
        (half_frame.scale
            - lerp(
                tokens::component::tooltip::SCALE_START,
                1.0,
                tooltip_scale_easing(0.5)
            ))
        .abs()
            < 0.001
    );

    let shown = start + duration_ms(tokens::component::tooltip::FADE_IN_DURATION_MS);
    state.advance(shown);
    assert_eq!(state.phase, TooltipPhase::Shown);
    assert_eq!(
        state.animation_frame(shown),
        TooltipAnimationFrame {
            scale: 1.0,
            alpha: 1.0,
        }
    );

    state.dismiss(shown);
    assert_eq!(state.phase, TooltipPhase::Dismissing);
    assert_eq!(
        state.animation_frame(shown),
        TooltipAnimationFrame {
            scale: 1.0,
            alpha: 1.0,
        }
    );

    let exit_half = shown + duration_ms(tokens::component::tooltip::FADE_OUT_DURATION_MS / 2);
    let exit_progress = state.progress(exit_half);
    let exit_half_frame = state.animation_frame(exit_half);
    assert!((exit_half_frame.alpha - (1.0 - exit_progress)).abs() < 0.001);
    assert!(
        (exit_half_frame.scale
            - lerp(
                1.0,
                tokens::component::tooltip::SCALE_START,
                tooltip_scale_easing(exit_progress)
            ))
        .abs()
            < 0.001
    );

    let hidden = shown + duration_ms(tokens::component::tooltip::FADE_OUT_DURATION_MS);
    state.advance(hidden);
    assert_eq!(state.phase, TooltipPhase::Hidden);
    assert_eq!(
        state.animation_frame(hidden),
        TooltipAnimationFrame {
            scale: tokens::component::tooltip::SCALE_START,
            alpha: 0.0,
        }
    );
}

#[test]
fn tooltip_alpha_style_scales_container_colors() {
    let theme = Theme::Light;
    let style = tooltip_container_style_alpha(tooltip_style::rich(&theme), 0.5);

    assert_eq!(
        style.background,
        Some(Background::Color(alpha_color(
            theme.colors().surface.container.base,
            0.5
        )))
    );
    assert_eq!(
        style.text_color,
        Some(alpha_color(theme.colors().surface.text_variant, 0.5))
    );
}
