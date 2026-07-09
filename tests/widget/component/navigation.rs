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
    assert_eq!(RailMetrics::header_slot_height(), 80.0);
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
