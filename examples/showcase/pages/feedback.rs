use iced_material as material;
use material::widget::page;

use super::super::{Message, Showcase};

pub(super) fn view(state: &Showcase) -> material::Element<'_, Message> {
    page::sections([
        page::section("Progress", progress_indicators(state)).into(),
        page::section("Badges", badges()).into(),
        page::section("Snackbars", snackbars()).into(),
        page::section(
            "Tooltip",
            material::widget::tooltip::plain(
                material::widget::button::assist_chip("Hint")
                    .on_press_maybe(state.enabled.then_some(Message::Increment)),
                "Material 3 plain tooltip",
                material::widget::tooltip::Position::Top,
            ),
        )
        .into(),
    ])
    .into()
}

fn progress_indicators(state: &Showcase) -> material::Element<'_, Message> {
    let progress = state.progress / 100.0;
    let linear_phase = state.progress_animation.linear_phase();
    let loading_phase = state.progress_animation.loading_phase();

    page::stack([
        page::row([
            material::text::body_large("Determinate")
                .width(iced::Length::Fill)
                .into(),
            material::text::body_large(format!("{:.0}%", state.progress)).into(),
        ])
        .into(),
        material::widget::slider::continuous(0.0..=100.0, state.progress, Message::SliderChanged)
            .step(1.0)
            .into(),
        material::widget::progress_bar::linear(progress, linear_phase).into(),
        material::widget::progress_bar::linear_indeterminate(linear_phase, false).into(),
        page::row([
            material::widget::progress_bar::loading_indicator(loading_phase).into(),
            material::widget::progress_bar::contained_loading_indicator(loading_phase).into(),
        ])
        .spacing(16)
        .into(),
    ])
    .spacing(10)
    .into()
}

fn badges() -> material::Element<'static, Message> {
    page::row([
        material::text::body_large("Badges").into(),
        material::widget::badge::small().into(),
        material::widget::badge::large("3").into(),
        material::widget::badge::large("99+").into(),
    ])
    .into()
}

fn snackbars() -> material::Element<'static, Message> {
    page::stack([
        material::widget::snackbar::single_line_with_action(
            "Photo archived",
            material::widget::snackbar::action("Undo").on_press(Message::Decrement),
        )
        .into(),
        material::widget::snackbar::two_line_with_action(
            "Offline changes will sync when the device reconnects.",
            material::widget::snackbar::icon_action("close").on_press(Message::Increment),
        )
        .into(),
    ])
    .spacing(8)
    .into()
}
