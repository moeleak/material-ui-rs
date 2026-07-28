use super::*;

fn assert_close(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() < 0.001,
        "expected {expected}, got {actual}",
    );
}

#[test]
fn bounded_ripple_radius_matches_compose_extra_radius_for_non_round_bounds() {
    let radius = ripple_target_radius(Size::new(100.0, 40.0));

    assert_close(
        radius,
        (50.0_f32 * 50.0 + 20.0 * 20.0).sqrt() + tokens::state::RIPPLE_BOUNDED_EXTRA_RADIUS,
    );
}

#[test]
fn bounded_ripple_radius_uses_compose_extra_radius_for_round_bounds() {
    let radius = ripple_target_radius(Size::new(40.0, 40.0));

    assert_close(
        radius,
        (20.0_f32 * 20.0 + 20.0 * 20.0).sqrt() + tokens::state::RIPPLE_BOUNDED_EXTRA_RADIUS,
    );
}

#[test]
fn partial_round_bounds_use_compose_bounded_radius() {
    let radius = ripple_target_radius(Size::new(80.0, 40.0));

    assert_close(
        radius,
        (40.0_f32 * 40.0 + 20.0 * 20.0).sqrt() + tokens::state::RIPPLE_BOUNDED_EXTRA_RADIUS,
    );
}

#[test]
fn unbounded_ripple_radius_fits_within_bounds() {
    let radius = unbounded_ripple_target_radius(Size::new(100.0, 40.0));

    assert_close(radius, (50.0_f32 * 50.0 + 20.0 * 20.0).sqrt());
}

#[test]
fn ripple_starts_at_android_foreground_start_radius() {
    let start = Instant::now();
    let ripple = Ripple::new(Point::new(50.0, 20.0), start);
    let circle = ripple.circle(Size::new(100.0, 40.0), start);

    assert_close(circle.radius, 30.0);
}

#[test]
fn ripple_enter_opacity_is_linear() {
    let start = Instant::now();
    let ripple = Ripple::new(Point::new(20.0, 20.0), start);

    assert_close(ripple.opacity(start), 0.0);
    assert_close(ripple.opacity(start + duration_ms(75)), 1.0);
}

#[test]
fn rounded_rect_span_clips_full_round_corners() {
    let size = Size::new(40.0, 40.0);
    let radius = border::radius(9999.0);

    let top = rounded_rect_span_at_y(size, radius, 0.0).unwrap();
    assert_close(top.0, 20.0);
    assert_close(top.1, 20.0);

    let middle = rounded_rect_span_at_y(size, radius, 20.0).unwrap();
    assert_close(middle.0, 0.0);
    assert_close(middle.1, 40.0);

    let upper = rounded_rect_span_at_y(size, radius, 10.0).unwrap();
    assert_close(upper.0, 20.0 - (20.0_f32 * 20.0 - 10.0 * 10.0).sqrt());
    assert_close(upper.1, 20.0 + (20.0_f32 * 20.0 - 10.0 * 10.0).sqrt());
}

#[test]
fn rounded_rect_span_keeps_square_bounds_without_radius() {
    let span =
        rounded_rect_span_at_y(Size::new(80.0, 40.0), border::Radius::default(), 8.0).unwrap();

    assert_close(span.0, 0.0);
    assert_close(span.1, 80.0);
}

#[test]
fn ripple_clip_sampling_is_bounded_for_runtime_cost() {
    assert_eq!(ripple_clip_sample_count(1.0), RIPPLE_CLIP_MIN_SAMPLES);
    assert_eq!(ripple_clip_sample_count(100.0), RIPPLE_CLIP_MAX_SAMPLES);
}

#[test]
fn patterned_noise_phase_matches_aosp_shader_units() {
    let start = Instant::now();
    let ripple = Ripple::new(Point::new(20.0, 20.0), start);
    let (sparkle_phase, turbulence_phase) = ripple_noise_phases(ripple, start + duration_ms(214));

    assert_close(turbulence_phase, 1.0);
    assert_close(sparkle_phase, 0.001);
}

#[test]
fn short_press_ripple_holds_before_fade_out() {
    let start = Instant::now();
    let mut ripple = Ripple::new(Point::new(20.0, 20.0), start);
    let release = start + duration_ms(50);

    ripple.exit(release);

    assert_eq!(ripple.exit_delay, duration_ms(175));
    assert_close(ripple.opacity(release + duration_ms(174)), 1.0);
    assert_close(ripple.opacity(release + duration_ms(250)), 0.5);
    assert!(ripple.has_finished_exit(release + duration_ms(325)));
}

#[test]
fn pressing_again_moves_existing_active_ripple_to_exiting() {
    let start = Instant::now();
    let mut state = ButtonState::default();

    state.press(Point::new(10.0, 10.0), start);
    state.press(Point::new(20.0, 20.0), start + duration_ms(20));

    assert!(state.ripples.has_active_ripple());
    assert_eq!(state.ripples.exiting_ripple_count(), 1);
    assert!(state.has_visible_ripples(start + duration_ms(75)));
}

#[test]
fn patterned_ripple_opacity_tracks_aosp_progress() {
    let start = Instant::now();
    let release = start + duration_ms(50);
    let mut state = ButtonState::default();

    state.press(Point::new(10.0, 10.0), start);
    state.release(release);

    let early_release_opacity = state.ripple_opacity(release + duration_ms(25));
    let completed_enter_opacity = state.ripple_opacity(release + duration_ms(400));

    assert!(early_release_opacity > 0.0);
    assert!(completed_enter_opacity > early_release_opacity);
    assert!(state.ripple_opacity(release + duration_ms(590)) < 0.5);
    assert_close(state.ripple_opacity(release + duration_ms(826)), 0.0);
}

#[test]
fn button_draw_style_uses_active_surface_for_hover_state_layer() {
    let theme = Theme::Light;
    let class: StyleFn<'_, Theme> = Box::new(crate::style::button::fab_primary);
    let active = button_draw_style(&theme, &class, Status::Active);
    let styled_hover = theme.style(&class, Status::Hovered);
    let drawn_hover = button_draw_style(&theme, &class, Status::Hovered);

    assert_ne!(styled_hover, active);
    assert_eq!(drawn_hover, active);
}

#[test]
fn button_hover_state_layer_animates_with_compose_default_tween() {
    let start = Instant::now();
    let mut state = ButtonState::default();

    assert!(state.sync_hover(true, start));
    assert_close(state.state_layer_opacity(), 0.0);

    assert!(state.advance(start + duration_ms(7)));
    assert_close(
        state.state_layer_opacity(),
        tokens::state::HOVER_STATE_LAYER_OPACITY * (7.0 / 15.0),
    );

    assert!(!state.advance(start + duration_ms(tokens::state::STATE_LAYER_TRANSITION_DURATION_MS)));
    assert_close(
        state.state_layer_opacity(),
        tokens::state::HOVER_STATE_LAYER_OPACITY,
    );
}

#[test]
fn button_redraw_without_cursor_does_not_resync_hover() {
    let redraw = Event::Window(window::Event::RedrawRequested(Instant::now()));

    assert!(
        !ButtonInteraction {
            event: &redraw,
            cursor: mouse::Cursor::Unavailable,
            is_hovered: false,
        }
        .should_sync_hover()
    );
    assert!(
        ButtonInteraction {
            event: &redraw,
            cursor: mouse::Cursor::Available(Point::new(20.0, 20.0)),
            is_hovered: false,
        }
        .should_sync_hover()
    );
}

#[test]
fn button_initial_redraw_hover_snaps_to_hover_layer_target() {
    let start = Instant::now();
    let redraw = Event::Window(window::Event::RedrawRequested(start));
    let mut state = ButtonState::default();

    assert!(
        ButtonInteraction {
            event: &redraw,
            cursor: mouse::Cursor::Unavailable,
            is_hovered: true,
        }
        .should_snap_initial_redraw(&state)
    );
    assert!(state.sync_hover(true, start));

    state.snap_state_layer_to_hover_target();

    assert_close(
        state.state_layer_opacity(),
        tokens::state::HOVER_STATE_LAYER_OPACITY,
    );
    assert!(!state.state_layer_opacity.is_animating());
}

#[test]
fn button_draw_uses_hover_layer_target_for_fresh_hovered_state() {
    let state = ButtonState::default();

    assert_close(
        ButtonDrawState {
            state: &state,
            status: Status::Hovered,
        }
        .layer_opacity(),
        tokens::state::HOVER_STATE_LAYER_OPACITY,
    );
    assert_eq!(
        ButtonDrawState {
            state: &state,
            status: Status::Active,
        }
        .layer_opacity(),
        0.0
    );
}

#[test]
fn button_draw_keeps_mouse_hover_enter_animation() {
    let start = Instant::now();
    let mut state = ButtonState {
        last_status: Some(Status::Active),
        ..ButtonState::default()
    };
    assert!(state.sync_hover(true, start));

    assert_eq!(
        ButtonDrawState {
            state: &state,
            status: Status::Hovered,
        }
        .layer_opacity(),
        0.0
    );
}

#[test]
fn button_hover_state_layer_keeps_animating_while_ripple_is_visible() {
    let start = Instant::now();
    let mut state = ButtonState::default();

    state.press(Point::new(10.0, 10.0), start);
    state.release(start + duration_ms(1));
    assert!(state.sync_hover(true, start));

    assert!(state.advance(start + duration_ms(7)));
    assert!(state.has_visible_ripples(start + duration_ms(7)));
    assert!(state.state_layer_opacity() > 0.0);
    assert!(state.state_layer_opacity() < tokens::state::HOVER_STATE_LAYER_OPACITY);
}

#[test]
fn button_hover_state_layer_reverses_smoothly_when_pointer_leaves_mid_enter() {
    let start = Instant::now();
    let exit = start + duration_ms(7);
    let mut state = ButtonState::default();

    assert!(state.sync_hover(true, start));
    assert!(state.advance(exit));
    assert!(state.sync_hover(false, exit));

    let interrupted_opacity = tokens::state::HOVER_STATE_LAYER_OPACITY * (7.0 / 15.0);
    assert_close(state.state_layer_opacity(), interrupted_opacity);
    assert_eq!(state.state_layer_opacity.to, 0.0);

    assert!(state.advance(exit + duration_ms(7)));
    assert_close(
        state.state_layer_opacity(),
        interrupted_opacity * (1.0 - 7.0 / 15.0),
    );
}

#[test]
fn button_hover_state_layer_retargets_smoothly_when_pointer_reenters_mid_exit() {
    let start = Instant::now();
    let exit = start + duration_ms(8);
    let reenter = start + duration_ms(15);
    let mut state = ButtonState::default();

    assert!(state.sync_hover(true, start));
    assert!(state.advance(start + duration_ms(7)));

    assert!(state.sync_hover(false, exit));
    let opacity_at_exit = tokens::state::HOVER_STATE_LAYER_OPACITY * (8.0 / 15.0);
    assert_close(state.state_layer_opacity(), opacity_at_exit);

    assert!(state.sync_hover(true, reenter));
    assert_close(
        state.state_layer_opacity(),
        opacity_at_exit * (1.0 - 7.0 / 15.0),
    );
    assert_eq!(
        state.state_layer_opacity.to,
        tokens::state::HOVER_STATE_LAYER_OPACITY
    );
}

#[test]
fn button_draw_style_uses_ripple_for_pressed_state_layer() {
    let theme = Theme::Light;
    let class: StyleFn<'_, Theme> = Box::new(crate::style::button::text);
    let active = button_draw_style(&theme, &class, Status::Active);
    let pressed = button_draw_style(&theme, &class, Status::Pressed);

    assert_eq!(pressed, active);
}

#[test]
fn button_draw_style_removes_pressed_elevation_when_ripple_is_active() {
    let theme = Theme::Light;
    let class: StyleFn<'_, Theme> = Box::new(crate::style::button::fab_primary);
    let active = button_draw_style(&theme, &class, Status::Active);
    let pressed = button_draw_style(&theme, &class, Status::Pressed);

    assert_eq!(pressed, active);
}

#[test]
fn button_draw_style_does_not_reuse_hover_state_layer_during_press() {
    let theme = Theme::Light;
    let class: StyleFn<'_, Theme> = Box::new(crate::style::button::text);
    let bounds = Rectangle::new(Point::ORIGIN, Size::new(96.0, 40.0));
    let status = button_status(
        true,
        true,
        bounds,
        mouse::Cursor::Available(Point::new(20.0, 20.0)),
    );
    let active = button_draw_style(&theme, &class, Status::Active);
    let hovered = theme.style(&class, Status::Hovered);
    let pressed = button_draw_style(&theme, &class, status);

    assert_eq!(status, Status::Pressed);
    assert_eq!(pressed, active);
    assert_ne!(pressed, hovered);
}

#[test]
fn ripple_origin_clamps_to_android_foreground_radius() {
    let size = Size::new(100.0, 40.0);
    let target_radius = ripple_target_radius(size);
    let start_radius = get_ripple_start_radius(size);
    let clamped =
        clamped_ripple_origin(Point::new(500.0, 500.0), size, target_radius, start_radius);
    let center = Point::new(50.0, 20.0);
    let dx = clamped.x - center.x;
    let dy = clamped.y - center.y;

    assert_close(
        (dx * dx + dy * dy).sqrt(),
        (target_radius - start_radius).max(0.0),
    );
}

#[test]
fn press_origin_ignores_mouse_outside_bounds() {
    let bounds = Rectangle {
        x: 10.0,
        y: 20.0,
        width: 40.0,
        height: 30.0,
    };
    let event = Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left));

    assert_eq!(
        press_origin(
            &event,
            bounds,
            mouse::Cursor::Available(Point::new(80.0, 80.0))
        ),
        None
    );
}

#[test]
fn press_origin_returns_mouse_position_relative_to_bounds() {
    let bounds = Rectangle {
        x: 10.0,
        y: 20.0,
        width: 40.0,
        height: 30.0,
    };
    let event = Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left));

    assert_eq!(
        press_origin(
            &event,
            bounds,
            mouse::Cursor::Available(Point::new(25.0, 35.0))
        ),
        Some(Point::new(15.0, 15.0))
    );
}

#[test]
fn touch_press_origin_prefers_translated_cursor_position() {
    let bounds = Rectangle {
        x: 10.0,
        y: 120.0,
        width: 40.0,
        height: 30.0,
    };
    let event = Event::Touch(touch::Event::FingerPressed {
        id: touch::Finger(0),
        position: Point::new(25.0, 35.0),
    });

    assert_eq!(
        press_origin(
            &event,
            bounds,
            mouse::Cursor::Available(Point::new(25.0, 135.0))
        ),
        Some(Point::new(15.0, 15.0))
    );
}

#[test]
fn touch_press_origin_does_not_fallback_to_raw_position_when_cursor_is_available() {
    let bounds = Rectangle {
        x: 10.0,
        y: 20.0,
        width: 40.0,
        height: 30.0,
    };
    let event = Event::Touch(touch::Event::FingerPressed {
        id: touch::Finger(0),
        position: Point::new(25.0, 35.0),
    });

    assert_eq!(
        press_origin(
            &event,
            bounds,
            mouse::Cursor::Available(Point::new(25.0, 135.0))
        ),
        None
    );
}

#[test]
fn touch_press_origin_does_not_fallback_to_raw_position_when_cursor_is_levitating() {
    let bounds = Rectangle {
        x: 10.0,
        y: 20.0,
        width: 40.0,
        height: 30.0,
    };
    let event = Event::Touch(touch::Event::FingerPressed {
        id: touch::Finger(0),
        position: Point::new(25.0, 35.0),
    });

    assert_eq!(
        press_origin(
            &event,
            bounds,
            mouse::Cursor::Levitating(Point::new(25.0, 135.0))
        ),
        None
    );
}

#[test]
fn touch_release_uses_translated_cursor_position() {
    let bounds = Rectangle {
        x: 10.0,
        y: 120.0,
        width: 40.0,
        height: 30.0,
    };
    let event = Event::Touch(touch::Event::FingerLifted {
        id: touch::Finger(0),
        position: Point::new(25.0, 35.0),
    });

    assert!(release_is_over(
        &event,
        bounds,
        mouse::Cursor::Available(Point::new(25.0, 135.0))
    ));
}

#[test]
fn touch_release_does_not_fallback_to_raw_position_when_cursor_is_levitating() {
    let bounds = Rectangle {
        x: 10.0,
        y: 20.0,
        width: 40.0,
        height: 30.0,
    };
    let event = Event::Touch(touch::Event::FingerLifted {
        id: touch::Finger(0),
        position: Point::new(25.0, 35.0),
    });

    assert!(!release_is_over(
        &event,
        bounds,
        mouse::Cursor::Levitating(Point::new(25.0, 135.0))
    ));
}

#[test]
fn touch_press_origin_falls_back_to_raw_position_without_cursor() {
    let bounds = Rectangle {
        x: 10.0,
        y: 20.0,
        width: 40.0,
        height: 30.0,
    };
    let event = Event::Touch(touch::Event::FingerPressed {
        id: touch::Finger(0),
        position: Point::new(25.0, 35.0),
    });

    assert_eq!(
        press_origin(&event, bounds, mouse::Cursor::Unavailable),
        Some(Point::new(15.0, 15.0))
    );
}

#[test]
fn touch_move_beyond_click_slop_cancels_click_candidate() {
    let press_position = Some(Point::new(25.0, 35.0));
    let small_move = Event::Touch(touch::Event::FingerMoved {
        id: touch::Finger(0),
        position: Point::new(25.0 + TOUCH_CLICK_SLOP / 2.0, 35.0),
    });
    let scroll_move = Event::Touch(touch::Event::FingerMoved {
        id: touch::Finger(0),
        position: Point::new(25.0 + TOUCH_CLICK_SLOP + 1.0, 35.0),
    });

    assert!(
        !TouchClick {
            press_position,
            event: &small_move,
            cursor: mouse::Cursor::Unavailable,
        }
        .moved_beyond_slop()
    );
    assert!(
        TouchClick {
            press_position,
            event: &scroll_move,
            cursor: mouse::Cursor::Unavailable,
        }
        .moved_beyond_slop()
    );
}
