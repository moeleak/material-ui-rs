# Compose Forms and Feedback

The showcase is the canonical example for larger widget groups. These snippets
show the common composition pattern: keep widget state in your app state, map
widget actions into your message enum, and use `page` helpers for layout.

## Text Inputs

```rust
let input = material::widget::text_input::outlined("Write a note", &state.note)
    .on_input(Message::TextChanged);

let editor = material::widget::text_editor::outlined_area(&state.editor_content)
    .placeholder("Write details")
    .on_action(Message::EditorAction);
```

Text widgets follow `iced` ownership patterns. Single-line input owns a `String`
in your state; the editor uses `material::widget::text_editor::Content`.

## Select and Combobox

```rust
let select_options = ["Assist", "Suggestion", "Filter"];
let select = material::widget::select::outlined(
    select_options,
    state.select_choice,
    Message::SelectChanged,
)
.placeholder("Choose a chip")
.label("Chip type");

let combobox = material::widget::combobox::outlined_with_input(
    &state.combobox_options,
    "Search a chip",
    &state.combobox_input,
    state.combobox_choice.as_ref(),
    Message::ComboboxSelected,
)
.label("Searchable chip")
.on_input(Message::ComboboxInputChanged);
```

Use a combobox when the option list is searchable or user input should filter
the options. Use select when the option list is short and static.

## Progress and Loading

```rust
let progress = state.progress / 100.0;
let linear_phase = state.progress_animation.linear_phase();
let loading_phase = state.progress_animation.loading_phase();

material::widget::progress_bar::linear(progress, linear_phase);
material::widget::progress_bar::linear_indeterminate(linear_phase, false);
material::widget::progress_bar::loading_indicator(loading_phase);
```

Store `progress_bar::IndeterminateState` and advance it from a frame
subscription when you use indeterminate indicators.

## Dialogs, Snackbars, and Tooltips

```rust
material::widget::button::filled_action(
    "Open alert dialog",
    Message::DialogOpened,
);

material::widget::button::filled_action(
    "Show snackbar",
    Message::ShowSnackbar,
);
```

Dialog and snackbar transitions are stateful. Keep their transition state in the
application model, update it from messages and frames, and render overlays at
the top level of the page.

For local contextual help, wrap an anchor with a tooltip:

```rust
material::widget::tooltip::plain(
    material::widget::button::assist_chip("Plain"),
    "Material 3 plain tooltip",
    material::widget::tooltip::Position::Top,
)
```

Use rich tooltips when you need a title, supporting text, or action.
