use iced::widget::{Column, Row, Space, Stack};
use iced::{Length, alignment};
use material::widget::page;
use material_ui_rs as material;

use super::super::{Message, Showcase};

pub(super) fn view(state: &Showcase) -> material::Element<'_, Message> {
    page::sections([
        page::section("Log viewer", log_viewer(state)).into(),
        page::section("Top app bars", top_app_bars()).into(),
        page::section("Search view", search_view(state)).into(),
        page::section("Toolbars", toolbars()).into(),
        page::section("Bottom app bar", bottom_app_bar()).into(),
        page::section("Bottom sheets", bottom_sheets(state)).into(),
        page::section("Side sheets", side_sheets(state)).into(),
    ])
    .into()
}

fn log_viewer(state: &Showcase) -> material::Element<'_, Message> {
    use iced::Length;
    use iced::widget::Container;
    use material::widget::log_viewer;

    Container::new(
        log_viewer::view(&state.log_entries, &state.log_viewer, Message::LogViewer)
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fixed(620.0))
    .into()
}

fn top_app_bars() -> material::Element<'static, Message> {
    use material::widget::app_bar;

    page::component_stack([
        app_bar::with_status_bar(app_bar::small(
            "Small",
            Some(app_bar::icon_action("menu", Message::MenuPressed)),
            app_bar::icon_actions([("search", Message::Increment), ("info", Message::Increment)]),
        ))
        .into(),
        app_bar::with_status_bar(app_bar::medium(
            "Medium",
            Some(app_bar::icon_action("menu", Message::MenuPressed)),
            app_bar::icon_actions([("search", Message::Increment), ("info", Message::Increment)]),
        ))
        .into(),
        app_bar::with_status_bar(app_bar::large(
            "Large",
            Some(app_bar::icon_action("menu", Message::MenuPressed)),
            app_bar::icon_actions([("search", Message::Increment), ("info", Message::Increment)]),
        ))
        .into(),
    ])
    .into()
}

fn search_view(state: &Showcase) -> material::Element<'_, Message> {
    let results = material::widget::list::group([
        material::widget::list::one_line_icon("input", "Inputs").into(),
        material::widget::list::one_line_icon("tune", "Controls").into(),
        material::widget::list::one_line_icon("info", "Feedback").into(),
    ]);

    material::widget::search::docked_view(
        "Search components",
        &state.search_query,
        Message::SearchChanged,
        results,
    )
    .into()
}

fn bottom_app_bar() -> material::Element<'static, Message> {
    use material::widget::app_bar;
    use material::widget::button::{self, FabSize, FabVariant};

    app_bar::bottom(
        app_bar::icon_actions([
            ("menu", Message::MenuPressed),
            ("search", Message::Increment),
            ("info", Message::Increment),
        ]),
        Some(button::action(
            button::fab("add", FabVariant::Primary, FabSize::Standard),
            Message::Increment,
        )),
    )
    .into()
}

fn toolbars() -> material::Element<'static, Message> {
    use material::widget::button::{self, FabSize, FabVariant};
    use material::widget::toolbar;

    let docked = toolbar::docked(toolbar::icon_actions([
        ("arrow_back", Message::Decrement),
        ("arrow_forward", Message::Increment),
        ("add", Message::Increment),
        ("tab", Message::Increment),
        ("star", Message::Increment),
        ("search", Message::Increment),
    ]));

    let docked_vibrant = toolbar::docked_vibrant([
        toolbar::vibrant_icon_action("edit", Message::Increment),
        toolbar::selected_vibrant_icon_action("check", Message::Increment),
        toolbar::vibrant_icon_action("delete", Message::Decrement),
        toolbar::vibrant_icon_action("more_vert", Message::MenuPressed),
    ]);

    let floating = toolbar::floating([
        toolbar::selected_icon_action("format_bold", Message::Increment),
        toolbar::icon_action("format_italic", Message::Increment),
        toolbar::icon_action("format_underlined", Message::Increment),
        toolbar::icon_action("format_color_text", Message::Increment),
    ]);

    let floating_vibrant = toolbar::floating_with_fab(
        toolbar::floating_vibrant(toolbar::vibrant_icon_actions([
            ("share", Message::Increment),
            ("add", Message::Increment),
            ("edit", Message::Increment),
        ])),
        button::fab("search", FabVariant::Primary, FabSize::Standard).on_press(Message::Increment),
    );

    let vertical = toolbar::vertical_floating([
        toolbar::icon_action("undo", Message::Decrement),
        toolbar::icon_action("redo", Message::Increment),
        toolbar::selected_icon_action("palette", Message::Increment),
    ]);

    page::component_stack([
        docked.into(),
        docked_vibrant.into(),
        page::row([floating.into(), vertical.into()]).into(),
        floating_vibrant.into(),
    ])
    .into()
}

fn bottom_sheets(state: &Showcase) -> material::Element<'static, Message> {
    let width = page::preview_width(state.window_size.width);
    let standard = material::widget::sheet::standard_bottom(sheet_content(
        "Standard bottom sheet",
        "Coexists with the page and keeps secondary content available.",
    ));

    let modal_preview = page::preview_pane(material::widget::sheet::modal_overlay(sheet_content(
        "Modal bottom sheet",
        "Uses a scrim and blocks interaction behind the sheet.",
    )));

    page::component_stack([
        page::centered_preview(width, standard).into(),
        page::centered_preview(width, modal_preview).into(),
    ])
    .into()
}

fn side_sheets(state: &Showcase) -> material::Element<'static, Message> {
    let width = page::preview_width(state.window_size.width);
    let standard = page::preview_pane(
        Row::new()
            .push(side_sheet_preview_content())
            .push(material::widget::sheet::standard_side(side_sheet_content(
                "Standard side sheet",
                "Coexists with the page while supporting content remains visible.",
            )))
            .width(Length::Fill)
            .height(Length::Fill),
    );

    let modal = page::preview_pane(
        Stack::with_children([
            side_sheet_preview_content(),
            material::widget::sheet::modal_side_overlay(
                material::widget::sheet::Side::Right,
                side_sheet_content(
                    "Modal side sheet",
                    "Uses a scrim and keeps focus on a temporary side task.",
                ),
            )
            .into(),
        ])
        .width(Length::Fill)
        .height(Length::Fill),
    );

    page::component_stack([
        page::centered_preview(width, standard).into(),
        page::centered_preview(width, modal).into(),
    ])
    .into()
}

fn side_sheet_preview_content() -> material::Element<'static, Message> {
    let supporting_items = material::widget::list::group([
        material::widget::list::one_line_icon("description", "Project brief").into(),
        material::widget::list::one_line_icon("schedule", "Recent activity").into(),
    ]);

    material::widget::container::surface_container_highest(
        Column::new()
            .push(material::text::title_medium("Supporting page"))
            .push(material::text::body_medium(
                "Primary content remains visible beside the standard sheet.",
            ))
            .push(Space::new().height(Length::Fixed(page::STACK_SPACING)))
            .push(supporting_items)
            .spacing(page::COMPACT_STACK_SPACING),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .padding(page::CARD_PADDING)
    .into()
}

fn sheet_content(
    title: &'static str,
    supporting: &'static str,
) -> material::Element<'static, Message> {
    use material::widget::button::{self, ButtonVariant};

    material::widget::sheet::bottom_content(page::compact_stack([
        material::text::title_medium(title).into(),
        material::text::body_medium(supporting).into(),
        page::row([
            button::action(
                button::button("Dismiss", ButtonVariant::Text),
                Message::Decrement,
            ),
            button::action(
                button::button("Apply", ButtonVariant::Filled),
                Message::Increment,
            ),
        ])
        .into(),
    ]))
    .into()
}

fn side_sheet_content(
    title: &'static str,
    supporting: &'static str,
) -> material::Element<'static, Message> {
    use material::widget::button::{self, ButtonVariant, IconButtonVariant};

    let header = Row::new()
        .push(material::text::title_medium(title).width(Length::Fill))
        .push(button::action(
            button::icon_button("close", IconButtonVariant::Standard),
            Message::Decrement,
        ))
        .align_y(alignment::Vertical::Center);
    let actions = Row::new()
        .push(Space::new().width(Length::Fill))
        .push(button::action(
            button::button("Apply", ButtonVariant::Filled),
            Message::Increment,
        ))
        .spacing(page::ROW_SPACING)
        .align_y(alignment::Vertical::Center);

    material::widget::sheet::side_content(
        Column::new()
            .push(header)
            .push(material::text::body_medium(supporting))
            .push(Space::new().height(Length::Fill))
            .push(actions)
            .spacing(page::COMPACT_STACK_SPACING)
            .height(Length::Fill),
    )
    .into()
}
