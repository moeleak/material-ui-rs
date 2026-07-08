use iced_widget::core::{Element, keyboard};

use super::*;

#[derive(Debug, Clone)]
enum Message {
    Pressed,
    Toggled,
}

type TestElement<'a> = Element<'a, Message, Theme, iced_widget::Renderer>;

fn toggled(_: bool) -> Message {
    Message::Toggled
}

fn toggled_at(_: bool, _: Point) -> Message {
    Message::Toggled
}

#[test]
fn centered_icon_text_uses_square_icon_bounds() {
    let icon: Text<'_, Theme, iced_widget::Renderer> =
        centered_icon_text("add", tokens::component::fab::ICON_SIZE);

    assert_eq!(
        Widget::<Message, Theme, iced_widget::Renderer>::size(&icon),
        Size {
            width: Length::Fixed(tokens::component::fab::ICON_SIZE),
            height: Length::Fixed(tokens::component::fab::ICON_SIZE),
        }
    );
}

#[test]
fn text_field_touch_cursor_preserves_translated_cursor_position() {
    let raw_position = Point::new(24.0, 48.0);
    let translated_position = Point::new(24.0, 148.0);
    let event = Event::Touch(touch::Event::FingerPressed {
        id: touch::Finger(1),
        position: raw_position,
    });

    assert_eq!(
        text_field_touch_cursor(&event, mouse::Cursor::Available(translated_position)),
        mouse::Cursor::Available(translated_position)
    );
}

#[test]
fn text_field_touch_cursor_uses_finger_position_without_cursor() {
    let position = Point::new(24.0, 48.0);
    let event = Event::Touch(touch::Event::FingerPressed {
        id: touch::Finger(1),
        position,
    });

    assert_eq!(
        text_field_touch_cursor(&event, mouse::Cursor::Unavailable),
        mouse::Cursor::Available(position)
    );
}

#[test]
fn touch_events_convert_to_mouse_events_for_text_editor() {
    let position = Point::new(24.0, 48.0);

    assert_eq!(
        touch_as_mouse_event(&Event::Touch(touch::Event::FingerPressed {
            id: touch::Finger(1),
            position,
        })),
        Some(Event::Mouse(mouse::Event::ButtonPressed(
            mouse::Button::Left
        )))
    );
    assert_eq!(
        touch_as_mouse_event(&Event::Touch(touch::Event::FingerMoved {
            id: touch::Finger(1),
            position,
        })),
        Some(Event::Mouse(mouse::Event::CursorMoved { position }))
    );
    assert_eq!(
        touch_as_mouse_event(&Event::Touch(touch::Event::FingerLifted {
            id: touch::Finger(1),
            position,
        })),
        Some(Event::Mouse(mouse::Event::ButtonReleased(
            mouse::Button::Left
        )))
    );
}

fn keyboard_key_pressed(key: keyboard::Key, text: Option<&str>) -> Event {
    Event::Keyboard(keyboard::Event::KeyPressed {
        key: key.clone(),
        modified_key: key,
        physical_key: keyboard::key::Physical::Unidentified(
            keyboard::key::NativeCode::Unidentified,
        ),
        location: keyboard::Location::Standard,
        modifiers: keyboard::Modifiers::default(),
        text: text.map(Into::into),
        repeat: false,
    })
}

#[test]
fn text_caret_refresh_tracks_text_entry_events() {
    assert!(text_caret_refresh_event(&keyboard_key_pressed(
        keyboard::Key::Character("a".into()),
        Some("a"),
    )));
    assert!(text_caret_refresh_event(&Event::InputMethod(
        input_method::Event::Commit("a".into())
    )));
    assert!(text_caret_refresh_event(&Event::InputMethod(
        input_method::Event::Preedit("pinyin".into(), None)
    )));
}

#[test]
fn text_caret_refresh_ignores_unfocus_keys() {
    assert!(!text_caret_refresh_event(&keyboard_key_pressed(
        keyboard::Key::Named(keyboard::key::Named::Escape),
        None,
    )));
}

#[test]
fn press_is_over_accepts_touch_positions_without_cursor() {
    let bounds = Rectangle::new(Point::new(10.0, 20.0), Size::new(100.0, 48.0));

    assert!(press_is_over(
        &Event::Touch(touch::Event::FingerPressed {
            id: touch::Finger(1),
            position: Point::new(20.0, 30.0),
        }),
        bounds,
        mouse::Cursor::Unavailable
    ));
    assert!(!press_is_over(
        &Event::Touch(touch::Event::FingerPressed {
            id: touch::Finger(1),
            position: Point::new(200.0, 300.0),
        }),
        bounds,
        mouse::Cursor::Unavailable
    ));
}

#[test]
fn press_is_over_prefers_translated_cursor_for_touch() {
    let bounds = Rectangle::new(Point::new(10.0, 120.0), Size::new(100.0, 48.0));

    assert!(press_is_over(
        &Event::Touch(touch::Event::FingerPressed {
            id: touch::Finger(1),
            position: Point::new(20.0, 30.0),
        }),
        bounds,
        mouse::Cursor::Available(Point::new(20.0, 130.0))
    ));
    assert!(!press_is_over(
        &Event::Touch(touch::Event::FingerPressed {
            id: touch::Finger(1),
            position: Point::new(20.0, 130.0),
        }),
        bounds,
        mouse::Cursor::Available(Point::new(20.0, 30.0))
    ));
    assert!(!press_is_over(
        &Event::Touch(touch::Event::FingerPressed {
            id: touch::Finger(1),
            position: Point::new(20.0, 130.0),
        }),
        bounds,
        mouse::Cursor::Levitating(Point::new(20.0, 30.0))
    ));
}

#[test]
fn text_field_keyboard_activation_waits_for_confirmed_touch_tap() {
    let bounds = Rectangle::new(Point::new(10.0, 120.0), Size::new(100.0, 48.0));
    let mut activation = None;

    assert!(!text_field_keyboard_activation(
        &mut activation,
        &Event::Touch(touch::Event::FingerPressed {
            id: touch::Finger(1),
            position: Point::new(20.0, 130.0),
        }),
        bounds,
        mouse::Cursor::Available(Point::new(20.0, 130.0))
    ));
    assert!(activation.is_some());

    assert!(text_field_keyboard_activation(
        &mut activation,
        &Event::Touch(touch::Event::FingerLifted {
            id: touch::Finger(1),
            position: Point::new(20.0, 130.0),
        }),
        bounds,
        mouse::Cursor::Available(Point::new(20.0, 130.0))
    ));
    assert!(activation.is_none());
}

#[test]
fn text_field_keyboard_activation_cancels_touch_scroll() {
    let bounds = Rectangle::new(Point::new(10.0, 120.0), Size::new(100.0, 48.0));
    let mut activation = None;

    assert!(!text_field_keyboard_activation(
        &mut activation,
        &Event::Touch(touch::Event::FingerPressed {
            id: touch::Finger(1),
            position: Point::new(20.0, 130.0),
        }),
        bounds,
        mouse::Cursor::Unavailable
    ));
    assert!(!text_field_keyboard_activation(
        &mut activation,
        &Event::Touch(touch::Event::FingerMoved {
            id: touch::Finger(1),
            position: Point::new(20.0, 130.0 + TEXT_FIELD_TOUCH_SLOP + 1.0),
        }),
        bounds,
        mouse::Cursor::Unavailable
    ));
    assert!(activation.is_none());

    assert!(!text_field_keyboard_activation(
        &mut activation,
        &Event::Touch(touch::Event::FingerLifted {
            id: touch::Finger(1),
            position: Point::new(20.0, 130.0),
        }),
        bounds,
        mouse::Cursor::Unavailable
    ));
}

#[test]
fn text_field_keyboard_activation_uses_translated_cursor_position() {
    let bounds = Rectangle::new(Point::new(10.0, 120.0), Size::new(100.0, 48.0));
    let mut activation = None;

    assert!(!text_field_keyboard_activation(
        &mut activation,
        &Event::Touch(touch::Event::FingerPressed {
            id: touch::Finger(1),
            position: Point::new(20.0, 30.0),
        }),
        bounds,
        mouse::Cursor::Available(Point::new(20.0, 130.0))
    ));
    assert!(activation.is_some());

    assert!(text_field_keyboard_activation(
        &mut activation,
        &Event::Touch(touch::Event::FingerLifted {
            id: touch::Finger(1),
            position: Point::new(20.0, 30.0),
        }),
        bounds,
        mouse::Cursor::Available(Point::new(20.0, 130.0))
    ));
    assert!(activation.is_none());
}

#[test]
fn text_field_keyboard_activation_rejects_raw_position_when_translated_cursor_is_outside() {
    let bounds = Rectangle::new(Point::new(10.0, 120.0), Size::new(100.0, 48.0));
    let mut activation = None;

    assert!(!text_field_keyboard_activation(
        &mut activation,
        &Event::Touch(touch::Event::FingerPressed {
            id: touch::Finger(1),
            position: Point::new(20.0, 130.0),
        }),
        bounds,
        mouse::Cursor::Available(Point::new(20.0, 530.0))
    ));
    assert!(activation.is_none());

    assert!(!text_field_keyboard_activation(
        &mut activation,
        &Event::Touch(touch::Event::FingerLifted {
            id: touch::Finger(1),
            position: Point::new(20.0, 130.0),
        }),
        bounds,
        mouse::Cursor::Available(Point::new(20.0, 530.0))
    ));
}

#[test]
fn text_field_visible_keyboard_activation_rejects_touch_without_visible_bounds() {
    let mut activation = Some(TextFieldTouchActivation::new(
        touch::Finger(1),
        Point::new(20.0, 130.0),
    ));

    assert!(!text_field_visible_keyboard_activation(
        &mut activation,
        &Event::Touch(touch::Event::FingerPressed {
            id: touch::Finger(1),
            position: Point::new(20.0, 130.0),
        }),
        None,
        mouse::Cursor::Unavailable
    ));
    assert!(activation.is_none());
}

#[test]
fn text_field_inner_touch_handling_delays_inside_press() {
    let bounds = Rectangle::new(Point::new(10.0, 120.0), Size::new(100.0, 48.0));

    assert_eq!(
        text_field_inner_touch_handling(
            true,
            &Event::Touch(touch::Event::FingerPressed {
                id: touch::Finger(1),
                position: Point::new(20.0, 130.0),
            }),
            bounds,
            mouse::Cursor::Available(Point::new(20.0, 130.0)),
            None,
            false,
        ),
        TextFieldInnerTouchHandling::Suppress
    );
}

#[test]
fn text_field_inner_touch_handling_forwards_outside_press() {
    let bounds = Rectangle::new(Point::new(10.0, 120.0), Size::new(100.0, 48.0));

    assert_eq!(
        text_field_inner_touch_handling(
            true,
            &Event::Touch(touch::Event::FingerPressed {
                id: touch::Finger(1),
                position: Point::new(200.0, 130.0),
            }),
            bounds,
            mouse::Cursor::Available(Point::new(200.0, 130.0)),
            None,
            false,
        ),
        TextFieldInnerTouchHandling::Forward
    );
}

#[test]
fn text_field_inner_touch_handling_confirms_tap_on_release() {
    let bounds = Rectangle::new(Point::new(10.0, 120.0), Size::new(100.0, 48.0));
    let activation = Some(TextFieldTouchActivation::new(
        touch::Finger(1),
        Point::new(20.0, 130.0),
    ));

    assert_eq!(
        text_field_inner_touch_handling(
            true,
            &Event::Touch(touch::Event::FingerLifted {
                id: touch::Finger(1),
                position: Point::new(20.0, 130.0),
            }),
            bounds,
            mouse::Cursor::Unavailable,
            activation,
            true,
        ),
        TextFieldInnerTouchHandling::ConfirmedTap
    );
}

#[test]
fn text_field_inner_touch_handling_suppresses_pending_scroll() {
    let bounds = Rectangle::new(Point::new(10.0, 120.0), Size::new(100.0, 48.0));
    let activation = Some(TextFieldTouchActivation::new(
        touch::Finger(1),
        Point::new(20.0, 130.0),
    ));

    assert_eq!(
        text_field_inner_touch_handling(
            true,
            &Event::Touch(touch::Event::FingerMoved {
                id: touch::Finger(1),
                position: Point::new(20.0, 140.0),
            }),
            bounds,
            mouse::Cursor::Unavailable,
            activation,
            false,
        ),
        TextFieldInnerTouchHandling::Suppress
    );
}

#[test]
fn text_field_inner_touch_handling_suppresses_touch_without_visible_bounds() {
    assert_eq!(
        text_field_inner_touch_handling_for_visible_bounds(
            true,
            &Event::Touch(touch::Event::FingerPressed {
                id: touch::Finger(1),
                position: Point::new(20.0, 130.0),
            }),
            None,
            mouse::Cursor::Unavailable,
            None,
            false,
        ),
        TextFieldInnerTouchHandling::Suppress
    );
}

#[test]
fn release_is_over_accepts_touch_lift_positions_without_cursor() {
    let bounds = Rectangle::new(Point::new(10.0, 20.0), Size::new(100.0, 48.0));

    assert!(release_is_over(
        &Event::Touch(touch::Event::FingerLifted {
            id: touch::Finger(1),
            position: Point::new(20.0, 30.0),
        }),
        bounds,
        mouse::Cursor::Unavailable
    ));
    assert!(!release_is_over(
        &Event::Touch(touch::Event::FingerLifted {
            id: touch::Finger(1),
            position: Point::new(200.0, 300.0),
        }),
        bounds,
        mouse::Cursor::Unavailable
    ));
    assert!(!release_is_over(
        &Event::Touch(touch::Event::FingerLost {
            id: touch::Finger(1),
            position: Point::new(20.0, 30.0),
        }),
        bounds,
        mouse::Cursor::Unavailable
    ));
}

#[test]
fn release_is_over_prefers_translated_cursor_for_touch() {
    let bounds = Rectangle::new(Point::new(10.0, 120.0), Size::new(100.0, 48.0));

    assert!(release_is_over(
        &Event::Touch(touch::Event::FingerLifted {
            id: touch::Finger(1),
            position: Point::new(20.0, 30.0),
        }),
        bounds,
        mouse::Cursor::Available(Point::new(20.0, 130.0))
    ));
    assert!(!release_is_over(
        &Event::Touch(touch::Event::FingerLifted {
            id: touch::Finger(1),
            position: Point::new(20.0, 130.0),
        }),
        bounds,
        mouse::Cursor::Available(Point::new(20.0, 30.0))
    ));
    assert!(!release_is_over(
        &Event::Touch(touch::Event::FingerLifted {
            id: touch::Finger(1),
            position: Point::new(20.0, 130.0),
        }),
        bounds,
        mouse::Cursor::Levitating(Point::new(20.0, 30.0))
    ));
}

#[test]
fn release_is_over_uses_mouse_cursor_for_mouse_release() {
    let bounds = Rectangle::new(Point::new(10.0, 20.0), Size::new(100.0, 48.0));
    let event = Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left));

    assert!(release_is_over(
        &event,
        bounds,
        mouse::Cursor::Available(Point::new(20.0, 30.0))
    ));
    assert!(!release_is_over(
        &event,
        bounds,
        mouse::Cursor::Available(Point::new(200.0, 300.0))
    ));
}

#[test]
fn selection_control_hit_bounds_expand_small_icon_to_touch_target() {
    let content = Rectangle::new(Point::new(20.0, 100.0), Size::new(120.0, 18.0));
    let control = Rectangle::new(Point::new(20.0, 100.0), Size::new(18.0, 18.0));
    let bounds = selection_control_hit_bounds_from_rects(
        content,
        control,
        tokens::component::checkbox::STATE_LAYER_SIZE,
    );

    assert_eq!(
        bounds,
        Rectangle::new(Point::new(9.0, 89.0), Size::new(131.0, 40.0))
    );
}

#[test]
fn selection_control_hit_bounds_keep_radio_target_height() {
    let content = Rectangle::new(Point::new(16.0, 80.0), Size::new(160.0, 20.0));
    let control = Rectangle::new(Point::new(16.0, 80.0), Size::new(20.0, 20.0));
    let bounds = selection_control_hit_bounds_from_rects(
        content,
        control,
        tokens::component::radio::TARGET_SIZE,
    );

    assert_eq!(
        bounds,
        Rectangle::new(Point::new(2.0, 66.0), Size::new(174.0, 48.0))
    );
}

#[test]
fn selection_control_hit_bounds_cover_switch_state_layer_edges() {
    let content = Rectangle::new(Point::new(24.0, 48.0), Size::new(168.0, 32.0));
    let control = Rectangle::new(
        Point::new(24.0, 48.0),
        Size::new(
            tokens::component::switch::TRACK_WIDTH,
            tokens::component::switch::TRACK_HEIGHT,
        ),
    );
    let bounds = selection_control_hit_bounds_from_rects(
        content,
        control,
        tokens::component::switch::STATE_LAYER_SIZE,
    );

    assert_eq!(
        bounds,
        Rectangle::new(Point::new(20.0, 44.0), Size::new(172.0, 40.0))
    );
}

#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "windows")))]
#[test]
fn ime_caret_suppression_is_enabled_on_desktop_composition_platforms() {
    assert!(should_suppress_ime_caret());
}

#[cfg(any(target_arch = "wasm32", target_os = "android", target_os = "windows"))]
#[test]
fn ime_caret_suppression_is_disabled_on_platform_owned_ime_caret() {
    assert!(!should_suppress_ime_caret());
}

#[test]
fn material_button_constructors_compile_to_elements() {
    use button::{ButtonVariant, ChipVariant, FabSize, FabVariant, IconButtonVariant};

    let _: TestElement<'_> = button::button("Filled", ButtonVariant::Filled)
        .on_press(Message::Pressed)
        .into();
    let _: TestElement<'_> = button::action(
        button::button("Filled", ButtonVariant::Filled),
        Message::Pressed,
    );
    let _: TestElement<'_> = button::optional_action(
        button::button("Maybe", ButtonVariant::Filled),
        Some(Message::Pressed),
    );
    let _: Vec<TestElement<'_>> = button::enabled_actions(
        true,
        Message::Pressed,
        [
            button::button("One", ButtonVariant::Filled),
            button::button("Two", ButtonVariant::Text),
        ],
    );
    let _: TestElement<'_> = button::button("Elevated", ButtonVariant::Elevated)
        .on_press(Message::Pressed)
        .into();
    let _: TestElement<'_> = button::button("Tonal", ButtonVariant::FilledTonal)
        .on_press(Message::Pressed)
        .into();
    let _: TestElement<'_> = button::action(
        button::button("Outlined", ButtonVariant::Outlined),
        Message::Pressed,
    );
    let _: TestElement<'_> = button::action(
        button::button("Text", ButtonVariant::Text),
        Message::Pressed,
    );
    let _: TestElement<'_> = button::icon_button("add", IconButtonVariant::Standard)
        .on_press(Message::Pressed)
        .into();
    let _: TestElement<'_> = button::icon_button("add", IconButtonVariant::Filled)
        .on_press(Message::Pressed)
        .into();
    let _: TestElement<'_> = button::icon_button("add", IconButtonVariant::FilledTonal)
        .on_press(Message::Pressed)
        .into();
    let _: TestElement<'_> = button::icon_button("add", IconButtonVariant::Outlined)
        .on_press(Message::Pressed)
        .into();
    let _: TestElement<'_> = button::fab("add", FabVariant::Primary, FabSize::Standard)
        .on_press(Message::Pressed)
        .into();
    let _: TestElement<'_> = button::action(
        button::fab("add", FabVariant::Primary, FabSize::Standard),
        Message::Pressed,
    );
    let _: TestElement<'_> = button::fab("add", FabVariant::Primary, FabSize::Small)
        .on_press(Message::Pressed)
        .into();
    let _: TestElement<'_> = button::fab("add", FabVariant::Primary, FabSize::Large)
        .on_press(Message::Pressed)
        .into();
    let _: TestElement<'_> = button::fab("add", FabVariant::Secondary, FabSize::Standard)
        .on_press(Message::Pressed)
        .into();
    let _: TestElement<'_> = button::fab("add", FabVariant::Tertiary, FabSize::Standard)
        .on_press(Message::Pressed)
        .into();
    let _: TestElement<'_> = button::fab("add", FabVariant::Surface, FabSize::Standard)
        .on_press(Message::Pressed)
        .into();
    let _: TestElement<'_> = button::fab("add", FabVariant::Surface, FabSize::Small)
        .on_press(Message::Pressed)
        .into();
    let _: TestElement<'_> = button::fab("add", FabVariant::Surface, FabSize::Large)
        .on_press(Message::Pressed)
        .into();
    let _: TestElement<'_> = button::extended_fab("Create", FabVariant::Primary)
        .on_press(Message::Pressed)
        .into();
    let _: TestElement<'_> = button::extended_fab_with_icon("add", "Create", FabVariant::Primary)
        .on_press(Message::Pressed)
        .into();
    let _: TestElement<'_> = button::extended_fab("Share", FabVariant::Secondary)
        .on_press(Message::Pressed)
        .into();
    let _: TestElement<'_> = button::extended_fab_with_icon("add", "Add", FabVariant::Tertiary)
        .on_press(Message::Pressed)
        .into();
    let _: TestElement<'_> = button::extended_fab("Reroute", FabVariant::Surface)
        .on_press(Message::Pressed)
        .into();
    let _: TestElement<'_> = button::chip("Assist", ChipVariant::Assist)
        .on_press(Message::Pressed)
        .into();
    let _: TestElement<'_> = button::chip("Elevated assist", ChipVariant::ElevatedAssist)
        .on_press(Message::Pressed)
        .into();
    let _: TestElement<'_> = button::chip("Suggestion", ChipVariant::Suggestion)
        .on_press(Message::Pressed)
        .into();
    let _: TestElement<'_> = button::chip("Elevated suggestion", ChipVariant::ElevatedSuggestion)
        .on_press(Message::Pressed)
        .into();
    let _: TestElement<'_> = button::chip("Filter", ChipVariant::Filter)
        .on_press(Message::Pressed)
        .into();
    let _: TestElement<'_> = button::chip("Selected filter", ChipVariant::SelectedFilter)
        .on_press(Message::Pressed)
        .into();
    let _: TestElement<'_> = button::chip("Input", ChipVariant::Input)
        .on_press(Message::Pressed)
        .into();
    let _: TestElement<'_> = button::chip("Selected input", ChipVariant::SelectedInput)
        .on_press(Message::Pressed)
        .into();
}

#[test]
fn material_badge_constructors_compile_to_elements() {
    let _: TestElement<'_> = badge::small().into();
    let _: TestElement<'_> = badge::large("3").into();
    let _: TestElement<'_> = badge::large("99+").into();
}

#[test]
fn material_container_constructors_compile_to_elements() {
    let surface = Text::new("Surface");
    let _: TestElement<'_> = container::surface_container_high(surface).into();
}

#[test]
fn material_toolbar_constructors_compile_to_elements() {
    let _: TestElement<'_> = toolbar::docked(toolbar::icon_actions([
        ("arrow_back", Message::Pressed),
        ("add", Message::Pressed),
    ]))
    .into();
    let _: TestElement<'_> = toolbar::docked_vibrant(toolbar::vibrant_icon_actions([
        ("edit", Message::Pressed),
        ("delete", Message::Pressed),
    ]))
    .into();
    let _: TestElement<'_> = toolbar::floating(toolbar::icon_actions([
        ("format_bold", Message::Pressed),
        ("format_italic", Message::Pressed),
    ]))
    .into();
    let _: TestElement<'_> = toolbar::floating_vibrant([
        toolbar::selected_vibrant_icon_action("format_bold", Message::Pressed),
        toolbar::vibrant_icon_action("format_italic", Message::Pressed),
    ])
    .into();
    let _: TestElement<'_> =
        toolbar::vertical_floating(toolbar::icon_actions([("undo", Message::Pressed)])).into();
    let _: TestElement<'_> = toolbar::vertical_floating_vibrant(toolbar::vibrant_icon_actions([(
        "redo",
        Message::Pressed,
    )]))
    .into();
    let floating = toolbar::floating(toolbar::icon_actions([("share", Message::Pressed)]));
    let fab = button::fab(
        "add",
        button::FabVariant::Primary,
        button::FabSize::Standard,
    )
    .on_press(Message::Pressed);
    let _: TestElement<'_> = toolbar::floating_with_fab(floating, fab).into();

    let picker_state = theme_picker::State::new();
    let content: TestElement<'_> = Text::new("Theme picker content").into();
    let _: TestElement<'_> = theme_picker::floating_over(
        content,
        &picker_state,
        theme_picker::MaterialColor::Purple,
        theme_picker::FLOATING_MARGIN,
        Message::Pressed,
        |_| Message::Pressed,
    );
}

#[test]
fn material_card_constructors_compile_to_elements() {
    let elevated = Text::new("Elevated card");
    let _: TestElement<'_> = card::elevated(elevated).into();

    let filled = Text::new("Filled card");
    let _: TestElement<'_> = card::filled(filled).into();

    let outlined = Text::new("Outlined card");
    let _: TestElement<'_> = card::outlined(outlined).into();

    let legacy = Text::new("Legacy path");
    let _: TestElement<'_> = container::outlined_card(legacy).into();
}

#[test]
fn material_dialog_constructors_compile_to_elements() {
    let content: TestElement<'_> = Text::new("Custom dialog content").into();
    let _: TestElement<'_> = dialog::basic(content).into();

    let actions: [TestElement<'_>; 2] = [
        dialog::action("Cancel").on_press(Message::Pressed).into(),
        dialog::action("OK").on_press(Message::Pressed).into(),
    ];
    let _: TestElement<'_> =
        dialog::alert("Title", "Supporting text", dialog::actions(actions)).into();

    let actions: [TestElement<'_>; 2] = [
        dialog::action_button("Cancel", Message::Pressed),
        dialog::action_button("OK", Message::Pressed),
    ];
    let _: TestElement<'_> =
        dialog::alert("Title", "Supporting text", dialog::actions(actions)).into();

    let actions: [TestElement<'_>; 1] = [dialog::action("Done").on_press(Message::Pressed).into()];
    let _: TestElement<'_> =
        dialog::alert_with_icon("info", "Title", "Supporting text", dialog::actions(actions))
            .into();

    let actions: [TestElement<'_>; 2] = [
        dialog::action_button_alpha("Cancel", Message::Pressed, 0.5),
        dialog::action_button_alpha("OK", Message::Pressed, 0.5),
    ];
    let _: TestElement<'_> = dialog::alert_with_icon_alpha(
        "info",
        "Title",
        "Supporting text",
        dialog::actions(actions),
        0.5,
    )
    .into();

    let content: TestElement<'_> = Text::new("Scrim content").into();
    let _: TestElement<'_> = dialog::scrim(content).into();

    let content: TestElement<'_> = Text::new("Animated scrim content").into();
    let _: TestElement<'_> = dialog::scrim_alpha(content, 0.5).into();

    let content: TestElement<'_> = Text::new("Modal dialog").into();
    let _: TestElement<'_> = dialog::modal_overlay(content).into();

    let content: TestElement<'_> = Text::new("Modal layer dialog").into();
    let _: TestElement<'_> = dialog::modal_layer(content);

    let content: TestElement<'_> = Text::new("Page content").into();
    let dialog_content: TestElement<'_> = Text::new("Dialog content").into();
    let _: TestElement<'_> = dialog::modal(content, dialog_content);

    let transition = dialog::Transition::default();
    let content: TestElement<'_> = Text::new("Page content").into();
    let dialog_content: TestElement<'_> = Text::new("Animated dialog content").into();
    let _: TestElement<'_> = dialog::modal_animated(
        content,
        &transition,
        iced_widget::core::time::Instant::now(),
        dialog_content,
    );
}

#[test]
fn material_app_bar_constructors_compile_to_elements() {
    let _: TestElement<'_> = app_bar::icon_action("info", Message::Pressed);
    let _: Vec<TestElement<'_>> =
        app_bar::icon_actions([("search", Message::Pressed), ("info", Message::Pressed)]);

    let leading = app_bar::icon_button("menu")
        .on_press(Message::Pressed)
        .into();
    let actions = [app_bar::icon_button("search")
        .on_press(Message::Pressed)
        .into()];
    let small = app_bar::small("Small", Some(leading), actions);
    let _: TestElement<'_> = app_bar::with_status_bar(small).into();

    let leading = app_bar::icon_button("menu")
        .on_press(Message::Pressed)
        .into();
    let actions = [app_bar::icon_button("search")
        .on_press(Message::Pressed)
        .into()];
    let _: TestElement<'_> = app_bar::medium("Medium", Some(leading), actions).into();

    let actions = [app_bar::icon_button("info")
        .on_press(Message::Pressed)
        .into()];
    let fab = button::fab(
        "add",
        button::FabVariant::Primary,
        button::FabSize::Standard,
    )
    .on_press(Message::Pressed)
    .into();
    let _: TestElement<'_> = app_bar::bottom(actions, Some(fab)).into();
}

#[test]
fn material_navigation_constructors_compile_to_elements() {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Page {
        First,
        Second,
    }

    let destinations = [
        navigation::Destination::new(Page::First, "1", "First"),
        navigation::Destination::new(Page::Second, "2", "Second"),
    ];
    let selection = navigation::Selection::new(Page::First);

    let _: TestElement<'_> =
        navigation::navigation_bar(&destinations, selection, |_| Message::Pressed).into();
    let _: TestElement<'_> =
        navigation::navigation_rail(&destinations, selection, |_| Message::Pressed).into();
    let _: TestElement<'_> =
        navigation::navigation_rail_fitting_content(&destinations, selection, |_| Message::Pressed)
            .into();
    let _: TestElement<'_> = navigation::navigation_rail_with_header(
        &destinations,
        selection,
        |_| Message::Pressed,
        Text::new("Header"),
    )
    .into();
    let _: TestElement<'_> = navigation::navigation_rail_with_header_fitting_content(
        &destinations,
        selection,
        |_| Message::Pressed,
        Text::new("Header"),
    )
    .into();
    let _: TestElement<'_> = navigation::navigation_rail_with_menu(
        &destinations,
        selection,
        |_| Message::Pressed,
        Message::Pressed,
    )
    .into();
    let _: TestElement<'_> = navigation::navigation_rail_with_menu_fitting_content(
        &destinations,
        selection,
        |_| Message::Pressed,
        Message::Pressed,
    )
    .into();
    let _: TestElement<'_> = navigation::navigation_rail_expanded_with_menu(
        "Navigation",
        &destinations,
        selection,
        |_| Message::Pressed,
        Message::Pressed,
    )
    .into();
    let _: TestElement<'_> = navigation::navigation_rail_expanded_with_menu_fitting_content(
        "Navigation",
        &destinations,
        selection,
        |_| Message::Pressed,
        Message::Pressed,
    )
    .into();
    let _: TestElement<'_> = navigation::navigation_rail_expanded_with_menu_at_width(
        "Navigation",
        &destinations,
        selection,
        |_| Message::Pressed,
        Message::Pressed,
        220.0,
    )
    .into();
    let _: TestElement<'_> =
        navigation::navigation_drawer("Navigation", &destinations, selection, |_| Message::Pressed)
            .into();
    let _: TestElement<'_> = navigation::navigation_drawer_at_width(
        "Navigation",
        &destinations,
        selection,
        |_| Message::Pressed,
        240.0,
    )
    .into();
    let _: TestElement<'_> = navigation::navigation_drawer_with_menu(
        "Navigation",
        &destinations,
        selection,
        |_| Message::Pressed,
        Message::Pressed,
    )
    .into();
    let _: TestElement<'_> = navigation::navigation_drawer_with_menu_at_width(
        "Navigation",
        &destinations,
        selection,
        |_| Message::Pressed,
        Message::Pressed,
        240.0,
    )
    .into();
    let content = Text::new("Navigation suite content");
    let _: TestElement<'_> = navigation::navigation_suite(
        1080.0,
        980.0,
        &destinations,
        selection,
        |_| Message::Pressed,
        content,
    );
    let content = Text::new("Navigation suite menu content");
    let state = navigation::NavigationState::new(Page::First);
    let _: TestElement<'_> = navigation::navigation_suite_with_menu(
        "Navigation",
        1080.0,
        980.0,
        &destinations,
        &state,
        |_| Message::Pressed,
        Message::Pressed,
        content,
    );
    let content = Text::new("Navigation suite builder content");
    let _: TestElement<'_> = navigation::suite(&destinations, &state)
        .window_size(Size::new(1080.0, 980.0))
        .with_menu("Navigation", Message::Pressed)
        .view(|_| Message::Pressed, content);
}

#[test]
fn material_tabs_constructors_compile_to_elements() {
    let _: TestElement<'_> = tabs::bar([
        tabs::primary_icon_label("input", "Inputs", true)
            .on_press(Message::Pressed)
            .into(),
        tabs::primary_inline_icon_label("tune", "Controls", false)
            .on_press(Message::Pressed)
            .into(),
        tabs::primary_label("Feedback", false)
            .on_press(Message::Pressed)
            .into(),
    ])
    .into();

    let _: TestElement<'_> = tabs::bar([
        tabs::secondary_icon_label("info", "Overview", true)
            .on_press(Message::Pressed)
            .into(),
        tabs::secondary_label("Details", false)
            .on_press(Message::Pressed)
            .into(),
    ])
    .into();

    let tab_state = tabs::State::new(0);
    let _: TestElement<'_> = tabs::animated_bar(
        tabs::Variant::Primary,
        3,
        &tab_state,
        [
            tabs::primary_label_for_animated_bar("Inputs", true)
                .on_press(Message::Pressed)
                .into(),
            tabs::primary_inline_icon_label_for_animated_bar("tune", "Controls", false)
                .on_press(Message::Pressed)
                .into(),
            tabs::primary_icon_label_action_for_animated_bar(
                "info",
                "Feedback",
                false,
                Message::Pressed,
            ),
        ],
    )
    .into();

    let _: TestElement<'_> = tabs::animated_primary_icon_label_bar(
        &tab_state,
        [
            ("input", "Inputs", Message::Pressed),
            ("tune", "Controls", Message::Pressed),
        ],
    )
    .into();

    let _: TestElement<'_> = tabs::animated_bar(
        tabs::Variant::Secondary,
        1,
        &tab_state,
        [tabs::secondary_label_action_for_animated_bar(
            "Details",
            true,
            Message::Pressed,
        )],
    )
    .into();

    let _: TestElement<'_> = tabs::animated_secondary_label_bar(
        &tab_state,
        [("Details", Message::Pressed), ("History", Message::Pressed)],
    )
    .into();
}

#[test]
fn material_segmented_button_constructors_compile_to_elements() {
    use segmented_button::SegmentPosition;

    let segment_state = segmented_button::State::new(0);
    let _: TestElement<'_> = segmented_button::group([
        segmented_button::animated_selectable_label_action(
            "List",
            1.0,
            SegmentPosition::First,
            Message::Pressed,
        ),
        segmented_button::animated_selectable_label_action(
            "Grid",
            0.0,
            SegmentPosition::Last,
            Message::Pressed,
        ),
    ])
    .into();
    let _: TestElement<'_> =
        segmented_button::group(segmented_button::animated_selectable_label_actions(
            &segment_state,
            [
                ("List", Message::Pressed),
                ("Grid", Message::Pressed),
                ("Map", Message::Pressed),
            ],
        ))
        .into();
}

#[test]
fn material_select_constructor_compiles_to_element() {
    let options = ["Assist", "Suggestion", "Filter"];
    let _: TestElement<'_> = select::outlined(options, Some("Assist"), |_| Message::Pressed)
        .placeholder("Choose")
        .label("Chip type")
        .into();
}

#[test]
fn material_select_defaults_to_fill_width() {
    let options = ["Assist", "Suggestion", "Filter"];
    let select: select::Select<'_, _, _, _, Message, iced_widget::Renderer> =
        select::outlined(options, Some("Assist"), |_| Message::Pressed);

    assert_eq!(
        Widget::<Message, Theme, iced_widget::Renderer>::size(&select).width,
        Length::Fill
    );
}

#[test]
fn material_combobox_constructor_compiles_to_element() {
    let selected = "Assist";
    let options =
        combobox::State::with_selection(vec!["Assist", "Suggestion", "Filter"], Some(&selected));
    let _: TestElement<'_> =
        combobox::outlined(&options, "Choose", Some(&selected), |_| Message::Pressed)
            .label("Chip type")
            .into();
}

#[test]
fn material_combobox_with_input_constructor_compiles_to_element() {
    let options = combobox::State::new(vec!["Assist", "Suggestion", "Filter"]);
    let _: TestElement<'_> =
        combobox::outlined_with_input(&options, "Choose", "xxx", None, |_| Message::Pressed)
            .on_input(|_| Message::Pressed)
            .into();
}

#[test]
fn material_list_item_constructors_compile_to_elements() {
    let _: TestElement<'_> = list::one_line("Single line").into();
    let _: TestElement<'_> = list::one_line_with_leading_icon("info", "With leading icon").into();
    let _: TestElement<'_> = list::two_line("Two line", "Supporting text").into();
    let _: TestElement<'_> = list::two_line_with_trailing("Inventory", "In stock", "42").into();
    let _: TestElement<'_> =
        list::three_line("Three line", "Supporting text", "Second supporting line").into();
    let _: TestElement<'_> = list::group([
        list::one_line("First").into(),
        list::one_line("Second").into(),
    ])
    .into();
}

#[test]
fn material_sheet_constructors_compile_to_elements() {
    let content: TestElement<'_> = Text::new("Sheet content").into();
    let _: TestElement<'_> = sheet::modal_bottom(content).into();

    let content: TestElement<'_> = Text::new("Sheet content").into();
    let _: TestElement<'_> = sheet::standard_bottom(content).into();

    let content: TestElement<'_> = Text::new("Bottom sheet content").into();
    let _: TestElement<'_> = sheet::bottom_content(content).into();

    let content: TestElement<'_> = Text::new("Scrim content").into();
    let _: TestElement<'_> = sheet::scrim(content).into();

    let content: TestElement<'_> = Text::new("Modal overlay content").into();
    let _: TestElement<'_> = sheet::modal_overlay(content).into();

    let content: TestElement<'_> = Text::new("Modal side sheet content").into();
    let _: TestElement<'_> = sheet::modal_side(content).into();

    let content: TestElement<'_> = Text::new("Left modal side sheet content").into();
    let _: TestElement<'_> = sheet::modal_side_on(sheet::Side::Left, content).into();

    let content: TestElement<'_> = Text::new("Detached modal side sheet content").into();
    let _: TestElement<'_> = sheet::detached_modal_side(content).into();

    let content: TestElement<'_> = Text::new("Detached left modal side sheet content").into();
    let _: TestElement<'_> = sheet::detached_modal_side_on(sheet::Side::Left, content).into();

    let content: TestElement<'_> = Text::new("Standard side sheet content").into();
    let _: TestElement<'_> = sheet::standard_side(content).into();

    let content: TestElement<'_> = Text::new("Side sheet content").into();
    let _: TestElement<'_> = sheet::side_content(content).into();

    let content: TestElement<'_> = Text::new("Left standard side sheet content").into();
    let _: TestElement<'_> = sheet::standard_side_on(sheet::Side::Left, content).into();

    let content: TestElement<'_> = Text::new("Detached standard side sheet content").into();
    let _: TestElement<'_> = sheet::detached_standard_side(content).into();

    let content: TestElement<'_> = Text::new("Detached left standard side sheet content").into();
    let _: TestElement<'_> = sheet::detached_standard_side_on(sheet::Side::Left, content).into();

    let content: TestElement<'_> = Text::new("Side scrim content").into();
    let _: TestElement<'_> = sheet::side_scrim(content).into();

    let content: TestElement<'_> = Text::new("Side overlay content").into();
    let _: TestElement<'_> = sheet::modal_side_overlay(sheet::Side::Right, content).into();
}

#[test]
fn material_snackbar_constructors_compile_to_elements() {
    let _: TestElement<'_> = snackbar::single_line("Archived").into();
    let _: TestElement<'_> = snackbar::single_line_with_action(
        "Archived",
        snackbar::action_button("Undo", Message::Pressed),
    )
    .into();
    let _: TestElement<'_> = snackbar::two_line("Longer message").into();
    let _: TestElement<'_> = snackbar::two_line_with_action(
        "Longer message",
        snackbar::icon_action_button("close", Message::Pressed),
    )
    .into();

    let content: TestElement<'_> = Text::new("Content").into();
    let _: TestElement<'_> = snackbar::host_single_line_with_action(
        content,
        &snackbar::Transition::default(),
        iced_widget::core::time::Instant::now(),
        "Archived",
        "Undo",
        Message::Pressed,
    );
}

#[test]
fn material_data_table_constructors_compile_to_elements() {
    #[derive(Clone)]
    struct Row {
        name: &'static str,
        quantity: u32,
    }

    let rows = [Row {
        name: "Assist",
        quantity: 24,
    }];
    let columns: [iced_widget::table::Column<'_, '_, Row, Message, Theme, iced_widget::Renderer>;
        2] = [
        data_table::weighted_column(2, "Name", |row: Row| row.name),
        data_table::compact_numeric_column("Quantity", |row: Row| row.quantity.to_string()),
    ];

    let _: TestElement<'_> = data_table::standard(columns, rows).into();
    assert_eq!(data_table::COMPACT_NUMERIC_COLUMN_WIDTH, 88.0);
}

#[test]
fn material_data_table_cells_use_token_heights() {
    let header: Container<'_, Message, Theme, iced_widget::Renderer> =
        data_table::header_cell("Name");
    let body: Container<'_, Message, Theme, iced_widget::Renderer> =
        data_table::body_cell("Assist");

    assert_eq!(
        Widget::<Message, Theme, iced_widget::Renderer>::size(&header),
        Size {
            width: Length::Shrink,
            height: Length::Fixed(tokens::component::data_table::HEADER_CONTAINER_HEIGHT),
        }
    );
    assert_eq!(
        Widget::<Message, Theme, iced_widget::Renderer>::size(&body),
        Size {
            width: Length::Shrink,
            height: Length::Fixed(tokens::component::data_table::ROW_ITEM_CONTAINER_HEIGHT),
        }
    );
}

#[test]
fn material_slider_and_progress_constructors_compile_to_elements() {
    let _: TestElement<'_> = slider::continuous(0.0..=100.0, 42.0, |_| Message::Pressed).into();
    let _: TestElement<'_> =
        progress_bar::linear(progress_bar::LinearProgressMode::determinate(0.42, 0.0)).into();
    let _: TestElement<'_> = progress_bar::linear(
        progress_bar::LinearProgressMode::four_color_indeterminate(0.0),
    )
    .into();
    let _: TestElement<'_> =
        progress_bar::loading(progress_bar::LoadingIndicatorMode::indeterminate(0.0)).into();
    let _: TestElement<'_> = progress_bar::loading(
        progress_bar::LoadingIndicatorMode::contained_indeterminate(0.0),
    )
    .into();
    let _: TestElement<'_> =
        progress_bar::loading(progress_bar::LoadingIndicatorMode::determinate(0.42)).into();
    let _: TestElement<'_> = progress_bar::loading(
        progress_bar::LoadingIndicatorMode::contained_determinate(0.42),
    )
    .into();
}

#[test]
fn material_rule_constructors_compile_to_elements() {
    let _: TestElement<'_> = rule::horizontal_full_width().into();
    let _: TestElement<'_> = rule::horizontal_inset().into();
    let _: TestElement<'_> = rule::vertical_full_height().into();
    let _: TestElement<'_> = rule::vertical_inset().into();
}

#[test]
fn material_tooltip_constructor_compiles_to_element() {
    let content = button::chip("Hint", button::ChipVariant::Assist).on_press(Message::Pressed);
    let _: TestElement<'_> =
        tooltip::plain(content, "Material 3 plain tooltip", tooltip::Position::Top).into();

    let content = button::chip("Rich", button::ChipVariant::Assist).on_press(Message::Pressed);
    let _: TestElement<'_> =
        tooltip::rich(content, "Material 3 rich tooltip", tooltip::Position::Top).into();

    let content =
        button::chip("Rich title", button::ChipVariant::Assist).on_press(Message::Pressed);
    let _: TestElement<'_> = tooltip::rich_with_title(
        content,
        "Rich tooltip",
        "Supporting text",
        tooltip::Position::Top,
    )
    .into();

    let content =
        button::chip("Rich action", button::ChipVariant::Assist).on_press(Message::Pressed);
    let action = tooltip::rich_action_button("Action", Message::Pressed);
    let _: TestElement<'_> = tooltip::rich_with_title_action(
        content,
        "Rich tooltip",
        "Supporting text",
        action,
        tooltip::Position::Top,
    )
    .into();
}

#[test]
fn material_selection_constructors_compile_to_elements() {
    let _: TestElement<'_> = checkbox::standard(true, "Enable actions", toggled);
    let _: TestElement<'_> = toggler::standard(true, "Dark theme", toggled);
    let _: TestElement<'_> = toggler::standard_with_origin(true, "Dark theme", toggled_at);
    let controller = theme_picker::ThemeController::new(theme_picker::MaterialColor::Purple, true);
    let _: TestElement<'_> = controller.dark_mode_switch("Dark theme", |_| Message::Toggled);
}

#[test]
fn material_text_input_constructor_compiles_to_element() {
    let _: TestElement<'_> = text_input::outlined("Write a note", "value")
        .on_input(|_| Message::Pressed)
        .into();
}

#[test]
fn material_search_constructors_compile_to_elements() {
    let results = Text::<Theme, iced_widget::Renderer>::new("Results");

    let _: TestElement<'_> =
        search::bar("Search Material components", "query", |_| Message::Pressed).into();
    let _: TestElement<'_> = search::bar_with_trailing(
        "Search Material components",
        "query",
        |_| Message::Pressed,
        Some(
            button::icon_button("close", button::IconButtonVariant::Standard)
                .on_press(Message::Pressed)
                .into(),
        ),
    )
    .into();
    let _: TestElement<'_> =
        search::docked_view("Search components", "query", |_| Message::Pressed, results).into();

    let results = Text::<Theme, iced_widget::Renderer>::new("Results");
    let _: TestElement<'_> =
        search::full_screen_view("Search components", "query", |_| Message::Pressed, results)
            .into();
}

#[test]
fn material_picker_constructors_compile_to_elements() {
    let date = picker::DatePickerState::new(picker::Date::new(2026, 7, 4));
    let range = picker::DateRangePickerState::new(
        picker::Date::new(2026, 7, 4),
        picker::Date::new(2026, 7, 10),
    );
    let time = picker::TimePickerState::new(14, 30, false);

    let _: TestElement<'_> = picker::date_picker(&date, |_| Message::Pressed);
    let _: TestElement<'_> =
        picker::date_picker_with_mode_toggle(&date, |_| Message::Pressed, false);
    let _: TestElement<'_> = picker::date_picker_dialog(
        &date,
        |_| Message::Pressed,
        picker::date_picker_dialog_actions([
            dialog::action_button("Cancel", Message::Pressed),
            dialog::action_button("OK", Message::Pressed),
        ]),
    );
    let _: TestElement<'_> = picker::date_picker_dialog_with_mode_toggle(
        &date,
        |_| Message::Pressed,
        false,
        picker::date_picker_dialog_actions([
            dialog::action_button("Cancel", Message::Pressed),
            dialog::action_button("OK", Message::Pressed),
        ]),
    );
    let _: TestElement<'_> = picker::date_range_picker(&range, |_| Message::Pressed);
    let _: TestElement<'_> =
        picker::date_range_picker_with_mode_toggle(&range, |_| Message::Pressed, false);
    let _: TestElement<'_> = picker::date_range_picker_dialog(
        &range,
        |_| Message::Pressed,
        picker::date_picker_dialog_actions([
            dialog::action_button("Cancel", Message::Pressed),
            dialog::action_button("OK", Message::Pressed),
        ]),
    );
    let _: TestElement<'_> = picker::date_range_picker_dialog_with_mode_toggle(
        &range,
        |_| Message::Pressed,
        false,
        picker::date_picker_dialog_actions([
            dialog::action_button("Cancel", Message::Pressed),
            dialog::action_button("OK", Message::Pressed),
        ]),
    );
    let _: TestElement<'_> = picker::time_picker(&time, |_| Message::Pressed);
    let _: TestElement<'_> = picker::time_picker_dialog(
        &time,
        picker::TimePickerDisplayMode::Picker,
        |_| Message::Pressed,
        Some(Message::Toggled),
        dialog::actions([
            dialog::action_button("Cancel", Message::Pressed),
            dialog::action_button("OK", Message::Pressed),
        ]),
    );
    let _: TestElement<'_> = picker::rich_time_picker_dialog(
        &time,
        picker::TimePickerDisplayMode::Scroll,
        |_| Message::Pressed,
        Some(Message::Toggled),
        dialog::actions([
            dialog::action_button("Cancel", Message::Pressed),
            dialog::action_button("OK", Message::Pressed),
        ]),
    );
}

#[test]
fn material_text_editor_constructor_compiles_to_element() {
    let content = text_editor::Content::with_text("value");
    let _: TestElement<'_> = text_editor::outlined(&content)
        .placeholder("Write a note")
        .on_action(|_| Message::Pressed)
        .into();
    let _: TestElement<'_> = text_editor::outlined_area(&content)
        .placeholder("Write details")
        .on_action(|_| Message::Pressed)
        .into();
    assert_eq!(
        text_editor::OUTLINED_AREA_HEIGHT,
        tokens::component::text_field::CONTAINER_HEIGHT * 2.0
    );
}

#[test]
fn checkbox_checkmark_svg_uses_m3_rect_mark_geometry() {
    let svg = String::from_utf8(checkbox_checkmark_svg(1.0)).expect("valid svg");

    assert!(svg.contains("viewBox=\"0 0 18 18\""));
    assert!(svg.contains("scale(1 -1) translate(7 -14) rotate(45)"));
    assert!(svg.contains("width=\"2\" height=\"5.656854\""));
    assert!(svg.contains("width=\"11.313708\" height=\"2\""));
}
