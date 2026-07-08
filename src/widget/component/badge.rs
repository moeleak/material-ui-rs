//! Material 3 badge constructors with token-backed layout defaults.

use iced_widget::core::text as core_text;
use iced_widget::core::widget::{Operation, Tree};
use iced_widget::core::{
    Clipboard, Element, Event, Layout, Length, Padding, Point, Rectangle, Shell, Size, Vector,
    Widget, alignment, layout, mouse, overlay, renderer,
};
use iced_widget::text;
use iced_widget::{Container, Text};

use super::absolute_line_height;
use crate::{Theme, style::badge as badge_style, tokens};

use std::fmt;

pub fn small<'a, Message, Renderer>() -> Container<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
{
    Container::new(Text::new(""))
        .width(Length::Fixed(tokens::component::badge::SMALL_SIZE))
        .height(Length::Fixed(tokens::component::badge::SMALL_SIZE))
        .style(badge_style::default)
}

pub fn large<'a, Message, Renderer>(
    label: impl text::IntoFragment<'a>,
) -> Container<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
{
    let label_text = tokens::component::badge::LABEL_TEXT;

    Container::new(
        Text::new(label)
            .size(label_text.size)
            .line_height(absolute_line_height(label_text.line_height)),
    )
    .height(Length::Fixed(
        tokens::component::badge::LARGE_CONTAINER_HEIGHT,
    ))
    .max_width(tokens::component::badge::LARGE_CONTAINER_MAX_WIDTH)
    .padding(Padding::from([
        0.0,
        tokens::component::badge::LARGE_HORIZONTAL_SPACE,
    ]))
    .align_x(alignment::Horizontal::Center)
    .align_y(alignment::Vertical::Center)
    .style(badge_style::default)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BadgedBoxPlacement {
    IconOnly,
    WithContent,
}

impl BadgedBoxPlacement {
    fn horizontal_offset(self) -> f32 {
        match self {
            Self::IconOnly => tokens::component::badge::ICON_ONLY_OFFSET,
            Self::WithContent => tokens::component::badge::WITH_CONTENT_HORIZONTAL_OFFSET,
        }
    }

    fn vertical_offset(self) -> f32 {
        match self {
            Self::IconOnly => tokens::component::badge::ICON_ONLY_OFFSET,
            Self::WithContent => tokens::component::badge::WITH_CONTENT_VERTICAL_OFFSET,
        }
    }
}

pub fn badged_box<'a, Message, Renderer>(
    anchor: impl Into<Element<'a, Message, Theme, Renderer>>,
    badge: impl Into<Element<'a, Message, Theme, Renderer>>,
    placement: BadgedBoxPlacement,
) -> BadgedBox<'a, Message, Renderer>
where
    Renderer: iced_widget::core::Renderer,
{
    BadgedBox::new(anchor, badge, placement)
}

pub struct BadgedBox<'a, Message, Renderer = iced_widget::Renderer>
where
    Renderer: iced_widget::core::Renderer,
{
    width: Length,
    height: Length,
    placement: BadgedBoxPlacement,
    children: Vec<Element<'a, Message, Theme, Renderer>>,
}

impl<'a, Message, Renderer> BadgedBox<'a, Message, Renderer>
where
    Renderer: iced_widget::core::Renderer,
{
    pub fn new(
        anchor: impl Into<Element<'a, Message, Theme, Renderer>>,
        badge: impl Into<Element<'a, Message, Theme, Renderer>>,
        placement: BadgedBoxPlacement,
    ) -> Self {
        let anchor = anchor.into();
        let size_hint = anchor.as_widget().size_hint();

        Self {
            width: size_hint.width,
            height: size_hint.height,
            placement,
            children: vec![anchor, badge.into()],
        }
    }
}

impl<Message, Renderer> fmt::Debug for BadgedBox<'_, Message, Renderer>
where
    Renderer: iced_widget::core::Renderer,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BadgedBox")
            .field("width", &self.width)
            .field("height", &self.height)
            .field("placement", &self.placement)
            .finish_non_exhaustive()
    }
}

impl<Message, Renderer> Widget<Message, Theme, Renderer> for BadgedBox<'_, Message, Renderer>
where
    Renderer: iced_widget::core::Renderer,
{
    fn children(&self) -> Vec<Tree> {
        self.children.iter().map(Tree::new).collect()
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&self.children);
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let limits = limits.width(self.width).height(self.height);
        let anchor =
            self.children[0]
                .as_widget_mut()
                .layout(&mut tree.children[0], renderer, &limits);
        let size = limits.resolve(self.width, self.height, anchor.size());
        let badge = self.children[1].as_widget_mut().layout(
            &mut tree.children[1],
            renderer,
            &layout::Limits::NONE,
        );
        let badge_position = badged_box_badge_position(&anchor, &badge, self.placement);

        layout::Node::with_children(
            size,
            vec![anchor.move_to(Point::ORIGIN), badge.move_to(badge_position)],
        )
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        operation.container(None, layout.bounds());
        operation.traverse(&mut |operation| {
            self.children
                .iter_mut()
                .zip(&mut tree.children)
                .zip(layout.children())
                .for_each(|((child, state), layout)| {
                    child
                        .as_widget_mut()
                        .operate(state, layout, renderer, operation);
                });
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
        for ((child, tree), layout) in self
            .children
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
            .rev()
        {
            child.as_widget_mut().update(
                tree, event, layout, cursor, renderer, clipboard, shell, viewport,
            );

            if shell.is_event_captured() {
                return;
            }
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
        self.children
            .iter()
            .rev()
            .zip(tree.children.iter().rev())
            .zip(layout.children().rev())
            .map(|((child, tree), layout)| {
                child
                    .as_widget()
                    .mouse_interaction(tree, layout, cursor, viewport, renderer)
            })
            .find(|interaction| *interaction != mouse::Interaction::None)
            .unwrap_or_default()
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
        if layout.bounds().intersection(viewport).is_some() {
            for (index, ((child, tree), layout)) in self
                .children
                .iter()
                .zip(&tree.children)
                .zip(layout.children())
                .enumerate()
            {
                if index == 0 {
                    child
                        .as_widget()
                        .draw(tree, renderer, theme, style, layout, cursor, viewport);
                } else {
                    renderer.with_layer(*viewport, |renderer| {
                        child
                            .as_widget()
                            .draw(tree, renderer, theme, style, layout, cursor, viewport);
                    });
                }
            }
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        overlay::from_children(
            &mut self.children,
            tree,
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message, Renderer> From<BadgedBox<'a, Message, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + 'a,
{
    fn from(badged_box: BadgedBox<'a, Message, Renderer>) -> Self {
        Self::new(badged_box)
    }
}

fn badged_box_badge_position(
    anchor: &layout::Node,
    badge: &layout::Node,
    placement: BadgedBoxPlacement,
) -> Point {
    let anchor_measured = anchor.size();
    let badge_measured = badge.size();

    Point::new(
        anchor_measured.width - placement.horizontal_offset(),
        -badge_measured.height + placement.vertical_offset(),
    )
}

#[cfg(test)]
#[path = "../../../tests/widget/component/badge.rs"]
mod tests;
