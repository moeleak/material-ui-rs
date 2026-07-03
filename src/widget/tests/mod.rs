use iced_widget::core::Element;

use super::*;

#[derive(Debug, Clone)]
enum Message {
    Pressed,
    Toggled,
}

type TestElement<'a> = Element<'a, Message, Theme, iced_widget::Renderer>;

fn toggled(_: bool) -> Message {
    Message::Toggled
}

#[test]
fn centered_icon_text_uses_square_icon_bounds() {
    let icon: Text<'_, Theme, iced_widget::Renderer> =
        centered_icon_text("add", tokens::component::fab::ICON_SIZE);

    assert_eq!(
        Widget::<Message, Theme, iced_widget::Renderer>::size(&icon),
        Size {
            width: Length::Fixed(tokens::component::fab::ICON_SIZE),
            height: Length::Fixed(tokens::component::fab::ICON_SIZE),
        }
    );
}

#[test]
fn material_button_constructors_compile_to_elements() {
    let _: TestElement<'_> = button::filled("Filled").on_press(Message::Pressed).into();
    let _: TestElement<'_> = button::filled_action("Filled", Message::Pressed);
    let _: TestElement<'_> = button::maybe_action(button::filled("Maybe"), true, Message::Pressed);
    let _: Vec<TestElement<'_>> = button::enabled_actions(
        true,
        Message::Pressed,
        [button::filled("One"), button::text("Two")],
    );
    let _: TestElement<'_> = button::filled_tonal("Tonal")
        .on_press(Message::Pressed)
        .into();
    let _: TestElement<'_> = button::outlined_action("Outlined", Message::Pressed);
    let _: TestElement<'_> = button::text_action("Text", Message::Pressed);
    let _: TestElement<'_> = button::outlined_icon("add")
        .on_press(Message::Pressed)
        .into();
    let _: TestElement<'_> = button::primary_fab("add").on_press(Message::Pressed).into();
    let _: TestElement<'_> = button::primary_fab_action("add", Message::Pressed);
    let _: TestElement<'_> = button::primary_small_fab("add")
        .on_press(Message::Pressed)
        .into();
    let _: TestElement<'_> = button::primary_large_fab("add")
        .on_press(Message::Pressed)
        .into();
    let _: TestElement<'_> = button::secondary_fab("add")
        .on_press(Message::Pressed)
        .into();
    let _: TestElement<'_> = button::tertiary_fab("add")
        .on_press(Message::Pressed)
        .into();
    let _: TestElement<'_> = button::surface_fab("add").on_press(Message::Pressed).into();
    let _: TestElement<'_> = button::surface_small_fab("add")
        .on_press(Message::Pressed)
        .into();
    let _: TestElement<'_> = button::surface_large_fab("add")
        .on_press(Message::Pressed)
        .into();
    let _: TestElement<'_> = button::primary_extended_fab("Create")
        .on_press(Message::Pressed)
        .into();
    let _: TestElement<'_> = button::primary_extended_fab_with_icon("add", "Create")
        .on_press(Message::Pressed)
        .into();
    let _: TestElement<'_> = button::secondary_extended_fab("Share")
        .on_press(Message::Pressed)
        .into();
    let _: TestElement<'_> = button::tertiary_extended_fab_with_icon("add", "Add")
        .on_press(Message::Pressed)
        .into();
    let _: TestElement<'_> = button::surface_extended_fab("Reroute")
        .on_press(Message::Pressed)
        .into();
    let _: TestElement<'_> = button::assist_chip("Assist")
        .on_press(Message::Pressed)
        .into();
}

#[test]
fn material_badge_constructors_compile_to_elements() {
    let _: TestElement<'_> = badge::small().into();
    let _: TestElement<'_> = badge::large("3").into();
    let _: TestElement<'_> = badge::large("99+").into();
}

#[test]
fn material_container_constructors_compile_to_elements() {
    let surface = Text::new("Surface");
    let _: TestElement<'_> = container::surface_container_high(surface).into();
}

#[test]
fn material_toolbar_constructors_compile_to_elements() {
    let _: TestElement<'_> = toolbar::docked(toolbar::icon_actions([
        ("arrow_back", Message::Pressed),
        ("add", Message::Pressed),
    ]))
    .into();
    let _: TestElement<'_> = toolbar::docked_vibrant(toolbar::vibrant_icon_actions([
        ("edit", Message::Pressed),
        ("delete", Message::Pressed),
    ]))
    .into();
    let _: TestElement<'_> = toolbar::floating(toolbar::icon_actions([
        ("format_bold", Message::Pressed),
        ("format_italic", Message::Pressed),
    ]))
    .into();
    let _: TestElement<'_> = toolbar::floating_vibrant([
        toolbar::selected_vibrant_icon_action("format_bold", Message::Pressed),
        toolbar::vibrant_icon_action("format_italic", Message::Pressed),
    ])
    .into();
    let _: TestElement<'_> =
        toolbar::vertical_floating(toolbar::icon_actions([("undo", Message::Pressed)])).into();
    let _: TestElement<'_> = toolbar::vertical_floating_vibrant(toolbar::vibrant_icon_actions([(
        "redo",
        Message::Pressed,
    )]))
    .into();
    let floating = toolbar::floating(toolbar::icon_actions([("share", Message::Pressed)]));
    let fab = button::primary_fab("add").on_press(Message::Pressed);
    let _: TestElement<'_> = toolbar::floating_with_fab(floating, fab).into();

    let picker_state = theme_picker::State::new();
    let content: TestElement<'_> = Text::new("Theme picker content").into();
    let _: TestElement<'_> = theme_picker::floating_over(
        content,
        &picker_state,
        theme_picker::MaterialColor::Purple,
        theme_picker::FLOATING_MARGIN,
        Message::Pressed,
        |_| Message::Pressed,
    );
}

#[test]
fn material_card_constructors_compile_to_elements() {
    let elevated = Text::new("Elevated card");
    let _: TestElement<'_> = card::elevated(elevated).into();

    let filled = Text::new("Filled card");
    let _: TestElement<'_> = card::filled(filled).into();

    let outlined = Text::new("Outlined card");
    let _: TestElement<'_> = card::outlined(outlined).into();

    let legacy = Text::new("Legacy path");
    let _: TestElement<'_> = container::outlined_card(legacy).into();
}

#[test]
fn material_dialog_constructors_compile_to_elements() {
    let content: TestElement<'_> = Text::new("Custom dialog content").into();
    let _: TestElement<'_> = dialog::basic(content).into();

    let actions: [TestElement<'_>; 2] = [
        dialog::action("Cancel").on_press(Message::Pressed).into(),
        dialog::action("OK").on_press(Message::Pressed).into(),
    ];
    let _: TestElement<'_> =
        dialog::alert("Title", "Supporting text", dialog::actions(actions)).into();

    let actions: [TestElement<'_>; 2] = [
        dialog::action_button("Cancel", Message::Pressed),
        dialog::action_button("OK", Message::Pressed),
    ];
    let _: TestElement<'_> =
        dialog::alert("Title", "Supporting text", dialog::actions(actions)).into();

    let actions: [TestElement<'_>; 1] = [dialog::action("Done").on_press(Message::Pressed).into()];
    let _: TestElement<'_> =
        dialog::alert_with_icon("info", "Title", "Supporting text", dialog::actions(actions))
            .into();

    let actions: [TestElement<'_>; 2] = [
        dialog::action_button_alpha("Cancel", Message::Pressed, 0.5),
        dialog::action_button_alpha("OK", Message::Pressed, 0.5),
    ];
    let _: TestElement<'_> = dialog::alert_with_icon_alpha(
        "info",
        "Title",
        "Supporting text",
        dialog::actions(actions),
        0.5,
    )
    .into();

    let content: TestElement<'_> = Text::new("Scrim content").into();
    let _: TestElement<'_> = dialog::scrim(content).into();

    let content: TestElement<'_> = Text::new("Animated scrim content").into();
    let _: TestElement<'_> = dialog::scrim_alpha(content, 0.5).into();

    let content: TestElement<'_> = Text::new("Modal dialog").into();
    let _: TestElement<'_> = dialog::modal_overlay(content).into();

    let content: TestElement<'_> = Text::new("Modal layer dialog").into();
    let _: TestElement<'_> = dialog::modal_layer(content);

    let content: TestElement<'_> = Text::new("Page content").into();
    let dialog_content: TestElement<'_> = Text::new("Dialog content").into();
    let _: TestElement<'_> = dialog::modal(content, dialog_content);

    let transition = dialog::Transition::default();
    let content: TestElement<'_> = Text::new("Page content").into();
    let dialog_content: TestElement<'_> = Text::new("Animated dialog content").into();
    let _: TestElement<'_> = dialog::modal_animated(
        content,
        &transition,
        iced_widget::core::time::Instant::now(),
        dialog_content,
    );
}

#[test]
fn material_app_bar_constructors_compile_to_elements() {
    let _: TestElement<'_> = app_bar::icon_action("info", Message::Pressed);
    let _: Vec<TestElement<'_>> =
        app_bar::icon_actions([("search", Message::Pressed), ("info", Message::Pressed)]);

    let leading = app_bar::icon_button("menu")
        .on_press(Message::Pressed)
        .into();
    let actions = [app_bar::icon_button("search")
        .on_press(Message::Pressed)
        .into()];
    let small = app_bar::small("Small", Some(leading), actions);
    let _: TestElement<'_> = app_bar::with_status_bar(small).into();

    let leading = app_bar::icon_button("menu")
        .on_press(Message::Pressed)
        .into();
    let actions = [app_bar::icon_button("search")
        .on_press(Message::Pressed)
        .into()];
    let _: TestElement<'_> = app_bar::medium("Medium", Some(leading), actions).into();

    let actions = [app_bar::icon_button("info")
        .on_press(Message::Pressed)
        .into()];
    let fab = button::primary_fab("add").on_press(Message::Pressed).into();
    let _: TestElement<'_> = app_bar::bottom(actions, Some(fab)).into();
}

#[test]
fn material_navigation_constructors_compile_to_elements() {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Page {
        First,
        Second,
    }

    let destinations = [
        navigation::Destination::new(Page::First, "1", "First"),
        navigation::Destination::new(Page::Second, "2", "Second"),
    ];
    let selection = navigation::Selection::new(Page::First);

    let _: TestElement<'_> =
        navigation::navigation_bar(&destinations, selection, |_| Message::Pressed).into();
    let _: TestElement<'_> =
        navigation::navigation_rail(&destinations, selection, |_| Message::Pressed).into();
    let _: TestElement<'_> =
        navigation::navigation_rail_fitting_content(&destinations, selection, |_| Message::Pressed)
            .into();
    let _: TestElement<'_> = navigation::navigation_rail_with_header(
        &destinations,
        selection,
        |_| Message::Pressed,
        Text::new("Header"),
    )
    .into();
    let _: TestElement<'_> = navigation::navigation_rail_with_header_fitting_content(
        &destinations,
        selection,
        |_| Message::Pressed,
        Text::new("Header"),
    )
    .into();
    let _: TestElement<'_> = navigation::navigation_rail_with_menu(
        &destinations,
        selection,
        |_| Message::Pressed,
        Message::Pressed,
    )
    .into();
    let _: TestElement<'_> = navigation::navigation_rail_with_menu_fitting_content(
        &destinations,
        selection,
        |_| Message::Pressed,
        Message::Pressed,
    )
    .into();
    let _: TestElement<'_> = navigation::navigation_rail_expanded_with_menu(
        "Navigation",
        &destinations,
        selection,
        |_| Message::Pressed,
        Message::Pressed,
    )
    .into();
    let _: TestElement<'_> = navigation::navigation_rail_expanded_with_menu_fitting_content(
        "Navigation",
        &destinations,
        selection,
        |_| Message::Pressed,
        Message::Pressed,
    )
    .into();
    let _: TestElement<'_> = navigation::navigation_rail_expanded_with_menu_at_width(
        "Navigation",
        &destinations,
        selection,
        |_| Message::Pressed,
        Message::Pressed,
        220.0,
    )
    .into();
    let _: TestElement<'_> =
        navigation::navigation_drawer("Navigation", &destinations, selection, |_| Message::Pressed)
            .into();
    let _: TestElement<'_> = navigation::navigation_drawer_at_width(
        "Navigation",
        &destinations,
        selection,
        |_| Message::Pressed,
        240.0,
    )
    .into();
    let _: TestElement<'_> = navigation::navigation_drawer_with_menu(
        "Navigation",
        &destinations,
        selection,
        |_| Message::Pressed,
        Message::Pressed,
    )
    .into();
    let _: TestElement<'_> = navigation::navigation_drawer_with_menu_at_width(
        "Navigation",
        &destinations,
        selection,
        |_| Message::Pressed,
        Message::Pressed,
        240.0,
    )
    .into();
    let content = Text::new("Navigation suite content");
    let _: TestElement<'_> = navigation::navigation_suite(
        1080.0,
        980.0,
        &destinations,
        selection,
        |_| Message::Pressed,
        content,
    );
    let content = Text::new("Navigation suite menu content");
    let state = navigation::NavigationState::new(Page::First);
    let _: TestElement<'_> = navigation::navigation_suite_with_menu(
        "Navigation",
        1080.0,
        980.0,
        &destinations,
        &state,
        |_| Message::Pressed,
        Message::Pressed,
        content,
    );
    let content = Text::new("Navigation suite builder content");
    let _: TestElement<'_> = navigation::suite(&destinations, &state)
        .window_size(Size::new(1080.0, 980.0))
        .with_menu("Navigation", Message::Pressed)
        .view(|_| Message::Pressed, content);
}

#[test]
fn material_tabs_constructors_compile_to_elements() {
    let _: TestElement<'_> = tabs::bar([
        tabs::primary_icon_label("input", "Inputs", true)
            .on_press(Message::Pressed)
            .into(),
        tabs::primary_inline_icon_label("tune", "Controls", false)
            .on_press(Message::Pressed)
            .into(),
        tabs::primary_label("Feedback", false)
            .on_press(Message::Pressed)
            .into(),
    ])
    .into();

    let _: TestElement<'_> = tabs::bar([
        tabs::secondary_icon_label("info", "Overview", true)
            .on_press(Message::Pressed)
            .into(),
        tabs::secondary_label("Details", false)
            .on_press(Message::Pressed)
            .into(),
    ])
    .into();

    let tab_state = tabs::State::new(0);
    let _: TestElement<'_> = tabs::animated_bar(
        tabs::Variant::Primary,
        3,
        &tab_state,
        [
            tabs::primary_label_for_animated_bar("Inputs", true)
                .on_press(Message::Pressed)
                .into(),
            tabs::primary_inline_icon_label_for_animated_bar("tune", "Controls", false)
                .on_press(Message::Pressed)
                .into(),
            tabs::primary_icon_label_action_for_animated_bar(
                "info",
                "Feedback",
                false,
                Message::Pressed,
            ),
        ],
    )
    .into();

    let _: TestElement<'_> = tabs::animated_primary_icon_label_bar(
        &tab_state,
        [
            ("input", "Inputs", Message::Pressed),
            ("tune", "Controls", Message::Pressed),
        ],
    )
    .into();

    let _: TestElement<'_> = tabs::animated_bar(
        tabs::Variant::Secondary,
        1,
        &tab_state,
        [tabs::secondary_label_action_for_animated_bar(
            "Details",
            true,
            Message::Pressed,
        )],
    )
    .into();

    let _: TestElement<'_> = tabs::animated_secondary_label_bar(
        &tab_state,
        [("Details", Message::Pressed), ("History", Message::Pressed)],
    )
    .into();
}

#[test]
fn material_segmented_button_constructors_compile_to_elements() {
    use segmented_button::SegmentPosition;

    let segment_state = segmented_button::State::new(0);
    let _: TestElement<'_> = segmented_button::group([
        segmented_button::animated_selectable_label_action(
            "List",
            1.0,
            SegmentPosition::First,
            Message::Pressed,
        ),
        segmented_button::animated_selectable_label_action(
            "Grid",
            0.0,
            SegmentPosition::Last,
            Message::Pressed,
        ),
    ])
    .into();
    let _: TestElement<'_> =
        segmented_button::group(segmented_button::animated_selectable_label_actions(
            &segment_state,
            [
                ("List", Message::Pressed),
                ("Grid", Message::Pressed),
                ("Map", Message::Pressed),
            ],
        ))
        .into();
}

#[test]
fn material_pick_list_constructor_compiles_to_element() {
    let options = ["Assist", "Suggestion", "Filter"];
    let _: TestElement<'_> = pick_list::outlined(options, Some("Assist"), |_| Message::Pressed)
        .placeholder("Choose")
        .into();
}

#[test]
fn material_pick_list_defaults_to_fill_width() {
    let options = ["Assist", "Suggestion", "Filter"];
    let select: select::Select<'_, _, _, _, Message, iced_widget::Renderer> =
        pick_list::outlined(options, Some("Assist"), |_| Message::Pressed);

    assert_eq!(
        Widget::<Message, Theme, iced_widget::Renderer>::size(&select).width,
        Length::Fill
    );
}

#[test]
fn material_combo_box_constructor_compiles_to_element() {
    let selected = "Assist";
    let options =
        combo_box::State::with_selection(vec!["Assist", "Suggestion", "Filter"], Some(&selected));
    let _: TestElement<'_> =
        combo_box::outlined(&options, "Choose", Some(&selected), |_| Message::Pressed).into();
}

#[test]
fn material_combo_box_with_input_constructor_compiles_to_element() {
    let options = combo_box::State::new(vec!["Assist", "Suggestion", "Filter"]);
    let _: TestElement<'_> =
        combo_box::outlined_with_input(&options, "Choose", "xxx", None, |_| Message::Pressed)
            .on_input(|_| Message::Pressed)
            .into();
}

#[test]
fn material_list_item_constructors_compile_to_elements() {
    let _: TestElement<'_> = list::one_line("Single line").into();
    let _: TestElement<'_> = list::one_line_with_leading_icon("info", "With leading icon").into();
    let _: TestElement<'_> = list::two_line("Two line", "Supporting text").into();
    let _: TestElement<'_> = list::two_line_with_trailing("Inventory", "In stock", "42").into();
    let _: TestElement<'_> =
        list::three_line("Three line", "Supporting text", "Second supporting line").into();
    let _: TestElement<'_> = list::group([
        list::one_line("First").into(),
        list::one_line("Second").into(),
    ])
    .into();
}

#[test]
fn material_sheet_constructors_compile_to_elements() {
    let content: TestElement<'_> = Text::new("Sheet content").into();
    let _: TestElement<'_> = sheet::modal_bottom(content).into();

    let content: TestElement<'_> = Text::new("Sheet content").into();
    let _: TestElement<'_> = sheet::standard_bottom(content).into();

    let content: TestElement<'_> = Text::new("Bottom sheet content").into();
    let _: TestElement<'_> = sheet::bottom_content(content).into();

    let content: TestElement<'_> = Text::new("Scrim content").into();
    let _: TestElement<'_> = sheet::scrim(content).into();

    let content: TestElement<'_> = Text::new("Modal overlay content").into();
    let _: TestElement<'_> = sheet::modal_overlay(content).into();

    let content: TestElement<'_> = Text::new("Modal side sheet content").into();
    let _: TestElement<'_> = sheet::modal_side(content).into();

    let content: TestElement<'_> = Text::new("Left modal side sheet content").into();
    let _: TestElement<'_> = sheet::modal_side_on(sheet::Side::Left, content).into();

    let content: TestElement<'_> = Text::new("Detached modal side sheet content").into();
    let _: TestElement<'_> = sheet::detached_modal_side(content).into();

    let content: TestElement<'_> = Text::new("Detached left modal side sheet content").into();
    let _: TestElement<'_> = sheet::detached_modal_side_on(sheet::Side::Left, content).into();

    let content: TestElement<'_> = Text::new("Standard side sheet content").into();
    let _: TestElement<'_> = sheet::standard_side(content).into();

    let content: TestElement<'_> = Text::new("Side sheet content").into();
    let _: TestElement<'_> = sheet::side_content(content).into();

    let content: TestElement<'_> = Text::new("Left standard side sheet content").into();
    let _: TestElement<'_> = sheet::standard_side_on(sheet::Side::Left, content).into();

    let content: TestElement<'_> = Text::new("Detached standard side sheet content").into();
    let _: TestElement<'_> = sheet::detached_standard_side(content).into();

    let content: TestElement<'_> = Text::new("Detached left standard side sheet content").into();
    let _: TestElement<'_> = sheet::detached_standard_side_on(sheet::Side::Left, content).into();

    let content: TestElement<'_> = Text::new("Side scrim content").into();
    let _: TestElement<'_> = sheet::side_scrim(content).into();

    let content: TestElement<'_> = Text::new("Side overlay content").into();
    let _: TestElement<'_> = sheet::modal_side_overlay(sheet::Side::Right, content).into();
}

#[test]
fn material_snackbar_constructors_compile_to_elements() {
    let _: TestElement<'_> = snackbar::single_line("Archived").into();
    let _: TestElement<'_> = snackbar::single_line_with_action(
        "Archived",
        snackbar::action_button("Undo", Message::Pressed),
    )
    .into();
    let _: TestElement<'_> = snackbar::two_line("Longer message").into();
    let _: TestElement<'_> = snackbar::two_line_with_action(
        "Longer message",
        snackbar::icon_action_button("close", Message::Pressed),
    )
    .into();

    let content: TestElement<'_> = Text::new("Content").into();
    let _: TestElement<'_> = snackbar::host_single_line_with_action(
        content,
        &snackbar::Transition::default(),
        iced_widget::core::time::Instant::now(),
        "Archived",
        "Undo",
        Message::Pressed,
    );
}

#[test]
fn material_data_table_constructors_compile_to_elements() {
    #[derive(Clone)]
    struct Row {
        name: &'static str,
        quantity: u32,
    }

    let rows = [Row {
        name: "Assist",
        quantity: 24,
    }];
    let columns: [iced_widget::table::Column<'_, '_, Row, Message, Theme, iced_widget::Renderer>;
        2] = [
        data_table::weighted_column(2, "Name", |row: Row| row.name),
        data_table::compact_numeric_column("Quantity", |row: Row| row.quantity.to_string()),
    ];

    let _: TestElement<'_> = data_table::standard(columns, rows).into();
    assert_eq!(data_table::COMPACT_NUMERIC_COLUMN_WIDTH, 88.0);
}

#[test]
fn material_data_table_cells_use_token_heights() {
    let header: Container<'_, Message, Theme, iced_widget::Renderer> =
        data_table::header_cell("Name");
    let body: Container<'_, Message, Theme, iced_widget::Renderer> =
        data_table::body_cell("Assist");

    assert_eq!(
        Widget::<Message, Theme, iced_widget::Renderer>::size(&header),
        Size {
            width: Length::Shrink,
            height: Length::Fixed(tokens::component::data_table::HEADER_CONTAINER_HEIGHT),
        }
    );
    assert_eq!(
        Widget::<Message, Theme, iced_widget::Renderer>::size(&body),
        Size {
            width: Length::Shrink,
            height: Length::Fixed(tokens::component::data_table::ROW_ITEM_CONTAINER_HEIGHT),
        }
    );
}

#[test]
fn material_slider_and_progress_constructors_compile_to_elements() {
    let _: TestElement<'_> = slider::continuous(0.0..=100.0, 42.0, |_| Message::Pressed).into();
    let _: TestElement<'_> = progress_bar::linear(0.42, 0.0).into();
    let _: TestElement<'_> = progress_bar::linear_indeterminate(0.0, true).into();
    let _: TestElement<'_> = progress_bar::loading_indicator(0.0).into();
    let _: TestElement<'_> = progress_bar::contained_loading_indicator(0.0).into();
    let _: TestElement<'_> = progress_bar::determinate_loading_indicator(0.42).into();
    let _: TestElement<'_> = progress_bar::determinate_contained_loading_indicator(0.42).into();
}

#[test]
fn material_rule_constructors_compile_to_elements() {
    let _: TestElement<'_> = rule::horizontal_full_width().into();
    let _: TestElement<'_> = rule::horizontal_inset().into();
    let _: TestElement<'_> = rule::vertical_full_height().into();
    let _: TestElement<'_> = rule::vertical_inset().into();
}

#[test]
fn material_tooltip_constructor_compiles_to_element() {
    let content = button::assist_chip("Hint").on_press(Message::Pressed);
    let _: TestElement<'_> =
        tooltip::plain(content, "Material 3 plain tooltip", tooltip::Position::Top).into();

    let content = button::assist_chip("Rich").on_press(Message::Pressed);
    let _: TestElement<'_> =
        tooltip::rich(content, "Material 3 rich tooltip", tooltip::Position::Top).into();

    let content = button::assist_chip("Rich title").on_press(Message::Pressed);
    let _: TestElement<'_> = tooltip::rich_with_title(
        content,
        "Rich tooltip",
        "Supporting text",
        tooltip::Position::Top,
    )
    .into();

    let content = button::assist_chip("Rich action").on_press(Message::Pressed);
    let action = tooltip::rich_action_button("Action", Message::Pressed);
    let _: TestElement<'_> = tooltip::rich_with_title_action(
        content,
        "Rich tooltip",
        "Supporting text",
        action,
        tooltip::Position::Top,
    )
    .into();
}

#[test]
fn material_selection_constructors_compile_to_elements() {
    let _: TestElement<'_> = checkbox::standard(true, "Enable actions", toggled);
    let _: TestElement<'_> = toggler::standard(true, "Dark theme", toggled);
}

#[test]
fn material_text_input_constructor_compiles_to_element() {
    let _: TestElement<'_> = text_input::outlined("Write a note", "value")
        .on_input(|_| Message::Pressed)
        .into();
}

#[test]
fn material_text_editor_constructor_compiles_to_element() {
    let content = text_editor::Content::with_text("value");
    let _: TestElement<'_> = text_editor::outlined(&content)
        .placeholder("Write a note")
        .on_action(|_| Message::Pressed)
        .into();
    let _: TestElement<'_> = text_editor::outlined_area(&content)
        .placeholder("Write details")
        .on_action(|_| Message::Pressed)
        .into();
    assert_eq!(
        text_editor::OUTLINED_AREA_HEIGHT,
        tokens::component::text_field::CONTAINER_HEIGHT * 2.0
    );
}

#[test]
fn checkbox_checkmark_svg_uses_m3_rect_mark_geometry() {
    let svg = String::from_utf8(checkbox_checkmark_svg(1.0)).expect("valid svg");

    assert!(svg.contains("viewBox=\"0 0 18 18\""));
    assert!(svg.contains("scale(1 -1) translate(7 -14) rotate(45)"));
    assert!(svg.contains("width=\"2\" height=\"5.656854\""));
    assert!(svg.contains("width=\"11.313708\" height=\"2\""));
}
