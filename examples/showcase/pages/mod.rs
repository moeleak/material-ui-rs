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

    material::widget::page::surface(header(state, page), content).into()
}

fn header(state: &Showcase, page: ShowcasePage) -> material::Element<'static, Message> {
    let header = material::widget::page::header("material-ui-rs 0.4.2", page_label(page));

    if state.cjk_font_status == super::CjkFontStatus::Loaded {
        header
            .push(
                material::text::body_large("中文字体已按需加载")
                    .font(material::fonts::NOTO_SANS_CJK_SC),
            )
            .into()
    } else {
        header.into()
    }
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
