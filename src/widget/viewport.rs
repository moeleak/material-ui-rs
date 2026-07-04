//! Reusable viewport helpers for clipped animated content.

use std::fmt;

use iced_widget::core::widget::{self, Tree, tree};
use iced_widget::core::{
    Clipboard, Element, Event, Layout, Length, Point, Rectangle, Shell, Size, Vector, Widget,
    layout, mouse, overlay, renderer,
};

use crate::Theme;

/// A viewport that clips its child to a visible size while optionally laying it
/// out at a larger internal size.
///
/// This is useful for Material size transitions where content should keep its
/// final layout while the visible viewport expands or collapses.
pub struct Viewport<'a, Message, Renderer = iced_widget::Renderer> {
    content: Element<'a, Message, Theme, Renderer>,
    width: Length,
    height: Length,
    layout_width: Option<f32>,
    layout_height: Option<f32>,
}

impl<'a, Message, Renderer> Viewport<'a, Message, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + 'a,
{
    pub fn new(content: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        let content = content.into();
        let size = content.as_widget().size();

        Self {
            content,
            width: size.width,
            height: size.height,
            layout_width: None,
            layout_height: None,
        }
    }

    pub fn fixed_height(
        content: impl Into<Element<'a, Message, Theme, Renderer>>,
        visible_height: f32,
        layout_height: f32,
    ) -> Self {
        Self::new(content)
            .height(Length::Fixed(visible_height))
            .layout_height(layout_height)
    }

    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    pub fn layout_width(mut self, layout_width: f32) -> Self {
        self.layout_width = Some(layout_width);
        self
    }

    pub fn layout_height(mut self, layout_height: f32) -> Self {
        self.layout_height = Some(layout_height);
        self
    }
}

/// Creates a vertically clipped viewport with a fixed visible height and a
/// fixed internal layout height.
pub fn fixed_height<'a, Message, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
    visible_height: f32,
    layout_height: f32,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + 'a,
{
    Viewport::fixed_height(content, visible_height, layout_height).into()
}

impl<Message, Renderer> fmt::Debug for Viewport<'_, Message, Renderer> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Viewport")
            .field("width", &self.width)
            .field("height", &self.height)
            .field("layout_width", &self.layout_width)
            .field("layout_height", &self.layout_height)
            .finish_non_exhaustive()
    }
}

impl<Message, Renderer> Widget<Message, Theme, Renderer> for Viewport<'_, Message, Renderer>
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
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn size_hint(&self) -> Size<Length> {
        self.size()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let size = limits.resolve(self.width, self.height, Size::ZERO);
        let content_size = Size::new(
            self.layout_width.unwrap_or(size.width).max(size.width),
            self.layout_height.unwrap_or(size.height).max(size.height),
        );
        let content_limits = layout::Limits::new(content_size, content_size);
        let content = self
            .content
            .as_widget_mut()
            .layout(tree, renderer, &content_limits)
            .move_to(Point::ORIGIN);

        layout::Node::with_children(size, vec![content])
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        if let Some(content_layout) = layout.children().next() {
            self.content
                .as_widget_mut()
                .operate(tree, content_layout, renderer, operation);
        }
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
        let Some(visible_viewport) = layout.bounds().intersection(viewport) else {
            return;
        };
        let Some(content_layout) = layout.children().next() else {
            return;
        };
        let cursor = if cursor.is_over(visible_viewport) {
            cursor
        } else {
            mouse::Cursor::Unavailable
        };

        self.content.as_widget_mut().update(
            tree,
            event,
            content_layout,
            cursor,
            renderer,
            clipboard,
            shell,
            &visible_viewport,
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
        let Some(visible_viewport) = layout.bounds().intersection(viewport) else {
            return mouse::Interaction::None;
        };
        let Some(content_layout) = layout.children().next() else {
            return mouse::Interaction::None;
        };

        if !cursor.is_over(visible_viewport) {
            return mouse::Interaction::None;
        }

        self.content.as_widget().mouse_interaction(
            tree,
            content_layout,
            cursor,
            &visible_viewport,
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
        let Some(visible_viewport) = layout.bounds().intersection(viewport) else {
            return;
        };
        let Some(content_layout) = layout.children().next() else {
            return;
        };

        renderer.with_layer(visible_viewport, |renderer| {
            self.content.as_widget().draw(
                tree,
                renderer,
                theme,
                style,
                content_layout,
                cursor,
                &visible_viewport,
            );
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
        let visible_viewport = layout.bounds().intersection(viewport)?;
        let content_layout = layout.children().next()?;

        self.content.as_widget_mut().overlay(
            tree,
            content_layout,
            renderer,
            &visible_viewport,
            translation,
        )
    }
}

impl<'a, Message, Renderer> From<Viewport<'a, Message, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + 'a,
{
    fn from(viewport: Viewport<'a, Message, Renderer>) -> Self {
        Element::new(viewport)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_height_viewport_separates_visible_and_layout_height() {
        let viewport: Viewport<'_, (), iced_widget::Renderer> =
            Viewport::fixed_height(iced_widget::Space::new(), 40.0, 120.0);

        assert_eq!(viewport.height, Length::Fixed(40.0));
        assert_eq!(viewport.layout_height, Some(120.0));
    }
}
