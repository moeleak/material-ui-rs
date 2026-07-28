use super::*;

type TestParagraph = <iced_widget::Renderer as core_text::Renderer>::Paragraph;

fn assert_close(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() < 0.001,
        "expected {expected}, got {actual}",
    );
}

fn assert_rectangle_close(actual: Rectangle, expected: Rectangle) {
    assert_close(actual.x, expected.x);
    assert_close(actual.y, expected.y);
    assert_close(actual.width, expected.width);
    assert_close(actual.height, expected.height);
}

#[test]
fn animated_scalar_retargets_from_last_advanced_value() {
    let start = Instant::now();
    let mut scalar = AnimatedScalar::new(0.0);

    scalar.set_target(1.0, start, duration_ms(100), tokens::motion::EASING_LINEAR);
    assert!(scalar.advance(start + duration_ms(50)));
    scalar.set_target(
        0.0,
        start + duration_ms(80),
        duration_ms(100),
        tokens::motion::EASING_LINEAR,
    );

    assert_close(scalar.value, 0.8);
    assert_eq!(scalar.to, 0.0);

    assert!(scalar.advance(start + duration_ms(130)));
    assert_close(scalar.value, 0.4);
}

#[test]
fn text_field_floating_label_notch_uses_label_width_with_padding() {
    let field = Rectangle {
        x: 10.0,
        y: 20.0,
        width: 160.0,
        height: 56.0,
    };

    let notch = text_field_floating_label_notch(field, 26.0, 80.0, 80.0, 1.0).unwrap();

    assert_rectangle_close(
        notch,
        Rectangle {
            x: 26.0,
            y: 20.0,
            width: 84.0,
            height: 0.0,
        },
    );
}

#[test]
fn text_field_floating_label_notch_expands_from_label_start() {
    let field = Rectangle {
        x: 10.0,
        y: 20.0,
        width: 160.0,
        height: 56.0,
    };

    let notch = text_field_floating_label_notch(field, 26.0, 80.0, 80.0, 0.5).unwrap();

    assert_rectangle_close(
        notch,
        Rectangle {
            x: 26.0,
            y: 20.0,
            width: 42.0,
            height: 0.0,
        },
    );
}

#[test]
fn text_field_floating_label_notch_follows_interpolated_text_width() {
    let field = Rectangle {
        x: 10.0,
        y: 20.0,
        width: 200.0,
        height: 56.0,
    };

    let notch = text_field_floating_label_notch(field, 26.0, 120.0, 80.0, 0.5).unwrap();

    assert_rectangle_close(
        notch,
        Rectangle {
            x: 26.0,
            y: 20.0,
            width: 52.0,
            height: 0.0,
        },
    );
}

#[test]
fn text_field_floating_label_notch_stays_hidden_before_float() {
    let field = Rectangle {
        x: 10.0,
        y: 20.0,
        width: 160.0,
        height: 56.0,
    };

    assert!(text_field_floating_label_notch(field, 26.0, 80.0, 80.0, 0.0).is_none());
}

#[test]
fn text_field_state_tracks_active_ime_preedit() {
    let mut state = TextFieldState::<TestParagraph>::new(false);

    assert!(!state.ime_preedit_active);
    assert!(state.set_ime_preedit("pin yin"));
    assert!(state.ime_preedit_active);
    assert!(!state.set_ime_preedit("more"));
    assert!(state.clear_ime_preedit());
    assert!(!state.ime_preedit_active);
    assert!(!state.clear_ime_preedit());
}
