use iced_material as material;
use material::widget::page;

use super::super::{Message, Showcase};

pub(super) fn view(state: &Showcase) -> material::Element<'_, Message> {
    let input = material::widget::text_input::outlined("Write a note", &state.note)
        .on_input(Message::TextChanged);

    let editor = material::widget::text_editor::outlined_area(&state.editor_content)
        .placeholder("Write details")
        .on_action(Message::EditorAction);

    let select_options = ["Assist", "Suggestion", "Filter"];
    let select = material::widget::pick_list::outlined(
        select_options,
        state.select_choice,
        Message::SelectChanged,
    )
    .placeholder("Choose a chip");

    let combo_box = material::widget::combo_box::outlined_with_input(
        &state.combo_options,
        "Search a chip",
        &state.combo_input,
        state.combo_choice.as_ref(),
        Message::ComboSelected,
    )
    .on_input(Message::ComboInputChanged);

    page::sections([
        page::section("Text fields", page::stack([input.into(), editor.into()])).into(),
        page::section(
            "Selection fields",
            page::stack([select.into(), combo_box.into()]),
        )
        .into(),
        page::section("Search", search_bar(state)).into(),
        page::section("Dividers", dividers()).into(),
    ])
    .into()
}

fn search_bar(state: &Showcase) -> material::Element<'_, Message> {
    material::widget::search::bar_with_trailing(
        "Search Material components",
        &state.search_query,
        Message::SearchChanged,
        Some(material::widget::app_bar::icon_action(
            "tune",
            Message::Increment,
        )),
    )
    .into()
}

fn dividers() -> material::Element<'static, Message> {
    page::compact_stack([
        material::widget::rule::horizontal_full_width().into(),
        page::divider_row([
            material::text::body_large("Full").into(),
            material::widget::rule::vertical_full_height().into(),
            material::text::body_large("Inset").into(),
        ])
        .into(),
        material::widget::rule::horizontal_inset().into(),
    ])
    .into()
}
