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
                    let percent = f64::from(cursor_position.x - bounds.x) / f64::from(bounds.width);
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
                | Event::Touch(touch::Event::FingerLost { .. })
                    if state.is_dragging =>
                {
                    if let Some(on_release) = self.on_release.clone() {
                        shell.publish(on_release);
                    }

                    state.is_dragging = false;
                }
                Event::Mouse(mouse::Event::CursorMoved { .. })
                | Event::Touch(touch::Event::FingerMoved { .. })
                    if state.is_dragging =>
                {
                    if let Some(value) = cursor.land().position().and_then(locate) {
                        change(value);
                    }

                    shell.capture_event();
                }
                Event::Mouse(mouse::Event::WheelScrolled { delta })
                    if state.keyboard_modifiers.control() && cursor.is_over(layout.bounds()) =>
                {
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
                Event::Keyboard(keyboard::Event::KeyPressed { key, .. })
                    if cursor.is_over(layout.bounds()) =>
                {
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
        let thumb_bounds =
            thumb_bounds_with_size(bounds, self.value, &self.range, handle_width, handle_height);
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
#[path = "../../../tests/widget/component/slider.rs"]
mod tests;
