//! Material 3 sized widget constructors.
//!
//! The style traits exposed by `iced` control colors, borders, and shadows, but
//! not layout defaults like button height or checkbox size. These helpers apply
//! the Material 3 component metrics from [`crate::tokens`] at construction time.

use iced_widget::checkbox as iced_checkbox;
use iced_widget::container as iced_container;
use iced_widget::core::svg as core_svg;
use iced_widget::core::text as core_text;
use iced_widget::core::time::Instant;
use iced_widget::core::widget as core_widget;
use iced_widget::core::widget::tree::{self, Tree};
use iced_widget::core::{
    Background, Border, Clipboard, Color, Element, Event, Layout, Length, Padding, Pixels, Point,
    Rectangle, Shell, Size, Vector, Widget, alignment, border, input_method, layout, mouse,
    overlay, renderer, touch, window,
};
use iced_widget::radio as iced_radio;
use iced_widget::rule as iced_rule;
use iced_widget::text::{self, LineHeight};
use iced_widget::text_editor as iced_text_editor;
use iced_widget::text_input as iced_text_input;
use iced_widget::toggler as iced_toggler;
use iced_widget::tooltip as iced_tooltip;
use iced_widget::{
    Container, Row, Rule, Text, TextEditor as IcedTextEditor, TextInput as IcedTextInput,
};

use crate::utils::mix;
use crate::{Theme, fonts, tokens, web_input};
use crate::{
    button as button_style, checkbox as checkbox_style, container as container_style,
    menu as menu_style, pick_list as pick_list_style, rule as rule_style, slider as slider_style,
    text_editor as text_editor_style, text_input as text_input_style, toggler as toggler_style,
    tooltip as tooltip_style,
};

pub mod app_bar;
pub mod badge;
pub mod card;
pub mod combo_box;
pub mod data_table;
pub mod dialog;
pub mod list;
mod menu_overlay;
pub mod navigation;
pub mod page;
pub mod progress_bar;
pub mod search;
pub mod segmented_button;
pub mod select;
pub mod sheet;
pub mod snackbar;
mod support;
pub mod tabs;
pub mod theme_picker;
pub mod toolbar;

use support::{
    AnimatedScalar, SelectionState, TextFieldState, alpha_border, alpha_color, bool_value,
    duration_ms, lerp, scaled_rect, solid_color,
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

#[cfg(target_os = "windows")]
fn normalize_windows_ime_request(
    input_method: &mut input_method::InputMethod,
    avoid_bounds: Rectangle,
) {
    let input_method::InputMethod::Enabled {
        cursor, preedit, ..
    } = input_method
    else {
        return;
    };

    if !preedit
        .as_ref()
        .is_some_and(|preedit| !preedit.content.is_empty())
    {
        return;
    }

    *preedit = None;

    let bounds_right = avoid_bounds.x + avoid_bounds.width;
    let bounds_bottom = avoid_bounds.y + avoid_bounds.height;
    let cursor_right = cursor.x + cursor.width;
    let cursor_bottom = cursor.y + cursor.height;

    if cursor.x < bounds_right
        && cursor_right > avoid_bounds.x
        && cursor.y < bounds_bottom
        && cursor_bottom > avoid_bounds.y
    {
        cursor.x = avoid_bounds.x;
        cursor.width = avoid_bounds.width;
        cursor.height = (bounds_bottom - cursor.y).max(cursor.height);
    }
}

#[cfg(not(target_os = "windows"))]
fn normalize_windows_ime_request(
    _input_method: &mut input_method::InputMethod,
    _avoid_bounds: Rectangle,
) {
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

fn centered_icon_text<'a, Renderer>(
    icon: impl text::IntoFragment<'a>,
    size: f32,
) -> Text<'a, Theme, Renderer>
where
    Renderer: core_text::Renderer,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    fonts::icon(icon, size)
        .width(Length::Fixed(size))
        .height(Length::Fixed(size))
        .center()
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
    iced_widget::core::Font: Into<Renderer::Font>,
{
    let icon = centered_icon_text(icon, tokens::component::icon_button::ICON_SIZE);

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
    iced_widget::core::Font: Into<Renderer::Font>,
{
    sized_fab_content(
        icon,
        tokens::component::fab::CONTAINER_WIDTH,
        tokens::component::fab::CONTAINER_HEIGHT,
        tokens::component::fab::ICON_SIZE,
    )
}

fn sized_fab_content<'a, Message, Renderer>(
    icon: impl text::IntoFragment<'a>,
    width: f32,
    height: f32,
    icon_size: f32,
) -> Container<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    let icon = centered_icon_text(icon, icon_size);

    Container::new(icon)
        .center_x(Length::Fixed(width))
        .center_y(Length::Fixed(height))
}

fn extended_fab_content<'a, Message, Renderer>(
    label: impl text::IntoFragment<'a>,
) -> Container<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
{
    let label_text = tokens::component::fab::EXTENDED_LABEL_TEXT;

    Container::new(text_with_metrics(
        label,
        label_text.size,
        label_text.line_height,
    ))
    .height(Length::Fixed(
        tokens::component::fab::EXTENDED_CONTAINER_HEIGHT,
    ))
    .padding(Padding {
        top: 0.0,
        right: tokens::component::fab::EXTENDED_TRAILING_SPACE,
        bottom: 0.0,
        left: tokens::component::fab::EXTENDED_LEADING_SPACE,
    })
    .align_y(alignment::Vertical::Center)
}

fn extended_fab_icon_content<'a, Message, Renderer>(
    icon: impl text::IntoFragment<'a>,
    label: impl text::IntoFragment<'a>,
) -> Container<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    let label_text = tokens::component::fab::EXTENDED_LABEL_TEXT;
    let content = Row::new()
        .push(centered_icon_text(
            icon,
            tokens::component::fab::EXTENDED_ICON_SIZE,
        ))
        .push(text_with_metrics(
            label,
            label_text.size,
            label_text.line_height,
        ))
        .spacing(tokens::component::fab::EXTENDED_ICON_LABEL_SPACE)
        .align_y(alignment::Vertical::Center);

    Container::new(content)
        .height(Length::Fixed(
            tokens::component::fab::EXTENDED_CONTAINER_HEIGHT,
        ))
        .padding(Padding {
            top: 0.0,
            right: tokens::component::fab::EXTENDED_TRAILING_SPACE,
            bottom: 0.0,
            left: tokens::component::fab::EXTENDED_LEADING_SPACE,
        })
        .align_y(alignment::Vertical::Center)
}

pub mod button;
pub mod slider {
    //! Material 3 slider constructors with token-backed layout defaults.

    use super::*;
    use iced_widget::core::keyboard;
    use iced_widget::core::keyboard::key::{self, Key};
    use iced_widget::slider::{Catalog as SliderCatalog, HandleShape, Status, Style, StyleFn};
    use num_traits::FromPrimitive;
    use std::fmt;
    use std::ops::RangeInclusive;

    /// A Material 3 continuous slider with thumb elevation.
    pub struct Slider<'a, T, Message> {
        range: RangeInclusive<T>,
        step: T,
        shift_step: Option<T>,
        value: T,
        default: Option<T>,
        on_change: Box<dyn Fn(T) -> Message + 'a>,
        on_release: Option<Message>,
        width: Length,
        height: f32,
        class: StyleFn<'a, Theme>,
        status: Option<Status>,
    }

    impl<T, Message> fmt::Debug for Slider<'_, T, Message>
    where
        T: fmt::Debug,
    {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("Slider")
                .field("range", &self.range)
                .field("step", &self.step)
                .field("shift_step", &self.shift_step)
                .field("value", &self.value)
                .field("default", &self.default)
                .field("width", &self.width)
                .field("height", &self.height)
                .field("status", &self.status)
                .finish_non_exhaustive()
        }
    }

    impl<'a, T, Message> Slider<'a, T, Message>
    where
        T: Copy + From<u8> + PartialOrd,
        Message: Clone + 'a,
    {
        fn new(range: RangeInclusive<T>, value: T, on_change: impl Fn(T) -> Message + 'a) -> Self {
            let value = clamped_value(&range, value);

            Self {
                range,
                step: T::from(1),
                shift_step: None,
                value,
                default: None,
                on_change: Box::new(on_change),
                on_release: None,
                width: Length::Fill,
                height: tokens::component::slider::STATE_LAYER_SIZE,
                class: Box::new(slider_style::default),
                status: None,
            }
        }

        /// Sets the optional default value.
        pub fn default(mut self, default: impl Into<T>) -> Self {
            self.default = Some(default.into());
            self
        }

        /// Sets the release message.
        pub fn on_release(mut self, on_release: Message) -> Self {
            self.on_release = Some(on_release);
            self
        }

        /// Sets the width of the slider.
        pub fn width(mut self, width: impl Into<Length>) -> Self {
            self.width = width.into();
            self
        }

        /// Sets the height of the slider.
        pub fn height(mut self, height: impl Into<Pixels>) -> Self {
            self.height = height.into().0;
            self
        }

        /// Sets the step size of the slider.
        pub fn step(mut self, step: impl Into<T>) -> Self {
            self.step = step.into();
            self
        }

        /// Sets the optional shift-step size.
        pub fn shift_step(mut self, shift_step: impl Into<T>) -> Self {
            self.shift_step = Some(shift_step.into());
            self
        }

        /// Sets the slider style.
        pub fn style(mut self, style: impl Fn(&Theme, Status) -> Style + 'a) -> Self {
            self.class = Box::new(style);
            self
        }
    }

    pub fn continuous<'a, T, Message>(
        range: RangeInclusive<T>,
        value: T,
        on_change: impl Fn(T) -> Message + 'a,
    ) -> Slider<'a, T, Message>
    where
        T: Copy + From<u8> + PartialOrd,
        Message: Clone + 'a,
    {
        Slider::new(range, value, on_change)
    }

    impl<'a, T, Message, Renderer> Widget<Message, Theme, Renderer> for Slider<'a, T, Message>
    where
        T: Copy + Into<f64> + FromPrimitive,
        Message: Clone,
        Renderer: iced_widget::core::Renderer,
    {
        fn tag(&self) -> tree::Tag {
            tree::Tag::of::<State>()
        }

        fn state(&self) -> tree::State {
            tree::State::new(State::default())
        }

        fn size(&self) -> Size<Length> {
            Size {
                width: self.width,
                height: Length::Shrink,
            }
        }

        fn layout(
            &mut self,
            _tree: &mut Tree,
            _renderer: &Renderer,
            limits: &layout::Limits,
        ) -> layout::Node {
            layout::atomic(limits, self.width, self.height)
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
            let state = tree.state.downcast_mut::<State>();

            let mut update = || {
                let current_value = self.value;

                let locate = |cursor_position: Point| -> Option<T> {
                    let bounds = layout.bounds();

                    if cursor_position.x <= bounds.x {
                        Some(*self.range.start())
                    } else if cursor_position.x >= bounds.x + bounds.width {
                        Some(*self.range.end())
                    } else {
                        let step = if state.keyboard_modifiers.shift() {
                            self.shift_step.unwrap_or(self.step)
                        } else {
                            self.step
                        }
                        .into();

                        let start = (*self.range.start()).into();
                        let end = (*self.range.end()).into();
                        let percent =
                            f64::from(cursor_position.x - bounds.x) / f64::from(bounds.width);
                        let steps = (percent * (end - start) / step).round();
                        let value = steps * step + start;

                        T::from_f64(value.min(end))
                    }
                };

                let increment = |value: T| -> Option<T> {
                    let step = if state.keyboard_modifiers.shift() {
                        self.shift_step.unwrap_or(self.step)
                    } else {
                        self.step
                    }
                    .into();

                    let steps = (value.into() / step).round();
                    let new_value = step * (steps + 1.0);

                    if new_value > (*self.range.end()).into() {
                        return Some(*self.range.end());
                    }

                    T::from_f64(new_value)
                };

                let decrement = |value: T| -> Option<T> {
                    let step = if state.keyboard_modifiers.shift() {
                        self.shift_step.unwrap_or(self.step)
                    } else {
                        self.step
                    }
                    .into();

                    let steps = (value.into() / step).round();
                    let new_value = step * (steps - 1.0);

                    if new_value < (*self.range.start()).into() {
                        return Some(*self.range.start());
                    }

                    T::from_f64(new_value)
                };

                let mut change = |new_value: T| {
                    if (self.value.into() - new_value.into()).abs() > f64::EPSILON {
                        shell.publish((self.on_change)(new_value));
                        self.value = new_value;
                    }
                };

                match &event {
                    Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                    | Event::Touch(touch::Event::FingerPressed { .. }) => {
                        if let Some(cursor_position) = cursor.position_over(layout.bounds()) {
                            if state.keyboard_modifiers.command() {
                                if let Some(default) = self.default {
                                    change(default);
                                }

                                state.is_dragging = false;
                            } else {
                                if let Some(value) = locate(cursor_position) {
                                    change(value);
                                }

                                state.is_dragging = true;
                            }

                            shell.capture_event();
                        }
                    }
                    Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                    | Event::Touch(touch::Event::FingerLifted { .. })
                    | Event::Touch(touch::Event::FingerLost { .. }) => {
                        if state.is_dragging {
                            if let Some(on_release) = self.on_release.clone() {
                                shell.publish(on_release);
                            }

                            state.is_dragging = false;
                        }
                    }
                    Event::Mouse(mouse::Event::CursorMoved { .. })
                    | Event::Touch(touch::Event::FingerMoved { .. }) => {
                        if state.is_dragging {
                            if let Some(value) = cursor.land().position().and_then(locate) {
                                change(value);
                            }

                            shell.capture_event();
                        }
                    }
                    Event::Mouse(mouse::Event::WheelScrolled { delta })
                        if state.keyboard_modifiers.control() =>
                    {
                        if cursor.is_over(layout.bounds()) {
                            let delta = match delta {
                                mouse::ScrollDelta::Lines { x: _, y } => y,
                                mouse::ScrollDelta::Pixels { x: _, y } => y,
                            };

                            if *delta < 0.0 {
                                if let Some(value) = decrement(current_value) {
                                    change(value);
                                }
                            } else if let Some(value) = increment(current_value) {
                                change(value);
                            }

                            shell.capture_event();
                        }
                    }
                    Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) => {
                        if cursor.is_over(layout.bounds()) {
                            match key {
                                Key::Named(key::Named::ArrowUp) => {
                                    if let Some(value) = increment(current_value) {
                                        change(value);
                                    }

                                    shell.capture_event();
                                }
                                Key::Named(key::Named::ArrowDown) => {
                                    if let Some(value) = decrement(current_value) {
                                        change(value);
                                    }

                                    shell.capture_event();
                                }
                                _ => (),
                            }
                        }
                    }
                    Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) => {
                        state.keyboard_modifiers = *modifiers;
                    }
                    _ => {}
                }
            };

            update();

            let current_status = if state.is_dragging {
                Status::Dragged
            } else if cursor.is_over(layout.bounds()) {
                Status::Hovered
            } else {
                Status::Active
            };

            if let Event::Window(window::Event::RedrawRequested(_now)) = event {
                self.status = Some(current_status);
            } else if self.status.is_some_and(|status| status != current_status) {
                shell.request_redraw();
            }
        }

        fn draw(
            &self,
            _tree: &Tree,
            renderer: &mut Renderer,
            theme: &Theme,
            _style: &renderer::Style,
            layout: Layout<'_>,
            _cursor: mouse::Cursor,
            _viewport: &Rectangle,
        ) {
            let bounds = layout.bounds();
            let style = theme.style(&self.class, self.status.unwrap_or(Status::Active));
            let (handle_width, handle_height, handle_border_radius) = match style.handle.shape {
                HandleShape::Circle { radius } => (radius * 2.0, radius * 2.0, radius.into()),
                HandleShape::Rectangle {
                    width,
                    border_radius,
                } => (f32::from(width), bounds.height, border_radius),
            };
            let thumb_bounds = thumb_bounds_with_size(
                bounds,
                self.value,
                &self.range,
                handle_width,
                handle_height,
            );
            let rail_y = bounds.y + bounds.height / 2.0;

            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle {
                        x: bounds.x,
                        y: rail_y - style.rail.width / 2.0,
                        width: thumb_bounds.x - bounds.x + handle_width / 2.0,
                        height: style.rail.width,
                    },
                    border: style.rail.border,
                    ..renderer::Quad::default()
                },
                style.rail.backgrounds.0,
            );

            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle {
                        x: thumb_bounds.x + handle_width / 2.0,
                        y: rail_y - style.rail.width / 2.0,
                        width: bounds.width - (thumb_bounds.x - bounds.x) - handle_width / 2.0,
                        height: style.rail.width,
                    },
                    border: style.rail.border,
                    ..renderer::Quad::default()
                },
                style.rail.backgrounds.1,
            );

            renderer.fill_quad(
                renderer::Quad {
                    bounds: thumb_bounds,
                    border: Border {
                        radius: handle_border_radius,
                        width: style.handle.border_width,
                        color: style.handle.border_color,
                    },
                    shadow: thumb_shadow(theme),
                    ..renderer::Quad::default()
                },
                style.handle.background,
            );
        }

        fn mouse_interaction(
            &self,
            tree: &Tree,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
            _viewport: &Rectangle,
            _renderer: &Renderer,
        ) -> mouse::Interaction {
            let state = tree.state.downcast_ref::<State>();

            if state.is_dragging {
                if cfg!(target_os = "windows") {
                    mouse::Interaction::Pointer
                } else {
                    mouse::Interaction::Grabbing
                }
            } else if cursor.is_over(layout.bounds()) {
                if cfg!(target_os = "windows") {
                    mouse::Interaction::Pointer
                } else {
                    mouse::Interaction::Grab
                }
            } else {
                mouse::Interaction::default()
            }
        }
    }

    impl<'a, T, Message, Renderer> From<Slider<'a, T, Message>>
        for Element<'a, Message, Theme, Renderer>
    where
        T: Copy + Into<f64> + FromPrimitive + 'a,
        Message: Clone + 'a,
        Renderer: iced_widget::core::Renderer + 'a,
    {
        fn from(slider: Slider<'a, T, Message>) -> Self {
            Element::new(slider)
        }
    }

    fn clamped_value<T>(range: &RangeInclusive<T>, value: T) -> T
    where
        T: Copy + PartialOrd,
    {
        if value < *range.start() {
            *range.start()
        } else if value > *range.end() {
            *range.end()
        } else {
            value
        }
    }

    fn thumb_bounds_with_size<T>(
        bounds: Rectangle,
        value: T,
        range: &RangeInclusive<T>,
        handle_width: f32,
        handle_height: f32,
    ) -> Rectangle
    where
        T: Copy + Into<f64>,
    {
        let range_start = (*range.start()).into() as f32;
        let range_end = (*range.end()).into() as f32;
        let offset = if range_start >= range_end {
            0.0
        } else {
            let value = (value.into() as f32).clamp(range_start, range_end);

            (bounds.width - handle_width) * (value - range_start) / (range_end - range_start)
        };
        let rail_y = bounds.y + bounds.height / 2.0;

        Rectangle {
            x: bounds.x + offset,
            y: rail_y - handle_height / 2.0,
            width: handle_width,
            height: handle_height,
        }
    }

    fn thumb_shadow(theme: &Theme) -> iced_widget::core::Shadow {
        crate::utils::shadow_from_elevation(
            tokens::component::slider::HANDLE_ELEVATION,
            theme.colors().shadow,
        )
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    struct State {
        is_dragging: bool,
        keyboard_modifiers: keyboard::Modifiers,
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn thumb_shadow_uses_material_legacy_slider_elevation() {
            let theme = Theme::Light;
            let shadow = thumb_shadow(&theme);
            let layer = tokens::elevation::shadow(2).ambient;

            assert_eq!(tokens::component::slider::HANDLE_ELEVATION, 2.0);
            assert_eq!(shadow.offset.y, layer.y);
            assert_eq!(shadow.blur_radius, layer.blur);
        }

        #[test]
        fn thumb_bounds_follow_slider_value_fraction() {
            let bounds = Rectangle::new(Point::ORIGIN, Size::new(120.0, 40.0));
            let thumb = thumb_bounds_with_size(
                bounds,
                50.0,
                &(0.0..=100.0),
                tokens::component::slider::HANDLE_WIDTH,
                tokens::component::slider::HANDLE_HEIGHT,
            );

            assert_eq!(thumb.x, 50.0);
            assert_eq!(thumb.y, 10.0);
            assert_eq!(thumb.width, tokens::component::slider::HANDLE_WIDTH);
            assert_eq!(thumb.height, tokens::component::slider::HANDLE_HEIGHT);
        }

        #[test]
        fn thumb_bounds_do_not_panic_for_degenerate_range() {
            let bounds = Rectangle::new(Point::ORIGIN, Size::new(120.0, 40.0));
            let thumb = thumb_bounds_with_size(
                bounds,
                50.0,
                &(100.0..=0.0),
                tokens::component::slider::HANDLE_WIDTH,
                tokens::component::slider::HANDLE_HEIGHT,
            );

            assert_eq!(thumb.x, 0.0);
            assert_eq!(thumb.y, 10.0);
        }
    }
}

pub mod rule {
    //! Material 3 divider constructors with token-backed thickness and insets.

    use super::*;

    pub fn horizontal_full_width<'a>() -> Rule<'a, Theme> {
        iced_rule::horizontal(tokens::component::divider::THICKNESS).style(rule_style::full_width)
    }

    pub fn horizontal_inset<'a>() -> Rule<'a, Theme> {
        iced_rule::horizontal(tokens::component::divider::THICKNESS).style(rule_style::inset)
    }

    pub fn vertical_full_height<'a>() -> Rule<'a, Theme> {
        iced_rule::vertical(tokens::component::divider::THICKNESS).style(rule_style::full_width)
    }

    pub fn vertical_inset<'a>() -> Rule<'a, Theme> {
        iced_rule::vertical(tokens::component::divider::THICKNESS).style(rule_style::inset)
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
        super::card::elevated(content)
    }

    pub fn filled_card<'a, Message, Renderer>(
        content: impl Into<Element<'a, Message, Theme, Renderer>>,
    ) -> Container<'a, Message, Theme, Renderer>
    where
        Message: 'a,
        Renderer: iced_widget::core::Renderer + 'a,
    {
        super::card::filled(content)
    }

    pub fn outlined_card<'a, Message, Renderer>(
        content: impl Into<Element<'a, Message, Theme, Renderer>>,
    ) -> Container<'a, Message, Theme, Renderer>
    where
        Message: 'a,
        Renderer: iced_widget::core::Renderer + 'a,
    {
        super::card::outlined(content)
    }
}

pub mod pick_list {
    //! Material 3 select constructors with token-backed layout defaults.

    use super::*;
    use std::borrow::Borrow;

    pub fn outlined<'a, T, L, V, Message, Renderer>(
        options: L,
        selected: Option<V>,
        on_select: impl Fn(T) -> Message + 'a,
    ) -> select::Select<'a, T, L, V, Message, Renderer>
    where
        T: ToString + PartialEq + Clone + 'a,
        L: Borrow<[T]> + 'a,
        V: Borrow<T> + 'a,
        Message: Clone + 'a,
        Renderer: core_text::Renderer + 'a,
    {
        select::Select::new(options, selected, on_select)
            .padding(Padding {
                top: tokens::component::text_field::TOP_SPACE,
                right: tokens::component::text_field::TRAILING_SPACE,
                bottom: tokens::component::text_field::BOTTOM_SPACE,
                left: tokens::component::text_field::LEADING_SPACE,
            })
            .option_padding(select::menu_option_padding())
            .text_size(tokens::component::text_field::INPUT_TEXT_SIZE)
            .text_line_height(absolute_line_height(
                tokens::component::text_field::INPUT_TEXT_LINE_HEIGHT,
            ))
            .width(Length::Fill)
            .style(pick_list_style::default)
            .menu_style(menu_style::outlined_select)
    }
}

pub mod tooltip;
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

    fn caretless_input_layer_style(
        theme: &Theme,
        _status: iced_text_input::Status,
    ) -> iced_text_input::Style {
        input_layer_style(
            theme,
            iced_text_input::Status::Focused { is_hovered: false },
        )
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
        value: String,
        is_populated: bool,
        is_enabled: bool,
        is_secure: bool,
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
                .field("is_secure", &self.is_secure)
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
                value: value.to_owned(),
                is_populated: !value.is_empty(),
                is_enabled: false,
                is_secure: false,
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
            self.is_secure = is_secure;
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
            let was_focused = state.is_focused;

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
                    if state.clear_ime_preedit() {
                        shell.request_redraw();
                    }
                }
                Event::InputMethod(input_method::Event::Preedit(content, _)) => {
                    if state.set_ime_preedit(content) {
                        shell.request_redraw();
                    }
                }
                Event::InputMethod(
                    input_method::Event::Opened
                    | input_method::Event::Closed
                    | input_method::Event::Commit(_),
                ) => {
                    if state.clear_ime_preedit() {
                        shell.request_redraw();
                    }
                }
                Event::Window(window::Event::RedrawRequested(now)) => {
                    if state.label_float.advance(*now) {
                        shell.request_redraw();
                    }
                }
                _ => {}
            }

            if was_focused != state.is_focused {
                if state.is_focused {
                    web_input::show_mobile_keyboard();
                } else {
                    web_input::hide_mobile_keyboard();
                }
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

            normalize_windows_ime_request(shell.input_method_mut(), bounds);
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

            let input_layout = layout.children().next().unwrap();
            let caretless_input;
            let input =
                if state.ime_preedit_active && self.is_enabled && !cfg!(target_os = "windows") {
                    // Keep iced_winit's IME preedit overlay, but suppress iced's own
                    // blinking caret so composition does not show two insertion marks.
                    let mut input = IcedTextInput::new("", self.value.as_str())
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
                        .style(caretless_input_layer_style);

                    if self.is_secure {
                        input = input.secure(true);
                    }

                    if let Some(font) = self.font {
                        input = input.font(font);
                    }

                    caretless_input = input;
                    &caretless_input
                } else {
                    &self.input
                };

            <IcedTextInput<'_, Message, Theme, Renderer> as Widget<Message, Theme, Renderer>>::draw(
                input,
                &tree.children[0],
                renderer,
                theme,
                defaults,
                input_layout,
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

pub mod text_editor {
    //! Material 3 outlined multi-line text field constructors.

    use super::*;

    pub use iced_text_editor::{Action, Binding, Content, KeyPress};

    /// Default height for a compact outlined text area preview.
    pub const OUTLINED_AREA_HEIGHT: f32 = tokens::component::text_field::CONTAINER_HEIGHT * 2.0;

    /// A Material 3 outlined multi-line text field.
    pub struct TextEditor<'a, Message, Renderer = iced_widget::Renderer>
    where
        Renderer: core_text::Renderer,
    {
        inner: IcedTextEditor<'a, core_text::highlighter::PlainText, Message, Theme, Renderer>,
    }

    impl<Message, Renderer> std::fmt::Debug for TextEditor<'_, Message, Renderer>
    where
        Renderer: core_text::Renderer,
    {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("TextEditor").finish_non_exhaustive()
        }
    }

    impl<'a, Message, Renderer> TextEditor<'a, Message, Renderer>
    where
        Message: 'a,
        Renderer: core_text::Renderer + 'a,
    {
        pub fn new(content: &'a Content<Renderer>) -> Self {
            let inner = IcedTextEditor::new(content)
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
                .min_height(tokens::component::text_field::CONTAINER_HEIGHT)
                .style(text_editor_style::default);

            Self { inner }
        }

        pub fn id(mut self, id: impl Into<core_widget::Id>) -> Self {
            self.inner = self.inner.id(id);
            self
        }

        pub fn placeholder(mut self, placeholder: impl text::IntoFragment<'a>) -> Self {
            self.inner = self.inner.placeholder(placeholder);
            self
        }

        pub fn width(mut self, width: impl Into<Pixels>) -> Self {
            self.inner = self.inner.width(width);
            self
        }

        pub fn height(mut self, height: impl Into<Length>) -> Self {
            self.inner = self.inner.height(height);
            self
        }

        pub fn min_height(mut self, min_height: impl Into<Pixels>) -> Self {
            self.inner = self.inner.min_height(min_height);
            self
        }

        pub fn max_height(mut self, max_height: impl Into<Pixels>) -> Self {
            self.inner = self.inner.max_height(max_height);
            self
        }

        pub fn on_action(mut self, on_edit: impl Fn(Action) -> Message + 'a) -> Self {
            self.inner = self.inner.on_action(on_edit);
            self
        }

        pub fn font(mut self, font: impl Into<Renderer::Font>) -> Self {
            self.inner = self.inner.font(font);
            self
        }

        pub fn size(mut self, size: impl Into<Pixels>) -> Self {
            self.inner = self.inner.size(size);
            self
        }

        pub fn line_height(mut self, line_height: impl Into<core_text::LineHeight>) -> Self {
            self.inner = self.inner.line_height(line_height);
            self
        }

        pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
            self.inner = self.inner.padding(padding);
            self
        }

        pub fn wrapping(mut self, wrapping: core_text::Wrapping) -> Self {
            self.inner = self.inner.wrapping(wrapping);
            self
        }

        pub fn key_binding(
            mut self,
            key_binding: impl Fn(KeyPress) -> Option<Binding<Message>> + 'a,
        ) -> Self {
            self.inner = self.inner.key_binding(key_binding);
            self
        }

        pub fn style(
            mut self,
            style: impl Fn(&Theme, iced_text_editor::Status) -> iced_text_editor::Style + 'a,
        ) -> Self
        where
            <Theme as iced_text_editor::Catalog>::Class<'a>:
                From<iced_text_editor::StyleFn<'a, Theme>>,
        {
            self.inner = self.inner.style(style);
            self
        }
    }

    impl<Message, Renderer> Widget<Message, Theme, Renderer> for TextEditor<'_, Message, Renderer>
    where
        Renderer: core_text::Renderer,
    {
        fn tag(&self) -> tree::Tag {
            self.inner.tag()
        }

        fn state(&self) -> tree::State {
            self.inner.state()
        }

        fn children(&self) -> Vec<Tree> {
            self.inner.children()
        }

        fn diff(&self, tree: &mut Tree) {
            self.inner.diff(tree);
        }

        fn size(&self) -> Size<Length> {
            Widget::<Message, Theme, Renderer>::size(&self.inner)
        }

        fn size_hint(&self) -> Size<Length> {
            Widget::<Message, Theme, Renderer>::size_hint(&self.inner)
        }

        fn layout(
            &mut self,
            tree: &mut Tree,
            renderer: &Renderer,
            limits: &layout::Limits,
        ) -> layout::Node {
            self.inner.layout(tree, renderer, limits)
        }

        fn operate(
            &mut self,
            tree: &mut Tree,
            layout: Layout<'_>,
            renderer: &Renderer,
            operation: &mut dyn core_widget::Operation,
        ) {
            self.inner.operate(tree, layout, renderer, operation);
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
            self.inner.update(
                tree, event, layout, cursor, renderer, clipboard, shell, viewport,
            );

            normalize_windows_ime_request(shell.input_method_mut(), layout.bounds());
        }

        fn mouse_interaction(
            &self,
            tree: &Tree,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
            viewport: &Rectangle,
            renderer: &Renderer,
        ) -> mouse::Interaction {
            self.inner
                .mouse_interaction(tree, layout, cursor, viewport, renderer)
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
            self.inner
                .draw(tree, renderer, theme, defaults, layout, cursor, viewport);
        }

        fn overlay<'b>(
            &'b mut self,
            tree: &'b mut Tree,
            layout: Layout<'b>,
            renderer: &Renderer,
            viewport: &Rectangle,
            translation: Vector,
        ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
            self.inner
                .overlay(tree, layout, renderer, viewport, translation)
        }
    }

    impl<'a, Message, Renderer> From<TextEditor<'a, Message, Renderer>>
        for Element<'a, Message, Theme, Renderer>
    where
        Message: 'a,
        Renderer: core_text::Renderer + 'a,
    {
        fn from(text_editor: TextEditor<'a, Message, Renderer>) -> Self {
            Element::new(text_editor)
        }
    }

    pub fn outlined<'a, Message, Renderer>(
        content: &'a Content<Renderer>,
    ) -> TextEditor<'a, Message, Renderer>
    where
        Renderer: core_text::Renderer + 'a,
        Message: 'a,
    {
        TextEditor::new(content)
    }

    pub fn outlined_area<'a, Message, Renderer>(
        content: &'a Content<Renderer>,
    ) -> TextEditor<'a, Message, Renderer>
    where
        Renderer: core_text::Renderer + 'a,
        Message: 'a,
    {
        outlined(content).height(Length::Fixed(OUTLINED_AREA_HEIGHT))
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
mod tests;
