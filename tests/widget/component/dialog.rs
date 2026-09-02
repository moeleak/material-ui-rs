use iced_widget::core::{Point, Widget, mouse, touch};

use super::*;

#[derive(Debug, Clone)]
enum Message {}

#[test]
fn dialog_container_style_uses_material_tokens() {
    let theme = Theme::Light;
    let colors = theme.colors();
    let style = container_style(&theme);

    assert_eq!(
        style.background,
        Some(Background::Color(colors.surface.container.high))
    );
    assert_eq!(
        style.border.radius.top_left,
        tokens::component::dialog::CONTAINER_SHAPE
    );
    assert_eq!(style.shadow.offset.y, 4.0);
    assert_eq!(style.shadow.blur_radius, 8.0);
}

#[test]
fn dialog_content_styles_use_material_color_roles() {
    let theme = Theme::Light;
    let colors = theme.colors();

    assert_eq!(icon_style(&theme).color, Some(colors.secondary.color));
    assert_eq!(title_style(&theme).color, Some(colors.surface.text));
    assert_eq!(
        supporting_text_style(&theme).color,
        Some(colors.surface.text_variant)
    );
}

#[test]
fn dialog_alpha_styles_scale_material_color_roles() {
    let theme = Theme::Light;
    let colors = theme.colors();
    let alpha = 0.5;

    assert_eq!(
        container_style_alpha(&theme, alpha).background,
        Some(Background::Color(alpha_color(
            colors.surface.container.high,
            alpha
        )))
    );
    assert_eq!(
        icon_style_alpha(&theme, alpha).color,
        Some(alpha_color(colors.secondary.color, alpha))
    );
    assert_eq!(
        title_style_alpha(&theme, alpha).color,
        Some(alpha_color(colors.surface.text, alpha))
    );
    assert_eq!(
        supporting_text_style_alpha(&theme, alpha).color,
        Some(alpha_color(colors.surface.text_variant, alpha))
    );
}

#[test]
fn dialog_title_alignment_follows_icon_presence() {
    assert_eq!(title_alignment(true), alignment::Horizontal::Center);
    assert_eq!(title_alignment(false), alignment::Horizontal::Left);
}

#[test]
fn dialog_title_text_fills_width_for_alignment() {
    let title: Text<'_, Theme, iced_widget::Renderer> =
        title_text("Discard draft?", alignment::Horizontal::Center, 1.0);

    assert_eq!(
        Widget::<Message, Theme, iced_widget::Renderer>::size(&title).width,
        Length::Fill
    );
}

#[test]
fn dialog_scrim_uses_material_scrim_opacity() {
    let theme = Theme::Light;
    let style = scrim_style_alpha(&theme, 1.0);
    let Some(Background::Color(color)) = style.background else {
        panic!("expected solid scrim background");
    };

    assert_eq!(color.a, tokens::component::dialog::SCRIM_OPACITY);
    assert_eq!(style.text_color, Some(theme.colors().surface.text));
}

#[test]
fn dialog_transition_matches_android_platform_timing() {
    let start = Instant::now();
    let mut transition = Transition::default();

    transition.show(start);

    assert_eq!(transition.phase(), TransitionPhase::Showing);
    assert_eq!(
        transition.scale(start),
        tokens::component::dialog::ENTER_SCALE_FROM
    );
    assert_eq!(transition.alpha(start), 0.0);
    assert_eq!(transition.scrim_alpha(start), 0.0);

    let alpha_finished =
        start + duration_ms(tokens::component::dialog::ALPHA_ANIMATION_DURATION_MS);
    assert_eq!(transition.alpha(alpha_finished), 1.0);
    assert!(transition.scale(alpha_finished) < 1.0);

    let shown = start + duration_ms(tokens::component::dialog::SCALE_ANIMATION_DURATION_MS);
    assert_eq!(transition.scale(shown), 1.0);
    assert_eq!(transition.scrim_alpha(shown), 1.0);
    assert!(!transition.advance(shown));
    assert_eq!(transition.phase(), TransitionPhase::Shown);
    assert!(!transition.is_animating());

    transition.dismiss(shown);
    assert_eq!(transition.phase(), TransitionPhase::Dismissing);
    assert_eq!(transition.scale(shown), 1.0);
    assert_eq!(transition.alpha(shown), 1.0);

    let faded = shown + duration_ms(tokens::component::dialog::ALPHA_ANIMATION_DURATION_MS);
    assert_eq!(transition.alpha(faded), 0.0);
    assert!(transition.scale(faded) > tokens::component::dialog::EXIT_SCALE_TO);
    assert_eq!(transition.phase(), TransitionPhase::Dismissing);

    let hidden = shown + duration_ms(tokens::component::dialog::SCALE_ANIMATION_DURATION_MS);
    assert_eq!(
        transition.scale(hidden),
        tokens::component::dialog::EXIT_SCALE_TO
    );
    assert_eq!(transition.alpha(hidden), 0.0);
    assert!(!transition.advance(hidden));
    assert_eq!(transition.phase(), TransitionPhase::Hidden);
}

#[test]
fn android_decelerate_matches_platform_factor_formula() {
    assert_eq!(android_decelerate(0.0, 2.5), 0.0);
    assert_eq!(android_decelerate(1.0, 2.5), 1.0);
    assert!((android_decelerate(0.5, 1.5) - 0.875).abs() < 0.001);
    assert!((android_decelerate(0.5, 2.5) - 0.96875).abs() < 0.001);
}

#[test]
fn dismissing_scaled_dialog_preserves_child_state() {
    let interactive: Element<'_, Message, Theme, iced_widget::Renderer> =
        scaled(Text::new("Dialog"), 1.0, true);
    let dismissing: Element<'_, Message, Theme, iced_widget::Renderer> =
        scaled(Text::new("Dialog"), 1.0, false);

    assert_eq!(interactive.as_widget().tag(), dismissing.as_widget().tag());
    assert_eq!(interactive.as_widget().children().len(), 1);
    assert_eq!(dismissing.as_widget().children().len(), 1);
}

#[test]
fn scaled_dialog_layer_uses_viewport_to_preserve_shadow() {
    let bounds = Rectangle {
        x: 100.0,
        y: 100.0,
        width: 240.0,
        height: 160.0,
    };
    let viewport = Rectangle {
        x: 0.0,
        y: 0.0,
        width: 800.0,
        height: 600.0,
    };

    assert_eq!(scaled_layer_bounds(bounds, &viewport), Some(viewport));
}

#[test]
fn scaled_dialog_transforms_mouse_motion_into_child_coordinates() {
    let transformation = Transformation::translate(100.0, 50.0) * Transformation::scale(0.5);
    let event = Event::Mouse(mouse::Event::CursorMoved {
        position: Point::new(150.0, 100.0),
    });

    assert_eq!(
        transform_pointer_event(&event, transformation.inverse()),
        Some(Event::Mouse(mouse::Event::CursorMoved {
            position: Point::new(100.0, 100.0),
        }))
    );
}

#[test]
fn scaled_dialog_transforms_touch_positions_into_child_coordinates() {
    let transformation = Transformation::translate(100.0, 50.0) * Transformation::scale(0.5);
    let event = Event::Touch(touch::Event::FingerPressed {
        id: touch::Finger(7),
        position: Point::new(150.0, 100.0),
    });

    assert_eq!(
        transform_pointer_event(&event, transformation.inverse()),
        Some(Event::Touch(touch::Event::FingerPressed {
            id: touch::Finger(7),
            position: Point::new(100.0, 100.0),
        }))
    );
}
