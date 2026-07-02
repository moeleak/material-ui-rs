use iced::time::{Duration, Instant};
use iced::widget::{column, row, scrollable, text};
use iced::{Alignment, Color, Element, Length, Size, Subscription, window};
use iced_material as material;
use material::{ColorQuartet, ColorScheme, Inverse, Outline, Surface, SurfaceContainer, Theme};

fn type_scale_line_height(
    scale: material::tokens::typography::TypeScale,
) -> iced::widget::text::LineHeight {
    iced::widget::text::LineHeight::Absolute(scale.line_height.into())
}

pub fn main() -> iced::Result {
    let window_size = Size::new(1080.0, 980.0);

    iced::application(Demo::default, update, view)
        .title("iced_material example")
        .font(material::fonts::ROBOTO_REGULAR_BYTES)
        .font(material::fonts::ROBOTO_MEDIUM_BYTES)
        .font(material::fonts::ROBOTO_BOLD_BYTES)
        .font(material::fonts::NOTO_SANS_CJK_SC_REGULAR_BYTES)
        .font(material::fonts::NOTO_SANS_CJK_SC_MEDIUM_BYTES)
        .font(material::fonts::NOTO_SANS_CJK_SC_BOLD_BYTES)
        .font(material::fonts::MATERIAL_SYMBOLS_ROUNDED_BYTES)
        .default_font(material::fonts::ROBOTO)
        .subscription(subscription)
        .theme(theme)
        .window(window::Settings {
            size: window_size,
            min_size: Some(Size::new(420.0, 720.0)),
            position: window::Position::Centered,
            ..window::Settings::default()
        })
        .run()
}

#[derive(Debug, Clone)]
enum Message {
    Navigate(DemoPage),
    Increment,
    Decrement,
    TextChanged(String),
    EditorAction(material::widget::text_editor::Action),
    SelectChanged(&'static str),
    ComboSelected(&'static str),
    ComboInputChanged(String),
    SliderChanged(f32),
    EnabledChanged(bool),
    DarkModeChanged(bool),
    ChoiceSelected(RadioChoice),
    ToggleDrawer,
    WindowResized(Size),
    Frame(Instant),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DemoPage {
    Inputs,
    Controls,
    Feedback,
    Surfaces,
    Navigation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RadioChoice {
    Standard,
    Expressive,
    Dense,
}

#[derive(Debug, Clone, Copy)]
struct InventoryRow {
    component: &'static str,
    status: &'static str,
    count: u32,
}

const NAV_DESTINATIONS: [material::widget::navigation::Destination<DemoPage>; 5] = [
    material::widget::navigation::Destination::new(DemoPage::Inputs, "input", "Inputs"),
    material::widget::navigation::Destination::new(DemoPage::Controls, "tune", "Controls"),
    material::widget::navigation::Destination::new(DemoPage::Feedback, "info", "Feedback")
        .badge("3"),
    material::widget::navigation::Destination::new(DemoPage::Surfaces, "layers", "Surfaces")
        .small_badge(),
    material::widget::navigation::Destination::new(
        DemoPage::Navigation,
        "navigation",
        "Navigation",
    ),
];

const INVENTORY_ROWS: [InventoryRow; 3] = [
    InventoryRow {
        component: "Buttons",
        status: "Enabled",
        count: 4,
    },
    InventoryRow {
        component: "Selection",
        status: "Animated",
        count: 3,
    },
    InventoryRow {
        component: "Inputs",
        status: "Focused",
        count: 5,
    },
];

#[derive(Debug)]
struct Demo {
    navigation: material::widget::navigation::NavigationState<DemoPage>,
    rail_expansion: material::widget::navigation::NavigationRailExpansionState,
    window_size: Size,
    count: i32,
    note: String,
    editor_content: material::widget::text_editor::Content,
    select_choice: Option<&'static str>,
    combo_options: material::widget::combo_box::State<&'static str>,
    combo_choice: Option<&'static str>,
    combo_input: String,
    progress: f32,
    enabled: bool,
    dark_mode: bool,
    radio_choice: Option<RadioChoice>,
    visible_scheme: ColorScheme,
    animation: Option<ThemeAnimation>,
}

#[derive(Debug, Clone, Copy)]
struct ThemeAnimation {
    from: ColorScheme,
    to: ColorScheme,
    started_at: Instant,
}

impl Default for Demo {
    fn default() -> Self {
        let initial_theme = Theme::Dark;

        Self {
            navigation: material::widget::navigation::NavigationState::new(DemoPage::Inputs),
            rail_expansion: material::widget::navigation::NavigationRailExpansionState::new(false),
            window_size: Size::new(1080.0, 980.0),
            count: 0,
            note: String::new(),
            editor_content: material::widget::text_editor::Content::with_text(
                "Material 3 multi-line text editor",
            ),
            select_choice: Some("Assist"),
            combo_options: material::widget::combo_box::State::with_selection(
                vec!["Assist", "Suggestion", "Filter"],
                Some(&"Suggestion"),
            ),
            combo_choice: Some("Suggestion"),
            combo_input: String::new(),
            progress: 42.0,
            enabled: true,
            dark_mode: true,
            radio_choice: Some(RadioChoice::Standard),
            visible_scheme: initial_theme.colors(),
            animation: None,
        }
    }
}

impl Demo {
    fn theme(&self) -> Theme {
        Theme::new("Material 3 animated", self.visible_scheme)
    }

    fn navigation_selection(&self) -> material::widget::navigation::Selection<DemoPage> {
        self.navigation.selection()
    }

    fn adaptive_navigation_layout(&self) -> material::widget::navigation::AdaptiveLayout {
        material::widget::navigation::adaptive_layout(
            self.window_size.width,
            self.window_size.height,
        )
    }
}

fn update(state: &mut Demo, message: Message) {
    match message {
        Message::Navigate(page) => {
            state
                .navigation
                .select(page, Instant::now(), state.adaptive_navigation_layout());
        }
        Message::Increment => state.count += 1,
        Message::Decrement => state.count -= 1,
        Message::TextChanged(note) => state.note = note,
        Message::EditorAction(action) => state.editor_content.perform(action),
        Message::SelectChanged(choice) => state.select_choice = Some(choice),
        Message::ComboSelected(choice) => {
            state.combo_choice = Some(choice);
            state.combo_input.clear();
            state.combo_options.set_selection(Some(&choice));
        }
        Message::ComboInputChanged(input) => {
            state.combo_options.set_input(input.clone());
            state.combo_input = input;
            state.combo_choice = None;
        }
        Message::SliderChanged(progress) => state.progress = progress,
        Message::EnabledChanged(enabled) => state.enabled = enabled,
        Message::ChoiceSelected(choice) => state.radio_choice = Some(choice),
        Message::ToggleDrawer => state.rail_expansion.toggle(Instant::now()),
        Message::WindowResized(size) => state.window_size = size,
        Message::DarkModeChanged(dark_mode) => {
            state.dark_mode = dark_mode;

            let target = if dark_mode {
                Theme::Dark.colors()
            } else {
                Theme::Light.colors()
            };

            state.animation = Some(ThemeAnimation {
                from: state.visible_scheme,
                to: target,
                started_at: Instant::now(),
            });
        }
        Message::Frame(now) => {
            if let Some(animation) = state.animation {
                let duration =
                    Duration::from_millis(u64::from(material::tokens::motion::DURATION_MEDIUM4_MS));
                let progress = now
                    .saturating_duration_since(animation.started_at)
                    .as_secs_f32()
                    / duration.as_secs_f32();

                if progress >= 1.0 {
                    state.visible_scheme = animation.to;
                    state.animation = None;
                } else {
                    state.visible_scheme = lerp_color_scheme(
                        animation.from,
                        animation.to,
                        emphasized_decelerate(progress),
                    );
                }
            }

            let _ = state.navigation.advance(now);
            let _ = state.rail_expansion.advance(now);
        }
    }
}

fn theme(state: &Demo) -> Theme {
    state.theme()
}

fn subscription(state: &Demo) -> Subscription<Message> {
    let mut subscriptions =
        vec![iced::window::resize_events().map(|(_id, size)| Message::WindowResized(size))];

    if state.animation.is_some()
        || state.navigation.is_animating()
        || state.rail_expansion.is_animating()
    {
        subscriptions.push(iced::window::frames().map(Message::Frame));
    }

    Subscription::batch(subscriptions)
}

fn view(state: &Demo) -> Element<'_, Message, Theme> {
    let selection = state.navigation_selection();

    match state.adaptive_navigation_layout() {
        material::widget::navigation::AdaptiveLayout::NavigationBar => column![
            page_content(state),
            material::widget::navigation::navigation_bar(
                &NAV_DESTINATIONS,
                selection,
                Message::Navigate,
            )
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .into(),
        material::widget::navigation::AdaptiveLayout::NavigationRail => {
            let navigation: Element<'_, Message, Theme> = if state.rail_expansion.is_visible() {
                let drawer_width =
                    material::widget::navigation::navigation_rail_expanded_width_for_progress(
                        state.rail_expansion.progress(),
                    );

                if drawer_width > 0.0 {
                    material::widget::navigation::navigation_rail_expanded_with_menu_at_width(
                        "Example",
                        &NAV_DESTINATIONS,
                        selection,
                        Message::Navigate,
                        Message::ToggleDrawer,
                        drawer_width,
                    )
                    .into()
                } else {
                    material::widget::navigation::navigation_rail_with_menu(
                        &NAV_DESTINATIONS,
                        selection,
                        Message::Navigate,
                        Message::ToggleDrawer,
                    )
                    .into()
                }
            } else {
                material::widget::navigation::navigation_rail_with_menu(
                    &NAV_DESTINATIONS,
                    selection,
                    Message::Navigate,
                    Message::ToggleDrawer,
                )
                .into()
            };

            row![navigation, page_content(state)]
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        }
    }
}

fn page_content(state: &Demo) -> Element<'_, Message, Theme> {
    let page = state.navigation.selected();
    let content = match page {
        DemoPage::Inputs => inputs_page(state),
        DemoPage::Controls => controls_page(state),
        DemoPage::Feedback => feedback_page(state),
        DemoPage::Surfaces => surfaces_page(state),
        DemoPage::Navigation => navigation_page(state),
    };

    let page = column![header(page), content]
        .spacing(28)
        .padding(28)
        .width(Length::Fill)
        .max_width(980);

    scrollable(
        material::widget::container::surface_container_high(page)
            .width(Length::Fill)
            .center_x(Length::Fill),
    )
    .height(Length::Fill)
    .into()
}

fn header(page: DemoPage) -> Element<'static, Message, Theme> {
    let body_large = material::tokens::typography::BODY_LARGE;
    let headline_large = material::tokens::typography::HEADLINE_LARGE;
    let chinese_sample = "中文字体 Noto Sans CJK";

    column![
        text("iced_material 0.14.2")
            .size(headline_large.size)
            .line_height(type_scale_line_height(headline_large)),
        text(page_label(page))
            .size(body_large.size)
            .line_height(type_scale_line_height(body_large)),
        text(chinese_sample)
            .font(material::fonts::font_for_content_type_scale(
                chinese_sample,
                body_large,
            ))
            .size(body_large.size)
            .line_height(type_scale_line_height(body_large)),
    ]
    .spacing(6)
    .into()
}

fn inputs_page(state: &Demo) -> Element<'_, Message, Theme> {
    let input = material::widget::text_input::outlined("Write a note", &state.note)
        .on_input(Message::TextChanged);

    let editor = material::widget::text_editor::outlined(&state.editor_content)
        .placeholder("Write details")
        .on_action(Message::EditorAction)
        .height(Length::Fixed(112.0));

    let select_options = ["Assist", "Suggestion", "Filter"];
    let select = material::widget::pick_list::outlined(
        select_options,
        state.select_choice,
        Message::SelectChanged,
    )
    .placeholder("Choose a chip")
    .width(Length::Fill);

    let combo_box = material::widget::combo_box::outlined_with_input(
        &state.combo_options,
        "Search a chip",
        &state.combo_input,
        state.combo_choice.as_ref(),
        Message::ComboSelected,
    )
    .on_input(Message::ComboInputChanged);

    column![
        section("Text fields", column![input, editor].spacing(16).into()),
        material::widget::rule::horizontal_inset(),
        section(
            "Selection fields",
            column![select, combo_box].spacing(16).into()
        ),
        material::widget::rule::horizontal_inset(),
        section("Dividers", dividers()),
    ]
    .spacing(24)
    .width(Length::Fill)
    .into()
}

fn controls_page(state: &Demo) -> Element<'_, Message, Theme> {
    column![
        section("Counter", counter_controls(state)),
        material::widget::rule::horizontal_inset(),
        section("Actions", action_buttons(state)),
        material::widget::rule::horizontal_inset(),
        section("Chips", chips(state)),
        material::widget::rule::horizontal_inset(),
        section("Selection controls", selection_controls(state)),
    ]
    .spacing(24)
    .width(Length::Fill)
    .into()
}

fn feedback_page(state: &Demo) -> Element<'_, Message, Theme> {
    column![
        section("Progress", progress_indicators(state)),
        material::widget::rule::horizontal_inset(),
        section("Badges", badges()),
        material::widget::rule::horizontal_inset(),
        section(
            "Tooltip",
            material::widget::tooltip::plain(
                material::widget::button::assist_chip("Hint")
                    .on_press_maybe(state.enabled.then_some(Message::Increment)),
                "Material 3 plain tooltip",
                material::widget::tooltip::Position::Top,
            )
            .into(),
        ),
    ]
    .spacing(24)
    .width(Length::Fill)
    .into()
}

fn surfaces_page(_state: &Demo) -> Element<'static, Message, Theme> {
    column![
        section("Cards", cards()),
        material::widget::rule::horizontal_inset(),
        section("Lists", lists()),
        material::widget::rule::horizontal_inset(),
        section("Data table", data_table()),
    ]
    .spacing(24)
    .width(Length::Fill)
    .into()
}

fn navigation_page(state: &Demo) -> Element<'_, Message, Theme> {
    let selection = state.navigation_selection();
    let bar = material::widget::navigation::navigation_bar(
        &NAV_DESTINATIONS,
        selection,
        Message::Navigate,
    );
    let rail = material::widget::navigation::navigation_rail_with_menu(
        &NAV_DESTINATIONS,
        selection,
        Message::Navigate,
        Message::ToggleDrawer,
    )
    .height(Length::Fixed(360.0));
    let expanded_rail = material::widget::navigation::navigation_rail_expanded_with_menu(
        "Example",
        &NAV_DESTINATIONS,
        selection,
        Message::Navigate,
        Message::ToggleDrawer,
    )
    .height(Length::Fixed(360.0));

    column![
        section("Navigation bar", bar.into()),
        material::widget::rule::horizontal_inset(),
        section("Navigation rail with menu", rail.into()),
        material::widget::rule::horizontal_inset(),
        section("Expanded navigation rail", expanded_rail.into()),
    ]
    .spacing(24)
    .width(Length::Fill)
    .into()
}

fn section<'a>(
    title: &'static str,
    body: Element<'a, Message, Theme>,
) -> Element<'a, Message, Theme> {
    let title_medium = material::tokens::typography::TITLE_MEDIUM;

    column![
        text(title)
            .size(title_medium.size)
            .line_height(type_scale_line_height(title_medium)),
        body
    ]
    .spacing(12)
    .width(Length::Fill)
    .into()
}

fn counter_controls(state: &Demo) -> Element<'_, Message, Theme> {
    let headline_medium = material::tokens::typography::HEADLINE_MEDIUM;

    row![
        material::widget::button::outlined("Minus").on_press(Message::Decrement),
        text(state.count)
            .size(headline_medium.size)
            .line_height(type_scale_line_height(headline_medium)),
        material::widget::button::filled("Plus").on_press(Message::Increment),
    ]
    .spacing(12)
    .align_y(Alignment::Center)
    .into()
}

fn action_buttons(state: &Demo) -> Element<'_, Message, Theme> {
    row![
        material::widget::button::filled("Filled")
            .on_press_maybe(state.enabled.then_some(Message::Increment)),
        material::widget::button::filled_tonal("Tonal")
            .on_press_maybe(state.enabled.then_some(Message::Increment)),
        material::widget::button::text("Text")
            .on_press_maybe(state.enabled.then_some(Message::Increment)),
        material::widget::button::primary_fab("+")
            .on_press_maybe(state.enabled.then_some(Message::Increment)),
    ]
    .spacing(12)
    .align_y(Alignment::Center)
    .into()
}

fn chips(state: &Demo) -> Element<'_, Message, Theme> {
    row![
        material::widget::button::assist_chip("Assist")
            .on_press_maybe(state.enabled.then_some(Message::Increment)),
        material::widget::button::suggestion_chip("Suggestion")
            .on_press_maybe(state.enabled.then_some(Message::Increment)),
        material::widget::button::filter_chip("Filter")
            .on_press_maybe(state.enabled.then_some(Message::Increment)),
        material::widget::button::selected_filter_chip("Selected")
            .on_press_maybe(state.enabled.then_some(Message::Increment)),
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .into()
}

fn selection_controls(state: &Demo) -> Element<'_, Message, Theme> {
    let switches = column![
        material::widget::checkbox::standard(
            state.enabled,
            "Enable actions",
            Message::EnabledChanged,
        ),
        material::widget::toggler::standard(
            state.dark_mode,
            "Dark theme",
            Message::DarkModeChanged,
        ),
    ]
    .spacing(12);

    let radios = row![
        material::widget::radio::standard(
            "Standard",
            RadioChoice::Standard,
            state.radio_choice,
            Message::ChoiceSelected,
        ),
        material::widget::radio::standard(
            "Expressive",
            RadioChoice::Expressive,
            state.radio_choice,
            Message::ChoiceSelected,
        ),
        material::widget::radio::standard(
            "Dense",
            RadioChoice::Dense,
            state.radio_choice,
            Message::ChoiceSelected,
        ),
    ]
    .spacing(12)
    .align_y(Alignment::Center);

    column![switches, radios].spacing(18).into()
}

fn progress_indicators(state: &Demo) -> Element<'_, Message, Theme> {
    let body_large = material::tokens::typography::BODY_LARGE;

    column![
        row![
            text("Progress")
                .size(body_large.size)
                .line_height(type_scale_line_height(body_large))
                .width(Length::Fill),
            text(format!("{:.0}%", state.progress))
                .size(body_large.size)
                .line_height(type_scale_line_height(body_large)),
        ]
        .spacing(12),
        material::widget::slider::continuous(0.0..=100.0, state.progress, Message::SliderChanged)
            .step(1.0),
        material::widget::progress_bar::linear(0.0..=100.0, state.progress),
    ]
    .spacing(10)
    .into()
}

fn badges() -> Element<'static, Message, Theme> {
    let body_large = material::tokens::typography::BODY_LARGE;

    row![
        text("Badges")
            .size(body_large.size)
            .line_height(type_scale_line_height(body_large)),
        material::widget::badge::small(),
        material::widget::badge::large("3"),
        material::widget::badge::large("99+"),
    ]
    .spacing(12)
    .align_y(Alignment::Center)
    .into()
}

fn cards() -> Element<'static, Message, Theme> {
    let body_medium = material::tokens::typography::BODY_MEDIUM;
    let title_medium = material::tokens::typography::TITLE_MEDIUM;

    let elevated_card = material::widget::card::elevated(
        column![
            text("Elevated")
                .size(title_medium.size)
                .line_height(type_scale_line_height(title_medium)),
            text("Level 1")
                .size(body_medium.size)
                .line_height(type_scale_line_height(body_medium)),
        ]
        .spacing(2),
    )
    .padding(12)
    .height(Length::Fixed(78.0))
    .width(Length::Fill);

    let filled_card = material::widget::card::filled(
        column![
            text("Filled")
                .size(title_medium.size)
                .line_height(type_scale_line_height(title_medium)),
            text("Container")
                .size(body_medium.size)
                .line_height(type_scale_line_height(body_medium)),
        ]
        .spacing(2),
    )
    .padding(12)
    .height(Length::Fixed(78.0))
    .width(Length::Fill);

    let outlined_card = material::widget::card::outlined(
        column![
            text("Outlined")
                .size(title_medium.size)
                .line_height(type_scale_line_height(title_medium)),
            text("1px stroke")
                .size(body_medium.size)
                .line_height(type_scale_line_height(body_medium)),
        ]
        .spacing(2),
    )
    .padding(12)
    .height(Length::Fixed(78.0))
    .width(Length::Fill);

    column![elevated_card, filled_card, outlined_card]
        .spacing(8)
        .width(Length::Fill)
        .into()
}

fn lists() -> Element<'static, Message, Theme> {
    column![
        material::widget::list::one_line_with_leading_icon("*", "One-line list item"),
        material::widget::list::two_line_with_trailing("Messages", "Supporting text", "24"),
        material::widget::list::three_line("Three-line item", "Supporting text", "Second line"),
    ]
    .spacing(0)
    .width(Length::Fill)
    .into()
}

fn dividers() -> Element<'static, Message, Theme> {
    let body_large = material::tokens::typography::BODY_LARGE;

    column![
        material::widget::rule::horizontal_full_width(),
        row![
            text("Full")
                .size(body_large.size)
                .line_height(type_scale_line_height(body_large)),
            material::widget::rule::vertical_full_height(),
            text("Inset")
                .size(body_large.size)
                .line_height(type_scale_line_height(body_large)),
        ]
        .height(Length::Fixed(32.0))
        .spacing(16)
        .align_y(Alignment::Center),
        material::widget::rule::horizontal_inset(),
    ]
    .spacing(8)
    .into()
}

fn data_table() -> Element<'static, Message, Theme> {
    material::widget::data_table::standard(
        [
            material::widget::data_table::column("Component", |row: InventoryRow| row.component)
                .width(Length::FillPortion(2)),
            material::widget::data_table::column("State", |row: InventoryRow| row.status),
            material::widget::data_table::numeric_column("Count", |row: InventoryRow| {
                row.count.to_string()
            })
            .width(Length::Fixed(88.0)),
        ],
        INVENTORY_ROWS,
    )
    .width(Length::Fill)
    .into()
}

fn page_label(page: DemoPage) -> &'static str {
    match page {
        DemoPage::Inputs => "Inputs",
        DemoPage::Controls => "Controls",
        DemoPage::Feedback => "Feedback",
        DemoPage::Surfaces => "Surfaces",
        DemoPage::Navigation => "Navigation",
    }
}

fn emphasized_decelerate(progress: f32) -> f32 {
    if progress <= 0.0 {
        return 0.0;
    }

    if progress >= 1.0 {
        return 1.0;
    }

    let easing = material::tokens::motion::EASING_EMPHASIZED_DECELERATE;

    cubic_bezier(progress, easing.x1, easing.y1, easing.x2, easing.y2)
}

fn cubic_bezier(progress: f32, x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    let progress = progress.clamp(0.0, 1.0);
    let mut low = 0.0;
    let mut high = 1.0;

    for _ in 0..20 {
        let mid = (low + high) * 0.5;

        if bezier_axis(mid, x1, x2) < progress {
            low = mid;
        } else {
            high = mid;
        }
    }

    bezier_axis((low + high) * 0.5, y1, y2).clamp(0.0, 1.0)
}

fn bezier_axis(t: f32, p1: f32, p2: f32) -> f32 {
    let inverse = 1.0 - t;

    3.0 * inverse * inverse * t * p1 + 3.0 * inverse * t * t * p2 + t * t * t
}

fn lerp_color_scheme(from: ColorScheme, to: ColorScheme, amount: f32) -> ColorScheme {
    ColorScheme {
        primary: lerp_color_quartet(from.primary, to.primary, amount),
        secondary: lerp_color_quartet(from.secondary, to.secondary, amount),
        tertiary: lerp_color_quartet(from.tertiary, to.tertiary, amount),
        error: lerp_color_quartet(from.error, to.error, amount),
        surface: lerp_surface(from.surface, to.surface, amount),
        inverse: lerp_inverse(from.inverse, to.inverse, amount),
        outline: lerp_outline(from.outline, to.outline, amount),
        shadow: lerp_color(from.shadow, to.shadow, amount),
        scrim: lerp_color(from.scrim, to.scrim, amount),
    }
}

fn lerp_color_quartet(from: ColorQuartet, to: ColorQuartet, amount: f32) -> ColorQuartet {
    ColorQuartet {
        color: lerp_color(from.color, to.color, amount),
        text: lerp_color(from.text, to.text, amount),
        container: lerp_color(from.container, to.container, amount),
        container_text: lerp_color(from.container_text, to.container_text, amount),
    }
}

fn lerp_surface(from: Surface, to: Surface, amount: f32) -> Surface {
    Surface {
        color: lerp_color(from.color, to.color, amount),
        text: lerp_color(from.text, to.text, amount),
        text_variant: lerp_color(from.text_variant, to.text_variant, amount),
        container: lerp_surface_container(from.container, to.container, amount),
    }
}

fn lerp_surface_container(
    from: SurfaceContainer,
    to: SurfaceContainer,
    amount: f32,
) -> SurfaceContainer {
    SurfaceContainer {
        lowest: lerp_color(from.lowest, to.lowest, amount),
        low: lerp_color(from.low, to.low, amount),
        base: lerp_color(from.base, to.base, amount),
        high: lerp_color(from.high, to.high, amount),
        highest: lerp_color(from.highest, to.highest, amount),
    }
}

fn lerp_inverse(from: Inverse, to: Inverse, amount: f32) -> Inverse {
    Inverse {
        inverse_surface: lerp_color(from.inverse_surface, to.inverse_surface, amount),
        inverse_surface_text: lerp_color(
            from.inverse_surface_text,
            to.inverse_surface_text,
            amount,
        ),
        inverse_primary: lerp_color(from.inverse_primary, to.inverse_primary, amount),
    }
}

fn lerp_outline(from: Outline, to: Outline, amount: f32) -> Outline {
    Outline {
        color: lerp_color(from.color, to.color, amount),
        variant: lerp_color(from.variant, to.variant, amount),
    }
}

fn lerp_color(from: Color, to: Color, amount: f32) -> Color {
    let amount = amount.clamp(0.0, 1.0);

    Color {
        r: lerp_component(from.r, to.r, amount),
        g: lerp_component(from.g, to.g, amount),
        b: lerp_component(from.b, to.b, amount),
        a: lerp_component(from.a, to.a, amount),
    }
}

fn lerp_component(from: f32, to: f32, amount: f32) -> f32 {
    from + (to - from) * amount
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emphasized_decelerate_has_expected_endpoints() {
        assert_eq!(emphasized_decelerate(0.0), 0.0);
        assert!((emphasized_decelerate(1.0) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn theme_lerp_reaches_target_color_scheme() {
        let target = Theme::Light.colors();

        assert_eq!(
            lerp_color_scheme(Theme::Dark.colors(), target, 1.0)
                .surface
                .color,
            target.surface.color
        );
    }

    #[test]
    fn combo_input_preserves_typed_query_and_clears_stale_selection() {
        let mut demo = Demo::default();

        update(&mut demo, Message::ComboInputChanged("xxx".into()));

        assert_eq!(demo.combo_choice, None);
        assert_eq!(demo.combo_input, "xxx");

        update(&mut demo, Message::ComboSelected("Assist"));

        assert_eq!(demo.combo_choice, Some("Assist"));
        assert_eq!(demo.combo_input, "");
    }

    #[test]
    fn navigation_starts_selection_animation() {
        let mut demo = Demo::default();

        update(&mut demo, Message::Navigate(DemoPage::Controls));

        assert_eq!(demo.navigation.selected(), DemoPage::Controls);
        assert!(demo.navigation.is_animating());
        assert_eq!(
            demo.navigation.selection().progress(DemoPage::Controls),
            0.0
        );
        assert_eq!(demo.navigation.selection().progress(DemoPage::Inputs), 1.0);
    }

    #[test]
    fn menu_toggles_expanded_navigation_rail() {
        let mut demo = Demo::default();

        update(&mut demo, Message::ToggleDrawer);

        assert!(demo.rail_expansion.is_open());
        assert!(demo.rail_expansion.is_animating());

        update(
            &mut demo,
            Message::Frame(Instant::now() + Duration::from_millis(500)),
        );

        assert!(demo.rail_expansion.is_visible());

        update(&mut demo, Message::ToggleDrawer);

        assert!(!demo.rail_expansion.is_open());
    }

    #[test]
    fn navigation_uses_material_symbol_icon_names() {
        assert_eq!(material::fonts::all().len(), 7);
        assert_eq!(NAV_DESTINATIONS[0].icon, "input");
        assert_eq!(NAV_DESTINATIONS[1].icon, "tune");
        assert_eq!(NAV_DESTINATIONS[2].icon, "info");
        assert_eq!(NAV_DESTINATIONS[3].icon, "layers");
        assert_eq!(NAV_DESTINATIONS[4].icon, "navigation");
    }

    #[test]
    fn chinese_sample_uses_bundled_noto_sans_cjk() {
        assert_eq!(
            material::fonts::font_for_content_type_scale(
                "中文字体 Noto Sans CJK",
                material::tokens::typography::BODY_LARGE,
            ),
            material::fonts::NOTO_SANS_CJK_SC
        );
    }

    #[test]
    fn resize_updates_adaptive_layout_inputs() {
        let mut demo = Demo::default();

        update(&mut demo, Message::WindowResized(Size::new(500.0, 900.0)));

        assert_eq!(
            demo.adaptive_navigation_layout(),
            material::widget::navigation::AdaptiveLayout::NavigationBar
        );

        update(&mut demo, Message::WindowResized(Size::new(900.0, 900.0)));

        assert_eq!(
            demo.adaptive_navigation_layout(),
            material::widget::navigation::AdaptiveLayout::NavigationRail
        );
    }
}
