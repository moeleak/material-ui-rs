use std::cell::Cell;

use iced_widget::core::alignment;
use iced_widget::core::layout::{self, Layout};
use iced_widget::core::mouse;
use iced_widget::core::overlay;
use iced_widget::core::renderer;
use iced_widget::core::text::{self, Text};
use iced_widget::core::time::Instant;
use iced_widget::core::touch;
use iced_widget::core::widget::Widget;
use iced_widget::core::widget::tree::{self, Tree};
use iced_widget::core::window;
use iced_widget::core::{
    Clipboard, Element, Event, Length, Padding, Pixels, Point, Rectangle, Shell, Size, Vector,
    border,
};
use iced_widget::scrollable::{self, Scrollable};

use super::reveal::{RevealAnimation, RevealFrame};
use super::support::{alpha_border, alpha_color};
use crate::tokens;

#[cfg(test)]
const CLOSED_ALPHA_TARGET: f32 = 0.0;
const EXPANDED_ALPHA_TARGET: f32 = 1.0;

fn selected_option_border() -> iced_widget::core::Border {
    border::rounded(0.0)
}

pub(super) struct Menu<'a, 'b, T, Message, Theme = crate::Theme, Renderer = iced_widget::Renderer>
where
    Theme: iced_widget::overlay::menu::Catalog,
    Renderer: text::Renderer,
    'b: 'a,
{
    state: &'a mut State,
    options: &'a [T],
    hovered_option: &'a mut Option<usize>,
    on_selected: Box<dyn FnMut(T) -> Message + 'a>,
    on_option_hovered: Option<&'a dyn Fn(T) -> Message>,
    on_dismiss: Option<Box<dyn FnMut(Instant) + 'a>>,
    width: f32,
    padding: Padding,
    text_size: Option<Pixels>,
    text_line_height: text::LineHeight,
    text_shaping: text::Shaping,
    font: Option<Renderer::Font>,
    class: &'a <Theme as iced_widget::overlay::menu::Catalog>::Class<'b>,
}

impl<'a, 'b, T, Message, Theme, Renderer> Menu<'a, 'b, T, Message, Theme, Renderer>
where
    T: ToString + Clone,
    Message: 'a,
    Theme: iced_widget::overlay::menu::Catalog + 'a,
    Renderer: text::Renderer + 'a,
    'b: 'a,
{
    pub(super) fn new(
        state: &'a mut State,
        options: &'a [T],
        hovered_option: &'a mut Option<usize>,
        on_selected: impl FnMut(T) -> Message + 'a,
        on_option_hovered: Option<&'a dyn Fn(T) -> Message>,
        class: &'a <Theme as iced_widget::overlay::menu::Catalog>::Class<'b>,
    ) -> Self {
        Self {
            state,
            options,
            hovered_option,
            on_selected: Box::new(on_selected),
            on_option_hovered,
            on_dismiss: None,
            width: 0.0,
            padding: Padding::ZERO,
            text_size: None,
            text_line_height: text::LineHeight::default(),
            text_shaping: text::Shaping::default(),
            font: None,
            class,
        }
    }

    pub(super) fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub(super) fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = padding.into();
        self
    }

    pub(super) fn text_size(mut self, text_size: impl Into<Pixels>) -> Self {
        self.text_size = Some(text_size.into());
        self
    }

    pub(super) fn text_line_height(mut self, line_height: impl Into<text::LineHeight>) -> Self {
        self.text_line_height = line_height.into();
        self
    }

    pub(super) fn text_shaping(mut self, shaping: text::Shaping) -> Self {
        self.text_shaping = shaping;
        self
    }

    pub(super) fn font(mut self, font: impl Into<Renderer::Font>) -> Self {
        self.font = Some(font.into());
        self
    }

    pub(super) fn on_dismiss(mut self, on_dismiss: impl FnMut(Instant) + 'a) -> Self {
        self.on_dismiss = Some(Box::new(on_dismiss));
        self
    }

    pub(super) fn overlay(
        self,
        position: Point,
        viewport: Rectangle,
        target_height: f32,
        menu_height: Length,
    ) -> overlay::Element<'a, Message, Theme, Renderer> {
        overlay::Element::new(Box::new(Overlay::new(
            position,
            viewport,
            self,
            target_height,
            menu_height,
        )))
    }
}

#[derive(Debug)]
pub(super) struct State {
    tree: Tree,
    animation: RevealAnimation,
    dismiss_requested: Cell<bool>,
}

impl State {
    pub(super) fn new() -> Self {
        Self {
            tree: Tree::empty(),
            animation: RevealAnimation::closed(),
            dismiss_requested: Cell::new(false),
        }
    }

    pub(super) fn start_open(&mut self, _item_count: usize, now: Instant) {
        self.dismiss_requested.set(false);
        self.animation = RevealAnimation::closed();
        self.animation.open(now);
    }

    pub(super) fn reverse_open(&mut self, now: Instant) {
        self.dismiss_requested.set(false);
        self.animation.open(now);
    }

    pub(super) fn start_close(&mut self, now: Instant) {
        self.dismiss_requested.set(false);
        self.animation.close(now);
    }

    pub(super) fn advance(&mut self, now: Instant) -> bool {
        self.animation.advance(now)
    }

    pub(super) fn is_animating(&self) -> bool {
        self.animation.is_animating()
    }

    pub(super) fn is_visible(&self) -> bool {
        self.animation.is_visible()
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

struct Overlay<'a, 'b, Message, Theme, Renderer>
where
    Theme: iced_widget::overlay::menu::Catalog,
    Renderer: text::Renderer,
{
    position: Point,
    viewport: Rectangle,
    animation: &'a mut RevealAnimation,
    dismiss_requested: &'a Cell<bool>,
    tree: &'a mut Tree,
    list: Scrollable<'a, Message, Theme, Renderer>,
    width: f32,
    target_height: f32,
    class: &'a <Theme as iced_widget::overlay::menu::Catalog>::Class<'b>,
}

impl<'a, 'b, Message, Theme, Renderer> Overlay<'a, 'b, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: iced_widget::overlay::menu::Catalog + scrollable::Catalog + 'a,
    Renderer: text::Renderer + 'a,
    'b: 'a,
{
    fn new<T>(
        position: Point,
        viewport: Rectangle,
        menu: Menu<'a, 'b, T, Message, Theme, Renderer>,
        target_height: f32,
        menu_height: Length,
    ) -> Self
    where
        T: Clone + ToString,
    {
        let Menu {
            state,
            options,
            hovered_option,
            on_selected,
            on_option_hovered,
            on_dismiss,
            width,
            padding,
            font,
            text_size,
            text_line_height,
            text_shaping,
            class,
        } = menu;

        let opens_down = opens_down(position, target_height, viewport.height);
        let _ = state.animation.advance(Instant::now());
        let frame = MenuAnimationFrame::new(state.animation.frame(), opens_down);

        let list = Scrollable::new(List {
            options,
            hovered_option,
            on_selected,
            on_option_hovered,
            on_dismiss,
            dismiss_requested: &state.dismiss_requested,
            font,
            text_size,
            text_line_height,
            text_shaping,
            padding,
            class,
            frame,
        })
        .height(menu_height);

        state.tree.diff(&list as &dyn Widget<_, _, _>);

        Self {
            position,
            viewport,
            animation: &mut state.animation,
            dismiss_requested: &state.dismiss_requested,
            tree: &mut state.tree,
            list,
            width,
            target_height,
            class,
        }
    }
}

impl<Message, Theme, Renderer> iced_widget::core::Overlay<Message, Theme, Renderer>
    for Overlay<'_, '_, Message, Theme, Renderer>
where
    Theme: iced_widget::overlay::menu::Catalog,
    Renderer: text::Renderer,
{
    fn layout(&mut self, renderer: &Renderer, bounds: Size) -> layout::Node {
        let space_below = bounds.height - (self.position.y + self.target_height);
        let space_above = self.position.y;
        let shadow_padding = menu_shadow_padding();

        let limits = layout::Limits::new(
            Size::ZERO,
            Size::new(
                bounds.width - self.position.x,
                if space_below > space_above {
                    space_below
                } else {
                    space_above
                },
            ),
        )
        .width(self.width);

        let node = self.list.layout(self.tree, renderer, &limits);
        let size = node.size();
        let content_position =
            menu_content_position(self.position, self.target_height, size, bounds.height);
        let overlay_position = Point::new(
            content_position.x - shadow_padding,
            content_position.y - shadow_padding,
        );

        layout::Node::with_children(
            Size::new(
                size.width + shadow_padding * 2.0,
                size.height + shadow_padding * 2.0,
            ),
            vec![node.move_to(Point::new(shadow_padding, shadow_padding))],
        )
        .move_to(overlay_position)
    }

    fn update(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) {
        let Some(content_layout) = layout.children().next() else {
            return;
        };
        let bounds = content_layout.bounds();

        self.list.update(
            self.tree,
            event,
            content_layout,
            cursor,
            renderer,
            clipboard,
            shell,
            &bounds,
        );

        if self.dismiss_requested.replace(false) {
            self.animation.close(Instant::now());
            shell.request_redraw();
        }

        if let Event::Window(window::Event::RedrawRequested(now)) = event
            && self.animation.advance(*now)
        {
            shell.request_redraw();
        }
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let Some(content_layout) = layout.children().next() else {
            return mouse::Interaction::default();
        };

        self.list
            .mouse_interaction(self.tree, content_layout, cursor, &self.viewport, renderer)
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        defaults: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        let Some(content_layout) = layout.children().next() else {
            return;
        };
        let bounds = content_layout.bounds();
        let frame = MenuAnimationFrame::new(
            self.animation.frame(),
            opens_down(self.position, self.target_height, self.viewport.height),
        );
        let alpha = frame.alpha();

        if alpha <= 0.001 {
            return;
        }

        let style = alpha_menu_style(
            iced_widget::overlay::menu::Catalog::style(theme, self.class),
            alpha,
        );
        let reveal = frame.reveal_bounds(bounds);
        let reveal_shadow_bounds = menu_shadow_bounds(reveal);

        renderer.with_layer(self.viewport, |renderer| {
            renderer.with_layer(reveal_shadow_bounds, |renderer| {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds,
                        border: style.border,
                        shadow: style.shadow,
                        ..renderer::Quad::default()
                    },
                    style.background,
                );

                self.list.draw(
                    self.tree,
                    renderer,
                    theme,
                    defaults,
                    content_layout,
                    cursor,
                    &bounds,
                );
            });
        });
    }
}

struct List<'a, 'b, T, Message, Theme, Renderer>
where
    Theme: iced_widget::overlay::menu::Catalog,
    Renderer: text::Renderer,
{
    options: &'a [T],
    hovered_option: &'a mut Option<usize>,
    on_selected: Box<dyn FnMut(T) -> Message + 'a>,
    on_option_hovered: Option<&'a dyn Fn(T) -> Message>,
    on_dismiss: Option<Box<dyn FnMut(Instant) + 'a>>,
    dismiss_requested: &'a Cell<bool>,
    padding: Padding,
    text_size: Option<Pixels>,
    text_line_height: text::LineHeight,
    text_shaping: text::Shaping,
    font: Option<Renderer::Font>,
    class: &'a <Theme as iced_widget::overlay::menu::Catalog>::Class<'b>,
    frame: MenuAnimationFrame,
}

#[derive(Debug, Clone, Copy)]
struct ListState {
    is_hovered: Option<bool>,
}

impl<T, Message, Theme, Renderer> iced_widget::core::Widget<Message, Theme, Renderer>
    for List<'_, '_, T, Message, Theme, Renderer>
where
    T: Clone + ToString,
    Theme: iced_widget::overlay::menu::Catalog,
    Renderer: text::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<ListState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(ListState { is_hovered: None })
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: Length::Fill,
            height: Length::Shrink,
        }
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let text_size = self.text_size.unwrap_or_else(|| renderer.default_size());
        let text_line_height = self.text_line_height.to_absolute(text_size);

        let intrinsic = Size::new(
            0.0,
            (f32::from(text_line_height) + self.padding.y()) * self.options.len() as f32,
        );

        layout::Node::new(limits.resolve(Length::Fill, Length::Shrink, intrinsic))
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if self.frame.cursor_visible(cursor, *_viewport)
                    && let Some(index) = *self.hovered_option
                    && let Some(option) = self.options.get(index)
                {
                    if let Some(on_dismiss) = &mut self.on_dismiss {
                        on_dismiss(Instant::now());
                        self.dismiss_requested.set(true);
                    }

                    shell.publish((self.on_selected)(option.clone()));
                    shell.capture_event();
                    shell.request_redraw();
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if let Some(cursor_position) = cursor.position_in(layout.bounds())
                    && self.frame.cursor_visible(cursor, *_viewport)
                {
                    let text_size = self.text_size.unwrap_or_else(|| renderer.default_size());
                    let option_height =
                        f32::from(self.text_line_height.to_absolute(text_size)) + self.padding.y();
                    let new_hovered_option = (cursor_position.y / option_height) as usize;

                    if *self.hovered_option != Some(new_hovered_option)
                        && let Some(option) = self.options.get(new_hovered_option)
                    {
                        if let Some(on_option_hovered) = self.on_option_hovered {
                            shell.publish(on_option_hovered(option.clone()));
                        }

                        shell.request_redraw();
                    }

                    *self.hovered_option = Some(new_hovered_option);
                }
            }
            Event::Touch(touch::Event::FingerPressed { .. }) => {
                if let Some(cursor_position) = cursor.position_in(layout.bounds())
                    && self.frame.cursor_visible(cursor, *_viewport)
                {
                    let text_size = self.text_size.unwrap_or_else(|| renderer.default_size());
                    let option_height =
                        f32::from(self.text_line_height.to_absolute(text_size)) + self.padding.y();
                    let index = (cursor_position.y / option_height) as usize;

                    *self.hovered_option = Some(index);

                    if let Some(option) = self.options.get(index) {
                        if let Some(on_dismiss) = &mut self.on_dismiss {
                            on_dismiss(Instant::now());
                            self.dismiss_requested.set(true);
                        }

                        shell.publish((self.on_selected)(option.clone()));
                        shell.capture_event();
                        shell.request_redraw();
                    }
                }
            }
            _ => {}
        }

        let state = tree.state.downcast_mut::<ListState>();

        if let Event::Window(window::Event::RedrawRequested(_now)) = event {
            state.is_hovered = Some(cursor.is_over(layout.bounds()));
        } else if state
            .is_hovered
            .is_some_and(|is_hovered| is_hovered != cursor.is_over(layout.bounds()))
        {
            shell.request_redraw();
        }
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        if cursor.is_over(layout.bounds()) && self.frame.cursor_visible(cursor, *viewport) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
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
        viewport: &Rectangle,
    ) {
        let style = iced_widget::overlay::menu::Catalog::style(theme, self.class);
        let bounds = layout.bounds();
        let alpha = self.frame.alpha();
        let text_size = self.text_size.unwrap_or_else(|| renderer.default_size());
        let option_height =
            f32::from(self.text_line_height.to_absolute(text_size)) + self.padding.y();

        if alpha <= 0.001 {
            return;
        }

        let offset = viewport.y - bounds.y;
        let start = (offset / option_height).max(0.0) as usize;
        let end = ((offset + viewport.height) / option_height).ceil() as usize;
        let visible_options = &self.options[start..end.min(self.options.len())];

        for (i, option) in visible_options.iter().enumerate() {
            let i = start + i;

            let is_selected = *self.hovered_option == Some(i);
            let bounds = Rectangle {
                x: bounds.x,
                y: bounds.y + option_height * i as f32,
                width: bounds.width,
                height: option_height,
            };

            if is_selected {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle {
                            x: bounds.x + style.border.width,
                            width: bounds.width - style.border.width * 2.0,
                            ..bounds
                        },
                        border: selected_option_border(),
                        ..renderer::Quad::default()
                    },
                    style.selected_background.scale_alpha(alpha),
                );
            }

            renderer.fill_text(
                Text {
                    content: option.to_string(),
                    bounds: Size::new(f32::INFINITY, bounds.height),
                    size: text_size,
                    line_height: self.text_line_height,
                    font: self.font.unwrap_or_else(|| renderer.default_font()),
                    align_x: text::Alignment::Default,
                    align_y: alignment::Vertical::Center,
                    shaping: self.text_shaping,
                    wrapping: text::Wrapping::default(),
                },
                Point::new(bounds.x + self.padding.left, bounds.center_y()),
                alpha_color(
                    if is_selected {
                        style.selected_text_color
                    } else {
                        style.text_color
                    },
                    alpha,
                ),
                *viewport,
            );
        }
    }
}

impl<'a, 'b, T, Message, Theme, Renderer> From<List<'a, 'b, T, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    T: ToString + Clone,
    Message: 'a,
    Theme: 'a + iced_widget::overlay::menu::Catalog,
    Renderer: 'a + text::Renderer,
    'b: 'a,
{
    fn from(list: List<'a, 'b, T, Message, Theme, Renderer>) -> Self {
        Element::new(list)
    }
}

#[derive(Debug, Clone, Copy)]
struct MenuAnimationFrame {
    reveal: f32,
    alpha: f32,
    opens_down: bool,
    is_closing: bool,
}

impl MenuAnimationFrame {
    fn new(frame: RevealFrame, opens_down: bool) -> Self {
        Self {
            reveal: frame.reveal,
            alpha: frame.alpha,
            opens_down,
            is_closing: frame.is_closing,
        }
    }

    fn alpha(&self) -> f32 {
        self.alpha
    }

    fn reveal_bounds(&self, bounds: Rectangle) -> Rectangle {
        let height = bounds.height * self.reveal;

        if self.opens_down {
            Rectangle { height, ..bounds }
        } else {
            Rectangle {
                y: bounds.y + bounds.height - height,
                height,
                ..bounds
            }
        }
    }

    fn cursor_visible(&self, cursor: mouse::Cursor, bounds: Rectangle) -> bool {
        if self.is_closing {
            return false;
        }

        let Some(position) = cursor.position_in(bounds) else {
            return false;
        };

        if self.opens_down {
            position.y <= bounds.height * self.reveal
        } else {
            position.y >= bounds.height * (EXPANDED_ALPHA_TARGET - self.reveal)
        }
    }
}

fn menu_shadow_padding() -> f32 {
    let layer =
        tokens::elevation::shadow(tokens::component::menu::CONTAINER_ELEVATION_LEVEL).ambient;

    layer.blur + layer.y.abs()
}

pub(super) fn menu_shadow_bounds(bounds: Rectangle) -> Rectangle {
    let padding = menu_shadow_padding();

    Rectangle {
        x: bounds.x - padding,
        y: bounds.y - padding,
        width: bounds.width + padding * 2.0,
        height: bounds.height + padding * 2.0,
    }
}

fn menu_content_position(
    position: Point,
    target_height: f32,
    content_size: Size,
    viewport_height: f32,
) -> Point {
    if opens_down(position, target_height, viewport_height) {
        position + Vector::new(0.0, target_height)
    } else {
        position - Vector::new(0.0, content_size.height)
    }
}

fn opens_down(position: Point, target_height: f32, viewport_height: f32) -> bool {
    viewport_height - (position.y + target_height) > position.y
}

pub(super) fn alpha_menu_style(
    mut style: iced_widget::overlay::menu::Style,
    alpha: f32,
) -> iced_widget::overlay::menu::Style {
    style.background = style.background.scale_alpha(alpha);
    style.border = alpha_border(style.border, alpha);
    style.shadow.color = alpha_color(style.shadow.color, alpha);
    style
}

#[cfg(test)]
#[path = "../../../tests/widget/internal/menu_overlay.rs"]
mod tests;
