# Themes

`iced_material::Theme` is the crate's theme type.

## Built-in Themes

- `Theme::Dark`
- `Theme::Light`
- `Theme::ALL`

`Theme` implements the `iced` base theme contract, including base colors,
palette roles, mode detection, and display names.

## Custom Themes

Create a custom theme from a name and `ColorScheme`:

```rust
let theme = material::Theme::new("Brand", color_scheme);
```

The theme stores whether the scheme is dark based on the surface color
lightness.

## Color Scheme Types

The crate exports:

- `ColorScheme`
- `ColorQuartet`
- `Surface`
- `SurfaceContainer`
- `Inverse`
- `Outline`
- `Custom`

These types model Material color roles. Widget and style code should consume
roles instead of hard-coded colors.

## Feature Support

With the `animate` feature, `Theme` and `ColorScheme` support interpolation
through `iced_anim`. With the `serde` feature, theme data derives
serialization/deserialization support.
