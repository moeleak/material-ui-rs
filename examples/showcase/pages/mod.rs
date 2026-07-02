mod controls;
mod feedback;
mod inputs;
mod navigation;
mod surfaces;

use iced_material as material;

use super::{Message, Showcase, ShowcasePage};

pub(super) fn view(state: &Showcase) -> material::Element<'_, Message> {
    let page = state.navigation.selected();
    let content = match page {
        ShowcasePage::Inputs => inputs::view(state),
        ShowcasePage::Controls => controls::view(state),
        ShowcasePage::Feedback => feedback::view(state),
        ShowcasePage::Surfaces => surfaces::view(),
        ShowcasePage::Navigation => navigation::view(state),
    };

    material::widget::page::surface(header(page), content).into()
}

fn header(page: ShowcasePage) -> material::Element<'static, Message> {
    let body_large = material::tokens::typography::BODY_LARGE;
    let chinese_sample = "中文字体 Noto Sans CJK";

    material::widget::page::header("iced_material 0.14.2", page_label(page))
        .push(material::text::body_large(chinese_sample).font(
            material::fonts::font_for_content_type_scale(chinese_sample, body_large),
        ))
        .into()
}

fn page_label(page: ShowcasePage) -> &'static str {
    match page {
        ShowcasePage::Inputs => "Inputs",
        ShowcasePage::Controls => "Controls",
        ShowcasePage::Feedback => "Feedback",
        ShowcasePage::Surfaces => "Surfaces",
        ShowcasePage::Navigation => "Navigation",
    }
}

#[cfg(test)]
pub(super) fn navigation_showcase_rail_height() -> f32 {
    navigation::showcase_rail_height()
}
