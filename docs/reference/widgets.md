# Widgets

Material constructors live under `material::widget`. The root file is the shared
constructor entry point; larger or stateful components are grouped into focused
modules.

## Layout and App Structure

- `page` provides page surfaces, headers, sections, rows, stacks, and common
  showcase-style layout helpers.
- `navigation` provides destinations, adaptive navigation state, bottom
  navigation bar, rail, drawer, and the `suite` builder.
- `app_bar`, `toolbar`, `sheet`, `dialog`, `snackbar`, and `viewport` provide
  higher-level structure and overlays.

## Inputs

- `text_input` provides Material text fields.
- `text_editor` provides multi-line text editor constructors.
- `select` and `combobox` provide selection fields.
- `picker` provides date picker, date range picker, time picker, time input, and
  time scroll state.
- `search` provides search bar and search view helpers.

## Actions and Selection Controls

- `button` provides elevated, filled, tonal, outlined, text, icon, FAB, extended
  FAB, and chip constructors.
- `checkbox`, `radio`, `slider`, `segmented_button`, `tabs`, and `toggler`
  provide Material-sized selection controls.

## Feedback and Data Display

- `progress_bar` provides determinate and indeterminate progress indicators.
- `badge`, `tooltip`, `card`, `list`, `data_table`, and `rule` provide common
  display components.
- `log_viewer` provides structured severity rows, selection state, contextual
  close/copy actions, and bottom-anchored scrolling. Page titles remain the
  responsibility of the caller; advance its state from frame events while the
  contextual bar is animating.
- `theme_picker` provides a floating Material color picker and animated theme
  reveal helpers.

## Media and Collections

- `image` (feature `image`) provides images with corners from the shape scale,
  cropped to cover their bounds, and placeholders for images that are loading,
  failed to load, or are missing. `image::Cache` decodes images in the
  background, optionally cropped to an aspect ratio and down to a bounded size,
  and keeps the most recently requested ones; `image::cached` shows an image from it, or its placeholder.
- `grid` provides fluid grids of cells at most a given width, with Material
  spacing, and `grid::aspect_ratio` for content of a fixed aspect ratio inside
  a cell, such as a cover above its title.
- `menu` provides context menus: `menu::context` wraps content with a menu of
  actions that opens at the pointer when the content is right-clicked. Actions
  take an icon, are disabled until they have a message, and can be marked
  destructive; dividers separate groups.

See [Show a library of covers](../how-to/cover-library.md) for the three
together.

## Pattern

The constructors set Material metrics such as height, padding, shape, typography,
and icon size. Colors and visual states are still resolved by the style catalogs
for `Theme`.

```rust
use material::widget::button::{self, ButtonVariant};

button::button("Save", ButtonVariant::Filled).on_press(Message::Save);
material::widget::text_input::outlined("Name", &state.name).on_input(Message::NameChanged);
```
