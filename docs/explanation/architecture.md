# Architecture

`iced_material` is shaped around a simple split:

- constructors choose Material layout metrics;
- styles choose Material colors and state visuals;
- tokens provide the shared source of truth;
- examples show complete application composition.

## Constructor Entry Point

`src/widget.rs` is the shared constructor entry point. Small constructor families
can live in that file, while large or stateful components live under
`src/widget/component/` and are re-exported from `material::widget`.

This gives users a single public namespace:

```rust
material::widget::button::filled("Save")
material::widget::navigation::suite(&destinations, &state)
material::widget::picker::date_picker(&state.date_picker, Message::Date)
```

## Why Constructors Exist

`iced` style catalogs resolve colors, borders, shadows, and similar visual
properties. They do not set all Material component metrics, such as button
height, text field padding, navigation indicator size, or picker geometry.

Material constructors fill that gap by applying token-backed metrics at widget
creation time.

## State Ownership

Stateful components follow the same model as `iced` applications:

- persistent component state is stored in the app model;
- widget actions map into the app message enum;
- animations advance from subscriptions or frame messages;
- views borrow state and return `material::Element`.

The quickstart and showcase examples are the reference composition patterns.
