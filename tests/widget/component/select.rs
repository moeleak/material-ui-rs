use super::*;
use iced_widget::core::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq, Eq)]
enum InputMessage {
    Opened,
    Closed,
    Selected(u8),
}

type InputSelect = Select<'static, u8, [u8; 3], u8, InputMessage, iced_tiny_skia::Renderer>;
type InputParagraph = <iced_tiny_skia::Renderer as text::Renderer>::Paragraph;

struct InputHarness {
    select: InputSelect,
    tree: Tree,
    node: layout::Node,
    renderer: iced_tiny_skia::Renderer,
    messages: Vec<InputMessage>,
}

impl InputHarness {
    fn new() -> Self {
        let mut select = outlined([1, 2, 3], Some(2), InputMessage::Selected)
            .width(120.0)
            .on_open(InputMessage::Opened)
            .on_close(InputMessage::Closed);
        let renderer = iced_tiny_skia::Renderer::new(crate::fonts::ROBOTO, Pixels(16.0));
        let mut tree =
            Tree::new(&select as &dyn Widget<InputMessage, Theme, iced_tiny_skia::Renderer>);
        let node = select
            .layout(
                &mut tree,
                &renderer,
                &layout::Limits::new(Size::ZERO, Size::new(360.0, 800.0)),
            )
            .move_to(Point::new(20.0, 30.0));
        Self {
            select,
            tree,
            node,
            renderer,
            messages: Vec::new(),
        }
    }

    fn send(&mut self, event: Event, cursor: mouse::Cursor) -> bool {
        let mut shell = Shell::new(&mut self.messages);
        self.select.update(
            &mut self.tree,
            &event,
            Layout::new(&self.node),
            cursor,
            &self.renderer,
            &mut iced_widget::core::clipboard::Null,
            &mut shell,
            &Rectangle::with_size(Size::new(360.0, 800.0)),
        );
        shell.is_event_captured()
    }

    fn state(&self) -> &State<InputParagraph> {
        self.tree.state.downcast_ref::<State<InputParagraph>>()
    }
}

fn pointer_events(is_touch: bool, positions: &[Point]) -> Vec<(Event, mouse::Cursor)> {
    positions
        .iter()
        .enumerate()
        .map(|(index, &position)| {
            let first = index == 0;
            let last = index == positions.len() - 1;
            if is_touch {
                let id = touch::Finger(0);
                let event = if first {
                    touch::Event::FingerPressed { id, position }
                } else if last {
                    touch::Event::FingerLifted { id, position }
                } else {
                    touch::Event::FingerMoved { id, position }
                };
                (Event::Touch(event), mouse::Cursor::Unavailable)
            } else {
                let event = if first {
                    mouse::Event::ButtonPressed(mouse::Button::Left)
                } else if last {
                    mouse::Event::ButtonReleased(mouse::Button::Left)
                } else {
                    mouse::Event::CursorMoved { position }
                };
                (Event::Mouse(event), mouse::Cursor::Available(position))
            }
        })
        .collect()
}

#[test]
fn select_widget_opens_only_on_valid_mouse_or_touch_release() {
    let origin = Point::new(40.0, 50.0);
    for is_touch in [false, true] {
        for motion in [0.0, 8.0] {
            let mut input = InputHarness::new();
            let moved = origin + Vector::new(motion, 0.0);
            let mut events = pointer_events(is_touch, &[origin, moved, moved]).into_iter();
            let (event, cursor) = events.next().unwrap();
            assert_eq!(input.send(event, cursor), !is_touch);
            assert!(!input.state().is_open);
            assert!(!input.state().menu.is_visible());
            assert!(input.messages.is_empty());

            let (event, cursor) = events.next().unwrap();
            assert!(!input.send(event, cursor));
            assert!(!input.state().is_open);
            let (event, cursor) = events.next().unwrap();
            assert!(input.send(event.clone(), cursor));
            assert!(input.state().is_open);
            assert_eq!(input.state().hovered_option, Some(1));
            assert_eq!(input.messages, [InputMessage::Opened]);
            let _ = input.send(event, cursor);
            assert_eq!(input.messages, [InputMessage::Opened]);
        }
    }
}

#[test]
fn select_widget_mouse_and_touch_drags_do_not_open_after_returning_to_field() {
    let origin = Point::new(40.0, 50.0);
    for is_touch in [false, true] {
        for moved in [Point::new(49.0, 50.0), Point::new(180.0, 50.0)] {
            let mut input = InputHarness::new();
            for (event, cursor) in pointer_events(is_touch, &[origin, moved, origin, origin]) {
                let _ = input.send(event, cursor);
                assert!(!input.state().is_open);
                assert!(input.messages.is_empty());
            }
            for (event, cursor) in pointer_events(is_touch, &[origin, origin]) {
                let _ = input.send(event, cursor);
            }
            assert_eq!(input.messages, [InputMessage::Opened]);
        }
    }
}

#[test]
fn select_widget_mouse_drag_uses_raw_events_with_translated_batch_cursor() {
    let mut input = InputHarness::new();
    let raw_origin = Point::new(40.0, 10.0);
    let cursor = mouse::Cursor::Available(Point::new(40.0, 50.0));
    let _ = input.send(
        Event::Mouse(mouse::Event::CursorMoved {
            position: raw_origin,
        }),
        cursor,
    );
    let _ = input.send(
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
        cursor,
    );
    for position in [Point::new(80.0, 10.0), raw_origin] {
        let _ = input.send(Event::Mouse(mouse::Event::CursorMoved { position }), cursor);
    }
    let _ = input.send(
        Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
        cursor,
    );
    assert!(!input.state().is_open);
    assert!(input.messages.is_empty());
}

#[test]
fn select_widget_unfocus_cancels_pending_open() {
    let origin = Point::new(40.0, 50.0);
    for is_touch in [false, true] {
        let mut input = InputHarness::new();
        let mut events = pointer_events(is_touch, &[origin, origin]).into_iter();
        let (event, cursor) = events.next().unwrap();
        let _ = input.send(event, cursor);
        let _ = input.send(
            Event::Window(window::Event::Unfocused),
            mouse::Cursor::Unavailable,
        );
        let (event, cursor) = events.next().unwrap();
        let _ = input.send(event, cursor);
        assert!(!input.state().is_open);
        assert!(input.messages.is_empty());
    }
}

#[test]
fn select_widget_preserves_outside_press_dismissal_without_reopening_on_release() {
    let mut input = InputHarness::new();
    input
        .tree
        .state
        .downcast_mut::<State<InputParagraph>>()
        .set_open(true, Instant::now());
    let cursor = mouse::Cursor::Available(Point::new(240.0, 150.0));
    assert!(input.send(
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
        cursor,
    ));
    assert!(!input.state().is_open);
    assert_eq!(input.messages, [InputMessage::Closed]);
    let _ = input.send(
        Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
        cursor,
    );
    assert!(!input.state().is_open);
    assert_eq!(input.messages, [InputMessage::Closed]);
}

#[test]
fn select_widget_preserves_command_wheel_selection_while_closed() {
    let mut input = InputHarness::new();
    let cursor = mouse::Cursor::Available(Point::new(40.0, 50.0));
    let _ = input.send(
        Event::Keyboard(keyboard::Event::ModifiersChanged(
            keyboard::Modifiers::COMMAND,
        )),
        cursor,
    );
    assert!(input.send(
        Event::Mouse(mouse::Event::WheelScrolled {
            delta: mouse::ScrollDelta::Lines { x: 0.0, y: -1.0 },
        }),
        cursor,
    ));
    assert!(!input.state().is_open);
    assert_eq!(input.messages, [InputMessage::Selected(3)]);
}

#[test]
fn selection_fields_inside_scrollable_allow_touch_scrolling_and_taps() {
    #[derive(Debug, Clone)]
    enum ScrollMessage {
        Opened,
        Input,
        Scrolled(f32),
    }

    fn input_is_focused(tree: &Tree) -> bool {
        type InputState = iced_widget::text_input::State<InputParagraph>;
        if tree.tag == tree::Tag::of::<InputState>() {
            tree.state.downcast_ref::<InputState>().is_focused()
        } else {
            tree.children.iter().any(input_is_focused)
        }
    }

    for kind in 0..3 {
        for drag in [false, true] {
            let combo_state = crate::widget::combobox::State::new(vec![1_u8, 2, 3]);
            let field: Element<'_, ScrollMessage, Theme, iced_tiny_skia::Renderer> = match kind {
                0 => outlined([1_u8, 2, 3], Some(2), |_| ScrollMessage::Input)
                    .on_open(ScrollMessage::Opened)
                    .into(),
                1 => crate::widget::combobox::outlined(&combo_state, "Choose", None, |_| {
                    ScrollMessage::Input
                })
                .on_open(ScrollMessage::Opened)
                .into(),
                2 => crate::widget::text_input::outlined("Input", "")
                    .on_input(|_| ScrollMessage::Input)
                    .into(),
                _ => unreachable!(),
            };
            let content = iced_widget::Column::new()
                .push(iced_widget::Space::new().height(60))
                .push(field)
                .push(iced_widget::Space::new().height(900));
            let mut scrollable = iced_widget::Scrollable::new(content)
                .width(240)
                .height(200)
                .on_scroll(|viewport| ScrollMessage::Scrolled(viewport.absolute_offset().y));
            let renderer = iced_tiny_skia::Renderer::new(crate::fonts::ROBOTO, Pixels(16.0));
            let mut tree = Tree::new(
                &scrollable as &dyn Widget<ScrollMessage, Theme, iced_tiny_skia::Renderer>,
            );
            let viewport = Rectangle::with_size(Size::new(240.0, 200.0));
            let node = scrollable.layout(
                &mut tree,
                &renderer,
                &layout::Limits::new(Size::ZERO, viewport.size()),
            );
            let origin = Point::new(40.0, 90.0);
            let end = if drag { Point::new(40.0, 30.0) } else { origin };
            let mut messages = Vec::new();
            // Small individual moves still form a drag, even when parent scroll
            // translation keeps the child's local cursor near its press origin.
            let mut positions = vec![origin];
            if drag {
                positions.extend(
                    (1..=15).map(|step| Point::new(origin.x, origin.y - step as f32 * 4.0)),
                );
            } else {
                positions.push(origin);
            }
            positions.push(end);
            let last = positions.len() - 1;
            for (index, position) in positions.into_iter().enumerate() {
                let id = touch::Finger(0);
                let event = Event::Touch(match index {
                    0 => touch::Event::FingerPressed { id, position },
                    i if i == last => touch::Event::FingerLifted { id, position },
                    _ => touch::Event::FingerMoved { id, position },
                });
                scrollable.update(
                    &mut tree,
                    &event,
                    Layout::new(&node),
                    mouse::Cursor::Available(position),
                    &renderer,
                    &mut iced_widget::core::clipboard::Null,
                    &mut Shell::new(&mut messages),
                    &viewport,
                );
                if index == 0 {
                    assert!(messages.is_empty(), "field {kind} activated on touch down");
                    assert!(!input_is_focused(&tree));
                }
            }

            let did_scroll = messages
                .iter()
                .any(|message| matches!(message, ScrollMessage::Scrolled(y) if *y > 0.0));
            let opened = messages
                .iter()
                .filter(|message| matches!(message, ScrollMessage::Opened))
                .count();
            assert_eq!(did_scroll, drag, "field {kind}, drag={drag}, {messages:?}");
            assert_eq!(opened, usize::from(!drag && kind != 2), "field {kind}");
            assert_eq!(input_is_focused(&tree), !drag && kind != 0, "field {kind}");
            assert!(
                !messages
                    .iter()
                    .any(|message| matches!(message, ScrollMessage::Input))
            );
        }
    }
}

#[test]
fn material_menu_height_uses_five_visible_options_max() {
    assert_eq!(
        material_menu_height(3),
        Length::Fixed(tokens::component::select::MENU_LIST_ITEM_CONTAINER_HEIGHT * 3.0)
    );
    assert_eq!(
        material_menu_height(8),
        Length::Fixed(
            tokens::component::select::MENU_LIST_ITEM_CONTAINER_HEIGHT * MAX_VISIBLE_OPTIONS as f32
        )
    );
}

#[test]
fn material_option_padding_produces_m3_menu_item_height() {
    let padding = menu_option_padding();

    assert_eq!(
        tokens::component::text_field::INPUT_TEXT_LINE_HEIGHT + padding.y(),
        tokens::component::select::MENU_LIST_ITEM_CONTAINER_HEIGHT
    );
}

#[test]
fn select_status_tracks_open_state_before_hover_state() {
    assert_eq!(select_status(false, false), Status::Active);
    assert_eq!(select_status(false, true), Status::Hovered);
    assert_eq!(
        select_status(true, false),
        Status::Opened { is_hovered: false }
    );
    assert_eq!(
        select_status(true, true),
        Status::Opened { is_hovered: true }
    );
}

#[test]
fn select_menu_remains_visible_during_close_animation() {
    let start = Instant::now();
    let mut menu = menu_overlay::State::new();

    menu.start_open(3, start);
    assert!(!menu.advance(start + Duration::from_secs(2)));
    menu.start_close(start + Duration::from_secs(2));

    assert!(menu.is_visible());
    assert!(menu.is_animating());

    assert!(!menu.advance(start + Duration::from_secs(4)));
    assert!(!menu.is_visible());
}

#[test]
fn default_handle_rotation_matches_compose_trailing_icon_targets() {
    assert_eq!(menu_handle_rotation_target(false), 0.0);
    assert_eq!(menu_handle_rotation_target(true), 1.0);
    assert_eq!(menu_handle_rotation_radians(0.0), 0.0);
    assert_eq!(
        menu_handle_rotation_radians(0.5),
        std::f32::consts::FRAC_PI_2
    );
    assert_eq!(menu_handle_rotation_radians(1.0), std::f32::consts::PI);
}

#[test]
fn default_handle_arrow_points_match_material_icon_viewbox() {
    assert_eq!(
        default_handle_arrow_points(MENU_HANDLE_VIEWPORT_SIZE),
        [
            Point::new(7.0, 10.0),
            Point::new(12.0, 15.0),
            Point::new(17.0, 10.0),
        ]
    );
    assert_eq!(
        default_handle_arrow_points(MENU_HANDLE_VIEWPORT_SIZE / 2.0),
        [
            Point::new(3.5, 5.0),
            Point::new(6.0, 7.5),
            Point::new(8.5, 5.0),
        ]
    );
}

#[test]
fn select_prefers_down_when_menu_fits_below() {
    let position = Point::new(0.0, 500.0);
    let target_height = 56.0;
    let viewport = Rectangle {
        x: 0.0,
        y: 0.0,
        width: 800.0,
        height: 940.0,
    };

    let anchor = prefer_down_when_menu_fits(position, viewport, target_height, 144.0);

    assert_eq!(anchor.position.y + anchor.target_height, 556.0);
    assert!(viewport.height - (anchor.position.y + anchor.target_height) > anchor.position.y);
}

#[test]
fn select_keeps_default_anchor_when_menu_does_not_fit_below() {
    let position = Point::new(0.0, 500.0);
    let target_height = 56.0;
    let viewport = Rectangle {
        x: 0.0,
        y: 0.0,
        width: 800.0,
        height: 620.0,
    };

    let anchor = prefer_down_when_menu_fits(position, viewport, target_height, 144.0);

    assert_eq!(anchor.position, position);
    assert_eq!(anchor.target_height, target_height);
}
