//! Material 3 date and time picker constructors.

use std::cell::Cell;
use std::f32::consts::{FRAC_PI_2, TAU};
use std::fmt;
use std::ops::{Range, RangeInclusive};
use std::sync::Arc;

use iced_widget::button::{Status, Style};
use iced_widget::canvas::{self, Canvas, Frame, Geometry, Path, Stroke, Text as CanvasText};
use iced_widget::core::text as core_text;
use iced_widget::core::time::Instant;
use iced_widget::core::widget::{self, Tree, tree};
use iced_widget::core::{
    Background, Border, Clipboard, Color, Element, Event, Layout, Length, Padding, Point,
    Rectangle, Shell, Size, Vector, Widget, alignment, border, event, layout, mouse, overlay,
    renderer, touch, window,
};
use iced_widget::graphics::geometry;
use iced_widget::text::{self, LineHeight};
use iced_widget::{Column, Container, Row, Scrollable, Space, Stack, Text};

use super::absolute_line_height;
use super::button::Button;
use super::support::{AnimatedScalar, alpha_color, duration_ms, lerp};
use super::text_input;
use super::viewport;
use crate::button as button_style;
use crate::utils::{disabled_container, disabled_text, mix, shadow_from_level};
use crate::{Theme, fonts, tokens};

const DEFAULT_START_YEAR: i32 = 1900;
const DEFAULT_END_YEAR: i32 = 2100;
const MILLIS_PER_DAY: i64 = 86_400_000;
const WEEKDAYS_SHORT: [&str; 7] = ["S", "M", "T", "W", "T", "F", "S"];
const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];
const MONTHS_ABBR: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];
const HOURS: [u8; 12] = [12, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];
const EXTRA_HOURS: [u8; 12] = [0, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23];
const MINUTES: [u8; 12] = [0, 5, 10, 15, 20, 25, 30, 35, 40, 45, 50, 55];
const CLOCK_ANGLE_SCALE: f32 = 10_000.0;
const CLOCK_LABEL_CLIP_STRIPS: usize = 40;
const DATE_DISPLAY_PARALLAX_OFFSET: f32 = -48.0;
const RANGE_PICKER_RENDER_MONTHS_BEFORE: i32 = 2;
const RANGE_PICKER_RENDER_MONTHS_AFTER: i32 = 24;
const TIME_SCROLL_TOUCH_SLOP: f32 = 8.0;
const TIME_SCROLL_FLING_MIN_VELOCITY: f32 = 120.0;
const TIME_SCROLL_FLING_MAX_VELOCITY: f32 = 2800.0;
const TIME_SCROLL_FLING_DECELERATION: f32 = 3200.0;

/// A day of week used for date picker layout and formatting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Weekday {
    Sunday,
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
}

impl Weekday {
    const ALL: [Self; 7] = [
        Self::Sunday,
        Self::Monday,
        Self::Tuesday,
        Self::Wednesday,
        Self::Thursday,
        Self::Friday,
        Self::Saturday,
    ];

    fn sunday_first_index(self) -> usize {
        match self {
            Self::Sunday => 0,
            Self::Monday => 1,
            Self::Tuesday => 2,
            Self::Wednesday => 3,
            Self::Thursday => 4,
            Self::Friday => 5,
            Self::Saturday => 6,
        }
    }

    pub fn short_label(self) -> &'static str {
        WEEKDAYS_SHORT[self.sunday_first_index()]
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Sunday => "Sunday",
            Self::Monday => "Monday",
            Self::Tuesday => "Tuesday",
            Self::Wednesday => "Wednesday",
            Self::Thursday => "Thursday",
            Self::Friday => "Friday",
            Self::Saturday => "Saturday",
        }
    }
}

/// A proleptic Gregorian calendar date.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Date {
    pub year: i32,
    pub month: u8,
    pub day: u8,
}

impl Date {
    /// Creates a date when the year/month/day combination is valid.
    pub fn new(year: i32, month: u8, day: u8) -> Option<Self> {
        if !(1..=12).contains(&month) {
            return None;
        }

        if !(1..=days_in_month(year, month)).contains(&day) {
            return None;
        }

        Some(Self { year, month, day })
    }

    /// Returns today's UTC date.
    pub fn today_utc() -> Self {
        Self::from_utc_millis(current_utc_millis())
    }

    /// Converts a UTC timestamp to the date at the start of that UTC day.
    pub fn from_utc_millis(millis: i64) -> Self {
        Self::from_days_since_epoch(millis.div_euclid(MILLIS_PER_DAY))
    }

    /// Converts this date to UTC milliseconds at the start of the day.
    pub fn to_utc_millis(self) -> i64 {
        self.days_since_epoch() * MILLIS_PER_DAY
    }

    fn from_days_since_epoch(days: i64) -> Self {
        let (year, month, day) = civil_from_days(days);

        Self { year, month, day }
    }

    fn days_since_epoch(self) -> i64 {
        days_from_civil(self.year, self.month, self.day)
    }

    pub fn weekday(self) -> Weekday {
        Weekday::ALL[(self.days_since_epoch() + 4).rem_euclid(7) as usize]
    }

    fn weekday_index_from(self, first_day_of_week: Weekday) -> usize {
        (self.weekday().sunday_first_index() + 7 - first_day_of_week.sunday_first_index()) % 7
    }

    fn month_start(self) -> YearMonth {
        YearMonth {
            year: self.year,
            month: self.month,
        }
    }

    fn format_headline(self) -> String {
        format!(
            "{}, {} {}, {}",
            self.weekday().name(),
            MONTHS_ABBR[usize::from(self.month - 1)],
            self.day,
            self.year
        )
    }

    fn format_input(self) -> String {
        format!("{:02}/{:02}/{:04}", self.month, self.day, self.year)
    }

    fn format_range_label(self) -> String {
        format!(
            "{} {}, {}",
            MONTHS_ABBR[usize::from(self.month - 1)],
            self.day,
            self.year
        )
    }
}

#[cfg(target_arch = "wasm32")]
fn current_utc_millis() -> i64 {
    js_sys::Date::now() as i64
}

#[cfg(not(target_arch = "wasm32"))]
fn current_utc_millis() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or(0)
}

/// A year and month pair used by the date picker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct YearMonth {
    pub year: i32,
    pub month: u8,
}

impl YearMonth {
    pub fn new(year: i32, month: u8) -> Option<Self> {
        (1..=12).contains(&month).then_some(Self { year, month })
    }

    pub fn current_utc() -> Self {
        Date::today_utc().month_start()
    }

    pub fn start_date(self) -> Date {
        Date {
            year: self.year,
            month: self.month,
            day: 1,
        }
    }

    pub fn add_months(self, delta: i32) -> Self {
        let month_index = self.year * 12 + i32::from(self.month - 1) + delta;
        let year = month_index.div_euclid(12);
        let month = month_index.rem_euclid(12) as u8 + 1;

        Self { year, month }
    }

    pub fn format(self) -> String {
        format!("{} {}", MONTHS[usize::from(self.month - 1)], self.year)
    }
}

/// Date picker display mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateDisplayMode {
    Picker,
    Input,
}

/// Date and year enablement used by Material date pickers.
#[derive(Clone)]
pub struct SelectableDates {
    is_selectable_date: Arc<dyn Fn(Date) -> bool + Send + Sync + 'static>,
    is_selectable_year: Arc<dyn Fn(i32) -> bool + Send + Sync + 'static>,
}

impl fmt::Debug for SelectableDates {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SelectableDates").finish_non_exhaustive()
    }
}

impl Default for SelectableDates {
    fn default() -> Self {
        Self::all()
    }
}

impl SelectableDates {
    /// Allows every date and year, matching the official default.
    pub fn all() -> Self {
        Self {
            is_selectable_date: Arc::new(|_| true),
            is_selectable_year: Arc::new(|_| true),
        }
    }

    /// Creates selectable-date predicates.
    pub fn new(
        is_selectable_date: impl Fn(Date) -> bool + Send + Sync + 'static,
        is_selectable_year: impl Fn(i32) -> bool + Send + Sync + 'static,
    ) -> Self {
        Self {
            is_selectable_date: Arc::new(is_selectable_date),
            is_selectable_year: Arc::new(is_selectable_year),
        }
    }

    pub fn is_selectable_date(&self, date: Date) -> bool {
        (self.is_selectable_date)(date)
    }

    pub fn is_selectable_year(&self, year: i32) -> bool {
        (self.is_selectable_year)(year)
    }
}

/// Formatter hooks used by Material date and date range pickers.
#[derive(Clone)]
pub struct DatePickerFormatter {
    format_month_year: Arc<dyn Fn(YearMonth) -> String + Send + Sync + 'static>,
    format_date: Arc<dyn Fn(Date, bool) -> String + Send + Sync + 'static>,
}

impl fmt::Debug for DatePickerFormatter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DatePickerFormatter")
            .finish_non_exhaustive()
    }
}

impl Default for DatePickerFormatter {
    fn default() -> Self {
        Self::english()
    }
}

impl DatePickerFormatter {
    /// Creates an English formatter matching the default Material skeletons.
    pub fn english() -> Self {
        Self::new(YearMonth::format, |date, for_content_description| {
            if for_content_description {
                date.format_headline()
            } else {
                date.format_range_label()
            }
        })
    }

    /// Creates a formatter from month/year and date closures.
    pub fn new(
        format_month_year: impl Fn(YearMonth) -> String + Send + Sync + 'static,
        format_date: impl Fn(Date, bool) -> String + Send + Sync + 'static,
    ) -> Self {
        Self {
            format_month_year: Arc::new(format_month_year),
            format_date: Arc::new(format_date),
        }
    }

    pub fn format_month_year(&self, month: YearMonth) -> String {
        (self.format_month_year)(month)
    }

    pub fn format_date(&self, date: Date, for_content_description: bool) -> String {
        (self.format_date)(date, for_content_description)
    }
}

#[derive(Debug, Clone)]
struct DateDisplayAnimation {
    progress: AnimatedScalar,
    from: DateDisplayMode,
    to: DateDisplayMode,
}

impl DateDisplayAnimation {
    fn new(mode: DateDisplayMode) -> Self {
        Self {
            progress: AnimatedScalar::new(1.0),
            from: mode,
            to: mode,
        }
    }

    fn start(&mut self, from: DateDisplayMode, to: DateDisplayMode, now: Instant) {
        if from == to {
            return;
        }

        self.from = from;
        self.to = to;
        self.progress = AnimatedScalar::new(0.0);
        self.progress.set_target(
            1.0,
            now,
            duration_ms(tokens::motion::DURATION_MEDIUM2_MS),
            tokens::motion::EASING_EMPHASIZED,
        );
    }

    fn advance(&mut self, now: Instant) -> bool {
        self.progress.advance(now)
    }

    fn is_animating(&self) -> bool {
        self.progress.is_animating()
    }

    fn progress(&self) -> f32 {
        self.progress.value.clamp(0.0, 1.0)
    }

    #[cfg(test)]
    fn offset(&self, input_height: f32) -> f32 {
        self.mode_offset(self.to, input_height)
    }

    fn source_mode(&self) -> DateDisplayMode {
        self.from
    }

    fn target_mode(&self) -> DateDisplayMode {
        self.to
    }

    fn mode_offset(&self, mode: DateDisplayMode, input_height: f32) -> f32 {
        if self.from == self.to {
            return 0.0;
        }

        let progress = self.progress();
        let full_slide_offset = input_height;

        match (self.from, self.to, mode) {
            (DateDisplayMode::Picker, DateDisplayMode::Input, DateDisplayMode::Picker) => {
                DATE_DISPLAY_PARALLAX_OFFSET * progress
            }
            (DateDisplayMode::Picker, DateDisplayMode::Input, DateDisplayMode::Input) => {
                full_slide_offset * (1.0 - progress)
            }
            (DateDisplayMode::Input, DateDisplayMode::Picker, DateDisplayMode::Input) => {
                full_slide_offset * progress
            }
            (DateDisplayMode::Input, DateDisplayMode::Picker, DateDisplayMode::Picker) => {
                DATE_DISPLAY_PARALLAX_OFFSET * (1.0 - progress)
            }
            _ => 0.0,
        }
    }

    fn mode_alpha(&self, mode: DateDisplayMode) -> f32 {
        if self.from == self.to {
            return 1.0;
        }

        if mode == self.to {
            self.progress()
        } else if mode == self.from {
            1.0 - self.progress()
        } else {
            0.0
        }
    }

    fn content_height(
        &self,
        picker_height: f32,
        input_height: f32,
        display_mode: DateDisplayMode,
    ) -> f32 {
        let height_for = |mode| match mode {
            DateDisplayMode::Picker => picker_height,
            DateDisplayMode::Input => input_height,
        };

        if self.from == self.to {
            return height_for(display_mode);
        }

        lerp(height_for(self.from), height_for(self.to), self.progress())
    }

    fn content_layout_height(&self, picker_height: f32, input_height: f32) -> f32 {
        if self.from == self.to {
            return picker_height.max(input_height);
        }

        picker_height.max(input_height)
    }
}

#[derive(Debug, Clone)]
struct PickerAnimation {
    display: DateDisplayAnimation,
    month: MonthAnimation,
    year_picker: AnimatedScalar,
    selection: AnimatedScalar,
    selected_date: Option<Date>,
}

#[derive(Debug, Clone)]
struct MonthAnimation {
    progress: AnimatedScalar,
    from: YearMonth,
    to: YearMonth,
}

impl MonthAnimation {
    fn new(month: YearMonth) -> Self {
        Self {
            progress: AnimatedScalar::new(1.0),
            from: month,
            to: month,
        }
    }

    fn start(&mut self, from: YearMonth, to: YearMonth, now: Instant) {
        if from == to {
            return;
        }

        self.from = from;
        self.to = to;
        self.progress = AnimatedScalar::new(0.0);
        self.progress.set_target(
            1.0,
            now,
            duration_ms(tokens::motion::DURATION_MEDIUM2_MS),
            tokens::motion::EASING_EMPHASIZED,
        );
    }

    fn advance(&mut self, now: Instant) -> bool {
        self.progress.advance(now)
    }

    fn is_animating(&self) -> bool {
        self.progress.is_animating()
    }

    fn progress(&self) -> f32 {
        self.progress.value.clamp(0.0, 1.0)
    }

    fn visible_months(&self, current: YearMonth) -> (YearMonth, YearMonth) {
        if self.is_animating() {
            (self.from, self.to)
        } else {
            (current, current)
        }
    }

    fn month_offset(&self, month: YearMonth) -> f32 {
        if self.from == self.to {
            return 0.0;
        }

        let direction = month_delta(self.from, self.to).signum() as f32;
        let width = month_grid_slide_width();
        let progress = self.progress();

        if month == self.from {
            -direction * width * progress
        } else if month == self.to {
            direction * width * (1.0 - progress)
        } else {
            0.0
        }
    }
}

impl PickerAnimation {
    fn new(
        display_mode: DateDisplayMode,
        year_picker_visible: bool,
        selected_date: Option<Date>,
        displayed_month: YearMonth,
    ) -> Self {
        Self {
            display: DateDisplayAnimation::new(display_mode),
            month: MonthAnimation::new(displayed_month),
            year_picker: AnimatedScalar::new(if year_picker_visible { 1.0 } else { 0.0 }),
            selection: AnimatedScalar::new(1.0),
            selected_date,
        }
    }

    fn set_year_picker_visible(&mut self, visible: bool, now: Instant) {
        self.year_picker.set_target(
            if visible { 1.0 } else { 0.0 },
            now,
            duration_ms(tokens::motion::DURATION_MEDIUM2_MS),
            tokens::motion::EASING_EMPHASIZED,
        );
    }

    fn select_date(&mut self, date: Date, now: Instant) {
        self.selected_date = Some(date);
        self.selection = AnimatedScalar::new(0.0);
        self.selection.set_target(
            1.0,
            now,
            duration_ms(tokens::motion::DURATION_SHORT4_MS),
            tokens::motion::EASING_EMPHASIZED,
        );
    }

    fn set_displayed_month(&mut self, from: YearMonth, to: YearMonth, now: Instant) {
        self.month.start(from, to, now);
    }

    fn advance(&mut self, now: Instant) -> bool {
        self.display.advance(now)
            | self.month.advance(now)
            | self.year_picker.advance(now)
            | self.selection.advance(now)
    }

    fn is_animating(&self) -> bool {
        self.display.is_animating()
            || self.month.is_animating()
            || self.year_picker.is_animating()
            || self.selection.is_animating()
    }

    fn year_picker_progress(&self) -> f32 {
        self.year_picker.value.clamp(0.0, 1.0)
    }

    fn selected_date_progress(&self, date: Date, selected: bool) -> f32 {
        if selected && self.selected_date == Some(date) {
            self.selection.value.clamp(0.0, 1.0)
        } else if selected {
            1.0
        } else {
            0.0
        }
    }

    fn range_background_progress(&self, range_position: DateRangePosition) -> f32 {
        if range_position.draws_range_background() && self.selection.is_animating() {
            self.selection.value.clamp(0.0, 1.0)
        } else if range_position.draws_range_background() {
            1.0
        } else {
            0.0
        }
    }
}

/// State held by a Material date picker.
#[derive(Debug, Clone)]
pub struct DatePickerState {
    selected_date: Option<Date>,
    displayed_month: YearMonth,
    display_mode: DateDisplayMode,
    year_range: RangeInclusive<i32>,
    selectable_dates: SelectableDates,
    formatter: DatePickerFormatter,
    first_day_of_week: Weekday,
    year_picker_visible: bool,
    input_value: String,
    year_picker_scroll_id: widget::Id,
    animation: PickerAnimation,
}

impl Default for DatePickerState {
    fn default() -> Self {
        Self::new(None)
    }
}

impl DatePickerState {
    /// Creates date picker state with the default Material year range (1900..=2100).
    pub fn new(selected_date: Option<Date>) -> Self {
        let displayed_month = selected_date
            .map(Date::month_start)
            .unwrap_or_else(YearMonth::current_utc);
        let input_value = selected_date.map(Date::format_input).unwrap_or_default();

        Self {
            selected_date,
            displayed_month,
            display_mode: DateDisplayMode::Picker,
            year_range: DEFAULT_START_YEAR..=DEFAULT_END_YEAR,
            selectable_dates: SelectableDates::all(),
            formatter: DatePickerFormatter::default(),
            first_day_of_week: Weekday::Sunday,
            year_picker_visible: false,
            input_value,
            year_picker_scroll_id: widget::Id::unique(),
            animation: PickerAnimation::new(
                DateDisplayMode::Picker,
                false,
                selected_date,
                displayed_month,
            ),
        }
    }

    /// Sets the supported year range, clamping the displayed month to it.
    pub fn year_range(mut self, year_range: RangeInclusive<i32>) -> Self {
        self.year_range = year_range;
        self.clamp_displayed_month();
        self.animation.month = MonthAnimation::new(self.displayed_month);
        self
    }

    /// Sets the initially displayed month independently from the selected date.
    ///
    /// Months outside the configured year range are ignored, matching Material state semantics.
    pub fn initial_displayed_month(mut self, displayed_month: YearMonth) -> Self {
        if self.year_range.contains(&displayed_month.year) {
            self.displayed_month = displayed_month;
            self.animation.month = MonthAnimation::new(displayed_month);
        }

        self
    }

    /// Sets date/year enablement predicates.
    pub fn selectable_dates(mut self, selectable_dates: SelectableDates) -> Self {
        self.selectable_dates = selectable_dates;
        if self
            .selected_date
            .is_some_and(|date| !self.date_is_selectable(date))
        {
            self.selected_date = None;
        }
        self
    }

    /// Sets the first day of week used by the calendar grid.
    pub fn first_day_of_week(mut self, first_day_of_week: Weekday) -> Self {
        self.first_day_of_week = first_day_of_week;
        self
    }

    /// Sets date display formatting hooks.
    pub fn formatter(mut self, formatter: DatePickerFormatter) -> Self {
        self.formatter = formatter;
        self
    }

    pub fn selected_date(&self) -> Option<Date> {
        self.selected_date
    }

    pub fn displayed_month(&self) -> YearMonth {
        self.displayed_month
    }

    pub fn display_mode(&self) -> DateDisplayMode {
        self.display_mode
    }

    pub fn year_picker_visible(&self) -> bool {
        self.year_picker_visible
    }

    pub fn input_value(&self) -> &str {
        &self.input_value
    }

    pub fn calendar_first_day_of_week(&self) -> Weekday {
        self.first_day_of_week
    }

    pub fn date_formatter(&self) -> &DatePickerFormatter {
        &self.formatter
    }

    /// Returns a task that positions the year picker near the displayed year.
    pub fn scroll_year_picker_to_displayed_year<Message>(&self) -> iced::Task<Message>
    where
        Message: Send + 'static,
    {
        iced::widget::operation::scroll_to(
            self.year_picker_scroll_id.clone(),
            widget::operation::scrollable::AbsoluteOffset {
                x: None,
                y: Some(year_picker_displayed_year_scroll_offset(
                    &self.year_range,
                    self.displayed_month.year,
                )),
            },
        )
    }

    pub fn is_input_valid(&self) -> bool {
        self.input_error().is_none()
    }

    pub fn input_error(&self) -> Option<String> {
        date_input_error(
            self.input_value(),
            self.year_range.clone(),
            &self.selectable_dates,
            &self.formatter,
            None,
        )
    }

    /// Applies a date picker action.
    pub fn update(&mut self, action: DatePickerAction) {
        self.update_at(action, Instant::now());
    }

    /// Applies a date picker action and returns the follow-up scroll task used
    /// when the year picker becomes visible.
    pub fn update_and_scroll_to_displayed_year<Message>(
        &mut self,
        action: DatePickerAction,
    ) -> iced::Task<Message>
    where
        Message: Send + 'static,
    {
        self.update_and_scroll_to_displayed_year_at(action, Instant::now())
    }

    /// Applies a date picker action at `now` and returns the follow-up scroll
    /// task used when the year picker becomes visible.
    pub fn update_and_scroll_to_displayed_year_at<Message>(
        &mut self,
        action: DatePickerAction,
        now: Instant,
    ) -> iced::Task<Message>
    where
        Message: Send + 'static,
    {
        let was_year_picker_visible = self.year_picker_visible();

        self.update_at(action, now);

        if !was_year_picker_visible && self.year_picker_visible() {
            self.scroll_year_picker_to_displayed_year()
        } else {
            iced::Task::none()
        }
    }

    /// Applies a date picker action, using `now` as the transition start time.
    pub fn update_at(&mut self, action: DatePickerAction, now: Instant) {
        match action {
            DatePickerAction::SelectDate(date) => {
                if self.date_is_selectable(date) {
                    self.selected_date = Some(date);
                    self.displayed_month = date.month_start();
                    self.input_value = date.format_input();
                    self.animation.select_date(date, now);
                }
            }
            DatePickerAction::PreviousMonth => {
                let previous = self.displayed_month.add_months(-1);
                if self.year_range.contains(&previous.year) {
                    let from = self.displayed_month;
                    self.displayed_month = previous;
                    self.animation.set_displayed_month(from, previous, now);
                }
            }
            DatePickerAction::NextMonth => {
                let next = self.displayed_month.add_months(1);
                if self.year_range.contains(&next.year) {
                    let from = self.displayed_month;
                    self.displayed_month = next;
                    self.animation.set_displayed_month(from, next, now);
                }
            }
            DatePickerAction::ToggleYearPicker => {
                self.year_picker_visible = !self.year_picker_visible;
                self.animation
                    .set_year_picker_visible(self.year_picker_visible, now);
            }
            DatePickerAction::SelectYear(year) => {
                if self.year_range.contains(&year) {
                    self.displayed_month.year = year;
                    self.year_picker_visible = false;
                    self.animation.set_year_picker_visible(false, now);
                }
            }
            DatePickerAction::SetDisplayMode(display_mode) => {
                if self.display_mode != display_mode {
                    self.animation
                        .display
                        .start(self.display_mode, display_mode, now);

                    if display_mode == DateDisplayMode::Input {
                        self.input_value = self
                            .selected_date
                            .map(Date::format_input)
                            .unwrap_or_default();
                    } else if let Some(date) = self.selected_date {
                        self.displayed_month = date.month_start();
                    }

                    self.display_mode = display_mode;
                    self.year_picker_visible = false;
                    self.animation.set_year_picker_visible(false, now);
                }
            }
            DatePickerAction::InputChanged(value) => {
                self.input_value = normalize_date_input(value);

                if date_input_digit_count(&self.input_value) < 8 {
                    self.selected_date = None;
                } else if self.input_error().is_none()
                    && let Some(date) = parse_date_input(&self.input_value)
                {
                    self.selected_date = Some(date);
                    self.displayed_month = date.month_start();
                    self.animation.select_date(date, now);
                } else {
                    self.selected_date = None;
                }
            }
        }
    }

    /// Advances any running Material picker transitions.
    pub fn advance(&mut self, now: Instant) -> bool {
        self.animation.advance(now)
    }

    /// Returns whether this picker needs frame ticks for animations.
    pub fn is_animating(&self) -> bool {
        self.animation.is_animating()
    }

    /// Returns a frame subscription while picker animations are running.
    pub fn subscription<Message, F>(&self, on_frame: F) -> iced::Subscription<Message>
    where
        Message: 'static,
        F: Fn(Instant) -> Message + Send + Clone + 'static,
    {
        if self.is_animating() {
            iced::window::frames().map(on_frame)
        } else {
            iced::Subscription::none()
        }
    }

    fn date_is_in_range(&self, date: Date) -> bool {
        self.year_range.contains(&date.year)
    }

    fn date_is_selectable(&self, date: Date) -> bool {
        self.date_is_in_range(date)
            && self.selectable_dates.is_selectable_year(date.year)
            && self.selectable_dates.is_selectable_date(date)
    }

    fn clamp_displayed_month(&mut self) {
        if self.displayed_month.year < *self.year_range.start() {
            self.displayed_month.year = *self.year_range.start();
            self.displayed_month.month = 1;
        } else if self.displayed_month.year > *self.year_range.end() {
            self.displayed_month.year = *self.year_range.end();
            self.displayed_month.month = 12;
        }

        if self
            .selected_date
            .is_some_and(|date| !self.date_is_selectable(date))
        {
            self.selected_date = None;
        }
    }
}

/// Date picker state transitions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DatePickerAction {
    SelectDate(Date),
    PreviousMonth,
    NextMonth,
    ToggleYearPicker,
    SelectYear(i32),
    SetDisplayMode(DateDisplayMode),
    InputChanged(String),
}

/// State held by a Material date range picker.
#[derive(Debug, Clone)]
pub struct DateRangePickerState {
    selected_start_date: Option<Date>,
    selected_end_date: Option<Date>,
    displayed_month: YearMonth,
    display_mode: DateDisplayMode,
    year_range: RangeInclusive<i32>,
    selectable_dates: SelectableDates,
    formatter: DatePickerFormatter,
    first_day_of_week: Weekday,
    year_picker_visible: bool,
    start_input_value: String,
    end_input_value: String,
    months_scroll_id: widget::Id,
    year_picker_scroll_id: widget::Id,
    animation: PickerAnimation,
}

impl Default for DateRangePickerState {
    fn default() -> Self {
        Self::new(None, None)
    }
}

impl DateRangePickerState {
    /// Creates date range picker state with the default Material year range (1900..=2100).
    pub fn new(selected_start_date: Option<Date>, selected_end_date: Option<Date>) -> Self {
        let (selected_start_date, selected_end_date) =
            normalize_range_selection(selected_start_date, selected_end_date);
        let displayed_month = selected_start_date
            .or(selected_end_date)
            .map(Date::month_start)
            .unwrap_or_else(YearMonth::current_utc);
        let start_input_value = selected_start_date
            .map(Date::format_input)
            .unwrap_or_default();
        let end_input_value = selected_end_date
            .map(Date::format_input)
            .unwrap_or_default();

        Self {
            selected_start_date,
            selected_end_date,
            displayed_month,
            display_mode: DateDisplayMode::Picker,
            year_range: DEFAULT_START_YEAR..=DEFAULT_END_YEAR,
            selectable_dates: SelectableDates::all(),
            formatter: DatePickerFormatter::default(),
            first_day_of_week: Weekday::Sunday,
            year_picker_visible: false,
            start_input_value,
            end_input_value,
            months_scroll_id: widget::Id::unique(),
            year_picker_scroll_id: widget::Id::unique(),
            animation: PickerAnimation::new(
                DateDisplayMode::Picker,
                false,
                selected_start_date.or(selected_end_date),
                displayed_month,
            ),
        }
    }

    pub fn year_range(mut self, year_range: RangeInclusive<i32>) -> Self {
        self.year_range = year_range;
        self.clamp_displayed_month();
        self.animation.month = MonthAnimation::new(self.displayed_month);
        self.sync_input_values_from_selection();
        self
    }

    /// Sets the initially displayed month independently from the selected range.
    ///
    /// Months outside the configured year range are ignored, matching Material state semantics.
    pub fn initial_displayed_month(mut self, displayed_month: YearMonth) -> Self {
        if self.year_range.contains(&displayed_month.year) {
            self.displayed_month = displayed_month;
            self.animation.month = MonthAnimation::new(displayed_month);
        }

        self
    }

    pub fn selectable_dates(mut self, selectable_dates: SelectableDates) -> Self {
        self.selectable_dates = selectable_dates;
        if self
            .selected_start_date
            .is_some_and(|date| !self.date_is_selectable(date))
        {
            self.selected_start_date = None;
            self.selected_end_date = None;
        } else if self
            .selected_end_date
            .is_some_and(|date| !self.date_is_selectable(date))
        {
            self.selected_end_date = None;
        }
        self.sync_input_values_from_selection();
        self
    }

    /// Sets the first day of week used by the calendar grid.
    pub fn first_day_of_week(mut self, first_day_of_week: Weekday) -> Self {
        self.first_day_of_week = first_day_of_week;
        self
    }

    /// Sets date display formatting hooks.
    pub fn formatter(mut self, formatter: DatePickerFormatter) -> Self {
        self.formatter = formatter;
        self
    }

    pub fn selected_start_date(&self) -> Option<Date> {
        self.selected_start_date
    }

    pub fn selected_end_date(&self) -> Option<Date> {
        self.selected_end_date
    }

    pub fn displayed_month(&self) -> YearMonth {
        self.displayed_month
    }

    pub fn year_picker_visible(&self) -> bool {
        self.year_picker_visible
    }

    pub fn display_mode(&self) -> DateDisplayMode {
        self.display_mode
    }

    pub fn calendar_first_day_of_week(&self) -> Weekday {
        self.first_day_of_week
    }

    pub fn date_formatter(&self) -> &DatePickerFormatter {
        &self.formatter
    }

    pub fn start_input_value(&self) -> &str {
        &self.start_input_value
    }

    pub fn end_input_value(&self) -> &str {
        &self.end_input_value
    }

    /// Returns a task that positions the range month list at the displayed month.
    pub fn scroll_to_displayed_month<Message>(&self) -> iced::Task<Message>
    where
        Message: Send + 'static,
    {
        iced::widget::operation::scroll_to(
            self.months_scroll_id.clone(),
            widget::operation::scrollable::AbsoluteOffset {
                x: None,
                y: Some(range_displayed_month_scroll_offset(self)),
            },
        )
    }

    /// Returns a task that positions the year picker near the displayed year.
    pub fn scroll_year_picker_to_displayed_year<Message>(&self) -> iced::Task<Message>
    where
        Message: Send + 'static,
    {
        iced::widget::operation::scroll_to(
            self.year_picker_scroll_id.clone(),
            widget::operation::scrollable::AbsoluteOffset {
                x: None,
                y: Some(year_picker_displayed_year_scroll_offset(
                    &self.year_range,
                    self.displayed_month.year,
                )),
            },
        )
    }

    pub fn is_start_input_valid(&self) -> bool {
        self.start_input_error().is_none()
    }

    pub fn is_end_input_valid(&self) -> bool {
        self.end_input_error().is_none()
    }

    pub fn start_input_error(&self) -> Option<String> {
        let end = parse_date_input(self.end_input_value());
        date_input_error(
            self.start_input_value(),
            self.year_range.clone(),
            &self.selectable_dates,
            &self.formatter,
            end.map(DateInputRangeBound::StartBeforeOrEqual),
        )
    }

    pub fn end_input_error(&self) -> Option<String> {
        let start = parse_date_input(self.start_input_value());
        date_input_error(
            self.end_input_value(),
            self.year_range.clone(),
            &self.selectable_dates,
            &self.formatter,
            start.map(DateInputRangeBound::EndAfterOrEqual),
        )
    }

    /// Applies a date range picker action.
    pub fn update(&mut self, action: DateRangePickerAction) {
        self.update_at(action, Instant::now());
    }

    /// Applies a date range picker action and returns the follow-up scroll task
    /// used when the year picker becomes visible.
    pub fn update_and_scroll_to_displayed_year<Message>(
        &mut self,
        action: DateRangePickerAction,
    ) -> iced::Task<Message>
    where
        Message: Send + 'static,
    {
        self.update_and_scroll_to_displayed_year_at(action, Instant::now())
    }

    /// Applies a date range picker action at `now` and returns the follow-up
    /// scroll task used when the year picker becomes visible.
    pub fn update_and_scroll_to_displayed_year_at<Message>(
        &mut self,
        action: DateRangePickerAction,
        now: Instant,
    ) -> iced::Task<Message>
    where
        Message: Send + 'static,
    {
        let was_year_picker_visible = self.year_picker_visible();

        self.update_at(action, now);

        if !was_year_picker_visible && self.year_picker_visible() {
            self.scroll_year_picker_to_displayed_year()
        } else {
            iced::Task::none()
        }
    }

    /// Applies a date range picker action, using `now` as the transition start time.
    pub fn update_at(&mut self, action: DateRangePickerAction, now: Instant) {
        match action {
            DateRangePickerAction::SelectDate(date) => {
                if !self.date_is_selectable(date) {
                    return;
                }

                match (self.selected_start_date, self.selected_end_date) {
                    (None, _) | (Some(_), Some(_)) => {
                        self.selected_start_date = Some(date);
                        self.selected_end_date = None;
                    }
                    (Some(start), None) if date < start => {
                        self.selected_start_date = Some(date);
                    }
                    (Some(_), None) => {
                        self.selected_end_date = Some(date);
                    }
                }

                self.animation.select_date(date, now);
                self.sync_input_values_from_selection();
            }
            DateRangePickerAction::PreviousMonth => {
                let previous = self.displayed_month.add_months(-1);
                if self.year_range.contains(&previous.year) {
                    self.displayed_month = previous;
                }
            }
            DateRangePickerAction::NextMonth => {
                let next = self.displayed_month.add_months(1);
                if self.year_range.contains(&next.year) {
                    self.displayed_month = next;
                }
            }
            DateRangePickerAction::ToggleYearPicker => {
                self.year_picker_visible = !self.year_picker_visible;
                self.animation
                    .set_year_picker_visible(self.year_picker_visible, now);
            }
            DateRangePickerAction::SelectYear(year) => {
                if self.year_range.contains(&year) {
                    self.displayed_month.year = year;
                    self.year_picker_visible = false;
                    self.animation.set_year_picker_visible(false, now);
                }
            }
            DateRangePickerAction::SetDisplayMode(display_mode) => {
                if self.display_mode != display_mode {
                    self.animation
                        .display
                        .start(self.display_mode, display_mode, now);

                    if display_mode == DateDisplayMode::Input {
                        self.sync_input_values_from_selection();
                    } else if let Some(start) = self.selected_start_date {
                        self.displayed_month = start.month_start();
                    }

                    self.display_mode = display_mode;
                    self.year_picker_visible = false;
                    self.animation.set_year_picker_visible(false, now);
                }
            }
            DateRangePickerAction::StartInputChanged(value) => {
                self.start_input_value = normalize_date_input(value);
                let start = self.range_input_date(self.start_input_value());

                if date_input_digit_count(&self.start_input_value) < 8 {
                    self.selected_start_date = None;
                    self.selected_end_date = None;
                } else if self.start_input_error().is_none()
                    && let Some(start) = start
                {
                    if self.selected_end_date.is_none_or(|end| start <= end) {
                        self.selected_start_date = Some(start);
                        self.displayed_month = start.month_start();
                        self.animation.select_date(start, now);
                    }
                } else {
                    self.selected_start_date = None;
                }
            }
            DateRangePickerAction::EndInputChanged(value) => {
                self.end_input_value = normalize_date_input(value);
                let end = self.range_input_date(self.end_input_value());

                if date_input_digit_count(&self.end_input_value) < 8 {
                    self.selected_end_date = None;
                } else if self.end_input_error().is_none()
                    && let Some(end) = end
                {
                    if self.selected_start_date.is_some_and(|start| start <= end) {
                        self.selected_end_date = Some(end);
                        self.displayed_month = self
                            .selected_start_date
                            .map(Date::month_start)
                            .unwrap_or_else(|| end.month_start());
                        self.animation.select_date(end, now);
                    }
                } else {
                    self.selected_end_date = None;
                }
            }
        }
    }

    /// Advances any running Material range picker transitions.
    pub fn advance(&mut self, now: Instant) -> bool {
        self.animation.advance(now)
    }

    /// Returns whether this range picker needs frame ticks for animations.
    pub fn is_animating(&self) -> bool {
        self.animation.is_animating()
    }

    /// Returns a frame subscription while range picker animations are running.
    pub fn subscription<Message, F>(&self, on_frame: F) -> iced::Subscription<Message>
    where
        Message: 'static,
        F: Fn(Instant) -> Message + Send + Clone + 'static,
    {
        if self.is_animating() {
            iced::window::frames().map(on_frame)
        } else {
            iced::Subscription::none()
        }
    }

    fn date_is_in_range(&self, date: Date) -> bool {
        self.year_range.contains(&date.year)
    }

    fn date_is_selectable(&self, date: Date) -> bool {
        self.date_is_in_range(date)
            && self.selectable_dates.is_selectable_year(date.year)
            && self.selectable_dates.is_selectable_date(date)
    }

    fn date_range_position(&self, date: Date) -> DateRangePosition {
        match (self.selected_start_date, self.selected_end_date) {
            (Some(start), Some(end)) if start == end && date == start => DateRangePosition::Single,
            (Some(start), Some(_)) if date == start => DateRangePosition::Start,
            (Some(_), Some(end)) if date == end => DateRangePosition::End,
            (Some(start), Some(end)) if start < date && date < end => DateRangePosition::Middle,
            (Some(start), None) if date == start => DateRangePosition::Single,
            _ => DateRangePosition::None,
        }
    }

    fn range_input_date(&self, input: &str) -> Option<Date> {
        let date = parse_date_input(input)?;

        self.date_is_selectable(date).then_some(date)
    }

    fn sync_input_values_from_selection(&mut self) {
        self.start_input_value = self
            .selected_start_date
            .map(Date::format_input)
            .unwrap_or_default();
        self.end_input_value = self
            .selected_end_date
            .map(Date::format_input)
            .unwrap_or_default();
    }

    fn clamp_displayed_month(&mut self) {
        if self.displayed_month.year < *self.year_range.start() {
            self.displayed_month.year = *self.year_range.start();
            self.displayed_month.month = 1;
        } else if self.displayed_month.year > *self.year_range.end() {
            self.displayed_month.year = *self.year_range.end();
            self.displayed_month.month = 12;
        }

        if self
            .selected_start_date
            .is_some_and(|date| !self.date_is_selectable(date))
        {
            self.selected_start_date = None;
            self.selected_end_date = None;
        } else if self
            .selected_end_date
            .is_some_and(|date| !self.date_is_selectable(date))
        {
            self.selected_end_date = None;
        }
    }
}

/// Date range picker state transitions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DateRangePickerAction {
    SelectDate(Date),
    PreviousMonth,
    NextMonth,
    ToggleYearPicker,
    SelectYear(i32),
    SetDisplayMode(DateDisplayMode),
    StartInputChanged(String),
    EndInputChanged(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DateRangePosition {
    None,
    Single,
    Start,
    Middle,
    End,
}

impl DateRangePosition {
    fn draws_range_background(self) -> bool {
        matches!(self, Self::Start | Self::Middle | Self::End)
    }

    fn is_middle(self) -> bool {
        self == Self::Middle
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RangeMonthSelectionInfo {
    start_column: usize,
    start_row: usize,
    end_column: usize,
    end_row: usize,
    first_is_selection_start: bool,
    last_is_selection_end: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct RangeBackgroundRect {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DateInputRangeBound {
    StartBeforeOrEqual(Date),
    EndAfterOrEqual(Date),
}

/// Time picker selection mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimePickerSelectionMode {
    Hour,
    Minute,
}

/// AM/PM period.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Period {
    Am,
    Pm,
}

/// Layout used by the analog time picker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimePickerLayout {
    Vertical,
    Horizontal,
}

#[derive(Debug, Clone)]
struct TimePickerAnimation {
    clock_angle: AnimatedScalar,
    selection: AnimatedScalar,
    period: AnimatedScalar,
    previous_selection: TimePickerSelectionMode,
    selected_selection: TimePickerSelectionMode,
    previous_period: Period,
    selected_period: Period,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ClockUpdate {
    Animate,
    Drag,
    RawAngle(i32),
    Settle,
}

impl TimePickerAnimation {
    fn new(hour: u8, minute: u8, selection: TimePickerSelectionMode) -> Self {
        let period = period_for_hour(hour);

        Self {
            clock_angle: AnimatedScalar::new(time_selector_angle(hour, minute, selection)),
            selection: AnimatedScalar::new(1.0),
            period: AnimatedScalar::new(1.0),
            previous_selection: selection,
            selected_selection: selection,
            previous_period: period,
            selected_period: period,
        }
    }

    fn set_selection(
        &mut self,
        from: TimePickerSelectionMode,
        to: TimePickerSelectionMode,
        now: Instant,
    ) {
        if from == to {
            return;
        }

        self.previous_selection = from;
        self.selected_selection = to;
        self.selection = AnimatedScalar::new(0.0);
        self.selection.set_target(
            1.0,
            now,
            duration_ms(tokens::motion::DURATION_MEDIUM1_MS),
            tokens::motion::EASING_EMPHASIZED,
        );
    }

    fn set_period(&mut self, from: Period, to: Period, now: Instant) {
        if from == to {
            return;
        }

        let start = self.period_progress(to);
        self.previous_period = from;
        self.selected_period = to;
        self.period = AnimatedScalar::new(start);
        self.period.set_target(
            1.0,
            now,
            duration_ms(tokens::motion::DURATION_SHORT4_MS),
            tokens::motion::EASING_EMPHASIZED,
        );
    }

    fn set_clock_target(
        &mut self,
        hour: u8,
        minute: u8,
        selection: TimePickerSelectionMode,
        now: Instant,
    ) {
        let target = nearest_angle(
            self.clock_angle.value,
            time_selector_angle(hour, minute, selection),
        );
        if (target - self.clock_angle.value).abs() <= 1.0 / CLOCK_ANGLE_SCALE {
            self.clock_angle = AnimatedScalar::new(target);
            return;
        }

        self.clock_angle.set_target(
            target,
            now,
            duration_ms(tokens::motion::DURATION_SHORT4_MS),
            tokens::motion::EASING_EMPHASIZED,
        );
    }

    fn snap_clock_angle(&mut self, angle: f32) {
        let target = nearest_angle(self.clock_angle.value, angle);
        self.clock_angle = AnimatedScalar::new(target);
    }

    fn advance(&mut self, now: Instant) -> bool {
        self.clock_angle.advance(now) | self.selection.advance(now) | self.period.advance(now)
    }

    fn is_animating(&self) -> bool {
        self.clock_angle.is_animating()
            || self.selection.is_animating()
            || self.period.is_animating()
    }

    fn clock_angle(&self) -> f32 {
        self.clock_angle.value
    }

    fn selection_progress(&self, selection: TimePickerSelectionMode) -> f32 {
        let progress = self.selection.value.clamp(0.0, 1.0);

        if selection == self.selected_selection {
            progress
        } else if selection == self.previous_selection {
            1.0 - progress
        } else {
            0.0
        }
    }

    fn period_progress(&self, period: Period) -> f32 {
        if self.previous_period == self.selected_period {
            return if period == self.selected_period {
                1.0
            } else {
                0.0
            };
        }

        let progress = self.period.value.clamp(0.0, 1.0);

        if period == self.selected_period {
            progress
        } else if period == self.previous_period {
            1.0 - progress
        } else {
            0.0
        }
    }
}

/// Display mode used by a time picker dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimePickerDisplayMode {
    Picker,
    Input,
    Scroll,
}

impl TimePickerDisplayMode {
    /// Returns the alternate official dialog display mode.
    pub fn toggled(self) -> Self {
        match self {
            Self::Picker => Self::Input,
            Self::Input | Self::Scroll => Self::Picker,
        }
    }

    /// Returns the alternate official scroll dialog display mode.
    pub fn scroll_toggled(self) -> Self {
        match self {
            Self::Scroll => Self::Input,
            Self::Picker | Self::Input => Self::Scroll,
        }
    }

    fn title(self) -> &'static str {
        match self {
            Self::Picker => "Select Time",
            Self::Input => "Enter Time",
            Self::Scroll => "Select Time",
        }
    }

    fn toggle_icon(self) -> &'static str {
        match self {
            Self::Picker => "keyboard",
            Self::Input => "schedule",
            Self::Scroll => "keyboard",
        }
    }

    pub fn scroll_toggle_icon(self) -> &'static str {
        match self {
            Self::Scroll => "keyboard",
            Self::Picker | Self::Input => "swipe_vertical",
        }
    }
}

/// State held by a Material time picker.
#[derive(Debug, Clone)]
pub struct TimePickerState {
    hour: u8,
    minute: u8,
    is_24_hour: bool,
    selection: TimePickerSelectionMode,
    auto_switch_to_minute: bool,
    hour_input: String,
    minute_input: String,
    scroll_hour_anchor: u8,
    scroll_minute_anchor: u8,
    animation: TimePickerAnimation,
}

impl TimePickerState {
    /// Creates time picker state. Invalid initial values are clamped to official ranges.
    pub fn new(hour: u8, minute: u8, is_24_hour: bool) -> Self {
        let hour = hour.min(23);
        let minute = minute.min(59);

        Self {
            hour,
            minute,
            is_24_hour,
            selection: TimePickerSelectionMode::Hour,
            auto_switch_to_minute: true,
            hour_input: two_digit(hour_for_display(hour, is_24_hour)),
            minute_input: two_digit(minute),
            scroll_hour_anchor: if is_24_hour {
                hour
            } else {
                hour_for_display(hour, is_24_hour)
            },
            scroll_minute_anchor: minute,
            animation: TimePickerAnimation::new(hour, minute, TimePickerSelectionMode::Hour),
        }
    }

    /// Sets whether analog hour taps automatically move focus to minute selection.
    pub fn auto_switch_to_minute(mut self, auto_switch_to_minute: bool) -> Self {
        self.auto_switch_to_minute = auto_switch_to_minute;
        self
    }

    pub fn hour(&self) -> u8 {
        self.hour
    }

    pub fn minute(&self) -> u8 {
        self.minute
    }

    pub fn is_24_hour(&self) -> bool {
        self.is_24_hour
    }

    pub fn selection(&self) -> TimePickerSelectionMode {
        self.selection
    }

    pub fn auto_switches_to_minute(&self) -> bool {
        self.auto_switch_to_minute
    }

    pub fn hour_input(&self) -> &str {
        &self.hour_input
    }

    pub fn minute_input(&self) -> &str {
        &self.minute_input
    }

    pub fn period(&self) -> Period {
        period_for_hour(self.hour)
    }

    pub fn hour_for_display(&self) -> u8 {
        hour_for_display(self.hour, self.is_24_hour)
    }

    pub fn formatted_time(&self) -> String {
        if self.is_24_hour {
            format!("{:02}:{:02}", self.hour, self.minute)
        } else {
            let suffix = if self.period() == Period::Pm {
                "PM"
            } else {
                "AM"
            };
            format!("{:02}:{:02} {suffix}", self.hour_for_display(), self.minute)
        }
    }

    pub fn is_hour_input_valid(&self) -> bool {
        let Ok(value) = self.hour_input.parse::<u8>() else {
            return false;
        };
        let min = if self.is_24_hour { 0 } else { 1 };
        let max = if self.is_24_hour { 23 } else { 12 };

        (min..=max).contains(&value)
    }

    pub fn is_minute_input_valid(&self) -> bool {
        self.minute_input
            .parse::<u8>()
            .is_ok_and(|value| value <= 59)
    }

    pub fn is_input_valid(&self) -> bool {
        self.is_hour_input_valid() && self.is_minute_input_valid()
    }

    /// Applies a time picker action.
    pub fn update(&mut self, action: TimePickerAction) {
        self.update_at(action, Instant::now());
    }

    /// Applies a time picker action, using `now` as the transition start time.
    pub fn update_at(&mut self, action: TimePickerAction, now: Instant) {
        let previous_selection = self.selection;
        let previous_period = self.period();
        let clock_update = match &action {
            TimePickerAction::DragHour(_) | TimePickerAction::DragMinute(_) => ClockUpdate::Drag,
            TimePickerAction::DragHourAngle(_, angle)
            | TimePickerAction::DragMinuteAngle(_, angle) => ClockUpdate::RawAngle(*angle),
            TimePickerAction::FinishDrag => ClockUpdate::Settle,
            _ => ClockUpdate::Animate,
        };

        match action {
            TimePickerAction::SetSelection(selection) => self.selection = selection,
            TimePickerAction::SelectHour(hour)
            | TimePickerAction::DragHour(hour)
            | TimePickerAction::DragHourAngle(hour, _) => {
                self.hour = if self.is_24_hour {
                    hour.min(23)
                } else {
                    hour_to_24(hour, self.period())
                };
                self.hour_input = two_digit(self.hour_for_display());
                self.sync_hour_scroll_anchor();
            }
            TimePickerAction::SelectMinute(minute) => {
                self.minute = minute.min(59);
                self.minute_input = two_digit(self.minute);
                self.sync_minute_scroll_anchor();
            }
            TimePickerAction::DragMinute(minute) => {
                self.minute = minute.min(59);
                self.minute_input = two_digit(self.minute);
                self.sync_minute_scroll_anchor();
            }
            TimePickerAction::DragMinuteAngle(minute, _) => {
                self.minute = minute.min(59);
                self.minute_input = two_digit(self.minute);
                self.sync_minute_scroll_anchor();
            }
            TimePickerAction::ScrollHour(hour) => {
                self.hour = if self.is_24_hour {
                    hour.min(23)
                } else {
                    hour_to_24(hour, self.period())
                };
                self.hour_input = two_digit(self.hour_for_display());
            }
            TimePickerAction::ScrollMinute(minute) => {
                self.minute = minute.min(59);
                self.minute_input = two_digit(self.minute);
            }
            TimePickerAction::SetPeriod(period) => {
                let display = self.hour_for_display();
                self.hour = hour_to_24(display, period);
                self.sync_hour_scroll_anchor();
            }
            TimePickerAction::TogglePeriod => {
                self.hour = (self.hour + 12) % 24;
                self.sync_hour_scroll_anchor();
            }
            TimePickerAction::HourInputChanged(input) => {
                self.hour_input = keep_digits(input, 2);
                if let Ok(value) = self.hour_input.parse::<u8>() {
                    let max = if self.is_24_hour { 23 } else { 12 };
                    let min = if self.is_24_hour { 0 } else { 1 };
                    if (min..=max).contains(&value) {
                        self.hour = if self.is_24_hour {
                            value
                        } else {
                            hour_to_24(value, self.period())
                        };
                        self.sync_hour_scroll_anchor();
                        if self.auto_switch_to_minute && !self.is_24_hour && value > 1 {
                            self.selection = TimePickerSelectionMode::Minute;
                        }
                    }
                }
            }
            TimePickerAction::MinuteInputChanged(input) => {
                self.minute_input = keep_digits(input, 2);
                if let Ok(value) = self.minute_input.parse::<u8>()
                    && value <= 59
                {
                    self.minute = value;
                    self.sync_minute_scroll_anchor();
                }
            }
            TimePickerAction::FinishDrag => {}
        }

        self.animation
            .set_selection(previous_selection, self.selection, now);
        self.animation
            .set_period(previous_period, self.period(), now);
        match clock_update {
            ClockUpdate::Drag => {
                self.animation.snap_clock_angle(time_selector_angle(
                    self.hour,
                    self.minute,
                    self.selection,
                ));
            }
            ClockUpdate::RawAngle(angle) => {
                self.animation.snap_clock_angle(unpack_angle(angle));
            }
            ClockUpdate::Settle | ClockUpdate::Animate => {
                self.animation
                    .set_clock_target(self.hour, self.minute, self.selection, now);
            }
        }
    }

    /// Advances any running Material time picker transitions.
    pub fn advance(&mut self, now: Instant) -> bool {
        self.animation.advance(now)
    }

    /// Returns whether this picker needs frame ticks for animations.
    pub fn is_animating(&self) -> bool {
        self.animation.is_animating()
    }

    /// Returns a frame subscription while picker animations are running.
    pub fn subscription<Message, F>(&self, on_frame: F) -> iced::Subscription<Message>
    where
        Message: 'static,
        F: Fn(Instant) -> Message + Send + Clone + 'static,
    {
        if self.is_animating() {
            iced::window::frames().map(on_frame)
        } else {
            iced::Subscription::none()
        }
    }

    fn sync_hour_scroll_anchor(&mut self) {
        self.scroll_hour_anchor = if self.is_24_hour {
            self.hour
        } else {
            self.hour_for_display()
        };
    }

    fn sync_minute_scroll_anchor(&mut self) {
        self.scroll_minute_anchor = self.minute;
    }
}

impl Default for TimePickerState {
    fn default() -> Self {
        Self::new(12, 0, false)
    }
}

/// Time picker state transitions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TimePickerAction {
    SetSelection(TimePickerSelectionMode),
    SelectHour(u8),
    SelectMinute(u8),
    DragHour(u8),
    DragMinute(u8),
    #[doc(hidden)]
    DragHourAngle(u8, i32),
    #[doc(hidden)]
    DragMinuteAngle(u8, i32),
    #[doc(hidden)]
    FinishDrag,
    ScrollHour(u8),
    ScrollMinute(u8),
    SetPeriod(Period),
    TogglePeriod,
    HourInputChanged(String),
    MinuteInputChanged(String),
}

/// Creates a Material 3 date picker.
pub fn date_picker<'a, Message, Renderer>(
    state: &'a DatePickerState,
    on_action: impl Fn(DatePickerAction) -> Message + Clone + 'a,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    date_picker_with_mode_toggle(state, on_action, true)
}

/// Creates a Material 3 date picker with explicit mode toggle visibility.
pub fn date_picker_with_mode_toggle<'a, Message, Renderer>(
    state: &'a DatePickerState,
    on_action: impl Fn(DatePickerAction) -> Message + Clone + 'a,
    show_mode_toggle: bool,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    Container::new(date_picker_body(state, on_action, show_mode_toggle))
        .width(Length::Fixed(
            tokens::component::date_picker::CONTAINER_WIDTH,
        ))
        .height(Length::Fixed(
            tokens::component::date_picker::CONTAINER_HEIGHT,
        ))
        .style(date_picker_container_style)
        .into()
}

/// Creates a Material 3 date range picker.
pub fn date_range_picker<'a, Message, Renderer>(
    state: &'a DateRangePickerState,
    on_action: impl Fn(DateRangePickerAction) -> Message + Clone + 'a,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    date_range_picker_with_mode_toggle(state, on_action, true)
}

/// Creates a Material 3 date range picker with explicit mode toggle visibility.
pub fn date_range_picker_with_mode_toggle<'a, Message, Renderer>(
    state: &'a DateRangePickerState,
    on_action: impl Fn(DateRangePickerAction) -> Message + Clone + 'a,
    show_mode_toggle: bool,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    Container::new(date_range_picker_body(
        state,
        on_action,
        show_mode_toggle,
        tokens::component::date_picker::CONTAINER_HEIGHT,
    ))
    .width(Length::Fixed(
        tokens::component::date_picker::CONTAINER_WIDTH,
    ))
    .height(Length::Fixed(
        tokens::component::date_picker::CONTAINER_HEIGHT,
    ))
    .style(date_picker_container_style)
    .into()
}

/// Creates a Material 3 date picker dialog surface.
pub fn date_picker_dialog<'a, Message, Renderer>(
    state: &'a DatePickerState,
    on_action: impl Fn(DatePickerAction) -> Message + Clone + 'a,
    actions: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    date_picker_dialog_with_mode_toggle(state, on_action, true, actions)
}

/// Creates a Material 3 date picker dialog surface with explicit mode toggle visibility.
pub fn date_picker_dialog_with_mode_toggle<'a, Message, Renderer>(
    state: &'a DatePickerState,
    on_action: impl Fn(DatePickerAction) -> Message + Clone + 'a,
    show_mode_toggle: bool,
    actions: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    date_picker_dialog_surface(
        date_picker_body(state, on_action, show_mode_toggle),
        actions,
    )
}

/// Creates a Material 3 date range picker dialog surface.
pub fn date_range_picker_dialog<'a, Message, Renderer>(
    state: &'a DateRangePickerState,
    on_action: impl Fn(DateRangePickerAction) -> Message + Clone + 'a,
    actions: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    date_range_picker_dialog_with_mode_toggle(state, on_action, true, actions)
}

/// Creates a Material 3 date range picker dialog surface with explicit mode toggle visibility.
pub fn date_range_picker_dialog_with_mode_toggle<'a, Message, Renderer>(
    state: &'a DateRangePickerState,
    on_action: impl Fn(DateRangePickerAction) -> Message + Clone + 'a,
    show_mode_toggle: bool,
    actions: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    date_picker_dialog_surface(
        date_range_picker_body(
            state,
            on_action,
            show_mode_toggle,
            date_picker_dialog_content_height(),
        ),
        actions,
    )
}

/// Creates a right-aligned Material 3 date picker dialog actions row.
pub fn date_picker_dialog_actions<'a, Message, Renderer>(
    buttons: impl IntoIterator<Item = Element<'a, Message, Theme, Renderer>>,
) -> Container<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + 'a,
{
    Container::new(
        Row::with_children(buttons.into_iter())
            .spacing(tokens::component::date_picker::DIALOG_ACTIONS_MAIN_AXIS_SPACE)
            .align_y(alignment::Vertical::Center),
    )
    .width(Length::Fill)
    .align_x(alignment::Horizontal::Right)
}

/// Creates a Material 3 analog time picker.
pub fn time_picker<'a, Message, Renderer>(
    state: &'a TimePickerState,
    on_action: impl Fn(TimePickerAction) -> Message + Clone + 'a,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'static + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    time_picker_with_layout(state, on_action, TimePickerLayout::Vertical)
}

/// Creates a Material 3 analog time picker with an explicit layout.
pub fn time_picker_with_layout<'a, Message, Renderer>(
    state: &'a TimePickerState,
    on_action: impl Fn(TimePickerAction) -> Message + Clone + 'a,
    layout: TimePickerLayout,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'static + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    Container::new(time_picker_body(state, on_action, layout))
        .padding(24.0)
        .style(time_picker_container_style)
        .into()
}

/// Creates a Material 3 time picker dialog surface.
pub fn time_picker_dialog<'a, Message, Renderer>(
    state: &'a TimePickerState,
    display_mode: TimePickerDisplayMode,
    on_action: impl Fn(TimePickerAction) -> Message + Clone + 'a,
    on_display_mode_toggle: Option<Message>,
    actions: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'static + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    let content = match display_mode {
        TimePickerDisplayMode::Picker => {
            time_picker_body(state, on_action, TimePickerLayout::Vertical)
        }
        TimePickerDisplayMode::Input => time_input_body(state, on_action),
        TimePickerDisplayMode::Scroll => time_scroll_body(state, on_action),
    };

    let mut actions_row = Row::new()
        .height(Length::Fixed(tokens::component::button::CONTAINER_HEIGHT))
        .spacing(tokens::component::time_picker_dialog::ACTIONS_HORIZONTAL_SPACE)
        .align_y(alignment::Vertical::Center);

    if let Some(message) = on_display_mode_toggle {
        actions_row = actions_row.push(time_picker_display_mode_toggle(display_mode, message));
    }

    actions_row = actions_row
        .push(Space::new().width(Length::Fill))
        .push(actions.into());

    Container::new(
        Column::new()
            .push(time_picker_dialog_title::<Message, Renderer>(display_mode))
            .push(
                Container::new(content)
                    .width(Length::Fill)
                    .height(time_picker_dialog_content_height(display_mode))
                    .align_x(alignment::Horizontal::Center)
                    .align_y(alignment::Vertical::Center),
            )
            .push(
                Container::new(actions_row)
                    .width(Length::Fill)
                    .padding(Padding {
                        top: 0.0,
                        right: tokens::component::time_picker_dialog::CONTENT_PADDING,
                        bottom: tokens::component::time_picker_dialog::ACTIONS_BOTTOM_SPACE,
                        left: tokens::component::time_picker_dialog::CONTENT_PADDING,
                    }),
            ),
    )
    .style(time_picker_dialog_container_style)
    .into()
}

fn time_picker_dialog_content_height(display_mode: TimePickerDisplayMode) -> Length {
    match display_mode {
        TimePickerDisplayMode::Picker => Length::Shrink,
        TimePickerDisplayMode::Input | TimePickerDisplayMode::Scroll => {
            Length::Fixed(tokens::component::time_picker_dialog::MIN_HEIGHT_FOR_TIME_PICKER)
        }
    }
}

/// Creates a Material 3 rich time picker dialog surface.
pub fn rich_time_picker_dialog<'a, Message, Renderer>(
    state: &'a TimePickerState,
    display_mode: TimePickerDisplayMode,
    on_action: impl Fn(TimePickerAction) -> Message + Clone + 'a,
    on_display_mode_toggle: Option<Message>,
    actions: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'static + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    let content = match display_mode {
        TimePickerDisplayMode::Picker => {
            time_picker_body(state, on_action, TimePickerLayout::Vertical)
        }
        TimePickerDisplayMode::Input => time_input_body(state, on_action),
        TimePickerDisplayMode::Scroll => time_scroll_body(state, on_action),
    };

    let mut actions_row = Row::new()
        .height(Length::Fixed(tokens::component::button::CONTAINER_HEIGHT))
        .spacing(tokens::component::time_picker_dialog::ACTIONS_HORIZONTAL_SPACE)
        .align_y(alignment::Vertical::Center);

    if let Some(message) = on_display_mode_toggle {
        actions_row = actions_row.push(time_picker_display_mode_toggle(display_mode, message));
    }

    actions_row = actions_row
        .push(Space::new().width(Length::Fill))
        .push(actions.into());

    Container::new(
        Column::new()
            .push(
                Container::new(content)
                    .width(Length::Fill)
                    .padding(Padding {
                        top: tokens::component::time_picker_dialog::RICH_CONTENT_TOP_SPACE,
                        right: tokens::component::time_picker_dialog::RICH_CONTENT_PADDING,
                        bottom: tokens::component::time_picker_dialog::RICH_CONTENT_ACTIONS_SPACE,
                        left: tokens::component::time_picker_dialog::RICH_CONTENT_PADDING,
                    })
                    .align_x(alignment::Horizontal::Center),
            )
            .push(
                Container::new(actions_row)
                    .width(Length::Fill)
                    .padding(Padding {
                        top: 0.0,
                        right: tokens::component::time_picker_dialog::RICH_CONTENT_PADDING,
                        bottom: tokens::component::time_picker_dialog::RICH_ACTIONS_BOTTOM_SPACE,
                        left: tokens::component::time_picker_dialog::RICH_CONTENT_PADDING,
                    }),
            ),
    )
    .style(rich_time_picker_dialog_container_style)
    .into()
}

fn date_picker_body<'a, Message, Renderer>(
    state: &'a DatePickerState,
    on_action: impl Fn(DatePickerAction) -> Message + Clone + 'a,
    show_mode_toggle: bool,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    let picker_alpha = state.animation.display.mode_alpha(DateDisplayMode::Picker);
    let input_alpha = state.animation.display.mode_alpha(DateDisplayMode::Input);

    Column::new()
        .push(date_header(state, on_action.clone(), show_mode_toggle))
        .push(animated_date_display_content(
            date_picker_content(state, on_action.clone(), picker_alpha),
            date_input_content(state, on_action, input_alpha),
            state.display_mode,
            &state.animation.display,
            date_display_picker_height(),
            date_display_input_height(),
        ))
        .into()
}

fn date_range_picker_body<'a, Message, Renderer>(
    state: &'a DateRangePickerState,
    on_action: impl Fn(DateRangePickerAction) -> Message + Clone + 'a,
    show_mode_toggle: bool,
    container_height: f32,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    let picker_alpha = state.animation.display.mode_alpha(DateDisplayMode::Picker);
    let input_alpha = state.animation.display.mode_alpha(DateDisplayMode::Input);

    Column::new()
        .push(date_range_header(
            state,
            on_action.clone(),
            show_mode_toggle,
        ))
        .push(animated_date_display_content(
            date_range_picker_content(state, on_action.clone(), container_height, picker_alpha),
            date_range_input_content(state, on_action, input_alpha),
            state.display_mode,
            &state.animation.display,
            container_height - tokens::component::date_picker::RANGE_HEADER_CONTAINER_HEIGHT,
            date_display_input_height(),
        ))
        .into()
}

fn animated_date_display_content<'a, Message, Renderer>(
    picker: Element<'a, Message, Theme, Renderer>,
    input: Element<'a, Message, Theme, Renderer>,
    display_mode: DateDisplayMode,
    animation: &DateDisplayAnimation,
    picker_height: f32,
    input_height: f32,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + 'a,
{
    let height = animation.content_height(picker_height, input_height, display_mode);

    if !animation.is_animating() {
        let content = match display_mode {
            DateDisplayMode::Picker => picker,
            DateDisplayMode::Input => input,
        };

        return Container::new(content)
            .height(Length::Fixed(height))
            .clip(true)
            .into();
    }

    let picker = translated(
        picker,
        Vector::new(
            0.0,
            animation.mode_offset(DateDisplayMode::Picker, input_height),
        ),
    );
    let input = translated(
        input,
        Vector::new(
            0.0,
            animation.mode_offset(DateDisplayMode::Input, input_height),
        ),
    );

    let stack = match (animation.source_mode(), animation.target_mode()) {
        (DateDisplayMode::Picker, DateDisplayMode::Input) => Stack::new().push(picker).push(input),
        (DateDisplayMode::Input, DateDisplayMode::Picker) => Stack::new().push(input).push(picker),
        _ => match display_mode {
            DateDisplayMode::Picker => Stack::new().push(picker),
            DateDisplayMode::Input => Stack::new().push(input),
        },
    };

    viewport::fixed_height(
        stack.width(Length::Fill).height(Length::Fixed(
            animation.content_layout_height(picker_height, input_height),
        )),
        height,
        animation.content_layout_height(picker_height, input_height),
    )
}

fn date_display_picker_height() -> f32 {
    tokens::component::date_picker::MONTH_YEAR_CONTAINER_HEIGHT
        + tokens::component::date_picker::WEEKDAY_CONTAINER_HEIGHT
        + tokens::component::date_picker::CALENDAR_CELL_SIZE
            * tokens::component::date_picker::MAX_CALENDAR_ROWS as f32
}

fn date_display_input_height() -> f32 {
    10.0 + tokens::component::text_field::CONTAINER_HEIGHT
        + 4.0
        + tokens::typography::BODY_SMALL.line_height
}

fn date_picker_dialog_surface<'a, Message, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
    actions: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + 'a,
{
    Container::new(
        Column::new()
            .push(content)
            .push(date_picker_dialog_actions_container(actions)),
    )
    .width(Length::Fixed(
        tokens::component::date_picker::CONTAINER_WIDTH,
    ))
    .max_height(tokens::component::date_picker::CONTAINER_HEIGHT)
    .style(date_picker_container_style)
    .into()
}

fn date_picker_dialog_actions_container<'a, Message, Renderer>(
    actions: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + 'a,
{
    Container::new(actions)
        .width(Length::Fill)
        .height(Length::Fixed(tokens::component::button::CONTAINER_HEIGHT))
        .align_x(alignment::Horizontal::Right)
        .padding(Padding {
            top: 0.0,
            right: tokens::component::date_picker::DIALOG_ACTIONS_END_SPACE,
            bottom: tokens::component::date_picker::DIALOG_ACTIONS_BOTTOM_SPACE,
            left: 0.0,
        })
        .into()
}

fn date_picker_dialog_content_height() -> f32 {
    tokens::component::date_picker::CONTAINER_HEIGHT
        - tokens::component::button::CONTAINER_HEIGHT
        - tokens::component::date_picker::DIALOG_ACTIONS_BOTTOM_SPACE
}

fn time_picker_body<'a, Message, Renderer>(
    state: &'a TimePickerState,
    on_action: impl Fn(TimePickerAction) -> Message + Clone + 'a,
    layout: TimePickerLayout,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'static + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    let clock = clock_face(state, on_action.clone());

    match layout {
        TimePickerLayout::Vertical => Column::new()
            .push(clock_display(state, on_action.clone(), true))
            .push(Space::new().height(Length::Fixed(
                tokens::component::time_picker::CLOCK_DISPLAY_BOTTOM_SPACE,
            )))
            .push(clock)
            .push(Space::new().height(Length::Fixed(
                tokens::component::time_picker::CLOCK_FACE_BOTTOM_SPACE,
            )))
            .align_x(alignment::Horizontal::Center)
            .into(),
        TimePickerLayout::Horizontal => Row::new()
            .push(clock_display(state, on_action.clone(), false))
            .push(Space::new().width(Length::Fixed(
                tokens::component::time_picker::CLOCK_DISPLAY_BOTTOM_SPACE,
            )))
            .push(clock)
            .align_y(alignment::Vertical::Center)
            .into(),
    }
}

fn time_input_body<'a, Message, Renderer>(
    state: &'a TimePickerState,
    on_action: impl Fn(TimePickerAction) -> Message + Clone + 'a,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    let hour = time_input_field(state, on_action.clone(), TimePickerSelectionMode::Hour);
    let minute = time_input_field(state, on_action.clone(), TimePickerSelectionMode::Minute);
    let mut content = Row::new()
        .push(hour)
        .push(display_separator_with_height::<Message, Renderer>(
            tokens::component::time_input::PERIOD_SELECTOR_CONTAINER_HEIGHT,
        ))
        .push(minute)
        .spacing(0.0)
        .align_y(alignment::Vertical::Top);

    if !state.is_24_hour {
        content = content
            .push(Space::new().width(Length::Fixed(
                tokens::component::time_picker::PERIOD_TOGGLE_MARGIN,
            )))
            .push(period_toggle_sized(
                state,
                on_action,
                true,
                tokens::component::time_input::PERIOD_SELECTOR_CONTAINER_WIDTH,
                tokens::component::time_input::PERIOD_SELECTOR_CONTAINER_HEIGHT,
            ));
    }

    content.into()
}

fn time_scroll_body<'a, Message, Renderer>(
    state: &'a TimePickerState,
    on_action: impl Fn(TimePickerAction) -> Message + Clone + 'a,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    let hour = time_scroll_field(state, on_action.clone(), TimePickerSelectionMode::Hour);
    let minute = time_scroll_field(state, on_action.clone(), TimePickerSelectionMode::Minute);
    let mut content = Row::new()
        .push(hour)
        .push(display_separator_with_size::<Message, Renderer>(
            tokens::component::time_picker::TIME_SCROLL_SEPARATOR_WIDTH,
            tokens::component::time_picker::TIME_SCROLL_FIELD_HEIGHT,
        ))
        .push(minute)
        .align_y(alignment::Vertical::Top);

    if !state.is_24_hour {
        content = content
            .push(Space::new().width(Length::Fixed(
                tokens::component::time_picker::RICH_PERIOD_SELECTOR_START_SPACE,
            )))
            .push(period_toggle_sized(
                state,
                on_action,
                true,
                tokens::component::time_picker::RICH_PERIOD_SELECTOR_WIDTH,
                tokens::component::time_picker::RICH_PERIOD_SELECTOR_HEIGHT,
            ));
    }

    content.into()
}

fn time_scroll_field<'a, Message, Renderer>(
    state: &'a TimePickerState,
    on_action: impl Fn(TimePickerAction) -> Message + Clone + 'a,
    selection: TimePickerSelectionMode,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    let scroll_field = Canvas::new(TimeScrollField {
        selection,
        is_24_hour: state.is_24_hour,
        selected_value: time_scroll_selected_value(state, selection),
        anchor_value: time_scroll_anchor_value(state, selection),
        option_count: time_scroll_option_count(state, selection),
        on_action: Arc::new(on_action),
    })
    .width(Length::Fixed(
        tokens::component::time_picker::TIME_SCROLL_FIELD_WIDTH,
    ))
    .height(Length::Fixed(
        tokens::component::time_picker::TIME_SCROLL_FIELD_HEIGHT,
    ));

    Container::new(
        scroll_field
            .width(Length::Fixed(
                tokens::component::time_picker::TIME_SCROLL_FIELD_WIDTH,
            ))
            .height(Length::Fixed(
                tokens::component::time_picker::TIME_SCROLL_FIELD_HEIGHT,
            )),
    )
    .width(Length::Fixed(
        tokens::component::time_picker::TIME_SCROLL_FIELD_WIDTH,
    ))
    .height(Length::Fixed(
        tokens::component::time_picker::TIME_SCROLL_FIELD_HEIGHT,
    ))
    .clip(true)
    .style(time_scroll_field_container_style)
    .into()
}

struct TimeScrollField<F> {
    selection: TimePickerSelectionMode,
    is_24_hour: bool,
    selected_value: u8,
    anchor_value: u8,
    option_count: u8,
    on_action: Arc<F>,
}

#[derive(Debug, Clone, Default)]
struct TimeScrollFieldState {
    offset_y: f32,
    velocity_y: f32,
    last_frame: Option<Instant>,
    last_reported_value: Option<u8>,
    drag: Option<TimeScrollDrag>,
    selection: Option<TimePickerSelectionMode>,
    anchor_value: Option<u8>,
    option_count: u8,
    is_24_hour: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TimeScrollPointer {
    Mouse,
    Touch(touch::Finger),
}

impl TimeScrollPointer {
    fn is_touch(self) -> bool {
        matches!(self, Self::Touch(_))
    }
}

#[derive(Debug, Clone, Copy)]
struct TimeScrollDrag {
    pointer: TimeScrollPointer,
    start: Point,
    last: Point,
    last_at: Instant,
    has_scrolled: bool,
}

impl TimeScrollDrag {
    fn new(pointer: TimeScrollPointer, position: Point, now: Instant) -> Self {
        Self {
            pointer,
            start: position,
            last: position,
            last_at: now,
            has_scrolled: false,
        }
    }

    fn moved_beyond_slop(self, position: Point) -> bool {
        let dx = position.x - self.start.x;
        let dy = position.y - self.start.y;

        dx * dx + dy * dy > TIME_SCROLL_TOUCH_SLOP * TIME_SCROLL_TOUCH_SLOP
    }
}

impl<Message, F, Renderer> canvas::Program<Message, Theme, Renderer> for TimeScrollField<F>
where
    Message: Clone,
    F: Fn(TimePickerAction) -> Message,
    Renderer: geometry::Renderer,
{
    type State = TimeScrollFieldState;

    fn update(
        &self,
        state: &mut Self::State,
        event: &canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        self.sync_state(state);

        match event {
            event::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                self.press(state, event, bounds, cursor, TimeScrollPointer::Mouse)
            }
            event::Event::Touch(touch::Event::FingerPressed { id, .. }) => {
                self.press(state, event, bounds, cursor, TimeScrollPointer::Touch(*id))
            }
            event::Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                self.drag(state, event, bounds, cursor, TimeScrollPointer::Mouse)
            }
            event::Event::Touch(touch::Event::FingerMoved { id, .. }) => {
                self.drag(state, event, bounds, cursor, TimeScrollPointer::Touch(*id))
            }
            event::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                self.release(state, event, bounds, cursor, TimeScrollPointer::Mouse)
            }
            event::Event::Touch(
                touch::Event::FingerLifted { id, .. } | touch::Event::FingerLost { id, .. },
            ) => self.release(state, event, bounds, cursor, TimeScrollPointer::Touch(*id)),
            event::Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                if cursor.position_over(bounds).is_none() {
                    return None;
                }

                state.velocity_y = 0.0;
                state.last_frame = None;
                let delta_y = match *delta {
                    mouse::ScrollDelta::Lines { y, .. } => -y * 60.0,
                    mouse::ScrollDelta::Pixels { y, .. } => -y,
                };

                if self.scroll_by(state, delta_y) {
                    Some(self.scroll_or_redraw(state))
                } else {
                    Some(canvas::Action::capture())
                }
            }
            event::Event::Window(window::Event::RedrawRequested(now)) => {
                self.advance_inertia(state, *now)
            }
            _ => None,
        }
    }

    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry<Renderer>> {
        let mut frame = Frame::new(renderer, bounds.size());
        let width = tokens::component::time_picker::TIME_SCROLL_FIELD_WIDTH;
        let height = tokens::component::time_picker::TIME_SCROLL_FIELD_HEIGHT;
        let item_height = tokens::component::time_picker::TIME_SCROLL_ITEM_HEIGHT;
        let offset_y = self.effective_offset(state);
        let selected_row = time_scroll_row_offset_for_offset(self.option_count, offset_y);

        let layer_style = time_scroll_selection_layer_style(theme);
        if let Some(Background::Color(color)) = layer_style.background {
            let layer_top = (height - item_height) / 2.0;
            let layer = Path::rounded_rectangle(
                Point::new(0.0, layer_top),
                Size::new(width, item_height),
                layer_style.border.radius,
            );
            frame.fill(&layer, color);
        }

        for row_offset in -1..=i16::from(self.option_count) {
            let row_index = row_offset + 1;
            let top = f32::from(row_index) * item_height - offset_y;

            if top + item_height < 0.0 || top > height {
                continue;
            }

            let value = self.value_for_row_offset(row_offset);
            let selected = row_offset == selected_row;
            let scale = if selected {
                tokens::component::time_picker::TIME_SELECTOR_LABEL_TEXT
            } else {
                tokens::component::time_picker::CLOCK_DIAL_LABEL_TEXT
            };
            let style = time_scroll_item_style(theme, Status::Active, selected);
            let text = CanvasText {
                content: time_scroll_label(value, self.selection),
                position: Point::new(width / 2.0, top + item_height / 2.0),
                max_width: width,
                color: style.text_color,
                size: scale.size.into(),
                line_height: LineHeight::Absolute(scale.size.into()),
                font: fonts::roboto_for_type_scale(scale),
                align_x: core_text::Alignment::Center,
                align_y: alignment::Vertical::Center,
                shaping: text::Shaping::Advanced,
            };

            frame.fill_text(text);
        }

        vec![frame.into_geometry()]
    }

    fn mouse_interaction(
        &self,
        _state: &Self::State,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if cursor.is_over(bounds) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }
}

impl<F> TimeScrollField<F> {
    fn sync_state(&self, state: &mut TimeScrollFieldState) {
        if state.selection != Some(self.selection)
            || state.anchor_value != Some(self.anchor_value)
            || state.option_count != self.option_count
            || state.is_24_hour != self.is_24_hour
        {
            state.offset_y = 0.0;
            state.velocity_y = 0.0;
            state.last_frame = None;
            state.last_reported_value = Some(self.selected_value);
            state.drag = None;
            state.selection = Some(self.selection);
            state.anchor_value = Some(self.anchor_value);
            state.option_count = self.option_count;
            state.is_24_hour = self.is_24_hour;
        } else {
            state.offset_y = state.offset_y.clamp(0.0, self.max_offset());
        }
    }

    fn effective_offset(&self, state: &TimeScrollFieldState) -> f32 {
        if state.selection == Some(self.selection)
            && state.anchor_value == Some(self.anchor_value)
            && state.option_count == self.option_count
            && state.is_24_hour == self.is_24_hour
        {
            state.offset_y.clamp(0.0, self.max_offset())
        } else {
            0.0
        }
    }

    fn press<Message>(
        &self,
        state: &mut TimeScrollFieldState,
        event: &canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
        pointer: TimeScrollPointer,
    ) -> Option<canvas::Action<Message>> {
        let position = event_position(event, bounds, cursor)?;
        if !local_point_is_in_bounds(position, bounds.size()) {
            return None;
        }

        state.velocity_y = 0.0;
        state.last_frame = None;
        state.drag = Some(TimeScrollDrag::new(pointer, position, Instant::now()));

        Some(canvas::Action::capture())
    }

    fn drag<Message>(
        &self,
        state: &mut TimeScrollFieldState,
        event: &canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
        pointer: TimeScrollPointer,
    ) -> Option<canvas::Action<Message>>
    where
        F: Fn(TimePickerAction) -> Message,
    {
        let position = event_position(event, bounds, cursor)?;
        let now = Instant::now();
        let Some(drag) = state.drag.as_mut() else {
            return None;
        };

        if drag.pointer != pointer {
            return None;
        }

        let delta_y = if drag.has_scrolled {
            drag.last.y - position.y
        } else if drag.moved_beyond_slop(position) {
            drag.has_scrolled = true;
            drag.start.y - position.y
        } else {
            drag.last = position;
            drag.last_at = now;
            return Some(canvas::Action::capture());
        };
        let elapsed = now.duration_since(drag.last_at).as_secs_f32();

        drag.last = position;
        drag.last_at = now;

        if elapsed > 0.0 {
            state.velocity_y = (delta_y / elapsed).clamp(
                -TIME_SCROLL_FLING_MAX_VELOCITY,
                TIME_SCROLL_FLING_MAX_VELOCITY,
            );
        }

        if self.scroll_by(state, delta_y) {
            Some(self.scroll_or_redraw(state))
        } else {
            Some(canvas::Action::capture())
        }
    }

    fn release<Message>(
        &self,
        state: &mut TimeScrollFieldState,
        event: &canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
        pointer: TimeScrollPointer,
    ) -> Option<canvas::Action<Message>>
    where
        F: Fn(TimePickerAction) -> Message,
    {
        let position = event_position(event, bounds, cursor);
        let drag = state.drag?;

        if drag.pointer != pointer {
            return None;
        }

        state.drag = None;

        if !drag.has_scrolled {
            let position = position.unwrap_or(drag.last);
            let tap_offset = self.effective_offset(state);
            state.offset_y = 0.0;
            state.velocity_y = 0.0;
            state.last_frame = None;
            state.last_reported_value = Some(self.selected_value);
            let action = self.select_action_for_position(position.y, tap_offset);

            return Some(canvas::Action::publish((self.on_action)(action)).and_capture());
        }

        if pointer.is_touch() && state.velocity_y.abs() >= TIME_SCROLL_FLING_MIN_VELOCITY {
            state.velocity_y = state.velocity_y.clamp(
                -TIME_SCROLL_FLING_MAX_VELOCITY,
                TIME_SCROLL_FLING_MAX_VELOCITY,
            );
            state.last_frame = None;

            Some(canvas::Action::request_redraw().and_capture())
        } else {
            state.velocity_y = 0.0;
            state.last_frame = None;
            self.settle(state);
            Some(self.force_scroll_action(state))
        }
    }

    fn advance_inertia<Message>(
        &self,
        state: &mut TimeScrollFieldState,
        now: Instant,
    ) -> Option<canvas::Action<Message>>
    where
        F: Fn(TimePickerAction) -> Message,
    {
        if state.velocity_y.abs() < f32::EPSILON {
            return None;
        }

        let elapsed = state
            .last_frame
            .map(|last_frame| now.duration_since(last_frame).as_secs_f32())
            .unwrap_or(1.0 / 60.0)
            .clamp(0.0, 1.0 / 15.0);

        if elapsed <= 0.0 {
            return Some(canvas::Action::request_redraw().and_capture());
        }

        state.last_frame = Some(now);
        let velocity = state.velocity_y;
        let moved = self.scroll_by(state, velocity * elapsed);
        let speed = (velocity.abs() - TIME_SCROLL_FLING_DECELERATION * elapsed).max(0.0);

        state.velocity_y = velocity.signum() * speed;

        if !moved || speed < TIME_SCROLL_FLING_MIN_VELOCITY {
            state.velocity_y = 0.0;
            state.last_frame = None;
            self.settle(state);

            return Some(self.force_scroll_action(state));
        }

        Some(self.scroll_or_redraw(state))
    }

    fn scroll_or_redraw<Message>(&self, state: &mut TimeScrollFieldState) -> canvas::Action<Message>
    where
        F: Fn(TimePickerAction) -> Message,
    {
        let value = self.value_for_offset(state.offset_y);

        if state.last_reported_value != Some(value) {
            state.last_reported_value = Some(value);
            canvas::Action::publish((self.on_action)(self.scroll_action(value))).and_capture()
        } else {
            canvas::Action::request_redraw().and_capture()
        }
    }

    fn force_scroll_action<Message>(
        &self,
        state: &mut TimeScrollFieldState,
    ) -> canvas::Action<Message>
    where
        F: Fn(TimePickerAction) -> Message,
    {
        let value = self.value_for_offset(state.offset_y);
        state.last_reported_value = Some(value);

        canvas::Action::publish((self.on_action)(self.scroll_action(value))).and_capture()
    }

    fn scroll_by(&self, state: &mut TimeScrollFieldState, delta_y: f32) -> bool {
        let previous = state.offset_y;
        state.offset_y = (state.offset_y + delta_y).clamp(0.0, self.max_offset());

        (state.offset_y - previous).abs() > f32::EPSILON
    }

    fn settle(&self, state: &mut TimeScrollFieldState) {
        let row_offset = time_scroll_row_offset_for_offset(self.option_count, state.offset_y);
        state.offset_y =
            f32::from(row_offset) * tokens::component::time_picker::TIME_SCROLL_ITEM_HEIGHT;
    }

    fn max_offset(&self) -> f32 {
        time_scroll_max_offset(self.option_count)
    }

    fn value_for_offset(&self, offset_y: f32) -> u8 {
        let row_offset = time_scroll_row_offset_for_offset(self.option_count, offset_y);

        self.value_for_row_offset(row_offset)
    }

    fn value_for_row_offset(&self, row_offset: i16) -> u8 {
        time_scroll_value_for_anchor(
            self.anchor_value,
            self.selection,
            self.is_24_hour,
            row_offset,
        )
    }

    fn scroll_action(&self, value: u8) -> TimePickerAction {
        match self.selection {
            TimePickerSelectionMode::Hour => TimePickerAction::ScrollHour(value),
            TimePickerSelectionMode::Minute => TimePickerAction::ScrollMinute(value),
        }
    }

    fn select_action_for_position(&self, y: f32, offset_y: f32) -> TimePickerAction {
        let item_height = tokens::component::time_picker::TIME_SCROLL_ITEM_HEIGHT;
        let row_offset = ((y + offset_y) / item_height).floor() as i16 - 1;
        let value = self.value_for_row_offset(row_offset.clamp(-1, i16::from(self.option_count)));

        match self.selection {
            TimePickerSelectionMode::Hour => TimePickerAction::SelectHour(value),
            TimePickerSelectionMode::Minute => TimePickerAction::SelectMinute(value),
        }
    }
}

fn time_picker_dialog_title<'a, Message, Renderer>(
    display_mode: TimePickerDisplayMode,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    let scale = tokens::component::time_picker_dialog::TITLE_TEXT;

    Container::new(
        Text::new(display_mode.title())
            .size(scale.size)
            .line_height(absolute_line_height(scale.line_height))
            .font(fonts::roboto_for_type_scale(scale)),
    )
    .padding(Padding {
        top: tokens::component::time_picker_dialog::TITLE_TOP_SPACE,
        right: tokens::component::time_picker_dialog::CONTENT_PADDING,
        bottom: tokens::component::time_picker_dialog::TITLE_BOTTOM_SPACE,
        left: tokens::component::time_picker_dialog::CONTENT_PADDING,
    })
    .width(Length::Fill)
    .into()
}

fn time_picker_display_mode_toggle<'a, Message, Renderer>(
    display_mode: TimePickerDisplayMode,
    on_press: Message,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    super::button::icon_button(display_mode.toggle_icon()).on_press(on_press)
}

fn date_header<'a, Message, Renderer>(
    state: &'a DatePickerState,
    on_action: impl Fn(DatePickerAction) -> Message + Clone + 'a,
    show_mode_toggle: bool,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    let title = match state.display_mode {
        DateDisplayMode::Picker => "Select date",
        DateDisplayMode::Input => "Enter date",
    };
    let headline = state
        .selected_date
        .map(|date| state.formatter.format_date(date, false))
        .unwrap_or_else(|| "Select date".to_owned());
    let title_scale = tokens::component::date_picker::TITLE_TEXT;
    let headline_scale = tokens::component::date_picker::HEADLINE_TEXT;
    let mut headline_row = Row::new()
        .push(
            Text::new(headline)
                .size(headline_scale.size)
                .line_height(absolute_line_height(headline_scale.line_height))
                .font(fonts::roboto_for_type_scale(headline_scale))
                .width(Length::Fill),
        )
        .align_y(alignment::Vertical::Center);

    if show_mode_toggle {
        let toggle_mode = if state.display_mode == DateDisplayMode::Picker {
            DateDisplayMode::Input
        } else {
            DateDisplayMode::Picker
        };
        let toggle_icon = if state.display_mode == DateDisplayMode::Picker {
            "edit"
        } else {
            "calendar_today"
        };
        let toggle = super::button::icon_button(toggle_icon)
            .on_press(on_action(DatePickerAction::SetDisplayMode(toggle_mode)));
        headline_row = headline_row.push(toggle);
    }

    Container::new(
        Column::new()
            .push(
                Container::new(
                    Text::new(title)
                        .size(title_scale.size)
                        .line_height(absolute_line_height(title_scale.line_height))
                        .font(fonts::roboto_for_type_scale(title_scale)),
                )
                .padding(Padding {
                    top: tokens::component::date_picker::HEADER_TITLE_TOP_SPACE,
                    right: tokens::component::date_picker::HEADER_TITLE_END_SPACE,
                    bottom: 0.0,
                    left: tokens::component::date_picker::HEADER_TITLE_START_SPACE,
                }),
            )
            .push(Space::new().height(Length::Fill))
            .push(
                Container::new(headline_row)
                    .padding(Padding {
                        top: 0.0,
                        right: tokens::component::date_picker::HEADER_HEADLINE_END_SPACE,
                        bottom: tokens::component::date_picker::HEADER_HEADLINE_BOTTOM_SPACE,
                        left: tokens::component::date_picker::HEADER_HEADLINE_START_SPACE,
                    })
                    .width(Length::Fill),
            ),
    )
    .height(Length::Fixed(
        tokens::component::date_picker::HEADER_CONTAINER_HEIGHT,
    ))
    .style(date_header_style)
    .into()
}

fn date_range_header<'a, Message, Renderer>(
    state: &'a DateRangePickerState,
    on_action: impl Fn(DateRangePickerAction) -> Message + Clone + 'a,
    show_mode_toggle: bool,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    let title = match state.display_mode {
        DateDisplayMode::Picker => {
            if state.selected_start_date.is_none() || state.selected_end_date.is_some() {
                "Select start date"
            } else {
                "Select end date"
            }
        }
        DateDisplayMode::Input => "Enter dates",
    };
    let start = state
        .selected_start_date
        .map(|date| state.formatter.format_date(date, false))
        .unwrap_or_else(|| "Start date".to_owned());
    let end = state
        .selected_end_date
        .map(|date| state.formatter.format_date(date, false))
        .unwrap_or_else(|| "End date".to_owned());
    let title_scale = tokens::component::date_picker::TITLE_TEXT;
    let headline_scale = tokens::component::date_picker::RANGE_HEADLINE_TEXT;
    let mut headline_row = Row::new()
        .push(
            Text::new(format!("{start} - {end}"))
                .size(headline_scale.size)
                .line_height(absolute_line_height(headline_scale.line_height))
                .font(fonts::roboto_for_type_scale(headline_scale))
                .width(Length::Fill),
        )
        .align_y(alignment::Vertical::Center);

    if show_mode_toggle {
        let toggle_mode = if state.display_mode == DateDisplayMode::Picker {
            DateDisplayMode::Input
        } else {
            DateDisplayMode::Picker
        };
        let toggle_icon = if state.display_mode == DateDisplayMode::Picker {
            "edit"
        } else {
            "calendar_today"
        };
        let toggle = super::button::icon_button(toggle_icon).on_press(on_action(
            DateRangePickerAction::SetDisplayMode(toggle_mode),
        ));
        headline_row = headline_row.push(toggle);
    }

    Container::new(
        Column::new()
            .push(
                Container::new(
                    Text::new(title)
                        .size(title_scale.size)
                        .line_height(absolute_line_height(title_scale.line_height))
                        .font(fonts::roboto_for_type_scale(title_scale)),
                )
                .padding(Padding {
                    top: tokens::component::date_picker::HEADER_TITLE_TOP_SPACE,
                    right: tokens::component::date_picker::HEADER_TITLE_END_SPACE,
                    bottom: 0.0,
                    left: tokens::component::date_picker::HEADER_TITLE_START_SPACE,
                }),
            )
            .push(Space::new().height(Length::Fill))
            .push(
                Container::new(headline_row)
                    .padding(Padding {
                        top: 0.0,
                        right: tokens::component::date_picker::HEADER_HEADLINE_END_SPACE,
                        bottom: tokens::component::date_picker::HEADER_HEADLINE_BOTTOM_SPACE,
                        left: tokens::component::date_picker::HEADER_HEADLINE_START_SPACE,
                    })
                    .width(Length::Fill),
            ),
    )
    .height(Length::Fixed(
        tokens::component::date_picker::RANGE_HEADER_CONTAINER_HEIGHT,
    ))
    .style(date_header_style)
    .into()
}

fn date_picker_content<'a, Message, Renderer>(
    state: &'a DatePickerState,
    on_action: impl Fn(DatePickerAction) -> Message + Clone + 'a,
    content_alpha: f32,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    let year_picker_progress = state.animation.year_picker_progress();
    let year_picker_alpha =
        content_alpha * year_picker_content_alpha(year_picker_progress, state.year_picker_visible);
    let calendar = Column::new()
        .push(weekdays_row(state.first_day_of_week, content_alpha))
        .push(animated_month_grid(state, on_action.clone(), content_alpha));
    let content: Element<'a, Message, Theme, Renderer> = if year_picker_progress > 0.0 {
        Stack::new()
            .push(calendar)
            .push(year_grid(
                state,
                on_action.clone(),
                year_picker_progress,
                year_picker_alpha,
            ))
            .into()
    } else {
        calendar.into()
    };

    Column::new()
        .push(month_navigation(state, on_action, content_alpha))
        .push(content)
        .into()
}

fn date_range_picker_content<'a, Message, Renderer>(
    state: &'a DateRangePickerState,
    on_action: impl Fn(DateRangePickerAction) -> Message + Clone + 'a,
    container_height: f32,
    content_alpha: f32,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    let year_picker_progress = state.animation.year_picker_progress();
    let year_picker_alpha =
        content_alpha * year_picker_content_alpha(year_picker_progress, state.year_picker_visible);
    let calendar = Column::new()
        .push(weekdays_row(state.first_day_of_week, content_alpha))
        .push(range_months_list(
            state,
            on_action.clone(),
            container_height,
            content_alpha,
        ));

    if year_picker_progress > 0.0 {
        Stack::new()
            .push(calendar)
            .push(range_year_grid(
                state,
                on_action,
                year_picker_progress,
                year_picker_alpha,
            ))
            .into()
    } else {
        calendar.into()
    }
}

fn year_picker_content_alpha(progress: f32, visible: bool) -> f32 {
    let progress = progress.clamp(0.0, 1.0);

    if visible {
        lerp(0.6, 1.0, progress)
    } else {
        progress
    }
}

fn date_input_content<'a, Message, Renderer>(
    state: &'a DatePickerState,
    on_action: impl Fn(DatePickerAction) -> Message + Clone + 'a,
    content_alpha: f32,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    let input = text_input::outlined("MM/DD/YYYY", state.input_value())
        .error(!state.is_input_valid())
        .alpha(content_alpha)
        .on_input(move |value| on_action(DatePickerAction::InputChanged(value)));
    let mut content = Column::new().push(input);

    if let Some(error) = state.input_error() {
        content = content.push(date_input_error_label(error, content_alpha));
    }

    Container::new(content)
        .width(Length::Fill)
        .padding(Padding {
            top: 10.0,
            right: 24.0,
            bottom: 0.0,
            left: 24.0,
        })
        .style(move |theme| date_input_panel_style(theme, content_alpha))
        .into()
}

fn date_range_input_content<'a, Message, Renderer>(
    state: &'a DateRangePickerState,
    on_action: impl Fn(DateRangePickerAction) -> Message + Clone + 'a,
    content_alpha: f32,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    let start = text_input::outlined("Start date", state.start_input_value())
        .error(!state.is_start_input_valid())
        .alpha(content_alpha)
        .on_input({
            let on_action = on_action.clone();
            move |value| on_action(DateRangePickerAction::StartInputChanged(value))
        })
        .width(Length::Fill);
    let end = text_input::outlined("End date", state.end_input_value())
        .error(!state.is_end_input_valid())
        .alpha(content_alpha)
        .on_input(move |value| on_action(DateRangePickerAction::EndInputChanged(value)))
        .width(Length::Fill);
    let mut start = Column::new().push(start);
    let mut end = Column::new().push(end);

    if let Some(error) = state.start_input_error() {
        start = start.push(date_input_error_label(error, content_alpha));
    }

    if let Some(error) = state.end_input_error() {
        end = end.push(date_input_error_label(error, content_alpha));
    }

    Container::new(
        Row::new()
            .push(Container::new(start).width(Length::Fill))
            .push(Container::new(end).width(Length::Fill))
            .spacing(8.0)
            .align_y(alignment::Vertical::Top),
    )
    .width(Length::Fill)
    .padding(Padding {
        top: 10.0,
        right: 24.0,
        bottom: 0.0,
        left: 24.0,
    })
    .style(move |theme| date_input_panel_style(theme, content_alpha))
    .into()
}

fn date_input_error_label<'a, Message, Renderer>(
    error: String,
    content_alpha: f32,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    let scale = tokens::typography::BODY_SMALL;

    Container::new(
        Text::new(error)
            .size(scale.size)
            .line_height(absolute_line_height(scale.line_height))
            .font(fonts::roboto_for_type_scale(scale))
            .style(move |theme: &Theme| text::Style {
                color: Some(alpha_color(theme.colors().error.color, content_alpha)),
            }),
    )
    .padding(Padding {
        top: 4.0,
        right: 16.0,
        bottom: 0.0,
        left: 16.0,
    })
    .width(Length::Fill)
    .into()
}

fn month_navigation<'a, Message, Renderer>(
    state: &'a DatePickerState,
    on_action: impl Fn(DatePickerAction) -> Message + Clone + 'a,
    content_alpha: f32,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    let month = state.displayed_month;
    let previous = month.add_months(-1);
    let next = month.add_months(1);
    let can_previous = state.year_range.contains(&previous.year);
    let can_next = state.year_range.contains(&next.year);
    let arrow = if state.year_picker_visible {
        "arrow_drop_up"
    } else {
        "arrow_drop_down"
    };
    let year_button_content = Row::<Message, Theme, Renderer>::new()
        .push(
            Text::new(state.formatter.format_month_year(month))
                .size(tokens::typography::LABEL_LARGE.size)
                .line_height(absolute_line_height(
                    tokens::typography::LABEL_LARGE.line_height,
                ))
                .font(fonts::roboto_for_type_scale(
                    tokens::typography::LABEL_LARGE,
                ))
                .style(move |theme: &Theme| text::Style {
                    color: Some(alpha_color(theme.colors().surface.text, content_alpha)),
                }),
        )
        .push(
            fonts::filled_icon(arrow, 18.0).style(move |theme: &Theme| text::Style {
                color: Some(alpha_color(theme.colors().surface.text, content_alpha)),
            }),
        )
        .spacing(8.0)
        .align_y(alignment::Vertical::Center);
    let year_button = Button::new(
        Container::new(year_button_content)
            .height(Length::Fixed(tokens::component::button::CONTAINER_HEIGHT))
            .padding(Padding::from([0.0, 16.0]))
            .align_y(alignment::Vertical::Center),
    )
    .height(Length::Fixed(tokens::component::button::CONTAINER_HEIGHT))
    .padding(Padding::ZERO)
    .style(button_style::text)
    .on_press(on_action.clone()(DatePickerAction::ToggleYearPicker));

    let mut row = Row::new()
        .push(year_button)
        .push(Space::new().width(Length::Fill))
        .align_y(alignment::Vertical::Center);

    if !state.year_picker_visible {
        row = row
            .push(
                super::button::icon_button("keyboard_arrow_left").on_press_maybe(
                    can_previous.then(|| on_action.clone()(DatePickerAction::PreviousMonth)),
                ),
            )
            .push(
                super::button::icon_button("keyboard_arrow_right")
                    .on_press_maybe(can_next.then(|| on_action(DatePickerAction::NextMonth))),
            );
    }

    Container::new(row)
        .height(Length::Fixed(
            tokens::component::date_picker::MONTH_YEAR_CONTAINER_HEIGHT,
        ))
        .padding(Padding {
            top: 0.0,
            right: tokens::component::date_picker::HORIZONTAL_SPACE,
            bottom: 0.0,
            left: tokens::component::date_picker::HORIZONTAL_SPACE,
        })
        .into()
}

fn weekdays_row<'a, Message, Renderer>(
    first_day_of_week: Weekday,
    content_alpha: f32,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    let scale = tokens::component::date_picker::WEEKDAY_LABEL_TEXT;
    let mut row = Row::new()
        .height(Length::Fixed(
            tokens::component::date_picker::WEEKDAY_CONTAINER_HEIGHT,
        ))
        .align_y(alignment::Vertical::Center);

    for offset in 0..7 {
        let weekday = Weekday::ALL[(first_day_of_week.sunday_first_index() + offset) % 7];
        row = row.push(
            Container::new(
                Text::new(weekday.short_label())
                    .size(scale.size)
                    .line_height(absolute_line_height(scale.line_height))
                    .font(fonts::roboto_for_type_scale(scale))
                    .style(move |theme: &Theme| text::Style {
                        color: Some(alpha_color(theme.colors().surface.text, content_alpha)),
                    }),
            )
            .center_x(Length::Fixed(
                tokens::component::date_picker::CALENDAR_CELL_SIZE,
            ))
            .center_y(Length::Fixed(
                tokens::component::date_picker::CALENDAR_CELL_SIZE,
            )),
        );
    }

    Container::new(row)
        .padding(Padding {
            top: 0.0,
            right: tokens::component::date_picker::HORIZONTAL_SPACE,
            bottom: 0.0,
            left: tokens::component::date_picker::HORIZONTAL_SPACE,
        })
        .into()
}

fn animated_month_grid<'a, Message, Renderer>(
    state: &'a DatePickerState,
    on_action: impl Fn(DatePickerAction) -> Message + Clone + 'a,
    content_alpha: f32,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    if !state.animation.month.is_animating() {
        return month_grid_for_month(state, on_action, state.displayed_month, content_alpha);
    }

    let (from, to) = state.animation.month.visible_months(state.displayed_month);
    let from_grid = translated(
        month_grid_for_month(state, on_action.clone(), from, content_alpha),
        Vector::new(state.animation.month.month_offset(from), 0.0),
    );
    let to_grid = translated(
        month_grid_for_month(state, on_action, to, content_alpha),
        Vector::new(state.animation.month.month_offset(to), 0.0),
    );

    Stack::new()
        .push(from_grid)
        .push(to_grid)
        .width(Length::Fill)
        .clip(true)
        .into()
}

fn month_grid_for_month<'a, Message, Renderer>(
    state: &'a DatePickerState,
    on_action: impl Fn(DatePickerAction) -> Message + Clone + 'a,
    month: YearMonth,
    content_alpha: f32,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    let first = month.start_date();
    let first_weekday = first.weekday_index_from(state.first_day_of_week);
    let days = days_in_month(month.year, month.month);
    let today = Date::today_utc();
    let mut day = 1u8;
    let mut column = Column::new();

    for week in 0..tokens::component::date_picker::MAX_CALENDAR_ROWS {
        let mut row = Row::new().align_y(alignment::Vertical::Center);

        for weekday in 0..7 {
            let cell = week * 7 + weekday;

            if cell < first_weekday || day > days {
                row = row.push(
                    Space::new()
                        .width(Length::Fixed(
                            tokens::component::date_picker::CALENDAR_CELL_SIZE,
                        ))
                        .height(Length::Fixed(
                            tokens::component::date_picker::CALENDAR_CELL_SIZE,
                        )),
                );
            } else {
                let date = Date {
                    year: month.year,
                    month: month.month,
                    day,
                };
                let selected = state.selected_date == Some(date);
                let selected_progress = state.animation.selected_date_progress(date, selected);
                let is_today = date == today;
                let enabled = state.date_is_selectable(date);
                let action = on_action.clone()(DatePickerAction::SelectDate(date));

                row = row.push(
                    day_button(
                        day,
                        selected,
                        selected_progress,
                        is_today,
                        enabled,
                        weekday,
                        DateRangePosition::None,
                        0.0,
                        content_alpha,
                    )
                    .on_press_maybe(enabled.then_some(action)),
                );
                day += 1;
            }
        }

        column = column.push(row);
    }

    Container::new(column)
        .padding(Padding {
            top: 0.0,
            right: tokens::component::date_picker::HORIZONTAL_SPACE,
            bottom: 0.0,
            left: tokens::component::date_picker::HORIZONTAL_SPACE,
        })
        .into()
}

fn year_grid<'a, Message, Renderer>(
    state: &'a DatePickerState,
    on_action: impl Fn(DatePickerAction) -> Message + Clone + 'a,
    progress: f32,
    content_alpha: f32,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    let current_year = Date::today_utc().year;
    let selected_year = state.displayed_month.year;
    let mut column = Column::new().spacing(tokens::component::date_picker::YEAR_VERTICAL_SPACE);
    let mut years = state.year_range.clone().peekable();

    while years.peek().is_some() {
        let mut row = year_grid_row::<Message, Renderer>();

        for _ in 0..tokens::component::date_picker::YEARS_IN_ROW {
            if let Some(year) = years.next() {
                let enabled = state.selectable_dates.is_selectable_year(year);
                let action = on_action.clone()(DatePickerAction::SelectYear(year));
                row = row.push(Space::new().width(Length::Fill)).push(
                    year_button(
                        year,
                        year == selected_year,
                        year == current_year,
                        enabled,
                        content_alpha,
                    )
                    .on_press_maybe(enabled.then_some(action)),
                );
            } else {
                row = row
                    .push(Space::new().width(Length::Fill))
                    .push(year_placeholder());
            }
        }

        column = column.push(row.push(Space::new().width(Length::Fill)));
    }

    let full_height = tokens::component::date_picker::CALENDAR_CELL_SIZE
        * (tokens::component::date_picker::MAX_CALENDAR_ROWS as f32 + 1.0)
        - tokens::component::divider::THICKNESS;
    let height = (full_height * progress.clamp(0.0, 1.0)).max(1.0);

    Container::new(
        Scrollable::new(column)
            .id(state.year_picker_scroll_id.clone())
            .height(Length::Fixed(full_height))
            .width(Length::Fill),
    )
    .height(Length::Fixed(height))
    .clip(true)
    .padding(Padding {
        top: 0.0,
        right: tokens::component::date_picker::HORIZONTAL_SPACE,
        bottom: 0.0,
        left: tokens::component::date_picker::HORIZONTAL_SPACE,
    })
    .style(move |theme| year_picker_panel_style(theme, content_alpha))
    .into()
}

fn year_grid_row<'a, Message, Renderer>() -> Row<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + 'a,
{
    Row::new()
        .width(Length::Fill)
        .spacing(0.0)
        .align_y(alignment::Vertical::Center)
}

fn year_placeholder() -> Space {
    Space::new()
        .width(Length::Fixed(
            tokens::component::date_picker::YEAR_CONTAINER_WIDTH,
        ))
        .height(Length::Fixed(
            tokens::component::date_picker::YEAR_CONTAINER_HEIGHT,
        ))
}

fn range_months_list<'a, Message, Renderer>(
    state: &'a DateRangePickerState,
    on_action: impl Fn(DateRangePickerAction) -> Message + Clone + 'a,
    container_height: f32,
    content_alpha: f32,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    let months = range_rendered_months(state);
    let mut column = Column::new();

    for month in months {
        column = column
            .push(range_month_subhead::<Message, Renderer>(
                &state.formatter,
                month,
                content_alpha,
            ))
            .push(range_month_grid_for_month(
                state,
                on_action.clone(),
                month,
                content_alpha,
            ));
    }

    let height = container_height
        - tokens::component::date_picker::RANGE_HEADER_CONTAINER_HEIGHT
        - tokens::component::date_picker::WEEKDAY_CONTAINER_HEIGHT;

    Container::new(
        Scrollable::new(column)
            .id(state.months_scroll_id.clone())
            .height(Length::Fixed(height))
            .width(Length::Fill),
    )
    .height(Length::Fixed(height))
    .into()
}

fn range_month_subhead<'a, Message, Renderer>(
    formatter: &DatePickerFormatter,
    month: YearMonth,
    content_alpha: f32,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    let scale = tokens::component::date_picker::RANGE_MONTH_SUBHEAD_TEXT;

    Container::new(
        Text::new(formatter.format_month_year(month))
            .size(scale.size)
            .line_height(absolute_line_height(scale.line_height))
            .font(fonts::roboto_for_type_scale(scale))
            .style(move |theme: &Theme| text::Style {
                color: Some(alpha_color(
                    theme.colors().surface.text_variant,
                    content_alpha,
                )),
            }),
    )
    .padding(Padding {
        top: tokens::component::date_picker::RANGE_MONTH_SUBHEAD_TOP_SPACE,
        right: 0.0,
        bottom: tokens::component::date_picker::RANGE_MONTH_SUBHEAD_BOTTOM_SPACE,
        left: tokens::component::date_picker::RANGE_MONTH_SUBHEAD_START_SPACE,
    })
    .width(Length::Fill)
    .into()
}

fn range_month_selection_info(
    state: &DateRangePickerState,
    month: YearMonth,
) -> Option<RangeMonthSelectionInfo> {
    let start = state.selected_start_date?;
    let end = state.selected_end_date?;
    let month_start = month.start_date();
    let month_days = days_in_month(month.year, month.month);
    let month_end = Date {
        year: month.year,
        month: month.month,
        day: month_days,
    };

    if start > month_end || end < month_start {
        return None;
    }

    let first_weekday = month_start.weekday_index_from(state.first_day_of_week);
    let first_is_selection_start = start >= month_start;
    let last_is_selection_end = end <= month_end;
    let start_day = if first_is_selection_start {
        start.day
    } else {
        1
    };
    let end_day = if last_is_selection_end {
        end.day
    } else {
        month_days
    };
    let start_cell = first_weekday + usize::from(start_day - 1);
    let end_cell = first_weekday + usize::from(end_day - 1);

    Some(RangeMonthSelectionInfo {
        start_column: start_cell % 7,
        start_row: start_cell / 7,
        end_column: end_cell % 7,
        end_row: end_cell / 7,
        first_is_selection_start,
        last_is_selection_end,
    })
}

fn range_background_rects(
    info: RangeMonthSelectionInfo,
    size_width: f32,
) -> Vec<RangeBackgroundRect> {
    let item_container_width = tokens::component::date_picker::CALENDAR_CELL_SIZE;
    let item_container_height = tokens::component::date_picker::CALENDAR_CELL_SIZE;
    let item_state_layer_height = tokens::component::date_picker::DATE_CONTAINER_HEIGHT;
    let state_layer_vertical_padding = (item_container_height - item_state_layer_height) / 2.0;
    let horizontal_space_between_items = (size_width - 7.0 * item_container_width) / 7.0;
    let item_step = item_container_width + horizontal_space_between_items;
    let start_x = info.start_column as f32 * item_step
        + if info.first_is_selection_start {
            item_container_width / 2.0
        } else {
            0.0
        }
        + horizontal_space_between_items / 2.0;
    let start_y = info.start_row as f32 * item_container_height + state_layer_vertical_padding;
    let end_x = info.end_column as f32 * item_step
        + if info.last_is_selection_end {
            item_container_width / 2.0
        } else {
            item_container_width
        }
        + horizontal_space_between_items / 2.0;
    let end_y = info.end_row as f32 * item_container_height + state_layer_vertical_padding;
    let mut rects = Vec::with_capacity(3);

    rects.push(RangeBackgroundRect {
        x: start_x,
        y: start_y,
        width: if info.start_row == info.end_row {
            end_x - start_x
        } else {
            size_width - start_x
        },
        height: item_state_layer_height,
    });

    if info.start_row != info.end_row {
        for row_offset in (1..=(info.end_row - info.start_row - 1)).rev() {
            rects.push(RangeBackgroundRect {
                x: 0.0,
                y: start_y + row_offset as f32 * item_container_height,
                width: size_width,
                height: item_state_layer_height,
            });
        }

        rects.push(RangeBackgroundRect {
            x: 0.0,
            y: end_y,
            width: end_x,
            height: item_state_layer_height,
        });
    }

    rects
}

fn range_background_rects_with_progress(
    info: RangeMonthSelectionInfo,
    size_width: f32,
    progress: f32,
) -> Vec<RangeBackgroundRect> {
    let progress = progress.clamp(0.0, 1.0);
    let rects = range_background_rects(info, size_width);
    let total_width: f32 = rects.iter().map(|rect| rect.width.max(0.0)).sum();
    let mut remaining = total_width * progress;
    let mut clipped = Vec::with_capacity(rects.len());

    for mut rect in rects {
        if remaining <= 0.0 {
            break;
        }

        let width = rect.width.min(remaining);

        if width > f32::EPSILON {
            rect.width = width;
            clipped.push(rect);
        }

        remaining -= width;
    }

    clipped
}

fn range_endpoint_connector_rect(
    range_position: DateRangePosition,
    weekday: usize,
) -> Option<RangeBackgroundRect> {
    let cell_size = tokens::component::date_picker::CALENDAR_CELL_SIZE;
    let layer_height = tokens::component::date_picker::DATE_STATE_LAYER_HEIGHT;
    let y = (cell_size - layer_height) / 2.0;
    let half_width = cell_size / 2.0;

    match range_position {
        DateRangePosition::Start if weekday < 6 => Some(RangeBackgroundRect {
            x: half_width,
            y,
            width: half_width,
            height: layer_height,
        }),
        DateRangePosition::End if weekday > 0 => Some(RangeBackgroundRect {
            x: 0.0,
            y,
            width: half_width,
            height: layer_height,
        }),
        DateRangePosition::None
        | DateRangePosition::Single
        | DateRangePosition::Middle
        | DateRangePosition::Start
        | DateRangePosition::End => None,
    }
}

fn range_background_corner_radius(rect: RangeBackgroundRect) -> f32 {
    rect.height / 2.0
}

fn range_endpoint_connector_progress(
    range_background_progress: f32,
    selected_progress: f32,
) -> f32 {
    let range_background_progress = range_background_progress.clamp(0.0, 1.0);

    if selected_progress >= 1.0 {
        range_background_progress
    } else {
        0.0
    }
}

fn range_month_grid_for_month<'a, Message, Renderer>(
    state: &'a DateRangePickerState,
    on_action: impl Fn(DateRangePickerAction) -> Message + Clone + 'a,
    month: YearMonth,
    content_alpha: f32,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    let first = month.start_date();
    let first_weekday = first.weekday_index_from(state.first_day_of_week);
    let days = days_in_month(month.year, month.month);
    let today = Date::today_utc();
    let mut day = 1u8;
    let mut column = Column::new();
    let grid_width = tokens::component::date_picker::CALENDAR_CELL_SIZE * 7.0;
    let grid_height = tokens::component::date_picker::CALENDAR_CELL_SIZE
        * tokens::component::date_picker::MAX_CALENDAR_ROWS as f32;

    for week in 0..tokens::component::date_picker::MAX_CALENDAR_ROWS {
        let mut row = Row::new().align_y(alignment::Vertical::Center);

        for weekday in 0..7 {
            let cell = week * 7 + weekday;

            if cell < first_weekday || day > days {
                row = row.push(
                    Space::new()
                        .width(Length::Fixed(
                            tokens::component::date_picker::CALENDAR_CELL_SIZE,
                        ))
                        .height(Length::Fixed(
                            tokens::component::date_picker::CALENDAR_CELL_SIZE,
                        )),
                );
            } else {
                let date = Date {
                    year: month.year,
                    month: month.month,
                    day,
                };
                let selected = state.selected_start_date == Some(date)
                    || state.selected_end_date == Some(date);
                let selected_progress = state.animation.selected_date_progress(date, selected);
                let range_position = state.date_range_position(date);
                let range_background_progress =
                    state.animation.range_background_progress(range_position);
                let is_today = date == today;
                let enabled = state.date_is_selectable(date);
                let action = on_action.clone()(DateRangePickerAction::SelectDate(date));

                row = row.push(
                    day_button(
                        day,
                        selected,
                        selected_progress,
                        is_today,
                        enabled,
                        weekday,
                        range_position,
                        range_background_progress,
                        content_alpha,
                    )
                    .on_press_maybe(enabled.then_some(action)),
                );
                day += 1;
            }
        }

        column = column.push(row);
    }

    let mut grid = Stack::new()
        .width(Length::Fixed(grid_width))
        .height(Length::Fixed(grid_height));

    if let Some(info) = range_month_selection_info(state, month) {
        grid = grid.push(
            Canvas::new(RangeMonthBackground {
                info,
                progress: state
                    .animation
                    .range_background_progress(DateRangePosition::Middle),
                content_alpha,
            })
            .width(Length::Fixed(grid_width))
            .height(Length::Fixed(grid_height)),
        );
    }

    grid = grid.push(
        Container::new(column)
            .width(Length::Fixed(grid_width))
            .height(Length::Fixed(grid_height)),
    );

    Container::new(grid)
        .padding(Padding {
            top: 0.0,
            right: tokens::component::date_picker::HORIZONTAL_SPACE,
            bottom: 0.0,
            left: tokens::component::date_picker::HORIZONTAL_SPACE,
        })
        .into()
}

fn range_year_grid<'a, Message, Renderer>(
    state: &'a DateRangePickerState,
    on_action: impl Fn(DateRangePickerAction) -> Message + Clone + 'a,
    progress: f32,
    content_alpha: f32,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    let current_year = Date::today_utc().year;
    let selected_year = state.displayed_month.year;
    let mut column = Column::new().spacing(tokens::component::date_picker::YEAR_VERTICAL_SPACE);
    let mut years = state.year_range.clone().peekable();

    while years.peek().is_some() {
        let mut row = year_grid_row::<Message, Renderer>();

        for _ in 0..tokens::component::date_picker::YEARS_IN_ROW {
            if let Some(year) = years.next() {
                let enabled = state.selectable_dates.is_selectable_year(year);
                let action = on_action.clone()(DateRangePickerAction::SelectYear(year));
                row = row.push(Space::new().width(Length::Fill)).push(
                    year_button(
                        year,
                        year == selected_year,
                        year == current_year,
                        enabled,
                        content_alpha,
                    )
                    .on_press_maybe(enabled.then_some(action)),
                );
            } else {
                row = row
                    .push(Space::new().width(Length::Fill))
                    .push(year_placeholder());
            }
        }

        column = column.push(row.push(Space::new().width(Length::Fill)));
    }

    let full_height = tokens::component::date_picker::CALENDAR_CELL_SIZE
        * (tokens::component::date_picker::MAX_CALENDAR_ROWS as f32 + 1.0)
        - tokens::component::divider::THICKNESS;
    let height = (full_height * progress.clamp(0.0, 1.0)).max(1.0);

    Container::new(
        Scrollable::new(column)
            .id(state.year_picker_scroll_id.clone())
            .height(Length::Fixed(full_height))
            .width(Length::Fill),
    )
    .height(Length::Fixed(height))
    .clip(true)
    .padding(Padding {
        top: 0.0,
        right: tokens::component::date_picker::HORIZONTAL_SPACE,
        bottom: 0.0,
        left: tokens::component::date_picker::HORIZONTAL_SPACE,
    })
    .style(move |theme| year_picker_panel_style(theme, content_alpha))
    .into()
}

fn day_button<'a, Message, Renderer>(
    day: u8,
    selected: bool,
    selected_progress: f32,
    today: bool,
    enabled: bool,
    weekday: usize,
    range_position: DateRangePosition,
    range_background_progress: f32,
    content_alpha: f32,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    let label = Canvas::new(DayCell {
        day,
        selected,
        selected_progress,
        today,
        enabled,
        weekday,
        range_position,
        range_background_progress,
        content_alpha,
    })
    .width(Length::Fixed(
        tokens::component::date_picker::CALENDAR_CELL_SIZE,
    ))
    .height(Length::Fixed(
        tokens::component::date_picker::CALENDAR_CELL_SIZE,
    ));

    Button::new(label)
        .width(Length::Fixed(
            tokens::component::date_picker::CALENDAR_CELL_SIZE,
        ))
        .height(Length::Fixed(
            tokens::component::date_picker::CALENDAR_CELL_SIZE,
        ))
        .padding(Padding::ZERO)
        .style(move |theme, status| {
            day_button_style(
                theme,
                status,
                selected,
                selected_progress,
                enabled,
                range_position,
                content_alpha,
            )
        })
}

#[derive(Debug, Clone, Copy)]
struct RangeMonthBackground {
    info: RangeMonthSelectionInfo,
    progress: f32,
    content_alpha: f32,
}

impl<Message, Renderer> canvas::Program<Message, Theme, Renderer> for RangeMonthBackground
where
    Renderer: geometry::Renderer,
{
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry<Renderer>> {
        let mut frame = Frame::new(renderer, bounds.size());
        let progress = self.progress.clamp(0.0, 1.0);

        if progress > 0.0 {
            let color = alpha_color(
                theme.colors().secondary.container,
                progress * self.content_alpha,
            );
            let rects = range_background_rects_with_progress(self.info, frame.width(), progress);

            for rect in rects {
                let size = Size::new(rect.width, rect.height);
                let top_left = Point::new(rect.x, rect.y);
                let path = Path::rounded_rectangle(
                    top_left,
                    size,
                    border::Radius::from(range_background_corner_radius(rect)),
                );

                frame.fill(&path, color);
            }
        }

        vec![frame.into_geometry()]
    }
}

#[derive(Debug, Clone, Copy)]
struct DayCell {
    day: u8,
    selected: bool,
    selected_progress: f32,
    today: bool,
    enabled: bool,
    weekday: usize,
    range_position: DateRangePosition,
    range_background_progress: f32,
    content_alpha: f32,
}

impl<Message, Renderer> canvas::Program<Message, Theme, Renderer> for DayCell
where
    Renderer: geometry::Renderer,
{
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry<Renderer>> {
        let colors = theme.colors();
        let mut frame = Frame::new(renderer, bounds.size());
        let center = frame.center();
        let indicator_width = tokens::component::date_picker::DATE_CONTAINER_WIDTH;
        let indicator_height = tokens::component::date_picker::DATE_CONTAINER_HEIGHT;
        let selected_progress = if self.selected {
            self.selected_progress.clamp(0.0, 1.0)
        } else {
            0.0
        };
        let connector_progress =
            range_endpoint_connector_progress(self.range_background_progress, selected_progress);

        if connector_progress > 0.0 {
            if let Some(rect) = range_endpoint_connector_rect(self.range_position, self.weekday) {
                let path = Path::rounded_rectangle(
                    Point::new(rect.x, rect.y),
                    Size::new(rect.width, rect.height),
                    border::Radius::from(range_background_corner_radius(rect)),
                );

                frame.fill(
                    &path,
                    alpha_color(
                        colors.secondary.container,
                        connector_progress * self.content_alpha,
                    ),
                );
            }
        }

        let indicator_radius =
            indicator_width.min(indicator_height) / 2.0 * lerp(0.78, 1.0, selected_progress);
        let indicator = Path::circle(center, indicator_radius);

        if selected_progress > 0.0 {
            let selected_container = if self.enabled {
                colors.primary.color
            } else {
                disabled_container(colors.surface.text)
            };
            frame.fill(
                &indicator,
                alpha_color(selected_container, selected_progress * self.content_alpha),
            );
        }

        if self.today && selected_progress < 1.0 {
            frame.stroke(
                &indicator,
                Stroke::default()
                    .with_width(tokens::component::date_picker::DATE_TODAY_OUTLINE_WIDTH)
                    .with_color(alpha_color(
                        colors.primary.color,
                        (1.0 - selected_progress) * self.content_alpha,
                    )),
            );
        }

        let scale = tokens::component::date_picker::DATE_LABEL_TEXT;
        let unselected_text_color = if self.range_position.is_middle() {
            colors.secondary.container_text
        } else if self.today {
            colors.primary.color
        } else {
            colors.surface.text
        };
        let text_color = if !self.enabled {
            disabled_text(colors.surface.text)
        } else if selected_progress > 0.0 {
            mix(
                unselected_text_color,
                colors.primary.text,
                selected_progress,
            )
        } else {
            unselected_text_color
        };

        frame.fill_text(CanvasText {
            content: self.day.to_string(),
            position: center,
            max_width: indicator_width,
            color: alpha_color(text_color, self.content_alpha),
            size: scale.size.into(),
            line_height: absolute_line_height(scale.line_height),
            font: fonts::roboto_for_type_scale(scale),
            align_x: core_text::Alignment::Center,
            align_y: alignment::Vertical::Center,
            shaping: text::Shaping::Advanced,
        });

        vec![frame.into_geometry()]
    }
}

fn year_button<'a, Message, Renderer>(
    year: i32,
    selected: bool,
    current_year: bool,
    enabled: bool,
    content_alpha: f32,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    let scale = tokens::component::date_picker::YEAR_LABEL_TEXT;
    let label = Container::new(
        Text::new(year.to_string())
            .size(scale.size)
            .line_height(absolute_line_height(scale.line_height))
            .font(fonts::roboto_for_type_scale(scale)),
    )
    .center_x(Length::Fixed(
        tokens::component::date_picker::YEAR_CONTAINER_WIDTH,
    ))
    .center_y(Length::Fixed(
        tokens::component::date_picker::YEAR_CONTAINER_HEIGHT,
    ));

    Button::new(label)
        .width(Length::Fixed(
            tokens::component::date_picker::YEAR_CONTAINER_WIDTH,
        ))
        .height(Length::Fixed(
            tokens::component::date_picker::YEAR_CONTAINER_HEIGHT,
        ))
        .padding(Padding::ZERO)
        .style(move |theme, status| {
            year_button_style(
                theme,
                status,
                selected,
                current_year,
                enabled,
                content_alpha,
            )
        })
}

fn time_input_field<'a, Message, Renderer>(
    state: &'a TimePickerState,
    on_action: impl Fn(TimePickerAction) -> Message + Clone + 'a,
    selection: TimePickerSelectionMode,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    let selected = state.selection == selection;
    let valid = match selection {
        TimePickerSelectionMode::Hour => state.is_hour_input_valid(),
        TimePickerSelectionMode::Minute => state.is_minute_input_valid(),
    };
    let value = time_input_value(state, selection);

    let field: Element<'a, Message, Theme, Renderer> = if selected {
        let input = text_input::outlined_placeholder(time_input_label(selection), &value)
            .error(!valid)
            .on_input({
                let on_action = on_action.clone();
                move |value| match selection {
                    TimePickerSelectionMode::Hour => {
                        on_action(TimePickerAction::HourInputChanged(value))
                    }
                    TimePickerSelectionMode::Minute => {
                        on_action(TimePickerAction::MinuteInputChanged(value))
                    }
                }
            })
            .width(Length::Fixed(
                tokens::component::time_input::TIME_FIELD_CONTAINER_WIDTH,
            ));

        Container::new(input)
            .width(Length::Fixed(
                tokens::component::time_input::TIME_FIELD_CONTAINER_WIDTH,
            ))
            .center_y(Length::Fixed(
                tokens::component::time_input::TIME_FIELD_CONTAINER_HEIGHT,
            ))
            .into()
    } else {
        time_input_selector(value, valid)
            .on_press(on_action(TimePickerAction::SetSelection(selection)))
            .into()
    };

    Column::new()
        .push(field)
        .push(time_input_supporting_text(state, selection, valid))
        .width(Length::Fixed(
            tokens::component::time_input::TIME_FIELD_CONTAINER_WIDTH,
        ))
        .into()
}

fn time_input_selector<'a, Message, Renderer>(
    label: String,
    valid: bool,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    let scale = tokens::component::time_input::TIME_FIELD_LABEL_TEXT;
    let label = Container::new(
        Text::new(label)
            .size(scale.size)
            .line_height(LineHeight::Absolute(scale.size.into()))
            .font(fonts::roboto_for_type_scale(scale)),
    )
    .center_x(Length::Fixed(
        tokens::component::time_input::TIME_FIELD_CONTAINER_WIDTH,
    ))
    .center_y(Length::Fixed(
        tokens::component::time_input::TIME_FIELD_CONTAINER_HEIGHT,
    ));

    Button::new(label)
        .width(Length::Fixed(
            tokens::component::time_input::TIME_FIELD_CONTAINER_WIDTH,
        ))
        .height(Length::Fixed(
            tokens::component::time_input::TIME_FIELD_CONTAINER_HEIGHT,
        ))
        .padding(Padding::ZERO)
        .style(move |theme, status| time_input_selector_style(theme, status, valid))
}

fn time_input_supporting_text<'a, Message, Renderer>(
    state: &TimePickerState,
    selection: TimePickerSelectionMode,
    valid: bool,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    let scale = tokens::component::time_input::TIME_FIELD_SUPPORTING_TEXT;
    let color_is_error = !valid;

    Container::new(
        Text::new(time_input_supporting_label(state, selection, valid))
            .size(scale.size)
            .line_height(absolute_line_height(scale.line_height))
            .font(fonts::roboto_for_type_scale(scale))
            .style(move |theme: &Theme| {
                let colors = theme.colors();

                text::Style {
                    color: Some(if color_is_error {
                        colors.error.color
                    } else {
                        colors.surface.text_variant
                    }),
                }
            }),
    )
    .padding(Padding {
        top: tokens::component::time_input::TIME_FIELD_SUPPORTING_TEXT_TOP_SPACE,
        right: 0.0,
        bottom: 0.0,
        left: 0.0,
    })
    .height(Length::Fixed(
        tokens::component::time_input::TIME_FIELD_SUPPORTING_TEXT_TOP_SPACE
            + tokens::component::time_input::TIME_FIELD_SUPPORTING_TEXT.line_height
                * tokens::component::time_input::TIME_FIELD_SUPPORTING_TEXT_LINES,
    ))
    .width(Length::Fill)
    .into()
}

fn clock_display<'a, Message, Renderer>(
    state: &'a TimePickerState,
    on_action: impl Fn(TimePickerAction) -> Message + Clone + 'a,
    vertical_period: bool,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    let hour = time_selector(
        two_digit(state.hour_for_display()),
        state
            .animation
            .selection_progress(TimePickerSelectionMode::Hour),
    )
    .on_press(on_action.clone()(TimePickerAction::SetSelection(
        TimePickerSelectionMode::Hour,
    )));
    let minute = time_selector(
        two_digit(state.minute),
        state
            .animation
            .selection_progress(TimePickerSelectionMode::Minute),
    )
    .on_press(on_action.clone()(TimePickerAction::SetSelection(
        TimePickerSelectionMode::Minute,
    )));
    let display = Row::new()
        .push(hour)
        .push(display_separator())
        .push(minute)
        .spacing(0.0)
        .align_y(alignment::Vertical::Center);

    if state.is_24_hour {
        return display.into();
    }

    if vertical_period {
        return display
            .push(Space::new().width(Length::Fixed(
                tokens::component::time_picker::PERIOD_TOGGLE_MARGIN,
            )))
            .push(period_toggle(state, on_action, true))
            .into();
    }

    Column::new()
        .push(display)
        .push(Space::new().height(Length::Fixed(
            tokens::component::time_picker::PERIOD_TOGGLE_MARGIN,
        )))
        .push(period_toggle(state, on_action, false))
        .align_x(alignment::Horizontal::Center)
        .into()
}

fn display_separator<'a, Message, Renderer>() -> Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    display_separator_with_height(tokens::component::time_picker::TIME_SELECTOR_HEIGHT)
}

fn display_separator_with_height<'a, Message, Renderer>(
    height: f32,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    display_separator_with_size(
        tokens::component::time_picker::DISPLAY_SEPARATOR_WIDTH,
        height,
    )
}

fn display_separator_with_size<'a, Message, Renderer>(
    width: f32,
    height: f32,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    let scale = tokens::component::time_picker::TIME_SELECTOR_LABEL_TEXT;

    Container::new(
        Text::new(":")
            .size(scale.size)
            .line_height(LineHeight::Absolute(scale.size.into()))
            .font(fonts::roboto_for_type_scale(scale)),
    )
    .center_x(Length::Fixed(width))
    .center_y(Length::Fixed(height))
    .into()
}

fn time_selector<'a, Message, Renderer>(
    label: String,
    selected_progress: f32,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    let scale = tokens::component::time_picker::TIME_SELECTOR_LABEL_TEXT;
    let label = Container::new(
        Text::new(label)
            .size(scale.size)
            .line_height(LineHeight::Absolute(scale.size.into()))
            .font(fonts::roboto_for_type_scale(scale)),
    )
    .center_x(Length::Fixed(
        tokens::component::time_picker::TIME_SELECTOR_WIDTH,
    ))
    .center_y(Length::Fixed(
        tokens::component::time_picker::TIME_SELECTOR_HEIGHT,
    ));

    Button::new(label)
        .width(Length::Fixed(
            tokens::component::time_picker::TIME_SELECTOR_WIDTH,
        ))
        .height(Length::Fixed(
            tokens::component::time_picker::TIME_SELECTOR_HEIGHT,
        ))
        .padding(Padding::ZERO)
        .style(move |theme, status| time_selector_style(theme, status, selected_progress))
}

fn period_toggle<'a, Message, Renderer>(
    state: &'a TimePickerState,
    on_action: impl Fn(TimePickerAction) -> Message + Clone + 'a,
    vertical: bool,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    let width = if vertical {
        tokens::component::time_picker::PERIOD_SELECTOR_VERTICAL_WIDTH
    } else {
        tokens::component::time_picker::PERIOD_SELECTOR_HORIZONTAL_WIDTH
    };
    let height = if vertical {
        tokens::component::time_picker::PERIOD_SELECTOR_VERTICAL_HEIGHT
    } else {
        tokens::component::time_picker::PERIOD_SELECTOR_HORIZONTAL_HEIGHT
    };

    period_toggle_sized(state, on_action, vertical, width, height)
}

fn period_toggle_sized<'a, Message, Renderer>(
    state: &'a TimePickerState,
    on_action: impl Fn(TimePickerAction) -> Message + Clone + 'a,
    vertical: bool,
    width: f32,
    height: f32,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    let (button_width, button_height) = period_toggle_item_size(vertical, width, height);
    let am = period_button(
        "AM",
        state.animation.period_progress(Period::Am),
        period_toggle_item_radius(vertical, true),
        button_width,
        button_height,
    )
    .on_press(on_action.clone()(TimePickerAction::SetPeriod(Period::Am)));
    let pm = period_button(
        "PM",
        state.animation.period_progress(Period::Pm),
        period_toggle_item_radius(vertical, false),
        button_width,
        button_height,
    )
    .on_press(on_action(TimePickerAction::SetPeriod(Period::Pm)));

    let items: Element<'a, Message, Theme, Renderer> = if vertical {
        Column::new().spacing(0.0).push(am).push(pm).into()
    } else {
        Row::new().spacing(0.0).push(am).push(pm).into()
    };

    Container::new(
        Stack::new()
            .push(items)
            .push(period_toggle_separator::<Message, Renderer>(
                vertical, width, height,
            ))
            .width(Length::Fixed(width))
            .height(Length::Fixed(height))
            .clip(true),
    )
    .width(Length::Fixed(width))
    .height(Length::Fixed(height))
    .style(period_toggle_container_style)
    .into()
}

fn period_toggle_item_size(vertical: bool, width: f32, height: f32) -> (f32, f32) {
    if vertical {
        (width, height / 2.0)
    } else {
        (width / 2.0, height)
    }
}

fn period_toggle_item_radius(vertical: bool, first: bool) -> border::Radius {
    let radius = tokens::component::time_picker::PERIOD_SELECTOR_SHAPE;

    match (vertical, first) {
        (true, true) => border::Radius {
            top_left: radius,
            top_right: radius,
            bottom_right: 0.0,
            bottom_left: 0.0,
        },
        (true, false) => border::Radius {
            top_left: 0.0,
            top_right: 0.0,
            bottom_right: radius,
            bottom_left: radius,
        },
        (false, true) => border::Radius {
            top_left: radius,
            top_right: 0.0,
            bottom_right: 0.0,
            bottom_left: radius,
        },
        (false, false) => border::Radius {
            top_left: 0.0,
            top_right: radius,
            bottom_right: radius,
            bottom_left: 0.0,
        },
    }
}

fn period_toggle_separator<'a, Message, Renderer>(
    vertical: bool,
    width: f32,
    height: f32,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + 'a,
{
    let line = if vertical {
        Container::new(Space::new().width(Length::Fill).height(Length::Fixed(
            tokens::component::time_picker::PERIOD_SELECTOR_OUTLINE_WIDTH,
        )))
        .width(Length::Fixed(width))
        .height(Length::Fixed(
            tokens::component::time_picker::PERIOD_SELECTOR_OUTLINE_WIDTH,
        ))
    } else {
        Container::new(Space::new().width(Length::Fixed(
            tokens::component::time_picker::PERIOD_SELECTOR_OUTLINE_WIDTH,
        )))
        .width(Length::Fixed(
            tokens::component::time_picker::PERIOD_SELECTOR_OUTLINE_WIDTH,
        ))
        .height(Length::Fixed(height))
    }
    .style(period_toggle_separator_style);

    if vertical {
        Container::new(line)
            .width(Length::Fixed(width))
            .height(Length::Fixed(height))
            .center_y(Length::Fixed(height))
            .into()
    } else {
        Container::new(line)
            .width(Length::Fixed(width))
            .height(Length::Fixed(height))
            .center_x(Length::Fixed(width))
            .into()
    }
}

fn period_button<'a, Message, Renderer>(
    label: &'static str,
    selected_progress: f32,
    radius: border::Radius,
    width: f32,
    height: f32,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    let scale = tokens::component::time_picker::PERIOD_SELECTOR_LABEL_TEXT;
    let label = Container::new(
        Text::new(label)
            .size(scale.size)
            .line_height(absolute_line_height(scale.line_height))
            .font(fonts::roboto_for_type_scale(scale)),
    )
    .center_x(Length::Fixed(width))
    .center_y(Length::Fixed(height));

    Button::new(label)
        .width(Length::Fixed(width))
        .height(Length::Fixed(height))
        .padding(Padding::ZERO)
        .style(move |theme, status| period_button_style(theme, status, selected_progress, radius))
}

fn clock_face<'a, Message, Renderer>(
    state: &'a TimePickerState,
    on_action: impl Fn(TimePickerAction) -> Message + Clone + 'a,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + 'static + 'a,
{
    Canvas::new(ClockFace {
        hour: state.hour,
        minute: state.minute,
        is_24_hour: state.is_24_hour,
        selection: state.selection,
        previous_selection: state.animation.previous_selection,
        selected_selection: state.animation.selected_selection,
        selection_progress: state.animation.selection.value.clamp(0.0, 1.0),
        auto_switch_to_minute: state.auto_switch_to_minute,
        selector_angle: state.animation.clock_angle(),
        on_action: Arc::new(on_action),
    })
    .width(Length::Fixed(
        tokens::component::time_picker::CLOCK_DIAL_SIZE,
    ))
    .height(Length::Fixed(
        tokens::component::time_picker::CLOCK_DIAL_SIZE,
    ))
    .into()
}

#[derive(Clone)]
struct ClockFace<F> {
    hour: u8,
    minute: u8,
    is_24_hour: bool,
    selection: TimePickerSelectionMode,
    previous_selection: TimePickerSelectionMode,
    selected_selection: TimePickerSelectionMode,
    selection_progress: f32,
    auto_switch_to_minute: bool,
    selector_angle: f32,
    on_action: Arc<F>,
}

impl<Message, F, Renderer> canvas::Program<Message, Theme, Renderer> for ClockFace<F>
where
    Message: Clone,
    F: Fn(TimePickerAction) -> Message,
    Renderer: geometry::Renderer + 'static,
{
    type State = ClockFaceState<Renderer>;

    fn update(
        &self,
        state: &mut Self::State,
        event: &canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        match event {
            event::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let position = event_position(event, bounds, cursor)?;
                if !local_point_is_in_bounds(position, bounds.size()) {
                    return None;
                }
                state.drag = Some(ClockFaceDrag::new(ClockFacePointer::Mouse, position));
                let action = self.action_at(position, bounds.size());
                Some(canvas::Action::publish((self.on_action)(action)).and_capture())
            }
            event::Event::Touch(touch::Event::FingerPressed { id, .. }) => {
                if state.drag.is_some() {
                    return None;
                }

                let position = event_position(event, bounds, cursor)?;
                if !local_point_is_in_bounds(position, bounds.size()) {
                    return None;
                }
                state.drag = Some(ClockFaceDrag::new(ClockFacePointer::Touch(*id), position));
                let action = self.action_at(position, bounds.size());
                Some(canvas::Action::publish((self.on_action)(action)).and_capture())
            }
            event::Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if !state
                    .drag
                    .is_some_and(|drag| drag.pointer == ClockFacePointer::Mouse)
                {
                    return None;
                }

                let position = event_position(event, bounds, cursor)?;
                if let Some(drag) = &mut state.drag {
                    drag.update(position);
                }
                let action = self.drag_action_at(position, bounds.size());
                Some(canvas::Action::publish((self.on_action)(action)).and_capture())
            }
            event::Event::Touch(touch::Event::FingerMoved { id, .. }) => {
                if !state
                    .drag
                    .is_some_and(|drag| drag.pointer == ClockFacePointer::Touch(*id))
                {
                    return None;
                }

                let position = event_position(event, bounds, cursor)?;
                if let Some(drag) = &mut state.drag {
                    drag.update(position);
                }
                let action = self.drag_action_at(position, bounds.size());
                Some(canvas::Action::publish((self.on_action)(action)).and_capture())
            }
            event::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                self.finish_drag(state, ClockFacePointer::Mouse)
            }
            event::Event::Touch(
                touch::Event::FingerLifted { id, .. } | touch::Event::FingerLost { id, .. },
            ) => self.finish_drag(state, ClockFacePointer::Touch(*id)),
            _ => None,
        }
    }

    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry<Renderer>> {
        let render_key = self.render_key(theme);

        if state.render_key.get() != Some(render_key) {
            state.cache.clear();
            state.render_key.set(Some(render_key));
        }

        let geometry = state.cache.draw(renderer, bounds.size(), |frame| {
            let colors = theme.colors();
            let center = frame.center();
            let radius = frame.width().min(frame.height()) / 2.0;
            let dial = Path::circle(center, radius);

            frame.fill(&dial, colors.surface.container.highest);
            self.draw_labels(frame, theme, center, radius, ClockLabelPass::Background);
            self.draw_selector(frame, theme, center, radius);
            self.draw_labels(
                frame,
                theme,
                center,
                radius,
                ClockLabelPass::SelectedForeground,
            );
        });

        vec![geometry]
    }

    fn mouse_interaction(
        &self,
        _state: &Self::State,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if cursor.is_over(bounds) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }
}

struct ClockFaceState<Renderer = iced_widget::Renderer>
where
    Renderer: geometry::Renderer,
{
    drag: Option<ClockFaceDrag>,
    cache: canvas::Cache<Renderer>,
    render_key: Cell<Option<ClockFaceRenderKey>>,
}

impl<Renderer> Default for ClockFaceState<Renderer>
where
    Renderer: geometry::Renderer,
{
    fn default() -> Self {
        Self {
            drag: None,
            cache: canvas::Cache::new(),
            render_key: Cell::new(None),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct ClockFaceRenderKey {
    hour: u8,
    minute: u8,
    is_24_hour: bool,
    selection: TimePickerSelectionMode,
    previous_selection: TimePickerSelectionMode,
    selected_selection: TimePickerSelectionMode,
    selection_progress: f32,
    selector_angle: f32,
    dial_color: Color,
    selector_color: Color,
    label_text_color: Color,
    selected_text_color: Color,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ClockFacePointer {
    Mouse,
    Touch(touch::Finger),
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct ClockFaceDrag {
    pointer: ClockFacePointer,
    current: Point,
}

impl ClockFaceDrag {
    fn new(pointer: ClockFacePointer, start: Point) -> Self {
        Self {
            pointer,
            current: start,
        }
    }

    fn update(&mut self, position: Point) {
        self.current = position;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ClockLabelPass {
    Background,
    SelectedForeground,
}

impl<F> ClockFace<F>
where
    F: Sized,
{
    fn finish_drag<Message, Renderer>(
        &self,
        state: &mut ClockFaceState<Renderer>,
        pointer: ClockFacePointer,
    ) -> Option<canvas::Action<Message>>
    where
        F: Fn(TimePickerAction) -> Message,
        Renderer: geometry::Renderer,
    {
        if state.drag.is_some_and(|drag| drag.pointer == pointer) {
            state.drag = None;
            let action =
                if self.auto_switch_to_minute && self.selection == TimePickerSelectionMode::Hour {
                    TimePickerAction::SetSelection(TimePickerSelectionMode::Minute)
                } else {
                    TimePickerAction::FinishDrag
                };

            Some(canvas::Action::publish((self.on_action)(action)).and_capture())
        } else {
            None
        }
    }

    fn action_at(&self, position: Point, size: Size) -> TimePickerAction {
        self.action_at_with_mode(position, size, false)
    }

    fn drag_action_at(&self, position: Point, size: Size) -> TimePickerAction {
        self.action_at_with_mode(position, size, true)
    }

    fn action_at_with_mode(&self, position: Point, size: Size, dragging: bool) -> TimePickerAction {
        let center = Point::new(size.width / 2.0, size.height / 2.0);
        let dx = position.x - center.x;
        let dy = position.y - center.y;
        let angle = dy.atan2(dx);

        if self.selection == TimePickerSelectionMode::Minute {
            let snapped_angle = (angle / (TAU / 60.0) / 5.0).round() * 5.0 * (TAU / 60.0);
            let minute = angle_to_minute(snapped_angle);
            if dragging {
                TimePickerAction::DragMinuteAngle(minute, pack_angle(angle))
            } else {
                TimePickerAction::SelectMinute(minute)
            }
        } else {
            let action_angle = if dragging {
                angle
            } else {
                (angle / (TAU / 12.0)).round() * (TAU / 12.0)
            };
            let mut hour = angle_to_hour(action_angle);

            if self.is_24_hour {
                let distance = (dx * dx + dy * dy).sqrt();
                let max_distance = tokens::component::time_picker::MAX_DISTANCE;
                if distance < max_distance {
                    if hour == 12 {
                        hour = 0;
                    } else {
                        hour += 12;
                    }
                }
            }

            if dragging {
                TimePickerAction::DragHourAngle(hour, pack_angle(angle))
            } else {
                TimePickerAction::SelectHour(hour)
            }
        }
    }

    fn render_key(&self, theme: &Theme) -> ClockFaceRenderKey {
        let colors = theme.colors();

        ClockFaceRenderKey {
            hour: self.hour,
            minute: self.minute,
            is_24_hour: self.is_24_hour,
            selection: self.selection,
            previous_selection: self.previous_selection,
            selected_selection: self.selected_selection,
            selection_progress: self.selection_progress,
            selector_angle: self.selector_angle,
            dial_color: colors.surface.container.highest,
            selector_color: colors.primary.color,
            label_text_color: colors.surface.text,
            selected_text_color: colors.primary.text,
        }
    }

    fn draw_selector<Renderer>(
        &self,
        frame: &mut Frame<Renderer>,
        theme: &Theme,
        center: Point,
        radius: f32,
    ) where
        Renderer: geometry::Renderer,
    {
        let selector_color = theme.colors().primary.color;
        let angle = self.selector_angle;
        let (handle_center, handle_radius, selector_radius) =
            self.selector_handle_geometry(center, radius);
        let line_end = Point::new(
            center.x + (selector_radius - handle_radius) * angle.cos(),
            center.y + (selector_radius - handle_radius) * angle.sin(),
        );

        let line = Path::line(center, line_end);
        frame.stroke(
            &line,
            Stroke::default()
                .with_width(tokens::component::time_picker::CLOCK_DIAL_SELECTOR_TRACK_WIDTH)
                .with_color(selector_color),
        );
        frame.fill(&Path::circle(handle_center, handle_radius), selector_color);
        frame.fill(
            &Path::circle(
                center,
                tokens::component::time_picker::CLOCK_DIAL_SELECTOR_CENTER_SIZE / 2.0,
            ),
            selector_color,
        );
    }

    fn draw_labels<Renderer>(
        &self,
        frame: &mut Frame<Renderer>,
        theme: &Theme,
        center: Point,
        radius: f32,
        pass: ClockLabelPass,
    ) where
        Renderer: geometry::Renderer,
    {
        if self.previous_selection != self.selected_selection && self.selection_progress < 1.0 {
            self.draw_labels_for_selection(
                frame,
                theme,
                center,
                radius,
                self.previous_selection,
                1.0 - self.selection_progress,
                pass,
            );
            self.draw_labels_for_selection(
                frame,
                theme,
                center,
                radius,
                self.selected_selection,
                self.selection_progress,
                pass,
            );
        } else {
            self.draw_labels_for_selection(frame, theme, center, radius, self.selection, 1.0, pass);
        }
    }

    fn draw_labels_for_selection<Renderer>(
        &self,
        frame: &mut Frame<Renderer>,
        theme: &Theme,
        center: Point,
        radius: f32,
        selection: TimePickerSelectionMode,
        alpha: f32,
        pass: ClockLabelPass,
    ) where
        Renderer: geometry::Renderer,
    {
        if alpha <= 0.0 {
            return;
        }

        let label_radius = radius * 2.0 * tokens::component::time_picker::OUTER_CIRCLE_RADIUS_RATIO;
        let scale = tokens::component::time_picker::CLOCK_DIAL_LABEL_TEXT;

        match selection {
            TimePickerSelectionMode::Hour => {
                for hour in HOURS {
                    let value = if self.is_24_hour { hour % 12 } else { hour };
                    let angle = hour_angle(value);
                    self.draw_clock_label_for_pass(
                        frame,
                        theme,
                        center,
                        radius,
                        label_radius,
                        angle,
                        selection,
                        value,
                        &value.to_string(),
                        scale,
                        alpha,
                        pass,
                    );
                }

                if self.is_24_hour {
                    let inner_radius =
                        radius * 2.0 * tokens::component::time_picker::INNER_CIRCLE_RADIUS_RATIO;
                    for hour in EXTRA_HOURS {
                        let angle = hour_angle(hour);
                        self.draw_clock_label_for_pass(
                            frame,
                            theme,
                            center,
                            radius,
                            inner_radius,
                            angle,
                            selection,
                            hour,
                            &hour.to_string(),
                            scale,
                            alpha,
                            pass,
                        );
                    }
                }
            }
            TimePickerSelectionMode::Minute => {
                for minute in MINUTES {
                    let angle = minute_angle(minute);
                    self.draw_clock_label_for_pass(
                        frame,
                        theme,
                        center,
                        radius,
                        label_radius,
                        angle,
                        selection,
                        minute,
                        &two_digit(minute),
                        scale,
                        alpha,
                        pass,
                    );
                }
            }
        }
    }

    fn draw_clock_label_for_pass<Renderer>(
        &self,
        frame: &mut Frame<Renderer>,
        theme: &Theme,
        center: Point,
        clock_radius: f32,
        label_radius: f32,
        angle: f32,
        selection: TimePickerSelectionMode,
        value: u8,
        label: &str,
        scale: tokens::typography::TypeScale,
        alpha: f32,
        pass: ClockLabelPass,
    ) where
        Renderer: geometry::Renderer,
    {
        match pass {
            ClockLabelPass::Background => {
                draw_clock_label(
                    frame,
                    theme,
                    center,
                    label_radius,
                    angle,
                    label,
                    false,
                    scale,
                    alpha,
                );
            }
            ClockLabelPass::SelectedForeground => {
                if !self.label_matches_selected_value(selection, value)
                    || !self.label_matches_selector_ring(clock_radius, label_radius)
                    || !self.label_intersects_selector(
                        center,
                        clock_radius,
                        label_radius,
                        angle,
                        scale,
                    )
                {
                    return;
                }

                let (handle_center, handle_radius, _) =
                    self.selector_handle_geometry(center, clock_radius);
                draw_clock_label_clipped_to_circle(
                    frame,
                    theme,
                    center,
                    label_radius,
                    angle,
                    label,
                    true,
                    scale,
                    alpha,
                    handle_center,
                    handle_radius,
                );
            }
        }
    }

    fn selector_handle_geometry(&self, center: Point, radius: f32) -> (Point, f32, f32) {
        let handle_radius = tokens::component::time_picker::CLOCK_DIAL_SELECTOR_HANDLE_SIZE / 2.0;
        let outer = radius * 2.0 * tokens::component::time_picker::OUTER_CIRCLE_RADIUS_RATIO;
        let inner = radius * 2.0 * tokens::component::time_picker::INNER_CIRCLE_RADIUS_RATIO;
        let selector_radius = if self.selection == TimePickerSelectionMode::Hour
            && self.is_24_hour
            && self.hour >= 12
        {
            inner
        } else {
            outer
        };
        let angle = self.selector_angle;
        let handle_center = clock_label_position(center, selector_radius, angle);

        (handle_center, handle_radius, selector_radius)
    }

    fn label_matches_selected_value(&self, selection: TimePickerSelectionMode, value: u8) -> bool {
        match selection {
            TimePickerSelectionMode::Hour => {
                if self.is_24_hour {
                    value == selected_24_hour_label_value(self.hour)
                } else {
                    value == hour_for_display(self.hour, false)
                }
            }
            TimePickerSelectionMode::Minute => value == visible_minute(self.minute),
        }
    }

    fn label_matches_selector_ring(&self, clock_radius: f32, label_radius: f32) -> bool {
        let (_, _, selector_radius) = self.selector_handle_geometry(Point::ORIGIN, clock_radius);

        (label_radius - selector_radius).abs() <= 1.0
    }

    fn label_intersects_selector(
        &self,
        center: Point,
        radius: f32,
        label_radius: f32,
        label_angle: f32,
        scale: tokens::typography::TypeScale,
    ) -> bool {
        let (handle_center, handle_radius, _) = self.selector_handle_geometry(center, radius);
        let label_center = clock_label_position(center, label_radius, label_angle);
        let dx = label_center.x - handle_center.x;
        let dy = label_center.y - handle_center.y;
        let label_extent = scale.line_height.max(scale.size * 1.5) / 2.0;
        let threshold = handle_radius + label_extent;

        dx * dx + dy * dy <= threshold * threshold
    }
}

fn event_position(
    event: &canvas::Event,
    bounds: Rectangle,
    cursor: mouse::Cursor,
) -> Option<Point> {
    if cursor.position().is_some() {
        return cursor
            .position()
            .map(|position| Point::new(position.x - bounds.x, position.y - bounds.y));
    }

    if cursor.is_levitating() {
        return None;
    }

    match event {
        event::Event::Mouse(mouse::Event::CursorMoved { position }) => {
            Some(Point::new(position.x - bounds.x, position.y - bounds.y))
        }
        event::Event::Touch(
            touch::Event::FingerPressed { position, .. }
            | touch::Event::FingerMoved { position, .. }
            | touch::Event::FingerLifted { position, .. }
            | touch::Event::FingerLost { position, .. },
        ) => Some(Point::new(position.x - bounds.x, position.y - bounds.y)),
        _ => None,
    }
}

fn local_point_is_in_bounds(position: Point, size: Size) -> bool {
    position.x >= 0.0 && position.y >= 0.0 && position.x <= size.width && position.y <= size.height
}

fn draw_clock_label<Renderer>(
    frame: &mut Frame<Renderer>,
    theme: &Theme,
    center: Point,
    radius: f32,
    angle: f32,
    label: &str,
    selected: bool,
    scale: tokens::typography::TypeScale,
    alpha: f32,
) where
    Renderer: geometry::Renderer,
{
    let text = clock_label_text(theme, center, radius, angle, label, selected, scale, alpha);
    frame.fill_text(text);
}

fn draw_clock_label_clipped_to_circle<Renderer>(
    frame: &mut Frame<Renderer>,
    theme: &Theme,
    center: Point,
    radius: f32,
    angle: f32,
    label: &str,
    selected: bool,
    scale: tokens::typography::TypeScale,
    alpha: f32,
    clip_center: Point,
    clip_radius: f32,
) where
    Renderer: geometry::Renderer,
{
    let text = clock_label_text(theme, center, radius, angle, label, selected, scale, alpha);
    let strip_height = clip_radius * 2.0 / CLOCK_LABEL_CLIP_STRIPS as f32;

    for index in 0..CLOCK_LABEL_CLIP_STRIPS {
        let top = clip_center.y - clip_radius + index as f32 * strip_height;
        let middle = top + strip_height / 2.0;
        let dy = middle - clip_center.y;
        let half_width = (clip_radius * clip_radius - dy * dy).max(0.0).sqrt();

        if half_width <= 0.0 {
            continue;
        }

        frame.with_clip(
            Rectangle {
                x: clip_center.x - half_width,
                y: top,
                width: half_width * 2.0,
                height: strip_height,
            },
            |frame| frame.fill_text(text.clone()),
        );
    }
}

fn clock_label_text(
    theme: &Theme,
    center: Point,
    radius: f32,
    angle: f32,
    label: &str,
    selected: bool,
    scale: tokens::typography::TypeScale,
    alpha: f32,
) -> CanvasText {
    let colors = theme.colors();
    let color = if selected {
        colors.primary.text
    } else {
        colors.surface.text
    };
    let position = clock_label_position(center, radius, angle);

    CanvasText {
        content: label.to_owned(),
        position,
        max_width: 48.0,
        color: alpha_color(color, alpha),
        size: scale.size.into(),
        line_height: absolute_line_height(scale.line_height),
        font: fonts::roboto_for_type_scale(scale),
        align_x: core_text::Alignment::Center,
        align_y: alignment::Vertical::Center,
        shaping: text::Shaping::Advanced,
    }
}

fn clock_label_position(center: Point, radius: f32, angle: f32) -> Point {
    Point::new(
        center.x + radius * angle.cos(),
        center.y + radius * angle.sin(),
    )
}

fn translated<'a, Message, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
    translation: Vector,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + 'a,
{
    Element::new(Translated {
        content: content.into(),
        translation,
    })
}

struct Translated<'a, Message, Renderer> {
    content: Element<'a, Message, Theme, Renderer>,
    translation: Vector,
}

impl<Message, Renderer> fmt::Debug for Translated<'_, Message, Renderer> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Translated")
            .field("translation", &self.translation)
            .finish_non_exhaustive()
    }
}

impl<Message, Renderer> Widget<Message, Theme, Renderer> for Translated<'_, Message, Renderer>
where
    Renderer: iced_widget::core::Renderer,
{
    fn tag(&self) -> tree::Tag {
        self.content.as_widget().tag()
    }

    fn state(&self) -> tree::State {
        self.content.as_widget().state()
    }

    fn children(&self) -> Vec<Tree> {
        self.content.as_widget().children()
    }

    fn diff(&self, tree: &mut Tree) {
        self.content.as_widget().diff(tree);
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.content.as_widget().size_hint()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content.as_widget_mut().layout(tree, renderer, limits)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        self.content
            .as_widget_mut()
            .operate(tree, layout, renderer, operation);
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let translation = self.translation;

        self.content.as_widget_mut().update(
            tree,
            event,
            layout,
            cursor - translation,
            renderer,
            clipboard,
            shell,
            &(*viewport - translation),
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let translation = self.translation;

        self.content.as_widget().mouse_interaction(
            tree,
            layout,
            cursor - translation,
            &(*viewport - translation),
            renderer,
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let Some(viewport) = layout.bounds().intersection(viewport) else {
            return;
        };
        let translation = self.translation;

        renderer.with_layer(viewport, |renderer| {
            renderer.with_translation(translation, |renderer| {
                self.content.as_widget().draw(
                    tree,
                    renderer,
                    theme,
                    style,
                    layout,
                    cursor - translation,
                    &(viewport - translation),
                );
            });
        });
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        self.content.as_widget_mut().overlay(
            tree,
            layout,
            renderer,
            viewport,
            translation + self.translation,
        )
    }
}

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
    let background_alpha = date_input_panel_background_alpha(content_alpha);

    iced_widget::container::Style {
        background: (background_alpha > 0.0).then_some(Background::Color(alpha_color(
            colors.surface.container.high,
            background_alpha,
        ))),
        text_color: Some(alpha_color(colors.surface.text, content_alpha)),
        ..Default::default()
    }
}

fn date_input_panel_background_alpha(content_alpha: f32) -> f32 {
    if content_alpha > 0.0 { 1.0 } else { 0.0 }
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

#[cfg(test)]
mod tests {
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
    fn time_picker_drag_angle_snaps_to_visible_tick_and_release_keeps_target() {
        let start = Instant::now();
        let mut state = TimePickerState::new(12, 0, false);

        state.update_at(
            TimePickerAction::SetSelection(TimePickerSelectionMode::Minute),
            start,
        );
        let _ = state.advance(start + duration_ms(tokens::motion::DURATION_MEDIUM1_MS));

        let drag_start = start + duration_ms(tokens::motion::DURATION_MEDIUM1_MS + 16);
        let from = state.animation.clock_angle();
        let raw_angle = minute_angle(17);
        let drag_angle = nearest_angle(from, unpack_angle(pack_angle(raw_angle)));
        let target = nearest_angle(drag_angle, minute_angle(15));

        state.update_at(
            TimePickerAction::DragMinuteAngle(15, pack_angle(raw_angle)),
            drag_start,
        );

        assert_eq!(state.minute(), 15);
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

        let press = <ClockFace<_> as canvas::Program<
            TimePickerAction,
            Theme,
            iced_widget::Renderer,
        >>::update(
            &face,
            &mut state,
            &event::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
            bounds,
            mouse::Cursor::Available(top),
        )
        .expect("press should publish selection");
        let (message, _, _) = press.into_inner();
        assert_eq!(message, Some(TimePickerAction::SelectHour(12)));

        let drag = <ClockFace<_> as canvas::Program<
            TimePickerAction,
            Theme,
            iced_widget::Renderer,
        >>::update(
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

        let release = <ClockFace<_> as canvas::Program<
            TimePickerAction,
            Theme,
            iced_widget::Renderer,
        >>::update(
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

        let press = <ClockFace<_> as canvas::Program<
            TimePickerAction,
            Theme,
            iced_widget::Renderer,
        >>::update(
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

        let drag = <ClockFace<_> as canvas::Program<
            TimePickerAction,
            Theme,
            iced_widget::Renderer,
        >>::update(
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

        let release = <ClockFace<_> as canvas::Program<
            TimePickerAction,
            Theme,
            iced_widget::Renderer,
        >>::update(
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
    fn clock_face_minute_tap_and_drag_snap_to_visible_ticks() {
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
            TimePickerAction::SelectMinute(15)
        );
        assert_eq!(
            face.drag_action_at(minute_17, size),
            TimePickerAction::DragMinuteAngle(15, pack_angle(minute_angle(17)))
        );
        assert_eq!(
            face.drag_action_at(minute_8, size),
            TimePickerAction::DragMinuteAngle(10, pack_angle(minute_angle(8)))
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
        assert!(!face.label_intersects_selector(
            center,
            radius,
            label_radius,
            hour_angle(5),
            scale
        ));
    }

    #[test]
    fn clock_face_overlay_label_uses_current_snapped_value_only() {
        let face = ClockFace {
            hour: 12,
            minute: 15,
            is_24_hour: false,
            selection: TimePickerSelectionMode::Minute,
            previous_selection: TimePickerSelectionMode::Minute,
            selected_selection: TimePickerSelectionMode::Minute,
            selection_progress: 1.0,
            auto_switch_to_minute: true,
            selector_angle: minute_angle(13),
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

        assert!(face.label_intersects_selector(
            center,
            radius,
            label_radius,
            minute_angle(10),
            scale
        ));
        assert!(face.label_intersects_selector(
            center,
            radius,
            label_radius,
            minute_angle(15),
            scale
        ));
        assert!(!face.label_matches_selected_value(TimePickerSelectionMode::Minute, 10));
        assert!(face.label_matches_selected_value(TimePickerSelectionMode::Minute, 15));
        assert_eq!(visible_minute(17), 15);
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

        let release = <ClockFace<_> as canvas::Program<
            TimePickerAction,
            Theme,
            iced_widget::Renderer,
        >>::update(
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
}
