use super::{component, elevation, motion, shape, state, typography};

#[test]
fn m3_state_tokens_match_google_values() {
    assert_eq!(state::HOVER_STATE_LAYER_OPACITY, 0.08);
    assert_eq!(state::FOCUS_STATE_LAYER_OPACITY, 0.10);
    assert_eq!(state::PRESSED_STATE_LAYER_OPACITY, 0.10);
    assert_eq!(state::DRAGGED_STATE_LAYER_OPACITY, 0.16);
    assert_eq!(state::STATE_LAYER_TRANSITION_DURATION_MS, 15);
    assert_eq!(state::RIPPLE_BOUNDED_EXTRA_RADIUS, 10.0);
    assert_eq!(state::RIPPLE_START_RADIUS_FACTOR, 0.3);
    assert_eq!(state::RIPPLE_RADIUS_DURATION_MS, 225);
    assert_eq!(state::RIPPLE_ORIGIN_DURATION_MS, 225);
    assert_eq!(state::RIPPLE_FADE_IN_DURATION_MS, 75);
    assert_eq!(state::RIPPLE_FADE_OUT_DURATION_MS, 150);
    assert_eq!(state::RIPPLE_OPACITY_HOLD_DURATION_MS, 225);
    assert_eq!(state::RIPPLE_MAX_RIPPLES, 10);
    assert_eq!(state::RIPPLE_PATTERN_ENTER_DURATION_MS, 450);
    assert_eq!(state::RIPPLE_PATTERN_EXIT_DURATION_MS, 375);
    assert_eq!(state::RIPPLE_PATTERN_BACKGROUND_OPACITY_DURATION_MS, 80);
    assert_eq!(state::RIPPLE_PATTERN_NOISE_ANIMATION_DURATION_MS, 7000);
    assert_eq!(state::RIPPLE_PATTERN_RADIUS_SCALE, 2.3);
    assert_eq!(state::RIPPLE_PATTERN_FADE_IN_END, 0.13);
    assert_eq!(state::RIPPLE_PATTERN_FADE_OUT_START, 0.4);
    assert_eq!(state::RIPPLE_PATTERN_FADE_OUT_NOISE_END, 0.5);
    assert_eq!(state::RIPPLE_PATTERN_SOFT_CIRCLE_BLUR, 1.0);
    assert_eq!(state::RIPPLE_PATTERN_SOFT_RING_THICKNESS, 0.05);
    assert_eq!(state::RIPPLE_PATTERN_NOISE_DENSITY_SCALE, 2.1);
    assert_eq!(
        state::RIPPLE_PATTERN_DEFAULT_EFFECT_ALPHA,
        0x8d as f32 / 255.0
    );
}

#[test]
fn m3_motion_tokens_match_google_values() {
    assert_eq!(motion::DURATION_SHORT4_MS, 200);
    assert_eq!(motion::DURATION_MEDIUM2_MS, 300);
    assert_eq!(motion::DURATION_EXTRA_LONG4_MS, 1000);
    assert_eq!(motion::SPRING_DEFAULT_DISPLACEMENT_THRESHOLD, 0.01);
    assert_eq!(motion::EXPRESSIVE_DEFAULT_SPATIAL.damping_ratio, 0.8);
    assert_eq!(motion::EXPRESSIVE_DEFAULT_SPATIAL.stiffness, 380.0);
    assert_eq!(motion::EXPRESSIVE_DEFAULT_EFFECTS.damping_ratio, 1.0);
    assert_eq!(motion::EXPRESSIVE_DEFAULT_EFFECTS.stiffness, 1600.0);
    assert_eq!(motion::EXPRESSIVE_FAST_SPATIAL.damping_ratio, 0.6);
    assert_eq!(motion::EXPRESSIVE_FAST_SPATIAL.stiffness, 800.0);
    assert_eq!(motion::EXPRESSIVE_FAST_EFFECTS.damping_ratio, 1.0);
    assert_eq!(motion::EXPRESSIVE_FAST_EFFECTS.stiffness, 3800.0);
    assert_eq!(motion::EXPRESSIVE_SLOW_SPATIAL.damping_ratio, 0.8);
    assert_eq!(motion::EXPRESSIVE_SLOW_SPATIAL.stiffness, 200.0);
    assert_eq!(motion::EXPRESSIVE_SLOW_EFFECTS.damping_ratio, 1.0);
    assert_eq!(motion::EXPRESSIVE_SLOW_EFFECTS.stiffness, 800.0);
    assert_eq!(
        motion::EASING_EMPHASIZED_DECELERATE,
        motion::CubicBezier::new(0.05, 0.7, 0.1, 1.0)
    );
}

#[test]
fn cubic_bezier_transform_clamps_and_reaches_endpoints() {
    assert_eq!(motion::EASING_EMPHASIZED_DECELERATE.transform(-1.0), 0.0);
    assert_eq!(motion::EASING_EMPHASIZED_DECELERATE.transform(0.0), 0.0);
    assert_eq!(motion::EASING_EMPHASIZED_DECELERATE.transform(1.0), 1.0);
    assert_eq!(motion::EASING_EMPHASIZED_DECELERATE.transform(2.0), 1.0);
    assert!(
        motion::EASING_EMPHASIZED_DECELERATE.transform(0.5) > motion::EASING_LINEAR.transform(0.5)
    );
}

#[test]
fn m3_shape_and_elevation_tokens_match_google_values() {
    assert_eq!(shape::CORNER_EXTRA_LARGE, 28.0);
    assert_eq!(shape::CORNER_FULL, 9999.0);
    assert_eq!(elevation::level(2), 3.0);
    assert_eq!(elevation::level(5), 12.0);
}

#[test]
fn m3_component_sizing_tokens_match_google_values() {
    assert_eq!(component::badge::SMALL_SIZE, 6.0);
    assert_eq!(component::badge::LARGE_CONTAINER_HEIGHT, 16.0);
    assert_eq!(component::badge::LARGE_CONTAINER_MIN_WIDTH, 16.0);
    assert_eq!(component::badge::LARGE_CONTAINER_MAX_WIDTH, 34.0);
    assert_eq!(component::badge::LARGE_CONTAINER_SHAPE, 8.0);
    assert_eq!(component::badge::LARGE_HORIZONTAL_SPACE, 4.0);
    assert_eq!(component::badge::ICON_ONLY_OFFSET, 6.0);
    assert_eq!(component::badge::WITH_CONTENT_HORIZONTAL_OFFSET, 12.0);
    assert_eq!(component::badge::WITH_CONTENT_VERTICAL_OFFSET, 14.0);
    assert_eq!(component::badge::LABEL_TEXT, typography::LABEL_SMALL);
    assert_eq!(component::button::CONTAINER_HEIGHT, 40.0);
    assert_eq!(component::button::LABEL_TEXT_SIZE, 14.0);
    assert_eq!(component::button::LABEL_TEXT_LINE_HEIGHT, 20.0);
    assert_eq!(component::button::LABEL_TEXT_WEIGHT, 500);
    assert_eq!(component::button::FILLED_ELEVATION.hovered, 1);
    assert_eq!(component::button::ELEVATED_ELEVATION.active, 1);
    assert_eq!(component::button::ELEVATED_ELEVATION.hovered, 2);
    assert_eq!(component::button::FILLED_TONAL_ELEVATION.pressed, 0);
    assert_eq!(component::checkbox::CONTAINER_SIZE, 18.0);
    assert_eq!(component::checkbox::ICON_SIZE, 18.0);
    assert_eq!(component::checkbox::CHECKMARK_STROKE_WIDTH, 2.0);
    assert_eq!(component::checkbox::CHECKMARK_BOTTOM_LEFT_X, 7.0);
    assert_eq!(component::checkbox::CHECKMARK_BOTTOM_LEFT_Y, -14.0);
    assert_eq!(component::checkbox::CHECKMARK_SHORT_MARK_SIZE, 5.656_854);
    assert_eq!(component::checkbox::CHECKMARK_LONG_MARK_SIZE, 11.313_708);
    assert_eq!(component::checkbox::STATE_LAYER_SIZE, 40.0);
    assert_eq!(component::checkbox::LABEL_TEXT_SIZE, 16.0);
    assert_eq!(component::checkbox::LABEL_TEXT_LINE_HEIGHT, 24.0);
    assert_eq!(component::checkbox::LABEL_TEXT_WEIGHT, 400);
    assert_eq!(component::checkbox::SELECT_TRANSITION_DURATION_MS, 350);
    assert_eq!(component::checkbox::UNSELECT_TRANSITION_DURATION_MS, 150);
    assert_eq!(component::checkbox::OPACITY_TRANSITION_DURATION_MS, 50);
    assert_eq!(
        component::checkbox::SELECT_TRANSITION_EASING,
        motion::EASING_EMPHASIZED_DECELERATE
    );
    assert_eq!(
        component::checkbox::UNSELECT_TRANSITION_EASING,
        motion::EASING_EMPHASIZED_ACCELERATE
    );
    assert_eq!(
        component::checkbox::SELECTED_DISABLED_CONTAINER_OPACITY,
        0.38
    );
    assert_eq!(component::date_picker::CONTAINER_WIDTH, 360.0);
    assert_eq!(component::date_picker::CONTAINER_HEIGHT, 568.0);
    assert_eq!(component::date_picker::DATE_CONTAINER_WIDTH, 40.0);
    assert_eq!(component::date_picker::DATE_CONTAINER_HEIGHT, 40.0);
    assert_eq!(component::date_picker::HEADER_CONTAINER_HEIGHT, 120.0);
    assert_eq!(component::date_picker::RANGE_HEADER_CONTAINER_HEIGHT, 128.0);
    assert_eq!(
        component::date_picker::RANGE_MONTH_SUBHEAD_START_SPACE,
        24.0
    );
    assert_eq!(component::date_picker::RANGE_MONTH_SUBHEAD_TOP_SPACE, 20.0);
    assert_eq!(
        component::date_picker::RANGE_MONTH_SUBHEAD_BOTTOM_SPACE,
        8.0
    );
    assert_eq!(component::date_picker::DIALOG_ACTIONS_END_SPACE, 6.0);
    assert_eq!(component::date_picker::DIALOG_ACTIONS_BOTTOM_SPACE, 8.0);
    assert_eq!(component::date_picker::DIALOG_ACTIONS_MAIN_AXIS_SPACE, 8.0);
    assert_eq!(
        component::date_picker::DIALOG_ACTIONS_CROSS_AXIS_SPACE,
        12.0
    );
    assert_eq!(component::date_picker::YEAR_CONTAINER_WIDTH, 72.0);
    assert_eq!(component::date_picker::YEAR_CONTAINER_HEIGHT, 36.0);
    assert_eq!(component::time_picker::CLOCK_DIAL_SIZE, 256.0);
    assert_eq!(
        component::time_picker::CLOCK_DIAL_SELECTOR_HANDLE_SIZE,
        48.0
    );
    assert_eq!(component::time_picker::CLOCK_DIAL_SELECTOR_TRACK_WIDTH, 2.0);
    assert_eq!(
        component::time_picker::PERIOD_SELECTOR_HORIZONTAL_WIDTH,
        216.0
    );
    assert_eq!(
        component::time_picker::PERIOD_SELECTOR_VERTICAL_HEIGHT,
        80.0
    );
    assert_eq!(component::time_picker::PERIOD_TOGGLE_MARGIN, 12.0);
    assert_eq!(component::time_picker::PERIOD_SELECTOR_START_SPACE, 12.0);
    assert_eq!(component::time_picker::PERIOD_SELECTOR_ITEM_GAP, 1.0);
    assert_eq!(component::time_picker::TIME_SELECTOR_WIDTH, 96.0);
    assert_eq!(component::time_picker::TIME_SELECTOR_HEIGHT, 80.0);
    assert_eq!(component::time_picker::TIME_SCROLL_FIELD_WIDTH, 100.0);
    assert_eq!(component::time_picker::TIME_SCROLL_FIELD_HEIGHT, 120.0);
    assert_eq!(component::time_picker::TIME_SCROLL_SEPARATOR_WIDTH, 16.0);
    assert_eq!(component::time_picker::RICH_PERIOD_SELECTOR_WIDTH, 56.0);
    assert_eq!(component::time_picker::RICH_PERIOD_SELECTOR_HEIGHT, 120.0);
    assert_eq!(component::time_picker::RICH_PERIOD_SELECTOR_ITEM_GAP, 1.0);
    assert_eq!(component::time_picker_dialog::CONTENT_PADDING, 24.0);
    assert_eq!(component::time_picker_dialog::TITLE_TOP_SPACE, 24.0);
    assert_eq!(component::time_picker_dialog::TITLE_BOTTOM_SPACE, 20.0);
    assert_eq!(component::time_picker_dialog::ACTIONS_BOTTOM_SPACE, 24.0);
    assert_eq!(
        component::time_picker_dialog::MIN_HEIGHT_FOR_TIME_PICKER,
        300.0
    );
    assert_eq!(component::time_picker_dialog::RICH_CONTENT_PADDING, 12.0);
    assert_eq!(component::time_picker_dialog::RICH_CONTENT_TOP_SPACE, 12.0);
    assert_eq!(
        component::time_picker_dialog::RICH_CONTENT_ACTIONS_SPACE,
        12.0
    );
    assert_eq!(
        component::time_picker_dialog::RICH_ACTIONS_BOTTOM_SPACE,
        12.0
    );
    assert_eq!(component::time_input::TIME_FIELD_CONTAINER_WIDTH, 96.0);
    assert_eq!(component::time_input::TIME_FIELD_CONTAINER_HEIGHT, 72.0);
    assert_eq!(
        component::time_input::TIME_FIELD_SUPPORTING_TEXT_TOP_SPACE,
        7.0
    );
    assert_eq!(
        component::time_input::PERIOD_SELECTOR_CONTAINER_HEIGHT,
        72.0
    );
    assert_eq!(component::time_input::DISPLAY_SEPARATOR_WIDTH, 24.0);
    assert_eq!(component::switch::TRACK_WIDTH, 52.0);
    assert_eq!(component::switch::TRACK_HEIGHT, 32.0);
    assert_eq!(component::switch::TRACK_OUTLINE_WIDTH, 2.0);
    assert_eq!(component::switch::WITH_ICON_HANDLE_SIZE, 24.0);
    assert_eq!(component::switch::PRESSED_HANDLE_SIZE, 28.0);
    assert_eq!(component::switch::SELECTED_ICON_SIZE, 16.0);
    assert_eq!(component::switch::UNSELECTED_ICON_SIZE, 16.0);
    assert_eq!(component::switch::LABEL_TEXT_SIZE, 16.0);
    assert_eq!(component::switch::LABEL_TEXT_LINE_HEIGHT, 24.0);
    assert_eq!(component::switch::LABEL_TEXT_WEIGHT, 400);
    assert_eq!(component::switch::TRACK_COLOR_TRANSITION_DURATION_MS, 67);
    assert_eq!(component::switch::HANDLE_COLOR_TRANSITION_DURATION_MS, 67);
    assert_eq!(component::switch::HANDLE_SIZE_TRANSITION_DURATION_MS, 250);
    assert_eq!(
        component::switch::PRESSED_HANDLE_SIZE_TRANSITION_DURATION_MS,
        100
    );
    assert_eq!(
        component::switch::HANDLE_POSITION_TRANSITION_DURATION_MS,
        300
    );
    assert_eq!(
        component::switch::HANDLE_POSITION_TRANSITION_EASING,
        motion::CubicBezier::new(0.175, 0.885, 0.32, 1.275)
    );
    assert_eq!(component::switch::ICON_FILL_TRANSITION_DURATION_MS, 67);
    assert_eq!(component::switch::ICON_OPACITY_TRANSITION_DURATION_MS, 33);
    assert_eq!(
        component::switch::ICON_TRANSFORM_TRANSITION_DURATION_MS,
        167
    );
    assert_eq!(component::switch::DISABLED_TRACK_OPACITY, 0.12);
    assert_eq!(component::switch::DISABLED_SELECTED_HANDLE_OPACITY, 1.0);
    assert_eq!(component::slider::ACTIVE_TRACK_HEIGHT, 4.0);
    assert_eq!(component::slider::INACTIVE_TRACK_HEIGHT, 4.0);
    assert_eq!(component::slider::HANDLE_WIDTH, 20.0);
    assert_eq!(component::slider::HANDLE_HEIGHT, 20.0);
    assert_eq!(component::slider::HANDLE_ELEVATION, 2.0);
    assert_eq!(component::slider::STATE_LAYER_SIZE, 40.0);
    assert_eq!(component::slider::LABEL_CONTAINER_HEIGHT, 28.0);
    assert_eq!(component::linear_progress::TRACK_HEIGHT, 4.0);
    assert_eq!(component::linear_progress::ACTIVE_INDICATOR_HEIGHT, 4.0);
    assert_eq!(component::linear_progress::ACTIVE_WAVE_AMPLITUDE, 3.0);
    assert_eq!(component::linear_progress::ACTIVE_WAVE_WAVELENGTH, 40.0);
    assert_eq!(
        component::linear_progress::INDETERMINATE_ACTIVE_WAVE_WAVELENGTH,
        20.0
    );
    assert_eq!(component::linear_progress::STOP_SIZE, 4.0);
    assert_eq!(component::linear_progress::TRACK_ACTIVE_SPACE, 4.0);
    assert_eq!(component::linear_progress::TRACK_THICKNESS, 4.0);
    assert_eq!(component::linear_progress::WAVE_HEIGHT, 10.0);
    assert_eq!(
        component::linear_progress::DETERMINATE_TRANSITION_DURATION_MS,
        250
    );
    assert_eq!(component::linear_progress::INDETERMINATE_DURATION_MS, 1750);
    assert_eq!(
        component::linear_progress::FIRST_LINE_HEAD_DURATION_MS,
        1000
    );
    assert_eq!(component::linear_progress::FIRST_LINE_TAIL_DELAY_MS, 250);
    assert_eq!(component::linear_progress::SECOND_LINE_HEAD_DELAY_MS, 650);
    assert_eq!(component::linear_progress::SECOND_LINE_TAIL_DELAY_MS, 900);
    assert_eq!(
        component::linear_progress::DETERMINATE_EASING,
        motion::CubicBezier::new(0.4, 0.0, 0.6, 1.0)
    );
    assert_eq!(component::loading_indicator::CONTAINER_WIDTH, 48.0);
    assert_eq!(component::loading_indicator::CONTAINER_HEIGHT, 48.0);
    assert_eq!(
        component::loading_indicator::CONTAINER_SHAPE,
        shape::CORNER_FULL
    );
    assert_eq!(component::loading_indicator::ACTIVE_SIZE, 38.0);
    assert_eq!(component::loading_indicator::MORPH_INTERVAL_MS, 650);
    assert_eq!(
        component::loading_indicator::GLOBAL_ROTATION_DURATION_MS,
        4666
    );
    assert_eq!(component::loading_indicator::INDETERMINATE_SHAPE_COUNT, 7);
    assert_eq!(component::loading_indicator::DETERMINATE_SHAPE_COUNT, 2);
    assert_eq!(
        component::loading_indicator::MORPH_SPRING_DAMPING_RATIO,
        0.6
    );
    assert_eq!(component::loading_indicator::MORPH_SPRING_STIFFNESS, 200.0);
    assert_eq!(component::radio::ICON_SIZE, 20.0);
    assert_eq!(component::radio::TARGET_SIZE, 48.0);
    assert_eq!(component::radio::STATE_LAYER_SIZE, 40.0);
    assert_eq!(component::radio::OUTER_RING_WIDTH, 2.0);
    assert_eq!(component::radio::INNER_DOT_SIZE, 10.0);
    assert_eq!(component::radio::LABEL_TEXT_SIZE, 16.0);
    assert_eq!(component::radio::LABEL_TEXT_LINE_HEIGHT, 24.0);
    assert_eq!(component::radio::LABEL_TEXT_WEIGHT, 400);
    assert_eq!(component::radio::SELECT_TRANSITION_DURATION_MS, 300);
    assert_eq!(component::radio::ICON_COLOR_TRANSITION_DURATION_MS, 50);
    assert_eq!(
        component::radio::SELECT_TRANSITION_EASING,
        motion::EASING_EMPHASIZED_DECELERATE
    );
    assert_eq!(component::text_field::CONTAINER_HEIGHT, 56.0);
    assert_eq!(component::text_field::LEADING_SPACE, 16.0);
    assert_eq!(component::text_field::TRAILING_SPACE, 16.0);
    assert_eq!(component::text_field::TOP_SPACE, 16.0);
    assert_eq!(component::text_field::BOTTOM_SPACE, 16.0);
    assert_eq!(component::text_field::INPUT_TEXT_SIZE, 16.0);
    assert_eq!(component::text_field::INPUT_TEXT_LINE_HEIGHT, 24.0);
    assert_eq!(component::text_field::INPUT_TEXT_WEIGHT, 400);
    assert_eq!(component::text_field::LABEL_TEXT_SIZE, 16.0);
    assert_eq!(component::text_field::LABEL_TEXT_LINE_HEIGHT, 24.0);
    assert_eq!(component::text_field::LABEL_TEXT_WEIGHT, 400);
    assert_eq!(component::text_field::LABEL_TEXT_PADDING_BOTTOM, 8.0);
    assert_eq!(component::text_field::LABEL_TEXT_POPULATED_SIZE, 12.0);
    assert_eq!(
        component::text_field::LABEL_TEXT_POPULATED_LINE_HEIGHT,
        16.0
    );
    assert_eq!(component::text_field::LABEL_TEXT_POPULATED_WEIGHT, 400);
    assert_eq!(component::text_field::OUTLINE_LABEL_PADDING, 4.0);
    assert_eq!(component::text_field::FOCUS_OUTLINE_WIDTH, 3.0);
    assert_eq!(component::text_field::LABEL_TRANSITION_DURATION_MS, 150);
    assert_eq!(
        component::text_field::LABEL_TRANSITION_EASING,
        motion::EASING_STANDARD
    );
    assert_eq!(component::text_field::DISABLED_LEADING_ICON_OPACITY, 0.38);
    assert_eq!(component::divider::THICKNESS, 1.0);
    assert_eq!(component::divider::LIST_ITEM_LEADING_SPACE, 16);
    assert_eq!(component::list::ONE_LINE_CONTAINER_HEIGHT, 56.0);
    assert_eq!(component::list::TWO_LINE_CONTAINER_HEIGHT, 72.0);
    assert_eq!(component::list::THREE_LINE_CONTAINER_HEIGHT, 88.0);
    assert_eq!(component::list::LEADING_SPACE, 16.0);
    assert_eq!(component::list::TRAILING_SPACE, 16.0);
    assert_eq!(component::list::TOP_SPACE, 12.0);
    assert_eq!(component::list::BOTTOM_SPACE, 12.0);
    assert_eq!(component::list::LEADING_ICON_SIZE, 24.0);
    assert_eq!(component::list::LEADING_AVATAR_SIZE, 40.0);
    assert_eq!(component::list::LABEL_TEXT, typography::BODY_LARGE);
    assert_eq!(component::list::SUPPORTING_TEXT, typography::BODY_MEDIUM);
    assert_eq!(
        component::list::TRAILING_SUPPORTING_TEXT,
        typography::LABEL_SMALL
    );
    assert_eq!(component::list::DISABLED_LABEL_TEXT_OPACITY, 0.30);
    assert_eq!(component::list::DISABLED_ICON_OPACITY, 0.38);
    assert_eq!(component::menu::CONTAINER_ELEVATION_LEVEL, 2);
    assert_eq!(component::menu::TOP_SPACE, 8.0);
    assert_eq!(component::select::MENU_CONTAINER_ELEVATION_LEVEL, 2);
    assert_eq!(component::select::MENU_LIST_ITEM_CONTAINER_HEIGHT, 48.0);
    assert_eq!(component::select::TRAILING_ICON_SIZE, 24.0);
    assert_eq!(component::select::TEXT_FIELD_DISABLED_OUTLINE_WIDTH, 1.0);
    assert_eq!(component::navigation_bar::CONTAINER_HEIGHT, 80.0);
    assert_eq!(component::navigation_bar::CONTAINER_ELEVATION_LEVEL, 2);
    assert_eq!(component::navigation_bar::ACTIVE_INDICATOR_WIDTH, 64.0);
    assert_eq!(component::navigation_bar::ACTIVE_INDICATOR_HEIGHT, 32.0);
    assert_eq!(
        component::navigation_bar::ACTIVE_INDICATOR_SHAPE,
        shape::CORNER_FULL
    );
    assert_eq!(component::navigation_bar::ICON_SIZE, 24.0);
    assert_eq!(
        component::navigation_bar::LABEL_TEXT,
        typography::LABEL_MEDIUM
    );
    assert_eq!(component::navigation_bar::ITEM_HORIZONTAL_PADDING, 8.0);
    assert_eq!(component::navigation_bar::INDICATOR_TO_LABEL_PADDING, 4.0);
    assert_eq!(component::navigation_bar::ITEM_ANIMATION_DURATION_MS, 100);
    assert_eq!(component::navigation_rail::CONTAINER_WIDTH, 96.0);
    assert_eq!(component::navigation_rail::EXPANDED_CONTAINER_WIDTH, 220.0);
    assert_eq!(component::navigation_rail::CONTAINER_ELEVATION_LEVEL, 3);
    assert_eq!(component::navigation_rail::ACTIVE_INDICATOR_WIDTH, 56.0);
    assert_eq!(component::navigation_rail::ACTIVE_INDICATOR_HEIGHT, 32.0);
    assert_eq!(
        component::navigation_rail::EXPANDED_ACTIVE_INDICATOR_HEIGHT,
        56.0
    );
    assert_eq!(
        component::navigation_rail::EXPANDED_ACTIVE_INDICATOR_MARGIN_HORIZONTAL,
        20.0
    );
    assert_eq!(
        component::navigation_rail::EXPANDED_ACTIVE_INDICATOR_PADDING_START,
        16.0
    );
    assert_eq!(
        component::navigation_rail::EXPANDED_ACTIVE_INDICATOR_PADDING_END,
        16.0
    );
    assert_eq!(component::navigation_rail::ICON_SIZE, 24.0);
    assert_eq!(component::navigation_rail::ICON_LABEL_HORIZONTAL_SPACE, 8.0);
    assert_eq!(
        component::navigation_rail::LABEL_TEXT,
        typography::LABEL_MEDIUM
    );
    assert_eq!(component::navigation_rail::ITEM_WIDTH, 96.0);
    assert_eq!(component::navigation_rail::ITEM_HEIGHT, 64.0);
    assert_eq!(component::navigation_rail::VERTICAL_PADDING, 4.0);
    assert_eq!(component::navigation_rail::CONTENT_TOP_MARGIN, 44.0);
    assert_eq!(component::navigation_rail::ITEM_TOP_PADDING, 6.0);
    assert_eq!(component::navigation_rail::HEADER_PADDING, 40.0);
    assert_eq!(component::navigation_rail::ITEM_ANIMATION_DURATION_MS, 150);
    assert_eq!(component::navigation_drawer::CONTAINER_WIDTH, 360.0);
    assert_eq!(component::navigation_drawer::MINIMUM_CONTAINER_WIDTH, 240.0);
    assert_eq!(component::navigation_drawer::ACTIVE_INDICATOR_WIDTH, 336.0);
    assert_eq!(component::navigation_drawer::ACTIVE_INDICATOR_HEIGHT, 56.0);
    assert_eq!(
        component::navigation_drawer::ACTIVE_INDICATOR_SHAPE,
        shape::CORNER_FULL
    );
    assert_eq!(
        component::navigation_drawer::LABEL_TEXT,
        typography::LABEL_LARGE
    );
    assert_eq!(
        component::navigation_drawer::HEADLINE_TEXT,
        typography::TITLE_SMALL
    );
    assert_eq!(component::navigation_drawer::ITEM_HORIZONTAL_PADDING, 12.0);
    assert_eq!(component::navigation_drawer::LABEL_BADGE_SPACE, 12.0);
    assert_eq!(
        component::navigation_drawer::MODAL_CONTAINER_ELEVATION_LEVEL,
        1
    );
    assert_eq!(
        component::navigation_drawer::STANDARD_CONTAINER_ELEVATION_LEVEL,
        0
    );
    assert_eq!(component::adaptive_navigation::WIDTH_COMPACT_MAX, 600.0);
    assert_eq!(component::adaptive_navigation::WIDTH_MEDIUM_MAX, 840.0);
    assert_eq!(component::adaptive_navigation::HEIGHT_COMPACT_MAX, 480.0);
    assert_eq!(component::adaptive_navigation::HEIGHT_MEDIUM_MAX, 900.0);
    assert_eq!(component::dialog::CONTAINER_ELEVATION_LEVEL, 3);
    assert_eq!(component::dialog::CONTAINER_MIN_WIDTH, 280.0);
    assert_eq!(component::dialog::CONTAINER_MAX_WIDTH, 560.0);
    assert_eq!(component::dialog::CONTAINER_PADDING, 24.0);
    assert_eq!(component::dialog::ICON_BOTTOM_PADDING, 16.0);
    assert_eq!(component::dialog::TITLE_BOTTOM_PADDING, 16.0);
    assert_eq!(component::dialog::SUPPORTING_TEXT_BOTTOM_PADDING, 24.0);
    assert_eq!(component::dialog::ACTIONS_HORIZONTAL_SPACING, 8.0);
    assert_eq!(component::dialog::ACTIONS_VERTICAL_SPACING, 8.0);
    assert_eq!(component::dialog::SCRIM_OPACITY, 0.32);
    assert_eq!(component::dialog::ENTER_SCALE_FROM, 0.9);
    assert_eq!(component::dialog::EXIT_SCALE_TO, 0.9);
    assert_eq!(component::dialog::SCALE_ANIMATION_DURATION_MS, 220);
    assert_eq!(component::dialog::ALPHA_ANIMATION_DURATION_MS, 150);
    assert_eq!(component::dialog::SCRIM_ANIMATION_DURATION_MS, 220);
    assert_eq!(component::dialog::DECELERATE_CUBIC_FACTOR, 1.5);
    assert_eq!(component::dialog::DECELERATE_QUINT_FACTOR, 2.5);
    assert_eq!(
        component::dialog::ACTION_LABEL_TEXT,
        typography::LABEL_LARGE
    );
    assert_eq!(component::dialog::HEADLINE_TEXT, typography::HEADLINE_SMALL);
    assert_eq!(component::dialog::SUPPORTING_TEXT, typography::BODY_MEDIUM);
    assert_eq!(component::data_table::CONTAINER_SHAPE, 4.0);
    assert_eq!(component::data_table::OUTLINE_WIDTH, 1.0);
    assert_eq!(component::data_table::HEADER_CONTAINER_HEIGHT, 56.0);
    assert_eq!(component::data_table::ROW_ITEM_CONTAINER_HEIGHT, 52.0);
    assert_eq!(component::card::CONTAINER_SHAPE, 12.0);
    assert_eq!(component::card::ICON_SIZE, 24.0);
    assert_eq!(component::card::ELEVATED_ELEVATION.active, 1);
    assert_eq!(component::card::ELEVATED_ELEVATION.hovered, 2);
    assert_eq!(component::card::FILLED_ELEVATION.dragged, 3);
    assert_eq!(component::card::OUTLINED_OUTLINE_WIDTH, 1.0);
    assert_eq!(component::fab::CONTAINER_WIDTH, 56.0);
    assert_eq!(component::fab::CONTAINER_HEIGHT, 56.0);
    assert_eq!(component::fab::CONTAINER_SHAPE, 16.0);
    assert_eq!(component::fab::ICON_SIZE, 24.0);
    assert_eq!(component::fab::SMALL_CONTAINER_WIDTH, 40.0);
    assert_eq!(component::fab::SMALL_CONTAINER_HEIGHT, 40.0);
    assert_eq!(component::fab::SMALL_CONTAINER_SHAPE, shape::CORNER_MEDIUM);
    assert_eq!(component::fab::SMALL_ICON_SIZE, 24.0);
    assert_eq!(component::fab::LARGE_CONTAINER_WIDTH, 96.0);
    assert_eq!(component::fab::LARGE_CONTAINER_HEIGHT, 96.0);
    assert_eq!(
        component::fab::LARGE_CONTAINER_SHAPE,
        shape::CORNER_EXTRA_LARGE
    );
    assert_eq!(component::fab::LARGE_ICON_SIZE, 36.0);
    assert_eq!(component::fab::EXTENDED_CONTAINER_HEIGHT, 56.0);
    assert_eq!(
        component::fab::EXTENDED_CONTAINER_SHAPE,
        shape::CORNER_LARGE
    );
    assert_eq!(component::fab::EXTENDED_ICON_SIZE, 24.0);
    assert_eq!(component::fab::EXTENDED_ICON_LABEL_SPACE, 12.0);
    assert_eq!(component::fab::EXTENDED_LEADING_SPACE, 16.0);
    assert_eq!(component::fab::EXTENDED_TRAILING_SPACE, 20.0);
    assert_eq!(component::fab::EXTENDED_LABEL_TEXT, typography::LABEL_LARGE);
    assert_eq!(component::fab::ELEVATION.active, 3);
    assert_eq!(component::fab::ELEVATION.hovered, 4);
    assert_eq!(component::fab::EXTENDED_ELEVATION.active, 3);
    assert_eq!(component::fab::EXTENDED_ELEVATION.hovered, 4);
    assert_eq!(component::icon_button::CONTAINER_WIDTH, 40.0);
    assert_eq!(component::icon_button::CONTAINER_HEIGHT, 40.0);
    assert_eq!(component::icon_button::MINIMUM_INTERACTIVE_SIZE, 48.0);
    assert_eq!(component::icon_button::CONTAINER_SHAPE, 9999.0);
    assert_eq!(component::icon_button::ICON_SIZE, 24.0);
    assert_eq!(component::icon_button::DISABLED_CONTAINER_OPACITY, 0.12);
    assert_eq!(component::icon_button::OUTLINED_OUTLINE_WIDTH, 1.0);
    assert_eq!(component::chip::CONTAINER_HEIGHT, 32.0);
    assert_eq!(component::chip::CONTAINER_SHAPE, 8.0);
    assert_eq!(component::chip::OUTLINE_WIDTH, 1.0);
    assert_eq!(component::chip::SELECTED_OUTLINE_WIDTH, 0.0);
    assert_eq!(component::chip::ICON_SIZE, 18.0);
    assert_eq!(component::chip::LABEL_TEXT_SIZE, 14.0);
    assert_eq!(component::chip::LABEL_TEXT_LINE_HEIGHT, 20.0);
    assert_eq!(component::chip::LABEL_TEXT_WEIGHT, 500);
    assert_eq!(component::chip::LEADING_SPACE, 16.0);
    assert_eq!(component::chip::TRAILING_SPACE, 16.0);
    assert_eq!(component::chip::ICON_LABEL_SPACE, 8.0);
    assert_eq!(component::chip::WITH_LEADING_ICON_LEADING_SPACE, 8.0);
    assert_eq!(component::chip::WITH_TRAILING_ICON_TRAILING_SPACE, 8.0);
    assert_eq!(component::chip::AVATAR_SIZE, 24.0);
    assert_eq!(component::chip::ELEVATED_ELEVATION.active, 1);
    assert_eq!(component::chip::ELEVATED_ELEVATION.hovered, 2);
    assert_eq!(component::chip::SELECTED_FLAT_ELEVATION.hovered, 1);
    assert_eq!(component::segmented_button::CONTAINER_HEIGHT, 40.0);
    assert_eq!(
        component::segmented_button::CONTAINER_SHAPE,
        shape::CORNER_FULL
    );
    assert_eq!(component::segmented_button::OUTLINE_WIDTH, 1.0);
    assert_eq!(component::segmented_button::WITH_ICON_ICON_SIZE, 18.0);
    assert_eq!(component::segmented_button::LEADING_SPACE, 12.0);
    assert_eq!(component::segmented_button::TRAILING_SPACE, 12.0);
    assert_eq!(component::segmented_button::ICON_LABEL_SPACE, 8.0);
    assert_eq!(
        component::segmented_button::LABEL_TEXT,
        typography::LABEL_LARGE
    );
    assert_eq!(
        component::segmented_button::DISABLED_LABEL_TEXT_OPACITY,
        0.38
    );
    assert_eq!(component::segmented_button::DISABLED_OUTLINE_OPACITY, 0.12);
    assert_eq!(
        component::segmented_button::SELECT_TRANSITION_DURATION_MS,
        200
    );
    assert_eq!(
        component::segmented_button::SELECT_TRANSITION_EASING,
        motion::EASING_EMPHASIZED
    );
    assert_eq!(component::snackbar::ICON_SIZE, 24.0);
    assert_eq!(component::snackbar::WITH_SINGLE_LINE_CONTAINER_HEIGHT, 48.0);
    assert_eq!(component::snackbar::WITH_TWO_LINES_CONTAINER_HEIGHT, 68.0);
    assert_eq!(component::snackbar::MAX_WIDTH, 568.0);
    assert_eq!(component::snackbar::HORIZONTAL_MARGIN, 16.0);
    assert_eq!(component::snackbar::BOTTOM_MARGIN, 16.0);
    assert_eq!(component::snackbar::CONTAINER_ELEVATION_LEVEL, 3);
    assert_eq!(
        component::snackbar::CONTAINER_SHAPE,
        shape::CORNER_EXTRA_SMALL
    );
    assert_eq!(
        component::snackbar::SUPPORTING_TEXT,
        typography::BODY_MEDIUM
    );
    assert_eq!(
        component::snackbar::ACTION_LABEL_TEXT,
        typography::LABEL_LARGE
    );
    assert_eq!(component::snackbar::SLIDE_ANIMATION_DURATION_MS, 250);
    assert_eq!(component::snackbar::CONTENT_FADE_ANIMATION_DURATION_MS, 180);
    assert_eq!(component::snackbar::LONG_DURATION_MS, 2750);
    assert_eq!(
        component::snackbar::SLIDE_ANIMATION_EASING,
        motion::EASING_LEGACY
    );
    assert_eq!(
        component::snackbar::CONTENT_FADE_ANIMATION_EASING,
        motion::EASING_LEGACY
    );
    assert_eq!(component::search_bar::AVATAR_SIZE, 30.0);
    assert_eq!(component::search_bar::CONTAINER_HEIGHT, 56.0);
    assert_eq!(component::search_bar::ICON_SIZE, 24.0);
    assert_eq!(component::search_bar::LEADING_SPACE, 16.0);
    assert_eq!(component::search_bar::TRAILING_SPACE, 16.0);
    assert_eq!(component::search_bar::LEADING_ICON_LABEL_SPACE, 16.0);
    assert_eq!(component::search_bar::CONTAINER_ELEVATION_LEVEL, 3);
    assert_eq!(component::search_bar::CONTAINER_SHAPE, shape::CORNER_FULL);
    assert_eq!(component::search_bar::INPUT_TEXT, typography::BODY_LARGE);
    assert_eq!(component::search_view::DOCKED_HEADER_CONTAINER_HEIGHT, 56.0);
    assert_eq!(
        component::search_view::FULL_SCREEN_HEADER_CONTAINER_HEIGHT,
        72.0
    );
    assert_eq!(
        component::search_view::DOCKED_CONTAINER_SHAPE,
        shape::CORNER_EXTRA_LARGE
    );
    assert_eq!(
        component::search_view::FULL_SCREEN_CONTAINER_SHAPE,
        shape::CORNER_NONE
    );
    assert_eq!(component::app_bar::AVATAR_SIZE, 32.0);
    assert_eq!(component::app_bar::ICON_BUTTON_SPACE, 0.0);
    assert_eq!(component::app_bar::ICON_SIZE, 24.0);
    assert_eq!(component::app_bar::LEADING_SPACE, 4.0);
    assert_eq!(component::app_bar::TRAILING_SPACE, 4.0);
    assert_eq!(component::app_bar::TITLE_LEADING_SPACE, 16.0);
    assert_eq!(component::app_bar::CONTAINER_ELEVATION_LEVEL, 0);
    assert_eq!(component::app_bar::ON_SCROLL_CONTAINER_ELEVATION_LEVEL, 2);
    assert_eq!(component::app_bar::SMALL_CONTAINER_HEIGHT, 64.0);
    assert_eq!(component::app_bar::SMALL_SEARCH_CONTAINER_HEIGHT, 56.0);
    assert_eq!(
        component::app_bar::SMALL_TITLE_TEXT,
        typography::TITLE_LARGE
    );
    assert_eq!(component::app_bar::MEDIUM_CONTAINER_HEIGHT, 112.0);
    assert_eq!(
        component::app_bar::MEDIUM_TITLE_TEXT,
        typography::HEADLINE_SMALL
    );
    assert_eq!(component::app_bar::LARGE_CONTAINER_HEIGHT, 152.0);
    assert_eq!(
        component::app_bar::LARGE_TITLE_TEXT,
        typography::HEADLINE_MEDIUM
    );
    assert_eq!(component::bottom_app_bar::CONTAINER_HEIGHT, 80.0);
    assert_eq!(component::bottom_app_bar::CONTAINER_ELEVATION_LEVEL, 2);
    assert_eq!(
        component::bottom_app_bar::CONTAINER_SHAPE,
        shape::CORNER_NONE
    );
    assert_eq!(component::toolbar::DOCKED_CONTAINER_HEIGHT, 64.0);
    assert_eq!(component::toolbar::DOCKED_LEADING_SPACE, 16.0);
    assert_eq!(component::toolbar::DOCKED_TRAILING_SPACE, 16.0);
    assert_eq!(
        component::toolbar::DOCKED_CONTAINER_SHAPE,
        shape::CORNER_NONE
    );
    assert_eq!(
        component::toolbar::FLOATING_HORIZONTAL_CONTAINER_HEIGHT,
        64.0
    );
    assert_eq!(component::toolbar::FLOATING_VERTICAL_CONTAINER_WIDTH, 64.0);
    assert_eq!(component::toolbar::FLOATING_CONTAINER_LEADING_SPACE, 8.0);
    assert_eq!(component::toolbar::FLOATING_CONTAINER_TRAILING_SPACE, 8.0);
    assert_eq!(
        component::toolbar::FLOATING_CONTAINER_SHAPE,
        shape::CORNER_FULL
    );
    assert_eq!(component::toolbar::FLOATING_CONTAINER_ELEVATION_LEVEL, 3);
    assert_eq!(component::toolbar::ACTION_CONTAINER_WIDTH, 48.0);
    assert_eq!(component::toolbar::ACTION_CONTAINER_HEIGHT, 48.0);
    assert_eq!(component::toolbar::ACTION_ICON_SIZE, 24.0);
    assert_eq!(
        component::bottom_sheet::CONTAINER_SHAPE_TOP,
        shape::CORNER_EXTRA_LARGE
    );
    assert_eq!(component::bottom_sheet::CONTAINER_SHAPE_BOTTOM, 0.0);
    assert_eq!(component::bottom_sheet::MODAL_CONTAINER_ELEVATION_LEVEL, 1);
    assert_eq!(
        component::bottom_sheet::STANDARD_CONTAINER_ELEVATION_LEVEL,
        1
    );
    assert_eq!(component::bottom_sheet::DRAG_HANDLE_WIDTH, 32.0);
    assert_eq!(component::bottom_sheet::DRAG_HANDLE_HEIGHT, 4.0);
    assert_eq!(component::bottom_sheet::DRAG_HANDLE_VERTICAL_PADDING, 22.0);
    assert_eq!(component::bottom_sheet::CONTENT_PADDING, 24.0);
    assert_eq!(component::bottom_sheet::SHEET_PEEK_HEIGHT, 56.0);
    assert_eq!(component::bottom_sheet::SHEET_MAX_WIDTH, 640.0);
    assert_eq!(component::bottom_sheet::SCRIM_OPACITY, 0.32);
    assert_eq!(component::bottom_sheet::POSITIONAL_THRESHOLD, 56.0);
    assert_eq!(component::bottom_sheet::VELOCITY_THRESHOLD, 125.0);
    assert_eq!(component::bottom_sheet::ANIMATION_DURATION_MS, 300);
    assert_eq!(
        component::bottom_sheet::ANIMATION_EASING,
        motion::EASING_LEGACY
    );
    assert_eq!(component::side_sheet::DOCKED_CONTAINER_WIDTH, 256.0);
    assert_eq!(component::side_sheet::DETACHED_MARGIN, 16.0);
    assert_eq!(component::side_sheet::CONTENT_PADDING, 24.0);
    assert_eq!(
        component::side_sheet::DOCKED_STANDARD_CONTAINER_SHAPE,
        shape::CORNER_NONE
    );
    assert_eq!(
        component::side_sheet::DOCKED_MODAL_CONTAINER_SHAPE,
        shape::CORNER_LARGE
    );
    assert_eq!(
        component::side_sheet::DETACHED_CONTAINER_SHAPE,
        shape::CORNER_LARGE
    );
    assert_eq!(component::side_sheet::MODAL_CONTAINER_ELEVATION_LEVEL, 1);
    assert_eq!(component::side_sheet::STANDARD_CONTAINER_ELEVATION_LEVEL, 0);
    assert_eq!(component::side_sheet::SCRIM_OPACITY, 0.32);
    assert_eq!(component::side_sheet::ANIMATION_DURATION_MS, 275);
    assert_eq!(
        component::side_sheet::ANIMATION_EASING,
        motion::EASING_EMPHASIZED
    );
    assert_eq!(component::primary_tab::CONTAINER_HEIGHT, 48.0);
    assert_eq!(
        component::primary_tab::WITH_ICON_AND_LABEL_TEXT_CONTAINER_HEIGHT,
        64.0
    );
    assert_eq!(component::primary_tab::CONTAINER_ELEVATION_LEVEL, 0);
    assert_eq!(component::primary_tab::CONTAINER_SHAPE, shape::CORNER_NONE);
    assert_eq!(component::primary_tab::ACTIVE_INDICATOR_HEIGHT, 3.0);
    assert_eq!(component::primary_tab::ACTIVE_INDICATOR_SHAPE_TOP, 3.0);
    assert_eq!(component::primary_tab::ACTIVE_INDICATOR_SHAPE_BOTTOM, 0.0);
    assert_eq!(component::primary_tab::ICON_SIZE, 24.0);
    assert_eq!(component::primary_tab::HORIZONTAL_SPACE, 16.0);
    assert_eq!(component::primary_tab::INLINE_ICON_LABEL_SPACE, 8.0);
    assert_eq!(component::primary_tab::STACKED_ICON_LABEL_SPACE, 2.0);
    assert_eq!(component::primary_tab::LABEL_TEXT, typography::TITLE_SMALL);
    assert_eq!(component::primary_tab::INDICATOR_ANIMATION_DURATION_MS, 250);
    assert_eq!(
        component::primary_tab::INDICATOR_ANIMATION_EASING,
        motion::EASING_EMPHASIZED
    );
    assert_eq!(component::secondary_tab::CONTAINER_HEIGHT, 48.0);
    assert_eq!(component::secondary_tab::CONTAINER_ELEVATION_LEVEL, 0);
    assert_eq!(
        component::secondary_tab::CONTAINER_SHAPE,
        shape::CORNER_NONE
    );
    assert_eq!(component::secondary_tab::ACTIVE_INDICATOR_HEIGHT, 2.0);
    assert_eq!(
        component::secondary_tab::ACTIVE_INDICATOR_SHAPE,
        shape::CORNER_NONE
    );
    assert_eq!(component::secondary_tab::ICON_SIZE, 24.0);
    assert_eq!(component::secondary_tab::HORIZONTAL_SPACE, 16.0);
    assert_eq!(component::secondary_tab::ICON_LABEL_SPACE, 8.0);
    assert_eq!(
        component::secondary_tab::LABEL_TEXT,
        typography::TITLE_SMALL
    );
    assert_eq!(
        component::secondary_tab::INDICATOR_ANIMATION_DURATION_MS,
        250
    );
    assert_eq!(
        component::secondary_tab::INDICATOR_ANIMATION_EASING,
        motion::EASING_EMPHASIZED
    );
    assert_eq!(component::tooltip::SPACING_BETWEEN_TOOLTIP_AND_ANCHOR, 4.0);
    assert_eq!(component::tooltip::PLAIN_MIN_HEIGHT, 24.0);
    assert_eq!(component::tooltip::PLAIN_MIN_WIDTH, 40.0);
    assert_eq!(component::tooltip::PLAIN_MAX_WIDTH, 200.0);
    assert_eq!(component::tooltip::PLAIN_HORIZONTAL_SPACE, 8.0);
    assert_eq!(component::tooltip::PLAIN_VERTICAL_SPACE, 4.0);
    assert_eq!(component::tooltip::PLAIN_CONTAINER_SHAPE, 4.0);
    assert_eq!(component::tooltip::PLAIN_SUPPORTING_TEXT.size, 12.0);
    assert_eq!(component::tooltip::FADE_IN_DURATION_MS, 150);
    assert_eq!(component::tooltip::FADE_OUT_DURATION_MS, 75);
    assert_eq!(component::tooltip::SCALE_START, 0.8);
    assert_eq!(component::tooltip::ANIMATION_DURATION_MS, 150);
    assert_eq!(component::tooltip::RICH_MAX_WIDTH, 320.0);
    assert_eq!(component::tooltip::RICH_MIN_HEIGHT, 24.0);
    assert_eq!(component::tooltip::RICH_MIN_WIDTH, 40.0);
    assert_eq!(component::tooltip::RICH_HORIZONTAL_SPACE, 16.0);
    assert_eq!(
        component::tooltip::RICH_TEXT_VERTICAL_SPACE_WITHOUT_TITLE_OR_ACTION,
        4.0
    );
    assert_eq!(component::tooltip::RICH_HEIGHT_TO_SUBHEAD_FIRST_LINE, 28.0);
    assert_eq!(
        component::tooltip::RICH_HEIGHT_FROM_SUBHEAD_TO_TEXT_FIRST_LINE,
        24.0
    );
    assert_eq!(component::tooltip::RICH_TEXT_BOTTOM_PADDING, 16.0);
    assert_eq!(component::tooltip::RICH_ACTION_LABEL_MIN_HEIGHT, 36.0);
    assert_eq!(component::tooltip::RICH_ACTION_LABEL_BOTTOM_PADDING, 8.0);
    assert_eq!(component::tooltip::RICH_CONTAINER_SHAPE, 12.0);
    assert_eq!(component::tooltip::RICH_CONTAINER_ELEVATION_LEVEL, 2);
    assert_eq!(component::tooltip::RICH_SUBHEAD_TEXT.size, 14.0);
    assert_eq!(component::tooltip::RICH_SUPPORTING_TEXT.size, 14.0);
    assert_eq!(component::tooltip::RICH_ACTION_LABEL_TEXT.size, 14.0);
}

#[test]
fn m3_typography_tokens_match_google_values() {
    assert_eq!(typography::DISPLAY_LARGE.size, 57.0);
    assert_eq!(typography::DISPLAY_LARGE.line_height, 64.0);
    assert_eq!(typography::DISPLAY_LARGE.tracking, -0.25);
    assert_eq!(typography::LABEL_LARGE.size, 14.0);
    assert_eq!(typography::LABEL_LARGE.weight, 500);
    assert_eq!(typography::BODY_MEDIUM.tracking, 0.25);
}
