# Build Adaptive Navigation

`material::widget::navigation` provides a single suite that switches between a
bottom navigation bar and a navigation rail.

## Define Destinations

Use a small `Copy + Eq` identifier for each destination. Enum variants work
well.

```rust
use material::widget::navigation;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Page {
    Inputs,
    Controls,
    Feedback,
}

const DESTINATIONS: [navigation::Destination<Page>; 3] = [
    navigation::Destination::new(Page::Inputs, "input", "Inputs"),
    navigation::Destination::new(Page::Controls, "tune", "Controls"),
    navigation::Destination::new(Page::Feedback, "info", "Feedback").badge("3"),
];
```

## Store the State

```rust
struct App {
    navigation: navigation::NavigationState<Page>,
    window_size: iced::Size,
}

fn boot() -> App {
    App {
        navigation: navigation::NavigationState::new(Page::Inputs),
        window_size: iced::Size::new(1080.0, 980.0),
    }
}
```

## Update Navigation

Select using the current size so animation timing matches the active layout:

```rust
fn update(app: &mut App, message: Message) {
    match message {
        Message::Navigate(page) => {
            app.navigation.select_now_for_size(
                page,
                app.window_size,
            );
        }
        Message::MenuPressed => app.navigation.toggle_menu_now(),
        Message::Frame(now) => app.navigation.advance_frame(now),
        Message::WindowResized(size) => app.window_size = size,
    }
}
```

Forward the frame subscription:

```rust
fn subscription(app: &App) -> iced::Subscription<Message> {
    app.navigation.subscription(Message::Frame)
}
```

## Render the Suite

```rust
fn view(app: &App) -> material::Element<'_, Message> {
    let content = app.navigation.selected().view(app);

    navigation::suite(&DESTINATIONS, &app.navigation)
        .window_size(app.window_size)
        .with_menu("Menu", Message::MenuPressed)
        .view(Message::Navigate, content)
}
```

The suite uses Material adaptive navigation tokens. Compact width or compact
height uses the bottom navigation bar; larger viewports use the rail.
