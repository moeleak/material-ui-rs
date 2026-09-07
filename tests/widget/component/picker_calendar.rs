fn calendar_rows(layout: Layout<'_>) -> Vec<Vec<Rectangle>> {
    if layout.children().count() == 7 {
        vec![layout.children().map(|cell| cell.bounds()).collect()]
    } else {
        layout.children().flat_map(calendar_rows).collect()
    }
}

#[test]
fn narrow_calendars_keep_all_seven_columns_aligned_and_clickable() {
    let month = YearMonth::new(2026, 7).unwrap();
    let date_state = DatePickerState::new(Some(Date::new(2026, 7, 3).unwrap()));
    let range_state = DateRangePickerState::new(
        Some(Date::new(2026, 7, 3).unwrap()),
        Some(Date::new(2026, 7, 10).unwrap()),
    );
    let renderer = picker_test_renderer();

    for width in [264.0, 304.0, 360.0] {
        for first_day in [Weekday::Sunday, Weekday::Monday] {
            for range in [false, true] {
                let date_state = date_state.clone().first_day_of_week(first_day);
                let range_state = range_state.clone().first_day_of_week(first_day);
                let mut grid: Element<'_, Date, Theme, iced_widget::Renderer> = if range {
                    range_month_grid_for_month(
                        &range_state,
                        |action| match action {
                            DateRangePickerAction::SelectDate(date) => date,
                            _ => panic!("unexpected date range action"),
                        },
                        month,
                        1.0,
                    )
                } else {
                    month_grid_for_month(
                        &date_state,
                        |action| match action {
                            DatePickerAction::SelectDate(date) => date,
                            _ => panic!("unexpected date action"),
                        },
                        month,
                        1.0,
                    )
                };
                let mut tree = Tree::new(grid.as_widget());
                let viewport = Rectangle::with_size(Size::new(width, 288.0));
                let node = grid.as_widget_mut().layout(
                    &mut tree,
                    &renderer,
                    &layout::Limits::new(Size::ZERO, viewport.size()),
                );
                let rows = calendar_rows(Layout::new(&node));
                assert_eq!(rows.len(), 6);
                let cell_width = (width - 24.0) / 7.0;
                for row in &rows {
                    for (index, cell) in row.iter().enumerate() {
                        assert!(
                            (cell.width - cell_width).abs() < 0.001,
                            "width={width}, range={range}: {row:?}"
                        );
                        assert!((cell.x - (12.0 + index as f32 * cell_width)).abs() < 0.001);
                        assert_eq!(cell.height, 48.0);
                        assert!(cell.x >= 0.0 && cell.x + cell.width <= width);
                    }
                }

                let mut weekdays = weekdays_row::<Date, iced_widget::Renderer>(first_day, 1.0);
                let mut weekdays_tree = Tree::new(weekdays.as_widget());
                let weekdays_node = weekdays.as_widget_mut().layout(
                    &mut weekdays_tree,
                    &renderer,
                    &layout::Limits::new(Size::ZERO, viewport.size()),
                );
                let weekday_cells = calendar_rows(Layout::new(&weekdays_node));
                assert_eq!(weekday_cells[0], rows[0]);

                // Check every actual day, especially Saturday: the old fixed
                // cells compressed the final column and shifted hit targets.
                let first = month.start_date().weekday_index_from(first_day);
                let mut messages = Vec::new();
                for day in 1..=31 {
                    let index = first + usize::from(day - 1);
                    let position = rows[index / 7][index % 7].center();
                    let id = touch::Finger(1);
                    for event in [
                        touch::Event::FingerPressed { id, position },
                        touch::Event::FingerLifted { id, position },
                    ] {
                        grid.as_widget_mut().update(
                            &mut tree,
                            &Event::Touch(event),
                            Layout::new(&node),
                            mouse::Cursor::Available(position),
                            &renderer,
                            &mut iced_widget::core::clipboard::Null,
                            &mut Shell::new(&mut messages),
                            &viewport,
                        );
                    }
                    assert_eq!(messages.last(), Some(&Date::new(2026, 7, day).unwrap()));
                    assert_eq!(messages.len(), usize::from(day));
                }
            }
        }
    }
}

#[test]
fn narrow_date_ranges_connect_to_the_actual_endpoint_centers() {
    for width in [240.0, 280.0, 336.0] {
        let state = DateRangePickerState::new(
            Some(Date::new(2026, 7, 3).unwrap()),
            Some(Date::new(2026, 7, 10).unwrap()),
        );
        let info = range_month_selection_info(&state, YearMonth::new(2026, 7).unwrap()).unwrap();
        let rects = range_bg(info, width).rects();
        let friday_center = 5.5 * width / 7.0;
        assert!((rects[0].x - friday_center).abs() < 0.001);
        assert!((rects[0].x + rects[0].width - width).abs() < 0.001);
        assert_eq!(rects[1].x, 0.0);
        assert!((rects[1].width - friday_center).abs() < 0.001);

        for position in [DateRangePosition::Start, DateRangePosition::End] {
            let connector = RangeConnector {
                position,
                weekday: 5,
                width: width / 7.0,
            }
            .rect()
            .unwrap();
            assert!((connector.width - width / 14.0).abs() < 0.001);
            assert_eq!(
                connector.x,
                if position == DateRangePosition::Start {
                    width / 14.0
                } else {
                    0.0
                }
            );
        }

        // A range continuing from/to another month reaches the cell edges,
        // with no negative padding when the grid is narrower than 336 dp.
        let continuation = range_bg(
            RangeMonthSelectionInfo {
                start_column: 0,
                start_row: 0,
                end_column: 6,
                end_row: 0,
                first_is_selection_start: false,
                last_is_selection_end: false,
            },
            width,
        )
        .rects();
        assert_eq!(continuation[0].x, 0.0);
        assert!((continuation[0].width - width).abs() < 0.001);
    }
}

#[test]
fn date_picker_surfaces_align_scrollable_months_with_weekday_headers() {
    let selected = Date::new(2026, 7, 3).unwrap();
    let date_state = DatePickerState::new(Some(selected));
    let range_state =
        DateRangePickerState::new(Some(selected), Some(Date::new(2026, 7, 10).unwrap()));
    let renderer = picker_test_renderer();
    for mut picker in [
        date_picker(&date_state, |_| ()),
        date_range_picker(&range_state, |_| ()),
    ] {
        let mut tree = Tree::new(picker.as_widget());
        let node = picker.as_widget_mut().layout(
            &mut tree,
            &renderer,
            &layout::Limits::new(Size::ZERO, Size::new(304.0, 568.0)),
        );
        let rows = calendar_rows(Layout::new(&node));
        assert!(rows.len() >= 7);
        for row in rows {
            for (index, cell) in row.iter().enumerate() {
                assert!((cell.width - 40.0).abs() < 0.001);
                assert!((cell.center_x() - (32.0 + index as f32 * 40.0)).abs() < 0.001);
            }
        }
    }
}
