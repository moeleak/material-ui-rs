mod controls;
mod feedback;
mod inputs;
mod navigation;
mod structure;
mod surfaces;

use iced::Length;
use iced::widget::{Column, Space};
use material::widget::{page, theme_picker};
use material_ui_rs as material;

use super::{Message, Showcase, ShowcasePage};

pub(super) fn view(state: &Showcase) -> material::Element<'_, Message> {
    let page = state.navigation.selected();
    let content = match page {
        ShowcasePage::Inputs => inputs::view(state),
        ShowcasePage::Controls => controls::view(state),
        ShowcasePage::Feedback => feedback::view(state),
        ShowcasePage::Surfaces => surfaces::view(),
        ShowcasePage::Navigation => navigation::view(state),
        ShowcasePage::Structure => structure::view(state),
    };
    let content = Column::new()
        .push(content)
        .push(Space::new().height(Length::Fixed(floating_content_inset(state))));

    page::surface(header(page), content).into()
}

pub(super) fn floating_content_inset(state: &Showcase) -> f32 {
    (theme_picker::FLOATING_MARGIN
        + state.theme_controller.floating_clearance()
        + material::tokens::component::snackbar::BOTTOM_MARGIN
        - page::PADDING)
        .max(0.0)
}

fn header(page: ShowcasePage) -> material::Element<'static, Message> {
    material::widget::page::header("material-ui-rs 0.5.3", page_label(page)).into()
}

fn page_label(page: ShowcasePage) -> &'static str {
    match page {
        ShowcasePage::Inputs => "Inputs",
        ShowcasePage::Controls => "Controls",
        ShowcasePage::Feedback => "Feedback",
        ShowcasePage::Surfaces => "Surfaces",
        ShowcasePage::Navigation => "Navigation",
        ShowcasePage::Structure => "Structure",
    }
}
