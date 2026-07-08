# Material Tokens

Material tokens keep component behavior consistent across the crate.

The design rule is: use an existing token before adding a local constant. This
keeps metrics, motion, shape, typography, and state layers aligned when multiple
components share the same Material behavior.

## Global Tokens

Global token modules describe concepts shared across components:

- `shape` defines corner sizes.
- `elevation` defines levels and shadows.
- `motion` defines durations, cubic Bezier curves, and springs.
- `state` defines interactive opacities, disabled opacity, and ripple values.
- `typography` defines the Material type scale.

## Component Tokens

Component token modules describe exact component metrics:

- button container height and label scale;
- text field outlines and floating label metrics;
- navigation bar and rail dimensions;
- tooltip fade and rich tooltip spacing;
- dialog widths, shape, scrim opacity, and action spacing.

Widget constructors should consume these modules directly. Tests for geometry,
animation state, hit testing, and overlays should assert against tokens when the
expected value comes from Material.

## Local Constants

Local constants are still useful for implementation details that are not public
Material metrics, such as an internal interpolation factor or temporary drawing
helper value. They should not duplicate a value already present in
`material::tokens`.
