use iced::{Length, Padding, alignment};
use iced_material as material;
use material::widget::page;

use super::super::{Message, Showcase};

const TOP_APP_BAR_STATUS_INSET: f32 = 24.0;
const STRUCTURE_PREVIEW_MAX_WIDTH: f32 = 640.0;
const STRUCTURE_PREVIEW_HORIZONTAL_RESERVE: f32 = 160.0;
const STRUCTURE_PREVIEW_MIN_WIDTH: f32 = 320.0;

pub(super) fn view(state: &Showcase) -> material::Element<'_, Message> {
    page::sections([
        page::section("Top app bars", top_app_bars()).into(),
        page::section("Search view", search_view(state)).into(),
        page::section("Bottom app bar", bottom_app_bar()).into(),
        page::section("Bottom sheets", bottom_sheets(state)).into(),
    ])
    .into()
}

fn top_app_bars() -> material::Element<'static, Message> {
    page::stack([
        top_app_bar_preview(
            material::widget::app_bar::small(
                "Small",
                Some(
                    material::widget::app_bar::icon_button("menu")
                        .on_press(Message::MenuPressed)
                        .into(),
                ),
                [
                    material::widget::app_bar::icon_button("search")
                        .on_press(Message::Increment)
                        .into(),
                    material::widget::app_bar::icon_button("info")
                        .on_press(Message::Increment)
                        .into(),
                ],
            )
            .into(),
        )
        .into(),
        top_app_bar_preview(
            material::widget::app_bar::medium(
                "Medium",
                Some(
                    material::widget::app_bar::icon_button("menu")
                        .on_press(Message::MenuPressed)
                        .into(),
                ),
                [
                    material::widget::app_bar::icon_button("search")
                        .on_press(Message::Increment)
                        .into(),
                    material::widget::app_bar::icon_button("info")
                        .on_press(Message::Increment)
                        .into(),
                ],
            )
            .into(),
        )
        .into(),
        top_app_bar_preview(
            material::widget::app_bar::large(
                "Large",
                Some(
                    material::widget::app_bar::icon_button("menu")
                        .on_press(Message::MenuPressed)
                        .into(),
                ),
                [
                    material::widget::app_bar::icon_button("search")
                        .on_press(Message::Increment)
                        .into(),
                    material::widget::app_bar::icon_button("info")
                        .on_press(Message::Increment)
                        .into(),
                ],
            )
            .into(),
        )
        .into(),
    ])
    .spacing(12)
    .into()
}

fn top_app_bar_preview(
    app_bar: material::Element<'static, Message>,
) -> material::Element<'static, Message> {
    page::stack([
        material::Container::new(iced::widget::Space::new())
            .height(Length::Fixed(TOP_APP_BAR_STATUS_INSET))
            .width(Length::Fill)
            .style(material::container::surface)
            .into(),
        app_bar,
    ])
    .spacing(0)
    .into()
}

fn search_view(state: &Showcase) -> material::Element<'_, Message> {
    let results = page::stack([
        material::widget::list::one_line_with_leading_icon("input", "Inputs").into(),
        material::widget::list::one_line_with_leading_icon("tune", "Controls").into(),
        material::widget::list::one_line_with_leading_icon("info", "Feedback").into(),
    ])
    .spacing(0);

    material::widget::search::docked_view(
        "Search components",
        &state.search_query,
        Message::SearchChanged,
        results,
    )
    .width(Length::Fill)
    .into()
}

fn bottom_app_bar() -> material::Element<'static, Message> {
    material::widget::app_bar::bottom(
        [
            material::widget::app_bar::icon_button("menu")
                .on_press(Message::MenuPressed)
                .into(),
            material::widget::app_bar::icon_button("search")
                .on_press(Message::Increment)
                .into(),
            material::widget::app_bar::icon_button("info")
                .on_press(Message::Increment)
                .into(),
        ],
        Some(
            material::widget::button::primary_fab("+")
                .on_press(Message::Increment)
                .into(),
        ),
    )
    .into()
}

fn bottom_sheets(state: &Showcase) -> material::Element<'static, Message> {
    let width = structure_preview_width(state);
    let standard = material::widget::sheet::standard_bottom(sheet_content(
        "Standard bottom sheet",
        "Coexists with the page and keeps secondary content available.",
    ));

    let modal_preview = material::widget::sheet::modal_overlay(sheet_content(
        "Modal bottom sheet",
        "Uses a scrim and blocks interaction behind the sheet.",
    ))
    .height(Length::Fixed(260.0));

    page::stack([
        centered_preview(width, standard).into(),
        centered_preview(width, modal_preview).into(),
    ])
    .spacing(12)
    .into()
}

fn sheet_content(
    title: &'static str,
    supporting: &'static str,
) -> material::Element<'static, Message> {
    material::Container::new(
        page::stack([
            material::text::title_medium(title).into(),
            material::text::body_medium(supporting).into(),
            page::row([
                material::widget::button::text("Dismiss")
                    .on_press(Message::Decrement)
                    .into(),
                material::widget::button::filled("Apply")
                    .on_press(Message::Increment)
                    .into(),
            ])
            .into(),
        ])
        .spacing(8),
    )
    .padding(Padding {
        top: 0.0,
        right: 24.0,
        bottom: 24.0,
        left: 24.0,
    })
    .width(Length::Fill)
    .into()
}

fn centered_preview<'a>(
    width: f32,
    content: impl Into<material::Element<'a, Message>>,
) -> material::Container<'a, Message> {
    material::Container::new(material::Container::new(content).width(Length::Fixed(width)))
        .width(Length::Fill)
        .align_x(alignment::Horizontal::Center)
}

fn structure_preview_width(state: &Showcase) -> f32 {
    (state.window_size.width - STRUCTURE_PREVIEW_HORIZONTAL_RESERVE)
        .clamp(STRUCTURE_PREVIEW_MIN_WIDTH, STRUCTURE_PREVIEW_MAX_WIDTH)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn structure_preview_width_caps_wide_layouts_to_sheet_max_width() {
        let mut state = Showcase::default();

        state.window_size.width = 1920.0;
        assert_eq!(structure_preview_width(&state), 640.0);

        state.window_size.width = 420.0;
        assert_eq!(structure_preview_width(&state), 320.0);
    }
}
