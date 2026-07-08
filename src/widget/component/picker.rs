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
use iced_widget::renderer::wgpu::primitive;
use iced_widget::text::{self, LineHeight};
use iced_widget::{Column, Container, Row, Scrollable, Space, Stack, Text};

use super::absolute_line_height;
use super::button::Button;
use super::support::{AnimatedScalar, alpha_color, duration_ms, lerp};
use super::text_input;
use super::viewport;
use crate::style::button as button_style;
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

/// Visual options shared by date picker surfaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DatePickerOptions {
    pub show_mode_toggle: bool,
}

impl Default for DatePickerOptions {
    fn default() -> Self {
        Self {
            show_mode_toggle: true,
        }
    }
}

impl DatePickerOptions {
    /// Sets whether the picker/input mode toggle is shown in the header.
    pub const fn show_mode_toggle(mut self, show_mode_toggle: bool) -> Self {
        self.show_mode_toggle = show_mode_toggle;
        self
    }
}

/// Visual options for the analog time picker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimePickerOptions {
    pub layout: TimePickerLayout,
}

impl Default for TimePickerOptions {
    fn default() -> Self {
        Self {
            layout: TimePickerLayout::Vertical,
        }
    }
}

impl TimePickerOptions {
    /// Sets the analog time picker layout.
    pub const fn layout(mut self, layout: TimePickerLayout) -> Self {
        self.layout = layout;
        self
    }
}

/// Creates a Material 3 date picker.
pub fn date_picker<'a, Message, Renderer>(
    state: &'a DatePickerState,
    on_action: impl Fn(DatePickerAction) -> Message + Clone + 'a,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    date_picker_with(state, on_action, DatePickerOptions::default())
}

/// Creates a Material 3 date picker with custom visual options.
pub fn date_picker_with<'a, Message, Renderer>(
    state: &'a DatePickerState,
    on_action: impl Fn(DatePickerAction) -> Message + Clone + 'a,
    options: DatePickerOptions,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    Container::new(date_picker_body(state, on_action, options.show_mode_toggle))
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
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    date_range_picker_with(state, on_action, DatePickerOptions::default())
}

/// Creates a Material 3 date range picker with custom visual options.
pub fn date_range_picker_with<'a, Message, Renderer>(
    state: &'a DateRangePickerState,
    on_action: impl Fn(DateRangePickerAction) -> Message + Clone + 'a,
    options: DatePickerOptions,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    Container::new(date_range_picker_body(
        state,
        on_action,
        options.show_mode_toggle,
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
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    date_picker_dialog_with(state, on_action, actions, DatePickerOptions::default())
}

/// Creates a Material 3 date picker dialog surface with custom visual options.
pub fn date_picker_dialog_with<'a, Message, Renderer>(
    state: &'a DatePickerState,
    on_action: impl Fn(DatePickerAction) -> Message + Clone + 'a,
    actions: impl Into<Element<'a, Message, Theme, Renderer>>,
    options: DatePickerOptions,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    date_picker_dialog_surface(
        date_picker_body(state, on_action, options.show_mode_toggle),
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
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    date_range_picker_dialog_with(state, on_action, actions, DatePickerOptions::default())
}

/// Creates a Material 3 date range picker dialog surface with custom visual options.
pub fn date_range_picker_dialog_with<'a, Message, Renderer>(
    state: &'a DateRangePickerState,
    on_action: impl Fn(DateRangePickerAction) -> Message + Clone + 'a,
    actions: impl Into<Element<'a, Message, Theme, Renderer>>,
    options: DatePickerOptions,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    date_picker_dialog_surface(
        date_range_picker_body(
            state,
            on_action,
            options.show_mode_toggle,
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
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'static + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    time_picker_with(state, on_action, TimePickerOptions::default())
}

/// Creates a Material 3 analog time picker with custom visual options.
pub fn time_picker_with<'a, Message, Renderer>(
    state: &'a TimePickerState,
    on_action: impl Fn(TimePickerAction) -> Message + Clone + 'a,
    options: TimePickerOptions,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'static + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    Container::new(time_picker_body(state, on_action, options.layout))
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
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'static + 'a,
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
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'static + 'a,
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
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'a,
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
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'a,
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
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'static + 'a,
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
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'a,
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

include!("picker/time_scroll.rs");

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
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    super::button::icon_button(
        display_mode.toggle_icon(),
        super::button::IconButtonVariant::Standard,
    )
    .on_press(on_press)
}

fn date_header<'a, Message, Renderer>(
    state: &'a DatePickerState,
    on_action: impl Fn(DatePickerAction) -> Message + Clone + 'a,
    show_mode_toggle: bool,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'a,
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
        let toggle =
            super::button::icon_button(toggle_icon, super::button::IconButtonVariant::Standard)
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
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'a,
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
        let toggle =
            super::button::icon_button(toggle_icon, super::button::IconButtonVariant::Standard)
                .on_press(on_action(DateRangePickerAction::SetDisplayMode(
                    toggle_mode,
                )));
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
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    let year_picker_progress = state.animation.year_picker_progress();
    let year_picker_alpha =
        content_alpha * year_picker_content_alpha(year_picker_progress, state.year_picker_visible);
    let calendar = Column::new()
        .push(weekdays_row(state.first_day_of_week, content_alpha))
        .push(animated_month_grid(state, on_action.clone(), content_alpha));
    let content: Element<'a, Message, Theme, Renderer> =
        if year_picker_should_render(year_picker_progress, state.year_picker_visible) {
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
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'a,
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

    if year_picker_should_render(year_picker_progress, state.year_picker_visible) {
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

fn year_picker_should_render(progress: f32, visible: bool) -> bool {
    visible || progress > 0.0
}

fn date_input_content<'a, Message, Renderer>(
    state: &'a DatePickerState,
    on_action: impl Fn(DatePickerAction) -> Message + Clone + 'a,
    content_alpha: f32,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'a,
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
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'a,
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
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'a,
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
                super::button::icon_button(
                    "keyboard_arrow_left",
                    super::button::IconButtonVariant::Standard,
                )
                .on_press_maybe(
                    can_previous.then(|| on_action.clone()(DatePickerAction::PreviousMonth)),
                ),
            )
            .push(
                super::button::icon_button(
                    "keyboard_arrow_right",
                    super::button::IconButtonVariant::Standard,
                )
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
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'a,
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
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'a,
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
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'a,
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
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'a,
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
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'a,
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
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'a,
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
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'a,
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
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'a,
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
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'a,
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
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'a,
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
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'a,
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
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'a,
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
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'a,
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
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'a,
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
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'a,
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

include!("picker/clock.rs");

include!("picker/translated.rs");

include!("picker/style.rs");

include!("picker/helpers.rs");

#[cfg(test)]
#[path = "../../../tests/widget/component/picker.rs"]
mod tests;
