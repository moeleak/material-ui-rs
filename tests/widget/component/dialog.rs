use iced_widget::core::{Point, Widget, mouse, touch};

use super::*;

#[derive(Debug, Clone, PartialEq)]
enum Message {
    Account(String),
    Password(String),
    Remember(bool),
    Cancel,
    Confirm,
    BackgroundPressed,
}

fn login_form() -> Element<'static, Message, Theme, iced_widget::Renderer> {
    content(
        "Sign in",
        Column::new()
            .spacing(16.0)
            .push(
                crate::widget::text_input::outlined("Account", "a")
                    .id("dialog-account")
                    .on_input(Message::Account),
            )
            .push(
                crate::widget::text_input::outlined("Password", "")
                    .id("dialog-password")
                    .on_input(Message::Password),
            )
            .push(crate::widget::checkbox::standard(
                false,
                "Remember password",
                Message::Remember,
            )),
        actions([
            action_button("Cancel", Message::Cancel),
            action_button("Log in", Message::Confirm),
        ]),
    )
    .into()
}

fn test_modal(insets: Padding) -> Element<'static, Message, Theme, iced_widget::Renderer> {
    let start = Instant::now();
    let mut transition = Transition::default();
    transition.show(start);
    modal_animated_with(
        iced_widget::MouseArea::new(Space::new().width(Length::Fill).height(Length::Fill))
            .on_press(Message::BackgroundPressed),
        &transition,
        start + duration_ms(300),
        login_form(),
        ModalOptions::default()
            .insets(insets)
            .margin(tokens::component::dialog::WINDOW_MARGIN),
    )
}

fn test_renderer() -> iced_widget::Renderer {
    iced_widget::Renderer::Secondary(iced_tiny_skia::Renderer::new(
        crate::fonts::ROBOTO,
        iced_widget::core::Pixels(16.0),
    ))
}

fn dialog_layout<'a>(tree: &Tree, mut layout: Layout<'a>) -> Option<Layout<'a>> {
    if tree.tag == tree::Tag::of::<DialogContentTag>() {
        // Containers forward the child's tree tag but add their own layout node.
        while layout.children().count() == 1 {
            layout = layout.children().next().unwrap();
        }
        Some(layout)
    } else {
        tree.children
            .iter()
            .zip(layout.children())
            .find_map(|(tree, layout)| dialog_layout(tree, layout))
    }
}

#[test]
fn narrow_dialogs_reserve_actions_and_scroll_the_form_above_the_keyboard() {
    let renderer = test_renderer();
    for width in [240.0, 320.0, 360.0, 412.0, 900.0] {
        for (height, bottom) in [(640.0, 24.0), (640.0, 340.0), (300.0, 0.0)] {
            let insets = Padding {
                top: 24.0,
                bottom,
                ..Padding::ZERO
            };
            let mut modal = test_modal(insets);
            let mut tree = Tree::new(modal.as_widget());
            let node = modal.as_widget_mut().layout(
                &mut tree,
                &renderer,
                &layout::Limits::new(Size::ZERO, Size::new(width, height)),
            );
            let root = Layout::new(&node);
            let content = dialog_layout(&tree, root).unwrap();
            let mut children = content.children();
            let header = children.next().unwrap();
            let body = children.next().unwrap();
            let actions = children.next().unwrap();

            assert_eq!(root.bounds().size(), Size::new(width, height));
            let scrim = root.children().nth(1).unwrap().children().next().unwrap();
            assert_eq!(
                scrim.bounds(),
                root.bounds(),
                "scrim must remain full-window"
            );
            assert_eq!(
                content.bounds().width,
                (width - 2.0 * tokens::component::dialog::WINDOW_MARGIN)
                    .min(tokens::component::dialog::CONTAINER_MAX_WIDTH)
                    - 48.0,
                "the form must not shrink to its padding"
            );
            assert!(header.bounds().height >= 24.0);
            assert!(
                content.bounds().x >= insets.left + tokens::component::dialog::WINDOW_MARGIN + 24.0
            );
            assert!(actions.bounds().height >= 40.0);
            assert!(actions.bounds().y + actions.bounds().height <= height - bottom - 24.0);
            assert!(body.bounds().height > 0.0);
            let scroll_content = body.children().next().unwrap().bounds();
            if height - bottom <= 300.0 {
                assert!(
                    scroll_content.height > body.bounds().height,
                    "short dialogs must scroll the form"
                );
            } else {
                assert_eq!(
                    scroll_content.height,
                    body.bounds().height,
                    "ordinary dialogs retain their natural height"
                );
            }
        }
    }
}

#[test]
fn dialog_content_does_not_collapse_under_a_shrink_wrapper() {
    let renderer = test_renderer();
    let mut form: Element<'_, Message, Theme, iced_widget::Renderer> = Container::new(login_form())
        .width(Length::Shrink)
        .height(Length::Shrink)
        .into();
    let mut tree = Tree::new(form.as_widget());
    let node = form.as_widget_mut().layout(
        &mut tree,
        &renderer,
        &layout::Limits::new(Size::ZERO, Size::new(360.0, 300.0)),
    );
    assert_eq!(node.size().width, 360.0);
    assert!(node.size().height <= 300.0);
    let content = dialog_layout(&tree, Layout::new(&node)).unwrap();
    assert_eq!(content.bounds().width, 312.0);
}

#[derive(Default)]
struct FormProbe {
    account_focused: bool,
    scroll_translation: Vector,
    scroll_to_end: bool,
}

impl widget::Operation for FormProbe {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn widget::Operation)) {
        operate(self);
    }

    fn focusable(
        &mut self,
        id: Option<&widget::Id>,
        _bounds: Rectangle,
        state: &mut dyn widget::operation::focusable::Focusable,
    ) {
        if id == Some(&widget::Id::new("dialog-account")) {
            self.account_focused = state.is_focused();
        }
    }

    fn scrollable(
        &mut self,
        _id: Option<&widget::Id>,
        _bounds: Rectangle,
        _content_bounds: Rectangle,
        translation: Vector,
        state: &mut dyn widget::operation::scrollable::Scrollable,
    ) {
        self.scroll_translation = translation;
        if self.scroll_to_end {
            state.snap_to(widget::operation::scrollable::RelativeOffset {
                x: None,
                y: Some(1.0),
            });
        }
    }
}

#[test]
fn keyboard_resize_keeps_focus_and_scrollable_dialog_events_reach_children() {
    use iced_widget::core::keyboard::{self, Key, Location, Modifiers, key};
    let renderer = test_renderer();
    let size = Size::new(360.0, 640.0);
    let limits = layout::Limits::new(Size::ZERO, size);
    let mut modal = test_modal(Padding {
        top: 24.0,
        bottom: 24.0,
        ..Padding::ZERO
    });
    let mut tree = Tree::new(modal.as_widget());
    let node = modal.as_widget_mut().layout(&mut tree, &renderer, &limits);
    modal.as_widget_mut().operate(
        &mut tree,
        Layout::new(&node),
        &renderer,
        &mut widget::operation::focusable::focus::<()>(widget::Id::new("dialog-account")),
    );

    let mut modal = test_modal(Padding {
        top: 24.0,
        bottom: 340.0,
        ..Padding::ZERO
    });
    tree.diff(modal.as_widget());
    let node = modal.as_widget_mut().layout(&mut tree, &renderer, &limits);
    let mut probe = FormProbe::default();
    modal
        .as_widget_mut()
        .operate(&mut tree, Layout::new(&node), &renderer, &mut probe);
    assert!(
        probe.account_focused,
        "IME insets must not recreate the focused form"
    );

    let mut messages = Vec::new();
    let viewport = Rectangle::with_size(size);
    let key = Event::Keyboard(keyboard::Event::KeyPressed {
        key: Key::Character("z".into()),
        modified_key: Key::Character("z".into()),
        physical_key: key::Physical::Code(key::Code::KeyZ),
        location: Location::Standard,
        modifiers: Modifiers::default(),
        text: Some("z".into()),
        repeat: false,
    });
    modal.as_widget_mut().update(
        &mut tree,
        &key,
        Layout::new(&node),
        mouse::Cursor::Unavailable,
        &renderer,
        &mut iced_widget::core::clipboard::Null,
        &mut Shell::new(&mut messages),
        &viewport,
    );
    assert!(
        messages
            .iter()
            .any(|message| matches!(message, Message::Account(value) if value.contains('z')))
    );

    probe.scroll_to_end = true;
    modal
        .as_widget_mut()
        .operate(&mut tree, Layout::new(&node), &renderer, &mut probe);
    probe.scroll_to_end = false;
    modal
        .as_widget_mut()
        .operate(&mut tree, Layout::new(&node), &renderer, &mut probe);
    assert!(probe.scroll_translation.y > 0.0);

    let content = dialog_layout(&tree, Layout::new(&node)).unwrap();
    let body = content.children().nth(1).unwrap();
    let checkbox = body
        .children()
        .next()
        .unwrap()
        .children()
        .next()
        .unwrap()
        .children()
        .nth(2)
        .unwrap()
        .bounds();
    let checkbox_position = checkbox.center() - probe.scroll_translation;
    assert!(body.bounds().contains(checkbox_position));
    let confirm = content
        .children()
        .nth(2)
        .unwrap()
        .children()
        .next()
        .unwrap()
        .children()
        .nth(1)
        .unwrap()
        .bounds()
        .center();
    for position in [checkbox_position, confirm, Point::new(2.0, 2.0)] {
        for event in [
            Event::Touch(touch::Event::FingerPressed {
                id: touch::Finger(0),
                position,
            }),
            Event::Touch(touch::Event::FingerLifted {
                id: touch::Finger(0),
                position,
            }),
        ] {
            modal.as_widget_mut().update(
                &mut tree,
                &event,
                Layout::new(&node),
                mouse::Cursor::Available(position),
                &renderer,
                &mut iced_widget::core::clipboard::Null,
                &mut Shell::new(&mut messages),
                &viewport,
            );
        }
    }
    assert!(messages.contains(&Message::Remember(true)));
    assert!(messages.contains(&Message::Confirm));
    assert!(!messages.contains(&Message::BackgroundPressed));
}

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
