use iced_material as material;
use material::widget::page;

use super::super::{INVENTORY_ROWS, InventoryRow, Message};

pub(super) fn view() -> material::Element<'static, Message> {
    page::sections([
        page::section("Cards", cards()).into(),
        page::section("Lists", lists()).into(),
        page::section("Data table", data_table()).into(),
    ])
    .into()
}

fn cards() -> material::Element<'static, Message> {
    page::compact_stack([
        page::card(material::widget::card::elevated, "Elevated", "Level 1").into(),
        page::card(material::widget::card::filled, "Filled", "Container").into(),
        page::card(material::widget::card::outlined, "Outlined", "1px stroke").into(),
    ])
    .into()
}

fn lists() -> material::Element<'static, Message> {
    material::widget::list::group([
        material::widget::list::one_line_with_leading_icon("info", "One-line list item").into(),
        material::widget::list::two_line_with_trailing("Messages", "Supporting text", "24").into(),
        material::widget::list::three_line("Three-line item", "Supporting text", "Second line")
            .into(),
    ])
    .into()
}

fn data_table() -> material::Element<'static, Message> {
    material::widget::data_table::standard(
        [
            material::widget::data_table::weighted_column(2, "Component", |row: InventoryRow| {
                row.component
            }),
            material::widget::data_table::column("State", |row: InventoryRow| row.status),
            material::widget::data_table::compact_numeric_column("Count", |row: InventoryRow| {
                row.count.to_string()
            }),
        ],
        INVENTORY_ROWS,
    )
    .into()
}
