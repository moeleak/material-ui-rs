# iced_material

`iced_material` provides Material 3 inspired widgets, theme defaults, typography
helpers, and design tokens for `iced` 0.14 applications.

Use this book by the kind of work you are doing:

- Start with [Build a first Material app](tutorials/first-material-app.md) if
  you are learning the crate for the first time.
- Use the [How-to Guides](how-to/run-examples.md) when you already know what
  you want to build.
- Use the [Reference](reference/application.md) to find the public module that
  owns an API.
- Use the [Explanation](explanation/architecture.md) pages to understand why
  the crate is shaped around Material constructors, tokens, and theme roles.

## Repository Baseline

The crate currently targets:

- Rust 1.88 or newer.
- `iced` 0.14.
- A Material `Theme` exported from `iced_material`.
- A default feature set that enables SVG support, animation support, and canvas
  drawing.

The quickest runnable entry point is:

```sh
cargo run --example quickstart
```

The full component showcase is:

```sh
cargo run --example showcase
```

## API Shape

Most applications use these modules first:

- `iced_material::application` for bootstrapping an `iced` application with
  bundled fonts and the Material theme.
- `iced_material::widget` for Material-sized constructors and custom widgets.
- `iced_material::text` for Material typography helpers.
- `iced_material::tokens` for Material component metrics and motion constants.
- `iced_material::Theme` for light, dark, and custom color schemes.
