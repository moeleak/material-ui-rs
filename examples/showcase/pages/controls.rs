use iced_material as material;
use material::widget::page;

use super::super::{Message, RadioChoice, SegmentChoice, Showcase};

pub(super) fn view(state: &Showcase) -> material::Element<'_, Message> {
    page::sections([
        page::section("Counter", counter_controls(state)).into(),
        page::section("Actions", action_buttons(state)).into(),
        page::section("FABs", fabs(state)).into(),
        page::section("Chips", chips(state)).into(),
        page::section("Segmented buttons", segmented_buttons(state)).into(),
        page::section("Selection controls", selection_controls(state)).into(),
    ])
    .into()
}

fn counter_controls(state: &Showcase) -> material::Element<'_, Message> {
    page::row([
        material::widget::button::outlined_action("Minus", Message::Decrement),
        material::text::headline_medium(state.count.to_string()).into(),
        material::widget::button::filled_action("Plus", Message::Increment),
    ])
    .into()
}

fn action_buttons(state: &Showcase) -> material::Element<'_, Message> {
    use material::widget::button;

    page::row(button::enabled_actions(
        state.enabled,
        Message::Increment,
        [
            button::filled("Filled"),
            button::filled_tonal("Tonal"),
            button::text("Text"),
        ],
    ))
    .into()
}

fn fabs(state: &Showcase) -> material::Element<'_, Message> {
    use material::widget::button;

    page::stack([
        page::row(button::enabled_actions(
            state.enabled,
            Message::Increment,
            [
                button::surface_small_fab("add"),
                button::surface_fab("add"),
                button::surface_large_fab("add"),
                button::primary_fab("add"),
                button::secondary_fab("add"),
                button::tertiary_fab("add"),
            ],
        ))
        .into(),
        page::row(button::enabled_actions(
            state.enabled,
            Message::Increment,
            [
                button::primary_extended_fab_with_icon("add", "Create"),
                button::secondary_extended_fab_with_icon("share", "Share"),
                button::tertiary_extended_fab_with_icon("add", "Add"),
                button::surface_extended_fab("Reroute"),
            ],
        ))
        .into(),
    ])
    .into()
}

fn chips(state: &Showcase) -> material::Element<'_, Message> {
    use material::widget::button;

    page::compact_row(button::enabled_actions(
        state.enabled,
        Message::Increment,
        [
            button::assist_chip("Assist"),
            button::suggestion_chip("Suggestion"),
            button::filter_chip("Filter"),
            button::selected_filter_chip("Selected"),
        ],
    ))
    .into()
}

fn segmented_buttons(state: &Showcase) -> material::Element<'_, Message> {
    use material::widget::segmented_button;

    segmented_button::group(segmented_button::animated_selectable_label_actions(
        &state.segment_state,
        [
            ("List", Message::SegmentSelected(SegmentChoice::List)),
            ("Grid", Message::SegmentSelected(SegmentChoice::Grid)),
            ("Map", Message::SegmentSelected(SegmentChoice::Map)),
        ],
    ))
    .into()
}

fn selection_controls(state: &Showcase) -> material::Element<'_, Message> {
    let switches = page::component_stack([
        material::widget::checkbox::standard(
            state.enabled,
            "Enable actions",
            Message::EnabledChanged,
        )
        .into(),
        material::widget::toggler::standard(
            state.dark_mode,
            "Dark theme",
            Message::DarkModeChanged,
        )
        .into(),
    ]);

    let radios = page::row([
        material::widget::radio::standard(
            "Standard",
            RadioChoice::Standard,
            state.radio_choice,
            Message::ChoiceSelected,
        )
        .into(),
        material::widget::radio::standard(
            "Expressive",
            RadioChoice::Expressive,
            state.radio_choice,
            Message::ChoiceSelected,
        )
        .into(),
        material::widget::radio::standard(
            "Dense",
            RadioChoice::Dense,
            state.radio_choice,
            Message::ChoiceSelected,
        )
        .into(),
    ]);

    page::spacious_stack([switches.into(), radios.into()]).into()
}
