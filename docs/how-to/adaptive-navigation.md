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

## Android safe areas and keyboard

Keep the full window size for adaptive layout and pass safe-area padding to the
suite once. Its surface extends behind Android's system bars, while its content
and navigation items stay clear of them. A bottom navigation bar retains its
80dp item area; the bottom inset extends its background below the items.

Read `material::android::layout_insets()` at startup and whenever
`material::android::events()` produces `Event::InsetsChanged`. These layout
insets account for any area the native window has already excluded. Use the
event's raw insets to determine keyboard visibility, since a resized window
may have no remaining IME padding to apply:

```rust
// Android-only update branch:
Message::Android(material::android::Event::InsetsChanged(raw)) => {
    app.safe_area = material::android::layout_insets();
    app.ime_visible = raw.ime.bottom > 0.0;
}
```

```rust
let insets = app.safe_area.content();

navigation::suite(&DESTINATIONS, &app.navigation)
    .window_size(app.window_size)
    .with_menu("Menu", Message::MenuPressed)
    .insets(iced::Padding {
        top: insets.top,
        right: insets.right,
        bottom: insets.bottom,
        left: insets.left,
    })
    .navigation_bar_visible(!app.ime_visible)
    .view(Message::Navigate, content)
```

The hidden bottom bar releases its full height and preserves selection. The
rail and drawer remain available in their respective layouts. Do not add the
same system or IME padding to the root container or append a system-navigation
spacer to the page. Without `.insets(...)` the suite uses zero padding; without
`.navigation_bar_visible(...)` its bottom bar is visible.

For a standalone bar, use `navigation::bar_with` with
`navigation::NavigationBarOptions::default().insets(padding)`. It protects the
bar's interactive area without padding unrelated page content.

Keep compact navigation to three to five top-level destinations. The generated
Android app uses Components, Navigation, and Structure; Components contains
secondary Inputs, Controls, Feedback, and Surfaces tabs. Store the secondary
selection separately so returning to Components restores its previous page.

## Compact modal drawer

To use an AndroidX Compose-style modal drawer instead of the compact navigation
bar, opt in explicitly:

```rust
navigation::suite(&DESTINATIONS, &app.navigation)
    .window_size(app.window_size)
    .with_menu("Menu", Message::MenuPressed)
    .compact_navigation(navigation::CompactNavigation::ModalDrawer)
    .view(Message::Navigate, content)
```
