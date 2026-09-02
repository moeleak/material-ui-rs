use super::*;

#[test]
fn snackbar_host_defaults_to_material_bottom_margin() {
    assert_eq!(
        HostOptions::default().bottom_margin,
        tokens::component::snackbar::BOTTOM_MARGIN
    );
}

#[test]
fn snackbar_host_preserves_material_margin_above_fab_like_compose_scaffold() {
    let fab_height = tokens::component::fab::CONTAINER_HEIGHT;
    let fab_bottom_margin = 24.0;
    let options = HostOptions::default().above_fab(fab_height, fab_bottom_margin);

    assert_eq!(
        options.bottom_margin,
        fab_bottom_margin + fab_height + tokens::component::snackbar::BOTTOM_MARGIN
    );
}

#[test]
fn snackbar_transition_matches_android_slide_and_content_fade_timing() {
    let start = Instant::now();
    let mut transition = Transition::default();
    let hidden_distance = tokens::component::snackbar::WITH_SINGLE_LINE_CONTAINER_HEIGHT
        + tokens::component::snackbar::BOTTOM_MARGIN;

    transition.show(start);

    assert_eq!(transition.phase(), TransitionPhase::Showing);
    assert_eq!(
        transition.translation_y(start, hidden_distance),
        hidden_distance
    );
    assert_eq!(transition.content_alpha(start), 0.0);
    assert_eq!(
        transition.content_alpha(start + Duration::from_millis(70)),
        0.0
    );

    let halfway = start + Duration::from_millis(125);
    assert!(transition.translation_y(halfway, hidden_distance) < hidden_distance);
    assert!(transition.translation_y(halfway, hidden_distance) > 0.0);
    assert!(transition.content_alpha(halfway) > 0.0);
    assert!(transition.content_alpha(halfway) < 1.0);

    let shown = start
        + Duration::from_millis(u64::from(
            tokens::component::snackbar::SLIDE_ANIMATION_DURATION_MS,
        ));
    assert_eq!(transition.translation_y(shown, hidden_distance), 0.0);
    assert_eq!(transition.content_alpha(shown), 1.0);

    assert!(transition.advance(shown));
    assert_eq!(transition.phase(), TransitionPhase::Shown);
}

#[test]
fn snackbar_transition_auto_dismisses_after_android_long_duration() {
    let start = Instant::now();
    let mut transition = Transition::default();

    transition.show(start);
    let shown = start
        + Duration::from_millis(u64::from(
            tokens::component::snackbar::SLIDE_ANIMATION_DURATION_MS,
        ));
    let _ = transition.advance(shown);
    let _ = transition.advance(
        shown + Duration::from_millis(u64::from(tokens::component::snackbar::LONG_DURATION_MS)),
    );

    assert_eq!(transition.phase(), TransitionPhase::Dismissing);
    assert_eq!(
        transition.content_alpha(
            shown
                + Duration::from_millis(u64::from(tokens::component::snackbar::LONG_DURATION_MS,))
                + Duration::from_millis(u64::from(
                    tokens::component::snackbar::CONTENT_FADE_ANIMATION_DURATION_MS,
                )),
        ),
        0.0
    );

    let hidden = shown
        + Duration::from_millis(u64::from(tokens::component::snackbar::LONG_DURATION_MS))
        + Duration::from_millis(u64::from(
            tokens::component::snackbar::SLIDE_ANIMATION_DURATION_MS,
        ));
    assert!(!transition.advance(hidden));
    assert_eq!(transition.phase(), TransitionPhase::Hidden);
}

#[test]
fn snackbar_container_uses_inverse_surface_tokens_in_light_and_dark() {
    for theme in [Theme::Light, Theme::Dark] {
        let colors = theme.colors();
        let style = container_style(&theme);

        assert_eq!(
            style.background,
            Some(Background::Color(colors.inverse.inverse_surface))
        );
        assert_eq!(style.text_color, Some(colors.inverse.inverse_surface_text));
        assert_eq!(
            style.border.radius.top_left,
            tokens::component::snackbar::CONTAINER_SHAPE
        );
        assert_eq!(style.shadow.offset.y, 4.0);
        assert_eq!(style.shadow.blur_radius, 8.0);
    }
}

#[test]
fn snackbar_actions_use_inverse_tokens_in_light_and_dark() {
    for theme in [Theme::Light, Theme::Dark] {
        let colors = theme.colors();
        let action = action_style_alpha(&theme, Status::Active, 1.0);
        let icon = icon_action_style(&theme, Status::Active);

        assert_eq!(action.text_color, colors.inverse.inverse_primary);
        assert_eq!(action.background, None);
        assert_eq!(icon.text_color, colors.inverse.inverse_surface_text);
        assert_eq!(icon.background, None);
    }
}
