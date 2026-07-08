# Add a Theme Picker

`material::widget::theme_picker` provides a floating color picker and animated
theme reveal helpers.

## Store the Controller

```rust
use material::widget::theme_picker;

struct App {
    theme_controller: theme_picker::ThemeController,
    window_size: iced::Size,
}

impl Default for App {
    fn default() -> Self {
        Self {
            theme_controller: theme_picker::ThemeController::default(),
            window_size: iced::Size::new(1080.0, 980.0),
        }
    }
}
```

## Return the Current Theme

Attach the app theme to the `iced` application builder:

```rust
fn theme(app: &App) -> material::Theme {
    app.theme_controller.theme("Showcase")
}

material::application(App::default, update, view)
    .theme(theme)
    .run()
```

## Update the Controller

The controller needs the viewport size, the floating control bottom margin, and
the current frame time.

```rust
fn update(app: &mut App, message: Message) {
    match message {
        Message::ThemeChanged(action) => {
            app.theme_controller.update(
                action,
                app.window_size,
                theme_picker::FLOATING_MARGIN,
                iced::time::Instant::now(),
            );
        }
        Message::Frame(now) => {
            let _ = app.theme_controller.advance(now);
        }
        Message::WindowResized(size) => app.window_size = size,
    }
}
```

When the theme picker is used with adaptive navigation, calculate the bottom
margin from the active navigation layout:

```rust
let layout = material::widget::navigation::AdaptiveLayout::from_size(
    app.window_size.width,
    app.window_size.height,
);
let bottom_margin = theme_picker::bottom_margin_for_navigation_layout(layout);
```

## Render the Floating Picker

Wrap the main content with the controller helpers:

```rust
let content = app.theme_controller.controls_over(
    content,
    bottom_margin,
    Message::ThemeChanged,
);

app.theme_controller.reveal_over(content, iced::time::Instant::now())
```

`controls_over` adds the floating palette button and swatches. `reveal_over`
draws the radial theme transition over the content while a theme change is
animating.
