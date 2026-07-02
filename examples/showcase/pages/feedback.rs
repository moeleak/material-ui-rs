use iced_material as material;
use material::widget::page;

use super::super::{Message, Showcase};

pub(super) fn view(state: &Showcase) -> material::Element<'_, Message> {
    page::sections([
        page::section("Progress", progress_indicators(state)).into(),
        page::section("Badges", badges()).into(),
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
    page::stack([
        page::row([
            material::text::body_large("Progress")
                .width(iced::Length::Fill)
                .into(),
            material::text::body_large(format!("{:.0}%", state.progress)).into(),
        ])
        .into(),
        material::widget::slider::continuous(0.0..=100.0, state.progress, Message::SliderChanged)
            .step(1.0)
            .into(),
        material::widget::progress_bar::linear(0.0..=100.0, state.progress).into(),
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
