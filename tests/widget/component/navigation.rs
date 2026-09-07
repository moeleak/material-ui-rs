use super::*;
use iced_widget::core::time::Duration;
use iced_widget::core::{Pixels, Transformation, image};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Page {
    One,
    Two,
}

#[derive(Debug, Clone)]
enum Message {
    Frame,
}

#[test]
fn standard_rail_does_not_cast_a_shadow_over_the_adjacent_surface() {
    for theme in [Theme::Light, Theme::Dark] {
        let mut renderer = iced_tiny_skia::Renderer::new(crate::fonts::ROBOTO, Pixels(16.0));
        let mut element: Element<'_, Message, Theme, iced_tiny_skia::Renderer> =
            Container::new(Space::new())
                .width(80)
                .height(80)
                .style(rail_container)
                .into();
        let mut tree = Tree::new(element.as_widget());
        let size = Size::new(120.0, 100.0);
        let node = element.as_widget_mut().layout(
            &mut tree,
            &renderer,
            &layout::Limits::new(Size::ZERO, size),
        );
        let viewport = Rectangle::with_size(size);
        element.as_widget().draw(
            &tree,
            &mut renderer,
            &theme,
            &renderer::Style::default(),
            Layout::new(&node),
            mouse::Cursor::Unavailable,
            &viewport,
        );
        let mut pixels = tiny_skia::Pixmap::new(120, 100).unwrap();
        renderer.draw(
            &mut pixels.as_mut(),
            &mut tiny_skia::Mask::new(120, 100).unwrap(),
            &iced_widget::graphics::Viewport::with_physical_size(Size::new(120, 100), 1.0),
            &[viewport],
            theme.colors().surface.color,
        );
        let page = pixels.pixel(110, 40);
        for x in 80..100 {
            assert_eq!(
                pixels.pixel(x, 40),
                page,
                "standard rail cast a shadow onto its neighboring page: theme={theme}, x={x}"
            );
        }
    }
}

#[test]
fn window_size_classes_use_material_breakpoints() {
    assert_eq!(width_class(599.0), WindowWidthClass::Compact);
    assert_eq!(width_class(600.0), WindowWidthClass::Medium);
    assert_eq!(width_class(839.0), WindowWidthClass::Medium);
    assert_eq!(width_class(840.0), WindowWidthClass::Expanded);

    assert_eq!(height_class(479.0), WindowHeightClass::Compact);
    assert_eq!(height_class(480.0), WindowHeightClass::Medium);
    assert_eq!(height_class(900.0), WindowHeightClass::Expanded);
}

#[test]
fn adaptive_layout_matches_navigation_suite_default() {
    assert_eq!(adaptive_layout(480.0, 900.0), AdaptiveLayout::NavigationBar);
    assert_eq!(adaptive_layout(700.0, 420.0), AdaptiveLayout::NavigationBar);
    assert_eq!(
        adaptive_layout(700.0, 700.0),
        AdaptiveLayout::NavigationRail
    );
    assert_eq!(
        adaptive_layout(1080.0, 980.0),
        AdaptiveLayout::NavigationRail
    );
    assert_eq!(
        AdaptiveLayout::from_size(1080.0, 980.0),
        AdaptiveLayout::NavigationRail
    );
    assert_eq!(
        WindowSizeClass::from_size(420.0, 900.0).adaptive_navigation_layout(),
        AdaptiveLayout::NavigationBar
    );
    assert_eq!(
        item_animation_duration_ms(AdaptiveLayout::NavigationBar),
        tokens::component::navigation_bar::ITEM_ANIMATION_DURATION_MS
    );
    assert_eq!(
        item_animation_duration_ms(AdaptiveLayout::NavigationRail),
        tokens::component::navigation_rail::ITEM_ANIMATION_DURATION_MS
    );
}

#[test]
fn menu_builder_defaults_to_the_compatible_compact_navigation_bar() {
    let destinations = [Destination::new(Page::One, "1", "One")];
    let state = NavigationState::new(Page::One);
    let menu = suite(&destinations, &state).with_menu("Navigation", Message::Frame);

    assert_eq!(menu.compact_navigation, CompactNavigation::NavigationBar);

    let menu = menu.compact_navigation(CompactNavigation::ModalDrawer);
    assert_eq!(menu.compact_navigation, CompactNavigation::ModalDrawer);
}

fn layout_navigation(
    mut element: Element<'_, Message, Theme, SingleLineTestRenderer>,
    size: Size,
) -> layout::Node {
    let mut tree = Tree::new(element.as_widget());
    element.as_widget_mut().layout(
        &mut tree,
        &SingleLineTestRenderer,
        &layout::Limits::new(Size::ZERO, size),
    )
}

#[test]
fn standalone_navigation_bar_adds_safe_area_outside_its_item_region() {
    let destinations = [Destination::new(Page::One, "1", "One")];
    let insets = Padding {
        top: 4.0,
        right: 12.0,
        bottom: 24.0,
        left: 16.0,
    };
    let node = layout_navigation(
        bar_with(
            &destinations,
            Selection::new(Page::One),
            |_| Message::Frame,
            NavigationBarOptions::default().insets(insets),
        )
        .into(),
        Size::new(360.0, 800.0),
    );
    let layout = Layout::new(&node);
    let items = layout.children().next().unwrap();
    assert_eq!(layout.bounds().height, 108.0);
    assert_eq!(
        items.bounds(),
        Rectangle {
            x: 24.0,
            y: 4.0,
            width: 316.0,
            height: 80.0
        }
    );
    assert_eq!(items.children().next().unwrap().bounds().height, 80.0);

    let default_node = layout_navigation(
        bar(&destinations, Selection::new(Page::One), |_| Message::Frame).into(),
        Size::new(360.0, 800.0),
    );
    assert_eq!(default_node.size(), Size::new(360.0, 80.0));
}

#[test]
fn navigation_suite_consumes_each_inset_once_above_visible_bottom_bar() {
    let destinations = [Destination::new(Page::One, "1", "One")];
    let state = NavigationState::new(Page::One);
    let insets = Padding {
        top: 24.0,
        right: 12.0,
        bottom: 24.0,
        left: 16.0,
    };
    for with_menu in [false, true] {
        let suite = suite(&destinations, &state)
            .dimensions(360.0, 800.0)
            .insets(insets);
        let content = Space::new().width(Length::Fill).height(Length::Fill);
        let element = if with_menu {
            suite
                .with_menu("Menu", Message::Frame)
                .view(|_| Message::Frame, content)
        } else {
            suite.view(|_| Message::Frame, content)
        };
        let node = layout_navigation(element, Size::new(360.0, 800.0));
        let root = Layout::new(&node);
        let mut children = root.children();
        let page = children.next().unwrap().children().next().unwrap();
        let bar = children.next().unwrap();
        assert_eq!(
            page.bounds(),
            Rectangle {
                x: 16.0,
                y: 24.0,
                width: 332.0,
                height: 672.0
            }
        );
        assert_eq!(
            bar.bounds(),
            Rectangle {
                x: 0.0,
                y: 696.0,
                width: 360.0,
                height: 104.0
            }
        );
        assert_eq!(bar.children().next().unwrap().bounds().height, 80.0);
    }
}

#[test]
fn hidden_bottom_navigation_leaves_no_gap_above_keyboard() {
    let destinations = [Destination::new(Page::One, "1", "One")];
    let state = NavigationState::new(Page::One);
    let insets = Padding {
        top: 24.0,
        right: 12.0,
        bottom: 280.0,
        left: 16.0,
    };
    let element = suite(&destinations, &state)
        .dimensions(360.0, 800.0)
        .with_menu("Menu", Message::Frame)
        .insets(insets)
        .navigation_bar_visible(false)
        .view(
            |_| Message::Frame,
            Space::new().width(Length::Fill).height(Length::Fill),
        );
    let node = layout_navigation(element, Size::new(360.0, 800.0));
    let root = Layout::new(&node);
    let page = root.children().next().unwrap().children().next().unwrap();
    assert_eq!(root.children().count(), 1);
    assert_eq!(
        page.bounds(),
        Rectangle {
            x: 16.0,
            y: 24.0,
            width: 332.0,
            height: 496.0
        }
    );
}

#[test]
fn navigation_bar_visibility_and_insets_preserve_focused_input_tree() {
    use iced_widget::core::widget::{Id, operation::focusable};

    fn input_is_focused(tree: &Tree) -> bool {
        type InputState = iced_widget::text_input::State<SingleLineTestParagraph>;
        if tree.tag == tree::Tag::of::<InputState>() {
            tree.state.downcast_ref::<InputState>().is_focused()
        } else {
            tree.children.iter().any(input_is_focused)
        }
    }

    let destinations = [Destination::new(Page::One, "1", "One")];
    let state = NavigationState::new(Page::One);
    let input_id = Id::new("navigation-focus-regression");
    let limits = layout::Limits::new(Size::ZERO, Size::new(360.0, 800.0));
    for with_menu in [false, true] {
        let build = |visible, insets| {
            let input = iced_widget::text_input("Input", "persistent value")
                .id(input_id.clone())
                .on_input(|_| Message::Frame);
            let suite = suite(&destinations, &state)
                .dimensions(360.0, 800.0)
                .insets(insets)
                .navigation_bar_visible(visible);
            if with_menu {
                suite
                    .with_menu("Menu", Message::Frame)
                    .view(|_| Message::Frame, input)
            } else {
                suite.view(|_| Message::Frame, input)
            }
        };
        let mut element: Element<'_, Message, Theme, SingleLineTestRenderer> =
            build(true, Padding::ZERO);
        let mut tree = Tree::new(element.as_widget());
        let node = element
            .as_widget_mut()
            .layout(&mut tree, &SingleLineTestRenderer, &limits);
        element.as_widget_mut().operate(
            &mut tree,
            Layout::new(&node),
            &SingleLineTestRenderer,
            &mut focusable::focus::<()>(input_id.clone()),
        );
        assert!(input_is_focused(&tree));

        // Cover both overlaying IMEs and native-resized windows, which report
        // zero remaining keyboard inset while the navigation bar is hidden.
        for (visible, insets) in [
            (
                false,
                Padding {
                    bottom: 280.0,
                    ..Padding::ZERO
                },
            ),
            (
                true,
                Padding {
                    bottom: 24.0,
                    ..Padding::ZERO
                },
            ),
            (false, Padding::ZERO),
            (true, Padding::ZERO),
        ] {
            let mut element = build(visible, insets);
            tree.diff(element.as_widget());
            let _ = element
                .as_widget_mut()
                .layout(&mut tree, &SingleLineTestRenderer, &limits);
            assert!(
                input_is_focused(&tree),
                "focus lost with visible={visible}, insets={insets:?}"
            );
        }
    }
}

#[test]
fn safe_area_and_bottom_bar_visibility_preserve_rail_and_modal_drawer() {
    let destinations = [Destination::new(Page::One, "1", "One")];
    let state = NavigationState::new(Page::One);
    let insets = Padding {
        top: 24.0,
        right: 12.0,
        bottom: 32.0,
        left: 16.0,
    };
    for layout in [
        AdaptiveLayout::NavigationBar,
        AdaptiveLayout::NavigationRail,
    ] {
        let element = suite(&destinations, &state)
            .dimensions(360.0, 800.0)
            .layout(layout)
            .insets(insets)
            .navigation_bar_visible(false)
            .with_menu("Menu", Message::Frame)
            .compact_navigation(CompactNavigation::ModalDrawer)
            .view(
                |_| Message::Frame,
                Space::new().width(Length::Fill).height(Length::Fill),
            );
        let node = layout_navigation(element, Size::new(360.0, 800.0));
        let root = Layout::new(&node);
        let shell = root.children().next().unwrap();
        assert_eq!(
            shell.bounds(),
            Rectangle {
                x: 16.0,
                y: 24.0,
                width: 332.0,
                height: 744.0
            }
        );
        assert_eq!(
            shell.children().count(),
            2,
            "rail/top app bar and page remain visible"
        );
    }
}

#[test]
fn closing_modal_drawer_scrim_does_not_block_page_hit_testing() {
    let mut scrim =
        modal_drawer_scrim::<Message, SingleLineTestRenderer>(false, Message::Frame, 0.5);
    let mut tree = Tree::new(scrim.as_widget());
    let renderer = SingleLineTestRenderer;
    let limits = layout::Limits::new(Size::ZERO, Size::new(360.0, 640.0));
    let node = scrim.as_widget_mut().layout(&mut tree, &renderer, &limits);
    let bounds = Rectangle::new(Point::ORIGIN, node.size());
    let cursor = mouse::Cursor::Available(Point::new(180.0, 320.0));
    let interaction =
        scrim
            .as_widget()
            .mouse_interaction(&tree, Layout::new(&node), cursor, &bounds, &renderer);

    assert_eq!(interaction, mouse::Interaction::None);
}

#[test]
fn open_modal_drawer_scrim_blocks_page_hit_testing() {
    let mut scrim =
        modal_drawer_scrim::<Message, SingleLineTestRenderer>(true, Message::Frame, 1.0);
    let mut tree = Tree::new(scrim.as_widget());
    let renderer = SingleLineTestRenderer;
    let limits = layout::Limits::new(Size::ZERO, Size::new(360.0, 640.0));
    let node = scrim.as_widget_mut().layout(&mut tree, &renderer, &limits);
    let bounds = Rectangle::new(Point::ORIGIN, node.size());
    let cursor = mouse::Cursor::Available(Point::new(180.0, 320.0));
    let interaction =
        scrim
            .as_widget()
            .mouse_interaction(&tree, Layout::new(&node), cursor, &bounds, &renderer);

    assert_ne!(interaction, mouse::Interaction::None);
}

#[test]
fn selection_interpolates_previous_and_selected_destination() {
    let selection = Selection::transitioning(Page::Two, Page::One, 0.25);

    assert_eq!(selection.progress(Page::Two), 0.25);
    assert_eq!(selection.progress(Page::One), 0.75);
    assert_eq!(Selection::new(Page::One).progress(Page::One), 1.0);
}

#[test]
fn destination_badge_builders_attach_navigation_badges() {
    let small = Destination::new(Page::One, "1", "One").small_badge();
    let large = Destination::new(Page::Two, "2", "Two").badge("3");

    assert_eq!(small.badge, Some(Badge::Small));
    assert_eq!(large.badge, Some(Badge::Large("3")));
}

#[test]
fn navigation_state_exposes_animation_subscription() {
    let state = NavigationState::new(Page::One);
    let _: iced::Subscription<Message> = state.subscription(|_| Message::Frame);
}

#[test]
fn navigation_state_selects_using_window_size() {
    let start = Instant::now();
    let mut state = NavigationState::new(Page::One);

    state.select_for_size(Page::Two, start, Size::new(1080.0, 980.0));

    assert_eq!(state.selected(), Page::Two);
    assert!(state.is_animating());
    assert_eq!(state.selection().progress(Page::Two), 0.0);
}

#[test]
fn navigation_state_toggles_menu_expansion() {
    let start = Instant::now();
    let mut state = NavigationState::new(Page::One);

    state.toggle_menu(start);

    assert!(state.is_menu_open());
    assert!(state.is_menu_visible());
    assert!(state.is_animating());

    state.advance_frame(start + Duration::from_millis(50));

    assert!(state.menu_progress() > 0.0);
}

#[test]
fn compact_navigation_selection_closes_modal_drawer() {
    let start = Instant::now();
    let mut state = NavigationState::new(Page::One);
    state.toggle_menu_for_layout(start, AdaptiveLayout::NavigationBar);

    state.select(
        Page::Two,
        start + Duration::from_millis(50),
        AdaptiveLayout::NavigationBar,
    );

    assert_eq!(state.selected(), Page::Two);
    assert!(!state.is_menu_open());
    assert!(state.is_menu_visible());
}

#[test]
fn modal_navigation_drawer_matches_compose_tween_timing() {
    let start = Instant::now();
    let mut state = NavigationState::new(Page::One);
    state.toggle_menu_for_layout(start, AdaptiveLayout::NavigationBar);

    let half_duration = u64::from(tokens::component::navigation_drawer::ANIMATION_DURATION_MS) / 2;
    assert!(state.advance(start + Duration::from_millis(half_duration)));
    let expected = tokens::motion::EASING_LEGACY.transform(0.5);
    assert!((state.menu_progress() - expected).abs() < 0.001);

    assert!(!state.advance(
        start
            + Duration::from_millis(u64::from(
                tokens::component::navigation_drawer::ANIMATION_DURATION_MS,
            )),
    ));
    assert_eq!(state.menu_progress(), 1.0);
}

#[test]
fn modal_navigation_drawer_reverses_from_current_frame_without_jump() {
    let start = Instant::now();
    let mut state = NavigationState::new(Page::One);
    state.toggle_menu_for_layout(start, AdaptiveLayout::NavigationBar);

    let reversal = start + Duration::from_millis(128);
    state.toggle_menu_for_layout(reversal, AdaptiveLayout::NavigationBar);
    let progress_at_reversal = tokens::motion::EASING_LEGACY.transform(0.5);

    assert!((state.menu_progress() - progress_at_reversal).abs() < 0.001);
    assert!(!state.is_menu_open());
    assert!(state.advance(reversal + Duration::from_millis(64)));
    assert!(state.menu_progress() < progress_at_reversal);
}

#[test]
fn navigation_menu_icon_morphs_from_hamburger_to_arrow() {
    assert_eq!(
        navigation_menu_icon_segments(0.0, NAVIGATION_MENU_ICON_VIEWPORT_SIZE),
        [
            (Point::new(5.0, 7.0), Point::new(19.0, 7.0)),
            (Point::new(5.0, 12.0), Point::new(19.0, 12.0)),
            (Point::new(5.0, 17.0), Point::new(19.0, 17.0)),
        ]
    );
    assert_eq!(
        navigation_menu_icon_segments(1.0, NAVIGATION_MENU_ICON_VIEWPORT_SIZE),
        [
            (Point::new(12.0, 5.0), Point::new(19.0, 12.0)),
            (Point::new(5.0, 12.0), Point::new(19.0, 12.0)),
            (Point::new(12.0, 19.0), Point::new(19.0, 12.0)),
        ]
    );
}

#[test]
fn navigation_menu_icon_rotation_tracks_expansion_progress() {
    assert_eq!(NavigationMenuIcon { progress: 0.0 }.rotation_radians(), 0.0);
    assert_eq!(
        NavigationMenuIcon { progress: 0.5 }.rotation_radians(),
        std::f32::consts::FRAC_PI_2
    );
    assert_eq!(
        NavigationMenuIcon { progress: 1.0 }.rotation_radians(),
        std::f32::consts::PI
    );
}

#[test]
fn navigation_menu_button_matches_0_4_4_interaction_geometry() {
    let button = navigation_menu_button::<Message, iced_widget::Renderer>(Message::Frame, 0.5);

    assert_eq!(
        Widget::<Message, Theme, iced_widget::Renderer>::size(&button),
        Size::new(
            Length::Fixed(tokens::component::icon_button::CONTAINER_WIDTH),
            Length::Fixed(tokens::component::icon_button::CONTAINER_HEIGHT),
        )
    );
    assert_eq!(tokens::component::icon_button::CONTAINER_WIDTH, 40.0);
    assert_eq!(tokens::component::icon_button::CONTAINER_HEIGHT, 40.0);
    assert_eq!(tokens::component::icon_button::STATE_LAYER_WIDTH, 40.0);
    assert_eq!(tokens::component::icon_button::STATE_LAYER_HEIGHT, 40.0);
}

#[test]
fn navigation_state_owns_selection_animation_progress() {
    let start = Instant::now();
    let mut state = NavigationState::new(Page::One);

    state.select(Page::Two, start, AdaptiveLayout::NavigationRail);

    assert_eq!(state.selected(), Page::Two);
    assert!(state.is_animating());
    assert_eq!(state.selection().progress(Page::Two), 0.0);
    assert_eq!(state.selection().progress(Page::One), 1.0);

    let still_animating = state.advance(start + Duration::from_millis(50));

    assert!(still_animating);
    assert!(state.selection().progress(Page::Two) > 0.0);
    assert!(state.selection().progress(Page::One) < 1.0);
    assert_eq!(
        state.selection().size_progress(Page::Two),
        state.selection().alpha_progress(Page::Two)
    );

    let finished = state.advance(start + Duration::from_millis(500));

    assert!(!finished);
    assert!(!state.is_animating());
    assert_eq!(state.selection().progress(Page::Two), 1.0);
    assert_eq!(state.selection().progress(Page::One), 0.0);
}

#[test]
fn navigation_selection_timing_matches_androidx_material_durations() {
    let start = Instant::now();
    let mut bar = NavigationState::new(Page::One);
    let mut rail = NavigationState::new(Page::One);

    bar.select(Page::Two, start, AdaptiveLayout::NavigationBar);
    rail.select(Page::Two, start, AdaptiveLayout::NavigationRail);

    let _ = bar.advance(
        start
            + Duration::from_millis(u64::from(
                tokens::component::navigation_bar::ITEM_ANIMATION_DURATION_MS + 20,
            )),
    );
    let _ = rail.advance(
        start
            + Duration::from_millis(u64::from(
                tokens::component::navigation_bar::ITEM_ANIMATION_DURATION_MS + 20,
            )),
    );

    assert_eq!(bar.selection().progress(Page::Two), 1.0);
    assert!(rail.selection().progress(Page::Two) < 1.0);

    let _ = rail.advance(
        start
            + Duration::from_millis(u64::from(
                tokens::component::navigation_rail::ITEM_ANIMATION_DURATION_MS + 20,
            )),
    );

    assert_eq!(rail.selection().progress(Page::Two), 1.0);
}

#[test]
fn navigation_state_preserves_progress_when_transition_is_interrupted() {
    let start = Instant::now();
    let mut state = NavigationState::new(Page::One);

    state.select(Page::Two, start, AdaptiveLayout::NavigationRail);
    let _ = state.advance(start + Duration::from_millis(50));

    let two_progress = state.selection().progress(Page::Two);

    state.select(
        Page::One,
        start + Duration::from_millis(50),
        AdaptiveLayout::NavigationRail,
    );

    assert_eq!(state.selected(), Page::One);
    assert_eq!(state.selection().progress(Page::Two), two_progress);
    assert!(state.selection().progress(Page::One) > 0.0);
}

#[test]
fn navigation_state_reselect_does_not_start_duplicate_state_layer_feedback() {
    let start = Instant::now();
    let mut state = NavigationState::new(Page::One);

    state.select(Page::One, start, AdaptiveLayout::NavigationRail);

    assert_eq!(state.selected(), Page::One);
    assert!(!state.is_animating());
    assert_eq!(state.selection().progress(Page::One), 1.0);

    assert!(!state.advance(start + Duration::from_millis(50)));
    assert!(!state.is_animating());
}

#[test]
fn navigation_rail_expansion_state_animates_between_open_and_closed() {
    let start = Instant::now();
    let mut state = NavigationRailExpansionState::new(false);

    assert!(!state.is_open());
    assert!(!state.is_visible());
    assert_eq!(state.progress(), 0.0);

    state.open(start);

    assert!(state.is_open());
    assert!(state.is_visible());
    assert!(state.is_animating());

    let still_animating = state.advance(start + Duration::from_millis(50));

    assert!(still_animating);
    assert!(state.progress() > 0.0);

    state.close(start + Duration::from_millis(50));

    assert!(!state.is_open());
    assert!(state.is_visible());
    assert!(state.is_animating());

    let finished = state.advance(start + Duration::from_millis(500));

    assert!(!finished);
    assert!(!state.is_visible());
    assert_eq!(state.progress(), 0.0);
}

#[test]
fn navigation_rail_expansion_progress_does_not_bounce_at_edges() {
    let start = Instant::now();
    let mut state = NavigationRailExpansionState::new(false);

    state.open(start);
    let mut previous = state.progress.value;

    for step in 1_u64..=24 {
        let _ = state.advance(start + Duration::from_millis(step * 16));

        let progress = state.progress.value;
        assert!((0.0..=1.0).contains(&progress));
        assert!(
            progress + f32::EPSILON >= previous,
            "open progress should be monotonic: {progress} < {previous}"
        );
        previous = progress;
    }

    let close_start = start + Duration::from_millis(500);
    let _ = state.advance(close_start);
    assert_eq!(state.progress.value, 1.0);

    state.close(close_start);
    previous = state.progress.value;

    for step in 1_u64..=24 {
        let _ = state.advance(close_start + Duration::from_millis(step * 16));

        let progress = state.progress.value;
        assert!((0.0..=1.0).contains(&progress));
        assert!(
            progress <= previous + f32::EPSILON,
            "close progress should be monotonic: {progress} > {previous}"
        );
        previous = progress;
    }
}

#[test]
fn active_indicator_width_follows_selection_progress() {
    let target = tokens::component::navigation_bar::ACTIVE_INDICATOR_WIDTH;

    assert_eq!(animated_indicator_width(target, -1.0), 0.0);
    assert_eq!(animated_indicator_width(target, 0.0), 0.0);
    assert_eq!(animated_indicator_width(target, 0.5), target / 2.0);
    assert_eq!(animated_indicator_width(target, 1.0), target);
    assert_eq!(animated_indicator_width(target, 2.0), target * 2.0);
}

#[test]
fn navigation_bar_item_geometry_matches_material_vertical_offsets() {
    assert_eq!(BarMetrics::item_bottom_padding(), 16.0);
}

#[test]
fn navigation_rail_item_geometry_matches_material_vertical_offsets() {
    assert_eq!(RailMetrics::item_content_top_padding(), 6.0);
}

#[test]
fn navigation_rail_header_geometry_matches_material_header_padding() {
    assert_eq!(RailMetrics::header_bottom_padding(), 40.0);
    assert_eq!(
        RailMetrics::header_slot_height(),
        tokens::component::icon_button::CONTAINER_HEIGHT
            + tokens::component::navigation_rail::HEADER_PADDING
    );
    assert_eq!(RailMetrics::header_slot_height(), 80.0);
}

#[test]
fn navigation_menu_header_does_not_jump_at_collapsed_endpoint() {
    let expanded = ExpandedRailMetrics::new(expanded_rail_width(0.0));
    let collapsed_button_x = (tokens::component::navigation_rail::CONTAINER_WIDTH
        - tokens::component::icon_button::CONTAINER_WIDTH)
        / 2.0;

    assert_eq!(expanded.header_leading_space(), collapsed_button_x);
    assert_eq!(
        expanded.header_leading_space() + tokens::component::icon_button::CONTAINER_WIDTH / 2.0,
        RailMetrics::collapsed_icon_center_x()
    );
    assert_eq!(
        RailMetrics::header_slot_height(),
        tokens::component::icon_button::CONTAINER_HEIGHT + RailMetrics::header_bottom_padding()
    );
}

#[test]
fn navigation_rail_min_height_fits_all_destinations_and_header() {
    assert_eq!(rail_min_height(5, true), 468.0);
    assert_eq!(rail_min_height(5, false), 384.0);
    assert_eq!(
        rail_min_height(1, true),
        tokens::component::navigation_rail::CONTENT_TOP_MARGIN
            + RailMetrics::header_slot_height()
            + tokens::component::navigation_rail::VERTICAL_PADDING
            + RailMetrics::item_slot_height()
            + tokens::component::navigation_rail::VERTICAL_PADDING
    );
}

#[test]
fn navigation_rail_fitting_content_sets_minimum_height() {
    let destinations = [
        Destination::new(Page::One, "1", "One"),
        Destination::new(Page::Two, "2", "Two"),
    ];
    let selection = Selection::new(Page::One);

    let rail: Container<'_, Message, Theme, iced_widget::Renderer> = rail_with(
        &destinations,
        selection,
        |_| Message::Frame,
        NavigationRailOptions::default().fit_content(),
    );
    let rail_size = iced_widget::core::Widget::<Message, Theme, iced_widget::Renderer>::size(&rail);
    assert_eq!(
        rail_size.height,
        Length::Fixed(rail_min_height(destinations.len(), false))
    );

    let rail: Container<'_, Message, Theme, iced_widget::Renderer> = rail_with(
        &destinations,
        selection,
        |_| Message::Frame,
        NavigationRailOptions::default()
            .menu(Message::Frame)
            .fit_content(),
    );
    let rail_size = iced_widget::core::Widget::<Message, Theme, iced_widget::Renderer>::size(&rail);
    assert_eq!(
        rail_size.height,
        Length::Fixed(rail_min_height(destinations.len(), true))
    );

    let rail: Container<'_, Message, Theme, iced_widget::Renderer> = expanded_rail_with(
        "Navigation",
        &destinations,
        selection,
        |_| Message::Frame,
        Message::Frame,
        ExpandedRailOptions::default().fit_content(),
    );
    let rail_size = iced_widget::core::Widget::<Message, Theme, iced_widget::Renderer>::size(&rail);
    assert_eq!(
        rail_size.height,
        Length::Fixed(rail_min_height(destinations.len(), true))
    );
}

#[test]
fn navigation_rail_expanded_geometry_matches_material_expressive_attributes() {
    fn assert_close(actual: f32, expected: f32) {
        assert!((actual - expected).abs() < 0.000_1);
    }
    let expanded =
        ExpandedRailMetrics::new(tokens::component::navigation_rail::EXPANDED_CONTAINER_WIDTH);

    assert_eq!(
        ExpandedRailMetrics::new(0.0).width(),
        tokens::component::navigation_rail::CONTAINER_WIDTH
    );
    assert_eq!(expanded.indicator_width(), 180.0);
    assert_eq!(expanded.header_leading_space(), 28.0);
    assert_eq!(expanded.header_title_spacing(), 0.0);
    assert_eq!(
        expanded_rail_width(0.0),
        tokens::component::navigation_rail::CONTAINER_WIDTH
    );
    assert_eq!(
        expanded_rail_width(1.0),
        tokens::component::navigation_rail::EXPANDED_CONTAINER_WIDTH
    );
    assert_eq!(
        ExpandedRailMetrics::new(tokens::component::navigation_rail::CONTAINER_WIDTH).progress(),
        0.0
    );
    assert_eq!(
        ExpandedRailMetrics::new(tokens::component::navigation_rail::EXPANDED_CONTAINER_WIDTH)
            .progress(),
        1.0
    );
    assert_eq!(
        ExpandedRailMetrics::new(expanded_rail_width(0.5)).progress(),
        0.5
    );
    assert_eq!(
        ExpandedRailMetrics::new(tokens::component::navigation_rail::CONTAINER_WIDTH).label_alpha(),
        0.0
    );
    assert_eq!(
        ExpandedRailMetrics::new(expanded_rail_width(0.5)).label_alpha(),
        0.0
    );
    assert_close(
        ExpandedRailMetrics::new(expanded_rail_width(0.8)).label_alpha(),
        0.5,
    );
    assert_eq!(ExpandedRailMetrics::collapsed_label_alpha_for(1.0), 0.0);
    assert_eq!(ExpandedRailMetrics::collapsed_label_alpha_for(0.5), 0.5);
    assert_eq!(ExpandedRailMetrics::collapsed_label_alpha_for(0.0), 1.0);
    assert_eq!(
        RailMetrics::collapsed_label_top_padding(),
        RailMetrics::item_content_top_padding()
            + tokens::component::navigation_rail::ACTIVE_INDICATOR_HEIGHT
            + tokens::component::navigation_rail::ITEM_VERTICAL_PADDING
    );
    assert_eq!(
        RailMetrics::collapsed_label_width(),
        tokens::component::navigation_rail::ACTIVE_INDICATOR_WIDTH
    );
    assert_close(
        ExpandedRailMetrics::new(tokens::component::navigation_rail::EXPANDED_CONTAINER_WIDTH)
            .label_alpha(),
        1.0,
    );
    assert!(ExpandedRailMetrics::badge_uses_icon_anchor_for(0.0));
    assert!(!ExpandedRailMetrics::badge_uses_icon_anchor_for(0.01));
    assert_eq!(ExpandedRailMetrics::trailing_badge_alpha_for(-1.0), 0.0);
    assert_eq!(ExpandedRailMetrics::trailing_badge_alpha_for(0.5), 0.5);
    assert_eq!(ExpandedRailMetrics::trailing_badge_alpha_for(2.0), 1.0);
    assert_eq!(
        ExpandedRailMetrics::indicator_height_for(0.0),
        tokens::component::navigation_rail::ACTIVE_INDICATOR_HEIGHT
    );
    assert_eq!(
        ExpandedRailMetrics::indicator_height_for(1.0),
        tokens::component::navigation_rail::EXPANDED_ACTIVE_INDICATOR_HEIGHT
    );
    assert_eq!(ExpandedRailMetrics::icon_anchor_width(), 40.0);
    assert_eq!(expanded.label_leading_padding(), 48.0);
    assert_eq!(
        expanded.expanded_icon_center_x(),
        RailMetrics::collapsed_icon_center_x()
    );
    assert_eq!(expanded.expanded_icon_center_x(), 48.0);
}

#[test]
fn expanded_rail_header_title_stays_single_line_while_collapsing() {
    let scale = tokens::component::navigation_drawer::HEADLINE_TEXT;
    let mut headline = single_line_type_text::<SingleLineTestRenderer>("Xiaomi Powerbank", scale);
    let mut tree = Tree::new(&headline as &dyn Widget<Message, Theme, SingleLineTestRenderer>);
    let renderer = SingleLineTestRenderer;
    let limits = layout::Limits::new(Size::ZERO, Size::new(24.0, 200.0));

    let node = Widget::<Message, Theme, SingleLineTestRenderer>::layout(
        &mut headline,
        &mut tree,
        &renderer,
        &limits,
    );
    let paragraph = tree
        .state
        .downcast_ref::<iced_widget::core::widget::text::State<SingleLineTestParagraph>>();

    assert_eq!(paragraph.as_text().wrapping, text::Wrapping::None);
    assert_eq!(node.size().height, scale.line_height);
}

#[test]
fn navigation_rail_expanded_keeps_collapsed_vertical_slots() {
    assert_eq!(
        RailMetrics::item_slot_height(),
        tokens::component::navigation_rail::ITEM_HEIGHT
    );
    assert_eq!(
        RailMetrics::first_item_y_after_header(),
        tokens::component::navigation_rail::CONTENT_TOP_MARGIN
            + tokens::component::icon_button::CONTAINER_HEIGHT
            + tokens::component::navigation_rail::HEADER_PADDING
            + tokens::component::navigation_rail::VERTICAL_PADDING
    );
    assert_eq!(RailMetrics::first_item_y_after_header(), 128.0);
    assert_eq!(ExpandedRailMetrics::expanded_item_vertical_inset(), 4.0);
    assert_eq!(
        ExpandedRailMetrics::item_vertical_inset_for(0.0),
        RailMetrics::item_content_top_padding()
    );
    assert_eq!(
        ExpandedRailMetrics::item_vertical_inset_for(1.0),
        ExpandedRailMetrics::expanded_item_vertical_inset()
    );
    assert_eq!(
        ExpandedRailMetrics::new(expanded_rail_width(0.0)).expanded_icon_center_y(),
        RailMetrics::collapsed_icon_center_y()
    );
    assert_eq!(RailMetrics::collapsed_icon_center_y(), 22.0);
}

#[derive(Debug, Clone, Copy, Default)]
struct SingleLineTestRenderer;

impl renderer::Renderer for SingleLineTestRenderer {
    fn start_layer(&mut self, _bounds: Rectangle) {}

    fn end_layer(&mut self) {}

    fn start_transformation(&mut self, _transformation: Transformation) {}

    fn end_transformation(&mut self) {}

    fn reset(&mut self, _new_bounds: Rectangle) {}

    fn fill_quad(&mut self, _quad: renderer::Quad, _background: impl Into<Background>) {}

    fn allocate_image(
        &mut self,
        _handle: &image::Handle,
        _callback: impl FnOnce(Result<image::Allocation, image::Error>) + Send + 'static,
    ) {
    }
}

impl geometry::Renderer for SingleLineTestRenderer {
    type Geometry = iced_widget::renderer::wgpu::geometry::Geometry;
    type Frame = iced_widget::renderer::wgpu::geometry::Frame;

    fn new_frame(&self, _bounds: Rectangle) -> Self::Frame {
        panic!("navigation layout and input tests do not draw canvas geometry")
    }

    fn draw_geometry(&mut self, _geometry: Self::Geometry) {}
}

impl primitive::Renderer for SingleLineTestRenderer {
    fn draw_primitive(&mut self, _bounds: Rectangle, _primitive: impl primitive::Primitive) {}
}

impl core_text::Renderer for SingleLineTestRenderer {
    type Font = Font;
    type Paragraph = SingleLineTestParagraph;
    type Editor = SingleLineTestEditor;

    const ICON_FONT: Self::Font = Font::DEFAULT;
    const CHECKMARK_ICON: char = '0';
    const ARROW_DOWN_ICON: char = '0';
    const SCROLL_UP_ICON: char = '0';
    const SCROLL_DOWN_ICON: char = '0';
    const SCROLL_LEFT_ICON: char = '0';
    const SCROLL_RIGHT_ICON: char = '0';
    const ICED_LOGO: char = '0';

    fn default_font(&self) -> Self::Font {
        Font::DEFAULT
    }

    fn default_size(&self) -> Pixels {
        Pixels(16.0)
    }

    fn fill_paragraph(
        &mut self,
        _text: &Self::Paragraph,
        _position: Point,
        _color: Color,
        _clip_bounds: Rectangle,
    ) {
    }

    fn fill_editor(
        &mut self,
        _editor: &Self::Editor,
        _position: Point,
        _color: Color,
        _clip_bounds: Rectangle,
    ) {
    }

    fn fill_text(
        &mut self,
        _text: core_text::Text<String, Self::Font>,
        _position: Point,
        _color: Color,
        _clip_bounds: Rectangle,
    ) {
    }
}

#[derive(Debug, Clone, Default)]
struct SingleLineTestEditor;

impl core_text::Editor for SingleLineTestEditor {
    type Font = Font;

    fn with_text(_text: &str) -> Self {
        Self
    }

    fn is_empty(&self) -> bool {
        true
    }

    fn cursor(&self) -> core_text::editor::Cursor {
        core_text::editor::Cursor {
            position: core_text::editor::Position { line: 0, column: 0 },
            selection: None,
        }
    }

    fn selection(&self) -> core_text::editor::Selection {
        core_text::editor::Selection::Caret(Point::ORIGIN)
    }

    fn copy(&self) -> Option<String> {
        None
    }

    fn line(&self, _index: usize) -> Option<core_text::editor::Line<'_>> {
        None
    }

    fn line_count(&self) -> usize {
        0
    }

    fn perform(&mut self, _action: core_text::editor::Action) {}

    fn move_to(&mut self, _cursor: core_text::editor::Cursor) {}

    fn bounds(&self) -> Size {
        Size::ZERO
    }

    fn min_bounds(&self) -> Size {
        Size::ZERO
    }

    fn update(
        &mut self,
        _new_bounds: Size,
        _new_font: Self::Font,
        _new_size: Pixels,
        _new_line_height: LineHeight,
        _new_wrapping: text::Wrapping,
        _new_highlighter: &mut impl core_text::Highlighter,
    ) {
    }

    fn highlight<H: core_text::Highlighter>(
        &mut self,
        _font: Self::Font,
        _highlighter: &mut H,
        _format_highlight: impl Fn(&H::Highlight) -> core_text::highlighter::Format<Self::Font>,
    ) {
    }
}

#[derive(Debug, Clone)]
struct SingleLineTestParagraph {
    content_width: f32,
    bounds: Size,
    size: Pixels,
    line_height: LineHeight,
    font: Font,
    align_x: text::Alignment,
    align_y: alignment::Vertical,
    wrapping: text::Wrapping,
    shaping: text::Shaping,
}

impl Default for SingleLineTestParagraph {
    fn default() -> Self {
        Self {
            content_width: 0.0,
            bounds: Size::ZERO,
            size: Pixels(16.0),
            line_height: LineHeight::default(),
            font: Font::DEFAULT,
            align_x: text::Alignment::Default,
            align_y: alignment::Vertical::Top,
            wrapping: text::Wrapping::default(),
            shaping: text::Shaping::default(),
        }
    }
}

impl core_text::paragraph::Paragraph for SingleLineTestParagraph {
    type Font = Font;

    fn with_text(text: core_text::Text<&str, Self::Font>) -> Self {
        Self {
            content_width: text.content.chars().count() as f32 * 8.0,
            bounds: text.bounds,
            size: text.size,
            line_height: text.line_height,
            font: text.font,
            align_x: text.align_x,
            align_y: text.align_y,
            wrapping: text.wrapping,
            shaping: text.shaping,
        }
    }

    fn with_spans<Link>(
        text: core_text::Text<&[core_text::Span<'_, Link, Self::Font>], Self::Font>,
    ) -> Self {
        Self {
            bounds: text.bounds,
            size: text.size,
            line_height: text.line_height,
            font: text.font,
            align_x: text.align_x,
            align_y: text.align_y,
            wrapping: text.wrapping,
            shaping: text.shaping,
            ..Self::default()
        }
    }

    fn resize(&mut self, new_bounds: Size) {
        self.bounds = new_bounds;
    }

    fn compare(&self, text: core_text::Text<(), Self::Font>) -> core_text::Difference {
        if self.size != text.size
            || self.line_height != text.line_height
            || self.font != text.font
            || self.align_x != text.align_x
            || self.align_y != text.align_y
            || self.wrapping != text.wrapping
            || self.shaping != text.shaping
        {
            core_text::Difference::Shape
        } else if self.bounds != text.bounds {
            core_text::Difference::Bounds
        } else {
            core_text::Difference::None
        }
    }

    fn size(&self) -> Pixels {
        self.size
    }

    fn font(&self) -> Self::Font {
        self.font
    }

    fn line_height(&self) -> LineHeight {
        self.line_height
    }

    fn align_x(&self) -> text::Alignment {
        self.align_x
    }

    fn align_y(&self) -> alignment::Vertical {
        self.align_y
    }

    fn wrapping(&self) -> text::Wrapping {
        self.wrapping
    }

    fn shaping(&self) -> text::Shaping {
        self.shaping
    }

    fn bounds(&self) -> Size {
        self.bounds
    }

    fn min_bounds(&self) -> Size {
        let line_height = self.line_height.to_absolute(self.size).0;

        if self.wrapping == text::Wrapping::None {
            return Size::new(self.content_width, line_height);
        }

        let line_count = (self.content_width / self.bounds.width.max(1.0))
            .ceil()
            .max(1.0);

        Size::new(
            self.content_width.min(self.bounds.width),
            line_height * line_count,
        )
    }

    fn hit_test(&self, _point: Point) -> Option<core_text::Hit> {
        None
    }

    fn hit_span(&self, _point: Point) -> Option<usize> {
        None
    }

    fn span_bounds(&self, _index: usize) -> Vec<Rectangle> {
        Vec::new()
    }

    fn grapheme_position(&self, _line: usize, _index: usize) -> Option<Point> {
        None
    }
}

#[test]
fn navigation_drawer_width_tracks_material_minimum_and_standard_widths() {
    assert_eq!(drawer_width(-1.0), 0.0);
    assert_eq!(drawer_width(0.0), 0.0);
    assert_eq!(
        drawer_width(0.5),
        (tokens::component::navigation_drawer::MINIMUM_CONTAINER_WIDTH
            + tokens::component::navigation_drawer::CONTAINER_WIDTH)
            / 2.0
    );
    assert_eq!(
        drawer_width(1.0),
        tokens::component::navigation_drawer::CONTAINER_WIDTH
    );
    assert_eq!(
        drawer_width(2.0),
        tokens::component::navigation_drawer::CONTAINER_WIDTH
    );
}

#[test]
fn modal_navigation_drawer_leaves_a_compact_scrim_target() {
    assert_eq!(modal_drawer_width(Some(360.0)), 304.0);
    assert_eq!(modal_drawer_width(Some(600.0)), 360.0);
    assert_eq!(modal_drawer_width(None), 360.0);
}

#[test]
fn modal_navigation_drawer_keeps_fixed_layout_while_translating() {
    let width = modal_drawer_width(Some(360.0));
    let metrics = DrawerMetrics::new(width);

    assert_eq!(metrics.width(), 304.0);
    assert_eq!(modal_drawer_offset(width, 0.0), -304.0);
    assert_eq!(modal_drawer_offset(width, 0.25), -228.0);
    assert_eq!(modal_drawer_offset(width, 0.333), -203.0);
    assert_eq!(modal_drawer_offset(width, 0.5), -152.0);
    assert_eq!(modal_drawer_offset(width, 1.0), 0.0);
    assert_eq!(metrics.width(), 304.0);
}

#[test]
fn navigation_drawer_indicator_width_matches_container_padding() {
    assert_eq!(
        DrawerMetrics::new(0.0).width(),
        tokens::component::navigation_drawer::MINIMUM_CONTAINER_WIDTH
    );
    assert_eq!(
        DrawerMetrics::new(tokens::component::navigation_drawer::CONTAINER_WIDTH).indicator_width(),
        tokens::component::navigation_drawer::ACTIVE_INDICATOR_WIDTH
    );
    assert_eq!(
        DrawerMetrics::new(tokens::component::navigation_drawer::MINIMUM_CONTAINER_WIDTH)
            .indicator_width(),
        tokens::component::navigation_drawer::MINIMUM_CONTAINER_WIDTH
            - tokens::component::navigation_drawer::ITEM_HORIZONTAL_PADDING * 2.0
    );
}

#[test]
fn navigation_drawer_menu_header_aligns_to_item_icon_and_label_columns() {
    assert_eq!(DrawerMetrics::menu_header_leading_space(), 20.0);
    assert_eq!(DrawerMetrics::menu_header_title_spacing(), 4.0);

    let menu_icon_center = DrawerMetrics::menu_header_leading_space()
        + tokens::component::icon_button::CONTAINER_WIDTH / 2.0;
    let drawer_icon_center = tokens::component::navigation_drawer::ITEM_HORIZONTAL_PADDING
        + tokens::component::navigation_drawer::ITEM_CONTENT_LEADING_SPACE
        + tokens::component::navigation_drawer::ICON_SIZE / 2.0;
    let menu_title_start = DrawerMetrics::menu_header_leading_space()
        + tokens::component::icon_button::CONTAINER_WIDTH
        + DrawerMetrics::menu_header_title_spacing();
    let drawer_label_start = tokens::component::navigation_drawer::ITEM_HORIZONTAL_PADDING
        + tokens::component::navigation_drawer::ITEM_CONTENT_LEADING_SPACE
        + tokens::component::navigation_drawer::ICON_SIZE
        + tokens::component::navigation_drawer::ICON_LABEL_SPACE;

    assert_eq!(menu_icon_center, drawer_icon_center);
    assert_eq!(menu_title_start, drawer_label_start);
}

#[test]
fn navigation_drawer_badge_spacing_matches_material_row_spacing() {
    assert_eq!(DrawerMetrics::badge_space(), 12.0);
}

#[test]
fn navigation_badges_use_material_badged_box_placement() {
    assert_eq!(
        destination_badge_placement(Badge::Small),
        badge_widget::BadgedBoxPlacement::IconOnly
    );
    assert_eq!(
        destination_badge_placement(Badge::Large("3")),
        badge_widget::BadgedBoxPlacement::WithContent
    );
}

#[test]
fn navigation_trailing_badge_alpha_follows_expanded_label_visibility() {
    let theme = Theme::Light;
    let style = alpha_badge_style(&theme, 0.25);
    let Some(Background::Color(background)) = style.background else {
        panic!("badge background should be a color");
    };

    assert_eq!(background.a, 0.25);
    assert_eq!(style.text_color.unwrap().a, 0.25);
}

#[test]
fn navigation_press_surface_uses_material_state_opacity_on_pill_only() {
    let theme = Theme::Light;

    assert_eq!(
        NavigationLayer::opacity(NavigationLayer::target(false, false)),
        0.0
    );
    assert_eq!(
        NavigationLayer::opacity(NavigationLayer::target(true, false)),
        HOVERED_LAYER_OPACITY
    );
    assert_eq!(
        NavigationLayer::opacity(NavigationLayer::target(false, true)),
        0.0
    );
    assert_eq!(
        NavigationLayer::opacity(NavigationLayer::target(true, true)),
        HOVERED_LAYER_OPACITY
    );
    assert_eq!(
        layer_color(&theme, NavigationStateLayer::BarOrRail),
        theme.colors().surface.text
    );
    assert_eq!(
        layer_color(&theme, NavigationStateLayer::Drawer { progress: 1.0 }),
        theme.colors().secondary.container_text
    );
    assert_eq!(
        state_layer(
            layer_color(&theme, NavigationStateLayer::BarOrRail),
            NavigationLayer::opacity(NavigationLayer::target(true, false))
        ),
        state_layer(theme.colors().surface.text, HOVERED_LAYER_OPACITY)
    );
}

#[test]
fn navigation_press_surface_click_keeps_hover_layer_independent_from_ripple() {
    let start = Instant::now();
    let mut state = NavigationPressSurfaceState::default();

    assert!(state.sync_hover(true, start));
    let _ = state.advance(start + duration_ms(tokens::motion::DURATION_SHORT2_MS));
    assert_eq!(state.opacity(), HOVERED_LAYER_OPACITY);

    state.press(Point::new(32.0, 16.0), start + duration_ms(200));

    assert_eq!(state.opacity(), HOVERED_LAYER_OPACITY);
    assert!(state.has_visible_ripples(start + duration_ms(220)));

    state.release(true, start + duration_ms(240));

    assert_eq!(state.opacity(), HOVERED_LAYER_OPACITY);
    assert!(state.has_visible_ripples(start + duration_ms(260)));
}

#[test]
fn navigation_redraw_without_cursor_does_not_resync_hover() {
    let redraw = Event::Window(window::Event::RedrawRequested(Instant::now()));

    assert!(
        !NavigationInteraction {
            event: &redraw,
            cursor: mouse::Cursor::Unavailable,
            is_hovered: false,
        }
        .should_sync_hover()
    );
    assert!(
        NavigationInteraction {
            event: &redraw,
            cursor: mouse::Cursor::Available(Point::new(48.0, 160.0)),
            is_hovered: false,
        }
        .should_sync_hover()
    );
}

#[test]
fn navigation_initial_redraw_hover_snaps_to_hover_layer_target() {
    let start = Instant::now();
    let redraw = Event::Window(window::Event::RedrawRequested(start));
    let mut state = NavigationPressSurfaceState::default();

    assert!(
        NavigationInteraction {
            event: &redraw,
            cursor: mouse::Cursor::Unavailable,
            is_hovered: true,
        }
        .should_snap_initial_redraw(&state)
    );
    assert!(state.sync_hover(true, start));

    state.snap_to_interaction_target();

    assert_eq!(state.opacity(), HOVERED_LAYER_OPACITY);
    assert!(!state.state_layer_opacity.is_animating());
}

#[test]
fn navigation_draw_uses_hover_layer_target_for_fresh_hovered_state() {
    let state = NavigationPressSurfaceState::default();
    let bounds = Rectangle::new(Point::new(0.0, 120.0), Size::new(80.0, 56.0));

    assert_eq!(
        NavigationDrawState {
            state: &state,
            cursor: mouse::Cursor::Available(Point::new(40.0, 148.0)),
            bounds,
        }
        .opacity(),
        HOVERED_LAYER_OPACITY
    );
    assert_eq!(
        NavigationDrawState {
            state: &state,
            cursor: mouse::Cursor::Available(Point::new(140.0, 148.0)),
            bounds,
        }
        .opacity(),
        0.0
    );
}

#[test]
fn navigation_draw_ignores_stale_cursor_when_initial_hover_is_disabled() {
    let state = NavigationPressSurfaceState::new(false);
    let bounds = Rectangle::new(Point::new(0.0, 120.0), Size::new(80.0, 56.0));

    assert_eq!(
        NavigationDrawState {
            state: &state,
            cursor: mouse::Cursor::Available(Point::new(40.0, 148.0)),
            bounds,
        }
        .opacity(),
        0.0
    );
}

#[test]
fn navigation_touch_disables_hover_until_real_mouse_motion() {
    let mut state = NavigationPressSurfaceState::default();
    let touch = Event::Touch(touch::Event::FingerLifted {
        id: touch::Finger(0),
        position: Point::new(40.0, 148.0),
    });
    state.observe_pointer_kind(&touch);

    assert!(!state.hover_enabled);

    let mouse_move = Event::Mouse(mouse::Event::CursorMoved {
        position: Point::new(40.0, 148.0),
    });
    state.observe_pointer_kind(&mouse_move);

    assert!(state.hover_enabled);
}

struct NavigationInputHarness {
    surface: NavigationPressSurface<'static, Message, SingleLineTestRenderer>,
    tree: Tree,
    node: layout::Node,
    messages: Vec<Message>,
}

impl NavigationInputHarness {
    fn new() -> Self {
        let mut surface = press_surface(
            Space::new().width(120.0).height(80.0),
            Message::Frame,
            NavigationStateLayer::BarOrRail,
            NavigationIndicatorPlacement::TopCenter {
                top: 12.0,
                width: 64.0,
                height: 32.0,
            },
        );
        let mut tree = Tree::new(&surface as &dyn Widget<Message, Theme, SingleLineTestRenderer>);
        let node = surface
            .layout(
                &mut tree,
                &SingleLineTestRenderer,
                &layout::Limits::new(Size::ZERO, Size::new(120.0, 80.0)),
            )
            .move_to(Point::new(0.0, 720.0));
        Self {
            surface,
            tree,
            node,
            messages: Vec::new(),
        }
    }

    fn send(&mut self, event: Event, cursor: mouse::Cursor) -> bool {
        let mut shell = Shell::new(&mut self.messages);
        self.surface.update(
            &mut self.tree,
            &event,
            Layout::new(&self.node),
            cursor,
            &SingleLineTestRenderer,
            &mut iced_widget::core::clipboard::Null,
            &mut shell,
            &Rectangle::with_size(Size::new(360.0, 800.0)),
        );
        shell.is_event_captured()
    }

    fn touch(&mut self, event: touch::Event) -> bool {
        self.send(Event::Touch(event), mouse::Cursor::Unavailable)
    }

    fn state(&self) -> &NavigationPressSurfaceState {
        self.tree
            .state
            .downcast_ref::<NavigationPressSurfaceState>()
    }
}

#[test]
fn navigation_widget_tap_with_small_motion_selects_once_without_hover() {
    let mut input = NavigationInputHarness::new();
    let id = touch::Finger(0);
    assert!(!input.touch(touch::Event::FingerPressed {
        id,
        position: Point::new(40.0, 750.0)
    }));
    let _ = input.touch(touch::Event::FingerMoved {
        id,
        position: Point::new(44.0, 752.0),
    });
    assert!(input.touch(touch::Event::FingerLifted {
        id,
        position: Point::new(46.0, 752.0)
    }));
    assert!(!input.touch(touch::Event::FingerLifted {
        id,
        position: Point::new(46.0, 752.0)
    }));
    assert_eq!(input.messages.len(), 1);
    assert!(!input.state().is_hovered);
    assert!(!input.state().is_pressed);
    assert!(input.state().active_press.is_none());
}

#[test]
fn navigation_widget_drag_cancels_even_if_finger_returns_to_item() {
    let mut input = NavigationInputHarness::new();
    let id = touch::Finger(0);
    let _ = input.touch(touch::Event::FingerPressed {
        id,
        position: Point::new(40.0, 750.0),
    });
    assert!(!input.touch(touch::Event::FingerMoved {
        id,
        position: Point::new(40.0, 780.0)
    }));
    let _ = input.touch(touch::Event::FingerMoved {
        id,
        position: Point::new(40.0, 750.0),
    });
    let _ = input.touch(touch::Event::FingerLifted {
        id,
        position: Point::new(40.0, 750.0),
    });
    assert!(input.messages.is_empty());
    assert!(!input.state().is_pressed);
    assert!(!input.state().has_visible_ripples(Instant::now()));
}

#[test]
fn navigation_widget_release_checks_touch_distance_when_move_was_not_delivered() {
    let mut input = NavigationInputHarness::new();
    let id = touch::Finger(0);
    let _ = input.touch(touch::Event::FingerPressed {
        id,
        position: Point::new(20.0, 750.0),
    });
    let _ = input.touch(touch::Event::FingerLifted {
        id,
        position: Point::new(90.0, 750.0),
    });
    assert!(input.messages.is_empty());
    assert!(!input.state().is_pressed);
}

#[test]
fn navigation_widget_keeps_touch_ownership_across_other_fingers_and_mouse_events() {
    let mut input = NavigationInputHarness::new();
    let position = Point::new(40.0, 750.0);
    let id = touch::Finger(0);
    let other = touch::Finger(1);
    let _ = input.touch(touch::Event::FingerPressed { id, position });
    for event in [
        touch::Event::FingerPressed {
            id: other,
            position,
        },
        touch::Event::FingerMoved {
            id: other,
            position: Point::new(90.0, 750.0),
        },
        touch::Event::FingerLifted {
            id: other,
            position,
        },
        touch::Event::FingerLost {
            id: other,
            position,
        },
    ] {
        assert!(!input.touch(event));
    }
    for event in [
        mouse::Event::ButtonPressed(mouse::Button::Left),
        mouse::Event::ButtonReleased(mouse::Button::Left),
    ] {
        assert!(!input.send(Event::Mouse(event), mouse::Cursor::Available(position)));
    }
    assert!(input.messages.is_empty());
    assert!(input.state().is_pressed);
    let _ = input.touch(touch::Event::FingerLifted { id, position });
    assert_eq!(input.messages.len(), 1);
}

#[test]
fn navigation_widget_keeps_mouse_ownership_across_touch_events() {
    let mut input = NavigationInputHarness::new();
    let position = Point::new(40.0, 750.0);
    let cursor = mouse::Cursor::Available(position);
    let _ = input.send(
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
        cursor,
    );
    let _ = input.touch(touch::Event::FingerPressed {
        id: touch::Finger(0),
        position,
    });
    let _ = input.touch(touch::Event::FingerLifted {
        id: touch::Finger(0),
        position,
    });
    assert!(input.messages.is_empty());
    assert!(input.state().is_pressed);
    let _ = input.send(
        Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
        cursor,
    );
    assert_eq!(input.messages.len(), 1);
}

#[test]
fn navigation_widget_mouse_click_allows_taps_and_motion_up_to_slop() {
    let origin = Point::new(40.0, 750.0);
    for movement in [0.0, 4.0, 8.0] {
        let mut input = NavigationInputHarness::new();
        let _ = input.send(
            Event::Mouse(mouse::Event::CursorMoved { position: origin }),
            mouse::Cursor::Available(origin),
        );
        assert!(input.send(
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
            mouse::Cursor::Available(origin),
        ));
        let position = origin + Vector::new(movement, 0.0);
        let _ = input.send(
            Event::Mouse(mouse::Event::CursorMoved { position }),
            mouse::Cursor::Available(position),
        );
        assert!(input.send(
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
            mouse::Cursor::Available(position),
        ));
        assert!(!input.send(
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
            mouse::Cursor::Available(position),
        ));
        assert_eq!(input.messages.len(), 1, "movement={movement}");
        assert!(!input.state().is_pressed);
    }
}

#[test]
fn navigation_widget_mouse_drag_cancels_inside_item_and_after_leaving_and_returning() {
    let origin = Point::new(40.0, 750.0);
    for position in [Point::new(49.0, 750.0), Point::new(150.0, 750.0)] {
        let mut input = NavigationInputHarness::new();
        let _ = input.send(
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
            mouse::Cursor::Available(origin),
        );
        assert!(!input.send(
            Event::Mouse(mouse::Event::CursorMoved { position }),
            mouse::Cursor::Available(position),
        ));
        let _ = input.send(
            Event::Mouse(mouse::Event::CursorMoved { position: origin }),
            mouse::Cursor::Available(origin),
        );
        let _ = input.send(
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
            mouse::Cursor::Available(origin),
        );
        assert!(input.messages.is_empty(), "drag position={position:?}");
        assert!(!input.state().is_pressed);
        assert!(input.state().active_press.is_none());
        assert!(!input.state().has_visible_ripples(Instant::now()));
    }
}

#[test]
fn navigation_widget_mouse_motion_uses_event_position_when_batch_cursor_has_returned() {
    let raw_origin = Point::new(40.0, 50.0);
    let translated_origin = Point::new(40.0, 750.0);
    for movement in [8.0, 9.0] {
        let mut input = NavigationInputHarness::new();
        let cursor = mouse::Cursor::Available(translated_origin);
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
        for position in [raw_origin + Vector::new(movement, 0.0), raw_origin] {
            let _ = input.send(Event::Mouse(mouse::Event::CursorMoved { position }), cursor);
        }
        let _ = input.send(
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
            cursor,
        );
        assert_eq!(input.messages.len(), usize::from(movement <= 8.0));
        assert!(!input.state().is_pressed);
    }
}

#[test]
fn navigation_widget_mouse_release_checks_distance_without_move_event() {
    let mut input = NavigationInputHarness::new();
    let origin = Point::new(40.0, 750.0);
    let _ = input.send(
        Event::Mouse(mouse::Event::CursorMoved { position: origin }),
        mouse::Cursor::Available(origin),
    );
    let _ = input.send(
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
        mouse::Cursor::Available(origin),
    );
    let _ = input.send(
        Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
        mouse::Cursor::Available(Point::new(90.0, 750.0)),
    );
    assert!(input.messages.is_empty());
    assert!(!input.state().is_pressed);
}

#[test]
fn navigation_widget_mouse_loss_cancels_press_and_allows_next_click() {
    let position = Point::new(40.0, 750.0);
    let cursor = mouse::Cursor::Available(position);
    for cancel in [
        Event::Mouse(mouse::Event::CursorLeft),
        Event::Window(window::Event::Unfocused),
    ] {
        let mut input = NavigationInputHarness::new();
        let _ = input.send(
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
            cursor,
        );
        let _ = input.send(cancel, mouse::Cursor::Unavailable);
        let _ = input.send(
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
            cursor,
        );
        assert!(input.messages.is_empty());
        assert!(input.state().active_press.is_none());
        assert!(!input.state().is_pressed);
        assert!(!input.state().has_visible_ripples(Instant::now()));
        let _ = input.send(
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
            cursor,
        );
        let _ = input.send(
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
            cursor,
        );
        assert_eq!(input.messages.len(), 1);
    }
}

#[test]
fn navigation_widget_cancels_on_owner_loss_and_window_unfocus() {
    let position = Point::new(40.0, 750.0);
    let id = touch::Finger(0);
    for cancel in [
        Event::Touch(touch::Event::FingerLost { id, position }),
        Event::Window(window::Event::Unfocused),
    ] {
        let mut input = NavigationInputHarness::new();
        let _ = input.touch(touch::Event::FingerPressed { id, position });
        let _ = input.send(cancel, mouse::Cursor::Unavailable);
        let _ = input.touch(touch::Event::FingerLifted { id, position });
        assert!(input.messages.is_empty());
        assert!(input.state().active_press.is_none());
        assert!(!input.state().has_visible_ripples(Instant::now()));
    }
}

#[test]
fn navigation_widget_uses_translated_cursor_for_touch_hit_testing() {
    let mut input = NavigationInputHarness::new();
    let id = touch::Finger(0);
    for (event, position) in [
        (
            touch::Event::FingerPressed {
                id,
                position: Point::new(40.0, 50.0),
            },
            Point::new(40.0, 750.0),
        ),
        (
            touch::Event::FingerMoved {
                id,
                position: Point::new(40.0, 52.0),
            },
            Point::new(40.0, 752.0),
        ),
        (
            touch::Event::FingerLifted {
                id,
                position: Point::new(40.0, 54.0),
            },
            Point::new(40.0, 754.0),
        ),
    ] {
        let _ = input.send(Event::Touch(event), mouse::Cursor::Available(position));
    }
    assert_eq!(input.messages.len(), 1);
}

#[test]
fn navigation_touch_drag_is_not_hidden_by_scroll_translation() {
    let mut input = NavigationInputHarness::new();
    let id = touch::Finger(0);
    for (event, position) in [
        (
            touch::Event::FingerPressed {
                id,
                position: Point::new(40.0, 50.0),
            },
            Point::new(40.0, 750.0),
        ),
        (
            touch::Event::FingerMoved {
                id,
                position: Point::new(40.0, 100.0),
            },
            Point::new(40.0, 752.0),
        ),
        (
            touch::Event::FingerLifted {
                id,
                position: Point::new(40.0, 150.0),
            },
            Point::new(40.0, 754.0),
        ),
    ] {
        let _ = input.send(Event::Touch(event), mouse::Cursor::Available(position));
    }
    assert!(input.messages.is_empty());
}

#[test]
fn navigation_widget_cancels_touch_that_leaves_visible_hit_region() {
    let mut input = NavigationInputHarness::new();
    let position = Point::new(40.0, 750.0);
    let id = touch::Finger(0);
    let _ = input.touch(touch::Event::FingerPressed { id, position });
    let _ = input.send(
        Event::Touch(touch::Event::FingerMoved { id, position }),
        mouse::Cursor::Levitating(position),
    );
    let _ = input.touch(touch::Event::FingerLifted { id, position });
    assert!(input.messages.is_empty());
    assert!(!input.state().is_pressed);
}

#[test]
fn navigation_bar_and_rail_inside_scrollable_allow_touch_scrolling_and_taps() {
    #[derive(Debug, Clone)]
    enum ScrollMessage {
        Selected,
        Scrolled(f32),
    }

    fn item_is_pressed(tree: &Tree) -> bool {
        if tree.tag == tree::Tag::of::<NavigationPressSurfaceState>() {
            tree.state
                .downcast_ref::<NavigationPressSurfaceState>()
                .is_pressed
        } else {
            tree.children.iter().any(item_is_pressed)
        }
    }

    fn item_bounds(tree: &Tree, layout: Layout<'_>) -> Option<Rectangle> {
        if tree.tag == tree::Tag::of::<NavigationPressSurfaceState>() {
            Some(layout.bounds())
        } else {
            tree.children
                .iter()
                .zip(layout.children())
                .find_map(|(tree, layout)| item_bounds(tree, layout))
        }
    }

    for is_rail in [false, true] {
        for drag in [false, true] {
            let destinations = [Destination::new(Page::One, "1", "One")];
            let navigation: Element<'_, ScrollMessage, Theme, SingleLineTestRenderer> = if is_rail {
                rail_with(
                    &destinations,
                    Selection::new(Page::One),
                    |_| ScrollMessage::Selected,
                    NavigationRailOptions::default().fit_content(),
                )
                .into()
            } else {
                bar(&destinations, Selection::new(Page::One), |_| {
                    ScrollMessage::Selected
                })
                .into()
            };
            let content = Column::new()
                .push(Space::new().height(60))
                .push(navigation)
                .push(Space::new().height(900));
            let mut scrollable = iced_widget::Scrollable::new(content)
                .width(240)
                .height(200)
                .on_scroll(|viewport| ScrollMessage::Scrolled(viewport.absolute_offset().y));
            let renderer = SingleLineTestRenderer;
            let mut tree =
                Tree::new(&scrollable as &dyn Widget<ScrollMessage, Theme, SingleLineTestRenderer>);
            let viewport = Rectangle::with_size(Size::new(240.0, 200.0));
            let node = scrollable.layout(
                &mut tree,
                &renderer,
                &layout::Limits::new(Size::ZERO, viewport.size()),
            );
            let origin = item_bounds(&tree, Layout::new(&node)).unwrap().center();
            let end = if drag {
                Point::new(origin.x, origin.y - 60.0)
            } else {
                origin
            };
            let mut messages = Vec::new();
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
                    assert!(item_is_pressed(&tree), "is_rail={is_rail}");
                    assert!(messages.is_empty());
                }
            }

            let did_scroll = messages
                .iter()
                .any(|message| matches!(message, ScrollMessage::Scrolled(y) if *y > 0.0));
            let selected = messages
                .iter()
                .filter(|message| matches!(message, ScrollMessage::Selected))
                .count();
            assert_eq!(
                did_scroll, drag,
                "is_rail={is_rail}, drag={drag}, {messages:?}"
            );
            assert_eq!(selected, usize::from(!drag), "is_rail={is_rail}");
            assert!(!item_is_pressed(&tree));
        }
    }
}

#[test]
fn navigation_bar_safe_area_is_not_a_touch_target() {
    let destinations = [Destination::new(Page::One, "1", "One")];
    let mut bar = bar_with(
        &destinations,
        Selection::new(Page::One),
        |_| Message::Frame,
        NavigationBarOptions::default().insets(Padding {
            bottom: 24.0,
            ..Padding::ZERO
        }),
    );
    let mut tree = Tree::new(&bar as &dyn Widget<Message, Theme, SingleLineTestRenderer>);
    let node = bar.layout(
        &mut tree,
        &SingleLineTestRenderer,
        &layout::Limits::new(Size::ZERO, Size::new(360.0, 104.0)),
    );
    let mut messages = Vec::new();
    for y in [92.0, 40.0] {
        for event in [
            touch::Event::FingerPressed {
                id: touch::Finger(0),
                position: Point::new(180.0, y),
            },
            touch::Event::FingerLifted {
                id: touch::Finger(0),
                position: Point::new(180.0, y),
            },
        ] {
            bar.update(
                &mut tree,
                &Event::Touch(event),
                Layout::new(&node),
                mouse::Cursor::Unavailable,
                &SingleLineTestRenderer,
                &mut iced_widget::core::clipboard::Null,
                &mut Shell::new(&mut messages),
                &Rectangle::with_size(Size::new(360.0, 104.0)),
            );
        }
        assert_eq!(messages.len(), usize::from(y < 80.0));
    }
}

#[test]
fn navigation_draw_keeps_mouse_hover_enter_animation_after_first_frame() {
    let start = Instant::now();
    let mut state = NavigationPressSurfaceState::default();
    let bounds = Rectangle::new(Point::new(0.0, 120.0), Size::new(80.0, 56.0));

    assert!(state.sync_hover(true, start));
    let _ = state.advance(start);

    assert_eq!(
        NavigationDrawState {
            state: &state,
            cursor: mouse::Cursor::Available(Point::new(40.0, 148.0)),
            bounds,
        }
        .opacity(),
        0.0
    );
}

#[test]
fn navigation_press_surface_keeps_release_ripple_visible() {
    let start = Instant::now();
    let mut state = NavigationPressSurfaceState::default();

    assert!(state.sync_hover(true, start));
    state.press(Point::new(32.0, 16.0), start);

    assert_eq!(state.opacity(), 0.0);
    assert!(state.has_visible_ripples(start + duration_ms(75)));

    state.release(true, start);
    let still_animating = state.advance(start + Duration::from_millis(50));

    assert!(still_animating);
    assert!(state.has_visible_ripples(start + Duration::from_millis(50)));

    let finished = state.advance(
        start
            + duration_ms(
                tokens::state::RIPPLE_PATTERN_ENTER_DURATION_MS
                    + tokens::state::RIPPLE_PATTERN_EXIT_DURATION_MS,
            )
            + Duration::from_millis(1),
    );

    assert!(!finished);
    assert_eq!(state.opacity(), HOVERED_LAYER_OPACITY);
    assert!(!state.has_visible_ripples(
        start
            + duration_ms(
                tokens::state::RIPPLE_PATTERN_ENTER_DURATION_MS
                    + tokens::state::RIPPLE_PATTERN_EXIT_DURATION_MS,
            )
            + Duration::from_millis(1)
    ));
}

#[test]
fn navigation_press_surface_touch_release_keeps_ripple_without_hover_layer() {
    let start = Instant::now();
    let mut state = NavigationPressSurfaceState::default();

    state.press(Point::new(32.0, 16.0), start);
    state.release_with_hover(true, false, start + duration_ms(20));

    assert!(!state.is_hovered);
    assert!(state.has_visible_ripples(start + duration_ms(75)));
    assert_eq!(state.state_layer_opacity.to, 0.0);
}

#[test]
fn navigation_press_surface_clears_release_ripple_when_pointer_leaves_item() {
    let start = Instant::now();
    let mut state = NavigationPressSurfaceState::default();

    assert!(state.sync_hover(true, start));
    state.press(Point::new(32.0, 16.0), start);
    state.release(true, start);

    assert!(state.has_visible_ripples(start + duration_ms(75)));
    assert!(state.sync_hover(false, start + duration_ms(80)));
    assert!(!state.has_visible_ripples(start + duration_ms(80)));
}

#[test]
fn navigation_press_surface_discards_ripple_released_outside_item() {
    let start = Instant::now();
    let mut state = NavigationPressSurfaceState::default();

    assert!(state.sync_hover(true, start));
    state.press(Point::new(32.0, 16.0), start);
    state.release(false, start + duration_ms(20));

    assert!(!state.has_visible_ripples(start + duration_ms(75)));
    assert_eq!(state.ripples.exiting_ripple_count(), 0);
}

#[test]
fn navigation_press_surface_replaces_fast_repeated_ripples() {
    let start = Instant::now();
    let mut state = NavigationPressSurfaceState::default();

    assert!(state.sync_hover(true, start));
    state.press(Point::new(20.0, 16.0), start);
    state.release(true, start + duration_ms(20));
    state.press(Point::new(44.0, 16.0), start + duration_ms(40));

    assert!(state.ripples.has_active_ripple());
    assert_eq!(state.ripples.exiting_ripple_count(), 0);
}

#[test]
fn navigation_ripple_matches_compose_bounded_radius_for_indicator_bounds() {
    let radius = ripple_target_radius(Size::new(64.0, 32.0));

    assert!(
        (radius
            - ((32.0_f32 * 32.0 + 16.0 * 16.0).sqrt()
                + tokens::state::RIPPLE_BOUNDED_EXTRA_RADIUS))
            .abs()
            < 0.001
    );
}

fn pointer<'a>(event: &'a Event, cursor: mouse::Cursor) -> NavigationPointer<'a> {
    NavigationPointer { event, cursor }
}

#[test]
fn navigation_ripple_origin_uses_indicator_local_coordinates() {
    let indicator_bounds = Rectangle {
        x: 28.0,
        y: 12.0,
        width: 64.0,
        height: 32.0,
    };
    let event = Event::Touch(touch::Event::FingerPressed {
        id: touch::Finger(0),
        position: Point::new(20.0, 56.0),
    });

    let origin = pointer(&event, mouse::Cursor::Unavailable)
        .press_origin(indicator_bounds)
        .unwrap();

    assert_eq!(origin, Point::new(-8.0, 44.0));
}

#[test]
fn navigation_touch_hit_test_uses_finger_position_without_cursor() {
    let bounds = Rectangle::new(Point::new(0.0, 720.0), Size::new(120.0, 80.0));
    let event = Event::Touch(touch::Event::FingerPressed {
        id: touch::Finger(0),
        position: Point::new(48.0, 760.0),
    });

    assert!(pointer(&event, mouse::Cursor::Unavailable).is_over(bounds));
}

#[test]
fn navigation_touch_hit_test_prefers_translated_cursor_position() {
    let bounds = Rectangle::new(Point::new(0.0, 720.0), Size::new(120.0, 80.0));
    let event = Event::Touch(touch::Event::FingerPressed {
        id: touch::Finger(0),
        position: Point::new(48.0, 160.0),
    });

    assert!(pointer(&event, mouse::Cursor::Available(Point::new(48.0, 760.0))).is_over(bounds));
}

#[test]
fn navigation_touch_hit_test_does_not_fallback_when_cursor_is_available() {
    let bounds = Rectangle::new(Point::new(0.0, 720.0), Size::new(120.0, 80.0));
    let event = Event::Touch(touch::Event::FingerPressed {
        id: touch::Finger(0),
        position: Point::new(48.0, 760.0),
    });

    assert!(!pointer(&event, mouse::Cursor::Available(Point::new(48.0, 160.0))).is_over(bounds));
}

#[test]
fn navigation_touch_hit_test_does_not_fallback_when_cursor_is_levitating() {
    let bounds = Rectangle::new(Point::new(0.0, 720.0), Size::new(120.0, 80.0));
    let event = Event::Touch(touch::Event::FingerPressed {
        id: touch::Finger(0),
        position: Point::new(48.0, 760.0),
    });

    assert!(!pointer(&event, mouse::Cursor::Levitating(Point::new(48.0, 160.0))).is_over(bounds));
}

#[test]
fn navigation_touch_origin_prefers_translated_cursor_position() {
    let indicator_bounds = Rectangle {
        x: 28.0,
        y: 720.0,
        width: 64.0,
        height: 32.0,
    };
    let event = Event::Touch(touch::Event::FingerPressed {
        id: touch::Finger(0),
        position: Point::new(20.0, 56.0),
    });

    let origin = pointer(&event, mouse::Cursor::Available(Point::new(40.0, 736.0)))
        .press_origin(indicator_bounds)
        .unwrap();

    assert_eq!(origin, Point::new(12.0, 16.0));
}

#[test]
fn navigation_touch_origin_does_not_fallback_when_cursor_is_levitating() {
    let indicator_bounds = Rectangle {
        x: 28.0,
        y: 720.0,
        width: 64.0,
        height: 32.0,
    };
    let event = Event::Touch(touch::Event::FingerPressed {
        id: touch::Finger(0),
        position: Point::new(40.0, 736.0),
    });

    assert_eq!(
        pointer(&event, mouse::Cursor::Levitating(Point::new(40.0, 160.0)))
            .press_origin(indicator_bounds),
        None
    );
}

#[test]
fn navigation_touch_hit_test_rejects_positions_outside_bounds() {
    let bounds = Rectangle::new(Point::new(0.0, 720.0), Size::new(120.0, 80.0));
    let event = Event::Touch(touch::Event::FingerPressed {
        id: touch::Finger(0),
        position: Point::new(48.0, 680.0),
    });

    assert!(!pointer(&event, mouse::Cursor::Unavailable).is_over(bounds));
}

#[test]
fn navigation_rounded_rect_span_clips_full_round_indicator() {
    fn assert_close(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() < 0.001,
            "expected {expected}, got {actual}"
        );
    }

    let size = Size::new(64.0, 32.0);
    let radius = border::radius(tokens::shape::CORNER_FULL);

    let top = rounded_rect_span_at_y(size, radius, 0.0).unwrap();
    assert_close(top.0, 16.0);
    assert_close(top.1, 48.0);

    let middle = rounded_rect_span_at_y(size, radius, 16.0).unwrap();
    assert_close(middle.0, 0.0);
    assert_close(middle.1, 64.0);
}

#[test]
fn navigation_press_surface_indicator_bounds_follow_material_geometry() {
    let bounds = Rectangle {
        x: 10.0,
        y: 20.0,
        width: 100.0,
        height: 80.0,
    };

    let top_center = NavigationIndicatorPlacement::TopCenter {
        top: 12.0,
        width: 64.0,
        height: 32.0,
    }
    .bounds(bounds);

    assert_eq!(
        top_center,
        Rectangle {
            x: 28.0,
            y: 32.0,
            width: 64.0,
            height: 32.0
        }
    );

    let inset = NavigationIndicatorPlacement::Inset {
        x: 2.0,
        y: 4.0,
        width: 56.0,
        height: 32.0,
    }
    .bounds(bounds);

    assert_eq!(
        inset,
        Rectangle {
            x: 12.0,
            y: 24.0,
            width: 56.0,
            height: 32.0
        }
    );

    assert_eq!(NavigationIndicatorPlacement::Full.bounds(bounds), bounds);
}

#[test]
fn destination_icons_crossfade_outline_and_filled_faces_for_selected_state() {
    let theme = Theme::Light;

    let outline_unselected = destination_icon_outline_color(&theme, 0.0);
    let filled_unselected = destination_icon_filled_color(&theme, 0.0, false);

    assert_eq!(outline_unselected, theme.colors().surface.text_variant);
    assert_eq!(filled_unselected.a, 0.0);

    let outline_selected = destination_icon_outline_color(&theme, 1.0);
    let filled_selected = destination_icon_filled_color(&theme, 1.0, false);

    assert_eq!(outline_selected.a, 0.0);
    assert_eq!(filled_selected, theme.colors().secondary.container_text);

    let outline_mid = destination_icon_outline_color(&theme, 0.5);
    let filled_mid = destination_icon_filled_color(&theme, 0.5, true);

    assert_eq!(outline_mid.a, theme.colors().surface.text_variant.a * 0.5);
    assert_eq!(
        filled_mid.a,
        theme.colors().secondary.container_text.a * 0.5
    );
}
