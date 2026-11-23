use iced_widget::core::{Element, Length, Size, Widget, layout};
use iced_widget::{Space, Text};

use super::*;

type Renderer = iced_tiny_skia::Renderer;
type TestElement<'a> = Element<'a, (), crate::Theme, Renderer>;

#[test]
fn columns_match_the_iced_fluid_grid_count() {
    // 3 cells of 160 with 16 between them fit in 512 exactly.
    assert_eq!(columns_for(512.0, 160.0, SPACING), 3);
    // Any more width needs another, narrower column.
    assert_eq!(columns_for(513.0, 160.0, SPACING), 4);
    assert_eq!(columns_for(100.0, 160.0, SPACING), 1);
    assert_eq!(columns_for(0.0, 160.0, SPACING), 1);
    assert_eq!(columns_for(512.0, 0.0, 0.0), 1);
}

#[test]
fn cell_width_shares_the_width_left_after_spacing() {
    assert_eq!(cell_width(512.0, 3, SPACING), 160.0);
    assert_eq!(cell_width(100.0, 1, SPACING), 100.0);
    assert_eq!(cell_width(10.0, 4, SPACING), 0.0);
    assert_eq!(cell_width(100.0, 0, SPACING), 100.0);
}

#[test]
fn cells_never_get_wider_than_the_maximum() {
    for width in [200.0, 333.0, 640.0, 1023.0, 1920.0] {
        let columns = columns_for(width, 180.0, SPACING);

        assert!(cell_width(width, columns, SPACING) <= 180.0, "{width}");
    }
}

#[test]
fn height_follows_the_aspect_ratio() {
    assert_eq!(height_for(120.0, 2.0 / 3.0), 180.0);
    assert_eq!(height_for(120.0, 0.0), 120.0);
    assert_eq!(height_for(120.0, f32::NAN), 120.0);
}

#[test]
fn aspect_ratio_lays_content_out_at_the_full_width() {
    let mut cover: AspectRatio<'_, (), crate::Theme, Renderer> = aspect_ratio(
        Space::new().width(Length::Fill).height(Length::Fill),
        2.0 / 3.0,
    );
    let mut tree = iced_widget::core::widget::Tree::new(&cover as &dyn Widget<_, _, _>);
    let renderer = Renderer::new(crate::fonts::ROBOTO, iced_widget::core::Pixels(16.0));

    let node = Widget::layout(
        &mut cover,
        &mut tree,
        &renderer,
        &layout::Limits::new(Size::ZERO, Size::new(120.0, f32::INFINITY)),
    );

    assert_eq!(node.size(), Size::new(120.0, 180.0));
    assert_eq!(node.children()[0].size(), Size::new(120.0, 180.0));
}

#[test]
fn grid_is_fluid_with_material_spacing() {
    let cells: Vec<TestElement<'_>> = (0..5).map(|i| Text::new(i.to_string()).into()).collect();
    let grid = grid(160.0, cells);

    assert_eq!(
        Widget::<(), crate::Theme, Renderer>::size(&grid),
        Size::new(Length::Fill, Length::Shrink)
    );
}
