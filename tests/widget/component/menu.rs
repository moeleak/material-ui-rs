use iced_widget::Space;

use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Message {
    Opened,
    Play,
    Remove,
}

type Renderer = iced_tiny_skia::Renderer;
type Menu = ContextMenu<'static, Message, Renderer>;

const WINDOW: Size = Size::new(360.0, 800.0);

fn items() -> Vec<Item<Message>> {
    vec![
        item("Play")
            .icon("play_arrow")
            .on_press(Message::Play)
            .into(),
        item("Settings").icon("settings").into(),
        divider(),
        item("Remove")
            .icon("delete")
            .destructive()
            .on_press(Message::Remove)
            .into(),
    ]
}

struct Harness {
    menu: Menu,
    tree: Tree,
    node: layout::Node,
    renderer: Renderer,
    messages: Vec<Message>,
}

impl Harness {
    fn new() -> Self {
        let mut menu = context(Space::new().width(100.0).height(100.0))
            .items(items())
            .on_open(Message::Opened);
        let renderer = Renderer::new(crate::fonts::ROBOTO, Pixels(16.0));
        let mut tree = Tree::new(&menu as &dyn Widget<Message, Theme, Renderer>);
        let node = menu
            .layout(
                &mut tree,
                &renderer,
                &layout::Limits::new(Size::ZERO, WINDOW),
            )
            .move_to(Point::new(20.0, 30.0));

        Self {
            menu,
            tree,
            node,
            renderer,
            messages: Vec::new(),
        }
    }

    fn send(&mut self, event: Event, cursor: Point) -> bool {
        let mut shell = Shell::new(&mut self.messages);
        self.menu.update(
            &mut self.tree,
            &event,
            Layout::new(&self.node),
            mouse::Cursor::Available(cursor),
            &self.renderer,
            &mut iced_widget::core::clipboard::Null,
            &mut shell,
            &Rectangle::with_size(WINDOW),
        );
        shell.is_event_captured()
    }

    /// Sends `event` to the open menu, laid out in the window, and returns
    /// its bounds and whether it captured the event.
    fn send_to_menu(&mut self, event: Event, cursor: Point) -> Option<(Rectangle, bool)> {
        let mut overlay = self.menu.overlay(
            &mut self.tree,
            Layout::new(&self.node),
            &self.renderer,
            &Rectangle::with_size(WINDOW),
            Vector::ZERO,
        )?;
        let node = overlay.as_overlay_mut().layout(&self.renderer, WINDOW);
        let mut shell = Shell::new(&mut self.messages);
        overlay.as_overlay_mut().update(
            &event,
            Layout::new(&node),
            mouse::Cursor::Available(cursor),
            &self.renderer,
            &mut iced_widget::core::clipboard::Null,
            &mut shell,
        );

        Some((node.bounds(), shell.is_event_captured()))
    }

    fn state(&self) -> &State {
        self.tree.state.downcast_ref::<State>()
    }
}

fn right_click() -> Event {
    Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right))
}

fn left_click() -> Event {
    Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
}

fn key(named: key::Named) -> Event {
    Event::Keyboard(keyboard::Event::KeyPressed {
        key: keyboard::Key::Named(named),
        modified_key: keyboard::Key::Named(named),
        physical_key: keyboard::key::Physical::Unidentified(
            keyboard::key::NativeCode::Unidentified,
        ),
        location: keyboard::Location::Standard,
        modifiers: keyboard::Modifiers::default(),
        text: None,
        repeat: false,
    })
}

#[test]
fn right_click_opens_the_menu_at_the_pointer() {
    let mut harness = Harness::new();

    assert!(harness.send(right_click(), Point::new(50.0, 60.0)));
    assert_eq!(harness.state().anchor, Some(Point::new(50.0, 60.0)));
    assert!(harness.state().is_shown());
    assert_eq!(harness.messages, vec![Message::Opened]);
}

#[test]
fn right_click_outside_the_content_does_nothing() {
    let mut harness = Harness::new();

    assert!(!harness.send(right_click(), Point::new(200.0, 60.0)));
    assert!(!harness.send(left_click(), Point::new(50.0, 60.0)));
    assert_eq!(harness.state().anchor, None);
    assert!(harness.messages.is_empty());
}

#[test]
fn a_menu_without_items_never_opens() {
    let mut harness = Harness::new();
    harness.menu.items.clear();

    assert!(!harness.send(right_click(), Point::new(50.0, 60.0)));
    assert_eq!(harness.state().anchor, None);
}

#[test]
fn picking_an_action_produces_its_message_and_closes() {
    let mut harness = Harness::new();
    let _ = harness.send(right_click(), Point::new(50.0, 60.0));
    harness.messages.clear();

    let play = Point::new(60.0, 60.0 + menu_tokens::TOP_SPACE + 24.0);
    let (bounds, captured) = harness.send_to_menu(left_click(), play).unwrap();

    assert_eq!(bounds.position(), Point::new(50.0, 60.0));
    assert!(captured);
    assert_eq!(harness.messages, vec![Message::Play]);
    assert!(harness.state().animation.frame().is_closing);
}

#[test]
fn disabled_actions_and_dividers_do_nothing() {
    let mut harness = Harness::new();
    let _ = harness.send(right_click(), Point::new(50.0, 60.0));
    harness.messages.clear();

    let settings = Point::new(60.0, 60.0 + menu_tokens::TOP_SPACE + 48.0 + 24.0);
    let divider = Point::new(60.0, 60.0 + menu_tokens::TOP_SPACE + 96.0 + 8.0);

    for point in [settings, divider] {
        let (_, captured) = harness.send_to_menu(left_click(), point).unwrap();

        assert!(captured);
        assert!(harness.messages.is_empty());
        assert!(!harness.state().animation.frame().is_closing);
    }
}

#[test]
fn clicking_outside_closes_without_reaching_what_is_under_it() {
    let mut harness = Harness::new();
    let _ = harness.send(right_click(), Point::new(50.0, 60.0));

    let (_, captured) = harness
        .send_to_menu(left_click(), Point::new(10.0, 700.0))
        .unwrap();

    assert!(captured);
    assert!(harness.state().animation.frame().is_closing);
}

#[test]
fn right_clicking_outside_closes_and_lets_the_click_through() {
    let mut harness = Harness::new();
    let _ = harness.send(right_click(), Point::new(50.0, 60.0));

    let (_, captured) = harness
        .send_to_menu(right_click(), Point::new(10.0, 700.0))
        .unwrap();

    assert!(!captured);
    assert!(harness.state().animation.frame().is_closing);
}

#[test]
fn escape_closes_and_the_keys_pick_enabled_actions() {
    let mut harness = Harness::new();
    let _ = harness.send(right_click(), Point::new(50.0, 60.0));
    harness.messages.clear();
    let away = Point::new(300.0, 700.0);

    let _ = harness.send_to_menu(key(key::Named::ArrowDown), away);
    assert_eq!(harness.state().hovered, Some(0));
    // Down skips the disabled action and the divider.
    let _ = harness.send_to_menu(key(key::Named::ArrowDown), away);
    assert_eq!(harness.state().hovered, Some(3));
    let _ = harness.send_to_menu(key(key::Named::Enter), away);
    assert_eq!(harness.messages, vec![Message::Remove]);

    let _ = harness.send(right_click(), Point::new(50.0, 60.0));
    let (_, captured) = harness.send_to_menu(key(key::Named::Escape), away).unwrap();
    assert!(captured);
    assert!(harness.state().animation.frame().is_closing);
}

#[test]
fn closed_menus_have_no_overlay_once_the_animation_ends() {
    let mut harness = Harness::new();
    let now = Instant::now();
    let _ = harness.send(right_click(), Point::new(50.0, 60.0));
    let _ = harness.send_to_menu(key(key::Named::Escape), Point::ORIGIN);

    let _ = harness.send(
        Event::Window(window::Event::RedrawRequested(
            now + std::time::Duration::from_secs(5),
        )),
        Point::ORIGIN,
    );

    assert_eq!(harness.state().anchor, None);
    assert!(harness.send_to_menu(left_click(), Point::ORIGIN).is_none());
}

#[test]
fn menus_open_towards_the_side_with_room() {
    let size = Size::new(200.0, 300.0);
    let window = Size::new(800.0, 600.0);

    assert_eq!(
        menu_position(Point::new(100.0, 100.0), size, window),
        (Point::new(100.0, 100.0), true)
    );
    // Too close to the right edge: opens to the left of the pointer.
    assert_eq!(
        menu_position(Point::new(700.0, 100.0), size, window),
        (Point::new(500.0, 100.0), true)
    );
    // Too close to the bottom: opens upwards.
    assert_eq!(
        menu_position(Point::new(100.0, 500.0), size, window),
        (Point::new(100.0, 200.0), false)
    );
    // No room either way: opens downwards, kept in the window.
    assert_eq!(
        menu_position(Point::new(100.0, 250.0), size, Size::new(800.0, 400.0)),
        (Point::new(100.0, 100.0), true)
    );
}

#[test]
fn menu_size_follows_the_material_metrics() {
    let items = items();

    assert_eq!(menu_height(&items), 8.0 + 48.0 * 3.0 + 17.0 + 8.0);
    assert_eq!(action_width(40.0, true), 12.0 + 24.0 + 12.0 + 40.0 + 12.0);
    assert_eq!(action_width(40.0, false), 12.0 + 40.0 + 12.0);
    assert_eq!(menu_width([]), menu_tokens::CONTAINER_MIN_WIDTH);
    assert_eq!(menu_width([150.0, 90.0]), 150.0);
    assert_eq!(menu_width([900.0]), menu_tokens::CONTAINER_MAX_WIDTH);
}

#[test]
fn items_stack_after_the_top_space() {
    let items = items();
    let bounds = Rectangle::new(
        Point::new(10.0, 20.0),
        Size::new(200.0, menu_height(&items)),
    );
    let rows: Vec<_> = item_bounds(bounds, &items).collect();

    assert_eq!(
        rows[0],
        Rectangle::new(Point::new(10.0, 28.0), Size::new(200.0, 48.0))
    );
    assert_eq!(rows[2].height, 17.0);
    assert_eq!(rows[3].y, 28.0 + 96.0 + 17.0);
    assert_eq!(action_at(bounds, &items, Point::new(50.0, 30.0)), Some(0));
    assert_eq!(action_at(bounds, &items, Point::new(50.0, 80.0)), None);
    assert_eq!(action_at(bounds, &items, Point::new(50.0, 22.0)), None);
}

#[test]
fn arrow_keys_wrap_around_the_enabled_actions() {
    let items = items();

    assert_eq!(next_action(&items, None, true), Some(0));
    assert_eq!(next_action(&items, None, false), Some(3));
    assert_eq!(next_action(&items, Some(0), true), Some(3));
    assert_eq!(next_action(&items, Some(3), true), Some(0));
    assert_eq!(next_action(&items, Some(0), false), Some(3));
    assert_eq!(next_action::<Message>(&[divider()], None, true), None);
    assert_eq!(next_action::<Message>(&[], None, true), None);
}

#[test]
fn destructive_and_disabled_actions_have_their_colors() {
    let theme = Theme::Dark;
    let colors = theme.colors();

    let destructive = action_colors(&theme, &item("Remove").destructive().on_press(()));
    assert_eq!(destructive.label, colors.error.color);
    assert_eq!(destructive.icon, colors.error.color);

    let enabled = action_colors(&theme, &item("Play").on_press(()));
    assert_eq!(enabled.label, colors.surface.text);
    assert_eq!(enabled.icon, colors.surface.text_variant);

    let disabled = action_colors(&theme, &item::<()>("Settings").destructive());
    assert_eq!(disabled.label.a, tokens::state::DISABLED_LABEL_TEXT_OPACITY);
    assert_eq!(disabled.icon, disabled.label);
}
