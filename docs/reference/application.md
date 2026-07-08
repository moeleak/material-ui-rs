# Application Helpers

Application helpers live at the crate root and in `iced_material::application`.

## Main Entry Points

- `material::application(boot, update, view)` creates an `iced` application that
  uses `iced_material::Theme` and preloads bundled Material fonts.
- `material::with_material_fonts(application)` adds the bundled fonts to an
  existing `iced::Application`.
- `material::window(size)` returns centered window settings.
- `material::window_with_min_size(size, min_size)` returns centered window
  settings with a minimum size.
- `material::window_settings(size, min_size)` is the lower-level helper used by
  both window constructors.

## Return Types

Use `material::Element<'_, Message>` when a view returns widgets themed by
`iced_material::Theme`:

```rust
fn view(app: &App) -> material::Element<'_, Message> {
    material::text::body_large(app.title.clone()).into()
}
```

The crate also exports `material::Container` as the themed container alias.

## When to Use `iced::application`

Use `iced::application` directly if you are integrating Material widgets into an
application with another theme type. In that case, call
`material::with_material_fonts` only if you still want the bundled fonts.
