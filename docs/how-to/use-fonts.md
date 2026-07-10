# Use Bundled and CJK Fonts

`material-ui-rs` bundles Roboto and Material Symbols Rounded font files. The
application helper loads them for you.

## Use the Default Path

For a Material-themed app, use `material::application`:

```rust
use material_ui_rs as material;

material::application(boot, update, view)
    .title("My app")
    .run()
```

This is equivalent to creating an `iced` application and then calling
`material::with_material_fonts`.

## Add Material Fonts to an Existing Application

If an app is already created through `iced::application`, wrap it before
running:

```rust
let app = iced::application(boot, update, view);

material::with_material_fonts(app).run()
```

Use this when you need the regular `iced` application builder first but still
want the bundled fonts.

## Use Material Symbols

Use `material::fonts::icon` or the widget constructors that accept icon names:

```rust
let icon = material::fonts::icon("menu", 24.0);
```

Several common symbol names are mapped to codepoints. Other names are rendered
through the Material Symbols Rounded font, so the exact output depends on the
font's ligature support in the renderer.

## Load CJK Fonts on WebAssembly

Full CJK fonts are intentionally not embedded in the crate because they add
many megabytes to the WASM module. Keep the application startup path small and
return a web-font task only after content first needs CJK glyphs:

```rust
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
enum CjkFontStatus {
    #[default]
    Idle,
    Loading,
    Loaded,
}

fn update(state: &mut State, message: Message) -> iced::Task<Message> {
    match message {
        Message::TextChanged(value) => {
            let should_load = cfg!(target_arch = "wasm32")
                && state.cjk_font == CjkFontStatus::Idle
                && material::fonts::contains_cjk(&value);
            state.text = value;

            if should_load {
                state.cjk_font = CjkFontStatus::Loading;

                return material::fonts::load_web_font(
                    "fonts/NotoSansCJKsc-Regular.otf",
                )
                .map(Message::CjkFontLoaded);
            }
        }
        Message::CjkFontLoaded(result) => {
            state.cjk_font = if result.is_ok() {
                CjkFontStatus::Loaded
            } else {
                CjkFontStatus::Idle
            };
        }
    }

    iced::Task::none()
}
```

The URL should be same-origin when practical and served with long-lived cache
headers. A cross-origin URL must allow CORS. Use a raw `.ttf`, `.otf`, or `.ttc`
file; browser-oriented WOFF2 files and CSS `@font-face` rules cannot populate
iced's renderer font database. For a smaller download, host a locale or glyph
subset that matches the content your application supports.

Handle `Message::CjkFontLoaded` to stop showing a loading fallback and redraw
CJK content. The downloaded bytes remain outside the `.wasm` binary and can be
cached independently by the browser. `load_web_font` does not start until its
task is returned from boot or update, so the example makes no font request at
startup.

To let Trunk copy a self-hosted `web/fonts/` directory, add this optional host
page asset next to the Rust link:

```html
<link data-trunk rel="copy-dir" href="fonts" />
```

## Use CJK Font Constants

The crate exposes `Noto Sans CJK SC` font constants for applications that load
that font family themselves:

```rust
let scale = material::tokens::typography::BODY_LARGE;
let font = material::fonts::noto_sans_cjk_sc_for_type_scale(scale);
```

`material::fonts::font_for_content_type_scale` chooses Roboto or Noto Sans CJK
SC from the text content. Load a face whose internal family name is
`Noto Sans CJK SC` before relying on those constants. Applications targeting
Traditional Chinese, Japanese, or Korean should load and select the matching
regional Noto Sans CJK family so shared Han characters use locale-appropriate
glyph forms.
