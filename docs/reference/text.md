# Text

Text helpers live under `material::text`.

## Type Scale Constructors

The module exposes constructors for common Material type scales:

- `headline_large`
- `headline_medium`
- `title_medium`
- `body_large`
- `body_medium`
- `type_scale`

Use `type_scale` when you already have a token from
`material::tokens::typography`:

```rust
let label = material::text::type_scale(
    "Settings",
    material::tokens::typography::TITLE_MEDIUM,
);
```

## Text Color Styles

The module also exposes text color style functions for Material roles:

- `primary`, `primary_container`
- `secondary`, `secondary_container`
- `tertiary`, `tertiary_container`
- `error`, `error_container`
- `surface`, `surface_variant`
- `inverse_surface`
- `none`

Use these when the text color should be tied to a theme role instead of the
default surface text.
