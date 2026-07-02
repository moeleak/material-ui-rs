use iced::Size;
use iced::time::Instant;
use iced::widget::column;
use iced_material as material;
use material::widget::{button, navigation, page};

const WINDOW_SIZE: Size = Size::new(1080.0, 980.0);
const MIN_WINDOW_SIZE: Size = Size::new(420.0, 720.0);

pub fn main() -> iced::Result {
    material::application(boot, update, view)
        .title("iced_material quick start")
        .subscription(subscription)
        .window(material::window_with_min_size(WINDOW_SIZE, MIN_WINDOW_SIZE))
        .run()
}

#[derive(Debug, Clone)]
enum Message {
    Open(Page),
    Increment,
    Decrement,
    Menu,
    Frame(Instant),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Page {
    Home,
    Settings,
}

const DESTINATIONS: [navigation::Destination<Page>; 2] = [
    navigation::Destination::new(Page::Home, "home", "Home"),
    navigation::Destination::new(Page::Settings, "settings", "Settings"),
];

struct App {
    navigation: navigation::NavigationState<Page>,
    count: i32,
}

fn boot() -> App {
    App {
        navigation: navigation::NavigationState::new(Page::Home),
        count: 0,
    }
}

fn update(app: &mut App, message: Message) {
    match message {
        Message::Open(page) => app.navigation.select_now_for_size(page, WINDOW_SIZE),
        Message::Increment => app.count += 1,
        Message::Decrement => app.count -= 1,
        Message::Menu => app.navigation.toggle_menu_now(),
        Message::Frame(now) => app.navigation.advance_frame(now),
    }
}

fn subscription(app: &App) -> iced::Subscription<Message> {
    app.navigation.subscription(Message::Frame)
}

fn view(app: &App) -> material::Element<'_, Message> {
    navigation::suite(&DESTINATIONS, &app.navigation)
        .window_size(WINDOW_SIZE)
        .with_menu("Quick start", Message::Menu)
        .view(Message::Open, app.navigation.selected().view(app))
}

impl Page {
    fn view(self, app: &App) -> material::Element<'_, Message> {
        match self {
            Self::Home => page::surface(
                page::header("Home", "A small Material app"),
                column![
                    material::text::headline_medium(app.count.to_string()),
                    button::filled("Increment").on_press(Message::Increment),
                    button::outlined("Decrement").on_press(Message::Decrement),
                ]
                .spacing(12),
            )
            .into(),
            Self::Settings => page::surface(
                page::header("Settings", "Pages are enum variants"),
                material::text::body_large("Use the menu button in the rail"),
            )
            .into(),
        }
    }
}
