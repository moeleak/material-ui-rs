# material-ui-rs

[Live demo](https://material.leak.moe)

Material 3 inspired widgets and theme defaults for [`iced`](https://iced.rs)
0.14.

![material-ui-rs light showcase](assets/screenshots/light.png)
![material-ui-rs dark showcase](assets/screenshots/dark.png)

## Quick Start

Run the compact tutorial app:

```sh
cargo run --example quickstart
```

Run the full component showcase:

```sh
cargo run --example showcase
```

Build and serve the WebAssembly showcase:

```sh
trunk build web/index.html --release --dist dist --public-url /
python3 -m http.server 4173 --directory dist
```

Open <http://127.0.0.1:4173/>. Serve `dist/` over HTTP instead of opening the
HTML file directly, so the browser loads the JavaScript module and WASM with the
correct MIME types.

## Application CLI

The default Nix development shell includes the workspace CLI:

```sh
nix develop
cargo material-ui new material-app
```

Projects initialized with the Nix backend also include `cargo-material-ui` in
their generated development shell.

Without Nix, install the CLI and then start the interactive project wizard:

```sh
cargo install --path tools/cargo-material-ui
cargo material-ui new material-app
```

Use `cargo material-ui new [PATH]` to create a new project directory, or run
`cargo material-ui init` inside an existing directory to initialize it. The
wizard lets you select macOS, Linux, Windows, Web, or Android. When Nix is
available it also offers Native Rust or Nix Flake; the generated `flake.nix`
contains only the toolchains and SDKs required by the selected platforms.
Platforms and the build backend can be changed later:

```sh
cargo material-ui configure
cargo material-ui build --release
```

See [Create and build a multi-platform app](docs/how-to/create-multiplatform-app.md)
for generated files, non-interactive use, artifact locations, Android behavior,
and signing options.

## Documentation

The documentation is organized with Diátaxis:

- [Tutorials](docs/tutorials/first-material-app.md): learn by building a first
  Material app.
- [How-to guides](docs/how-to/run-examples.md): run examples, build the web
  showcase, use fonts, adaptive navigation, theme picking, forms, and feedback.
- [Reference](docs/reference/application.md): find module-level API entry
  points.
- [Explanation](docs/explanation/architecture.md): understand the architecture,
  tokens, theme model, and native/WebAssembly split.

Build the mdBook locally:

```sh
mdbook build
```

With Nix:

```sh
nix develop -c mdbook build
```

API documentation is available on [docs.rs](https://docs.rs/material-ui-rs).

## Components

The crate provides Material-sized constructors and token-backed styles for:

- Buttons, floating action buttons, icon buttons, and chips.
- Text input, text editor, select, and searchable combobox.
- Date picker, date range picker, time picker, time input, and time scroll.
- Checkbox, switch, radio, slider, tabs, segmented buttons, and progress
  indicators.
- Dividers, tooltips, badges, lists, cards, data tables, toolbars, sheets,
  dialogs, snackbars, and theme picker.
- Application, centered window, page surface, and adaptive navigation helpers.
- Material color schemes, typography, shape, elevation, motion, and state
  tokens.
- Bundled Roboto and Material Symbols Rounded font helpers.
- Noto Sans CJK SC font family helpers for applications that provide CJK fonts.

## Features

- `default`: Enables SVG support, animations, and canvas drawing.
- `serde`: Adds `serde` support for theme data.
- `animate`: Enables integration with `iced_anim`.
- `canvas`: Compatibility feature; canvas drawing is always enabled.
- `crisp`: Enables pixel snapping for crisp edges.
- `dialog`: Enables `iced_dialog` support.
- `selection`: Enables `iced_selection` support.
- `markdown`: Enables the markdown widget.
- `svg`: Enables the SVG widget.
- `qr_code`: Enables the QR code widget.

## License

MIT
