# Styles

Style functions live under `material::style`. They provide `iced` catalog styles
for `iced_material::Theme`.

Use widget constructors first. Reach for style functions when you are composing
lower-level `iced_widget` values and need Material colors, borders, shadows, or
state layers without using the full Material constructor.

## Catalog Areas

The style modules mirror common `iced_widget` catalogs:

- `button`
- `checkbox`
- `combobox`
- `container`
- `dialog`
- `list`
- `menu`
- `progress_bar`
- `radio`
- `rule`
- `scrollable`
- `select`
- `slider`
- `table`
- `text_editor`
- `text_input`
- `toggler`
- `tooltip`

Feature-gated style modules are available when their matching feature is
enabled, such as markdown, QR code, selection, and SVG support.

## Relationship to Widgets

Widget constructors apply Material metrics. Style functions apply visual
treatment for theme state. A custom constructor often needs both:

```rust
iced_widget::button(content)
    .style(material::style::button::filled)
```

Prefer an existing `material::widget` constructor unless you need custom layout
behavior.
