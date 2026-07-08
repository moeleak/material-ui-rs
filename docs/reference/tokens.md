# Tokens

Material tokens live under `material::tokens`.

Use tokens before adding local constants. They are the source of truth for
component metrics, shape, elevation, motion, state opacity, and typography.

## Token Groups

- `tokens::component` contains component-specific metrics such as button height,
  text field padding, navigation rail width, snackbar shape, and tooltip timing.
- `tokens::typography` contains Material type scale values and font weights.
- `tokens::shape` contains corner radius constants.
- `tokens::elevation` contains elevation levels and shadow helpers.
- `tokens::motion` contains durations, easing curves, and spring definitions.
- `tokens::state` contains hover, focus, pressed, dragged, disabled, and ripple
  constants.

## Component Tokens

Component token modules are named after Material component families:

```rust
let height = material::tokens::component::button::CONTAINER_HEIGHT;
let radius = material::tokens::component::text_field::CONTAINER_SHAPE;
let duration = material::tokens::component::tooltip::FADE_IN_DURATION_MS;
```

Use these values when building new Material constructors or tests for geometry
and animation behavior.
