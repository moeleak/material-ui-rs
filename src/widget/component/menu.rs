//! Material 3 context menus: the actions for a piece of content, opening at
//! the pointer when it is right-clicked, as for an item in a list or grid.
//!
//! ```ignore
//! menu::context(cover)
//!     .item(menu::item("Play").icon("play_arrow").on_press(Message::Play(id)))
//!     .item(menu::item("Show in folder").icon("folder_open").on_press(Message::Reveal(id)))
//!     .divider()
//!     .item(menu::item("Remove").icon("delete").destructive().on_press(Message::Remove(id)))
//! ```
//!
//! The menu opens below and to the right of the pointer, or on the side that
//! has room for it. A click outside of it, <kbd>Esc</kbd>, or losing focus
//! closes it; the arrow keys and <kbd>Enter</kbd> pick an item.

use iced_widget::core::keyboard::{self, key};
use iced_widget::core::text::{self as core_text, Paragraph as _};
use iced_widget::core::time::Instant;
use iced_widget::core::widget::{Operation, Tree, tree};
use iced_widget::core::{
    Clipboard, Color, Element, Event, Font, Layout, Length, Pixels, Point, Rectangle, Shell, Size,
    Vector, Widget, alignment, layout, mouse, overlay, renderer, touch, window,
};

use super::alpha_color;
use super::menu_overlay::{alpha_menu_style, menu_shadow_bounds};
use super::reveal::RevealAnimation;
use crate::{Theme, fonts, style::menu as menu_style, tokens};

use tokens::component::menu as menu_tokens;

/// An action in a menu: a label, optionally after an icon, that is disabled
/// until it has a message to produce.
#[derive(Debug, Clone, PartialEq)]
pub struct Action<Message> {
    label: String,
    icon: Option<String>,
    on_press: Option<Message>,
    destructive: bool,
}

/// An action labeled `label`.
pub fn item<Message>(label: impl Into<String>) -> Action<Message> {
    Action {
        label: label.into(),
        icon: None,
        on_press: None,
        destructive: false,
    }
}

impl<Message> Action<Message> {
    /// Shows the Material Symbols icon `name` before the label.
    pub fn icon(mut self, name: impl Into<String>) -> Self {
        self.icon = Some(name.into());
        self
    }

    /// Produces `message` when the action is picked.
    pub fn on_press(mut self, message: Message) -> Self {
        self.on_press = Some(message);
        self
    }

    /// Produces `message`, if any, when the action is picked; it is disabled otherwise.
    pub fn on_press_maybe(mut self, message: Option<Message>) -> Self {
        self.on_press = message;
        self
    }

    /// Shows the action in the error color, for actions that delete or remove.
    pub fn destructive(mut self) -> Self {
        self.destructive = true;
        self
    }

    /// The label of the action.
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Whether the action can be picked.
    pub fn is_enabled(&self) -> bool {
        self.on_press.is_some()
    }

    /// Whether the action deletes or removes something.
    pub fn is_destructive(&self) -> bool {
        self.destructive
    }
}

/// An entry in a menu.
#[derive(Debug, Clone, PartialEq)]
pub enum Item<Message> {
    /// An action to pick.
    Action(Action<Message>),
    /// A line between groups of actions.
    Divider,
}

impl<Message> From<Action<Message>> for Item<Message> {
    fn from(action: Action<Message>) -> Self {
        Self::Action(action)
    }
}

/// A line between groups of actions.
pub fn divider<Message>() -> Item<Message> {
    Item::Divider
}

impl<Message> Item<Message> {
    /// The height of the entry in a menu.
    pub fn height(&self) -> f32 {
        match self {
            Self::Action(_) => menu_tokens::LIST_ITEM_CONTAINER_HEIGHT,
            Self::Divider => {
                menu_tokens::DIVIDER_VERTICAL_SPACE * 2.0 + menu_tokens::DIVIDER_THICKNESS
            }
        }
    }

    fn enabled_action(&self) -> Option<&Action<Message>> {
        match self {
            Self::Action(action) if action.is_enabled() => Some(action),
            _ => None,
        }
    }
}

/// `content` with a menu of actions that opens at the pointer when it is
/// right-clicked; add the actions with [`ContextMenu::item`].
pub fn context<'a, Message, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> ContextMenu<'a, Message, Renderer> {
    ContextMenu {
        content: content.into(),
        items: Vec::new(),
        on_open: None,
    }
}

/// Content with a context menu; see [`context`].
pub struct ContextMenu<'a, Message, Renderer = iced_widget::Renderer> {
    content: Element<'a, Message, Theme, Renderer>,
    items: Vec<Item<Message>>,
    on_open: Option<Message>,
}

impl<Message, Renderer> std::fmt::Debug for ContextMenu<'_, Message, Renderer>
where
    Message: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ContextMenu")
            .field("items", &self.items)
            .field("on_open", &self.on_open)
            .finish_non_exhaustive()
    }
}

impl<Message, Renderer> ContextMenu<'_, Message, Renderer> {
    /// Adds an entry, such as an [`item`], to the menu.
    pub fn item(mut self, item: impl Into<Item<Message>>) -> Self {
        self.items.push(item.into());
        self
    }

    /// Adds a [`divider`] to the menu.
    pub fn divider(mut self) -> Self {
        self.items.push(Item::Divider);
        self
    }

    /// Adds `items` to the menu.
    pub fn items(mut self, items: impl IntoIterator<Item = Item<Message>>) -> Self {
        self.items.extend(items);
        self
    }

    /// Produces `message` when the menu opens, as for selecting the content.
    pub fn on_open(mut self, message: Message) -> Self {
        self.on_open = Some(message);
        self
    }
}

#[derive(Debug, Default)]
struct State {
    /// Where the menu was opened, in the coordinates of the content.
    anchor: Option<Point>,
    animation: RevealAnimation,
    hovered: Option<usize>,
}

impl State {
    fn open(&mut self, anchor: Point, now: Instant) {
        self.anchor = Some(anchor);
        self.hovered = None;
        self.animation = RevealAnimation::closed();
        self.animation.open(now);
    }

    fn close(&mut self, now: Instant) {
        self.hovered = None;
        self.animation.close(now);
    }

    fn is_shown(&self) -> bool {
        self.anchor.is_some() && self.animation.is_visible()
    }
}

impl<Message, Renderer> Widget<Message, Theme, Renderer> for ContextMenu<'_, Message, Renderer>
where
    Message: Clone,
    Renderer: core_text::Renderer,
    Font: Into<Renderer::Font>,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
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
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
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
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );

        let state = tree.state.downcast_mut::<State>();

        if let Event::Window(window::Event::RedrawRequested(now)) = event {
            if state.animation.advance(*now) {
                shell.request_redraw();
            } else if !state.animation.is_visible() {
                state.anchor = None;
            }

            return;
        }

        if shell.is_event_captured() || self.items.is_empty() {
            return;
        }

        if let Some(anchor) = opening_position(event, cursor, layout.bounds()) {
            state.open(anchor, Instant::now());

            if let Some(on_open) = &self.on_open {
                shell.publish(on_open.clone());
            }

            shell.capture_event();
            shell.request_redraw();
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
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
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        let Tree {
            state, children, ..
        } = tree;
        let state = state.downcast_mut::<State>();

        if state.is_shown()
            && !self.items.is_empty()
            && let Some(anchor) = state.anchor
        {
            return Some(overlay::Element::new(Box::new(Overlay {
                state,
                items: &self.items,
                anchor: anchor + translation,
                opens_down: true,
                font: renderer.default_font(),
            })));
        }

        self.content.as_widget_mut().overlay(
            &mut children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message, Renderer> From<ContextMenu<'a, Message, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: core_text::Renderer + 'a,
    Font: Into<Renderer::Font>,
{
    fn from(menu: ContextMenu<'a, Message, Renderer>) -> Self {
        Element::new(menu)
    }
}

/// Where a menu opens for `event` on content with `bounds`: at the pointer,
/// if the content was right-clicked or the menu key was pressed over it.
fn opening_position(event: &Event, cursor: mouse::Cursor, bounds: Rectangle) -> Option<Point> {
    match event {
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right))
        | Event::Keyboard(keyboard::Event::KeyPressed {
            key: keyboard::Key::Named(key::Named::ContextMenu),
            ..
        }) => cursor.position_over(bounds),
        _ => None,
    }
}

/// The width of a menu whose actions are `widths` wide, within the Material bounds.
pub fn menu_width(widths: impl IntoIterator<Item = f32>) -> f32 {
    widths
        .into_iter()
        .fold(menu_tokens::CONTAINER_MIN_WIDTH, f32::max)
        .min(menu_tokens::CONTAINER_MAX_WIDTH)
}

/// The width of an action whose label is `label_width` wide.
pub fn action_width(label_width: f32, has_icon: bool) -> f32 {
    let icon = if has_icon {
        menu_tokens::LIST_ITEM_ICON_SIZE + menu_tokens::LIST_ITEM_ICON_LABEL_SPACE
    } else {
        0.0
    };

    menu_tokens::LIST_ITEM_LEADING_SPACE
        + icon
        + label_width
        + menu_tokens::LIST_ITEM_TRAILING_SPACE
}

/// The height of a menu of `items`.
pub fn menu_height<Message>(items: &[Item<Message>]) -> f32 {
    menu_tokens::TOP_SPACE + items.iter().map(Item::height).sum::<f32>() + menu_tokens::BOTTOM_SPACE
}

/// Where a menu of `size` opening at `anchor` goes in a window of `bounds`,
/// and whether it opens downwards: below and to the right of the anchor, or
/// flipped to the side that has room, and kept in the window.
pub fn menu_position(anchor: Point, size: Size, bounds: Size) -> (Point, bool) {
    let x = if anchor.x + size.width <= bounds.width {
        anchor.x
    } else {
        anchor.x - size.width
    };

    let opens_down = anchor.y + size.height <= bounds.height || anchor.y < size.height;
    let y = if opens_down {
        anchor.y
    } else {
        anchor.y - size.height
    };

    let clamp = |value: f32, size: f32, bound: f32| value.min(bound - size).max(0.0);

    (
        Point::new(
            clamp(x, size.width, bounds.width),
            clamp(y, size.height, bounds.height),
        ),
        opens_down,
    )
}

/// The bounds of each of `items` in a menu with `bounds`.
pub fn item_bounds<Message>(
    bounds: Rectangle,
    items: &[Item<Message>],
) -> impl Iterator<Item = Rectangle> + '_ {
    items
        .iter()
        .scan(bounds.y + menu_tokens::TOP_SPACE, move |y, item| {
            let height = item.height();
            let item_bounds = Rectangle {
                x: bounds.x,
                y: *y,
                width: bounds.width,
                height,
            };

            *y += height;
            Some(item_bounds)
        })
}

/// The index of the enabled action under `point` in a menu of `items` with `bounds`.
pub fn action_at<Message>(
    bounds: Rectangle,
    items: &[Item<Message>],
    point: Point,
) -> Option<usize> {
    item_bounds(bounds, items)
        .zip(items)
        .position(|(item_bounds, item)| {
            item.enabled_action().is_some() && item_bounds.contains(point)
        })
}

/// The enabled action after `current` in a menu of `items`, or before it if
/// not `forward`, wrapping around.
pub fn next_action<Message>(
    items: &[Item<Message>],
    current: Option<usize>,
    forward: bool,
) -> Option<usize> {
    let count = items.len();

    (1..=count)
        .map(|step| match (current, forward) {
            (None, true) => step - 1,
            (None, false) => count - step,
            (Some(index), true) => (index + step) % count,
            (Some(index), false) => (index + count - step % count) % count,
        })
        .find(|&index| items[index].enabled_action().is_some())
}

struct Overlay<'a, Message, Renderer>
where
    Renderer: core_text::Renderer,
{
    state: &'a mut State,
    items: &'a [Item<Message>],
    anchor: Point,
    opens_down: bool,
    font: Renderer::Font,
}

impl<Message, Renderer> Overlay<'_, Message, Renderer>
where
    Message: Clone,
    Renderer: core_text::Renderer,
    Font: Into<Renderer::Font>,
{
    fn label_width(&self, label: &str) -> f32 {
        let text = menu_tokens::LIST_ITEM_LABEL_TEXT;

        Renderer::Paragraph::with_text(core_text::Text {
            content: label,
            bounds: Size::INFINITE,
            size: Pixels(text.size),
            line_height: core_text::LineHeight::Absolute(Pixels(text.line_height)),
            font: self.font,
            align_x: core_text::Alignment::Default,
            align_y: alignment::Vertical::Center,
            shaping: core_text::Shaping::Advanced,
            wrapping: core_text::Wrapping::None,
        })
        .min_width()
    }

    fn dismiss(&mut self, shell: &mut Shell<'_, Message>) {
        self.state.close(Instant::now());
        shell.request_redraw();
    }

    fn activate(&mut self, index: usize, shell: &mut Shell<'_, Message>) {
        if let Some(action) = self.items.get(index).and_then(Item::enabled_action)
            && let Some(message) = &action.on_press
        {
            shell.publish(message.clone());
        }

        self.dismiss(shell);
    }

    fn press(
        &mut self,
        bounds: Rectangle,
        position: Option<Point>,
        button: mouse::Button,
        shell: &mut Shell<'_, Message>,
    ) {
        match position {
            Some(position) if bounds.contains(position) => {
                if button == mouse::Button::Left
                    && let Some(index) = action_at(bounds, self.items, position)
                {
                    self.activate(index, shell);
                }

                shell.capture_event();
            }
            _ => {
                self.dismiss(shell);

                // A right-click elsewhere opens the menu of what is under it.
                if button != mouse::Button::Right {
                    shell.capture_event();
                }
            }
        }
    }

    fn hover(&mut self, index: Option<usize>, shell: &mut Shell<'_, Message>) {
        if self.state.hovered != index {
            self.state.hovered = index;
            shell.request_redraw();
        }
    }
}

impl<Message, Renderer> iced_widget::core::Overlay<Message, Theme, Renderer>
    for Overlay<'_, Message, Renderer>
where
    Message: Clone,
    Renderer: core_text::Renderer,
    Font: Into<Renderer::Font>,
{
    fn layout(&mut self, _renderer: &Renderer, bounds: Size) -> layout::Node {
        let width = menu_width(self.items.iter().filter_map(|item| match item {
            Item::Action(action) => Some(action_width(
                self.label_width(&action.label),
                action.icon.is_some(),
            )),
            Item::Divider => None,
        }));
        let size = Size::new(width, menu_height(self.items));
        let (position, opens_down) = menu_position(self.anchor, size, bounds);

        self.opens_down = opens_down;

        layout::Node::new(size).move_to(position)
    }

    fn update(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) {
        if self.state.animation.frame().is_closing {
            return;
        }

        let bounds = layout.bounds();

        match event {
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                let hovered = cursor
                    .position()
                    .and_then(|position| action_at(bounds, self.items, position));

                self.hover(hovered, shell);
            }
            Event::Mouse(mouse::Event::ButtonPressed(button)) => {
                self.press(bounds, cursor.position(), *button, shell);
            }
            Event::Touch(touch::Event::FingerPressed { position, .. }) => {
                self.press(bounds, Some(*position), mouse::Button::Left, shell);
            }
            Event::Mouse(mouse::Event::WheelScrolled { .. }) => {
                if cursor.is_over(bounds) {
                    shell.capture_event();
                } else {
                    self.dismiss(shell);
                }
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(named),
                ..
            }) => match named {
                key::Named::Escape => {
                    self.dismiss(shell);
                    shell.capture_event();
                }
                key::Named::ArrowDown | key::Named::ArrowUp => {
                    let next = next_action(
                        self.items,
                        self.state.hovered,
                        *named == key::Named::ArrowDown,
                    );

                    self.hover(next, shell);
                    shell.capture_event();
                }
                key::Named::Enter | key::Named::Space => {
                    if let Some(index) = self.state.hovered {
                        self.activate(index, shell);
                    }

                    shell.capture_event();
                }
                _ => {}
            },
            Event::Window(window::Event::Unfocused | window::Event::Resized(_)) => {
                self.dismiss(shell);
            }
            _ => {}
        }
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        let bounds = layout.bounds();

        match cursor.position() {
            Some(position) if bounds.contains(position) => {
                if action_at(bounds, self.items, position).is_some() {
                    mouse::Interaction::Pointer
                } else {
                    mouse::Interaction::Idle
                }
            }
            _ => mouse::Interaction::None,
        }
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
    ) {
        let frame = self.state.animation.frame();
        let alpha = frame.alpha;

        if alpha <= 0.001 {
            return;
        }

        let bounds = layout.bounds();
        let colors = theme.colors();
        let style = alpha_menu_style(menu_style::default(theme), alpha);
        let revealed = revealed_bounds(bounds, frame.reveal, self.opens_down);

        renderer.with_layer(menu_shadow_bounds(revealed), |renderer| {
            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border: style.border,
                    shadow: style.shadow,
                    ..renderer::Quad::default()
                },
                style.background,
            );

            for (index, (item, item_bounds)) in self
                .items
                .iter()
                .zip(item_bounds(bounds, self.items))
                .enumerate()
            {
                match item {
                    Item::Divider => renderer.fill_quad(
                        renderer::Quad {
                            bounds: Rectangle {
                                y: item_bounds.center_y() - menu_tokens::DIVIDER_THICKNESS / 2.0,
                                height: menu_tokens::DIVIDER_THICKNESS,
                                ..item_bounds
                            },
                            ..renderer::Quad::default()
                        },
                        alpha_color(colors.outline.variant, alpha),
                    ),
                    Item::Action(action) => self.draw_action(
                        renderer,
                        theme,
                        action,
                        item_bounds,
                        self.state.hovered == Some(index),
                        alpha,
                    ),
                }
            }
        });
    }
}

impl<Message, Renderer> Overlay<'_, Message, Renderer>
where
    Renderer: core_text::Renderer,
    Font: Into<Renderer::Font>,
{
    fn draw_action(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        action: &Action<Message>,
        bounds: Rectangle,
        is_hovered: bool,
        alpha: f32,
    ) {
        let colors = action_colors(theme, action);
        let text = menu_tokens::LIST_ITEM_LABEL_TEXT;
        let icon_size = menu_tokens::LIST_ITEM_ICON_SIZE;
        let mut x = bounds.x + menu_tokens::LIST_ITEM_LEADING_SPACE;

        if is_hovered {
            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    ..renderer::Quad::default()
                },
                alpha_color(
                    Color {
                        a: tokens::state::HOVER_STATE_LAYER_OPACITY,
                        ..colors.label
                    },
                    alpha,
                ),
            );
        }

        if let Some(icon) = &action.icon {
            renderer.fill_text(
                core_text::Text {
                    content: fonts::material_symbol_fragment(icon.as_str()).into_owned(),
                    bounds: Size::new(icon_size, icon_size),
                    size: Pixels(icon_size),
                    line_height: core_text::LineHeight::Absolute(Pixels(icon_size)),
                    font: fonts::MATERIAL_SYMBOLS_ROUNDED.into(),
                    align_x: core_text::Alignment::Center,
                    align_y: alignment::Vertical::Center,
                    shaping: core_text::Shaping::Advanced,
                    wrapping: core_text::Wrapping::None,
                },
                Point::new(x + icon_size / 2.0, bounds.center_y()),
                alpha_color(colors.icon, alpha),
                bounds,
            );

            x += icon_size + menu_tokens::LIST_ITEM_ICON_LABEL_SPACE;
        }

        let label_bounds = Rectangle {
            x,
            width: (bounds.x + bounds.width - menu_tokens::LIST_ITEM_TRAILING_SPACE - x).max(0.0),
            ..bounds
        };

        renderer.fill_text(
            core_text::Text {
                content: action.label.clone(),
                bounds: label_bounds.size(),
                size: Pixels(text.size),
                line_height: core_text::LineHeight::Absolute(Pixels(text.line_height)),
                font: self.font,
                align_x: core_text::Alignment::Default,
                align_y: alignment::Vertical::Center,
                shaping: core_text::Shaping::Advanced,
                wrapping: core_text::Wrapping::None,
            },
            Point::new(label_bounds.x, label_bounds.center_y()),
            alpha_color(colors.label, alpha),
            label_bounds,
        );
    }
}

/// The colors of an action's label and icon.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ActionColors {
    pub label: Color,
    pub icon: Color,
}

/// The colors of `action` in `theme`: the error color for destructive
/// actions, and faded for disabled ones.
pub fn action_colors<Message>(theme: &Theme, action: &Action<Message>) -> ActionColors {
    let colors = theme.colors();

    if !action.is_enabled() {
        let disabled = Color {
            a: tokens::state::DISABLED_LABEL_TEXT_OPACITY,
            ..colors.surface.text
        };

        return ActionColors {
            label: disabled,
            icon: disabled,
        };
    }

    if action.destructive {
        ActionColors {
            label: colors.error.color,
            icon: colors.error.color,
        }
    } else {
        ActionColors {
            label: colors.surface.text,
            icon: colors.surface.text_variant,
        }
    }
}

/// The part of a menu with `bounds` shown `reveal` (0 to 1) of the way open,
/// from its top if it opens downwards.
fn revealed_bounds(bounds: Rectangle, reveal: f32, opens_down: bool) -> Rectangle {
    let height = bounds.height * reveal;

    if opens_down {
        Rectangle { height, ..bounds }
    } else {
        Rectangle {
            y: bounds.y + bounds.height - height,
            height,
            ..bounds
        }
    }
}

#[cfg(test)]
#[path = "../../../tests/widget/component/menu.rs"]
mod tests;
