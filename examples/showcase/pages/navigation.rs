use iced::Length;
use iced_material as material;
use material::widget::page;

use super::super::{Message, NAV_DESTINATIONS, Showcase};

pub(super) fn view(state: &Showcase) -> material::Element<'_, Message> {
    let selection = state.navigation_selection();
    let rail_height = showcase_rail_height();
    let bar = material::widget::navigation::navigation_bar(
        &NAV_DESTINATIONS,
        selection,
        Message::Navigate,
    );
    let rail = material::widget::navigation::navigation_rail_with_menu(
        &NAV_DESTINATIONS,
        selection,
        Message::Navigate,
        Message::MenuPressed,
    )
    .height(Length::Fixed(rail_height));
    let expanded_rail = material::widget::navigation::navigation_rail_expanded_with_menu(
        "Showcase",
        &NAV_DESTINATIONS,
        selection,
        Message::Navigate,
        Message::MenuPressed,
    )
    .height(Length::Fixed(rail_height));

    page::sections([
        page::section("Navigation bar", bar).into(),
        page::section("Navigation rail with menu", rail).into(),
        page::section("Expanded navigation rail", expanded_rail).into(),
    ])
    .into()
}

pub(super) fn showcase_rail_height() -> f32 {
    material::widget::navigation::navigation_rail_min_height(NAV_DESTINATIONS.len(), true)
}
