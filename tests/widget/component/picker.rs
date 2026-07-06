use super::*;

#[test]
fn date_round_trips_utc_millis() {
    let date = Date::new(2026, 7, 4).unwrap();

    assert_eq!(Date::from_utc_millis(date.to_utc_millis()), date);
    assert_eq!(date.weekday(), Weekday::Saturday);
    assert_eq!(date.weekday_index_from(Weekday::Sunday), 6);
    assert_eq!(date.weekday_index_from(Weekday::Monday), 5);
}

#[test]
fn month_addition_crosses_years() {
    assert_eq!(
        YearMonth::new(2025, 12).unwrap().add_months(1),
        YearMonth::new(2026, 1).unwrap()
    );
    assert_eq!(
        YearMonth::new(2026, 1).unwrap().add_months(-1),
        YearMonth::new(2025, 12).unwrap()
    );
}

#[test]
fn date_input_accepts_common_material_patterns() {
    assert_eq!(parse_date_input("07/04/2026"), Date::new(2026, 7, 4));
    assert_eq!(parse_date_input("07042026"), Date::new(2026, 7, 4));
    assert_eq!(parse_date_input("2026-07-04"), Date::new(2026, 7, 4));
    assert_eq!(parse_date_input("02/30/2026"), None);
    assert_eq!(normalize_date_input("07042026".into()), "07/04/2026");
    assert_eq!(normalize_date_input("2026-07-04".into()), "07/04/2026");
}

#[test]
fn date_picker_input_validity_allows_empty_and_rejects_invalid_dates() {
    let selected = Date::new(2026, 7, 4).unwrap();
    let mut state = DatePickerState::new(Some(selected));

    assert!(state.is_input_valid());

    state.update(DatePickerAction::InputChanged("02/30/2026".into()));
    assert!(!state.is_input_valid());
    assert_eq!(state.selected_date(), None);

    state.update(DatePickerAction::InputChanged("0704".into()));
    assert!(state.is_input_valid());
    assert_eq!(state.input_value(), "07/04");
    assert_eq!(state.selected_date(), None);

    state.update(DatePickerAction::InputChanged("07042026".into()));
    assert!(state.is_input_valid());
    assert_eq!(state.input_value(), "07/04/2026");
    assert_eq!(state.selected_date(), Some(selected));
}

#[test]
fn date_input_error_text_matches_material_strings() {
    let selectable_dates = SelectableDates::all();
    let formatter = DatePickerFormatter::default();

    assert_eq!(
        date_input_error(
            "13/99/2026",
            2020..=2030,
            &selectable_dates,
            &formatter,
            None,
        ),
        Some("Date does not match expected pattern: MM/DD/YYYY".to_owned())
    );
    assert_eq!(
        date_input_error(
            "07/04/2031",
            2020..=2030,
            &selectable_dates,
            &formatter,
            None,
        ),
        Some("Date out of expected year range 2020 - 2030".to_owned())
    );

    let blocked = SelectableDates::new(|date| date.day != 4, |_| true);
    assert_eq!(
        date_input_error("07/04/2026", 2020..=2030, &blocked, &formatter, None),
        Some("Date not allowed: Jul 4, 2026".to_owned())
    );
    assert_eq!(
        date_input_error(
            "07/05/2026",
            2020..=2030,
            &selectable_dates,
            &formatter,
            Some(DateInputRangeBound::StartBeforeOrEqual(
                Date::new(2026, 7, 4).unwrap(),
            )),
        ),
        Some("Invalid date range input".to_owned())
    );
}

#[test]
fn date_picker_supports_first_day_of_week_and_custom_formatter() {
    let date = Date::new(2026, 7, 4).unwrap();
    let formatter = DatePickerFormatter::new(
        |month| format!("{:04}-{:02}", month.year, month.month),
        |date, for_content_description| {
            if for_content_description {
                format!("{}-{:02}-{:02} full", date.year, date.month, date.day)
            } else {
                format!("{}-{:02}-{:02}", date.year, date.month, date.day)
            }
        },
    );
    let state = DatePickerState::new(Some(date))
        .first_day_of_week(Weekday::Monday)
        .formatter(formatter);

    assert_eq!(state.calendar_first_day_of_week(), Weekday::Monday);
    assert_eq!(
        state
            .date_formatter()
            .format_month_year(state.displayed_month()),
        "2026-07"
    );
    assert_eq!(
        state.date_formatter().format_date(date, false),
        "2026-07-04"
    );
    assert_eq!(
        state.date_formatter().format_date(date, true),
        "2026-07-04 full"
    );
}

#[test]
fn date_picker_initial_displayed_month_is_independent_from_selection() {
    let selected = Date::new(2026, 7, 4).unwrap();
    let august = YearMonth::new(2026, 8).unwrap();
    let ignored = YearMonth::new(2028, 5).unwrap();
    let state = DatePickerState::new(Some(selected))
        .year_range(2026..=2026)
        .initial_displayed_month(august)
        .initial_displayed_month(ignored);

    assert_eq!(state.selected_date(), Some(selected));
    assert_eq!(state.displayed_month(), august);
}

#[test]
fn year_picker_initial_scroll_offset_matches_compose_index_rule() {
    let year_range = 2020..=2030;

    assert_eq!(year_picker_initial_item_index(&year_range, 2020), 0);
    assert_eq!(
        year_picker_displayed_year_scroll_offset(&year_range, 2020),
        0.0
    );
    assert_eq!(year_picker_initial_item_index(&year_range, 2026), 3);
    assert_eq!(
        year_picker_displayed_year_scroll_offset(&year_range, 2026),
        year_picker_row_height()
    );
    assert_eq!(year_picker_initial_item_index(&year_range, 2030), 7);
    assert_eq!(
        year_picker_displayed_year_scroll_offset(&year_range, 2030),
        2.0 * year_picker_row_height()
    );
}

#[test]
fn selectable_dates_block_date_and_year_selection() {
    let blocked = Date::new(2026, 7, 4).unwrap();
    let allowed = Date::new(2026, 7, 5).unwrap();
    let selectable = SelectableDates::new(move |date| date != blocked, |year| year != 2027);
    let mut state = DatePickerState::new(None)
        .year_range(2026..=2027)
        .selectable_dates(selectable);

    state.update(DatePickerAction::SelectDate(blocked));
    assert_eq!(state.selected_date(), None);

    state.update(DatePickerAction::SelectDate(allowed));
    assert_eq!(state.selected_date(), Some(allowed));

    state.update(DatePickerAction::InputChanged("07/04/2026".into()));
    assert_eq!(state.selected_date(), None);
    assert!(state.input_error().is_some());
}

#[test]
fn date_range_picker_selects_start_then_end_and_restarts_after_complete_range() {
    let start = Date::new(2026, 7, 4).unwrap();
    let end = Date::new(2026, 7, 10).unwrap();
    let restart = Date::new(2026, 8, 1).unwrap();
    let mut state = DateRangePickerState::new(None, None);

    state.update(DateRangePickerAction::SelectDate(start));
    assert_eq!(state.selected_start_date(), Some(start));
    assert_eq!(state.selected_end_date(), None);

    state.update(DateRangePickerAction::SelectDate(end));
    assert_eq!(state.selected_start_date(), Some(start));
    assert_eq!(state.selected_end_date(), Some(end));
    assert_eq!(state.date_range_position(start), DateRangePosition::Start);
    assert_eq!(
        state.date_range_position(Date::new(2026, 7, 7).unwrap()),
        DateRangePosition::Middle
    );
    assert_eq!(state.date_range_position(end), DateRangePosition::End);

    state.update(DateRangePickerAction::SelectDate(restart));
    assert_eq!(state.selected_start_date(), Some(restart));
    assert_eq!(state.selected_end_date(), None);
    assert_eq!(
        state.date_range_position(restart),
        DateRangePosition::Single
    );
}

#[test]
fn date_range_initial_selection_matches_material_validation_rules() {
    let start = Date::new(2026, 7, 10).unwrap();
    let earlier_end = Date::new(2026, 7, 4).unwrap();
    let end = Date::new(2026, 7, 12).unwrap();

    let state = DateRangePickerState::new(None, Some(end));
    assert_eq!(state.selected_start_date(), None);
    assert_eq!(state.selected_end_date(), None);
    assert_eq!(state.start_input_value(), "");
    assert_eq!(state.end_input_value(), "");

    let state = DateRangePickerState::new(Some(start), Some(earlier_end));
    assert_eq!(state.selected_start_date(), None);
    assert_eq!(state.selected_end_date(), None);

    let state = DateRangePickerState::new(Some(start), Some(end));
    assert_eq!(state.selected_start_date(), Some(start));
    assert_eq!(state.selected_end_date(), Some(end));
}

#[test]
fn date_range_initial_displayed_month_is_independent_from_selection() {
    let start = Date::new(2026, 7, 4).unwrap();
    let end = Date::new(2026, 7, 10).unwrap();
    let september = YearMonth::new(2026, 9).unwrap();
    let ignored = YearMonth::new(2028, 5).unwrap();
    let state = DateRangePickerState::new(Some(start), Some(end))
        .year_range(2026..=2026)
        .initial_displayed_month(september)
        .initial_displayed_month(ignored);

    assert_eq!(state.selected_start_date(), Some(start));
    assert_eq!(state.selected_end_date(), Some(end));
    assert_eq!(state.displayed_month(), september);
    assert_eq!(state.start_input_value(), "07/04/2026");
    assert_eq!(state.end_input_value(), "07/10/2026");
}

#[test]
fn date_range_builders_clear_invalid_selection_and_sync_inputs() {
    let start = Date::new(2026, 7, 4).unwrap();
    let end = Date::new(2026, 7, 10).unwrap();
    let state = DateRangePickerState::new(Some(start), Some(end)).year_range(2027..=2028);

    assert_eq!(state.selected_start_date(), None);
    assert_eq!(state.selected_end_date(), None);
    assert_eq!(state.start_input_value(), "");
    assert_eq!(state.end_input_value(), "");

    let selectable = SelectableDates::new(move |date| date != end, |_| true);
    let state = DateRangePickerState::new(Some(start), Some(end)).selectable_dates(selectable);

    assert_eq!(state.selected_start_date(), Some(start));
    assert_eq!(state.selected_end_date(), None);
    assert_eq!(state.start_input_value(), "07/04/2026");
    assert_eq!(state.end_input_value(), "");
}

#[test]
fn date_range_month_model_uses_official_year_range_indexing() {
    let mut state = DateRangePickerState::new(Some(Date::new(2026, 11, 1).unwrap()), None)
        .year_range(2026..=2026);

    let months = range_visible_months(&state);

    assert_eq!(months.first(), Some(&YearMonth::new(2026, 1).unwrap()));
    assert_eq!(months.last(), Some(&YearMonth::new(2026, 12).unwrap()));
    assert_eq!(months.len(), 12);
    assert_eq!(
        range_month_index(&state, YearMonth::new(2026, 11).unwrap()),
        10
    );
    assert_eq!(
        range_month_at_index(&state, 10),
        Some(YearMonth::new(2026, 11).unwrap())
    );

    state.update(DateRangePickerAction::PreviousMonth);

    assert_eq!(range_month_index(&state, state.displayed_month()), 9);
}

#[test]
fn date_range_render_window_keeps_displayed_month_near_top_for_large_ranges() {
    let state = DateRangePickerState::new(Some(Date::new(2026, 11, 1).unwrap()), None);
    let rendered = range_rendered_months(&state);

    assert_eq!(rendered.first(), Some(&YearMonth::new(2026, 9).unwrap()));
    assert_eq!(
        rendered[usize::try_from(RANGE_PICKER_RENDER_MONTHS_BEFORE).unwrap()],
        YearMonth::new(2026, 11).unwrap()
    );
    assert_eq!(
        range_displayed_month_scroll_offset(&state),
        RANGE_PICKER_RENDER_MONTHS_BEFORE as f32 * range_month_item_height()
    );
}

#[test]
fn date_range_month_selection_info_spans_partial_months() {
    let start = Date::new(2026, 7, 10).unwrap();
    let end = Date::new(2026, 8, 5).unwrap();
    let state = DateRangePickerState::new(Some(start), Some(end));

    assert_eq!(
        range_month_selection_info(&state, YearMonth::new(2026, 7).unwrap()),
        Some(RangeMonthSelectionInfo {
            start_column: 5,
            start_row: 1,
            end_column: 5,
            end_row: 4,
            first_is_selection_start: true,
            last_is_selection_end: false,
        })
    );
    assert_eq!(
        range_month_selection_info(&state, YearMonth::new(2026, 8).unwrap()),
        Some(RangeMonthSelectionInfo {
            start_column: 6,
            start_row: 0,
            end_column: 3,
            end_row: 1,
            first_is_selection_start: false,
            last_is_selection_end: true,
        })
    );
    assert_eq!(
        range_month_selection_info(&state, YearMonth::new(2026, 9).unwrap()),
        None
    );
}

#[test]
fn date_range_background_rects_match_compose_ltr_geometry() {
    let grid_width = tokens::component::date_picker::CALENDAR_CELL_SIZE * 7.0;

    assert_eq!(
        range_background_rects(
            RangeMonthSelectionInfo {
                start_column: 5,
                start_row: 1,
                end_column: 6,
                end_row: 1,
                first_is_selection_start: true,
                last_is_selection_end: true,
            },
            grid_width,
        ),
        vec![RangeBackgroundRect {
            x: 264.0,
            y: 52.0,
            width: 48.0,
            height: 40.0,
        }]
    );

    assert_eq!(
        range_background_rects(
            RangeMonthSelectionInfo {
                start_column: 5,
                start_row: 1,
                end_column: 2,
                end_row: 3,
                first_is_selection_start: true,
                last_is_selection_end: true,
            },
            grid_width,
        ),
        vec![
            RangeBackgroundRect {
                x: 264.0,
                y: 52.0,
                width: 72.0,
                height: 40.0,
            },
            RangeBackgroundRect {
                x: 0.0,
                y: 100.0,
                width: 336.0,
                height: 40.0,
            },
            RangeBackgroundRect {
                x: 0.0,
                y: 148.0,
                width: 120.0,
                height: 40.0,
            },
        ]
    );
}

#[test]
fn date_range_background_rows_keep_rounded_shape() {
    let grid_width = tokens::component::date_picker::CALENDAR_CELL_SIZE * 7.0;
    let rects = range_background_rects(
        RangeMonthSelectionInfo {
            start_column: 5,
            start_row: 1,
            end_column: 2,
            end_row: 3,
            first_is_selection_start: true,
            last_is_selection_end: true,
        },
        grid_width,
    );

    assert!(rects.len() > 1);
    for rect in rects {
        assert_eq!(
            range_background_corner_radius(rect),
            tokens::component::date_picker::DATE_CONTAINER_HEIGHT / 2.0
        );
    }
}

#[test]
fn date_range_endpoint_connectors_fill_selected_cell_half() {
    let start = range_endpoint_connector_rect(DateRangePosition::Start, 5);
    assert_eq!(
        start,
        Some(RangeBackgroundRect {
            x: 24.0,
            y: 4.0,
            width: 24.0,
            height: 40.0,
        })
    );
    assert_eq!(
        range_background_corner_radius(start.unwrap()),
        tokens::component::date_picker::DATE_CONTAINER_HEIGHT / 2.0
    );
    assert_eq!(
        range_endpoint_connector_rect(DateRangePosition::Start, 6),
        None
    );

    let end = range_endpoint_connector_rect(DateRangePosition::End, 1);
    assert_eq!(
        end,
        Some(RangeBackgroundRect {
            x: 0.0,
            y: 4.0,
            width: 24.0,
            height: 40.0,
        })
    );
    assert_eq!(
        range_background_corner_radius(end.unwrap()),
        tokens::component::date_picker::DATE_CONTAINER_HEIGHT / 2.0
    );
    assert_eq!(
        range_endpoint_connector_rect(DateRangePosition::End, 0),
        None
    );
    assert_eq!(
        range_endpoint_connector_rect(DateRangePosition::Single, 3),
        None
    );
    assert_eq!(
        range_endpoint_connector_rect(DateRangePosition::Middle, 3),
        None
    );
    assert_eq!(range_endpoint_connector_progress(1.0, 0.0), 0.0);
    assert_eq!(range_endpoint_connector_progress(1.0, 0.5), 0.0);
    assert_eq!(range_endpoint_connector_progress(1.0, 1.0), 1.0);
    assert_eq!(range_endpoint_connector_progress(0.4, 1.0), 0.4);
}

#[test]
fn date_range_background_rects_reveal_along_range_path() {
    let grid_width = tokens::component::date_picker::CALENDAR_CELL_SIZE * 7.0;
    let single_row = RangeMonthSelectionInfo {
        start_column: 5,
        start_row: 1,
        end_column: 6,
        end_row: 1,
        first_is_selection_start: true,
        last_is_selection_end: true,
    };
    let cross_row = RangeMonthSelectionInfo {
        start_column: 5,
        start_row: 1,
        end_column: 2,
        end_row: 3,
        first_is_selection_start: true,
        last_is_selection_end: true,
    };

    assert!(range_background_rects_with_progress(single_row, grid_width, 0.0).is_empty());
    assert_eq!(
        range_background_rects_with_progress(single_row, grid_width, 0.5),
        vec![RangeBackgroundRect {
            x: 264.0,
            y: 52.0,
            width: 24.0,
            height: 40.0,
        }]
    );
    assert_eq!(
        range_background_rects_with_progress(cross_row, grid_width, 0.25),
        vec![
            RangeBackgroundRect {
                x: 264.0,
                y: 52.0,
                width: 72.0,
                height: 40.0,
            },
            RangeBackgroundRect {
                x: 0.0,
                y: 100.0,
                width: 60.0,
                height: 40.0,
            },
        ]
    );
}

#[test]
fn date_picker_dialog_surface_uses_material_modal_tokens() {
    let theme = Theme::Light;
    let style = date_picker_container_style(&theme);

    assert_eq!(
        style.border.radius.top_left,
        tokens::component::date_picker::CONTAINER_SHAPE
    );
    assert_eq!(
        style.shadow.blur_radius,
        shadow_from_level(
            tokens::component::date_picker::CONTAINER_ELEVATION_LEVEL,
            theme.colors().shadow,
        )
        .blur_radius
    );
    assert_eq!(
        date_picker_dialog_content_height(),
        tokens::component::date_picker::CONTAINER_HEIGHT
            - tokens::component::button::CONTAINER_HEIGHT
            - tokens::component::date_picker::DIALOG_ACTIONS_BOTTOM_SPACE
    );
}

#[test]
fn date_range_input_mode_updates_start_and_end_fields() {
    let start = Date::new(2026, 8, 1).unwrap();
    let end = Date::new(2026, 8, 5).unwrap();
    let mut state = DateRangePickerState::new(Some(start), Some(end));

    state.update(DateRangePickerAction::SetDisplayMode(
        DateDisplayMode::Input,
    ));
    assert_eq!(state.display_mode(), DateDisplayMode::Input);
    assert_eq!(state.start_input_value(), "08/01/2026");
    assert_eq!(state.end_input_value(), "08/05/2026");

    state.update(DateRangePickerAction::StartInputChanged(
        "08/02/2026".into(),
    ));
    assert_eq!(state.selected_start_date(), Date::new(2026, 8, 2));
    assert!(state.is_start_input_valid());

    state.update(DateRangePickerAction::EndInputChanged("08/01/2026".into()));
    assert_eq!(state.selected_end_date(), None);
    assert!(!state.is_end_input_valid());

    state.update(DateRangePickerAction::EndInputChanged("08062026".into()));
    assert_eq!(state.end_input_value(), "08/06/2026");
    assert_eq!(state.selected_end_date(), Date::new(2026, 8, 6));
    assert!(state.is_end_input_valid());
}

#[test]
fn picker_update_helpers_return_year_scroll_tasks() {
    let mut date = DatePickerState::new(Date::new(2026, 7, 4));
    let _: iced::Task<()> =
        date.update_and_scroll_to_displayed_year(DatePickerAction::ToggleYearPicker);
    assert!(date.year_picker_visible());

    let mut range = DateRangePickerState::new(Date::new(2026, 7, 4), Date::new(2026, 7, 10));
    let _: iced::Task<()> =
        range.update_and_scroll_to_displayed_year(DateRangePickerAction::ToggleYearPicker);
    assert!(range.year_picker_visible());
}

#[test]
fn year_picker_renders_before_opening_animation_advances() {
    assert!(year_picker_should_render(0.0, true));
    assert!(year_picker_should_render(0.5, true));
    assert!(year_picker_should_render(0.5, false));
    assert!(!year_picker_should_render(0.0, false));
}

#[test]
fn date_picker_state_animates_mode_year_picker_and_selection() {
    let start = Instant::now();
    let selected = Date::new(2026, 7, 4).unwrap();
    let next = Date::new(2026, 7, 5).unwrap();
    let mut state = DatePickerState::new(Some(selected));

    state.update_at(
        DatePickerAction::SetDisplayMode(DateDisplayMode::Input),
        start,
    );
    assert!(state.is_animating());
    assert_eq!(
        state
            .animation
            .display
            .mode_offset(DateDisplayMode::Picker, date_display_input_height()),
        0.0
    );
    assert_eq!(
        state
            .animation
            .display
            .mode_offset(DateDisplayMode::Input, date_display_input_height()),
        date_display_input_height()
    );
    assert_eq!(
        state.animation.display.mode_alpha(DateDisplayMode::Picker),
        1.0
    );
    assert_eq!(
        state.animation.display.mode_alpha(DateDisplayMode::Input),
        0.0
    );

    let _ = state.advance(start + duration_ms(tokens::motion::DURATION_MEDIUM2_MS / 2));
    assert!(state.animation.display.progress() > 0.0);
    assert!(state.animation.display.progress() < 1.0);
    assert!(
        state
            .animation
            .display
            .mode_offset(DateDisplayMode::Picker, date_display_input_height())
            < 0.0
    );
    assert!(
        state.animation.display.mode_alpha(DateDisplayMode::Picker) > 0.0
            && state.animation.display.mode_alpha(DateDisplayMode::Picker) < 1.0
    );
    assert!(
        state.animation.display.mode_alpha(DateDisplayMode::Input) > 0.0
            && state.animation.display.mode_alpha(DateDisplayMode::Input) < 1.0
    );
    let animated_height = state.animation.display.content_height(
        date_display_picker_height(),
        date_display_input_height(),
        state.display_mode(),
    );
    assert!(animated_height > date_display_input_height());
    assert!(animated_height < date_display_picker_height());
    assert_eq!(
        state
            .animation
            .display
            .content_layout_height(date_display_picker_height(), date_display_input_height(),),
        date_display_picker_height()
    );

    let _ = state.advance(start + duration_ms(tokens::motion::DURATION_MEDIUM2_MS));
    assert!(!state.animation.display.is_animating());
    assert_eq!(
        state.animation.display.offset(date_display_input_height()),
        0.0
    );
    assert_eq!(
        state.animation.display.content_height(
            date_display_picker_height(),
            date_display_input_height(),
            state.display_mode(),
        ),
        date_display_input_height()
    );

    state.update_at(
        DatePickerAction::ToggleYearPicker,
        start + duration_ms(tokens::motion::DURATION_MEDIUM2_MS + 1),
    );
    assert!(state.is_animating());
    assert_eq!(state.animation.year_picker_progress(), 0.0);

    let _ = state.advance(start + duration_ms(tokens::motion::DURATION_MEDIUM2_MS + 151));
    assert!(state.animation.year_picker_progress() > 0.0);
    assert!(state.animation.year_picker_progress() < 1.0);

    state.update_at(
        DatePickerAction::SelectDate(next),
        start + duration_ms(tokens::motion::DURATION_MEDIUM2_MS + 200),
    );
    assert_eq!(state.selected_date(), Some(next));
    assert_eq!(state.animation.selected_date_progress(next, true), 0.0);
    assert!(state.is_animating());
}

#[test]
fn date_display_alpha_scales_calendar_control_styles() {
    let theme = Theme::Light;
    let colors = theme.colors();
    let year = year_button_style(&theme, Status::Active, true, false, true, 0.5);
    let day = day_button_style(
        &theme,
        Status::Hovered,
        false,
        0.0,
        true,
        DateRangePosition::None,
        0.25,
    );

    assert_eq!(
        year.background,
        Some(Background::Color(alpha_color(colors.primary.color, 0.5)))
    );
    assert_eq!(year.text_color, alpha_color(colors.primary.text, 0.5));
    assert_eq!(
        day.background,
        Some(Background::Color(alpha_color(
            mix(Color::TRANSPARENT, colors.surface.text, 0.08),
            0.25
        )))
    );

    let hidden_input = date_input_panel_style(&theme, 0.0);
    let entering_input = date_input_panel_style(&theme, 0.01);
    assert_eq!(hidden_input.background, None);
    assert_eq!(
        entering_input.background,
        Some(Background::Color(colors.surface.container.high))
    );

    assert_eq!(year_picker_content_alpha(0.0, true), 0.6);
    assert_eq!(year_picker_content_alpha(1.0, true), 1.0);
    assert_eq!(year_picker_content_alpha(0.5, false), 0.5);
    assert_eq!(
        year_picker_panel_style(&theme, 0.5).background,
        Some(Background::Color(alpha_color(
            colors.surface.container.high,
            0.5
        )))
    );
}

#[test]
fn date_picker_month_navigation_slides_between_adjacent_months() {
    let start = Instant::now();
    let selected = Date::new(2026, 7, 4).unwrap();
    let mut state = DatePickerState::new(Some(selected));
    let july = YearMonth::new(2026, 7).unwrap();
    let august = YearMonth::new(2026, 8).unwrap();

    state.update_at(DatePickerAction::NextMonth, start);

    assert_eq!(state.displayed_month(), august);
    assert!(state.is_animating());
    assert_eq!(state.animation.month.visible_months(august), (july, august));
    assert_eq!(state.animation.month.month_offset(july), -0.0);
    assert_eq!(
        state.animation.month.month_offset(august),
        month_grid_slide_width()
    );

    let _ = state.advance(start + duration_ms(tokens::motion::DURATION_MEDIUM2_MS / 2));
    assert!(state.animation.month.month_offset(july) < 0.0);
    assert!(state.animation.month.month_offset(august) > 0.0);
    assert!(state.animation.month.month_offset(august) < month_grid_slide_width());

    let _ = state.advance(start + duration_ms(tokens::motion::DURATION_MEDIUM2_MS));
    assert!(!state.animation.month.is_animating());
    assert_eq!(state.animation.month.month_offset(august), 0.0);

    state.update_at(
        DatePickerAction::PreviousMonth,
        start + duration_ms(tokens::motion::DURATION_MEDIUM2_MS + 1),
    );

    assert_eq!(state.displayed_month(), july);
    assert_eq!(state.animation.month.visible_months(july), (august, july));
    assert_eq!(
        state.animation.month.month_offset(july),
        -month_grid_slide_width()
    );
}

#[test]
fn date_range_picker_state_animates_display_and_selected_endpoint() {
    let start = Instant::now();
    let range_start = Date::new(2026, 7, 4).unwrap();
    let range_end = Date::new(2026, 7, 10).unwrap();
    let mut state = DateRangePickerState::new(Some(range_start), None);

    state.update_at(
        DateRangePickerAction::SetDisplayMode(DateDisplayMode::Input),
        start,
    );
    assert!(state.is_animating());
    assert_eq!(
        state
            .animation
            .display
            .mode_offset(DateDisplayMode::Picker, date_display_input_height()),
        0.0
    );
    assert_eq!(
        state
            .animation
            .display
            .mode_offset(DateDisplayMode::Input, date_display_input_height()),
        date_display_input_height()
    );

    state.update_at(
        DateRangePickerAction::SelectDate(range_end),
        start + duration_ms(16),
    );
    assert_eq!(state.selected_end_date(), Some(range_end));
    assert_eq!(state.animation.selected_date_progress(range_end, true), 0.0);
    assert_eq!(
        state
            .animation
            .range_background_progress(DateRangePosition::Middle),
        0.0
    );
}

#[test]
fn time_picker_state_animates_selection_and_clock_angle() {
    let start = Instant::now();
    let mut state = TimePickerState::new(23, 30, false);

    state.update_at(
        TimePickerAction::SetSelection(TimePickerSelectionMode::Minute),
        start,
    );
    assert!(state.is_animating());
    assert_eq!(
        state.animation.previous_selection,
        TimePickerSelectionMode::Hour
    );
    assert_eq!(
        state.animation.selected_selection,
        TimePickerSelectionMode::Minute
    );
    assert_eq!(
        state
            .animation
            .selection_progress(TimePickerSelectionMode::Minute),
        0.0
    );

    let _ = state.advance(start + duration_ms(tokens::motion::DURATION_SHORT4_MS / 2));
    assert!(state.animation.clock_angle() != hour_angle(23));
    assert!(
        state
            .animation
            .selection_progress(TimePickerSelectionMode::Minute)
            > 0.0
    );
}

#[test]
fn time_picker_state_animates_period_toggle() {
    let start = Instant::now();
    let mut state = TimePickerState::new(23, 5, false);

    assert_eq!(state.animation.period_progress(Period::Pm), 1.0);
    assert_eq!(state.animation.period_progress(Period::Am), 0.0);

    state.update_at(TimePickerAction::SetPeriod(Period::Am), start);

    assert_eq!(state.period(), Period::Am);
    assert!(state.is_animating());
    assert_eq!(state.animation.previous_period, Period::Pm);
    assert_eq!(state.animation.selected_period, Period::Am);
    assert_eq!(state.animation.period_progress(Period::Am), 0.0);
    assert_eq!(state.animation.period_progress(Period::Pm), 1.0);

    let _ = state.advance(start + duration_ms(tokens::motion::DURATION_SHORT4_MS / 2));
    let am_progress = state.animation.period_progress(Period::Am);
    let pm_progress = state.animation.period_progress(Period::Pm);

    assert!(am_progress > 0.0 && am_progress < 1.0);
    assert!(pm_progress > 0.0 && pm_progress < 1.0);
}

#[test]
fn time_picker_drag_angle_uses_minute_precision_and_release_keeps_target() {
    let start = Instant::now();
    let mut state = TimePickerState::new(12, 0, false);

    state.update_at(
        TimePickerAction::SetSelection(TimePickerSelectionMode::Minute),
        start,
    );
    let _ = state.advance(start + duration_ms(tokens::motion::DURATION_MEDIUM1_MS));

    let drag_start = start + duration_ms(tokens::motion::DURATION_MEDIUM1_MS + 16);
    let from = state.animation.clock_angle();
    let raw_angle = minute_angle(17) + TAU / 60.0 * 0.35;
    let drag_angle = nearest_angle(from, unpack_angle(pack_angle(raw_angle)));
    let target = nearest_angle(drag_angle, minute_angle(17));

    state.update_at(
        TimePickerAction::DragMinuteAngle(17, pack_angle(raw_angle)),
        drag_start,
    );

    assert_eq!(state.minute(), 17);
    assert!((state.animation.clock_angle() - drag_angle).abs() < 0.0001);
    assert!(!state.animation.clock_angle.is_animating());

    let settle_start = drag_start + duration_ms(16);
    state.update_at(TimePickerAction::FinishDrag, settle_start);
    assert!((state.animation.clock_angle() - drag_angle).abs() < 0.0001);
    assert!(state.animation.clock_angle.is_animating());

    let _ = state.advance(settle_start + duration_ms(tokens::motion::DURATION_SHORT4_MS / 2));
    let mid = state.animation.clock_angle();
    assert!((mid - drag_angle).abs() > 0.001);
    assert!((target - mid).abs() < (target - drag_angle).abs());
    assert!((target - mid).abs() > 0.001);

    let _ = state.advance(settle_start + duration_ms(tokens::motion::DURATION_SHORT4_MS));
    assert!((state.animation.clock_angle() - target).abs() < 0.0001);
    assert!(!state.animation.clock_angle.is_animating());
}

#[test]
fn time_picker_clock_angle_uses_shortest_wraparound_path() {
    let from = minute_angle(59);
    let to = nearest_angle(from, minute_angle(0));

    assert!((to - from).abs() < TAU / 12.0);
}

#[test]
fn time_state_keeps_12_hour_period() {
    let mut state = TimePickerState::new(23, 5, false);

    assert_eq!(state.hour_for_display(), 11);
    assert_eq!(state.period(), Period::Pm);

    state.update(TimePickerAction::SetPeriod(Period::Am));

    assert_eq!(state.hour(), 11);
    assert_eq!(state.formatted_time(), "11:05 AM");
}

#[test]
fn time_picker_display_mode_matches_material_dialog_labels_and_toggle() {
    assert_eq!(TimePickerDisplayMode::Picker.title(), "Select Time");
    assert_eq!(TimePickerDisplayMode::Input.title(), "Enter Time");
    assert_eq!(TimePickerDisplayMode::Scroll.title(), "Select Time");
    assert_eq!(TimePickerDisplayMode::Picker.toggle_icon(), "keyboard");
    assert_eq!(TimePickerDisplayMode::Input.toggle_icon(), "schedule");
    assert_eq!(TimePickerDisplayMode::Scroll.toggle_icon(), "keyboard");
    assert_eq!(
        TimePickerDisplayMode::Input.scroll_toggled(),
        TimePickerDisplayMode::Scroll
    );
    assert_eq!(
        TimePickerDisplayMode::Scroll.scroll_toggled(),
        TimePickerDisplayMode::Input
    );
    assert_eq!(
        TimePickerDisplayMode::Scroll.scroll_toggle_icon(),
        "keyboard"
    );
    assert_eq!(
        TimePickerDisplayMode::Input.scroll_toggle_icon(),
        "swipe_vertical"
    );
    assert_eq!(
        TimePickerDisplayMode::Picker.toggled(),
        TimePickerDisplayMode::Input
    );
    assert_eq!(
        TimePickerDisplayMode::Input.toggled(),
        TimePickerDisplayMode::Picker
    );
}

#[test]
fn time_scroll_values_wrap_and_preserve_12_hour_period() {
    let mut state = TimePickerState::new(23, 0, false);
    let values = time_scroll_values(&state, TimePickerSelectionMode::Hour);

    assert_eq!(values.len(), 14);
    assert_eq!(values[0], 10);
    assert_eq!(values[1], 11);
    assert_eq!(values[2], 12);
    assert_eq!(
        values
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        12
    );
    assert_eq!(
        time_scroll_value(&state, TimePickerSelectionMode::Hour, -1),
        10
    );
    assert_eq!(
        time_scroll_value(&state, TimePickerSelectionMode::Hour, 0),
        11
    );
    assert_eq!(
        time_scroll_value(&state, TimePickerSelectionMode::Hour, 1),
        12
    );
    assert_eq!(
        time_scroll_value(&state, TimePickerSelectionMode::Minute, -1),
        59
    );

    state.update(TimePickerAction::SelectHour(12));

    assert_eq!(state.hour(), 12);
    assert_eq!(state.period(), Period::Pm);
    assert_eq!(
        time_scroll_value(&state, TimePickerSelectionMode::Hour, 0),
        12
    );

    state.update(TimePickerAction::ScrollHour(11));

    assert_eq!(state.hour(), 23);
    assert_eq!(state.period(), Period::Pm);
    assert_eq!(
        time_scroll_value(&state, TimePickerSelectionMode::Hour, 0),
        12
    );

    let state = TimePickerState::new(0, 59, true);
    let hours = time_scroll_values(&state, TimePickerSelectionMode::Hour);
    let minutes = time_scroll_values(&state, TimePickerSelectionMode::Minute);

    assert_eq!(hours.len(), 26);
    assert_eq!(hours[0], 23);
    assert_eq!(hours[1], 0);
    assert_eq!(
        hours
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        24
    );
    assert_eq!(minutes.len(), 62);
    assert_eq!(minutes[0], 58);
    assert_eq!(minutes[1], 59);
    assert_eq!(minutes[2], 0);
    assert_eq!(
        minutes
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        60
    );
    assert_eq!(
        time_scroll_value(&state, TimePickerSelectionMode::Hour, -1),
        23
    );
    assert_eq!(
        time_scroll_value(&state, TimePickerSelectionMode::Minute, 1),
        0
    );
    assert_eq!(time_scroll_label(5, TimePickerSelectionMode::Minute), "05");
}

#[test]
fn time_scroll_offset_maps_center_row_to_scroll_actions() {
    let state = TimePickerState::new(23, 58, false);

    assert_eq!(
        time_scroll_action_for_offset(&state, TimePickerSelectionMode::Hour, 0.0),
        TimePickerAction::ScrollHour(11)
    );
    assert_eq!(
        time_scroll_action_for_offset(
            &state,
            TimePickerSelectionMode::Hour,
            tokens::component::time_picker::TIME_SCROLL_ITEM_HEIGHT,
        ),
        TimePickerAction::ScrollHour(12)
    );
    assert_eq!(
        time_scroll_action_for_offset(
            &state,
            TimePickerSelectionMode::Minute,
            tokens::component::time_picker::TIME_SCROLL_ITEM_HEIGHT * 2.0,
        ),
        TimePickerAction::ScrollMinute(0)
    );
}

#[test]
fn time_scroll_touch_drag_publishes_center_row_without_cursor() {
    let field = TimeScrollField {
        selection: TimePickerSelectionMode::Minute,
        is_24_hour: true,
        selected_value: 58,
        anchor_value: 58,
        option_count: 60,
        on_action: Arc::new(|action: TimePickerAction| action),
    };
    let mut state = TimeScrollFieldState::default();
    let bounds = Rectangle::new(
        Point::ORIGIN,
        Size::new(
            tokens::component::time_picker::TIME_SCROLL_FIELD_WIDTH,
            tokens::component::time_picker::TIME_SCROLL_FIELD_HEIGHT,
        ),
    );
    let center = Point::new(bounds.width / 2.0, bounds.height / 2.0);

    let _ = <TimeScrollField<_> as canvas::Program<
        TimePickerAction,
        Theme,
        iced_widget::Renderer,
    >>::update(
        &field,
        &mut state,
        &event::Event::Touch(touch::Event::FingerPressed {
            id: touch::Finger(1),
            position: center,
        }),
        bounds,
        mouse::Cursor::Unavailable,
    )
    .expect("touch press should be captured");

    let drag = <TimeScrollField<_> as canvas::Program<
        TimePickerAction,
        Theme,
        iced_widget::Renderer,
    >>::update(
        &field,
        &mut state,
        &event::Event::Touch(touch::Event::FingerMoved {
            id: touch::Finger(1),
            position: Point::new(
                center.x,
                center.y - tokens::component::time_picker::TIME_SCROLL_ITEM_HEIGHT * 2.0,
            ),
        }),
        bounds,
        mouse::Cursor::Unavailable,
    )
    .expect("touch drag should publish when the centered row changes");
    let (message, _, _) = drag.into_inner();

    assert_eq!(message, Some(TimePickerAction::ScrollMinute(0)));
    assert!(state.drag.is_some_and(|drag| drag.has_scrolled));
}

#[test]
fn time_scroll_inertia_is_touch_only() {
    let field = TimeScrollField {
        selection: TimePickerSelectionMode::Minute,
        is_24_hour: true,
        selected_value: 10,
        anchor_value: 10,
        option_count: 60,
        on_action: Arc::new(|action: TimePickerAction| action),
    };
    let mut state = TimeScrollFieldState::default();
    field.sync_state(&mut state);
    state.velocity_y = TIME_SCROLL_FLING_MIN_VELOCITY * 2.0;
    state.drag = Some(TimeScrollDrag {
        pointer: TimeScrollPointer::Mouse,
        start: Point::new(50.0, 60.0),
        last: Point::new(50.0, 0.0),
        last_at: Instant::now(),
        has_scrolled: true,
    });
    let bounds = Rectangle::new(
        Point::ORIGIN,
        Size::new(
            tokens::component::time_picker::TIME_SCROLL_FIELD_WIDTH,
            tokens::component::time_picker::TIME_SCROLL_FIELD_HEIGHT,
        ),
    );

    let _ = field
        .release::<TimePickerAction>(
            &mut state,
            &event::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
            bounds,
            mouse::Cursor::Available(Point::new(50.0, 0.0)),
            TimeScrollPointer::Mouse,
        )
        .expect("mouse release should settle");

    assert_eq!(state.velocity_y, 0.0);

    state.velocity_y = TIME_SCROLL_FLING_MIN_VELOCITY * 2.0;
    let _ = field
        .advance_inertia::<TimePickerAction>(&mut state, Instant::now() + duration_ms(16))
        .expect("touch inertia frame should advance while velocity is active");

    assert!(state.offset_y > 0.0);
    assert!(state.velocity_y > 0.0);
    assert!(state.velocity_y < TIME_SCROLL_FLING_MIN_VELOCITY * 2.0);
}

#[test]
fn time_scroll_uses_fixed_selection_layer_not_item_background() {
    let theme = Theme::Light;
    let item = time_scroll_item_style(&theme, Status::Active, true);
    let field = time_scroll_field_container_style(&theme);
    let layer = time_scroll_selection_layer_style(&theme);

    assert_eq!(item.background, None);
    assert_eq!(
        field.background,
        Some(Background::Color(theme.colors().surface.container.highest))
    );
    assert_eq!(
        layer.background,
        Some(Background::Color(theme.colors().primary.container))
    );
}

#[test]
fn time_period_toggle_uses_compose_container_separator_and_partial_shapes() {
    let theme = Theme::Light;
    let colors = theme.colors();

    assert_eq!(
        period_toggle_item_size(
            true,
            tokens::component::time_picker::PERIOD_SELECTOR_VERTICAL_WIDTH,
            tokens::component::time_picker::PERIOD_SELECTOR_VERTICAL_HEIGHT,
        ),
        (
            tokens::component::time_picker::PERIOD_SELECTOR_VERTICAL_WIDTH,
            tokens::component::time_picker::PERIOD_SELECTOR_VERTICAL_HEIGHT / 2.0,
        )
    );
    assert_eq!(
        period_toggle_item_size(
            false,
            tokens::component::time_picker::PERIOD_SELECTOR_HORIZONTAL_WIDTH,
            tokens::component::time_picker::PERIOD_SELECTOR_HORIZONTAL_HEIGHT,
        ),
        (
            tokens::component::time_picker::PERIOD_SELECTOR_HORIZONTAL_WIDTH / 2.0,
            tokens::component::time_picker::PERIOD_SELECTOR_HORIZONTAL_HEIGHT,
        )
    );

    let start = period_toggle_item_radius(true, true);
    let end = period_toggle_item_radius(true, false);
    assert_eq!(
        start.top_left,
        tokens::component::time_picker::PERIOD_SELECTOR_SHAPE
    );
    assert_eq!(start.bottom_left, 0.0);
    assert_eq!(end.top_left, 0.0);
    assert_eq!(
        end.bottom_left,
        tokens::component::time_picker::PERIOD_SELECTOR_SHAPE
    );

    let container = period_toggle_container_style(&theme);
    let separator = period_toggle_separator_style(&theme);
    assert_eq!(
        container.border.width,
        tokens::component::time_picker::PERIOD_SELECTOR_OUTLINE_WIDTH
    );
    assert_eq!(
        container.border.radius.top_left,
        tokens::component::time_picker::PERIOD_SELECTOR_SHAPE
    );
    assert_eq!(container.border.color, colors.outline.color);
    assert_eq!(
        separator.background,
        Some(Background::Color(colors.outline.color))
    );

    let selected = period_button_style(&theme, Status::Active, 1.0, start);
    let unselected = period_button_style(&theme, Status::Active, 0.0, end);
    let halfway = period_button_style(&theme, Status::Active, 0.5, start);
    assert_eq!(
        selected.background,
        Some(Background::Color(colors.tertiary.container))
    );
    assert_eq!(selected.text_color, colors.tertiary.container_text);
    assert_eq!(selected.border.width, 0.0);
    assert_eq!(selected.border.radius, start);
    assert_eq!(unselected.background, None);
    assert_eq!(unselected.text_color, colors.surface.text_variant);
    assert_eq!(unselected.border.width, 0.0);
    assert_eq!(unselected.border.radius, end);
    assert_eq!(
        halfway.background,
        Some(Background::Color(Color {
            a: colors.tertiary.container.a * 0.5,
            ..colors.tertiary.container
        }))
    );
    assert_eq!(
        halfway.text_color,
        mix(
            colors.surface.text_variant,
            colors.tertiary.container_text,
            0.5
        )
    );
}

#[test]
fn time_picker_dialog_surface_uses_material_dialog_tokens() {
    let theme = Theme::Light;
    let style = time_picker_dialog_container_style(&theme);

    assert_eq!(
        style.border.radius.top_left,
        tokens::component::time_picker_dialog::CONTAINER_SHAPE
    );
    assert_eq!(
        style.shadow.blur_radius,
        shadow_from_level(
            tokens::component::time_picker_dialog::CONTAINER_ELEVATION_LEVEL,
            theme.colors().shadow,
        )
        .blur_radius
    );
    assert_eq!(
        time_picker_dialog_content_height(TimePickerDisplayMode::Picker),
        Length::Shrink
    );
    assert_eq!(
        time_picker_dialog_content_height(TimePickerDisplayMode::Input),
        Length::Fixed(tokens::component::time_picker_dialog::MIN_HEIGHT_FOR_TIME_PICKER)
    );
    assert_eq!(
        time_picker_dialog_content_height(TimePickerDisplayMode::Scroll),
        Length::Fixed(tokens::component::time_picker_dialog::MIN_HEIGHT_FOR_TIME_PICKER)
    );
}

#[test]
fn rich_time_picker_dialog_uses_rich_container_color() {
    let theme = Theme::Light;
    let style = rich_time_picker_dialog_container_style(&theme);

    assert_eq!(
        style.background,
        Some(Background::Color(theme.colors().surface.container.base))
    );
    assert_eq!(
        style.border.radius.top_left,
        tokens::component::time_picker_dialog::CONTAINER_SHAPE
    );
}

#[test]
fn time_input_validity_tracks_user_text_and_auto_advances_12_hour_input() {
    let mut state = TimePickerState::new(14, 5, false);

    assert!(state.is_input_valid());
    assert_eq!(
        time_input_value(&state, TimePickerSelectionMode::Hour),
        "02"
    );
    assert_eq!(
        time_input_value(&state, TimePickerSelectionMode::Minute),
        "05"
    );

    state.update(TimePickerAction::HourInputChanged("99".into()));
    assert!(!state.is_hour_input_valid());
    assert_eq!(state.hour(), 14);
    assert_eq!(
        time_input_value(&state, TimePickerSelectionMode::Hour),
        "99"
    );

    state.update(TimePickerAction::HourInputChanged("2".into()));
    assert!(state.is_hour_input_valid());
    assert_eq!(state.selection(), TimePickerSelectionMode::Minute);

    state.update(TimePickerAction::MinuteInputChanged("75".into()));
    assert!(!state.is_minute_input_valid());
    assert_eq!(state.minute(), 5);
}

#[test]
fn time_input_selector_style_uses_error_color_for_invalid_value() {
    let theme = Theme::Light;
    let colors = theme.colors();

    let invalid = time_input_selector_style(&theme, Status::Active, false);
    let valid = time_input_selector_style(&theme, Status::Active, true);

    assert_eq!(invalid.text_color, colors.error.color);
    assert_eq!(valid.text_color, colors.surface.text);
    assert_eq!(
        invalid.border.radius.top_left,
        tokens::component::time_input::TIME_FIELD_CONTAINER_SHAPE
    );
}

#[test]
fn time_input_supporting_label_matches_selection_and_error_state() {
    let mut state = TimePickerState::new(14, 5, false);

    assert_eq!(
        time_input_supporting_label(&state, TimePickerSelectionMode::Hour, true),
        "Hour"
    );
    assert_eq!(
        time_input_supporting_label(&state, TimePickerSelectionMode::Minute, true),
        "Minute"
    );

    state.update(TimePickerAction::HourInputChanged("99".into()));
    assert_eq!(
        time_input_supporting_label(
            &state,
            TimePickerSelectionMode::Hour,
            state.is_hour_input_valid()
        ),
        "Hour must be 1–12"
    );

    let mut state = TimePickerState::new(14, 5, true);
    state.update(TimePickerAction::HourInputChanged("99".into()));
    assert_eq!(
        time_input_supporting_label(
            &state,
            TimePickerSelectionMode::Hour,
            state.is_hour_input_valid()
        ),
        "Hour must be 0–23"
    );

    state.update(TimePickerAction::MinuteInputChanged("75".into()));
    assert_eq!(
        time_input_supporting_label(
            &state,
            TimePickerSelectionMode::Minute,
            state.is_minute_input_valid()
        ),
        "Minute must be 0–59"
    );
}

#[test]
fn clock_face_24_hour_inner_top_selects_midnight() {
    let face = ClockFace {
        hour: 0,
        minute: 0,
        is_24_hour: true,
        selection: TimePickerSelectionMode::Hour,
        previous_selection: TimePickerSelectionMode::Hour,
        selected_selection: TimePickerSelectionMode::Hour,
        selection_progress: 1.0,
        auto_switch_to_minute: true,
        selector_angle: time_selector_angle(0, 0, TimePickerSelectionMode::Hour),
        on_action: Arc::new(|action: TimePickerAction| action),
    };
    let size = Size::new(
        tokens::component::time_picker::CLOCK_DIAL_SIZE,
        tokens::component::time_picker::CLOCK_DIAL_SIZE,
    );
    let center = Point::new(size.width / 2.0, size.height / 2.0);
    let inner_top = Point::new(
        center.x,
        center.y - size.height * tokens::component::time_picker::INNER_CIRCLE_RADIUS_RATIO,
    );
    let outer_top = Point::new(
        center.x,
        center.y - size.height * tokens::component::time_picker::OUTER_CIRCLE_RADIUS_RATIO,
    );

    assert_eq!(
        face.action_at(inner_top, size),
        TimePickerAction::SelectHour(0)
    );
    assert_eq!(
        face.action_at(outer_top, size),
        TimePickerAction::SelectHour(12)
    );
}

#[test]
fn clock_face_drag_updates_after_press() {
    let face = ClockFace {
        hour: 12,
        minute: 0,
        is_24_hour: false,
        selection: TimePickerSelectionMode::Hour,
        previous_selection: TimePickerSelectionMode::Hour,
        selected_selection: TimePickerSelectionMode::Hour,
        selection_progress: 1.0,
        auto_switch_to_minute: true,
        selector_angle: time_selector_angle(12, 0, TimePickerSelectionMode::Hour),
        on_action: Arc::new(|action: TimePickerAction| action),
    };
    let mut state = ClockFaceState::default();
    let size = Size::new(
        tokens::component::time_picker::CLOCK_DIAL_SIZE,
        tokens::component::time_picker::CLOCK_DIAL_SIZE,
    );
    let bounds = Rectangle::new(Point::new(0.0, 0.0), size);
    let center = Point::new(size.width / 2.0, size.height / 2.0);
    let top = Point::new(
        center.x,
        center.y - size.height * tokens::component::time_picker::OUTER_CIRCLE_RADIUS_RATIO,
    );
    let right = Point::new(
        center.x + size.height * tokens::component::time_picker::OUTER_CIRCLE_RADIUS_RATIO,
        center.y,
    );

    let press =
        <ClockFace<_> as canvas::Program<TimePickerAction, Theme, iced_widget::Renderer>>::update(
            &face,
            &mut state,
            &event::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
            bounds,
            mouse::Cursor::Available(top),
        )
        .expect("press should publish selection");
    let (message, _, _) = press.into_inner();
    assert_eq!(message, Some(TimePickerAction::SelectHour(12)));

    let drag =
        <ClockFace<_> as canvas::Program<TimePickerAction, Theme, iced_widget::Renderer>>::update(
            &face,
            &mut state,
            &event::Event::Mouse(mouse::Event::CursorMoved { position: right }),
            bounds,
            mouse::Cursor::Available(right),
        )
        .expect("drag should publish selection while pressed");
    let (message, _, _) = drag.into_inner();
    assert_eq!(
        message,
        Some(TimePickerAction::DragHourAngle(3, pack_angle(0.0)))
    );

    let release =
        <ClockFace<_> as canvas::Program<TimePickerAction, Theme, iced_widget::Renderer>>::update(
            &face,
            &mut state,
            &event::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
            bounds,
            mouse::Cursor::Available(right),
        )
        .expect("release should auto-switch after hour drag");
    let (message, _, _) = release.into_inner();
    assert_eq!(
        message,
        Some(TimePickerAction::SetSelection(
            TimePickerSelectionMode::Minute
        ))
    );
}

#[test]
fn clock_face_touch_drag_works_without_cursor() {
    let face = ClockFace {
        hour: 12,
        minute: 0,
        is_24_hour: false,
        selection: TimePickerSelectionMode::Hour,
        previous_selection: TimePickerSelectionMode::Hour,
        selected_selection: TimePickerSelectionMode::Hour,
        selection_progress: 1.0,
        auto_switch_to_minute: true,
        selector_angle: time_selector_angle(12, 0, TimePickerSelectionMode::Hour),
        on_action: Arc::new(|action: TimePickerAction| action),
    };
    let mut state = ClockFaceState::default();
    let size = Size::new(
        tokens::component::time_picker::CLOCK_DIAL_SIZE,
        tokens::component::time_picker::CLOCK_DIAL_SIZE,
    );
    let bounds = Rectangle::new(Point::new(0.0, 0.0), size);
    let center = Point::new(size.width / 2.0, size.height / 2.0);
    let top = Point::new(
        center.x,
        center.y - size.height * tokens::component::time_picker::OUTER_CIRCLE_RADIUS_RATIO,
    );
    let right = Point::new(
        center.x + size.height * tokens::component::time_picker::OUTER_CIRCLE_RADIUS_RATIO,
        center.y,
    );

    let press =
        <ClockFace<_> as canvas::Program<TimePickerAction, Theme, iced_widget::Renderer>>::update(
            &face,
            &mut state,
            &event::Event::Touch(touch::Event::FingerPressed {
                id: touch::Finger(7),
                position: top,
            }),
            bounds,
            mouse::Cursor::Unavailable,
        )
        .expect("touch press should publish selection");
    let (message, _, _) = press.into_inner();
    assert_eq!(message, Some(TimePickerAction::SelectHour(12)));

    let drag =
        <ClockFace<_> as canvas::Program<TimePickerAction, Theme, iced_widget::Renderer>>::update(
            &face,
            &mut state,
            &event::Event::Touch(touch::Event::FingerMoved {
                id: touch::Finger(7),
                position: right,
            }),
            bounds,
            mouse::Cursor::Unavailable,
        )
        .expect("touch drag should publish selection while pressed");
    let (message, _, _) = drag.into_inner();
    assert_eq!(
        message,
        Some(TimePickerAction::DragHourAngle(3, pack_angle(0.0)))
    );

    let release =
        <ClockFace<_> as canvas::Program<TimePickerAction, Theme, iced_widget::Renderer>>::update(
            &face,
            &mut state,
            &event::Event::Touch(touch::Event::FingerLifted {
                id: touch::Finger(7),
                position: right,
            }),
            bounds,
            mouse::Cursor::Unavailable,
        )
        .expect("touch release should auto-switch after hour drag");
    let (message, _, _) = release.into_inner();
    assert_eq!(
        message,
        Some(TimePickerAction::SetSelection(
            TimePickerSelectionMode::Minute
        ))
    );
}

#[test]
fn clock_face_minute_tap_and_drag_use_minute_precision() {
    let face = ClockFace {
        hour: 12,
        minute: 0,
        is_24_hour: false,
        selection: TimePickerSelectionMode::Minute,
        previous_selection: TimePickerSelectionMode::Minute,
        selected_selection: TimePickerSelectionMode::Minute,
        selection_progress: 1.0,
        auto_switch_to_minute: true,
        selector_angle: time_selector_angle(12, 0, TimePickerSelectionMode::Minute),
        on_action: Arc::new(|action: TimePickerAction| action),
    };
    let size = Size::new(
        tokens::component::time_picker::CLOCK_DIAL_SIZE,
        tokens::component::time_picker::CLOCK_DIAL_SIZE,
    );
    let center = Point::new(size.width / 2.0, size.height / 2.0);
    let radius = size.height * tokens::component::time_picker::OUTER_CIRCLE_RADIUS_RATIO;
    let minute_17 = Point::new(
        center.x + radius * minute_angle(17).cos(),
        center.y + radius * minute_angle(17).sin(),
    );
    let minute_8 = Point::new(
        center.x + radius * minute_angle(8).cos(),
        center.y + radius * minute_angle(8).sin(),
    );

    assert_eq!(
        face.action_at(minute_17, size),
        TimePickerAction::SelectMinute(17)
    );
    assert_eq!(
        face.drag_action_at(minute_17, size),
        TimePickerAction::DragMinuteAngle(17, pack_angle(minute_angle(17)))
    );
    assert_eq!(
        face.drag_action_at(minute_8, size),
        TimePickerAction::DragMinuteAngle(8, pack_angle(minute_angle(8)))
    );
}

#[test]
fn clock_face_overlay_label_follows_selector_geometry() {
    let face = ClockFace {
        hour: 5,
        minute: 0,
        is_24_hour: false,
        selection: TimePickerSelectionMode::Hour,
        previous_selection: TimePickerSelectionMode::Hour,
        selected_selection: TimePickerSelectionMode::Hour,
        selection_progress: 1.0,
        auto_switch_to_minute: true,
        selector_angle: hour_angle(4),
        on_action: Arc::new(|action: TimePickerAction| action),
    };
    let size = Size::new(
        tokens::component::time_picker::CLOCK_DIAL_SIZE,
        tokens::component::time_picker::CLOCK_DIAL_SIZE,
    );
    let center = Point::new(size.width / 2.0, size.height / 2.0);
    let radius = size.width / 2.0;
    let label_radius = radius * 2.0 * tokens::component::time_picker::OUTER_CIRCLE_RADIUS_RATIO;
    let scale = tokens::component::time_picker::CLOCK_DIAL_LABEL_TEXT;

    assert!(face.label_intersects_selector(center, radius, label_radius, hour_angle(4), scale));
    assert!(!face.label_intersects_selector(center, radius, label_radius, hour_angle(5), scale));
    assert!(face.label_uses_selector_foreground(
        center,
        radius,
        label_radius,
        hour_angle(4),
        scale
    ));
    assert!(!face.label_uses_selector_foreground(
        center,
        radius,
        label_radius,
        hour_angle(5),
        scale
    ));
}

#[test]
fn clock_face_overlay_label_masks_visible_minutes_under_selector() {
    let face = ClockFace {
        hour: 12,
        minute: 17,
        is_24_hour: false,
        selection: TimePickerSelectionMode::Minute,
        previous_selection: TimePickerSelectionMode::Minute,
        selected_selection: TimePickerSelectionMode::Minute,
        selection_progress: 1.0,
        auto_switch_to_minute: true,
        selector_angle: minute_angle(17),
        on_action: Arc::new(|action: TimePickerAction| action),
    };
    let size = Size::new(
        tokens::component::time_picker::CLOCK_DIAL_SIZE,
        tokens::component::time_picker::CLOCK_DIAL_SIZE,
    );
    let center = Point::new(size.width / 2.0, size.height / 2.0);
    let radius = size.width / 2.0;
    let label_radius = radius * 2.0 * tokens::component::time_picker::OUTER_CIRCLE_RADIUS_RATIO;
    let scale = tokens::component::time_picker::CLOCK_DIAL_LABEL_TEXT;

    assert!(face.label_intersects_selector(center, radius, label_radius, minute_angle(15), scale));
    assert!(face.label_intersects_selector(center, radius, label_radius, minute_angle(20), scale));
    assert!(!face.label_intersects_selector(center, radius, label_radius, minute_angle(10), scale));
    assert!(face.label_uses_selector_foreground(
        center,
        radius,
        label_radius,
        minute_angle(15),
        scale
    ));
    assert!(face.label_uses_selector_foreground(
        center,
        radius,
        label_radius,
        minute_angle(20),
        scale
    ));
    assert!(!face.label_uses_selector_foreground(
        center,
        radius,
        label_radius,
        minute_angle(10),
        scale
    ));
}

#[test]
fn clock_face_tap_auto_switches_from_hour_to_minute() {
    let face = ClockFace {
        hour: 12,
        minute: 0,
        is_24_hour: false,
        selection: TimePickerSelectionMode::Hour,
        previous_selection: TimePickerSelectionMode::Hour,
        selected_selection: TimePickerSelectionMode::Hour,
        selection_progress: 1.0,
        auto_switch_to_minute: true,
        selector_angle: time_selector_angle(12, 0, TimePickerSelectionMode::Hour),
        on_action: Arc::new(|action: TimePickerAction| action),
    };
    let mut state = ClockFaceState::default();
    let size = Size::new(
        tokens::component::time_picker::CLOCK_DIAL_SIZE,
        tokens::component::time_picker::CLOCK_DIAL_SIZE,
    );
    let bounds = Rectangle::new(Point::new(0.0, 0.0), size);
    let center = Point::new(size.width / 2.0, size.height / 2.0);
    let top = Point::new(
        center.x,
        center.y - size.height * tokens::component::time_picker::OUTER_CIRCLE_RADIUS_RATIO,
    );

    let _ =
        <ClockFace<_> as canvas::Program<TimePickerAction, Theme, iced_widget::Renderer>>::update(
            &face,
            &mut state,
            &event::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
            bounds,
            mouse::Cursor::Available(top),
        )
        .expect("press should publish selection");

    let release =
        <ClockFace<_> as canvas::Program<TimePickerAction, Theme, iced_widget::Renderer>>::update(
            &face,
            &mut state,
            &event::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
            bounds,
            mouse::Cursor::Available(top),
        )
        .expect("release should publish minute selection");
    let (message, _, _) = release.into_inner();

    assert_eq!(
        message,
        Some(TimePickerAction::SetSelection(
            TimePickerSelectionMode::Minute
        ))
    );
}
