fn days_in_month(year: i32, month: u8) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => 0,
    }
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn days_from_civil(year: i32, month: u8, day: u8) -> i64 {
    let year = i64::from(year) - i64::from(month <= 2);
    let era = year.div_euclid(400);
    let yoe = year - era * 400;
    let month = i64::from(month);
    let day = i64::from(day);
    let doy = (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;

    era * 146_097 + doe - 719_468
}

fn civil_from_days(days: i64) -> (i32, u8, u8) {
    let days = days + 719_468;
    let era = days.div_euclid(146_097);
    let doe = days - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096).div_euclid(365);
    let year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2).div_euclid(153);
    let day = doy - (153 * mp + 2).div_euclid(5) + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    let year = year + i64::from(month <= 2);

    (year as i32, month as u8, day as u8)
}

fn normalize_date_input(input: String) -> String {
    if input.contains('-')
        && let Some(date) = parse_date_input(&input)
    {
        return date.format_input();
    }

    let digits: String = input
        .chars()
        .filter(|digit| digit.is_ascii_digit())
        .take(8)
        .collect();
    let mut formatted = String::with_capacity(10);

    for (index, digit) in digits.chars().enumerate() {
        if index == 2 || index == 4 {
            formatted.push('/');
        }
        formatted.push(digit);
    }

    formatted
}

fn date_input_digit_count(input: &str) -> usize {
    input.chars().filter(|digit| digit.is_ascii_digit()).count()
}

fn date_input_error(
    input: &str,
    year_range: RangeInclusive<i32>,
    selectable_dates: &SelectableDates,
    formatter: &DatePickerFormatter,
    range_bound: Option<DateInputRangeBound>,
) -> Option<String> {
    let input = input.trim();

    if input.is_empty() || date_input_digit_count(input) < 8 {
        return None;
    }

    let Some(date) = parse_date_input(input) else {
        return Some("Date does not match expected pattern: MM/DD/YYYY".to_owned());
    };

    if !year_range.contains(&date.year) {
        return Some(format!(
            "Date out of expected year range {} - {}",
            year_range.start(),
            year_range.end()
        ));
    }

    if !selectable_dates.is_selectable_year(date.year) || !selectable_dates.is_selectable_date(date)
    {
        return Some(format!(
            "Date not allowed: {}",
            formatter.format_date(date, false)
        ));
    }

    match range_bound {
        Some(DateInputRangeBound::StartBeforeOrEqual(end)) if date > end => {
            Some("Invalid date range input".to_owned())
        }
        Some(DateInputRangeBound::EndAfterOrEqual(start)) if date < start => {
            Some("Invalid date range input".to_owned())
        }
        _ => None,
    }
}

fn parse_date_input(input: &str) -> Option<Date> {
    let input = input.trim();

    if input.is_empty() {
        return None;
    }

    if date_input_digit_count(input) == 8 && input.chars().all(|digit| digit.is_ascii_digit()) {
        let month = input[0..2].parse().ok()?;
        let day = input[2..4].parse().ok()?;
        let year = input[4..8].parse().ok()?;

        return Date::new(year, month, day);
    }

    if input.contains('-') {
        let parts: Vec<_> = input.split('-').collect();
        if parts.len() == 3 {
            let year = parts[0].parse().ok()?;
            let month = parts[1].parse().ok()?;
            let day = parts[2].parse().ok()?;

            return Date::new(year, month, day);
        }
    }

    let parts: Vec<_> = input.split('/').collect();
    if parts.len() == 3 {
        let month = parts[0].parse().ok()?;
        let day = parts[1].parse().ok()?;
        let year = parts[2].parse().ok()?;

        return Date::new(year, month, day);
    }

    None
}

fn hour_for_display(hour: u8, is_24_hour: bool) -> u8 {
    if is_24_hour {
        hour % 24
    } else {
        match hour % 12 {
            0 => 12,
            hour => hour,
        }
    }
}

fn period_for_hour(hour: u8) -> Period {
    if hour >= 12 { Period::Pm } else { Period::Am }
}

fn hour_to_24(hour: u8, period: Period) -> u8 {
    let hour = hour.clamp(1, 12);
    match (hour, period) {
        (12, Period::Am) => 0,
        (12, Period::Pm) => 12,
        (_, Period::Am) => hour,
        (_, Period::Pm) => hour + 12,
    }
}

fn time_input_label(selection: TimePickerSelectionMode) -> &'static str {
    match selection {
        TimePickerSelectionMode::Hour => "Hour",
        TimePickerSelectionMode::Minute => "Minute",
    }
}

fn time_input_value(state: &TimePickerState, selection: TimePickerSelectionMode) -> String {
    match selection {
        TimePickerSelectionMode::Hour if state.is_hour_input_valid() => {
            two_digit(state.hour_for_display())
        }
        TimePickerSelectionMode::Hour => state.hour_input.clone(),
        TimePickerSelectionMode::Minute => state.minute_input.clone(),
    }
}

#[cfg(test)]
fn time_scroll_value(
    state: &TimePickerState,
    selection: TimePickerSelectionMode,
    offset: i16,
) -> u8 {
    let anchor = time_scroll_anchor_value(state, selection);

    time_scroll_value_for_anchor(anchor, selection, state.is_24_hour, offset)
}

fn time_scroll_value_for_anchor(
    anchor: u8,
    selection: TimePickerSelectionMode,
    is_24_hour: bool,
    offset: i16,
) -> u8 {
    let anchor = i16::from(anchor);

    match selection {
        TimePickerSelectionMode::Hour => {
            if is_24_hour {
                wrap_u8(anchor + offset, 24)
            } else {
                wrap_u8(anchor - 1 + offset, 12) + 1
            }
        }
        TimePickerSelectionMode::Minute => wrap_u8(anchor + offset, 60),
    }
}

fn time_scroll_option_count(state: &TimePickerState, selection: TimePickerSelectionMode) -> u8 {
    match selection {
        TimePickerSelectionMode::Hour if state.is_24_hour => 24,
        TimePickerSelectionMode::Hour => 12,
        TimePickerSelectionMode::Minute => 60,
    }
}

fn time_scroll_selected_value(state: &TimePickerState, selection: TimePickerSelectionMode) -> u8 {
    match selection {
        TimePickerSelectionMode::Hour if state.is_24_hour => state.hour,
        TimePickerSelectionMode::Hour => state.hour_for_display(),
        TimePickerSelectionMode::Minute => state.minute,
    }
}

fn time_scroll_anchor_value(state: &TimePickerState, selection: TimePickerSelectionMode) -> u8 {
    match selection {
        TimePickerSelectionMode::Hour => state.scroll_hour_anchor,
        TimePickerSelectionMode::Minute => state.scroll_minute_anchor,
    }
}

#[cfg(test)]
fn time_scroll_values(state: &TimePickerState, selection: TimePickerSelectionMode) -> Vec<u8> {
    let count = time_scroll_option_count(state, selection);

    (-1..=i16::from(count))
        .map(|offset| time_scroll_value(state, selection, offset))
        .collect()
}

#[cfg(test)]
fn time_scroll_action_for_offset(
    state: &TimePickerState,
    selection: TimePickerSelectionMode,
    absolute_offset_y: f32,
) -> TimePickerAction {
    let row_offset = time_scroll_row_offset_for_offset(
        time_scroll_option_count(state, selection),
        absolute_offset_y,
    );
    let value = time_scroll_value(state, selection, row_offset);

    match selection {
        TimePickerSelectionMode::Hour => TimePickerAction::ScrollHour(value),
        TimePickerSelectionMode::Minute => TimePickerAction::ScrollMinute(value),
    }
}

fn time_scroll_row_offset_for_offset(option_count: u8, absolute_offset_y: f32) -> i16 {
    let item_height = tokens::component::time_picker::TIME_SCROLL_ITEM_HEIGHT;
    let max_row_offset = option_count.saturating_sub(1);

    (absolute_offset_y / item_height)
        .round()
        .clamp(0.0, f32::from(max_row_offset)) as i16
}

fn time_scroll_max_offset(option_count: u8) -> f32 {
    f32::from(option_count.saturating_sub(1))
        * tokens::component::time_picker::TIME_SCROLL_ITEM_HEIGHT
}

fn time_scroll_label(value: u8, selection: TimePickerSelectionMode) -> String {
    match selection {
        TimePickerSelectionMode::Hour => value.to_string(),
        TimePickerSelectionMode::Minute => two_digit(value),
    }
}

fn month_delta(from: YearMonth, to: YearMonth) -> i32 {
    (to.year - from.year) * 12 + i32::from(to.month) - i32::from(from.month)
}

fn month_grid_slide_width() -> f32 {
    tokens::component::date_picker::CONTAINER_WIDTH
}

fn year_picker_initial_item_index(year_range: &RangeInclusive<i32>, displayed_year: i32) -> usize {
    let start = *year_range.start();
    let end = *year_range.end();

    if end < start {
        return 0;
    }

    let displayed_index = (displayed_year - start).clamp(0, end - start) as usize;

    displayed_index.saturating_sub(tokens::component::date_picker::YEARS_IN_ROW)
}

fn year_picker_displayed_year_scroll_offset(
    year_range: &RangeInclusive<i32>,
    displayed_year: i32,
) -> f32 {
    let first_visible_item = year_picker_initial_item_index(year_range, displayed_year);
    let first_visible_row = first_visible_item / tokens::component::date_picker::YEARS_IN_ROW;

    first_visible_row as f32 * year_picker_row_height()
}

fn year_picker_row_height() -> f32 {
    tokens::component::date_picker::YEAR_CONTAINER_HEIGHT
        + tokens::component::date_picker::YEAR_VERTICAL_SPACE
}

fn time_input_supporting_label(
    state: &TimePickerState,
    selection: TimePickerSelectionMode,
    valid: bool,
) -> &'static str {
    match (valid, selection, state.is_24_hour) {
        (true, TimePickerSelectionMode::Hour, _) => "Hour",
        (true, TimePickerSelectionMode::Minute, _) => "Minute",
        (false, TimePickerSelectionMode::Hour, true) => "Hour must be 0–23",
        (false, TimePickerSelectionMode::Hour, false) => "Hour must be 1–12",
        (false, TimePickerSelectionMode::Minute, _) => "Minute must be 0–59",
    }
}

#[cfg(test)]
fn range_visible_months(state: &DateRangePickerState) -> Vec<YearMonth> {
    (0..range_month_count(state))
        .filter_map(|index| range_month_at_index(state, index))
        .collect()
}

fn range_rendered_months(state: &DateRangePickerState) -> Vec<YearMonth> {
    let window = range_render_window(state);

    (window.start..window.end)
        .filter_map(|index| range_month_at_index(state, index))
        .collect()
}

fn range_render_window(state: &DateRangePickerState) -> Range<usize> {
    let month_count = range_month_count(state);
    if month_count == 0 {
        return 0..0;
    }

    let max_window =
        (RANGE_PICKER_RENDER_MONTHS_BEFORE + RANGE_PICKER_RENDER_MONTHS_AFTER + 1).max(1) as usize;

    if month_count <= max_window {
        return 0..month_count;
    }

    let displayed_index = range_month_index(state, state.displayed_month);
    let start = displayed_index.saturating_sub(RANGE_PICKER_RENDER_MONTHS_BEFORE as usize);
    let end = (displayed_index + RANGE_PICKER_RENDER_MONTHS_AFTER as usize + 1).min(month_count);

    start..end
}

fn range_month_count(state: &DateRangePickerState) -> usize {
    let start = *state.year_range.start();
    let end = *state.year_range.end();

    if end < start {
        0
    } else {
        ((end - start + 1) as usize) * 12
    }
}

fn range_month_index(state: &DateRangePickerState, month: YearMonth) -> usize {
    let month_count = range_month_count(state);
    if month_count == 0 {
        return 0;
    }

    let start_year = *state.year_range.start();
    let index = (month.year - start_year) * 12 + i32::from(month.month.saturating_sub(1));

    index.clamp(0, month_count as i32 - 1) as usize
}

fn range_month_at_index(state: &DateRangePickerState, index: usize) -> Option<YearMonth> {
    if index >= range_month_count(state) {
        return None;
    }

    let start_year = *state.year_range.start();
    let year = start_year + (index / 12) as i32;
    let month = (index % 12) as u8 + 1;

    YearMonth::new(year, month)
}

fn range_displayed_month_scroll_offset(state: &DateRangePickerState) -> f32 {
    let window = range_render_window(state);
    let displayed_index = range_month_index(state, state.displayed_month);
    displayed_index.saturating_sub(window.start) as f32 * range_month_item_height()
}

fn range_month_item_height() -> f32 {
    tokens::component::date_picker::RANGE_MONTH_SUBHEAD_TOP_SPACE
        + tokens::component::date_picker::RANGE_MONTH_SUBHEAD_TEXT.line_height
        + tokens::component::date_picker::RANGE_MONTH_SUBHEAD_BOTTOM_SPACE
        + tokens::component::date_picker::CALENDAR_CELL_SIZE
            * tokens::component::date_picker::MAX_CALENDAR_ROWS as f32
}

fn normalize_range_selection(
    start: Option<Date>,
    end: Option<Date>,
) -> (Option<Date>, Option<Date>) {
    match (start, end) {
        (Some(start), Some(end)) if start <= end => (Some(start), Some(end)),
        (Some(start), None) => (Some(start), None),
        _ => (None, None),
    }
}

fn two_digit(value: u8) -> String {
    format!("{value:02}")
}

fn wrap_u8(value: i16, modulo: u8) -> u8 {
    value.rem_euclid(i16::from(modulo)) as u8
}

fn keep_digits(input: String, max_len: usize) -> String {
    input
        .chars()
        .filter(char::is_ascii_digit)
        .take(max_len)
        .collect()
}

fn time_selector_angle(hour: u8, minute: u8, selection: TimePickerSelectionMode) -> f32 {
    match selection {
        TimePickerSelectionMode::Hour => hour_angle(hour),
        TimePickerSelectionMode::Minute => minute_angle(minute),
    }
}

fn selected_24_hour_label_value(hour: u8) -> u8 {
    if hour == 12 { 0 } else { hour.min(23) }
}

fn visible_minute(minute: u8) -> u8 {
    (((u16::from(minute.min(59)) + 2) / 5 * 5) % 60) as u8
}

fn nearest_angle(current: f32, target: f32) -> f32 {
    let delta = (target - current + std::f32::consts::PI).rem_euclid(TAU) - std::f32::consts::PI;

    current + delta
}

fn pack_angle(angle: f32) -> i32 {
    (angle * CLOCK_ANGLE_SCALE).round() as i32
}

fn unpack_angle(angle: i32) -> f32 {
    angle as f32 / CLOCK_ANGLE_SCALE
}

fn hour_angle(hour: u8) -> f32 {
    TAU / 12.0 * f32::from(hour % 12) - FRAC_PI_2
}

fn minute_angle(minute: u8) -> f32 {
    TAU / 60.0 * f32::from(minute % 60) - FRAC_PI_2
}

fn angle_to_hour(angle: f32) -> u8 {
    let hour =
        (((angle + FRAC_PI_2 + TAU / 24.0).rem_euclid(TAU) / (TAU / 12.0)).floor() as u8) % 12;

    if hour == 0 { 12 } else { hour }
}

fn angle_to_minute(angle: f32) -> u8 {
    (((angle + FRAC_PI_2 + TAU / 120.0).rem_euclid(TAU) / (TAU / 60.0)).floor() as u8) % 60
}
