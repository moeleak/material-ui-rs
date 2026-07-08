# Theme and Color

The crate models Material 3 color roles instead of exposing one-off colors for
each widget.

## Color Roles

`ColorScheme` groups roles into:

- primary, secondary, tertiary, and error quartets;
- surface and surface container values;
- inverse roles;
- outline roles;
- shadow and scrim.

Each quartet includes a color, foreground text color, container color, and
container foreground color.

## Built-in and Custom Themes

`Theme::Dark` and `Theme::Light` provide built-in Material schemes. Custom
themes are created from a name and a `ColorScheme`.

The theme reports dark or light mode from the scheme surface color. That mode is
used by `iced` base theme integration and by style functions.

## Theme Picker

The floating theme picker builds custom schemes from Material-like seed colors.
It can animate color changes with a radial reveal when the `animate` feature is
enabled.

Use the picker for app-level theme exploration. For product themes, define a
custom `ColorScheme` and return it from the application theme callback.
