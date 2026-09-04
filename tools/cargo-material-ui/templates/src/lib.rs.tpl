#[cfg(target_os = "android")]
use iced::Padding;
use iced::time::Instant;
use iced::widget::column;
use iced::{Size, Subscription};
use material::widget::navigation;
use material_ui_rs as material;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DemoPage {
    #[cfg(target_os = "android")]
    Components,
    Inputs,
    Controls,
    Feedback,
    Surfaces,
    Navigation,
    Structure,
}

#[cfg(not(target_os = "android"))]
const DESTINATIONS: [navigation::Destination<DemoPage>; 6] = [
    navigation::Destination::new(DemoPage::Inputs, "input", "Inputs"),
    navigation::Destination::new(DemoPage::Controls, "tune", "Controls"),
    navigation::Destination::new(DemoPage::Feedback, "info", "Feedback").badge("3"),
    navigation::Destination::new(DemoPage::Surfaces, "layers", "Surfaces").small_badge(),
    navigation::Destination::new(DemoPage::Navigation, "navigation", "Navigation"),
    navigation::Destination::new(DemoPage::Structure, "layers", "Structure"),
];

#[cfg(target_os = "android")]
const DESTINATIONS: [navigation::Destination<DemoPage>; 3] = [
    navigation::Destination::new(DemoPage::Components, "widgets", "Components"),
    navigation::Destination::new(DemoPage::Navigation, "navigation", "Navigation"),
    navigation::Destination::new(DemoPage::Structure, "layers", "Structure"),
];

#[cfg(target_os = "android")]
const COMPONENT_PAGES: [DemoPage; 4] = [
    DemoPage::Inputs,
    DemoPage::Controls,
    DemoPage::Feedback,
    DemoPage::Surfaces,
];

pub fn run() -> iced::Result {
    let application = material::application(boot, update, view);
    #[cfg(target_os = "android")]
    let application = material::android::system_fonts()
        .into_iter()
        .fold(application, iced::Application::font);

    application
        .title({{label_rust}})
        .theme(theme)
        .subscription(subscription)
        .window(material::window_with_min_size(
            Size::new(960.0, 720.0),
            Size::new(360.0, 640.0),
        ))
        .run()
}

#[cfg(target_os = "android")]
pub fn run_android(app: material::android::AndroidApp) {
    material::android::run(app, run);
}

struct App {
    navigation: navigation::NavigationState<DemoPage>,
    window_size: Size,
    count: i32,
    note: String,
    theme: material::Theme,
    #[cfg(target_os = "android")]
    component_page: DemoPage,
    #[cfg(target_os = "android")]
    component_tabs: material::widget::tabs::State,
    #[cfg(target_os = "android")]
    ime_visible: bool,
    #[cfg(target_os = "android")]
    safe_area: material::android::SafeAreaInsets,
}

#[derive(Debug, Clone)]
enum Message {
    Increment,
    Decrement,
    ToggleTheme,
    NoteChanged(String),
    Navigate(DemoPage),
    ToggleMenu,
    NavigationFrame(Instant),
    WindowResized(Size),
    #[cfg(target_os = "android")]
    Android(material::android::Event),
}

fn boot() -> App {
    #[cfg(target_os = "android")]
    let window_size = Size::new(360.0, 640.0);
    #[cfg(not(target_os = "android"))]
    let window_size = Size::new(960.0, 720.0);

    #[cfg(target_os = "android")]
    let initial_page = DemoPage::Components;
    #[cfg(not(target_os = "android"))]
    let initial_page = DemoPage::Inputs;
    let theme = material::Theme::Dark;
    #[cfg(target_os = "android")]
    sync_system_bars(&theme);

    App {
        navigation: navigation::NavigationState::new(initial_page),
        window_size,
        count: 0,
        note: String::new(),
        theme,
        #[cfg(target_os = "android")]
        component_page: DemoPage::Inputs,
        #[cfg(target_os = "android")]
        component_tabs: material::widget::tabs::State::new(0),
        #[cfg(target_os = "android")]
        ime_visible: material::android::safe_area_insets().ime.bottom > 0.0,
        #[cfg(target_os = "android")]
        safe_area: material::android::layout_insets(),
    }
}

fn update(app: &mut App, message: Message) {
    match message {
        Message::Increment => app.count += 1,
        Message::Decrement => app.count -= 1,
        Message::NoteChanged(note) => app.note = note,
        Message::ToggleTheme => {
            app.theme = if app.theme.is_dark() {
                material::Theme::Light
            } else {
                material::Theme::Dark
            };
            #[cfg(target_os = "android")]
            sync_system_bars(&app.theme);
        }
        Message::Navigate(page) => {
            #[cfg(target_os = "android")]
            let page = if let Some(index) = COMPONENT_PAGES.iter().position(|item| *item == page) {
                app.component_page = page;
                app.component_tabs.select(
                    index,
                    Instant::now(),
                    material::widget::tabs::Variant::Secondary,
                );
                DemoPage::Components
            } else {
                page
            };
            let layout = navigation::adaptive_layout(app.window_size.width, app.window_size.height);
            app.navigation.select(page, Instant::now(), layout);
        }
        Message::ToggleMenu => app
            .navigation
            .toggle_menu_for_size(Instant::now(), app.window_size),
        Message::NavigationFrame(now) => {
            app.navigation.advance_frame(now);
            #[cfg(target_os = "android")]
            let _ = app.component_tabs.advance(now);
        }
        Message::WindowResized(size) => app.window_size = size,
        #[cfg(target_os = "android")]
        Message::Android(material::android::Event::InsetsChanged(safe_area)) => {
            app.safe_area = material::android::layout_insets();
            app.ime_visible = safe_area.ime.bottom > 0.0;
            app.navigation.advance_frame(Instant::now());
        }
    }
}

fn subscription(app: &App) -> Subscription<Message> {
    let subscriptions = vec![
        app.navigation.subscription(Message::NavigationFrame),
        iced::event::listen_with(|event, _, _| match event {
            iced::Event::Window(
                iced::window::Event::Opened { size, .. } | iced::window::Event::Resized(size),
            ) => Some(Message::WindowResized(size)),
            _ => None,
        }),
    ];
    #[cfg(target_os = "android")]
    let subscriptions = {
        let mut subscriptions = subscriptions;
        subscriptions.push(material::android::events().map(Message::Android));
        if app.component_tabs.is_animating() && !app.navigation.is_animating() {
            subscriptions.push(iced::window::frames().map(Message::NavigationFrame));
        }
        subscriptions
    };
    Subscription::batch(subscriptions)
}

fn theme(app: &App) -> material::Theme {
    app.theme.clone()
}

#[cfg(target_os = "android")]
fn sync_system_bars(theme: &material::Theme) {
    let style = material::android::SystemBarsStyle::from_theme(theme);
    if let Err(error) = material::android::set_system_bars(style) {
        eprintln!("Could not request Android system-bar update: {error:?}");
    }
}

fn view(app: &App) -> material::Element<'_, Message> {
    let selected = app.navigation.selected();
    #[cfg(target_os = "android")]
    let selected = if selected == DemoPage::Components {
        app.component_page
    } else {
        selected
    };
    let page_body = column![
        material::widget::text_input::outlined("Write a note", &app.note)
            .on_input(Message::NoteChanged),
        material::text::headline_medium(app.count.to_string()),
        material::widget::button::button(
            "Increment",
            material::widget::button::ButtonVariant::Filled,
        )
        .on_press(Message::Increment),
        material::widget::button::button(
            "Decrement",
            material::widget::button::ButtonVariant::Outlined,
        )
        .on_press(Message::Decrement),
        material::widget::button::button(
            "Toggle theme",
            material::widget::button::ButtonVariant::Text,
        )
        .on_press(Message::ToggleTheme),
    ]
    .spacing(12);
    let page = material::widget::page::surface(
        material::widget::page::header(
            selected.label(),
            "A multi-platform Material navigation demo",
        ),
        page_body,
    );
    let navigation_suite = navigation::suite(&DESTINATIONS, &app.navigation)
        .window_size(app.window_size)
        .with_menu({{label_rust}}, Message::ToggleMenu);
    #[cfg(target_os = "android")]
    let navigation_suite = {
        let insets = app.safe_area.content();
        navigation_suite
            .insets(Padding {
                top: insets.top,
                right: insets.right,
                bottom: insets.bottom,
                left: insets.left,
            })
            .navigation_bar_visible(!app.ime_visible)
    };
    #[cfg(target_os = "android")]
    let page: material::Element<'_, Message> = if app.navigation.selected() == DemoPage::Components
    {
        column![
            material::widget::tabs::animated_tabs(
                material::widget::tabs::Variant::Secondary,
                &app.component_tabs,
                COMPONENT_PAGES.map(|page| (
                    material::widget::tabs::Content::label(page.label()),
                    Message::Navigate(page),
                )),
            ),
            page,
        ]
        .into()
    } else {
        page.into()
    };
    navigation_suite.view(Message::Navigate, page)
}

impl DemoPage {
    const fn label(self) -> &'static str {
        match self {
            #[cfg(target_os = "android")]
            Self::Components => "Components",
            Self::Inputs => "Inputs",
            Self::Controls => "Controls",
            Self::Feedback => "Feedback",
            Self::Surfaces => "Surfaces",
            Self::Navigation => "Navigation",
            Self::Structure => "Structure",
        }
    }
}
