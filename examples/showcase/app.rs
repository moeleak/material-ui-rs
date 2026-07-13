#![cfg_attr(windows, windows_subsystem = "windows")]

#[path = "pages/mod.rs"]
mod pages;

use iced::time::Instant;
use iced::{Size, Subscription, Task};
use material::Theme;
use material::widget::{navigation, theme_picker};
use material_ui_rs as material;

pub fn main() -> iced::Result {
    let window_size = Size::new(1080.0, 980.0);

    material::application(boot, update, view)
        .title("material-ui-rs showcase")
        .subscription(subscription)
        .theme(theme)
        .window(material::window_with_min_size(
            window_size,
            Size::new(420.0, 720.0),
        ))
        .run()
}

#[cfg(any(target_arch = "wasm32", test))]
const CJK_CORE_FONT_URL: &str = "fonts/NotoSansSC-Core-0a7ff25a.otf";
#[cfg(any(target_arch = "wasm32", test))]
const CJK_REGIONAL_FONT_URL: &str = "fonts/NotoSansSC-faa6c9df.otf";

fn boot() -> (Showcase, Task<Message>) {
    let state = Showcase::default();

    #[cfg(any(target_arch = "wasm32", test))]
    let load_cjk_core =
        material::fonts::load_web_font(CJK_CORE_FONT_URL).map(|_| Message::CjkCoreFontFinished);
    #[cfg(not(any(target_arch = "wasm32", test)))]
    let load_cjk_core = Task::none();

    (state, load_cjk_core)
}

#[derive(Debug, Clone)]
enum Message {
    #[cfg(any(target_arch = "wasm32", test))]
    CjkCoreFontFinished,
    #[cfg(any(target_arch = "wasm32", test))]
    CjkRegionalFontFinished,
    Navigate(ShowcasePage),
    Increment,
    Decrement,
    TextChanged(String),
    EditorAction(material::widget::text_editor::Action),
    SelectChanged(&'static str),
    ComboboxSelected(&'static str),
    ComboboxInputChanged(String),
    SearchChanged(String),
    DatePickerChanged(material::widget::picker::DatePickerAction),
    DateRangePickerChanged(material::widget::picker::DateRangePickerAction),
    TimePickerChanged(material::widget::picker::TimePickerAction),
    SliderChanged(f32),
    EnabledChanged(bool),
    ThemeChanged(theme_picker::ThemeAction),
    ChoiceSelected(RadioChoice),
    SegmentSelected(SegmentChoice),
    PrimaryTabSelected(TabChoice),
    SecondaryTabSelected(TabChoice),
    LogViewer(material::widget::log_viewer::Action<u64>),
    MenuPressed,
    DialogOpened,
    DialogDismissed,
    DialogConfirmed,
    ShowSnackbar,
    SnackbarUndo,
    WindowResized(Size),
    Frame(Instant),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ShowcasePage {
    Inputs,
    Controls,
    Feedback,
    Surfaces,
    Navigation,
    Structure,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RadioChoice {
    Standard,
    Expressive,
    Dense,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SegmentChoice {
    List,
    Grid,
    Map,
}

impl SegmentChoice {
    const fn index(self) -> usize {
        match self {
            Self::List => 0,
            Self::Grid => 1,
            Self::Map => 2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TabChoice {
    Inputs,
    Controls,
    Feedback,
}

impl TabChoice {
    const fn index(self) -> usize {
        match self {
            Self::Inputs => 0,
            Self::Controls => 1,
            Self::Feedback => 2,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct InventoryRow {
    component: &'static str,
    status: &'static str,
    count: u32,
}

const NAV_DESTINATIONS: [navigation::Destination<ShowcasePage>; 6] = [
    navigation::Destination::new(ShowcasePage::Inputs, "input", "Inputs"),
    navigation::Destination::new(ShowcasePage::Controls, "tune", "Controls"),
    navigation::Destination::new(ShowcasePage::Feedback, "info", "Feedback").badge("3"),
    navigation::Destination::new(ShowcasePage::Surfaces, "layers", "Surfaces").small_badge(),
    navigation::Destination::new(ShowcasePage::Navigation, "navigation", "Navigation"),
    navigation::Destination::new(ShowcasePage::Structure, "layers", "Structure"),
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
struct Showcase {
    navigation: navigation::NavigationState<ShowcasePage>,
    window_size: Size,
    count: i32,
    note: String,
    editor_content: material::widget::text_editor::Content,
    select_choice: Option<&'static str>,
    combobox_options: material::widget::combobox::State<&'static str>,
    combobox_choice: Option<&'static str>,
    combobox_input: String,
    search_query: String,
    date_picker: material::widget::picker::DatePickerState,
    date_range_picker: material::widget::picker::DateRangePickerState,
    time_picker: material::widget::picker::TimePickerState,
    progress: f32,
    enabled: bool,
    radio_choice: Option<RadioChoice>,
    segment_choice: SegmentChoice,
    segment_state: material::widget::segmented_button::State,
    primary_tab: TabChoice,
    primary_tab_state: material::widget::tabs::State,
    secondary_tab: TabChoice,
    secondary_tab_state: material::widget::tabs::State,
    log_viewer: material::widget::log_viewer::State<u64>,
    log_entries: Vec<material::widget::log_viewer::LogEntry<u64>>,
    progress_animation: material::widget::progress_bar::IndeterminateState,
    alert_dialog: material::widget::dialog::Transition,
    snackbar: material::widget::snackbar::Transition,
    theme_controller: theme_picker::ThemeController,
}

impl Default for Showcase {
    fn default() -> Self {
        Self {
            navigation: navigation::NavigationState::new(ShowcasePage::Inputs),
            window_size: Size::new(1080.0, 980.0),
            count: 0,
            note: String::new(),
            editor_content: material::widget::text_editor::Content::with_text(
                "Material 3 multi-line text editor",
            ),
            select_choice: Some("Assist"),
            combobox_options: material::widget::combobox::State::with_selection(
                vec!["Assist", "Suggestion", "Filter"],
                Some(&"Suggestion"),
            ),
            combobox_choice: Some("Suggestion"),
            combobox_input: String::new(),
            search_query: String::new(),
            date_picker: material::widget::picker::DatePickerState::new(
                material::widget::picker::Date::new(2026, 7, 4),
            ),
            date_range_picker: material::widget::picker::DateRangePickerState::new(
                material::widget::picker::Date::new(2026, 7, 4),
                material::widget::picker::Date::new(2026, 7, 10),
            ),
            time_picker: material::widget::picker::TimePickerState::new(14, 30, false),
            progress: 42.0,
            enabled: true,
            radio_choice: Some(RadioChoice::Standard),
            segment_choice: SegmentChoice::List,
            segment_state: material::widget::segmented_button::State::new(
                SegmentChoice::List.index(),
            ),
            primary_tab: TabChoice::Inputs,
            primary_tab_state: material::widget::tabs::State::new(TabChoice::Inputs.index()),
            secondary_tab: TabChoice::Controls,
            secondary_tab_state: material::widget::tabs::State::new(TabChoice::Controls.index()),
            log_viewer: material::widget::log_viewer::State::new(),
            log_entries: sample_log_entries(),
            progress_animation: material::widget::progress_bar::IndeterminateState::new(
                Instant::now(),
            ),
            alert_dialog: material::widget::dialog::Transition::default(),
            snackbar: material::widget::snackbar::Transition::default(),
            theme_controller: theme_picker::ThemeController::default(),
        }
    }
}

impl Showcase {
    fn theme(&self) -> Theme {
        self.theme_controller.theme("Material 3 animated")
    }

    fn navigation_selection(&self) -> navigation::Selection<ShowcasePage> {
        self.navigation.selection()
    }

    fn adaptive_navigation_layout(&self) -> navigation::AdaptiveLayout {
        navigation::adaptive_layout(self.window_size.width, self.window_size.height)
    }
}

fn update(state: &mut Showcase, message: Message) -> Task<Message> {
    match message {
        #[cfg(any(target_arch = "wasm32", test))]
        Message::CjkCoreFontFinished => load_cjk_regional_font(),
        #[cfg(any(target_arch = "wasm32", test))]
        Message::CjkRegionalFontFinished => Task::none(),
        Message::Navigate(page) => {
            state
                .navigation
                .select(page, Instant::now(), state.adaptive_navigation_layout());
            Task::none()
        }
        Message::Increment => {
            state.count += 1;
            Task::none()
        }
        Message::Decrement => {
            state.count -= 1;
            Task::none()
        }
        Message::TextChanged(note) => {
            state.note = note;
            Task::none()
        }
        Message::EditorAction(action) => {
            state.editor_content.perform(action);
            Task::none()
        }
        Message::SelectChanged(choice) => {
            state.select_choice = Some(choice);
            Task::none()
        }
        Message::ComboboxSelected(choice) => {
            state.combobox_choice = Some(choice);
            state.combobox_input.clear();
            state.combobox_options.set_selection(Some(&choice));
            Task::none()
        }
        Message::ComboboxInputChanged(input) => {
            state.combobox_options.set_input(input.clone());
            state.combobox_input = input;
            state.combobox_choice = None;
            Task::none()
        }
        Message::SearchChanged(query) => {
            state.search_query = query;
            Task::none()
        }
        Message::DatePickerChanged(action) => state.date_picker.update_and_scroll(action),
        Message::DateRangePickerChanged(action) => {
            state.date_range_picker.update_and_scroll(action)
        }
        Message::TimePickerChanged(action) => {
            state.time_picker.update(action);
            Task::none()
        }
        Message::SliderChanged(progress) => {
            state.progress = progress;
            Task::none()
        }
        Message::EnabledChanged(enabled) => {
            state.enabled = enabled;
            Task::none()
        }
        Message::ChoiceSelected(choice) => {
            state.radio_choice = Some(choice);
            Task::none()
        }
        Message::SegmentSelected(choice) => {
            state.segment_choice = choice;
            state.segment_state.select(choice.index(), Instant::now());
            Task::none()
        }
        Message::PrimaryTabSelected(choice) => {
            state.primary_tab = choice;
            state.primary_tab_state.select(
                choice.index(),
                Instant::now(),
                material::widget::tabs::Variant::Primary,
            );
            Task::none()
        }
        Message::SecondaryTabSelected(choice) => {
            state.secondary_tab = choice;
            state.secondary_tab_state.select(
                choice.index(),
                Instant::now(),
                material::widget::tabs::Variant::Secondary,
            );
            Task::none()
        }
        Message::LogViewer(action) => state.log_viewer.update(action, &state.log_entries),
        Message::MenuPressed => {
            state.navigation.toggle_menu_now();
            Task::none()
        }
        Message::DialogOpened => {
            state.alert_dialog.show(Instant::now());
            Task::none()
        }
        Message::DialogDismissed => {
            state.alert_dialog.dismiss(Instant::now());
            Task::none()
        }
        Message::DialogConfirmed => {
            state.alert_dialog.dismiss(Instant::now());
            state.count += 1;
            Task::none()
        }
        Message::ShowSnackbar => {
            state.snackbar.show(Instant::now());
            Task::none()
        }
        Message::SnackbarUndo => {
            state.count -= 1;
            state.snackbar.dismiss(Instant::now());
            Task::none()
        }
        Message::WindowResized(size) => {
            state.window_size = size;
            Task::none()
        }
        Message::ThemeChanged(action) => {
            state.theme_controller.update(
                action,
                state.window_size,
                theme_picker::bottom_margin(state.adaptive_navigation_layout()),
                Instant::now(),
            );
            Task::none()
        }
        Message::Frame(now) => {
            let _ = state.theme_controller.advance(now);
            let _ = state.navigation.advance(now);
            let _ = state.segment_state.advance(now);
            let _ = state.primary_tab_state.advance(now);
            let _ = state.secondary_tab_state.advance(now);
            let _ = state.log_viewer.advance(now);
            state.progress_animation.advance(now);
            let _ = state.alert_dialog.advance(now);
            let _ = state.snackbar.advance(now);
            let _ = state.date_picker.advance(now);
            let _ = state.date_range_picker.advance(now);
            let _ = state.time_picker.advance(now);
            Task::none()
        }
    }
}

fn sample_log_entries() -> Vec<material::widget::log_viewer::LogEntry<u64>> {
    use material::widget::log_viewer::{LogEntry, LogLevel};

    vec![
        LogEntry::new(
            1,
            LogLevel::Info,
            "[0005] [354884390 0ms] inbound/tun[tun-in]: inbound redirect connection from 172.19.0.1:47892",
        ),
        LogEntry::new(
            2,
            LogLevel::Info,
            "[0005] [354884390 0ms] inbound/tun[tun-in]: inbound connection to 81.69.216.240:443",
        ),
        LogEntry::new(
            3,
            LogLevel::Info,
            "[0005] [354884390 0ms] router: found user id: 10404",
        ),
        LogEntry::new(
            4,
            LogLevel::Info,
            "[0005] [354884390 6ms] outbound/direct[direct]: outbound connection to 81.69.216.240:443",
        ),
        LogEntry::new(
            5,
            LogLevel::Error,
            "[0005] [953254993 5.0s] connection: open connection to 172.19.0.2:853 using outbound/direct[direct]: dial tcp 172.19.0.2:853: i/o timeout",
        ),
        LogEntry::new(
            6,
            LogLevel::Error,
            "[0005] [2920815984 5.4s] connection: open connection to 172.19.0.2:853 using outbound/direct[direct]: dial tcp 172.19.0.2:853: i/o timeout",
        ),
        LogEntry::new(
            7,
            LogLevel::Warn,
            "[0005] router: fallback route selected for user id: 10325",
        ),
        LogEntry::new(
            8,
            LogLevel::Debug,
            "[0005] [83404445 0ms] inbound/tun[tun-in]: inbound packet connection from 172.19.0.1:55755",
        ),
        LogEntry::new(
            9,
            LogLevel::Info,
            "[0005] [83404445 0ms] inbound/tun[tun-in]: inbound packet connection to 198.18.0.16:443",
        ),
        LogEntry::new(
            10,
            LogLevel::Trace,
            "[0005] [83404445 0ms] router: matching route rules",
        ),
    ]
}

#[cfg(any(target_arch = "wasm32", test))]
fn load_cjk_regional_font() -> Task<Message> {
    material::fonts::load_web_font(CJK_REGIONAL_FONT_URL).map(|_| Message::CjkRegionalFontFinished)
}

fn theme(state: &Showcase) -> Theme {
    state.theme()
}

fn subscription(state: &Showcase) -> Subscription<Message> {
    let mut subscriptions =
        vec![iced::window::resize_events().map(|(_id, size)| Message::WindowResized(size))];

    if state.theme_controller.is_animating()
        || state.navigation.is_animating()
        || state.segment_state.is_animating()
        || state.primary_tab_state.is_animating()
        || state.secondary_tab_state.is_animating()
        || state.log_viewer.is_animating()
        || state.alert_dialog.is_animating()
        || state.snackbar.is_active()
        || state.date_picker.is_animating()
        || state.date_range_picker.is_animating()
        || state.time_picker.is_animating()
        || (state.navigation.selected() == ShowcasePage::Feedback
            && state.progress_animation.is_animating())
    {
        subscriptions.push(iced::window::frames().map(Message::Frame));
    }

    Subscription::batch(subscriptions)
}

fn view(state: &Showcase) -> material::Element<'_, Message> {
    let now = Instant::now();
    let page_content = material::widget::snackbar::host(
        pages::view(state),
        &state.snackbar,
        now,
        "Photo archived",
        "Undo",
        Message::SnackbarUndo,
    );

    let content = navigation::suite(&NAV_DESTINATIONS, &state.navigation)
        .layout(state.adaptive_navigation_layout())
        .with_menu("Showcase", Message::MenuPressed)
        .view(Message::Navigate, page_content);
    let content = state.theme_controller.controls_over(
        content,
        theme_picker::bottom_margin(state.adaptive_navigation_layout()),
        Message::ThemeChanged,
    );

    let content = material::widget::dialog::modal_animated(
        content,
        &state.alert_dialog,
        now,
        alert_dialog(state.alert_dialog.alpha(now)),
    );

    state.theme_controller.reveal_over(content, now)
}

fn alert_dialog(alpha: f32) -> material::Element<'static, Message> {
    let action_options = material::widget::dialog::AlphaOptions::default().alpha(alpha);

    material::widget::dialog::alert_with(
        "Discard draft?",
        "Your current changes will be removed from this device.",
        material::widget::dialog::actions([
            material::widget::dialog::action_button_with(
                "Cancel",
                Message::DialogDismissed,
                action_options,
            ),
            material::widget::dialog::action_button_with(
                "Discard",
                Message::DialogConfirmed,
                action_options,
            ),
        ]),
        material::widget::dialog::AlertOptions::default()
            .icon("info")
            .alpha(alpha),
    )
    .into()
}

#[cfg(test)]
#[allow(unused_must_use)]
mod tests {
    use super::*;
    use iced::Point;

    #[test]
    fn combobox_input_preserves_typed_query_and_clears_stale_selection() {
        let mut showcase = Showcase::default();

        update(&mut showcase, Message::ComboboxInputChanged("xxx".into()));

        assert_eq!(showcase.combobox_choice, None);
        assert_eq!(showcase.combobox_input, "xxx");

        update(&mut showcase, Message::ComboboxSelected("Assist"));

        assert_eq!(showcase.combobox_choice, Some("Assist"));
        assert_eq!(showcase.combobox_input, "");
    }

    #[test]
    fn date_picker_action_updates_showcase_state() {
        let mut showcase = Showcase::default();
        let date = material::widget::picker::Date::new(2026, 12, 25).unwrap();

        update(
            &mut showcase,
            Message::DatePickerChanged(material::widget::picker::DatePickerAction::SelectDate(
                date,
            )),
        );

        assert_eq!(showcase.date_picker.selected_date(), Some(date));
        assert_eq!(
            showcase.date_picker.displayed_month(),
            material::widget::picker::YearMonth::new(2026, 12).unwrap()
        );
    }

    #[test]
    fn date_range_picker_action_updates_showcase_state() {
        let mut showcase = Showcase::default();
        let start = material::widget::picker::Date::new(2026, 8, 1).unwrap();
        let end = material::widget::picker::Date::new(2026, 8, 5).unwrap();

        update(
            &mut showcase,
            Message::DateRangePickerChanged(
                material::widget::picker::DateRangePickerAction::SelectDate(start),
            ),
        );
        update(
            &mut showcase,
            Message::DateRangePickerChanged(
                material::widget::picker::DateRangePickerAction::SelectDate(end),
            ),
        );

        assert_eq!(
            showcase.date_range_picker.selected_start_date(),
            Some(start)
        );
        assert_eq!(showcase.date_range_picker.selected_end_date(), Some(end));
    }

    #[test]
    fn time_picker_action_updates_showcase_state() {
        let mut showcase = Showcase::default();

        update(
            &mut showcase,
            Message::TimePickerChanged(material::widget::picker::TimePickerAction::SelectHour(9)),
        );
        update(
            &mut showcase,
            Message::TimePickerChanged(material::widget::picker::TimePickerAction::SelectMinute(
                45,
            )),
        );

        assert_eq!(showcase.time_picker.hour(), 21);
        assert_eq!(showcase.time_picker.minute(), 45);
    }

    #[test]
    fn navigation_starts_selection_animation() {
        let mut showcase = Showcase::default();

        update(&mut showcase, Message::Navigate(ShowcasePage::Controls));

        assert_eq!(showcase.navigation.selected(), ShowcasePage::Controls);
        assert!(showcase.navigation.is_animating());
        assert_eq!(
            showcase
                .navigation
                .selection()
                .progress(ShowcasePage::Controls),
            0.0
        );
        assert_eq!(
            showcase
                .navigation
                .selection()
                .progress(ShowcasePage::Inputs),
            1.0
        );
    }

    #[test]
    fn alert_dialog_messages_toggle_modal_state() {
        let mut showcase = Showcase::default();

        update(&mut showcase, Message::DialogOpened);
        assert_eq!(
            showcase.alert_dialog.phase(),
            material::widget::dialog::TransitionPhase::Showing
        );
        assert!(showcase.alert_dialog.is_active());

        update(&mut showcase, Message::DialogDismissed);
        assert_eq!(
            showcase.alert_dialog.phase(),
            material::widget::dialog::TransitionPhase::Dismissing
        );

        update(&mut showcase, Message::DialogOpened);
        update(&mut showcase, Message::DialogConfirmed);
        assert_eq!(
            showcase.alert_dialog.phase(),
            material::widget::dialog::TransitionPhase::Dismissing
        );
        assert_eq!(showcase.count, 1);
    }

    #[test]
    fn snackbar_button_starts_android_transition() {
        let mut showcase = Showcase::default();

        update(&mut showcase, Message::ShowSnackbar);

        assert_eq!(
            showcase.snackbar.phase(),
            material::widget::snackbar::TransitionPhase::Showing
        );
        assert!(showcase.snackbar.is_active());
    }

    #[test]
    fn snackbar_action_dismisses_with_exit_transition() {
        let mut showcase = Showcase::default();

        update(&mut showcase, Message::ShowSnackbar);
        update(&mut showcase, Message::SnackbarUndo);

        assert_eq!(showcase.count, -1);
        assert_eq!(
            showcase.snackbar.phase(),
            material::widget::snackbar::TransitionPhase::Dismissing
        );
    }

    #[test]
    fn theme_picker_uses_navigation_bar_clearance() {
        assert_eq!(
            theme_picker::bottom_margin(navigation::AdaptiveLayout::NavigationBar),
            theme_picker::FLOATING_MARGIN
                + material::tokens::component::navigation_bar::CONTAINER_HEIGHT
        );
        assert_eq!(
            theme_picker::bottom_margin(navigation::AdaptiveLayout::NavigationRail),
            theme_picker::FLOATING_MARGIN
        );
    }

    #[test]
    fn selecting_current_theme_does_not_start_animation() {
        let mut showcase = Showcase::default();

        update(
            &mut showcase,
            Message::ThemeChanged(theme_picker::ThemeAction::SetDarkMode {
                dark_mode: true,
                origin: Point::new(120.0, 360.0),
            }),
        );

        assert!(!showcase.theme_controller.is_animating());
        assert!(showcase.theme_controller.dark_mode());
    }

    #[test]
    fn dark_mode_action_starts_reveal_from_switch_origin() {
        let mut showcase = Showcase::default();
        let origin = Point::new(120.0, 640.0);

        update(
            &mut showcase,
            Message::ThemeChanged(theme_picker::ThemeAction::SetDarkMode {
                dark_mode: false,
                origin,
            }),
        );

        let animation = showcase
            .theme_controller
            .transition()
            .expect("dark mode should animate");

        assert!(!showcase.theme_controller.dark_mode());
        assert_eq!(animation.origin(), origin);
    }

    #[test]
    fn theme_picker_selects_color_and_closes() {
        let mut showcase = Showcase::default();

        update(
            &mut showcase,
            Message::ThemeChanged(theme_picker::ThemeAction::TogglePicker),
        );
        assert!(showcase.theme_controller.is_picker_open());

        update(
            &mut showcase,
            Message::ThemeChanged(theme_picker::ThemeAction::SelectColor(
                theme_picker::MaterialColor::Blue,
            )),
        );

        let expected_origin = theme_picker::swatch_center(
            showcase.window_size,
            theme_picker::bottom_margin(showcase.adaptive_navigation_layout()),
            theme_picker::MaterialColor::Blue,
        );
        let animation = showcase
            .theme_controller
            .transition()
            .expect("theme selection should animate");

        assert_eq!(
            showcase.theme_controller.selected_color(),
            theme_picker::MaterialColor::Blue
        );
        assert!(!showcase.theme_controller.is_picker_open());
        assert_eq!(animation.origin(), expected_origin);
    }

    #[test]
    fn navigation_uses_material_symbol_icon_names() {
        assert_eq!(material::fonts::all().len(), 5);
        assert_eq!(
            NAV_DESTINATIONS.map(|destination| destination.icon),
            ["input", "tune", "info", "layers", "navigation", "layers"]
        );

        for destination in NAV_DESTINATIONS {
            assert!(material::fonts::material_symbol_codepoint(destination.icon).is_some());
        }
    }

    #[test]
    fn cjk_fonts_load_serially_from_boot_without_input_trigger() {
        let (mut showcase, core_load) = boot();
        assert!(core_load.units() > 0);

        let input_update = update(&mut showcase, Message::TextChanged("中文".into()));
        assert_eq!(input_update.units(), 0);
        assert_eq!(showcase.note, "中文");

        let regional_load = update(&mut showcase, Message::CjkCoreFontFinished);
        assert!(regional_load.units() > 0);
        assert_eq!(showcase.note, "中文");

        let finished = update(&mut showcase, Message::CjkRegionalFontFinished);
        assert_eq!(finished.units(), 0);
        assert_eq!(showcase.note, "中文");
    }

    #[test]
    fn every_free_text_surface_preserves_cjk_input_during_font_loading() {
        let mut note = Showcase::default();
        let note_update = update(&mut note, Message::TextChanged("中文".into()));
        assert_eq!(note_update.units(), 0);
        assert_eq!(note.note, "中文");

        let mut editor = Showcase::default();
        let editor_update = update(
            &mut editor,
            Message::EditorAction(material::widget::text_editor::Action::Edit(
                iced::widget::text_editor::Edit::Insert('中'),
            )),
        );
        assert_eq!(editor_update.units(), 0);
        assert!(editor.editor_content.text().contains('中'));

        let mut combobox = Showcase::default();
        let combobox_update = update(&mut combobox, Message::ComboboxInputChanged("中文".into()));
        assert_eq!(combobox_update.units(), 0);
        assert_eq!(combobox.combobox_input, "中文");

        let mut search = Showcase::default();
        let search_update = update(&mut search, Message::SearchChanged("中文".into()));
        assert_eq!(search_update.units(), 0);
        assert_eq!(search.search_query, "中文");
    }

    #[test]
    fn resize_updates_adaptive_layout_inputs() {
        let mut showcase = Showcase::default();

        update(
            &mut showcase,
            Message::WindowResized(Size::new(500.0, 900.0)),
        );

        assert_eq!(
            showcase.adaptive_navigation_layout(),
            material::widget::navigation::AdaptiveLayout::NavigationBar
        );

        update(
            &mut showcase,
            Message::WindowResized(Size::new(900.0, 900.0)),
        );

        assert_eq!(
            showcase.adaptive_navigation_layout(),
            material::widget::navigation::AdaptiveLayout::NavigationRail
        );
    }
}
