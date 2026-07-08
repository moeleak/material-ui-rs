use material::widget::page;
use material_ui_rs as material;

use super::super::{Message, Showcase};

pub(super) fn view(state: &Showcase) -> material::Element<'_, Message> {
    page::sections([
        page::section("Progress", progress_indicators(state)).into(),
        page::section("Badges", badges()).into(),
        page::section("Snackbars", snackbars()).into(),
        page::section("Dialogs", dialogs()).into(),
        page::section("Tooltips", tooltips(state)).into(),
    ])
    .into()
}

fn progress_indicators(state: &Showcase) -> material::Element<'_, Message> {
    use material::widget::progress_bar::{self, LinearProgressMode, LoadingIndicatorMode};

    let progress = state.progress / 100.0;
    let linear_phase = state.progress_animation.linear_phase();
    let loading_phase = state.progress_animation.loading_phase();

    page::dense_stack([
        page::labeled_value_row("Determinate", format!("{:.0}%", state.progress)).into(),
        material::widget::slider::continuous(0.0..=100.0, state.progress, Message::SliderChanged)
            .step(1.0)
            .into(),
        progress_bar::linear(LinearProgressMode::determinate(progress, linear_phase)).into(),
        progress_bar::linear(LinearProgressMode::indeterminate(linear_phase)).into(),
        page::indicator_row([
            progress_bar::loading(LoadingIndicatorMode::indeterminate(loading_phase)).into(),
            progress_bar::loading(LoadingIndicatorMode::contained_indeterminate(loading_phase))
                .into(),
        ])
        .into(),
    ])
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
    use material::widget::button::{self, ButtonVariant};

    page::row([button::action(
        button::button("Show snackbar", ButtonVariant::Filled),
        Message::ShowSnackbar,
    )])
    .into()
}

fn dialogs() -> material::Element<'static, Message> {
    use material::widget::button::{self, ButtonVariant};

    page::row([button::action(
        button::button("Open alert dialog", ButtonVariant::Filled),
        Message::DialogOpened,
    )])
    .into()
}

fn tooltips(state: &Showcase) -> material::Element<'_, Message> {
    use material::widget::button::{self, ChipVariant};

    page::row([
        material::widget::tooltip::plain(
            button::optional_action(
                button::chip("Plain", ChipVariant::Assist),
                state.enabled.then_some(Message::Increment),
            ),
            "Material 3 plain tooltip",
            material::widget::tooltip::Position::Top,
        )
        .into(),
        material::widget::tooltip::rich_with_title_action(
            button::optional_action(
                button::chip("Rich", ChipVariant::Assist),
                state.enabled.then_some(Message::Increment),
            ),
            "Rich tooltip",
            "Additional context and a related action can be shown together.",
            material::widget::tooltip::rich_action_button("Action", Message::Increment),
            material::widget::tooltip::Position::Top,
        )
        .into(),
    ])
    .into()
}
