use iced_material as material;
use material::widget::page;

use super::super::{Message, NAV_DESTINATIONS, Showcase, TabChoice};

pub(super) fn view(state: &Showcase) -> material::Element<'_, Message> {
    let selection = state.navigation_selection();
    let bar = material::widget::navigation::navigation_bar(
        &NAV_DESTINATIONS,
        selection,
        Message::Navigate,
    );
    let rail = material::widget::navigation::navigation_rail_with_menu_fitting_content(
        &NAV_DESTINATIONS,
        selection,
        Message::Navigate,
        Message::MenuPressed,
    );
    let expanded_rail =
        material::widget::navigation::navigation_rail_expanded_with_menu_fitting_content(
            "Showcase",
            &NAV_DESTINATIONS,
            selection,
            Message::Navigate,
            Message::MenuPressed,
        );

    page::sections([
        page::section("Tabs", tabs(state)).into(),
        page::section("Navigation bar", bar).into(),
        page::section("Navigation rail with menu", rail).into(),
        page::section("Expanded navigation rail", expanded_rail).into(),
    ])
    .into()
}

fn tabs(state: &Showcase) -> material::Element<'_, Message> {
    page::component_stack([
        material::widget::tabs::animated_primary_icon_label_bar(
            &state.primary_tab_state,
            [
                (
                    "input",
                    "Inputs",
                    Message::PrimaryTabSelected(TabChoice::Inputs),
                ),
                (
                    "tune",
                    "Controls",
                    Message::PrimaryTabSelected(TabChoice::Controls),
                ),
                (
                    "info",
                    "Feedback",
                    Message::PrimaryTabSelected(TabChoice::Feedback),
                ),
            ],
        )
        .into(),
        material::widget::tabs::animated_secondary_label_bar(
            &state.secondary_tab_state,
            [
                ("Overview", Message::SecondaryTabSelected(TabChoice::Inputs)),
                (
                    "Details",
                    Message::SecondaryTabSelected(TabChoice::Controls),
                ),
                (
                    "History",
                    Message::SecondaryTabSelected(TabChoice::Feedback),
                ),
            ],
        )
        .into(),
    ])
    .into()
}
