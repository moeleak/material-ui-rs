use super::*;

#[test]
fn state_toggles_open_closed() {
    let mut state = State::new();

    assert!(!state.is_open());

    state.toggle();
    assert!(state.is_open());

    state.close();
    assert!(!state.is_open());
}

#[test]
fn state_toggle_animates_picker_panel_reveal() {
    let start = Instant::now();
    let mut state = State::new();

    assert_eq!(state.reveal(), 0.0);
    state.toggle_at(start);

    assert!(state.is_open());
    assert!(state.is_animating());
    assert_eq!(state.reveal(), 0.0);

    assert!(state.advance(start + duration_ms(PICKER_PANEL_TRANSITION_DURATION_MS / 2)));
    assert!(state.reveal() > 0.0);

    state.close_at(start + duration_ms(PICKER_PANEL_TRANSITION_DURATION_MS));
    assert!(!state.is_open());
    assert!(state.is_animating());
}

#[test]
fn bottom_margin_accounts_for_adaptive_navigation_clearance() {
    assert_eq!(
        bottom_margin(navigation::AdaptiveLayout::NavigationBar),
        FLOATING_MARGIN + tokens::component::navigation_bar::CONTAINER_HEIGHT
    );
    assert_eq!(
        bottom_margin(navigation::AdaptiveLayout::NavigationRail),
        FLOATING_MARGIN
    );
    assert_eq!(
        bottom_margin_for(
            navigation::AdaptiveLayout::NavigationBar,
            navigation::CompactNavigation::ModalDrawer,
        ),
        FLOATING_MARGIN
    );
}

#[test]
fn floating_clearance_tracks_the_revealed_picker_panel() {
    let start = Instant::now();
    let mut state = State::new();

    assert_eq!(state.floating_clearance(), PALETTE_BUTTON_SIZE);

    state.open_at(start);
    let _ = state.advance(start + duration_ms(PICKER_PANEL_TRANSITION_DURATION_MS));

    assert_eq!(
        state.floating_clearance(),
        PALETTE_BUTTON_SIZE + picker_panel_slot_height()
    );
}

#[test]
fn floating_panel_padding_keeps_reveal_above_independent_fab() {
    let padding = floating_padding(FLOATING_MARGIN, FLOATING_MARGIN + PALETTE_BUTTON_SIZE);

    assert_eq!(padding.right, FLOATING_MARGIN);
    assert_eq!(padding.bottom, FLOATING_MARGIN + PALETTE_BUTTON_SIZE);
}

#[test]
fn controller_toggle_picker_starts_panel_animation() {
    let mut controller = ThemeController::new(MaterialColor::Purple, true);

    controller.update(
        ThemeAction::TogglePicker,
        Size::new(1080.0, 980.0),
        FLOATING_MARGIN,
        Instant::now(),
    );

    assert!(controller.is_picker_open());
    assert!(controller.is_animating());
}

#[test]
fn controller_selects_color_and_closes_picker_from_action() {
    let mut controller = ThemeController::new(MaterialColor::Purple, true);
    let viewport = Size::new(1080.0, 980.0);
    let bottom_margin = FLOATING_MARGIN;
    let now = Instant::now();

    controller.update(ThemeAction::TogglePicker, viewport, bottom_margin, now);
    assert!(controller.is_picker_open());

    controller.update(
        ThemeAction::SelectColor(MaterialColor::Blue),
        viewport,
        bottom_margin,
        now,
    );

    let transition = controller
        .transition()
        .expect("selecting a different color should animate");

    assert_eq!(controller.selected_color(), MaterialColor::Blue);
    assert!(!controller.is_picker_open());
    assert_eq!(
        transition.origin(),
        swatch_center(viewport, bottom_margin, MaterialColor::Blue)
    );
}

#[test]
fn controller_dark_mode_action_uses_supplied_origin() {
    let mut controller = ThemeController::new(MaterialColor::Purple, true);
    let origin = Point::new(64.0, 512.0);
    let now = Instant::now();

    controller.update(
        ThemeAction::SetDarkMode {
            dark_mode: false,
            origin,
        },
        Size::new(1080.0, 980.0),
        FLOATING_MARGIN,
        now,
    );

    let transition = controller
        .transition()
        .expect("changing dark mode should animate");

    assert!(!controller.dark_mode());
    assert_eq!(transition.origin(), origin);
}

#[test]
fn material_colors_generate_distinct_primary_roles() {
    assert_eq!(MaterialColor::ALL.len(), 8);
    assert_eq!(
        MaterialColor::Purple.color_scheme(false).primary,
        Theme::Light.colors().primary
    );
    assert_ne!(
        MaterialColor::Blue.color_scheme(false).primary,
        Theme::Light.colors().primary
    );
    assert_ne!(
        MaterialColor::Green.color_scheme(true).primary,
        Theme::Dark.colors().primary
    );
    assert_ne!(
        MaterialColor::Blue
            .color_scheme(false)
            .surface
            .container
            .base,
        Theme::Light.colors().surface.container.base
    );
    assert_ne!(
        MaterialColor::Blue.color_scheme(false).secondary.container,
        Theme::Light.colors().secondary.container
    );
}

#[test]
fn material_colors_tint_menu_backgrounds() {
    let baseline = crate::style::menu::default(&Theme::Light);
    let blue = Theme::new("Blue", MaterialColor::Blue.color_scheme(false));
    let tinted = crate::style::menu::default(&blue);

    assert_ne!(tinted.background, baseline.background);
    assert_ne!(tinted.selected_background, baseline.selected_background);
}

#[test]
fn selected_swatch_uses_stronger_outline() {
    let theme = Theme::Light;
    let selected = swatch_style(&theme, Status::Active, MaterialColor::Blue, true);
    let unselected = swatch_style(&theme, Status::Active, MaterialColor::Blue, false);

    assert_eq!(selected.border.width, SELECTED_SWATCH_OUTLINE_WIDTH);
    assert_eq!(unselected.border.width, SWATCH_OUTLINE_WIDTH);
}

#[test]
fn swatch_centers_track_picker_layout() {
    let viewport = Size::new(1080.0, 980.0);
    let bottom_margin = FLOATING_MARGIN;
    let purple = swatch_center(viewport, bottom_margin, MaterialColor::Purple);
    let blue = swatch_center(viewport, bottom_margin, MaterialColor::Blue);
    let yellow = swatch_center(viewport, bottom_margin, MaterialColor::Yellow);

    assert_eq!(blue.x - purple.x, SWATCH_TARGET_SIZE + PICKER_PANEL_SPACING);
    assert_eq!(
        yellow.y - purple.y,
        SWATCH_TARGET_SIZE + PICKER_PANEL_SPACING
    );
    assert!(purple.x < palette_center(viewport, bottom_margin).x);
}

#[test]
fn picker_panel_reveal_height_clamps_to_slot_height() {
    assert_eq!(picker_panel_reveal_height(-1.0), 0.0);
    assert_eq!(picker_panel_reveal_height(0.0), 0.0);
    assert_eq!(
        picker_panel_reveal_height(1.0),
        picker_panel_height() + PICKER_PANEL_SPACING
    );
    assert_eq!(
        picker_panel_reveal_height(2.0),
        picker_panel_height() + PICKER_PANEL_SPACING
    );
}

#[test]
fn reveal_overlay_uses_android_style_thresholds() {
    assert_eq!(percent_past_threshold(0.5, 0.5), 0.0);
    assert_eq!(percent_past_threshold(1.0, 0.5), 1.0);
    assert_eq!(reveal_gradient_end_alpha(1.0), 0.0);
    assert_eq!(
        reveal_gradient_end_alpha(THEME_REVEAL_EDGE_FADE_THRESHOLD),
        1.0
    );
    assert!(reveal_gradient_end_alpha(0.80) > reveal_gradient_end_alpha(0.95));
    assert_eq!(reveal_start_fill_alpha(1.0), 0.0);
    assert!(reveal_start_fill_alpha(0.40) > reveal_start_fill_alpha(0.80));
}

#[test]
fn reveal_blur_is_strongest_mid_transition() {
    assert!(reveal_blur_ratio(0.5) > reveal_blur_ratio(0.1));
    assert!(reveal_blur_ratio(0.5) > reveal_blur_ratio(0.9));
    assert!(reveal_blur_width(1000.0, 0.5) > reveal_blur_width(1000.0, 0.0));
}
