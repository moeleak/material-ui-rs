//! Material page layout helpers.

use iced_widget::core::text as core_text;
use iced_widget::core::{Element, Length, alignment};
use iced_widget::text;
use iced_widget::{Column, Container, Row, Scrollable, Text};

use crate::{Theme, text as material_text, tokens};

/// Maximum content width used by Material page surfaces.
pub const MAX_WIDTH: f32 = 980.0;

/// Outer page padding used by Material page surfaces.
pub const PADDING: f32 = 28.0;

/// Vertical spacing between a page header and body.
pub const SPACING: f32 = 28.0;

/// Vertical spacing between page sections.
pub const SECTION_SPACING: f32 = 24.0;

/// Default spacing for grouped page content.
pub const STACK_SPACING: f32 = 16.0;

/// Compact spacing for dense vertical page content.
pub const COMPACT_STACK_SPACING: f32 = 8.0;

/// Spacing for repeated component previews.
pub const COMPONENT_STACK_SPACING: f32 = 12.0;

/// Dense spacing for related controls that need a little separation.
pub const DENSE_STACK_SPACING: f32 = 10.0;

/// Spacious spacing for separated control groups.
pub const SPACIOUS_STACK_SPACING: f32 = 18.0;

/// Default spacing for horizontal page actions.
pub const ROW_SPACING: f32 = 12.0;

/// Compact spacing for dense horizontal page actions.
pub const COMPACT_ROW_SPACING: f32 = 8.0;

/// Spacing for paired indicator previews.
pub const INDICATOR_ROW_SPACING: f32 = 16.0;

/// Height used by compact divider demonstration rows.
pub const DIVIDER_ROW_HEIGHT: f32 = 32.0;

/// Spacing used by compact divider demonstration rows.
pub const DIVIDER_ROW_SPACING: f32 = 16.0;

/// Padding used by compact showcase cards.
pub const CARD_PADDING: f32 = 12.0;

/// Height used by compact showcase cards.
pub const CARD_HEIGHT: f32 = 78.0;

/// Maximum width used by component previews.
pub const PREVIEW_MAX_WIDTH: f32 = tokens::component::bottom_sheet::SHEET_MAX_WIDTH;

/// Horizontal space reserved around component previews before clamping.
pub const PREVIEW_HORIZONTAL_RESERVE: f32 = 160.0;

/// Minimum width used by component previews.
pub const PREVIEW_MIN_WIDTH: f32 = 320.0;

/// Height used by fixed component preview panes.
pub const PREVIEW_HEIGHT: f32 = 260.0;

/// Creates a scrollable Material page surface.
pub fn surface<'a, Message, Renderer>(
    header: impl Into<Element<'a, Message, Theme, Renderer>>,
    body: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Scrollable<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
{
    let page = Column::new()
        .push(header)
        .push(body)
        .spacing(SPACING)
        .padding(PADDING)
        .width(Length::Fill)
        .max_width(MAX_WIDTH);

    Scrollable::new(
        super::container::surface_container_high(page)
            .width(Length::Fill)
            .center_x(Length::Fill),
    )
    .height(Length::Fill)
}

/// Computes a responsive width for component previews.
pub fn preview_width(viewport_width: f32) -> f32 {
    (viewport_width - PREVIEW_HORIZONTAL_RESERVE).clamp(PREVIEW_MIN_WIDTH, PREVIEW_MAX_WIDTH)
}

/// Centers preview content at a fixed Material preview width.
pub fn centered_preview<'a, Message, Renderer>(
    width: f32,
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Container<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + 'a,
{
    Container::new(Container::new(content).width(Length::Fixed(width)))
        .width(Length::Fill)
        .align_x(alignment::Horizontal::Center)
}

/// Creates a fixed-height component preview pane.
pub fn preview_pane<'a, Message, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Container<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + 'a,
{
    Container::new(content)
        .width(Length::Fill)
        .height(Length::Fixed(PREVIEW_HEIGHT))
}

/// Creates a fixed-height component preview pane with horizontal alignment.
pub fn aligned_preview_pane<'a, Message, Renderer>(
    alignment: alignment::Horizontal,
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Container<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + 'a,
{
    preview_pane(content).align_x(alignment)
}

/// Creates a simple Material page header.
pub fn header<'a, Message, Renderer>(
    title: impl text::IntoFragment<'a>,
    subtitle: impl text::IntoFragment<'a>,
) -> Column<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: core_text::Renderer + 'a,
{
    Column::new()
        .push(type_scale_text(title, tokens::typography::HEADLINE_LARGE))
        .push(type_scale_text(subtitle, tokens::typography::BODY_LARGE))
        .spacing(6)
}

/// Creates a titled Material page section.
pub fn section<'a, Message, Renderer>(
    title: impl text::IntoFragment<'a>,
    body: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Column<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: core_text::Renderer + 'a,
{
    Column::new()
        .push(type_scale_text(title, tokens::typography::TITLE_MEDIUM))
        .push(body)
        .spacing(12)
        .width(Length::Fill)
}

/// Creates a full-width page section list with dividers between sections.
pub fn sections<'a, Message, Renderer>(
    sections: impl IntoIterator<Item = Element<'a, Message, Theme, Renderer>>,
) -> Column<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
{
    let mut content = Column::new().spacing(SECTION_SPACING).width(Length::Fill);
    let mut first = true;

    for section in sections {
        if first {
            first = false;
        } else {
            content = content.push(super::rule::horizontal_inset());
        }

        content = content.push(section);
    }

    content
}

/// Creates a full-width vertical stack for page content.
pub fn stack<'a, Message, Renderer>(
    children: impl IntoIterator<Item = Element<'a, Message, Theme, Renderer>>,
) -> Column<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + 'a,
{
    Column::with_children(children.into_iter())
        .spacing(STACK_SPACING)
        .width(Length::Fill)
}

/// Creates a compact full-width vertical stack.
pub fn compact_stack<'a, Message, Renderer>(
    children: impl IntoIterator<Item = Element<'a, Message, Theme, Renderer>>,
) -> Column<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + 'a,
{
    stack(children).spacing(COMPACT_STACK_SPACING)
}

/// Creates a full-width stack for repeated component previews.
pub fn component_stack<'a, Message, Renderer>(
    children: impl IntoIterator<Item = Element<'a, Message, Theme, Renderer>>,
) -> Column<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + 'a,
{
    stack(children).spacing(COMPONENT_STACK_SPACING)
}

/// Creates a dense full-width vertical stack.
pub fn dense_stack<'a, Message, Renderer>(
    children: impl IntoIterator<Item = Element<'a, Message, Theme, Renderer>>,
) -> Column<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + 'a,
{
    stack(children).spacing(DENSE_STACK_SPACING)
}

/// Creates a spacious full-width vertical stack.
pub fn spacious_stack<'a, Message, Renderer>(
    children: impl IntoIterator<Item = Element<'a, Message, Theme, Renderer>>,
) -> Column<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + 'a,
{
    stack(children).spacing(SPACIOUS_STACK_SPACING)
}

/// Creates a centered row for page controls.
pub fn row<'a, Message, Renderer>(
    children: impl IntoIterator<Item = Element<'a, Message, Theme, Renderer>>,
) -> Row<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + 'a,
{
    Row::with_children(children.into_iter())
        .spacing(ROW_SPACING)
        .align_y(alignment::Vertical::Center)
}

/// Creates a centered row for paired indicator previews.
pub fn indicator_row<'a, Message, Renderer>(
    children: impl IntoIterator<Item = Element<'a, Message, Theme, Renderer>>,
) -> Row<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + 'a,
{
    row(children).spacing(INDICATOR_ROW_SPACING)
}

/// Creates a compact centered row for dense controls like chips and badges.
pub fn compact_row<'a, Message, Renderer>(
    children: impl IntoIterator<Item = Element<'a, Message, Theme, Renderer>>,
) -> Row<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + 'a,
{
    Row::with_children(children.into_iter())
        .spacing(COMPACT_ROW_SPACING)
        .align_y(alignment::Vertical::Center)
}

/// Creates a full-width row with a leading label and trailing value.
pub fn labeled_value_row<'a, Message, Renderer>(
    label: impl text::IntoFragment<'a>,
    value: impl text::IntoFragment<'a>,
) -> Row<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
{
    row([
        type_scale_text(label, tokens::typography::BODY_LARGE)
            .width(Length::Fill)
            .into(),
        type_scale_text(value, tokens::typography::BODY_LARGE).into(),
    ])
}

/// Creates a compact row for showing vertical dividers.
pub fn divider_row<'a, Message, Renderer>(
    children: impl IntoIterator<Item = Element<'a, Message, Theme, Renderer>>,
) -> Row<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + 'a,
{
    row(children)
        .height(Length::Fixed(DIVIDER_ROW_HEIGHT))
        .spacing(DIVIDER_ROW_SPACING)
}

/// Creates a compact titled card using one of the Material card constructors.
pub fn card<'a, Message, Renderer>(
    style: fn(Element<'a, Message, Theme, Renderer>) -> Container<'a, Message, Theme, Renderer>,
    title: impl text::IntoFragment<'a>,
    subtitle: impl text::IntoFragment<'a>,
) -> Container<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
{
    style(
        stack([
            type_scale_text(title, tokens::typography::TITLE_MEDIUM).into(),
            type_scale_text(subtitle, tokens::typography::BODY_MEDIUM).into(),
        ])
        .spacing(2)
        .into(),
    )
    .padding(CARD_PADDING)
    .height(Length::Fixed(CARD_HEIGHT))
    .width(Length::Fill)
}

/// Creates text using a Material type scale.
pub fn type_scale_text<'a, Renderer>(
    content: impl text::IntoFragment<'a>,
    scale: tokens::typography::TypeScale,
) -> Text<'a, Theme, Renderer>
where
    Renderer: core_text::Renderer + 'a,
{
    material_text::type_scale(content, scale)
}

#[cfg(test)]
mod tests {
    use iced_widget::core::Element;

    use super::*;

    #[derive(Debug, Clone)]
    enum Message {}

    type TestElement<'a> = Element<'a, Message, Theme, iced_widget::Renderer>;

    #[test]
    fn page_helpers_compile_to_elements() {
        let header = header("Title", "Subtitle");
        let body = section("Section", Text::new("Body"));
        let _: TestElement<'_> = surface(header, body).into();
        let _: TestElement<'_> = sections([
            section("First", Text::new("Body")).into(),
            section("Second", Text::new("Body")).into(),
        ])
        .into();
        let _: TestElement<'_> = stack([Text::new("One").into(), Text::new("Two").into()]).into();
        let _: TestElement<'_> =
            compact_stack([Text::new("One").into(), Text::new("Two").into()]).into();
        let _: TestElement<'_> =
            component_stack([Text::new("One").into(), Text::new("Two").into()]).into();
        let _: TestElement<'_> =
            dense_stack([Text::new("One").into(), Text::new("Two").into()]).into();
        let _: TestElement<'_> =
            spacious_stack([Text::new("One").into(), Text::new("Two").into()]).into();
        let _: TestElement<'_> = row([Text::new("One").into(), Text::new("Two").into()]).into();
        let _: TestElement<'_> =
            indicator_row([Text::new("One").into(), Text::new("Two").into()]).into();
        let _: TestElement<'_> =
            compact_row([Text::new("One").into(), Text::new("Two").into()]).into();
        let _: TestElement<'_> = labeled_value_row("Label", "Value").into();
        let _: TestElement<'_> =
            divider_row([Text::new("One").into(), Text::new("Two").into()]).into();
        let _: TestElement<'_> = card(super::super::card::elevated, "Card", "Subtitle").into();
        let _: TestElement<'_> = centered_preview(320.0, Text::new("Preview")).into();
        let _: TestElement<'_> = preview_pane(Text::new("Preview")).into();
        let _: TestElement<'_> =
            aligned_preview_pane(alignment::Horizontal::Right, Text::new("Preview")).into();
    }

    #[test]
    fn preview_width_caps_to_material_preview_bounds() {
        assert_eq!(preview_width(1920.0), PREVIEW_MAX_WIDTH);
        assert_eq!(preview_width(420.0), PREVIEW_MIN_WIDTH);
        assert_eq!(PREVIEW_HEIGHT, 260.0);
    }
}
