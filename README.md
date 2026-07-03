# iced_material [Live Demo](https://material.leak/moe)

Material 3 inspired widgets and theme defaults for
[`iced`](https://iced.rs) 0.14.

Showcase

![iced_material light showcase](assets/screenshots/light.png)
![iced_material dark showcase](assets/screenshots/dark.png)

## Quick Start

Run the 91-line animated quick start app:

```sh
cargo run --example quickstart
```

```rust
use iced::Size;
use iced_material as material;

fn main() -> iced::Result {
    material::application(boot, update, view)
        .title("iced_material quick start")
        .subscription(subscription)
        .window(material::window_with_min_size(
            Size::new(1080.0, 980.0),
            Size::new(420.0, 720.0),
        ))
        .run()
}
```

Core view composition uses page, menu navigation, and widget helpers:

```rust
use material::widget::{button, navigation, page};

fn view(app: &App) -> material::Element<'_, Message> {
    let content = page::surface(
        page::header("Home", "A small Material app"),
        button::filled("Increment").on_press(Message::Increment),
    );

    navigation::suite(&destinations, &app.navigation)
        .dimensions(1080.0, 980.0)
        .with_menu("Quick start", Message::Menu)
        .view(Message::Open, content)
}
```

## Components

The crate provides Material-sized constructors and token-backed styles for:

- Buttons, floating action buttons, icon buttons, and chips
- Text input, text editor, select, and searchable combo box
- Checkbox, switch, radio, slider, and progress indicator
- Dividers, tooltips, badges, lists, cards, data tables, toolbars, and theme picker
- Application, centered window, page surface, and adaptive navigation helpers
- Material color schemes, typography tokens, shape tokens, elevation, and motion constants
- Bundled Roboto and Material Symbols Rounded font helpers
- Noto Sans CJK SC font family helpers for applications that provide CJK fonts

## Features

- `default`: Enables SVG support and Material animations.
- `serde`: Adds `serde` support for theme data.
- `animate`: Enables integration with `iced_anim`.
- `crisp`: Enables pixel snapping for crisp edges.
- `dialog`: Enables `iced_dialog` support.
- `selection`: Enables `iced_selection` support.
- `markdown`: Enables the markdown widget.
- `svg`: Enables the SVG widget.
- `qr_code`: Enables the QR code widget.

## License

MIT
