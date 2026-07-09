mod controls;
mod feedback;
mod inputs;
mod navigation;
mod structure;
mod surfaces;

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

    material::widget::page::surface(header(page), content).into()
}

fn header(page: ShowcasePage) -> material::Element<'static, Message> {
    material::widget::page::header("material-ui-rs 0.4.1", page_label(page)).into()
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
