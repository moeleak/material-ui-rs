use super::*;

#[derive(Default)]
struct HostProbe {
    initialize: bool,
    scroll_y: Option<f32>,
    focused: bool,
    action_bounds: Option<Rectangle>,
}

impl widget::Operation for HostProbe {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn widget::Operation)) {
        operate(self);
    }

    fn scrollable(
        &mut self,
        _id: Option<&widget::Id>,
        _bounds: Rectangle,
        _content_bounds: Rectangle,
        translation: Vector,
        state: &mut dyn widget::operation::Scrollable,
    ) {
        if self.initialize {
            state.scroll_to(widget::operation::scrollable::AbsoluteOffset {
                x: None,
                y: Some(240.0),
            });
        }
        self.scroll_y = Some(translation.y);
    }

    fn focusable(
        &mut self,
        _id: Option<&widget::Id>,
        _bounds: Rectangle,
        state: &mut dyn widget::operation::Focusable,
    ) {
        if self.initialize {
            state.focus();
        }
        self.focused = state.is_focused();
    }

    fn text(&mut self, _id: Option<&widget::Id>, bounds: Rectangle, text: &str) {
        if text == "Undo" {
            self.action_bounds = Some(bounds);
        }
    }
}

fn rebuild_scroll_host(
    tree: &mut Tree,
    transition: &Transition,
    now: Instant,
    wrapped: bool,
    initialize: bool,
) -> HostProbe {
    let renderer = iced_widget::Renderer::Secondary(iced_tiny_skia::Renderer::new(
        fonts::ROBOTO,
        iced_widget::core::Pixels(16.0),
    ));
    let content = iced_widget::Scrollable::new(
        iced_widget::Column::new()
            .push(iced_widget::TextInput::new("Note", "draft").on_input(|_| ()))
            .push(iced_widget::Space::new().height(1200)),
    )
    .width(Length::Fill)
    .height(Length::Fill);
    let content: Element<'_, (), Theme, iced_widget::Renderer> = if wrapped {
        // Android's page has a stateless column around its tabs and scroller.
        iced_widget::Column::new()
            .push(iced_widget::Space::new().height(48))
            .push(content)
            .into()
    } else {
        content.into()
    };
    let mut host = host_with(
        content,
        transition,
        now,
        "Saved",
        "Undo",
        (),
        HostOptions::default(),
    );
    tree.diff(host.as_widget());
    let node = host.as_widget_mut().layout(
        tree,
        &renderer,
        &layout::Limits::new(Size::ZERO, Size::new(360.0, 400.0)),
    );
    let mut probe = HostProbe {
        initialize,
        ..HostProbe::default()
    };
    host.as_widget_mut()
        .operate(tree, Layout::new(&node), &renderer, &mut probe);
    if initialize {
        probe.initialize = false;
        host.as_widget_mut()
            .operate(tree, Layout::new(&node), &renderer, &mut probe);
    }
    probe
}

#[test]
fn snackbar_visibility_preserves_scroll_position_and_input_focus() {
    for wrapped in [false, true] {
        let mut tree = Tree::empty();
        let mut transition = Transition::default();
        let mut now = Instant::now();
        let probe = rebuild_scroll_host(&mut tree, &transition, now, wrapped, true);
        assert_eq!(probe.scroll_y, Some(240.0));
        assert!(probe.focused);

        let verify = |tree: &mut Tree, transition: &Transition, now| {
            let probe = rebuild_scroll_host(tree, transition, now, wrapped, false);
            assert_eq!(
                probe.scroll_y,
                Some(240.0),
                "wrapped={wrapped}, phase={:?}",
                transition.phase()
            );
            assert!(probe.focused, "input focus was reset");
            assert_eq!(probe.action_bounds.is_some(), transition.is_active());
        };
        for manual_dismiss in [true, false] {
            transition.show(now);
            verify(&mut tree, &transition, now);
            now += duration_ms(300);
            let _ = transition.advance(now);
            assert_eq!(transition.phase(), TransitionPhase::Shown);
            verify(&mut tree, &transition, now);
            transition.show(now); // Repeated presses refresh the timeout.
            verify(&mut tree, &transition, now);
            if manual_dismiss {
                transition.dismiss(now);
            } else {
                now += duration_ms(tokens::component::snackbar::LONG_DURATION_MS);
                let _ = transition.advance(now);
            }
            assert_eq!(transition.phase(), TransitionPhase::Dismissing);
            verify(&mut tree, &transition, now);
            now += duration_ms(300);
            let _ = transition.advance(now);
            assert_eq!(transition.phase(), TransitionPhase::Hidden);
            verify(&mut tree, &transition, now);
        }
    }
}

#[test]
fn snackbar_host_keeps_background_and_action_touch_targets_working() {
    use iced_widget::core::{Point, touch};
    let renderer = iced_widget::Renderer::Secondary(iced_tiny_skia::Renderer::new(
        fonts::ROBOTO,
        iced_widget::core::Pixels(16.0),
    ));
    let now = Instant::now();
    for visible in [false, true] {
        let mut transition = Transition::default();
        if visible {
            transition.show(now);
            let _ = transition.advance(now + duration_ms(300));
        }
        let content = Button::new(iced_widget::Space::new().width(100).height(40)).on_press(false);
        let mut host = host(
            content,
            &transition,
            now + duration_ms(300),
            "Saved",
            "Undo",
            true,
        );
        let mut tree = Tree::new(host.as_widget());
        let viewport = Rectangle::with_size(Size::new(360.0, 400.0));
        let node = host.as_widget_mut().layout(
            &mut tree,
            &renderer,
            &layout::Limits::new(Size::ZERO, viewport.size()),
        );
        let mut probe = HostProbe::default();
        host.as_widget_mut()
            .operate(&mut tree, Layout::new(&node), &renderer, &mut probe);
        let mut targets = vec![Point::new(20.0, 20.0)];
        if visible {
            targets.push(probe.action_bounds.unwrap().center());
        } else {
            assert!(probe.action_bounds.is_none());
        }
        let mut messages = Vec::new();
        for position in targets {
            let id = touch::Finger(1);
            for event in [
                touch::Event::FingerPressed { id, position },
                touch::Event::FingerLifted { id, position },
            ] {
                host.as_widget_mut().update(
                    &mut tree,
                    &Event::Touch(event),
                    Layout::new(&node),
                    mouse::Cursor::Available(position),
                    &renderer,
                    &mut iced_widget::core::clipboard::Null,
                    &mut Shell::new(&mut messages),
                    &viewport,
                );
            }
        }
        assert_eq!(
            messages,
            if visible {
                vec![false, true]
            } else {
                vec![false]
            }
        );
    }
}

#[test]
fn snackbar_host_defaults_to_material_bottom_margin() {
    assert_eq!(
        HostOptions::default().bottom_margin,
        tokens::component::snackbar::BOTTOM_MARGIN
    );
}

#[test]
fn snackbar_host_preserves_material_margin_above_fab_like_compose_scaffold() {
    let fab_height = tokens::component::fab::CONTAINER_HEIGHT;
    let fab_bottom_margin = 24.0;
    let options = HostOptions::default().above_fab(fab_height, fab_bottom_margin);

    assert_eq!(
        options.bottom_margin,
        fab_bottom_margin + fab_height + tokens::component::snackbar::BOTTOM_MARGIN
    );
}

#[test]
fn snackbar_transition_matches_android_slide_and_content_fade_timing() {
    let start = Instant::now();
    let mut transition = Transition::default();
    let hidden_distance = tokens::component::snackbar::WITH_SINGLE_LINE_CONTAINER_HEIGHT
        + tokens::component::snackbar::BOTTOM_MARGIN;

    transition.show(start);

    assert_eq!(transition.phase(), TransitionPhase::Showing);
    assert_eq!(
        transition.translation_y(start, hidden_distance),
        hidden_distance
    );
    assert_eq!(transition.content_alpha(start), 0.0);
    assert_eq!(
        transition.content_alpha(start + Duration::from_millis(70)),
        0.0
    );

    let halfway = start + Duration::from_millis(125);
    assert!(transition.translation_y(halfway, hidden_distance) < hidden_distance);
    assert!(transition.translation_y(halfway, hidden_distance) > 0.0);
    assert!(transition.content_alpha(halfway) > 0.0);
    assert!(transition.content_alpha(halfway) < 1.0);

    let shown = start
        + Duration::from_millis(u64::from(
            tokens::component::snackbar::SLIDE_ANIMATION_DURATION_MS,
        ));
    assert_eq!(transition.translation_y(shown, hidden_distance), 0.0);
    assert_eq!(transition.content_alpha(shown), 1.0);

    assert!(transition.advance(shown));
    assert_eq!(transition.phase(), TransitionPhase::Shown);
}

#[test]
fn snackbar_transition_auto_dismisses_after_android_long_duration() {
    let start = Instant::now();
    let mut transition = Transition::default();

    transition.show(start);
    let shown = start
        + Duration::from_millis(u64::from(
            tokens::component::snackbar::SLIDE_ANIMATION_DURATION_MS,
        ));
    let _ = transition.advance(shown);
    let _ = transition.advance(
        shown + Duration::from_millis(u64::from(tokens::component::snackbar::LONG_DURATION_MS)),
    );

    assert_eq!(transition.phase(), TransitionPhase::Dismissing);
    assert_eq!(
        transition.content_alpha(
            shown
                + Duration::from_millis(u64::from(tokens::component::snackbar::LONG_DURATION_MS,))
                + Duration::from_millis(u64::from(
                    tokens::component::snackbar::CONTENT_FADE_ANIMATION_DURATION_MS,
                )),
        ),
        0.0
    );

    let hidden = shown
        + Duration::from_millis(u64::from(tokens::component::snackbar::LONG_DURATION_MS))
        + Duration::from_millis(u64::from(
            tokens::component::snackbar::SLIDE_ANIMATION_DURATION_MS,
        ));
    assert!(!transition.advance(hidden));
    assert_eq!(transition.phase(), TransitionPhase::Hidden);
}

#[test]
fn snackbar_container_uses_inverse_surface_tokens_in_light_and_dark() {
    for theme in [Theme::Light, Theme::Dark] {
        let colors = theme.colors();
        let style = container_style(&theme);

        assert_eq!(
            style.background,
            Some(Background::Color(colors.inverse.inverse_surface))
        );
        assert_eq!(style.text_color, Some(colors.inverse.inverse_surface_text));
        assert_eq!(
            style.border.radius.top_left,
            tokens::component::snackbar::CONTAINER_SHAPE
        );
        assert_eq!(style.shadow.offset.y, 4.0);
        assert_eq!(style.shadow.blur_radius, 8.0);
    }
}

#[test]
fn snackbar_actions_use_inverse_tokens_in_light_and_dark() {
    for theme in [Theme::Light, Theme::Dark] {
        let colors = theme.colors();
        let action = action_style_alpha(&theme, Status::Active, 1.0);
        let icon = icon_action_style(&theme, Status::Active);

        assert_eq!(action.text_color, colors.inverse.inverse_primary);
        assert_eq!(action.background, None);
        assert_eq!(icon.text_color, colors.inverse.inverse_surface_text);
        assert_eq!(icon.background, None);
    }
}
