use crate::{Theme, widget};
use iced_widget::core::widget::Tree;
use iced_widget::core::{
    Element, Event, Font, Layout, Pixels, Point, Rectangle, Shell, Size, layout, mouse, touch,
    window,
};

struct Harness {
    element: Element<'static, bool, Theme, iced_widget::Renderer>,
    renderer: iced_widget::Renderer,
    tree: Tree,
    node: layout::Node,
    messages: Vec<bool>,
}

impl Harness {
    fn new(kind: usize) -> Self {
        let mut element: Element<'static, bool, Theme, iced_widget::Renderer> = match kind {
            0 => widget::button::Button::new(iced_widget::Space::new().width(120).height(48))
                .padding(0)
                .width(120)
                .height(48)
                .on_press(true)
                .into(),
            1 => widget::checkbox::Checkbox::new(false)
                .label("Choice")
                .width(120)
                .on_toggle(|value| value)
                .into(),
            2 => widget::radio::Radio::new("Choice", true, Some(false), |value| value)
                .width(120)
                .into(),
            3 => widget::toggler::Toggler::new(false)
                .label("Choice")
                .width(120)
                .on_toggle(|value| value)
                .into(),
            _ => unreachable!(),
        };
        let renderer: iced_widget::Renderer = iced_widget::renderer::fallback::Renderer::Secondary(
            iced_tiny_skia::Renderer::new(Font::DEFAULT, Pixels(16.0)),
        );
        let mut tree = Tree::new(element.as_widget());
        let node = element.as_widget_mut().layout(
            &mut tree,
            &renderer,
            &layout::Limits::new(Size::ZERO, Size::new(120.0, 48.0)),
        );
        Self {
            element,
            renderer,
            tree,
            node,
            messages: Vec::new(),
        }
    }

    fn send(&mut self, event: Event, cursor: mouse::Cursor) {
        self.element.as_widget_mut().update(
            &mut self.tree,
            &event,
            Layout::new(&self.node),
            cursor,
            &self.renderer,
            &mut iced_widget::core::clipboard::Null,
            &mut Shell::new(&mut self.messages),
            &Rectangle::with_size(Size::new(360.0, 800.0)),
        );
    }

    fn mouse(&mut self, event: mouse::Event, position: Point) {
        self.send(Event::Mouse(event), mouse::Cursor::Available(position));
    }

    fn down(&mut self, point: Point) {
        self.mouse(mouse::Event::CursorMoved { position: point }, point);
        self.mouse(mouse::Event::ButtonPressed(mouse::Button::Left), point);
    }

    fn up(&mut self, point: Point) {
        self.mouse(mouse::Event::ButtonReleased(mouse::Button::Left), point);
    }
}

#[test]
fn click_widgets_allow_taps_and_small_mouse_motion() {
    for kind in 0..4 {
        for end in [Point::new(12.0, 12.0), Point::new(16.0, 15.0)] {
            let mut h = Harness::new(kind);
            h.down(Point::new(12.0, 12.0));
            h.mouse(mouse::Event::CursorMoved { position: end }, end);
            h.up(end);
            assert_eq!(h.messages, [true], "widget {kind}");
        }
    }
}

#[test]
fn click_widgets_cancel_mouse_drags_even_if_the_pointer_returns() {
    let start = Point::new(12.0, 12.0);
    for kind in 0..4 {
        for far in [Point::new(42.0, 12.0), Point::new(180.0, 12.0)] {
            let mut h = Harness::new(kind);
            h.down(start);
            h.mouse(mouse::Event::CursorMoved { position: far }, far);
            h.mouse(mouse::Event::CursorMoved { position: start }, start);
            h.up(start);
            assert!(h.messages.is_empty(), "widget {kind}");
            h.down(start);
            h.up(start);
            assert_eq!(h.messages, [true], "next tap on widget {kind}");
        }
    }
}

#[test]
fn click_widgets_use_raw_mouse_motion_in_a_translated_event_batch() {
    let raw = Point::new(12.0, 12.0);
    let local = Point::new(12.0, 212.0);
    for kind in 0..4 {
        let mut h = Harness::new(kind);
        h.node = h.node.move_to(Point::new(0.0, 200.0));
        h.mouse(mouse::Event::CursorMoved { position: raw }, local);
        h.mouse(mouse::Event::ButtonPressed(mouse::Button::Left), local);
        h.mouse(
            mouse::Event::CursorMoved {
                position: Point::new(90.0, 12.0),
            },
            local,
        );
        h.mouse(mouse::Event::CursorMoved { position: raw }, local);
        h.up(local);
        assert!(h.messages.is_empty(), "widget {kind}");
    }
}

#[test]
fn click_widgets_check_release_distance_and_clear_lost_mouse_presses() {
    let start = Point::new(12.0, 12.0);
    for kind in 0..4 {
        let mut h = Harness::new(kind);
        h.down(start);
        h.up(Point::new(42.0, 12.0));
        assert!(h.messages.is_empty(), "release without move widget {kind}");
        for loss in [
            Event::Window(window::Event::Unfocused),
            Event::Mouse(mouse::Event::CursorLeft),
        ] {
            h.down(start);
            h.send(loss, mouse::Cursor::Unavailable);
            h.up(start);
            assert!(h.messages.is_empty(), "lost mouse widget {kind}");
        }
    }
}

#[test]
fn click_widgets_preserve_touch_ownership_and_cancel_touch_scrolls() {
    let start = Point::new(12.0, 12.0);
    for kind in 0..4 {
        for drag in [false, true] {
            let mut h = Harness::new(kind);
            h.send(
                Event::Touch(touch::Event::FingerPressed {
                    id: touch::Finger(0),
                    position: start,
                }),
                mouse::Cursor::Unavailable,
            );
            h.send(
                Event::Touch(touch::Event::FingerLifted {
                    id: touch::Finger(1),
                    position: start,
                }),
                mouse::Cursor::Unavailable,
            );
            assert!(h.messages.is_empty());
            if drag {
                h.send(
                    Event::Touch(touch::Event::FingerMoved {
                        id: touch::Finger(0),
                        position: Point::new(42.0, 12.0),
                    }),
                    mouse::Cursor::Unavailable,
                );
            }
            h.send(
                Event::Touch(touch::Event::FingerLifted {
                    id: touch::Finger(0),
                    position: start,
                }),
                mouse::Cursor::Unavailable,
            );
            assert_eq!(h.messages.len(), usize::from(!drag), "widget {kind}");
        }
    }
}

#[test]
fn scrollable_can_take_over_a_touch_started_on_click_widgets() {
    for kind in 0..4 {
        let mut h = Harness::new(kind);
        let child = std::mem::replace(&mut h.element, iced_widget::Space::new().into());
        h.element = iced_widget::Scrollable::new(
            iced_widget::Column::new()
                .push(iced_widget::Space::new().height(60))
                .push(child)
                .push(iced_widget::Space::new().height(400)),
        )
        .width(120)
        .height(180)
        .on_scroll(|_| false)
        .into();
        h.tree = Tree::new(h.element.as_widget());
        h.node = h.element.as_widget_mut().layout(
            &mut h.tree,
            &h.renderer,
            &layout::Limits::new(Size::ZERO, Size::new(120.0, 180.0)),
        );
        let start = Point::new(12.0, 72.0);
        let end = Point::new(12.0, 25.0);
        for event in [
            touch::Event::FingerPressed {
                id: touch::Finger(0),
                position: start,
            },
            touch::Event::FingerMoved {
                id: touch::Finger(0),
                position: end,
            },
            touch::Event::FingerLifted {
                id: touch::Finger(0),
                position: end,
            },
        ] {
            let position = match event {
                touch::Event::FingerPressed { position, .. }
                | touch::Event::FingerMoved { position, .. }
                | touch::Event::FingerLifted { position, .. } => position,
                _ => unreachable!(),
            };
            h.send(Event::Touch(event), mouse::Cursor::Available(position));
        }
        assert!(
            h.messages.contains(&false),
            "parent did not scroll: widget {kind}"
        );
        assert!(!h.messages.contains(&true), "drag clicked widget {kind}");
        h.messages.clear();
        h.send(
            Event::Touch(touch::Event::FingerPressed {
                id: touch::Finger(0),
                position: end,
            }),
            mouse::Cursor::Available(end),
        );
        h.send(
            Event::Touch(touch::Event::FingerLifted {
                id: touch::Finger(0),
                position: end,
            }),
            mouse::Cursor::Available(end),
        );
        assert!(
            h.messages.contains(&true),
            "tap after scrolling: widget {kind}"
        );
    }
}
