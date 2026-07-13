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

## Pattern

The constructors set Material metrics such as height, padding, shape, typography,
and icon size. Colors and visual states are still resolved by the style catalogs
for `Theme`.

```rust
material::widget::button::filled("Save").on_press(Message::Save);
material::widget::text_input::outlined("Name", &state.name).on_input(Message::NameChanged);
```
