#[derive(Debug, Clone)]
enum PickerTouchMessage {
    Time(TimePickerAction),
    Scrolled(f32),
}

struct PickerTouchHarness<'a> {
    widget: Element<'a, PickerTouchMessage, Theme, iced_widget::Renderer>,
    tree: Tree,
    node: layout::Node,
    renderer: iced_widget::Renderer,
    viewport: Rectangle,
    messages: Vec<PickerTouchMessage>,
}

impl<'a> PickerTouchHarness<'a> {
    fn new(state: &'a TimePickerState, width: f32) -> Self {
        let mut widget: Element<'_, _, _, iced_widget::Renderer> = Scrollable::new(
            Column::new()
                .push(Space::new().height(60))
                .push(time_picker(state, PickerTouchMessage::Time))
                .push(Space::new().height(900)),
        )
        .width(width)
        .height(600)
        .on_scroll(|viewport| PickerTouchMessage::Scrolled(viewport.absolute_offset().y))
        .into();
        let mut tree = Tree::new(widget.as_widget());
        let renderer = picker_test_renderer();
        let viewport = Rectangle::with_size(Size::new(width, 600.0));
        let node = widget.as_widget_mut().layout(
            &mut tree,
            &renderer,
            &layout::Limits::new(Size::ZERO, viewport.size()),
        );
        Self {
            widget,
            tree,
            node,
            renderer,
            viewport,
            messages: Vec::new(),
        }
    }

    fn bounds(&self, size: Size) -> Rectangle {
        fn find(layout: Layout<'_>, size: Size) -> Option<Rectangle> {
            if layout.bounds().size() == size {
                Some(layout.bounds())
            } else {
                layout.children().find_map(|child| find(child, size))
            }
        }
        find(Layout::new(&self.node), size).expect("picker part should be present")
    }

    fn event(&mut self, event: touch::Event, position: Point) {
        self.widget.as_widget_mut().update(
            &mut self.tree,
            &Event::Touch(event),
            Layout::new(&self.node),
            mouse::Cursor::Available(position),
            &self.renderer,
            &mut iced_widget::core::clipboard::Null,
            &mut Shell::new(&mut self.messages),
            &self.viewport,
        );
    }

    fn gesture(&mut self, origin: Point, delta: Vector) {
        let id = touch::Finger(7);
        self.event(
            touch::Event::FingerPressed {
                id,
                position: origin,
            },
            origin,
        );
        for step in 1..=20 {
            let position = origin + delta * (step as f32 / 20.0);
            self.event(touch::Event::FingerMoved { id, position }, position);
        }
        let position = origin + delta;
        self.event(touch::Event::FingerLifted { id, position }, position);
    }

    fn actions(&self) -> Vec<TimePickerAction> {
        self.messages
            .iter()
            .filter_map(|message| match message {
                PickerTouchMessage::Time(action) => Some(action.clone()),
                PickerTouchMessage::Scrolled(_) => None,
            })
            .collect()
    }

    fn did_scroll(&self) -> bool {
        self.messages
            .iter()
            .any(|message| matches!(message, PickerTouchMessage::Scrolled(y) if *y > 0.0))
    }
}

#[test]
fn time_picker_allows_page_swipes_over_the_dial_and_header() {
    let state = TimePickerState::new(12, 0, false);
    for width in [304.0, 328.0] {
        for target in 0..5 {
            let mut picker = PickerTouchHarness::new(&state, width);
            let dial = picker.bounds(Size::new(256.0, 256.0));
            let origin = match target {
                0 => dial.center(),
                1 => Point::new(dial.center_x() + 101.0, dial.center_y()),
                2 => Point::new(dial.x + 4.0, dial.y + 4.0),
                3 => picker.bounds(Size::new(96.0, 80.0)).center(),
                4 => {
                    let size = if width < 328.0 {
                        Size::new(108.0, 38.0)
                    } else {
                        Size::new(52.0, 40.0)
                    };
                    picker.bounds(size).center()
                }
                _ => unreachable!(),
            };
            picker.gesture(origin, Vector::new(0.0, -80.0));
            assert!(picker.did_scroll(), "width={width}, target={target}");
            assert!(
                picker.actions().is_empty(),
                "a page swipe changed the time: {:?}",
                picker.actions()
            );
        }
    }
}

#[test]
fn page_swipe_crossing_the_clock_never_becomes_a_clock_drag() {
    let state = TimePickerState::new(12, 0, false);
    let mut picker = PickerTouchHarness::new(&state, 328.0);
    let dial = picker.bounds(Size::new(256.0, 256.0));
    picker.gesture(
        Point::new(dial.center_x(), dial.y + dial.height + 40.0),
        Vector::new(0.0, -320.0),
    );
    assert!(picker.did_scroll());
    assert!(picker.actions().is_empty());
}

#[test]
fn clock_touch_tap_selects_and_auto_switches_without_scrolling() {
    let state = TimePickerState::new(12, 0, false);
    let mut picker = PickerTouchHarness::new(&state, 328.0);
    let dial = picker.bounds(Size::new(256.0, 256.0));
    let origin = Point::new(dial.center_x() + 101.0, dial.center_y());
    let id = touch::Finger(7);
    picker.event(
        touch::Event::FingerPressed {
            id,
            position: origin,
        },
        origin,
    );
    assert!(picker.actions().is_empty());
    picker.event(
        touch::Event::FingerLifted {
            id,
            position: origin,
        },
        origin,
    );
    assert_eq!(
        picker.actions(),
        [
            TimePickerAction::SelectHour(3),
            TimePickerAction::SetSelection(TimePickerSelectionMode::Minute)
        ]
    );
    assert!(!picker.did_scroll());
}

#[test]
fn clock_touch_selector_drag_still_changes_time() {
    let state = TimePickerState::new(12, 0, false);
    let mut picker = PickerTouchHarness::new(&state, 328.0);
    let dial = picker.bounds(Size::new(256.0, 256.0));
    let origin = Point::new(dial.center_x(), dial.center_y() - 101.0);
    picker.gesture(origin, Vector::new(101.0, 101.0));
    assert!(!picker.did_scroll());
    assert_eq!(
        picker.actions().first(),
        Some(&TimePickerAction::SelectHour(12))
    );
    assert!(
        picker
            .actions()
            .contains(&TimePickerAction::DragHourAngle(3, pack_angle(0.0)))
    );
    assert_eq!(
        picker.actions().last(),
        Some(&TimePickerAction::SetSelection(
            TimePickerSelectionMode::Minute
        ))
    );
}

#[test]
fn cancelled_clock_touches_do_not_select_or_auto_switch() {
    let state = TimePickerState::new(12, 0, false);
    for on_handle in [false, true] {
        let mut picker = PickerTouchHarness::new(&state, 328.0);
        let dial = picker.bounds(Size::new(256.0, 256.0));
        let origin = if on_handle {
            Point::new(dial.center_x(), dial.center_y() - 101.0)
        } else {
            Point::new(dial.center_x() + 101.0, dial.center_y())
        };
        let id = touch::Finger(7);
        picker.event(
            touch::Event::FingerPressed {
                id,
                position: origin,
            },
            origin,
        );
        picker.messages.clear();
        picker.event(
            touch::Event::FingerLost {
                id,
                position: origin,
            },
            origin,
        );
        if on_handle {
            assert_eq!(picker.actions(), [TimePickerAction::FinishDrag]);
        } else {
            assert!(picker.actions().is_empty());
        }
        picker.messages.clear();
        picker.event(
            touch::Event::FingerLifted {
                id,
                position: origin,
            },
            origin,
        );
        assert!(picker.actions().is_empty());
    }
}

#[test]
fn clock_widget_handles_cursorless_taps_and_ignores_hidden_touches() {
    let state = TimePickerState::new(12, 0, false);
    let renderer = picker_test_renderer();
    for hidden in [false, true] {
        let mut clock = clock_face(&state, |action| action);
        let mut tree = Tree::new(clock.as_widget());
        let node = clock
            .as_widget_mut()
            .layout(
                &mut tree,
                &renderer,
                &layout::Limits::new(Size::ZERO, Size::new(256.0, 256.0)),
            )
            .move_to(Point::new(40.0, 80.0));
        let position = Point::new(40.0 + 229.0, 80.0 + 128.0);
        let viewport = if hidden {
            Rectangle::new(Point::new(40.0, position.y + 1.0), Size::new(256.0, 127.0))
        } else {
            node.bounds()
        };
        let id = touch::Finger(2);
        let mut messages = Vec::new();
        for event in [
            touch::Event::FingerPressed { id, position },
            // A second finger must not take over the pending tap.
            touch::Event::FingerPressed {
                id: touch::Finger(3),
                position: Point::new(168.0, 107.0),
            },
            touch::Event::FingerLost {
                id: touch::Finger(3),
                position,
            },
            touch::Event::FingerPressed {
                id: touch::Finger(4),
                position: Point::ORIGIN,
            },
            touch::Event::FingerLost {
                id: touch::Finger(4),
                position: Point::ORIGIN,
            },
            touch::Event::FingerLifted { id, position },
        ] {
            clock.as_widget_mut().update(
                &mut tree,
                &Event::Touch(event),
                Layout::new(&node),
                mouse::Cursor::Unavailable,
                &renderer,
                &mut iced_widget::core::clipboard::Null,
                &mut Shell::new(&mut messages),
                &viewport,
            );
        }
        if hidden {
            assert!(messages.is_empty());
        } else {
            assert_eq!(
                messages,
                [
                    TimePickerAction::SelectHour(3),
                    TimePickerAction::SetSelection(TimePickerSelectionMode::Minute)
                ]
            );
        }
    }
}
