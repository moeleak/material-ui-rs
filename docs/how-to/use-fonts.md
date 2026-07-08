# Use Bundled and CJK Fonts

`iced_material` bundles Roboto and Material Symbols Rounded font files. The
application helper loads them for you.

## Use the Default Path

For a Material-themed app, use `material::application`:

```rust
use iced_material as material;

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

## Use CJK Font Constants

The crate exposes `Noto Sans CJK SC` font constants for applications that load
that font themselves:

```rust
let scale = material::tokens::typography::BODY_LARGE;
let font = material::fonts::noto_sans_cjk_sc_for_type_scale(scale);
```

`material::fonts::font_for_content_type_scale` chooses Roboto or Noto Sans CJK
SC from the text content. The crate does not bundle Noto Sans CJK SC bytes, so
load that family in your application if you depend on those constants.
