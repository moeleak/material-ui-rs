//! Material 3 sized widget constructors.
//!
//! The style traits exposed by `iced` control colors, borders, and shadows, but
//! not layout defaults like button height or checkbox size. These helpers apply
//! the Material 3 component metrics from [`crate::tokens`] at construction time.

use iced_widget::checkbox as iced_checkbox;
use iced_widget::core::svg as core_svg;
use iced_widget::core::text as core_text;
use iced_widget::core::time::{Duration, Instant};
use iced_widget::core::widget as core_widget;
use iced_widget::core::widget::tree::{self, Tree};
use iced_widget::core::{
    Background, Border, Clipboard, Color, Element, Event, Layout, Length, Padding, Pixels, Point,
    Rectangle, Shell, Size, Widget, alignment, border, layout, mouse, renderer, touch, window,
};
use iced_widget::radio as iced_radio;
use iced_widget::text::{self, LineHeight};
use iced_widget::text_input as iced_text_input;
use iced_widget::toggler as iced_toggler;
use iced_widget::tooltip as iced_tooltip;
use iced_widget::{
    Button, Container, ProgressBar, Slider, Text, TextInput as IcedTextInput, Tooltip,
};

use crate::utils::mix;
use crate::{Theme, tokens};
use crate::{
    button as button_style, checkbox as checkbox_style, container as container_style,
    progress_bar as progress_bar_style, slider as slider_style, text_input as text_input_style,
    toggler as toggler_style, tooltip as tooltip_style,
};

const SWITCH_ON_ICON_SVG: &[u8] = br##"
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
  <path d="M9.55 18.2 3.65 12.3 5.275 10.675 9.55 14.95 18.725 5.775 20.35 7.4Z"/>
</svg>
"##;

const SWITCH_OFF_ICON_SVG: &[u8] = br##"
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
  <path d="M6.4 19.2 4.8 17.6 10.4 12 4.8 6.4 6.4 4.8 12 10.4 17.6 4.8 19.2 6.4 13.6 12 19.2 17.6 17.6 19.2 12 13.6Z"/>
</svg>
"##;

fn absolute_line_height(value: f32) -> LineHeight {
    LineHeight::Absolute(value.into())
}

fn text_with_metrics<'a, Renderer>(
    content: impl text::IntoFragment<'a>,
    size: f32,
    line_height: f32,
) -> Text<'a, Theme, Renderer>
where
    Renderer: core_text::Renderer,
{
    Text::new(content)
        .size(size)
        .line_height(absolute_line_height(line_height))
}

fn checkbox_checkmark_svg(mark_progress: f32) -> Vec<u8> {
    let progress = mark_progress.clamp(0.0, 1.0);
    let short_height = lerp(
        tokens::component::checkbox::CHECKMARK_STROKE_WIDTH,
        tokens::component::checkbox::CHECKMARK_SHORT_MARK_SIZE,
        progress,
    );
    let long_width = tokens::component::checkbox::CHECKMARK_LONG_MARK_SIZE * progress;

    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 18 18"><g transform="scale(1 -1) translate({} {}) rotate(45)"><rect width="{}" height="{short_height}"/><rect width="{long_width}" height="{}"/></g></svg>"#,
        tokens::component::checkbox::CHECKMARK_BOTTOM_LEFT_X,
        tokens::component::checkbox::CHECKMARK_BOTTOM_LEFT_Y,
        tokens::component::checkbox::CHECKMARK_STROKE_WIDTH,
        tokens::component::checkbox::CHECKMARK_STROKE_WIDTH,
    )
    .into_bytes()
}

fn duration_ms(milliseconds: u16) -> Duration {
    Duration::from_millis(u64::from(milliseconds))
}

fn lerp(from: f32, to: f32, progress: f32) -> f32 {
    from + (to - from) * progress
}

fn bezier_axis(t: f32, p1: f32, p2: f32) -> f32 {
    let one_minus_t = 1.0 - t;

    3.0 * one_minus_t * one_minus_t * t * p1 + 3.0 * one_minus_t * t * t * p2 + t * t * t
}

fn cubic_bezier(progress: f32, easing: tokens::motion::CubicBezier) -> f32 {
    let target_x = progress.clamp(0.0, 1.0);
    let mut start = 0.0;
    let mut end = 1.0;

    for _ in 0..20 {
        let midpoint = (start + end) / 2.0;

        if bezier_axis(midpoint, easing.x1, easing.x2) < target_x {
            start = midpoint;
        } else {
            end = midpoint;
        }
    }

    bezier_axis((start + end) / 2.0, easing.y1, easing.y2)
}

fn bool_value(value: bool) -> f32 {
    if value { 1.0 } else { 0.0 }
}

fn solid_color(background: Background) -> Color {
    match background {
        Background::Color(color) => color,
        Background::Gradient(_) => Color::TRANSPARENT,
    }
}

fn alpha_color(mut color: Color, alpha: f32) -> Color {
    color.a *= alpha.clamp(0.0, 1.0);
    color
}

fn alpha_border(mut border: Border, alpha: f32) -> Border {
    border.color = alpha_color(border.color, alpha);
    border
}

fn scaled_rect(bounds: Rectangle, width: f32, height: f32) -> Rectangle {
    Rectangle {
        x: bounds.center_x() - width / 2.0,
        y: bounds.center_y() - height / 2.0,
        width,
        height,
    }
}

#[derive(Debug, Clone, Copy)]
struct AnimatedScalar {
    value: f32,
    from: f32,
    to: f32,
    started_at: Option<Instant>,
    duration: Duration,
    easing: tokens::motion::CubicBezier,
}

impl AnimatedScalar {
    fn new(value: f32) -> Self {
        Self {
            value,
            from: value,
            to: value,
            started_at: None,
            duration: Duration::ZERO,
            easing: tokens::motion::EASING_LINEAR,
        }
    }

    fn set_target(
        &mut self,
        to: f32,
        now: Instant,
        duration: Duration,
        easing: tokens::motion::CubicBezier,
    ) {
        if (self.to - to).abs() <= f32::EPSILON {
            return;
        }

        self.from = self.value;
        self.to = to;
        self.started_at = Some(now);
        self.duration = duration;
        self.easing = easing;
    }

    fn advance(&mut self, now: Instant) -> bool {
        let Some(started_at) = self.started_at else {
            self.value = self.to;
            return false;
        };

        if self.duration.is_zero() {
            self.value = self.to;
            self.started_at = None;
            return false;
        }

        let progress = (now.duration_since(started_at).as_secs_f32() / self.duration.as_secs_f32())
            .clamp(0.0, 1.0);

        self.value = lerp(self.from, self.to, cubic_bezier(progress, self.easing));

        if progress >= 1.0 {
            self.value = self.to;
            self.started_at = None;
            false
        } else {
            true
        }
    }

    fn is_animating(&self) -> bool {
        self.started_at.is_some()
    }
}

struct SelectionState<Paragraph: core_text::Paragraph, Status> {
    text: core_widget::text::State<Paragraph>,
    target: bool,
    position: AnimatedScalar,
    color: AnimatedScalar,
    size: AnimatedScalar,
    icon: AnimatedScalar,
    icon_opacity: AnimatedScalar,
    is_pressed: bool,
    last_status: Option<Status>,
}

impl<Paragraph: core_text::Paragraph, Status> SelectionState<Paragraph, Status> {
    fn new(target: bool) -> Self {
        let value = bool_value(target);

        Self {
            text: core_widget::text::State::<Paragraph>::default(),
            target,
            position: AnimatedScalar::new(value),
            color: AnimatedScalar::new(value),
            size: AnimatedScalar::new(value),
            icon: AnimatedScalar::new(value),
            icon_opacity: AnimatedScalar::new(value),
            is_pressed: false,
            last_status: None,
        }
    }

    fn is_animating(&self) -> bool {
        self.position.is_animating()
            || self.color.is_animating()
            || self.size.is_animating()
            || self.icon.is_animating()
            || self.icon_opacity.is_animating()
    }

    fn advance(&mut self, now: Instant) -> bool {
        self.position.advance(now)
            | self.color.advance(now)
            | self.size.advance(now)
            | self.icon.advance(now)
            | self.icon_opacity.advance(now)
    }
}

struct TextFieldState<Paragraph: core_text::Paragraph> {
    label: core_widget::text::State<Paragraph>,
    label_float: AnimatedScalar,
    is_focused: bool,
}

impl<Paragraph: core_text::Paragraph> TextFieldState<Paragraph> {
    fn new(is_populated: bool) -> Self {
        Self {
            label: core_widget::text::State::<Paragraph>::default(),
            label_float: AnimatedScalar::new(bool_value(is_populated)),
            is_focused: false,
        }
    }

    fn is_animating(&self) -> bool {
        self.label_float.is_animating()
    }
}

fn text_button_content<'a, Message, Renderer>(
    label: impl text::IntoFragment<'a>,
    label_size: f32,
    label_line_height: f32,
    height: f32,
    horizontal_padding: f32,
) -> Container<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
{
    Container::new(text_with_metrics(label, label_size, label_line_height))
        .height(Length::Fixed(height))
        .padding(Padding::from([0.0, horizontal_padding]))
        .align_y(alignment::Vertical::Center)
}

fn icon_button_content<'a, Message, Renderer>(
    icon: impl text::IntoFragment<'a>,
) -> Container<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
{
    let icon = Text::new(icon)
        .size(tokens::component::icon_button::ICON_SIZE)
        .line_height(absolute_line_height(
            tokens::component::icon_button::ICON_SIZE,
        ));

    Container::new(icon)
        .center_x(Length::Fixed(
            tokens::component::icon_button::CONTAINER_WIDTH,
        ))
        .center_y(Length::Fixed(
            tokens::component::icon_button::CONTAINER_HEIGHT,
        ))
}

fn fab_content<'a, Message, Renderer>(
    icon: impl text::IntoFragment<'a>,
) -> Container<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
{
    let icon = Text::new(icon)
        .size(tokens::component::fab::ICON_SIZE)
        .line_height(absolute_line_height(tokens::component::fab::ICON_SIZE));

    Container::new(icon)
        .center_x(Length::Fixed(tokens::component::fab::CONTAINER_WIDTH))
        .center_y(Length::Fixed(tokens::component::fab::CONTAINER_HEIGHT))
}

pub mod button {
    //! Material 3 button constructors with token-backed layout defaults.

    use super::*;

    fn standard<'a, Message, Renderer>(
        label: impl text::IntoFragment<'a>,
        style: fn(&Theme, iced_widget::button::Status) -> iced_widget::button::Style,
    ) -> Button<'a, Message, Theme, Renderer>
    where
        Message: Clone + 'a,
        Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
    {
        Button::new(text_button_content(
            label,
            tokens::component::button::LABEL_TEXT_SIZE,
            tokens::component::button::LABEL_TEXT_LINE_HEIGHT,
            tokens::component::button::CONTAINER_HEIGHT,
            tokens::component::button::LEADING_SPACE,
        ))
        .height(Length::Fixed(tokens::component::button::CONTAINER_HEIGHT))
        .padding(Padding::ZERO)
        .style(style)
    }

    fn chip<'a, Message, Renderer>(
        label: impl text::IntoFragment<'a>,
        style: fn(&Theme, iced_widget::button::Status) -> iced_widget::button::Style,
    ) -> Button<'a, Message, Theme, Renderer>
    where
        Message: Clone + 'a,
        Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
    {
        Button::new(text_button_content(
            label,
            tokens::component::chip::LABEL_TEXT_SIZE,
            tokens::component::chip::LABEL_TEXT_LINE_HEIGHT,
            tokens::component::chip::CONTAINER_HEIGHT,
            tokens::component::chip::LEADING_SPACE,
        ))
        .height(Length::Fixed(tokens::component::chip::CONTAINER_HEIGHT))
        .padding(Padding::ZERO)
        .style(style)
    }

    fn icon<'a, Message, Renderer>(
        icon: impl text::IntoFragment<'a>,
        style: fn(&Theme, iced_widget::button::Status) -> iced_widget::button::Style,
    ) -> Button<'a, Message, Theme, Renderer>
    where
        Message: Clone + 'a,
        Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
    {
        Button::new(icon_button_content(icon))
            .width(Length::Fixed(
                tokens::component::icon_button::CONTAINER_WIDTH,
            ))
            .height(Length::Fixed(
                tokens::component::icon_button::CONTAINER_HEIGHT,
            ))
            .padding(Padding::ZERO)
            .style(style)
    }

    fn fab<'a, Message, Renderer>(
        icon_content: impl text::IntoFragment<'a>,
        style: fn(&Theme, iced_widget::button::Status) -> iced_widget::button::Style,
    ) -> Button<'a, Message, Theme, Renderer>
    where
        Message: Clone + 'a,
        Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
    {
        Button::new(fab_content(icon_content))
            .width(Length::Fixed(tokens::component::fab::CONTAINER_WIDTH))
            .height(Length::Fixed(tokens::component::fab::CONTAINER_HEIGHT))
            .padding(Padding::ZERO)
            .style(style)
    }

    pub fn elevated<'a, Message, Renderer>(
        label: impl text::IntoFragment<'a>,
    ) -> Button<'a, Message, Theme, Renderer>
    where
        Message: Clone + 'a,
        Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
    {
        standard(label, button_style::elevated)
    }

    pub fn filled<'a, Message, Renderer>(
        label: impl text::IntoFragment<'a>,
    ) -> Button<'a, Message, Theme, Renderer>
    where
        Message: Clone + 'a,
        Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
    {
        standard(label, button_style::filled)
    }

    pub fn filled_tonal<'a, Message, Renderer>(
        label: impl text::IntoFragment<'a>,
    ) -> Button<'a, Message, Theme, Renderer>
    where
        Message: Clone + 'a,
        Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
    {
        standard(label, button_style::filled_tonal)
    }

    pub fn outlined<'a, Message, Renderer>(
        label: impl text::IntoFragment<'a>,
    ) -> Button<'a, Message, Theme, Renderer>
    where
        Message: Clone + 'a,
        Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
    {
        standard(label, button_style::outlined)
    }

    pub fn text<'a, Message, Renderer>(
        label: impl text::IntoFragment<'a>,
    ) -> Button<'a, Message, Theme, Renderer>
    where
        Message: Clone + 'a,
        Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
    {
        standard(label, button_style::text)
    }

    pub fn icon_button<'a, Message, Renderer>(
        icon_content: impl text::IntoFragment<'a>,
    ) -> Button<'a, Message, Theme, Renderer>
    where
        Message: Clone + 'a,
        Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
    {
        icon(icon_content, button_style::icon)
    }

    pub fn filled_icon<'a, Message, Renderer>(
        icon_content: impl text::IntoFragment<'a>,
    ) -> Button<'a, Message, Theme, Renderer>
    where
        Message: Clone + 'a,
        Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
    {
        icon(icon_content, button_style::filled_icon)
    }

    pub fn filled_tonal_icon<'a, Message, Renderer>(
        icon_content: impl text::IntoFragment<'a>,
    ) -> Button<'a, Message, Theme, Renderer>
    where
        Message: Clone + 'a,
        Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
    {
        icon(icon_content, button_style::filled_tonal_icon)
    }

    pub fn outlined_icon<'a, Message, Renderer>(
        icon_content: impl text::IntoFragment<'a>,
    ) -> Button<'a, Message, Theme, Renderer>
    where
        Message: Clone + 'a,
        Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
    {
        icon(icon_content, button_style::outlined_icon)
    }

    pub fn primary_fab<'a, Message, Renderer>(
        icon_content: impl text::IntoFragment<'a>,
    ) -> Button<'a, Message, Theme, Renderer>
    where
        Message: Clone + 'a,
        Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
    {
        fab(icon_content, button_style::fab_primary)
    }

    pub fn secondary_fab<'a, Message, Renderer>(
        icon_content: impl text::IntoFragment<'a>,
    ) -> Button<'a, Message, Theme, Renderer>
    where
        Message: Clone + 'a,
        Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
    {
        fab(icon_content, button_style::fab_secondary)
    }

    pub fn surface_fab<'a, Message, Renderer>(
        icon_content: impl text::IntoFragment<'a>,
    ) -> Button<'a, Message, Theme, Renderer>
    where
        Message: Clone + 'a,
        Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
    {
        fab(icon_content, button_style::fab_surface)
    }

    pub fn assist_chip<'a, Message, Renderer>(
        label: impl text::IntoFragment<'a>,
    ) -> Button<'a, Message, Theme, Renderer>
    where
        Message: Clone + 'a,
        Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
    {
        chip(label, button_style::assist_chip)
    }

    pub fn elevated_assist_chip<'a, Message, Renderer>(
        label: impl text::IntoFragment<'a>,
    ) -> Button<'a, Message, Theme, Renderer>
    where
        Message: Clone + 'a,
        Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
    {
        chip(label, button_style::elevated_assist_chip)
    }

    pub fn suggestion_chip<'a, Message, Renderer>(
        label: impl text::IntoFragment<'a>,
    ) -> Button<'a, Message, Theme, Renderer>
    where
        Message: Clone + 'a,
        Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
    {
        chip(label, button_style::suggestion_chip)
    }

    pub fn elevated_suggestion_chip<'a, Message, Renderer>(
        label: impl text::IntoFragment<'a>,
    ) -> Button<'a, Message, Theme, Renderer>
    where
        Message: Clone + 'a,
        Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
    {
        chip(label, button_style::elevated_suggestion_chip)
    }

    pub fn filter_chip<'a, Message, Renderer>(
        label: impl text::IntoFragment<'a>,
    ) -> Button<'a, Message, Theme, Renderer>
    where
        Message: Clone + 'a,
        Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
    {
        chip(label, button_style::filter_chip)
    }

    pub fn selected_filter_chip<'a, Message, Renderer>(
        label: impl text::IntoFragment<'a>,
    ) -> Button<'a, Message, Theme, Renderer>
    where
        Message: Clone + 'a,
        Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
    {
        chip(label, button_style::selected_filter_chip)
    }

    pub fn input_chip<'a, Message, Renderer>(
        label: impl text::IntoFragment<'a>,
    ) -> Button<'a, Message, Theme, Renderer>
    where
        Message: Clone + 'a,
        Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
    {
        chip(label, button_style::input_chip)
    }

    pub fn selected_input_chip<'a, Message, Renderer>(
        label: impl text::IntoFragment<'a>,
    ) -> Button<'a, Message, Theme, Renderer>
    where
        Message: Clone + 'a,
        Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
    {
        chip(label, button_style::selected_input_chip)
    }
}

pub mod slider {
    //! Material 3 slider constructors with token-backed layout defaults.

    use super::*;
    use std::ops::RangeInclusive;

    pub fn continuous<'a, T, Message>(
        range: RangeInclusive<T>,
        value: T,
        on_change: impl Fn(T) -> Message + 'a,
    ) -> Slider<'a, T, Message, Theme>
    where
        T: Copy + From<u8> + PartialOrd,
        Message: Clone,
    {
        Slider::new(range, value, on_change)
            .height(tokens::component::slider::STATE_LAYER_SIZE)
            .style(slider_style::default)
    }
}

pub mod progress_bar {
    //! Material 3 progress indicator constructors with token-backed layout defaults.

    use super::*;
    use std::ops::RangeInclusive;

    pub fn linear<'a>(range: RangeInclusive<f32>, value: f32) -> ProgressBar<'a, Theme> {
        ProgressBar::new(range, value)
            .girth(Length::Fixed(
                tokens::component::linear_progress::TRACK_HEIGHT,
            ))
            .style(progress_bar_style::default)
    }

    pub fn vertical_linear<'a>(range: RangeInclusive<f32>, value: f32) -> ProgressBar<'a, Theme> {
        linear(range, value).vertical()
    }
}

pub mod container {
    //! Material 3 container and card constructors.

    use super::*;

    fn styled<'a, Message, Renderer>(
        content: impl Into<Element<'a, Message, Theme, Renderer>>,
        style: fn(&Theme) -> iced_widget::container::Style,
    ) -> Container<'a, Message, Theme, Renderer>
    where
        Message: 'a,
        Renderer: iced_widget::core::Renderer + 'a,
    {
        Container::new(content).style(style)
    }

    pub fn transparent<'a, Message, Renderer>(
        content: impl Into<Element<'a, Message, Theme, Renderer>>,
    ) -> Container<'a, Message, Theme, Renderer>
    where
        Message: 'a,
        Renderer: iced_widget::core::Renderer + 'a,
    {
        styled(content, container_style::transparent)
    }

    pub fn surface<'a, Message, Renderer>(
        content: impl Into<Element<'a, Message, Theme, Renderer>>,
    ) -> Container<'a, Message, Theme, Renderer>
    where
        Message: 'a,
        Renderer: iced_widget::core::Renderer + 'a,
    {
        styled(content, container_style::surface)
    }

    pub fn surface_container_lowest<'a, Message, Renderer>(
        content: impl Into<Element<'a, Message, Theme, Renderer>>,
    ) -> Container<'a, Message, Theme, Renderer>
    where
        Message: 'a,
        Renderer: iced_widget::core::Renderer + 'a,
    {
        styled(content, container_style::surface_container_lowest)
    }

    pub fn surface_container_low<'a, Message, Renderer>(
        content: impl Into<Element<'a, Message, Theme, Renderer>>,
    ) -> Container<'a, Message, Theme, Renderer>
    where
        Message: 'a,
        Renderer: iced_widget::core::Renderer + 'a,
    {
        styled(content, container_style::surface_container_low)
    }

    pub fn surface_container<'a, Message, Renderer>(
        content: impl Into<Element<'a, Message, Theme, Renderer>>,
    ) -> Container<'a, Message, Theme, Renderer>
    where
        Message: 'a,
        Renderer: iced_widget::core::Renderer + 'a,
    {
        styled(content, container_style::surface_container)
    }

    pub fn surface_container_high<'a, Message, Renderer>(
        content: impl Into<Element<'a, Message, Theme, Renderer>>,
    ) -> Container<'a, Message, Theme, Renderer>
    where
        Message: 'a,
        Renderer: iced_widget::core::Renderer + 'a,
    {
        styled(content, container_style::surface_container_high)
    }

    pub fn surface_container_highest<'a, Message, Renderer>(
        content: impl Into<Element<'a, Message, Theme, Renderer>>,
    ) -> Container<'a, Message, Theme, Renderer>
    where
        Message: 'a,
        Renderer: iced_widget::core::Renderer + 'a,
    {
        styled(content, container_style::surface_container_highest)
    }

    pub fn outlined<'a, Message, Renderer>(
        content: impl Into<Element<'a, Message, Theme, Renderer>>,
    ) -> Container<'a, Message, Theme, Renderer>
    where
        Message: 'a,
        Renderer: iced_widget::core::Renderer + 'a,
    {
        styled(content, container_style::outlined)
    }

    pub fn elevated_card<'a, Message, Renderer>(
        content: impl Into<Element<'a, Message, Theme, Renderer>>,
    ) -> Container<'a, Message, Theme, Renderer>
    where
        Message: 'a,
        Renderer: iced_widget::core::Renderer + 'a,
    {
        styled(content, container_style::elevated_card)
    }

    pub fn filled_card<'a, Message, Renderer>(
        content: impl Into<Element<'a, Message, Theme, Renderer>>,
    ) -> Container<'a, Message, Theme, Renderer>
    where
        Message: 'a,
        Renderer: iced_widget::core::Renderer + 'a,
    {
        styled(content, container_style::filled_card)
    }

    pub fn outlined_card<'a, Message, Renderer>(
        content: impl Into<Element<'a, Message, Theme, Renderer>>,
    ) -> Container<'a, Message, Theme, Renderer>
    where
        Message: 'a,
        Renderer: iced_widget::core::Renderer + 'a,
    {
        styled(content, container_style::outlined_card)
    }
}

pub mod tooltip {
    //! Material 3 tooltip constructors with token-backed layout defaults.

    use super::*;

    pub use iced_tooltip::Position;

    pub fn plain<'a, Message, Renderer>(
        content: impl Into<Element<'a, Message, Theme, Renderer>>,
        supporting_text: impl text::IntoFragment<'a>,
        position: Position,
    ) -> Tooltip<'a, Message, Theme, Renderer>
    where
        Message: 'a,
        Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
    {
        let type_scale = tokens::component::tooltip::PLAIN_SUPPORTING_TEXT;
        let inner_horizontal_padding = (tokens::component::tooltip::PLAIN_HORIZONTAL_SPACE
            - tokens::component::tooltip::PLAIN_VERTICAL_SPACE)
            .max(0.0);

        let tooltip = Container::new(
            text_with_metrics(supporting_text, type_scale.size, type_scale.line_height)
                .width(Length::Fill)
                .wrapping(text::Wrapping::Word),
        )
        .padding(Padding {
            top: 0.0,
            right: inner_horizontal_padding,
            bottom: 0.0,
            left: inner_horizontal_padding,
        })
        .max_width(
            tokens::component::tooltip::PLAIN_MAX_WIDTH
                - tokens::component::tooltip::PLAIN_VERTICAL_SPACE * 2.0,
        );

        Tooltip::new(content, tooltip, position)
            .gap(tokens::component::tooltip::SPACING_BETWEEN_TOOLTIP_AND_ANCHOR)
            .padding(tokens::component::tooltip::PLAIN_VERTICAL_SPACE)
            .style(tooltip_style::plain)
    }
}

pub mod text_input {
    //! Material 3 outlined text field constructors with floating label support.

    use super::*;
    use iced_widget::core::text::Paragraph;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum LabelMode {
        Placeholder,
        Floating,
    }

    fn input_layer_style(theme: &Theme, status: iced_text_input::Status) -> iced_text_input::Style {
        let mut style = text_input_style::default(theme, status);

        style.background = Background::Color(Color::TRANSPARENT);
        style.border.width = 0.0;
        style.border.color = Color::TRANSPARENT;
        style.placeholder = Color::TRANSPARENT;

        style
    }

    fn status_style(
        theme: &Theme,
        is_enabled: bool,
        is_focused: bool,
        is_hovered: bool,
    ) -> (Color, f32, Color) {
        let colors = theme.colors();

        if !is_enabled {
            (
                alpha_color(
                    colors.surface.text,
                    tokens::component::text_field::DISABLED_OUTLINE_OPACITY,
                ),
                tokens::component::text_field::OUTLINE_WIDTH,
                alpha_color(
                    colors.surface.text,
                    tokens::component::text_field::DISABLED_LABEL_TEXT_OPACITY,
                ),
            )
        } else if is_focused {
            (
                colors.primary.color,
                tokens::component::text_field::FOCUS_OUTLINE_WIDTH,
                colors.primary.color,
            )
        } else if is_hovered {
            (
                colors.surface.text,
                tokens::component::text_field::HOVER_OUTLINE_WIDTH,
                colors.surface.text,
            )
        } else {
            (
                colors.outline.color,
                tokens::component::text_field::OUTLINE_WIDTH,
                colors.surface.text_variant,
            )
        }
    }

    fn draw_outline<Renderer>(
        renderer: &mut Renderer,
        bounds: Rectangle,
        color: Color,
        width: f32,
        label_width: f32,
        float_progress: f32,
        notch_background: Color,
    ) where
        Renderer: iced_widget::core::Renderer,
    {
        if width <= 0.0 {
            return;
        }

        let radius = tokens::component::text_field::CONTAINER_SHAPE;

        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: Border {
                    color,
                    width,
                    radius: radius.into(),
                },
                ..renderer::Quad::default()
            },
            Color::TRANSPARENT,
        );

        if float_progress <= 0.01 {
            return;
        }

        let left = bounds.x;
        let right = bounds.x + bounds.width;
        let top = bounds.y;
        let notch_center = left + tokens::component::text_field::LEADING_SPACE + label_width / 2.0;
        let notch_width = (label_width
            + tokens::component::text_field::OUTLINE_LABEL_PADDING * 2.0)
            * float_progress.clamp(0.0, 1.0);
        let notch_start = (notch_center - notch_width / 2.0).clamp(left, right);
        let notch_end = (notch_center + notch_width / 2.0).clamp(left, right);

        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle {
                    x: notch_start,
                    y: top,
                    width: notch_end - notch_start,
                    height: width.ceil() + 1.0,
                },
                ..renderer::Quad::default()
            },
            notch_background,
        );
    }

    /// A Material 3 outlined text field with an animated floating label.
    pub struct TextInput<'a, Message, Renderer = iced_widget::Renderer>
    where
        Renderer: iced_widget::core::Renderer + core_text::Renderer,
    {
        label: text::Fragment<'a>,
        is_populated: bool,
        is_enabled: bool,
        width: Length,
        font: Option<Renderer::Font>,
        label_mode: LabelMode,
        input: IcedTextInput<'a, Message, Theme, Renderer>,
    }

    impl<Message, Renderer> std::fmt::Debug for TextInput<'_, Message, Renderer>
    where
        Renderer: iced_widget::core::Renderer + core_text::Renderer,
    {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("TextInput")
                .field("is_populated", &self.is_populated)
                .field("is_enabled", &self.is_enabled)
                .field("width", &self.width)
                .field("label_mode", &self.label_mode)
                .finish_non_exhaustive()
        }
    }

    impl<'a, Message, Renderer> TextInput<'a, Message, Renderer>
    where
        Message: Clone + 'a,
        Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
    {
        pub fn new(label: impl text::IntoFragment<'a>, value: &str) -> Self {
            Self::with_mode(label, value, LabelMode::Floating)
        }

        fn placeholder(label: impl text::IntoFragment<'a>, value: &str) -> Self {
            Self::with_mode(label, value, LabelMode::Placeholder)
        }

        fn with_mode(
            label: impl text::IntoFragment<'a>,
            value: &str,
            label_mode: LabelMode,
        ) -> Self {
            let input = IcedTextInput::new("", value)
                .width(Length::Fill)
                .padding(Padding {
                    top: tokens::component::text_field::TOP_SPACE,
                    right: tokens::component::text_field::TRAILING_SPACE,
                    bottom: tokens::component::text_field::BOTTOM_SPACE,
                    left: tokens::component::text_field::LEADING_SPACE,
                })
                .size(tokens::component::text_field::INPUT_TEXT_SIZE)
                .line_height(absolute_line_height(
                    tokens::component::text_field::INPUT_TEXT_LINE_HEIGHT,
                ))
                .style(input_layer_style);

            Self {
                label: label.into_fragment(),
                is_populated: !value.is_empty(),
                is_enabled: false,
                width: Length::Fill,
                font: None,
                label_mode,
                input,
            }
        }

        pub fn id(mut self, id: impl Into<core_widget::Id>) -> Self {
            self.input = self.input.id(id);
            self
        }

        pub fn secure(mut self, is_secure: bool) -> Self {
            self.input = self.input.secure(is_secure);
            self
        }

        pub fn on_input(mut self, on_input: impl Fn(String) -> Message + 'a) -> Self {
            self.is_enabled = true;
            self.input = self.input.on_input(on_input);
            self
        }

        pub fn on_input_maybe(mut self, on_input: Option<impl Fn(String) -> Message + 'a>) -> Self {
            self.is_enabled = on_input.is_some();
            self.input = self.input.on_input_maybe(on_input);
            self
        }

        pub fn on_submit(mut self, message: Message) -> Self {
            self.input = self.input.on_submit(message);
            self
        }

        pub fn on_paste(mut self, on_paste: impl Fn(String) -> Message + 'a) -> Self {
            self.input = self.input.on_paste(on_paste);
            self
        }

        pub fn width(mut self, width: impl Into<Length>) -> Self {
            self.width = width.into();
            self.input = self.input.width(self.width);
            self
        }

        pub fn font(mut self, font: impl Into<Renderer::Font>) -> Self {
            let font = font.into();
            self.font = Some(font);
            self.input = self.input.font(font);
            self
        }
    }

    impl<Message, Renderer> Widget<Message, Theme, Renderer> for TextInput<'_, Message, Renderer>
    where
        Message: Clone,
        Renderer: iced_widget::core::Renderer + core_text::Renderer,
    {
        fn tag(&self) -> tree::Tag {
            tree::Tag::of::<TextFieldState<Renderer::Paragraph>>()
        }

        fn state(&self) -> tree::State {
            tree::State::new(TextFieldState::<Renderer::Paragraph>::new(
                self.label_mode == LabelMode::Floating && self.is_populated,
            ))
        }

        fn children(&self) -> Vec<Tree> {
            let input: &dyn Widget<Message, Theme, Renderer> = &self.input;

            vec![Tree::new(input)]
        }

        fn diff(&self, tree: &mut Tree) {
            let state = tree
                .state
                .downcast_mut::<TextFieldState<Renderer::Paragraph>>();

            let target = if self.label_mode == LabelMode::Floating {
                bool_value(self.is_populated || state.is_focused)
            } else {
                0.0
            };

            state.label_float.set_target(
                target,
                Instant::now(),
                duration_ms(tokens::component::text_field::LABEL_TRANSITION_DURATION_MS),
                tokens::component::text_field::LABEL_TRANSITION_EASING,
            );

            if tree.children.is_empty() {
                tree.children = self.children();
            } else {
                self.input.diff(&mut tree.children[0]);
                tree.children.truncate(1);
            }
        }

        fn size(&self) -> Size<Length> {
            Size {
                width: self.width,
                height: Length::Fixed(tokens::component::text_field::CONTAINER_HEIGHT),
            }
        }

        fn layout(
            &mut self,
            tree: &mut Tree,
            renderer: &Renderer,
            limits: &layout::Limits,
        ) -> layout::Node {
            let state = tree
                .state
                .downcast_mut::<TextFieldState<Renderer::Paragraph>>();

            let label_size = tokens::component::text_field::LABEL_TEXT_SIZE;
            let label_line_height = tokens::component::text_field::LABEL_TEXT_LINE_HEIGHT;

            let label_node = core_widget::text::layout(
                &mut state.label,
                renderer,
                &layout::Limits::NONE,
                self.label.as_ref(),
                core_widget::text::Format {
                    width: Length::Shrink,
                    height: Length::Shrink,
                    line_height: absolute_line_height(label_line_height),
                    size: Some(Pixels(label_size)),
                    font: self.font,
                    align_x: text::Alignment::Default,
                    align_y: alignment::Vertical::Top,
                    shaping: text::Shaping::Advanced,
                    wrapping: text::Wrapping::None,
                },
            );

            let intrinsic = Size::new(
                label_node.size().width
                    + tokens::component::text_field::LEADING_SPACE
                    + tokens::component::text_field::TRAILING_SPACE,
                tokens::component::text_field::CONTAINER_HEIGHT,
            );
            let size = limits.resolve(
                self.width,
                Length::Fixed(tokens::component::text_field::CONTAINER_HEIGHT),
                intrinsic,
            );
            let child_limits = layout::Limits::new(size, size);
            let input = <IcedTextInput<'_, Message, Theme, Renderer> as Widget<
                Message,
                Theme,
                Renderer,
            >>::layout(
                &mut self.input,
                &mut tree.children[0],
                renderer,
                &child_limits,
            );

            layout::Node::with_children(size, vec![input.move_to(Point::ORIGIN)])
        }

        fn operate(
            &mut self,
            tree: &mut Tree,
            layout: Layout<'_>,
            renderer: &Renderer,
            operation: &mut dyn core_widget::Operation,
        ) {
            operation.text(None, layout.bounds(), self.label.as_ref());
            operation.traverse(&mut |operation| {
                self.input.operate(
                    &mut tree.children[0],
                    layout.children().next().unwrap(),
                    renderer,
                    operation,
                );
            });
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
            let bounds = layout.bounds();
            let state = tree
                .state
                .downcast_mut::<TextFieldState<Renderer::Paragraph>>();

            match event {
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                    state.is_focused = self.is_enabled && cursor.is_over(bounds);
                }
                Event::Touch(touch::Event::FingerPressed { position, .. }) => {
                    state.is_focused = self.is_enabled && bounds.contains(*position);
                }
                Event::Keyboard(iced_widget::core::keyboard::Event::KeyPressed { key, .. })
                    if matches!(
                        key.as_ref(),
                        iced_widget::core::keyboard::Key::Named(
                            iced_widget::core::keyboard::key::Named::Escape
                        )
                    ) =>
                {
                    state.is_focused = false;
                }
                Event::Window(window::Event::RedrawRequested(now)) => {
                    if state.label_float.advance(*now) {
                        shell.request_redraw();
                    }
                }
                _ => {}
            }

            let target = if self.label_mode == LabelMode::Floating {
                bool_value(self.is_populated || state.is_focused)
            } else {
                0.0
            };
            state.label_float.set_target(
                target,
                Instant::now(),
                duration_ms(tokens::component::text_field::LABEL_TRANSITION_DURATION_MS),
                tokens::component::text_field::LABEL_TRANSITION_EASING,
            );

            if state.is_animating() {
                shell.request_redraw();
            }

            self.input.update(
                &mut tree.children[0],
                event,
                layout.children().next().unwrap(),
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
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
            self.input.mouse_interaction(
                &tree.children[0],
                layout.children().next().unwrap(),
                cursor,
                viewport,
                renderer,
            )
        }

        fn draw(
            &self,
            tree: &Tree,
            renderer: &mut Renderer,
            theme: &Theme,
            defaults: &renderer::Style,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
            viewport: &Rectangle,
        ) {
            let state = tree
                .state
                .downcast_ref::<TextFieldState<Renderer::Paragraph>>();
            let bounds = layout.bounds();
            let progress = if self.label_mode == LabelMode::Floating {
                state.label_float.value.clamp(0.0, 1.0)
            } else {
                0.0
            };
            let is_hovered = cursor.is_over(bounds);
            let (outline_color, outline_width, label_color) =
                status_style(theme, self.is_enabled, state.is_focused, is_hovered);
            let label_width = state.label.raw().min_bounds().width;
            let label_line_height = tokens::component::text_field::LABEL_TEXT_LINE_HEIGHT;

            draw_outline(
                renderer,
                bounds,
                outline_color,
                outline_width,
                label_width,
                progress,
                theme.colors().surface.container.high,
            );

            <IcedTextInput<'_, Message, Theme, Renderer> as Widget<Message, Theme, Renderer>>::draw(
                &self.input,
                &tree.children[0],
                renderer,
                theme,
                defaults,
                layout.children().next().unwrap(),
                cursor,
                viewport,
            );

            if self.label_mode == LabelMode::Placeholder && self.is_populated {
                return;
            }

            let floating_label_y = -label_line_height / 2.0;
            let label_y = bounds.y
                + lerp(
                    tokens::component::text_field::TOP_SPACE,
                    floating_label_y,
                    progress,
                );
            let label_x = bounds.x + tokens::component::text_field::LEADING_SPACE;

            core_widget::text::draw(
                renderer,
                defaults,
                Rectangle {
                    x: label_x,
                    y: label_y,
                    width: label_width,
                    height: label_line_height,
                },
                state.label.raw(),
                core_widget::text::Style {
                    color: Some(label_color),
                },
                viewport,
            );
        }
    }

    impl<'a, Message, Renderer> From<TextInput<'a, Message, Renderer>>
        for Element<'a, Message, Theme, Renderer>
    where
        Message: Clone + 'a,
        Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
    {
        fn from(text_input: TextInput<'a, Message, Renderer>) -> Self {
            Element::new(text_input)
        }
    }

    pub fn outlined<'a, Message, Renderer>(
        label: impl text::IntoFragment<'a>,
        value: &str,
    ) -> TextInput<'a, Message, Renderer>
    where
        Message: Clone + 'a,
        Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
    {
        TextInput::new(label, value)
    }

    pub fn outlined_floating<'a, Message, Renderer>(
        label: impl text::IntoFragment<'a>,
        value: &str,
    ) -> TextInput<'a, Message, Renderer>
    where
        Message: Clone + 'a,
        Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
    {
        TextInput::new(label, value)
    }

    pub fn outlined_placeholder<'a, Message, Renderer>(
        label: impl text::IntoFragment<'a>,
        value: &str,
    ) -> TextInput<'a, Message, Renderer>
    where
        Message: Clone + 'a,
        Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
    {
        TextInput::placeholder(label, value)
    }
}

pub mod radio {
    //! Material 3 radio constructors with token-backed size and motion defaults.

    use super::*;

    type StyleFn<'a> = Box<dyn Fn(&Theme, iced_radio::Status) -> iced_radio::Style + 'a>;

    /// A Material 3 radio button with animated selected state transitions.
    pub struct Radio<'a, Message, Renderer = iced_widget::Renderer>
    where
        Renderer: core_text::Renderer,
    {
        is_selected: bool,
        on_click: Option<Message>,
        label: String,
        width: Length,
        size: f32,
        spacing: f32,
        text_size: Option<Pixels>,
        text_line_height: LineHeight,
        text_shaping: text::Shaping,
        text_wrapping: text::Wrapping,
        font: Option<Renderer::Font>,
        style: StyleFn<'a>,
    }

    impl<Message, Renderer> std::fmt::Debug for Radio<'_, Message, Renderer>
    where
        Message: std::fmt::Debug,
        Renderer: core_text::Renderer,
    {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("Radio")
                .field("is_selected", &self.is_selected)
                .field("on_click", &self.on_click)
                .field("label", &self.label)
                .field("width", &self.width)
                .field("size", &self.size)
                .field("spacing", &self.spacing)
                .field("text_size", &self.text_size)
                .field("text_line_height", &self.text_line_height)
                .finish_non_exhaustive()
        }
    }

    impl<'a, Message, Renderer> Radio<'a, Message, Renderer>
    where
        Message: Clone,
        Renderer: core_text::Renderer,
    {
        pub fn new<F, V>(
            label: impl Into<String>,
            value: V,
            selected: Option<V>,
            on_select: F,
        ) -> Self
        where
            V: Eq + Copy,
            F: FnOnce(V) -> Message,
        {
            Self {
                is_selected: Some(value) == selected,
                on_click: Some(on_select(value)),
                label: label.into(),
                width: Length::Shrink,
                size: tokens::component::radio::ICON_SIZE,
                spacing: f32::from(tokens::component::divider::LIST_ITEM_LEADING_SPACE),
                text_size: Some(Pixels(tokens::component::radio::LABEL_TEXT_SIZE)),
                text_line_height: absolute_line_height(
                    tokens::component::radio::LABEL_TEXT_LINE_HEIGHT,
                ),
                text_shaping: text::Shaping::default(),
                text_wrapping: text::Wrapping::default(),
                font: None,
                style: Box::new(crate::radio::default),
            }
        }

        pub fn on_select_maybe(mut self, on_click: Option<Message>) -> Self {
            self.on_click = on_click;
            self
        }

        pub fn size(mut self, size: impl Into<Pixels>) -> Self {
            self.size = size.into().0;
            self
        }

        pub fn width(mut self, width: impl Into<Length>) -> Self {
            self.width = width.into();
            self
        }

        pub fn spacing(mut self, spacing: impl Into<Pixels>) -> Self {
            self.spacing = spacing.into().0;
            self
        }

        pub fn text_size(mut self, text_size: impl Into<Pixels>) -> Self {
            self.text_size = Some(text_size.into());
            self
        }

        pub fn text_line_height(mut self, line_height: impl Into<LineHeight>) -> Self {
            self.text_line_height = line_height.into();
            self
        }

        pub fn text_shaping(mut self, shaping: text::Shaping) -> Self {
            self.text_shaping = shaping;
            self
        }

        pub fn text_wrapping(mut self, wrapping: text::Wrapping) -> Self {
            self.text_wrapping = wrapping;
            self
        }

        pub fn font(mut self, font: impl Into<Renderer::Font>) -> Self {
            self.font = Some(font.into());
            self
        }

        pub fn style(
            mut self,
            style: impl Fn(&Theme, iced_radio::Status) -> iced_radio::Style + 'a,
        ) -> Self {
            self.style = Box::new(style);
            self
        }
    }

    impl<Message, Renderer> Radio<'_, Message, Renderer>
    where
        Renderer: core_text::Renderer,
    {
        fn current_status(&self, bounds: Rectangle, cursor: mouse::Cursor) -> iced_radio::Status {
            if cursor.is_over(bounds) && self.on_click.is_some() {
                iced_radio::Status::Hovered {
                    is_selected: self.is_selected,
                }
            } else {
                iced_radio::Status::Active {
                    is_selected: self.is_selected,
                }
            }
        }

        fn state_layer_color(
            &self,
            theme: &Theme,
            state: &SelectionState<Renderer::Paragraph, iced_radio::Status>,
            status: iced_radio::Status,
        ) -> Option<Color> {
            let is_hovered = matches!(status, iced_radio::Status::Hovered { .. });

            if !state.is_pressed && !is_hovered {
                return None;
            }

            let colors = theme.colors();
            let (color, opacity) = if self.is_selected {
                if state.is_pressed {
                    (
                        colors.surface.text,
                        tokens::state::PRESSED_STATE_LAYER_OPACITY,
                    )
                } else {
                    (
                        colors.primary.color,
                        tokens::state::HOVER_STATE_LAYER_OPACITY,
                    )
                }
            } else if state.is_pressed {
                (
                    colors.primary.color,
                    tokens::state::PRESSED_STATE_LAYER_OPACITY,
                )
            } else {
                (
                    colors.surface.text,
                    tokens::state::HOVER_STATE_LAYER_OPACITY,
                )
            };

            Some(alpha_color(color, opacity))
        }
    }

    impl<Message, Renderer> Widget<Message, Theme, Renderer> for Radio<'_, Message, Renderer>
    where
        Message: Clone,
        Renderer: core_text::Renderer,
    {
        fn tag(&self) -> tree::Tag {
            tree::Tag::of::<SelectionState<Renderer::Paragraph, iced_radio::Status>>()
        }

        fn state(&self) -> tree::State {
            let mut state =
                SelectionState::<Renderer::Paragraph, iced_radio::Status>::new(self.is_selected);

            state.size = AnimatedScalar::new(bool_value(self.is_selected));

            tree::State::new(state)
        }

        fn size(&self) -> Size<Length> {
            Size {
                width: self.width,
                height: Length::Shrink,
            }
        }

        fn layout(
            &mut self,
            tree: &mut Tree,
            renderer: &Renderer,
            limits: &layout::Limits,
        ) -> layout::Node {
            layout::next_to_each_other(
                &limits.width(self.width),
                self.spacing,
                |_| layout::Node::new(Size::new(self.size, self.size)),
                |limits| {
                    let state = tree
                        .state
                        .downcast_mut::<SelectionState<Renderer::Paragraph, iced_radio::Status>>();

                    core_widget::text::layout(
                        &mut state.text,
                        renderer,
                        limits,
                        &self.label,
                        core_widget::text::Format {
                            width: self.width,
                            height: Length::Shrink,
                            line_height: self.text_line_height,
                            size: self.text_size,
                            font: self.font,
                            align_x: text::Alignment::Default,
                            align_y: alignment::Vertical::Top,
                            shaping: self.text_shaping,
                            wrapping: self.text_wrapping,
                        },
                    )
                },
            )
        }

        fn update(
            &mut self,
            tree: &mut Tree,
            event: &Event,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
            _renderer: &Renderer,
            _clipboard: &mut dyn Clipboard,
            shell: &mut Shell<'_, Message>,
            _viewport: &Rectangle,
        ) {
            let state = tree
                .state
                .downcast_mut::<SelectionState<Renderer::Paragraph, iced_radio::Status>>();

            match event {
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                | Event::Touch(touch::Event::FingerPressed { .. }) => {
                    if cursor.is_over(layout.bounds()) {
                        state.is_pressed = true;

                        if let Some(on_click) = &self.on_click {
                            shell.publish(on_click.clone());
                            shell.capture_event();
                        }

                        shell.request_redraw();
                    }
                }
                Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                | Event::Touch(touch::Event::FingerLifted { .. })
                | Event::Touch(touch::Event::FingerLost { .. }) => {
                    if state.is_pressed {
                        state.is_pressed = false;
                        shell.request_redraw();
                    }
                }
                _ => {}
            }

            let now = match event {
                Event::Window(window::Event::RedrawRequested(now)) => Some(*now),
                _ => None,
            };

            if state.target != self.is_selected {
                let now = now.unwrap_or_else(Instant::now);

                state.target = self.is_selected;
                state.color.set_target(
                    bool_value(self.is_selected),
                    now,
                    duration_ms(tokens::component::radio::ICON_COLOR_TRANSITION_DURATION_MS),
                    tokens::motion::EASING_LINEAR,
                );

                if self.is_selected {
                    state.size = AnimatedScalar::new(0.0);
                    state.size.set_target(
                        1.0,
                        now,
                        duration_ms(tokens::component::radio::SELECT_TRANSITION_DURATION_MS),
                        tokens::component::radio::SELECT_TRANSITION_EASING,
                    );
                } else {
                    state.size = AnimatedScalar::new(1.0);
                }

                shell.request_redraw();
            }

            let current_status = self.current_status(layout.bounds(), cursor);

            if let Some(now) = now {
                if state.advance(now) {
                    shell.request_redraw();
                }

                state.last_status = Some(current_status);
            } else if state
                .last_status
                .is_some_and(|status| status != current_status)
                || state.is_animating()
            {
                shell.request_redraw();
            }
        }

        fn mouse_interaction(
            &self,
            _tree: &Tree,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
            _viewport: &Rectangle,
            _renderer: &Renderer,
        ) -> mouse::Interaction {
            if cursor.is_over(layout.bounds()) {
                if self.on_click.is_some() {
                    mouse::Interaction::Pointer
                } else {
                    mouse::Interaction::NotAllowed
                }
            } else {
                mouse::Interaction::default()
            }
        }

        fn draw(
            &self,
            tree: &Tree,
            renderer: &mut Renderer,
            theme: &Theme,
            defaults: &renderer::Style,
            layout: Layout<'_>,
            _cursor: mouse::Cursor,
            viewport: &Rectangle,
        ) {
            let state = tree
                .state
                .downcast_ref::<SelectionState<Renderer::Paragraph, iced_radio::Status>>();

            let mut children = layout.children();
            let control_layout = children.next().unwrap();
            let bounds = control_layout.bounds();
            let status = state.last_status.unwrap_or(iced_radio::Status::Active {
                is_selected: self.is_selected,
            });
            let unchecked_status = match status {
                iced_radio::Status::Active { .. } => {
                    iced_radio::Status::Active { is_selected: false }
                }
                iced_radio::Status::Hovered { .. } => {
                    iced_radio::Status::Hovered { is_selected: false }
                }
            };
            let checked_status = match status {
                iced_radio::Status::Active { .. } => {
                    iced_radio::Status::Active { is_selected: true }
                }
                iced_radio::Status::Hovered { .. } => {
                    iced_radio::Status::Hovered { is_selected: true }
                }
            };
            let current_style = (self.style)(theme, status);
            let unchecked_style = (self.style)(theme, unchecked_status);
            let checked_style = (self.style)(theme, checked_status);

            if let Some(layer_color) = self.state_layer_color(theme, state, status) {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: scaled_rect(
                            bounds,
                            tokens::component::radio::STATE_LAYER_SIZE,
                            tokens::component::radio::STATE_LAYER_SIZE,
                        ),
                        border: border::rounded(tokens::component::radio::STATE_LAYER_SIZE / 2.0),
                        ..renderer::Quad::default()
                    },
                    Background::Color(layer_color),
                );
            }

            let color_progress = state.color.value.clamp(0.0, 1.0);
            let icon_color = mix(
                unchecked_style.border_color,
                checked_style.border_color,
                color_progress,
            );
            let radius = bounds.width.min(bounds.height) / 2.0;

            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border: Border {
                        radius: radius.into(),
                        width: tokens::component::radio::OUTER_RING_WIDTH,
                        color: icon_color,
                    },
                    ..renderer::Quad::default()
                },
                Color::TRANSPARENT,
            );

            if color_progress > 0.0 {
                let dot_size =
                    tokens::component::radio::INNER_DOT_SIZE * state.size.value.clamp(0.0, 1.0);

                if dot_size > 0.0 {
                    renderer.fill_quad(
                        renderer::Quad {
                            bounds: scaled_rect(bounds, dot_size, dot_size),
                            border: border::rounded(dot_size / 2.0),
                            ..renderer::Quad::default()
                        },
                        Background::Color(alpha_color(checked_style.dot_color, color_progress)),
                    );
                }
            }

            let label_layout = children.next().unwrap();

            core_widget::text::draw(
                renderer,
                defaults,
                label_layout.bounds(),
                state.text.raw(),
                core_widget::text::Style {
                    color: current_style.text_color,
                },
                viewport,
            );
        }

        fn operate(
            &mut self,
            _tree: &mut Tree,
            layout: Layout<'_>,
            _renderer: &Renderer,
            operation: &mut dyn core_widget::Operation,
        ) {
            operation.text(None, layout.bounds(), &self.label);
        }
    }

    impl<'a, Message, Renderer> From<Radio<'a, Message, Renderer>>
        for Element<'a, Message, Theme, Renderer>
    where
        Message: Clone + 'a,
        Renderer: core_text::Renderer + 'a,
    {
        fn from(radio: Radio<'a, Message, Renderer>) -> Self {
            Element::new(radio)
        }
    }

    pub fn control<'a, Message, Renderer, V>(
        label: impl Into<String>,
        value: V,
        selected: Option<V>,
        on_select: impl FnOnce(V) -> Message,
    ) -> Radio<'a, Message, Renderer>
    where
        Message: Clone + 'a,
        Renderer: core_text::Renderer + 'a,
        V: Eq + Copy,
    {
        Radio::new(label, value, selected, on_select)
            .size(tokens::component::radio::ICON_SIZE)
            .spacing(f32::from(
                tokens::component::divider::LIST_ITEM_LEADING_SPACE,
            ))
            .text_size(tokens::component::radio::LABEL_TEXT_SIZE)
            .text_line_height(absolute_line_height(
                tokens::component::radio::LABEL_TEXT_LINE_HEIGHT,
            ))
            .style(crate::radio::default)
    }

    pub fn standard<'a, Message, Renderer, V>(
        label: impl Into<String>,
        value: V,
        selected: Option<V>,
        on_select: impl FnOnce(V) -> Message,
    ) -> Element<'a, Message, Theme, Renderer>
    where
        Message: Clone + 'a,
        Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
        V: Eq + Copy,
    {
        Container::new(control(label, value, selected, on_select))
            .center_y(Length::Fixed(tokens::component::radio::TARGET_SIZE))
            .into()
    }
}

pub mod checkbox {
    //! Material 3 checkbox constructors with token-backed size and motion defaults.

    use super::*;

    type StyleFn<'a> = Box<dyn Fn(&Theme, iced_checkbox::Status) -> iced_checkbox::Style + 'a>;

    /// A Material 3 checkbox with animated selected state transitions.
    pub struct Checkbox<'a, Message, Renderer = iced_widget::Renderer>
    where
        Renderer: core_text::Renderer,
    {
        is_checked: bool,
        on_toggle: Option<Box<dyn Fn(bool) -> Message + 'a>>,
        label: Option<text::Fragment<'a>>,
        width: Length,
        size: f32,
        spacing: f32,
        text_size: Option<Pixels>,
        text_line_height: LineHeight,
        text_shaping: text::Shaping,
        text_wrapping: text::Wrapping,
        font: Option<Renderer::Font>,
        style: StyleFn<'a>,
    }

    impl<Message, Renderer> std::fmt::Debug for Checkbox<'_, Message, Renderer>
    where
        Renderer: core_text::Renderer,
    {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("Checkbox")
                .field("is_checked", &self.is_checked)
                .field("has_on_toggle", &self.on_toggle.is_some())
                .field("has_label", &self.label.is_some())
                .field("width", &self.width)
                .field("size", &self.size)
                .field("spacing", &self.spacing)
                .field("text_size", &self.text_size)
                .field("text_line_height", &self.text_line_height)
                .finish_non_exhaustive()
        }
    }

    impl<'a, Message, Renderer> Checkbox<'a, Message, Renderer>
    where
        Renderer: core_text::Renderer,
    {
        pub fn new(is_checked: bool) -> Self {
            Self {
                is_checked,
                on_toggle: None,
                label: None,
                width: Length::Shrink,
                size: tokens::component::checkbox::CONTAINER_SIZE,
                spacing: f32::from(tokens::component::divider::LIST_ITEM_LEADING_SPACE),
                text_size: Some(Pixels(tokens::component::checkbox::LABEL_TEXT_SIZE)),
                text_line_height: absolute_line_height(
                    tokens::component::checkbox::LABEL_TEXT_LINE_HEIGHT,
                ),
                text_shaping: text::Shaping::default(),
                text_wrapping: text::Wrapping::default(),
                font: None,
                style: Box::new(checkbox_style::default),
            }
        }

        pub fn label(mut self, label: impl text::IntoFragment<'a>) -> Self {
            self.label = Some(label.into_fragment());
            self
        }

        pub fn on_toggle(mut self, on_toggle: impl Fn(bool) -> Message + 'a) -> Self {
            self.on_toggle = Some(Box::new(on_toggle));
            self
        }

        pub fn on_toggle_maybe(mut self, on_toggle: Option<impl Fn(bool) -> Message + 'a>) -> Self {
            self.on_toggle = on_toggle.map(|on_toggle| Box::new(on_toggle) as _);
            self
        }

        pub fn size(mut self, size: impl Into<Pixels>) -> Self {
            self.size = size.into().0;
            self
        }

        pub fn width(mut self, width: impl Into<Length>) -> Self {
            self.width = width.into();
            self
        }

        pub fn spacing(mut self, spacing: impl Into<Pixels>) -> Self {
            self.spacing = spacing.into().0;
            self
        }

        pub fn text_size(mut self, text_size: impl Into<Pixels>) -> Self {
            self.text_size = Some(text_size.into());
            self
        }

        pub fn text_line_height(mut self, line_height: impl Into<LineHeight>) -> Self {
            self.text_line_height = line_height.into();
            self
        }

        pub fn text_shaping(mut self, shaping: text::Shaping) -> Self {
            self.text_shaping = shaping;
            self
        }

        pub fn text_wrapping(mut self, wrapping: text::Wrapping) -> Self {
            self.text_wrapping = wrapping;
            self
        }

        pub fn font(mut self, font: impl Into<Renderer::Font>) -> Self {
            self.font = Some(font.into());
            self
        }

        pub fn style(
            mut self,
            style: impl Fn(&Theme, iced_checkbox::Status) -> iced_checkbox::Style + 'a,
        ) -> Self {
            self.style = Box::new(style);
            self
        }
    }

    impl<Message, Renderer> Checkbox<'_, Message, Renderer>
    where
        Renderer: core_text::Renderer,
    {
        fn current_status(
            &self,
            bounds: Rectangle,
            cursor: mouse::Cursor,
        ) -> iced_checkbox::Status {
            if self.on_toggle.is_none() {
                iced_checkbox::Status::Disabled {
                    is_checked: self.is_checked,
                }
            } else if cursor.is_over(bounds) {
                iced_checkbox::Status::Hovered {
                    is_checked: self.is_checked,
                }
            } else {
                iced_checkbox::Status::Active {
                    is_checked: self.is_checked,
                }
            }
        }
    }

    impl<Message, Renderer> Widget<Message, Theme, Renderer> for Checkbox<'_, Message, Renderer>
    where
        Renderer: core_text::Renderer + core_svg::Renderer,
    {
        fn tag(&self) -> tree::Tag {
            tree::Tag::of::<SelectionState<Renderer::Paragraph, iced_checkbox::Status>>()
        }

        fn state(&self) -> tree::State {
            tree::State::new(
                SelectionState::<Renderer::Paragraph, iced_checkbox::Status>::new(self.is_checked),
            )
        }

        fn size(&self) -> Size<Length> {
            Size {
                width: self.width,
                height: Length::Shrink,
            }
        }

        fn layout(
            &mut self,
            tree: &mut Tree,
            renderer: &Renderer,
            limits: &layout::Limits,
        ) -> layout::Node {
            layout::next_to_each_other(
                &limits.width(self.width),
                if self.label.is_some() {
                    self.spacing
                } else {
                    0.0
                },
                |_| layout::Node::new(Size::new(self.size, self.size)),
                |limits| {
                    if let Some(label) = self.label.as_deref() {
                        let state = tree
                            .state
                            .downcast_mut::<SelectionState<
                                Renderer::Paragraph,
                                iced_checkbox::Status,
                            >>();

                        core_widget::text::layout(
                            &mut state.text,
                            renderer,
                            limits,
                            label,
                            core_widget::text::Format {
                                width: self.width,
                                height: Length::Shrink,
                                line_height: self.text_line_height,
                                size: self.text_size,
                                font: self.font,
                                align_x: text::Alignment::Default,
                                align_y: alignment::Vertical::Top,
                                shaping: self.text_shaping,
                                wrapping: self.text_wrapping,
                            },
                        )
                    } else {
                        layout::Node::new(Size::ZERO)
                    }
                },
            )
        }

        fn update(
            &mut self,
            tree: &mut Tree,
            event: &Event,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
            _renderer: &Renderer,
            _clipboard: &mut dyn Clipboard,
            shell: &mut Shell<'_, Message>,
            _viewport: &Rectangle,
        ) {
            let state = tree
                .state
                .downcast_mut::<SelectionState<Renderer::Paragraph, iced_checkbox::Status>>();

            match event {
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                | Event::Touch(touch::Event::FingerPressed { .. }) => {
                    if cursor.is_over(layout.bounds()) {
                        if let Some(on_toggle) = &self.on_toggle {
                            shell.publish((on_toggle)(!self.is_checked));
                            shell.capture_event();
                            shell.request_redraw();
                        }
                    }
                }
                _ => {}
            }

            let now = match event {
                Event::Window(window::Event::RedrawRequested(now)) => Some(*now),
                _ => None,
            };

            if state.target != self.is_checked {
                let now = now.unwrap_or_else(Instant::now);
                let (duration, easing) = if self.is_checked {
                    (
                        duration_ms(tokens::component::checkbox::SELECT_TRANSITION_DURATION_MS),
                        tokens::component::checkbox::SELECT_TRANSITION_EASING,
                    )
                } else {
                    (
                        duration_ms(tokens::component::checkbox::UNSELECT_TRANSITION_DURATION_MS),
                        tokens::component::checkbox::UNSELECT_TRANSITION_EASING,
                    )
                };

                state.target = self.is_checked;
                state
                    .position
                    .set_target(bool_value(self.is_checked), now, duration, easing);
                state.color.set_target(
                    bool_value(self.is_checked),
                    now,
                    duration_ms(tokens::component::checkbox::OPACITY_TRANSITION_DURATION_MS),
                    tokens::motion::EASING_LINEAR,
                );
                state
                    .size
                    .set_target(bool_value(self.is_checked), now, duration, easing);
                shell.request_redraw();
            }

            let current_status = self.current_status(layout.bounds(), cursor);

            if let Some(now) = now {
                if state.advance(now) {
                    shell.request_redraw();
                }

                state.last_status = Some(current_status);
            } else if state
                .last_status
                .is_some_and(|status| status != current_status)
                || state.is_animating()
            {
                shell.request_redraw();
            }
        }

        fn mouse_interaction(
            &self,
            _tree: &Tree,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
            _viewport: &Rectangle,
            _renderer: &Renderer,
        ) -> mouse::Interaction {
            if cursor.is_over(layout.bounds()) && self.on_toggle.is_some() {
                mouse::Interaction::Pointer
            } else {
                mouse::Interaction::default()
            }
        }

        fn draw(
            &self,
            tree: &Tree,
            renderer: &mut Renderer,
            theme: &Theme,
            defaults: &renderer::Style,
            layout: Layout<'_>,
            _cursor: mouse::Cursor,
            viewport: &Rectangle,
        ) {
            let state = tree
                .state
                .downcast_ref::<SelectionState<Renderer::Paragraph, iced_checkbox::Status>>();

            let mut children = layout.children();
            let control_layout = children.next().unwrap();
            let bounds = control_layout.bounds();

            let status = state
                .last_status
                .unwrap_or(iced_checkbox::Status::Disabled {
                    is_checked: self.is_checked,
                });
            let unchecked_status = match status {
                iced_checkbox::Status::Active { .. } => {
                    iced_checkbox::Status::Active { is_checked: false }
                }
                iced_checkbox::Status::Hovered { .. } => {
                    iced_checkbox::Status::Hovered { is_checked: false }
                }
                iced_checkbox::Status::Disabled { .. } => {
                    iced_checkbox::Status::Disabled { is_checked: false }
                }
            };
            let checked_status = match status {
                iced_checkbox::Status::Active { .. } => {
                    iced_checkbox::Status::Active { is_checked: true }
                }
                iced_checkbox::Status::Hovered { .. } => {
                    iced_checkbox::Status::Hovered { is_checked: true }
                }
                iced_checkbox::Status::Disabled { .. } => {
                    iced_checkbox::Status::Disabled { is_checked: true }
                }
            };
            let current_style = (self.style)(theme, status);
            let unchecked_style = (self.style)(theme, unchecked_status);
            let checked_style = (self.style)(theme, checked_status);

            let selection = state.position.value.clamp(0.0, 1.0);
            let opacity = state.color.value.clamp(0.0, 1.0);
            let scale = 0.74 + 0.26 * state.size.value.clamp(0.0, 1.0);

            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border: alpha_border(unchecked_style.border, 1.0 - selection),
                    ..renderer::Quad::default()
                },
                unchecked_style.background.scale_alpha(1.0 - selection),
            );

            if selection > 0.0 {
                let selected_bounds =
                    scaled_rect(bounds, bounds.width * scale, bounds.height * scale);

                renderer.fill_quad(
                    renderer::Quad {
                        bounds: selected_bounds,
                        border: alpha_border(checked_style.border, selection),
                        ..renderer::Quad::default()
                    },
                    checked_style.background.scale_alpha(selection),
                );
            }

            if opacity > 0.0 {
                let icon_scale = 0.58 + 0.42 * selection;
                let icon_size = tokens::component::checkbox::ICON_SIZE * icon_scale;
                let mark_progress = if self.is_checked { selection } else { 1.0 };

                renderer.draw_svg(
                    core_svg::Svg::new(core_svg::Handle::from_memory(checkbox_checkmark_svg(
                        mark_progress,
                    )))
                    .color(checked_style.icon_color)
                    .opacity(opacity),
                    scaled_rect(bounds, icon_size, icon_size),
                    *viewport,
                );
            }

            if self.label.is_none() {
                return;
            }

            let label_layout = children.next().unwrap();

            core_widget::text::draw(
                renderer,
                defaults,
                label_layout.bounds(),
                state.text.raw(),
                core_widget::text::Style {
                    color: current_style.text_color,
                },
                viewport,
            );
        }

        fn operate(
            &mut self,
            _tree: &mut Tree,
            layout: Layout<'_>,
            _renderer: &Renderer,
            operation: &mut dyn core_widget::Operation,
        ) {
            if let Some(label) = self.label.as_deref() {
                operation.text(None, layout.bounds(), label);
            }
        }
    }

    impl<'a, Message, Renderer> From<Checkbox<'a, Message, Renderer>>
        for Element<'a, Message, Theme, Renderer>
    where
        Message: 'a,
        Renderer: core_text::Renderer + core_svg::Renderer + 'a,
    {
        fn from(checkbox: Checkbox<'a, Message, Renderer>) -> Self {
            Element::new(checkbox)
        }
    }

    pub fn control<'a, Message, Renderer>(is_checked: bool) -> Checkbox<'a, Message, Renderer>
    where
        Renderer: core_text::Renderer + core_svg::Renderer + 'a,
    {
        Checkbox::new(is_checked)
            .size(tokens::component::checkbox::CONTAINER_SIZE)
            .spacing(f32::from(
                tokens::component::divider::LIST_ITEM_LEADING_SPACE,
            ))
            .text_size(tokens::component::checkbox::LABEL_TEXT_SIZE)
            .text_line_height(absolute_line_height(
                tokens::component::checkbox::LABEL_TEXT_LINE_HEIGHT,
            ))
            .style(checkbox_style::default)
    }

    pub fn standard<'a, Message, Renderer>(
        is_checked: bool,
        label: impl text::IntoFragment<'a>,
        on_toggle: impl Fn(bool) -> Message + 'a,
    ) -> Element<'a, Message, Theme, Renderer>
    where
        Message: 'a,
        Renderer: iced_widget::core::Renderer + core_text::Renderer + core_svg::Renderer + 'a,
    {
        Container::new(control(is_checked).label(label).on_toggle(on_toggle))
            .center_y(Length::Fixed(tokens::component::checkbox::STATE_LAYER_SIZE))
            .into()
    }
}

pub mod toggler {
    //! Material 3 switch/toggler constructors with token-backed size and motion defaults.

    use super::*;

    type StyleFn<'a> = Box<dyn Fn(&Theme, iced_toggler::Status) -> iced_toggler::Style + 'a>;

    /// A Material 3 switch with animated handle motion and color transitions.
    pub struct Toggler<'a, Message, Renderer = iced_widget::Renderer>
    where
        Renderer: core_text::Renderer,
    {
        is_toggled: bool,
        on_toggle: Option<Box<dyn Fn(bool) -> Message + 'a>>,
        label: Option<text::Fragment<'a>>,
        width: Length,
        track_height: f32,
        spacing: f32,
        text_size: Option<Pixels>,
        text_line_height: LineHeight,
        text_alignment: text::Alignment,
        text_shaping: text::Shaping,
        text_wrapping: text::Wrapping,
        font: Option<Renderer::Font>,
        icons: bool,
        show_only_selected_icon: bool,
        style: StyleFn<'a>,
    }

    impl<Message, Renderer> std::fmt::Debug for Toggler<'_, Message, Renderer>
    where
        Renderer: core_text::Renderer,
    {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("Toggler")
                .field("is_toggled", &self.is_toggled)
                .field("has_on_toggle", &self.on_toggle.is_some())
                .field("has_label", &self.label.is_some())
                .field("width", &self.width)
                .field("track_height", &self.track_height)
                .field("spacing", &self.spacing)
                .field("text_size", &self.text_size)
                .field("text_line_height", &self.text_line_height)
                .field("text_alignment", &self.text_alignment)
                .field("icons", &self.icons)
                .field("show_only_selected_icon", &self.show_only_selected_icon)
                .finish_non_exhaustive()
        }
    }

    impl<'a, Message, Renderer> Toggler<'a, Message, Renderer>
    where
        Renderer: core_text::Renderer,
    {
        pub fn new(is_toggled: bool) -> Self {
            Self {
                is_toggled,
                on_toggle: None,
                label: None,
                width: Length::Shrink,
                track_height: tokens::component::switch::TRACK_HEIGHT,
                spacing: f32::from(tokens::component::divider::LIST_ITEM_LEADING_SPACE),
                text_size: Some(Pixels(tokens::component::switch::LABEL_TEXT_SIZE)),
                text_line_height: absolute_line_height(
                    tokens::component::switch::LABEL_TEXT_LINE_HEIGHT,
                ),
                text_alignment: text::Alignment::Default,
                text_shaping: text::Shaping::default(),
                text_wrapping: text::Wrapping::default(),
                font: None,
                icons: false,
                show_only_selected_icon: false,
                style: Box::new(toggler_style::default),
            }
        }

        pub fn label(mut self, label: impl text::IntoFragment<'a>) -> Self {
            self.label = Some(label.into_fragment());
            self
        }

        pub fn on_toggle(mut self, on_toggle: impl Fn(bool) -> Message + 'a) -> Self {
            self.on_toggle = Some(Box::new(on_toggle));
            self
        }

        pub fn on_toggle_maybe(mut self, on_toggle: Option<impl Fn(bool) -> Message + 'a>) -> Self {
            self.on_toggle = on_toggle.map(|on_toggle| Box::new(on_toggle) as _);
            self
        }

        pub fn size(mut self, size: impl Into<Pixels>) -> Self {
            self.track_height = size.into().0;
            self
        }

        pub fn width(mut self, width: impl Into<Length>) -> Self {
            self.width = width.into();
            self
        }

        pub fn spacing(mut self, spacing: impl Into<Pixels>) -> Self {
            self.spacing = spacing.into().0;
            self
        }

        pub fn text_size(mut self, text_size: impl Into<Pixels>) -> Self {
            self.text_size = Some(text_size.into());
            self
        }

        pub fn text_line_height(mut self, line_height: impl Into<LineHeight>) -> Self {
            self.text_line_height = line_height.into();
            self
        }

        pub fn text_alignment(mut self, alignment: impl Into<text::Alignment>) -> Self {
            self.text_alignment = alignment.into();
            self
        }

        pub fn text_shaping(mut self, shaping: text::Shaping) -> Self {
            self.text_shaping = shaping;
            self
        }

        pub fn text_wrapping(mut self, wrapping: text::Wrapping) -> Self {
            self.text_wrapping = wrapping;
            self
        }

        pub fn font(mut self, font: impl Into<Renderer::Font>) -> Self {
            self.font = Some(font.into());
            self
        }

        pub fn icons(mut self, icons: bool) -> Self {
            self.icons = icons;
            self
        }

        pub fn show_only_selected_icon(mut self, show_only_selected_icon: bool) -> Self {
            self.show_only_selected_icon = show_only_selected_icon;
            self
        }

        pub fn style(
            mut self,
            style: impl Fn(&Theme, iced_toggler::Status) -> iced_toggler::Style + 'a,
        ) -> Self {
            self.style = Box::new(style);
            self
        }
    }

    impl<Message, Renderer> Toggler<'_, Message, Renderer>
    where
        Renderer: core_text::Renderer,
    {
        fn track_size(&self) -> Size {
            let scale = self.track_height / tokens::component::switch::TRACK_HEIGHT;

            Size::new(
                tokens::component::switch::TRACK_WIDTH * scale,
                self.track_height,
            )
        }

        fn handle_size_for(&self, is_toggled: bool, is_pressed: bool) -> f32 {
            if is_pressed {
                tokens::component::switch::PRESSED_HANDLE_SIZE
            } else if self.icons || (self.show_only_selected_icon && is_toggled) {
                tokens::component::switch::WITH_ICON_HANDLE_SIZE
            } else if is_toggled {
                tokens::component::switch::SELECTED_HANDLE_SIZE
            } else {
                tokens::component::switch::UNSELECTED_HANDLE_SIZE
            }
        }

        fn shows_icons(&self) -> bool {
            self.icons || self.show_only_selected_icon
        }

        fn shows_off_icon(&self) -> bool {
            self.icons && !self.show_only_selected_icon
        }

        fn current_status(&self, bounds: Rectangle, cursor: mouse::Cursor) -> iced_toggler::Status {
            if self.on_toggle.is_none() {
                iced_toggler::Status::Disabled {
                    is_toggled: self.is_toggled,
                }
            } else if cursor.is_over(bounds) {
                iced_toggler::Status::Hovered {
                    is_toggled: self.is_toggled,
                }
            } else {
                iced_toggler::Status::Active {
                    is_toggled: self.is_toggled,
                }
            }
        }
    }

    impl<Message, Renderer> Widget<Message, Theme, Renderer> for Toggler<'_, Message, Renderer>
    where
        Renderer: core_text::Renderer + core_svg::Renderer,
    {
        fn tag(&self) -> tree::Tag {
            tree::Tag::of::<SelectionState<Renderer::Paragraph, iced_toggler::Status>>()
        }

        fn state(&self) -> tree::State {
            let mut state =
                SelectionState::<Renderer::Paragraph, iced_toggler::Status>::new(self.is_toggled);

            state.size = AnimatedScalar::new(self.handle_size_for(self.is_toggled, false));

            tree::State::new(state)
        }

        fn size(&self) -> Size<Length> {
            Size {
                width: self.width,
                height: Length::Shrink,
            }
        }

        fn layout(
            &mut self,
            tree: &mut Tree,
            renderer: &Renderer,
            limits: &layout::Limits,
        ) -> layout::Node {
            let track_size = self.track_size();

            layout::next_to_each_other(
                &limits.width(self.width),
                if self.label.is_some() {
                    self.spacing
                } else {
                    0.0
                },
                |_| layout::Node::new(track_size),
                |limits| {
                    if let Some(label) = self.label.as_deref() {
                        let state = tree
                            .state
                            .downcast_mut::<SelectionState<
                                Renderer::Paragraph,
                                iced_toggler::Status,
                            >>();

                        core_widget::text::layout(
                            &mut state.text,
                            renderer,
                            limits,
                            label,
                            core_widget::text::Format {
                                width: self.width,
                                height: Length::Shrink,
                                line_height: self.text_line_height,
                                size: self.text_size,
                                font: self.font,
                                align_x: self.text_alignment,
                                align_y: alignment::Vertical::Top,
                                shaping: self.text_shaping,
                                wrapping: self.text_wrapping,
                            },
                        )
                    } else {
                        layout::Node::new(Size::ZERO)
                    }
                },
            )
        }

        fn update(
            &mut self,
            tree: &mut Tree,
            event: &Event,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
            _renderer: &Renderer,
            _clipboard: &mut dyn Clipboard,
            shell: &mut Shell<'_, Message>,
            _viewport: &Rectangle,
        ) {
            let state = tree
                .state
                .downcast_mut::<SelectionState<Renderer::Paragraph, iced_toggler::Status>>();

            match event {
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                | Event::Touch(touch::Event::FingerPressed { .. }) => {
                    if cursor.is_over(layout.bounds()) {
                        if let Some(on_toggle) = &self.on_toggle {
                            state.is_pressed = true;
                            shell.publish((on_toggle)(!self.is_toggled));
                            shell.capture_event();
                            shell.request_redraw();
                        }
                    }
                }
                Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                | Event::Touch(touch::Event::FingerLifted { .. })
                | Event::Touch(touch::Event::FingerLost { .. }) => {
                    if state.is_pressed {
                        state.is_pressed = false;
                        shell.request_redraw();
                    }
                }
                _ => {}
            }

            let now = match event {
                Event::Window(window::Event::RedrawRequested(now)) => Some(*now),
                _ => None,
            };

            if state.target != self.is_toggled {
                let now = now.unwrap_or_else(Instant::now);

                state.target = self.is_toggled;
                state.position.set_target(
                    bool_value(self.is_toggled),
                    now,
                    duration_ms(tokens::component::switch::HANDLE_POSITION_TRANSITION_DURATION_MS),
                    tokens::component::switch::HANDLE_POSITION_TRANSITION_EASING,
                );
                state.color.set_target(
                    bool_value(self.is_toggled),
                    now,
                    duration_ms(tokens::component::switch::TRACK_COLOR_TRANSITION_DURATION_MS),
                    tokens::motion::EASING_LINEAR,
                );
                state.size.set_target(
                    self.handle_size_for(self.is_toggled, state.is_pressed),
                    now,
                    if state.is_pressed {
                        duration_ms(
                            tokens::component::switch::PRESSED_HANDLE_SIZE_TRANSITION_DURATION_MS,
                        )
                    } else {
                        duration_ms(tokens::component::switch::HANDLE_SIZE_TRANSITION_DURATION_MS)
                    },
                    if state.is_pressed {
                        tokens::motion::EASING_LINEAR
                    } else {
                        tokens::motion::EASING_STANDARD
                    },
                );
                state.icon.set_target(
                    bool_value(self.is_toggled),
                    now,
                    duration_ms(tokens::component::switch::ICON_TRANSFORM_TRANSITION_DURATION_MS),
                    tokens::motion::EASING_STANDARD,
                );
                state.icon_opacity.set_target(
                    bool_value(self.is_toggled),
                    now,
                    duration_ms(tokens::component::switch::ICON_OPACITY_TRANSITION_DURATION_MS),
                    tokens::motion::EASING_LINEAR,
                );
                shell.request_redraw();
            }

            let target_handle_size = self.handle_size_for(self.is_toggled, state.is_pressed);

            if (state.size.to - target_handle_size).abs() > f32::EPSILON {
                let now = now.unwrap_or_else(Instant::now);

                state.size.set_target(
                    target_handle_size,
                    now,
                    if state.is_pressed {
                        duration_ms(
                            tokens::component::switch::PRESSED_HANDLE_SIZE_TRANSITION_DURATION_MS,
                        )
                    } else {
                        duration_ms(tokens::component::switch::HANDLE_SIZE_TRANSITION_DURATION_MS)
                    },
                    if state.is_pressed {
                        tokens::motion::EASING_LINEAR
                    } else {
                        tokens::motion::EASING_STANDARD
                    },
                );
                shell.request_redraw();
            }

            let current_status = self.current_status(layout.bounds(), cursor);

            if let Some(now) = now {
                if state.advance(now) {
                    shell.request_redraw();
                }

                state.last_status = Some(current_status);
            } else if state
                .last_status
                .is_some_and(|status| status != current_status)
                || state.is_animating()
            {
                shell.request_redraw();
            }
        }

        fn mouse_interaction(
            &self,
            _tree: &Tree,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
            _viewport: &Rectangle,
            _renderer: &Renderer,
        ) -> mouse::Interaction {
            if cursor.is_over(layout.bounds()) {
                if self.on_toggle.is_some() {
                    mouse::Interaction::Pointer
                } else {
                    mouse::Interaction::NotAllowed
                }
            } else {
                mouse::Interaction::default()
            }
        }

        fn draw(
            &self,
            tree: &Tree,
            renderer: &mut Renderer,
            theme: &Theme,
            defaults: &renderer::Style,
            layout: Layout<'_>,
            _cursor: mouse::Cursor,
            viewport: &Rectangle,
        ) {
            let state = tree
                .state
                .downcast_ref::<SelectionState<Renderer::Paragraph, iced_toggler::Status>>();

            let mut children = layout.children();
            let toggler_layout = children.next().unwrap();
            let bounds = toggler_layout.bounds();

            let status = state.last_status.unwrap_or(iced_toggler::Status::Disabled {
                is_toggled: self.is_toggled,
            });
            let unselected_status = match status {
                iced_toggler::Status::Active { .. } => {
                    iced_toggler::Status::Active { is_toggled: false }
                }
                iced_toggler::Status::Hovered { .. } => {
                    iced_toggler::Status::Hovered { is_toggled: false }
                }
                iced_toggler::Status::Disabled { .. } => {
                    iced_toggler::Status::Disabled { is_toggled: false }
                }
            };
            let selected_status = match status {
                iced_toggler::Status::Active { .. } => {
                    iced_toggler::Status::Active { is_toggled: true }
                }
                iced_toggler::Status::Hovered { .. } => {
                    iced_toggler::Status::Hovered { is_toggled: true }
                }
                iced_toggler::Status::Disabled { .. } => {
                    iced_toggler::Status::Disabled { is_toggled: true }
                }
            };

            let current_style = (self.style)(theme, status);
            let unselected_style = (self.style)(theme, unselected_status);
            let selected_style = (self.style)(theme, selected_status);

            let color_progress = state.color.value.clamp(0.0, 1.0);
            let scale = bounds.height / tokens::component::switch::TRACK_HEIGHT;
            let colors = theme.colors();
            let is_disabled = matches!(status, iced_toggler::Status::Disabled { .. });
            let track_radius = current_style
                .border_radius
                .unwrap_or_else(|| border::Radius::new(bounds.height / 2.0));
            let track_border_width = lerp(
                unselected_style.background_border_width,
                selected_style.background_border_width,
                color_progress,
            );
            let track_border_color = mix(
                unselected_style.background_border_color,
                selected_style.background_border_color,
                color_progress,
            );
            let track_background = mix(
                solid_color(unselected_style.background),
                solid_color(selected_style.background),
                color_progress,
            );

            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border: Border {
                        radius: track_radius,
                        width: track_border_width,
                        color: track_border_color,
                    },
                    ..renderer::Quad::default()
                },
                track_background,
            );

            let handle_size = state.size.value * scale;
            let center_x = bounds.x
                + scale
                    * (tokens::component::switch::TRACK_HEIGHT / 2.0
                        + (tokens::component::switch::TRACK_WIDTH
                            - tokens::component::switch::TRACK_HEIGHT)
                            * state.position.value);
            let center_y = bounds.center_y();
            let handle_bounds = Rectangle {
                x: center_x - handle_size / 2.0,
                y: center_y - handle_size / 2.0,
                width: handle_size,
                height: handle_size,
            };
            let handle_background = mix(
                solid_color(unselected_style.foreground),
                solid_color(selected_style.foreground),
                color_progress,
            );
            let handle_border_width = lerp(
                unselected_style.foreground_border_width,
                selected_style.foreground_border_width,
                color_progress,
            );
            let handle_border_color = mix(
                unselected_style.foreground_border_color,
                selected_style.foreground_border_color,
                color_progress,
            );

            let state_layer_opacity = if is_disabled {
                0.0
            } else if state.is_pressed {
                tokens::state::PRESSED_STATE_LAYER_OPACITY
            } else if matches!(status, iced_toggler::Status::Hovered { .. }) {
                tokens::state::HOVER_STATE_LAYER_OPACITY
            } else {
                0.0
            };

            if state_layer_opacity > 0.0 {
                let state_layer_size = tokens::component::switch::STATE_LAYER_SIZE * scale;
                let state_layer_color = if self.is_toggled {
                    colors.primary.color
                } else {
                    colors.surface.text
                };

                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle {
                            x: center_x - state_layer_size / 2.0,
                            y: center_y - state_layer_size / 2.0,
                            width: state_layer_size,
                            height: state_layer_size,
                        },
                        border: Border {
                            radius: border::Radius::new(state_layer_size / 2.0),
                            ..Border::default()
                        },
                        ..renderer::Quad::default()
                    },
                    alpha_color(state_layer_color, state_layer_opacity),
                );
            }

            renderer.fill_quad(
                renderer::Quad {
                    bounds: handle_bounds,
                    border: Border {
                        radius: border::Radius::new(handle_size / 2.0),
                        width: handle_border_width,
                        color: handle_border_color,
                    },
                    ..renderer::Quad::default()
                },
                handle_background,
            );

            if self.shows_icons() {
                let icon_progress = state.icon.value.clamp(0.0, 1.0);
                let selected_icon_opacity = state.icon_opacity.value.clamp(0.0, 1.0);
                let unselected_icon_opacity = if self.shows_off_icon() {
                    1.0 - selected_icon_opacity
                } else {
                    0.0
                };
                let selected_icon_color = if is_disabled {
                    alpha_color(
                        colors.surface.text,
                        tokens::component::switch::DISABLED_SELECTED_ICON_OPACITY,
                    )
                } else {
                    colors.primary.container_text
                };
                let unselected_icon_color = if is_disabled {
                    alpha_color(
                        colors.surface.container.highest,
                        tokens::component::switch::DISABLED_UNSELECTED_ICON_OPACITY,
                    )
                } else {
                    colors.surface.container.highest
                };

                if selected_icon_opacity > 0.0 {
                    let icon_scale = 0.82 + 0.18 * icon_progress;
                    let icon_size =
                        tokens::component::switch::SELECTED_ICON_SIZE * scale * icon_scale;

                    renderer.draw_svg(
                        core_svg::Svg::new(core_svg::Handle::from_memory(SWITCH_ON_ICON_SVG))
                            .color(selected_icon_color)
                            .opacity(selected_icon_opacity),
                        scaled_rect(handle_bounds, icon_size, icon_size),
                        *viewport,
                    );
                }

                if unselected_icon_opacity > 0.0 {
                    let icon_size = tokens::component::switch::UNSELECTED_ICON_SIZE * scale;

                    renderer.draw_svg(
                        core_svg::Svg::new(core_svg::Handle::from_memory(SWITCH_OFF_ICON_SVG))
                            .color(unselected_icon_color)
                            .opacity(unselected_icon_opacity),
                        scaled_rect(handle_bounds, icon_size, icon_size),
                        *viewport,
                    );
                }
            }

            if self.label.is_none() {
                return;
            }

            let label_layout = children.next().unwrap();

            core_widget::text::draw(
                renderer,
                defaults,
                label_layout.bounds(),
                state.text.raw(),
                core_widget::text::Style {
                    color: current_style.text_color,
                },
                viewport,
            );
        }

        fn operate(
            &mut self,
            _tree: &mut Tree,
            layout: Layout<'_>,
            _renderer: &Renderer,
            operation: &mut dyn core_widget::Operation,
        ) {
            if let Some(label) = self.label.as_deref() {
                operation.text(None, layout.bounds(), label);
            }
        }
    }

    impl<'a, Message, Renderer> From<Toggler<'a, Message, Renderer>>
        for Element<'a, Message, Theme, Renderer>
    where
        Message: 'a,
        Renderer: core_text::Renderer + core_svg::Renderer + 'a,
    {
        fn from(toggler: Toggler<'a, Message, Renderer>) -> Self {
            Element::new(toggler)
        }
    }

    pub fn control<'a, Message, Renderer>(is_toggled: bool) -> Toggler<'a, Message, Renderer>
    where
        Renderer: core_text::Renderer + core_svg::Renderer + 'a,
    {
        Toggler::new(is_toggled)
            .size(tokens::component::switch::TRACK_HEIGHT)
            .spacing(f32::from(
                tokens::component::divider::LIST_ITEM_LEADING_SPACE,
            ))
            .text_size(tokens::component::switch::LABEL_TEXT_SIZE)
            .text_line_height(absolute_line_height(
                tokens::component::switch::LABEL_TEXT_LINE_HEIGHT,
            ))
            .show_only_selected_icon(true)
            .style(toggler_style::default)
    }

    pub fn standard<'a, Message, Renderer>(
        is_toggled: bool,
        label: impl text::IntoFragment<'a>,
        on_toggle: impl Fn(bool) -> Message + 'a,
    ) -> Element<'a, Message, Theme, Renderer>
    where
        Message: 'a,
        Renderer: iced_widget::core::Renderer + core_text::Renderer + core_svg::Renderer + 'a,
    {
        Container::new(control(is_toggled).label(label).on_toggle(on_toggle))
            .center_y(Length::Fixed(tokens::component::switch::STATE_LAYER_SIZE))
            .into()
    }
}

#[cfg(test)]
mod tests {
    use iced_widget::core::Element;

    use super::*;

    #[derive(Debug, Clone)]
    enum Message {
        Pressed,
        Toggled,
    }

    type TestElement<'a> = Element<'a, Message, Theme, iced_widget::Renderer>;

    fn toggled(_: bool) -> Message {
        Message::Toggled
    }

    #[test]
    fn material_button_constructors_compile_to_elements() {
        let _: TestElement<'_> = button::filled("Filled").on_press(Message::Pressed).into();
        let _: TestElement<'_> = button::filled_tonal("Tonal")
            .on_press(Message::Pressed)
            .into();
        let _: TestElement<'_> = button::outlined_icon("+").on_press(Message::Pressed).into();
        let _: TestElement<'_> = button::primary_fab("+").on_press(Message::Pressed).into();
        let _: TestElement<'_> = button::secondary_fab("+").on_press(Message::Pressed).into();
        let _: TestElement<'_> = button::surface_fab("+").on_press(Message::Pressed).into();
        let _: TestElement<'_> = button::assist_chip("Assist")
            .on_press(Message::Pressed)
            .into();
    }

    #[test]
    fn material_container_constructors_compile_to_elements() {
        let surface = Text::new("Surface");
        let _: TestElement<'_> = container::surface_container_high(surface).into();

        let elevated = Text::new("Elevated card");
        let _: TestElement<'_> = container::elevated_card(elevated).into();

        let filled = Text::new("Filled card");
        let _: TestElement<'_> = container::filled_card(filled).into();

        let outlined = Text::new("Outlined card");
        let _: TestElement<'_> = container::outlined_card(outlined).into();
    }

    #[test]
    fn material_slider_and_progress_constructors_compile_to_elements() {
        let _: TestElement<'_> = slider::continuous(0.0..=100.0, 42.0, |_| Message::Pressed).into();
        let _: TestElement<'_> = progress_bar::linear(0.0..=100.0, 42.0).into();
        let _: TestElement<'_> = progress_bar::vertical_linear(0.0..=100.0, 42.0).into();
    }

    #[test]
    fn material_tooltip_constructor_compiles_to_element() {
        let content = button::assist_chip("Hint").on_press(Message::Pressed);
        let _: TestElement<'_> =
            tooltip::plain(content, "Material 3 plain tooltip", tooltip::Position::Top).into();
    }

    #[test]
    fn material_selection_constructors_compile_to_elements() {
        let _: TestElement<'_> = checkbox::standard(true, "Enable actions", toggled);
        let _: TestElement<'_> = toggler::standard(true, "Dark theme", toggled);
    }

    #[test]
    fn material_text_input_constructor_compiles_to_element() {
        let _: TestElement<'_> = text_input::outlined("Write a note", "value")
            .on_input(|_| Message::Pressed)
            .into();
    }

    #[test]
    fn checkbox_checkmark_svg_uses_m3_rect_mark_geometry() {
        let svg = String::from_utf8(checkbox_checkmark_svg(1.0)).expect("valid svg");

        assert!(svg.contains("viewBox=\"0 0 18 18\""));
        assert!(svg.contains("scale(1 -1) translate(7 -14) rotate(45)"));
        assert!(svg.contains("width=\"2\" height=\"5.656854\""));
        assert!(svg.contains("width=\"11.313708\" height=\"2\""));
    }
}
