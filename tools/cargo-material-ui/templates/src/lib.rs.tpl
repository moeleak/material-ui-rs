#[cfg(target_os = "android")]
use iced::Padding;
use iced::time::Instant;
use iced::widget::column;
use iced::{Size, Subscription};
use material::widget::navigation;
use material_ui_rs as material;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DemoPage {
    Inputs,
    Controls,
    Feedback,
    Surfaces,
    Navigation,
    Structure,
}

const DESTINATIONS: [navigation::Destination<DemoPage>; 6] = [
    navigation::Destination::new(DemoPage::Inputs, "input", "Inputs"),
    navigation::Destination::new(DemoPage::Controls, "tune", "Controls"),
    navigation::Destination::new(DemoPage::Feedback, "info", "Feedback").badge("3"),
    navigation::Destination::new(DemoPage::Surfaces, "layers", "Surfaces").small_badge(),
    navigation::Destination::new(DemoPage::Navigation, "navigation", "Navigation"),
    navigation::Destination::new(DemoPage::Structure, "layers", "Structure"),
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
    #[cfg(target_os = "android")]
    safe_area: material::android::SafeAreaInsets,
}

#[derive(Debug, Clone)]
enum Message {
    Increment,
    Decrement,
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

    App {
        navigation: navigation::NavigationState::new(DemoPage::Inputs),
        window_size,
        count: 0,
        #[cfg(target_os = "android")]
        safe_area: material::android::safe_area_insets(),
    }
}

fn update(app: &mut App, message: Message) {
    match message {
        Message::Increment => app.count += 1,
        Message::Decrement => app.count -= 1,
        Message::Navigate(page) => {
            let layout = navigation::adaptive_layout(app.window_size.width, app.window_size.height);
            app.navigation.select(page, Instant::now(), layout);
        }
        Message::ToggleMenu => app
            .navigation
            .toggle_menu_for_size(Instant::now(), app.window_size),
        Message::NavigationFrame(now) => app.navigation.advance_frame(now),
        Message::WindowResized(size) => app.window_size = size,
        #[cfg(target_os = "android")]
        Message::Android(material::android::Event::InsetsChanged(safe_area)) => {
            app.safe_area = safe_area;
            app.navigation.advance_frame(Instant::now());
        }
    }
}

fn subscription(app: &App) -> Subscription<Message> {
    let subscriptions = vec![
        app.navigation.subscription(Message::NavigationFrame),
        iced::window::resize_events().map(|(_id, size)| Message::WindowResized(size)),
    ];
    #[cfg(target_os = "android")]
    let subscriptions = {
        let mut subscriptions = subscriptions;
        subscriptions.push(material::android::events().map(Message::Android));
        subscriptions
    };
    Subscription::batch(subscriptions)
}

fn theme(_app: &App) -> material::Theme {
    material::Theme::Dark
}

fn view(app: &App) -> material::Element<'_, Message> {
    let selected = app.navigation.selected();
    let page_body = column![
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
    ]
    .spacing(12);
    #[cfg(target_os = "android")]
    // Keep page content clear of navigation gestures while its surface draws edge to edge.
    let page_body = page_body.push(
        iced::widget::Space::new()
            .height(iced::Length::Fixed(app.safe_area.system().bottom)),
    );
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
    let navigation_suite = navigation_suite
        .compact_navigation(navigation::CompactNavigation::ModalDrawer);
    let content = navigation_suite.view(Message::Navigate, page);

    #[cfg(target_os = "android")]
    {
        let system_safe_area = app.safe_area.system();
        // Only the IME shortens the root surface; transparent system bars stay edge to edge.
        iced::widget::container(content)
            .padding(Padding {
                top: system_safe_area.top,
                right: system_safe_area.right,
                bottom: app.safe_area.ime.bottom,
                left: system_safe_area.left,
            })
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .into()
    }
    #[cfg(not(target_os = "android"))]
    {
        content
    }
}

impl DemoPage {
    const fn label(self) -> &'static str {
        match self {
            Self::Inputs => "Inputs",
            Self::Controls => "Controls",
            Self::Feedback => "Feedback",
            Self::Surfaces => "Surfaces",
            Self::Navigation => "Navigation",
            Self::Structure => "Structure",
        }
    }
}
