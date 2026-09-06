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

use crate::style::{
    button as button_style, checkbox as checkbox_style, container as container_style,
    rule as rule_style, slider as slider_style, text_editor as text_editor_style,
    text_input as text_input_style, toggler as toggler_style, tooltip as tooltip_style,
};
use crate::utils::mix;
use crate::{Theme, fonts, tokens, web_input};

#[path = "widget/component/app_bar.rs"]
pub mod app_bar;
#[path = "widget/component/badge.rs"]
pub mod badge;
#[path = "widget/component/card.rs"]
pub mod card;
#[path = "widget/internal/click.rs"]
mod click;
#[path = "widget/component/combobox.rs"]
pub mod combobox;
#[path = "widget/component/data_table.rs"]
pub mod data_table;
#[path = "widget/component/dialog.rs"]
pub mod dialog;
#[path = "widget/component/list.rs"]
pub mod list;
#[path = "widget/component/log_viewer.rs"]
pub mod log_viewer;
#[path = "widget/internal/menu_overlay.rs"]
mod menu_overlay;
#[path = "widget/component/navigation.rs"]
pub mod navigation;
#[path = "widget/component/page.rs"]
pub mod page;
#[path = "widget/component/picker.rs"]
pub mod picker;
#[path = "widget/component/progress_bar.rs"]
pub mod progress_bar;
#[path = "widget/internal/reveal.rs"]
mod reveal;
#[path = "widget/internal/ripple.rs"]
mod ripple;
#[path = "widget/component/search.rs"]
pub mod search;
#[path = "widget/component/segmented_button.rs"]
pub mod segmented_button;
#[path = "widget/component/select.rs"]
pub mod select;
#[path = "widget/component/sheet.rs"]
pub mod sheet;
#[path = "widget/component/snackbar.rs"]
pub mod snackbar;
#[path = "widget/internal/support.rs"]
mod support;
#[path = "widget/component/tabs.rs"]
pub mod tabs;
#[path = "widget/component/theme_picker.rs"]
pub mod theme_picker;
#[path = "widget/component/toolbar.rs"]
pub mod toolbar;
#[path = "widget/component/viewport.rs"]
pub mod viewport;

use support::{
    AnimatedScalar, SelectionState, TextFieldState, TextFieldTouchActivation, alpha_border,
    alpha_color, bool_value, draw_text_field_notched, draw_text_field_outline, duration_ms, lerp,
    scaled_rect, solid_color, text_field_floating_label_notch,
};

const TEXT_FIELD_TOUCH_SLOP: f32 = 8.0;

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

fn text_field_touch_cursor(event: &Event, cursor: mouse::Cursor) -> mouse::Cursor {
    match event {
        Event::Touch(
            touch::Event::FingerPressed { position, .. }
            | touch::Event::FingerMoved { position, .. }
            | touch::Event::FingerLifted { position, .. }
            | touch::Event::FingerLost { position, .. },
        ) if cursor.position().is_none() && !cursor.is_levitating() => {
            mouse::Cursor::Available(*position)
        }
        _ => cursor,
    }
}

fn touch_as_mouse_event(event: &Event) -> Option<Event> {
    match event {
        Event::Touch(touch::Event::FingerPressed { .. }) => Some(Event::Mouse(
            mouse::Event::ButtonPressed(mouse::Button::Left),
        )),
        Event::Touch(touch::Event::FingerMoved { position, .. }) => {
            Some(Event::Mouse(mouse::Event::CursorMoved {
                position: *position,
            }))
        }
        Event::Touch(touch::Event::FingerLifted { .. } | touch::Event::FingerLost { .. }) => Some(
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
        ),
        _ => None,
    }
}

fn text_field_touch_position(position: Point, cursor: mouse::Cursor) -> Option<Point> {
    if let Some(cursor_position) = cursor.position() {
        return Some(cursor_position);
    }

    if cursor.is_levitating() {
        return None;
    }

    Some(position)
}

fn text_field_keyboard_activation(
    touch_activation: &mut Option<TextFieldTouchActivation>,
    event: &Event,
    bounds: Rectangle,
    cursor: mouse::Cursor,
) -> bool {
    match event {
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => cursor.is_over(bounds),
        Event::Touch(touch::Event::FingerPressed { id, position }) => {
            if touch_activation.is_some_and(|activation| !activation.matches(*id)) {
                return false;
            }
            if let Some(local_position) = text_field_touch_position(*position, cursor) {
                *touch_activation = Some(
                    TextFieldTouchActivation::new(*id, *position)
                        .inside(bounds.contains(local_position)),
                );
            } else {
                *touch_activation = None;
            }

            false
        }
        Event::Touch(touch::Event::FingerMoved { id, position }) => {
            if touch_activation.is_some_and(|activation| {
                activation.matches(*id)
                    && (cursor.is_levitating()
                        || activation.moved_beyond_slop(*position, TEXT_FIELD_TOUCH_SLOP))
            }) {
                *touch_activation = None;
            }

            false
        }
        Event::Touch(touch::Event::FingerLifted { id, position }) => {
            let raw_position = *position;
            let position = text_field_touch_position(*position, cursor);

            if touch_activation.is_some_and(|activation| !activation.matches(*id)) {
                return false;
            }

            touch_activation.take().is_some_and(|activation| {
                position.is_some_and(|position| {
                    activation.started_inside()
                        && activation.matches(*id)
                        && bounds.contains(position)
                        && !activation.moved_beyond_slop(raw_position, TEXT_FIELD_TOUCH_SLOP)
                })
            })
        }
        Event::Touch(touch::Event::FingerLost { id, .. }) => {
            if touch_activation.is_some_and(|activation| activation.matches(*id)) {
                *touch_activation = None;
            }

            false
        }
        _ => false,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TextFieldInnerTouchHandling {
    Forward,
    Suppress,
    ConfirmedTap,
    ConfirmedOutsideTap,
}

#[derive(Debug, Clone, Copy)]
enum TextFieldTouchBounds {
    Visible(Rectangle),
    Hidden,
}

impl TextFieldTouchBounds {
    fn visible(bounds: Option<Rectangle>) -> Self {
        bounds.map(Self::Visible).unwrap_or(Self::Hidden)
    }
}

#[derive(Debug, Clone, Copy)]
struct TextFieldTouchContext<'a> {
    is_enabled: bool,
    event: &'a Event,
    bounds: TextFieldTouchBounds,
    cursor: mouse::Cursor,
    activation_before: Option<TextFieldTouchActivation>,
    confirmed_tap: bool,
}

impl TextFieldTouchContext<'_> {
    fn keyboard_activation(&self, touch_activation: &mut Option<TextFieldTouchActivation>) -> bool {
        let bounds = match self.bounds {
            TextFieldTouchBounds::Visible(bounds) => bounds,
            TextFieldTouchBounds::Hidden => Rectangle::with_size(Size::ZERO),
        };

        text_field_keyboard_activation(touch_activation, self.event, bounds, self.cursor)
    }

    fn inner_handling(self) -> TextFieldInnerTouchHandling {
        if !self.is_enabled || !matches!(self.event, Event::Touch(_)) {
            return TextFieldInnerTouchHandling::Forward;
        }

        if self.confirmed_tap {
            return TextFieldInnerTouchHandling::ConfirmedTap;
        }

        if let Event::Touch(touch::Event::FingerLifted { id, position }) = self.event
            && let Some(activation) = self.activation_before
            && activation.matches(*id)
            && !activation.started_inside()
            && text_field_touch_position(*position, self.cursor).is_some()
            && !activation.moved_beyond_slop(*position, TEXT_FIELD_TOUCH_SLOP)
        {
            return TextFieldInnerTouchHandling::ConfirmedOutsideTap;
        }
        TextFieldInnerTouchHandling::Suppress
    }
}

#[derive(Debug, Clone, Copy)]
struct TextInputActivation {
    cursor: mouse::Cursor,
    request_mobile_keyboard: bool,
    web_input_anchor: Option<Rectangle>,
    web_input_translation: Vector,
    inner_touch_handling: TextFieldInnerTouchHandling,
}

#[derive(Debug, Clone, Copy, Default)]
struct WebInputPositionState {
    // Scrollable parents translate the cursor before child updates and only
    // translate InputMethod::cursor back after the child returns. Remember the
    // event-space pointer so the Web bridge can undo the cumulative offset now.
    raw_pointer_position: Option<Point>,
    translation: Vector,
}

impl WebInputPositionState {
    fn update(&mut self, event: &Event, cursor: mouse::Cursor) {
        let raw_pointer_position = match event {
            Event::Mouse(mouse::Event::CursorMoved { position })
            | Event::Touch(
                touch::Event::FingerPressed { position, .. }
                | touch::Event::FingerMoved { position, .. }
                | touch::Event::FingerLifted { position, .. }
                | touch::Event::FingerLost { position, .. },
            ) => Some(*position),
            _ => None,
        };

        if let Some(position) = raw_pointer_position {
            self.raw_pointer_position = Some(position);
        }

        if let (Some(raw), Some(translated)) = (self.raw_pointer_position, cursor.land().position())
        {
            self.translation = translated - raw;
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct MobileTextInputState {
    touch_activation: Option<TextFieldTouchActivation>,
    web_input_position: WebInputPositionState,
}

fn mobile_text_input_activation(
    is_enabled: bool,
    state: &mut MobileTextInputState,
    event: &Event,
    visible_bounds: Option<Rectangle>,
    cursor: mouse::Cursor,
) -> TextInputActivation {
    text_input_activation(
        is_enabled,
        &mut state.touch_activation,
        &mut state.web_input_position,
        event,
        visible_bounds,
        cursor,
    )
}

fn text_input_activation(
    is_enabled: bool,
    touch_activation: &mut Option<TextFieldTouchActivation>,
    web_input_position: &mut WebInputPositionState,
    event: &Event,
    visible_bounds: Option<Rectangle>,
    cursor: mouse::Cursor,
) -> TextInputActivation {
    web_input_position.update(event, cursor);
    if !is_enabled || matches!(event, Event::Window(window::Event::Unfocused)) {
        *touch_activation = None;
    }
    let inner_cursor = text_field_touch_cursor(event, cursor);
    let touch = TextFieldTouchContext {
        is_enabled,
        event,
        bounds: TextFieldTouchBounds::visible(visible_bounds),
        cursor: inner_cursor,
        activation_before: *touch_activation,
        confirmed_tap: false,
    };
    let request_mobile_keyboard = is_enabled && touch.keyboard_activation(touch_activation);
    let web_input_anchor = if request_mobile_keyboard {
        visible_bounds.map(|bounds| {
            let position = inner_cursor.position().unwrap_or(bounds.position());

            Rectangle::new(position, Size::UNIT)
        })
    } else {
        None
    };
    let inner_touch_handling = TextFieldTouchContext {
        confirmed_tap: request_mobile_keyboard,
        ..touch
    }
    .inner_handling();

    TextInputActivation {
        cursor: inner_cursor,
        request_mobile_keyboard,
        web_input_anchor,
        web_input_translation: web_input_position.translation,
        inner_touch_handling,
    }
}

fn web_input_anchor(
    input_method: &input_method::InputMethod,
    visible_bounds: Option<Rectangle>,
    activation: TextInputActivation,
    started_focused: bool,
    is_focused: bool,
) -> Option<Rectangle> {
    if !is_focused {
        return None;
    }

    let anchor = match input_method {
        input_method::InputMethod::Enabled { cursor, .. } => Some(*cursor),
        input_method::InputMethod::Disabled if activation.web_input_anchor.is_some() => {
            activation.web_input_anchor
        }
        input_method::InputMethod::Disabled if started_focused != is_focused => {
            visible_bounds.map(|bounds| {
                Rectangle::new(bounds.position(), Size::new(1.0, bounds.height.max(1.0)))
            })
        }
        input_method::InputMethod::Disabled => None,
    }?;

    let anchor = visible_bounds.map_or(anchor, |bounds| {
        let right = bounds.x + bounds.width;
        let bottom = bounds.y + bounds.height;

        Rectangle::new(
            Point::new(
                anchor.x.clamp(bounds.x, right),
                anchor.y.clamp(bounds.y, bottom),
            ),
            Size::new(anchor.width.max(1.0), anchor.height.max(1.0)),
        )
    });

    Some(anchor - activation.web_input_translation)
}

fn sync_mobile_keyboard(
    started_focused: bool,
    is_focused: bool,
    request_mobile_keyboard: bool,
    input_anchor: Option<Rectangle>,
) {
    if let Some(anchor) = input_anchor {
        web_input::position_mobile_keyboard(anchor);
    }

    if started_focused != is_focused {
        if is_focused {
            if !request_mobile_keyboard {
                web_input::show_mobile_keyboard();
            }
        } else {
            web_input::hide_mobile_keyboard();
        }
    }

    if request_mobile_keyboard {
        web_input::show_mobile_keyboard();
    }
}

fn register_mobile_text_region(is_enabled: bool, bounds: Rectangle, viewport: &Rectangle) {
    if is_enabled && let Some(visible_bounds) = bounds.intersection(viewport) {
        web_input::register_text_region(visible_bounds);
    }
}

struct TextInputUpdateContext<'a, 'b, Message, Renderer> {
    renderer: &'a Renderer,
    clipboard: &'a mut dyn Clipboard,
    shell: &'a mut Shell<'b, Message>,
    viewport: &'a Rectangle,
}

fn update_mobile_text_input<'a, Message, Renderer>(
    input: &mut IcedTextInput<'a, Message, Theme, Renderer>,
    tree: &mut Tree,
    event: &Event,
    layout: Layout<'_>,
    activation: TextInputActivation,
    context: TextInputUpdateContext<'_, '_, Message, Renderer>,
) where
    Message: Clone,
    Renderer: iced_widget::core::Renderer + core_text::Renderer,
{
    match activation.inner_touch_handling {
        TextFieldInnerTouchHandling::Forward => {
            input.update(
                tree,
                event,
                layout,
                activation.cursor,
                context.renderer,
                &mut *context.clipboard,
                &mut *context.shell,
                context.viewport,
            );
        }
        TextFieldInnerTouchHandling::Suppress => {}
        TextFieldInnerTouchHandling::ConfirmedOutsideTap => {
            // Defer blur until a real outside tap, so another field can be used
            // as a scroll origin without dismissing the current keyboard.
            input.update(
                tree,
                &Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
                layout,
                mouse::Cursor::Unavailable,
                context.renderer,
                &mut *context.clipboard,
                &mut *context.shell,
                context.viewport,
            );
        }
        TextFieldInnerTouchHandling::ConfirmedTap => {
            let press = Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left));
            input.update(
                tree,
                &press,
                layout,
                activation.cursor,
                context.renderer,
                &mut *context.clipboard,
                &mut *context.shell,
                context.viewport,
            );

            let release = Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left));
            input.update(
                tree,
                &release,
                layout,
                activation.cursor,
                context.renderer,
                &mut *context.clipboard,
                &mut *context.shell,
                context.viewport,
            );
        }
    }

    refresh_text_input_caret(
        tree.state
            .downcast_mut::<iced_text_input::State<Renderer::Paragraph>>(),
        event,
        &mut *context.shell,
    );
}

fn press_is_over(event: &Event, bounds: Rectangle, cursor: mouse::Cursor) -> bool {
    match event {
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => cursor.is_over(bounds),
        Event::Touch(touch::Event::FingerPressed { position, .. }) => {
            touch_event_is_over(*position, bounds, cursor)
        }
        _ => false,
    }
}

fn release_is_over(event: &Event, bounds: Rectangle, cursor: mouse::Cursor) -> bool {
    match event {
        Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => cursor.is_over(bounds),
        Event::Touch(touch::Event::FingerLifted { position, .. }) => {
            touch_event_is_over(*position, bounds, cursor)
        }
        _ => false,
    }
}

fn touch_event_is_over(position: Point, bounds: Rectangle, cursor: mouse::Cursor) -> bool {
    if cursor.position().is_some() {
        return cursor.is_over(bounds);
    }

    if cursor.is_levitating() {
        return false;
    }

    bounds.contains(position)
}

fn selection_control_hit_bounds(layout: Layout<'_>, target_size: f32) -> Rectangle {
    let content_bounds = layout.bounds();
    let control_bounds = layout
        .children()
        .next()
        .map_or(content_bounds, |control| control.bounds());

    SelectionControlHitTarget {
        content: content_bounds,
        control: control_bounds,
        target_size,
    }
    .bounds()
}

#[derive(Debug, Clone, Copy)]
struct SelectionControlHitTarget {
    content: Rectangle,
    control: Rectangle,
    target_size: f32,
}

impl SelectionControlHitTarget {
    fn bounds(self) -> Rectangle {
        let target_height = self.content.height.max(self.target_size);
        let content_target = Rectangle {
            y: self.content.center_y() - target_height / 2.0,
            height: target_height,
            ..self.content
        };
        let control_padding =
            ((self.target_size - self.control.width.min(self.control.height)) / 2.0).max(0.0);
        let control_target = Rectangle {
            x: self.control.x - control_padding,
            y: self.control.y - control_padding,
            width: self.control.width + control_padding * 2.0,
            height: self.control.height + control_padding * 2.0,
        };

        union_bounds(content_target, control_target)
    }
}

fn union_bounds(a: Rectangle, b: Rectangle) -> Rectangle {
    let x = a.x.min(b.x);
    let y = a.y.min(b.y);
    let right = (a.x + a.width).max(b.x + b.width);
    let bottom = (a.y + a.height).max(b.y + b.height);

    Rectangle {
        x,
        y,
        width: right - x,
        height: bottom - y,
    }
}

fn should_suppress_ime_caret() -> bool {
    !cfg!(any(
        target_arch = "wasm32",
        target_os = "android",
        target_os = "windows"
    ))
}

fn text_caret_refresh_event(event: &Event) -> bool {
    match event {
        Event::Keyboard(iced_widget::core::keyboard::Event::KeyPressed { key, text, .. }) => {
            text.as_ref()
                .is_some_and(|text| text.chars().any(|c| !c.is_control()))
                || matches!(
                    key.as_ref(),
                    iced_widget::core::keyboard::Key::Named(
                        iced_widget::core::keyboard::key::Named::Enter
                            | iced_widget::core::keyboard::key::Named::Backspace
                            | iced_widget::core::keyboard::key::Named::Delete
                    )
                )
        }
        Event::InputMethod(input_method::Event::Preedit(content, _)) => !content.is_empty(),
        Event::InputMethod(input_method::Event::Commit(content)) => !content.is_empty(),
        _ => false,
    }
}

fn refresh_text_input_caret<Message, P>(
    state: &mut iced_text_input::State<P>,
    event: &Event,
    shell: &mut Shell<'_, Message>,
) where
    P: core_text::Paragraph,
{
    if !state.is_focused() || !text_caret_refresh_event(event) {
        return;
    }

    let value = {
        let text = <iced_text_input::State<P> as core_widget::operation::TextInput>::text(state);
        iced_text_input::Value::new(text)
    };
    let cursor = state.cursor().state(&value);

    core_widget::operation::Focusable::focus(state);

    match cursor {
        iced_text_input::cursor::State::Index(index) => {
            state.move_cursor_to(index);
        }
        iced_text_input::cursor::State::Selection { start, end } => {
            state.select_range(start, end);
        }
    }

    shell.request_redraw();
}

fn mobile_text_input<'a, Message, Renderer>(
    input: IcedTextInput<'a, Message, Theme, Renderer>,
    is_enabled: bool,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
{
    Element::new(MobileTextInput { input, is_enabled })
}

struct MobileTextInput<'a, Message, Renderer>
where
    Renderer: iced_widget::core::Renderer + core_text::Renderer,
{
    input: IcedTextInput<'a, Message, Theme, Renderer>,
    is_enabled: bool,
}

impl<Message, Renderer> Widget<Message, Theme, Renderer> for MobileTextInput<'_, Message, Renderer>
where
    Message: Clone,
    Renderer: iced_widget::core::Renderer + core_text::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<MobileTextInputState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(MobileTextInputState::default())
    }

    fn children(&self) -> Vec<Tree> {
        let input: &dyn Widget<Message, Theme, Renderer> = &self.input;

        vec![Tree::new(input)]
    }

    fn diff(&self, tree: &mut Tree) {
        if tree.children.is_empty() {
            tree.children = self.children();
        } else {
            self.input.diff(&mut tree.children[0]);
            tree.children.truncate(1);
        }
    }

    fn size(&self) -> Size<Length> {
        Widget::<Message, Theme, Renderer>::size(&self.input)
    }

    fn size_hint(&self) -> Size<Length> {
        Widget::<Message, Theme, Renderer>::size_hint(&self.input)
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let input = <IcedTextInput<'_, Message, Theme, Renderer> as Widget<
            Message,
            Theme,
            Renderer,
        >>::layout(&mut self.input, &mut tree.children[0], renderer, limits);

        layout::Node::with_children(input.size(), vec![input])
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn core_widget::Operation,
    ) {
        self.input.operate(
            &mut tree.children[0],
            layout.children().next().unwrap(),
            renderer,
            operation,
        );
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
        let visible_bounds = bounds.intersection(viewport);
        let input_layout = layout.children().next().unwrap();

        let started_focused = {
            let state = tree.children[0]
                .state
                .downcast_ref::<iced_text_input::State<Renderer::Paragraph>>();

            state.is_focused()
        };

        let activation = mobile_text_input_activation(
            self.is_enabled,
            tree.state.downcast_mut::<MobileTextInputState>(),
            event,
            visible_bounds,
            cursor,
        );

        update_mobile_text_input(
            &mut self.input,
            &mut tree.children[0],
            event,
            input_layout,
            activation,
            TextInputUpdateContext {
                renderer,
                clipboard,
                shell,
                viewport,
            },
        );

        normalize_windows_ime_request(shell.input_method_mut(), bounds);

        let is_focused = {
            let state = tree.children[0]
                .state
                .downcast_ref::<iced_text_input::State<Renderer::Paragraph>>();

            state.is_focused()
        };

        let input_anchor = web_input_anchor(
            shell.input_method(),
            visible_bounds,
            activation,
            started_focused,
            is_focused,
        );

        sync_mobile_keyboard(
            started_focused,
            is_focused,
            activation.request_mobile_keyboard,
            input_anchor,
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
        register_mobile_text_region(self.is_enabled, layout.bounds(), viewport);

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
    }
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

#[path = "widget/component/button.rs"]
pub mod button;
#[path = "widget/component/slider.rs"]
pub mod slider;

#[path = "widget/component/rule.rs"]
pub mod rule;

#[path = "widget/component/container.rs"]
pub mod container;

#[path = "widget/component/text_input.rs"]
pub mod text_input;
#[path = "widget/component/tooltip.rs"]
pub mod tooltip;

#[path = "widget/component/text_editor.rs"]
pub mod text_editor;

#[path = "widget/component/radio.rs"]
pub mod radio;

#[path = "widget/component/checkbox.rs"]
pub mod checkbox;

#[path = "widget/component/toggler.rs"]
pub mod toggler;

#[cfg(test)]
#[path = "../tests/widget.rs"]
mod tests;
