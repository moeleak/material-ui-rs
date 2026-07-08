# Build a First Material App

This tutorial builds the same shape as `examples/quickstart/app.rs`: an app with
Material fonts, a centered window, adaptive navigation, and two pages.

Run the finished example from the repository:

```sh
cargo run --example quickstart
```

## 1. Start the Application

Use `material::application` instead of `iced::application` when the whole app
uses `iced_material::Theme`. It preloads the bundled Roboto and Material Symbols
Rounded fonts and sets Roboto as the default font.

```rust
use iced::Size;
use iced_material as material;

const WINDOW_SIZE: Size = Size::new(1080.0, 980.0);
const MIN_WINDOW_SIZE: Size = Size::new(420.0, 720.0);

pub fn main() -> iced::Result {
    material::application(boot, update, view)
        .title("iced_material quick start")
        .subscription(subscription)
        .window(material::window_with_min_size(WINDOW_SIZE, MIN_WINDOW_SIZE))
        .run()
}
```

## 2. Model Pages as Data

Navigation destinations are usually enum variants. The enum is also the selected
page identifier stored in `NavigationState`.

```rust
use material::widget::navigation;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Page {
    Home,
    Settings,
}

const DESTINATIONS: [navigation::Destination<Page>; 2] = [
    navigation::Destination::new(Page::Home, "home", "Home"),
    navigation::Destination::new(Page::Settings, "settings", "Settings"),
];
```

The icon strings are Material Symbols names. The bundled helper maps common
names such as `home`, `menu`, `input`, `layers`, and `tune` to symbol
codepoints, and otherwise renders the string with the icon font.

## 3. Store Navigation State

`NavigationState` owns the selected page, rail expansion, and selection
animation state. If you use animated navigation, forward frame ticks into it.

```rust
use iced::time::Instant;

#[derive(Debug, Clone)]
enum Message {
    Open(Page),
    Increment,
    Decrement,
    Menu,
    Frame(Instant),
}

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

fn subscription(app: &App) -> iced::Subscription<Message> {
    app.navigation.subscription(Message::Frame)
}
```

## 4. Update State

Use the current window size when selecting a page so the navigation state can
animate for the active adaptive layout.

```rust
fn update(app: &mut App, message: Message) {
    match message {
        Message::Open(page) => app.navigation.select_now_for_size(page, WINDOW_SIZE),
        Message::Increment => app.count += 1,
        Message::Decrement => app.count -= 1,
        Message::Menu => app.navigation.toggle_menu_now(),
        Message::Frame(now) => app.navigation.advance_frame(now),
    }
}
```

Real applications should update the size from window resize messages instead of
using a constant.

## 5. Compose the View

The `page` helpers provide a Material surface layout. The `navigation::suite`
helper chooses a bottom navigation bar or rail from the provided size.

```rust
use iced::widget::column;
use material::widget::{button, page};

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
```

At this point the app has Material fonts, typography, button metrics, page
surface spacing, and adaptive navigation.
