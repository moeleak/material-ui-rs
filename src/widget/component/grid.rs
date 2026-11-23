//! Responsive grids of equally wide cells, such as a library of cover art.
//!
//! A [`grid`] lays its cells out in as many columns as fit cells at most
//! `max_cell_width` wide, as [`iced_widget::Grid::fluid`] does, with Material
//! spacing; rows are as tall as their tallest cell. [`aspect_ratio`] gives part
//! of a cell, such as a cover, a fixed aspect ratio, so that a title can follow
//! it in the same cell.

use iced_widget::core::widget::{Operation, Tree};
use iced_widget::core::{
    Clipboard, Element, Event, Layout, Length, Rectangle, Shell, Size, Vector, Widget, layout,
    mouse, overlay, renderer,
};
use iced_widget::{Container, Grid, Row};

pub use iced_widget::grid::Sizing;

/// The space between the cells of a [`grid`], the Material 3 gutter.
pub const SPACING: f32 = 16.0;

/// A grid of `cells` in as many columns as fit cells at most `max_cell_width`
/// wide, [`SPACING`] apart; each row is as tall as its tallest cell.
pub fn grid<'a, Message, Theme, Renderer>(
    max_cell_width: f32,
    cells: impl IntoIterator<Item = Element<'a, Message, Theme, Renderer>>,
) -> Grid<'a, Message, Theme, Renderer>
where
    Renderer: iced_widget::core::Renderer,
{
    Grid::with_children(cells)
        .fluid(max_cell_width)
        .spacing(SPACING)
        .height(Sizing::EvenlyDistribute(Length::Shrink))
}

/// A grid of `cells` exactly `cell_width` wide, [`SPACING`] apart, in as many
/// columns as fit: unlike a [`grid`], whose cells grow and shrink with the
/// width, its cells keep their size as the window resizes, and only the
/// number of columns changes, which steadies a wall of covers.
pub fn fixed<'a, Message, Theme, Renderer>(
    cell_width: f32,
    cells: impl IntoIterator<Item = Element<'a, Message, Theme, Renderer>>,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: iced_widget::container::Catalog + 'a,
    Renderer: iced_widget::core::Renderer + 'a,
{
    Row::with_children(
        cells
            .into_iter()
            .map(|cell| Container::new(cell).width(Length::Fixed(cell_width)).into()),
    )
    .spacing(SPACING)
    .width(Length::Fill)
    .wrap()
    .into()
}

/// How many columns a [`fixed`] grid `width` wide has for cells
/// `cell_width` wide, `spacing` apart: as many as fit, and at least one.
pub fn fixed_columns(width: f32, cell_width: f32, spacing: f32) -> usize {
    if cell_width + spacing <= 0.0 {
        return 1;
    }

    (((width + spacing) / (cell_width + spacing)).floor() as usize).max(1)
}

/// How many columns a [`grid`] `width` wide has for cells at most
/// `max_cell_width` wide, `spacing` apart, as its layout counts them: for
/// moving a selection by rows, or for loading the cells of the rows in view.
pub fn columns_for(width: f32, max_cell_width: f32, spacing: f32) -> usize {
    if max_cell_width + spacing <= 0.0 {
        return 1;
    }

    (((width + spacing) / (max_cell_width + spacing)).ceil() as usize).max(1)
}

/// The width of the cells of a [`grid`] `width` wide with `columns` columns,
/// `spacing` apart.
pub fn cell_width(width: f32, columns: usize, spacing: f32) -> f32 {
    let columns = columns.max(1);

    ((width - spacing * (columns - 1) as f32) / columns as f32).max(0.0)
}

/// `content` as wide as it may be, and `ratio` (width / height) times less
/// tall: a cover of 2:3 art is `aspect_ratio(cover, 2.0 / 3.0)`.
pub fn aspect_ratio<'a, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
    ratio: f32,
) -> AspectRatio<'a, Message, Theme, Renderer> {
    AspectRatio {
        content: content.into(),
        ratio,
    }
}

/// The height of content `width` wide at `ratio` (width / height).
pub fn height_for(width: f32, ratio: f32) -> f32 {
    if ratio > 0.0 && ratio.is_finite() {
        width / ratio
    } else {
        width
    }
}

/// Content with a fixed aspect ratio; see [`aspect_ratio`].
pub struct AspectRatio<'a, Message, Theme, Renderer> {
    content: Element<'a, Message, Theme, Renderer>,
    ratio: f32,
}

impl<Message, Theme, Renderer> std::fmt::Debug for AspectRatio<'_, Message, Theme, Renderer> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AspectRatio")
            .field("ratio", &self.ratio)
            .finish_non_exhaustive()
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for AspectRatio<'_, Message, Theme, Renderer>
where
    Renderer: iced_widget::core::Renderer,
{
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: Length::Fill,
            height: Length::Shrink,
        }
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let width = limits.max().width;
        let width = if width.is_finite() {
            width
        } else {
            limits.min().width
        };
        let size = Size::new(width, height_for(width, self.ratio));
        let content = self.content.as_widget_mut().layout(
            &mut tree.children[0],
            renderer,
            &layout::Limits::new(size, size),
        );

        layout::Node::with_children(size, vec![content])
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        self.content.as_widget_mut().operate(
            &mut tree.children[0],
            content_layout(layout),
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
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            content_layout(layout),
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
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            content_layout(layout),
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
            content_layout(layout),
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
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            content_layout(layout),
            renderer,
            viewport,
            translation,
        )
    }
}

fn content_layout(layout: Layout<'_>) -> Layout<'_> {
    layout
        .children()
        .next()
        .expect("an aspect ratio lays out its content")
}

impl<'a, Message, Theme, Renderer> From<AspectRatio<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: iced_widget::core::Renderer + 'a,
{
    fn from(aspect_ratio: AspectRatio<'a, Message, Theme, Renderer>) -> Self {
        Element::new(aspect_ratio)
    }
}

#[cfg(test)]
#[path = "../../../tests/widget/component/grid.rs"]
mod tests;
