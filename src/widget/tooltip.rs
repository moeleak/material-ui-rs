//! Material 3 tooltip constructors with token-backed layout defaults.

use super::*;

use iced_widget::core::Transformation;

pub use iced_tooltip::Position;

pub fn plain<'a, Message, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
    supporting_text: impl text::IntoFragment<'a>,
    position: Position,
) -> PlainTooltip<'a, Message, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
{
    let type_scale = tokens::component::tooltip::PLAIN_SUPPORTING_TEXT;

    let tooltip = Container::new(plain_supporting_text::<Renderer>(
        supporting_text,
        type_scale,
    ))
    .padding(Padding {
        top: 0.0,
        right: plain_tooltip_inner_horizontal_padding(),
        bottom: 0.0,
        left: plain_tooltip_inner_horizontal_padding(),
    })
    .max_width(plain_tooltip_inner_max_width());

    RichTooltip::new(content, tooltip, position)
        .gap(tokens::component::tooltip::SPACING_BETWEEN_TOOLTIP_AND_ANCHOR)
        .padding(tokens::component::tooltip::PLAIN_VERTICAL_SPACE)
        .interactive_surface(false)
        .style(tooltip_style::plain)
}

pub fn rich<'a, Message, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
    supporting_text: impl text::IntoFragment<'a>,
    position: Position,
) -> RichTooltip<'a, Message, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
{
    rich_surface(content, None, supporting_text, None, position)
}

pub fn rich_with_title<'a, Message, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
    title: impl text::IntoFragment<'a>,
    supporting_text: impl text::IntoFragment<'a>,
    position: Position,
) -> RichTooltip<'a, Message, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
{
    rich_surface(
        content,
        Some(title.into_fragment()),
        supporting_text,
        None,
        position,
    )
}

pub fn rich_with_title_action<'a, Message, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
    title: impl text::IntoFragment<'a>,
    supporting_text: impl text::IntoFragment<'a>,
    action: impl Into<Element<'a, Message, Theme, Renderer>>,
    position: Position,
) -> RichTooltip<'a, Message, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
{
    rich_surface(
        content,
        Some(title.into_fragment()),
        supporting_text,
        Some(action.into()),
        position,
    )
}

pub fn rich_action<'a, Message, Renderer>(
    label: impl text::IntoFragment<'a>,
) -> button::Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: iced_widget::graphics::geometry::Renderer + core_text::Renderer + 'a,
{
    button::text(label)
}

pub fn rich_action_button<'a, Message, Renderer>(
    label: impl text::IntoFragment<'a>,
    on_press: Message,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: iced_widget::graphics::geometry::Renderer + core_text::Renderer + 'a,
{
    rich_action(label).on_press(on_press).into()
}

fn plain_supporting_text<'a, Renderer>(
    supporting_text: impl text::IntoFragment<'a>,
    type_scale: tokens::typography::TypeScale,
) -> Text<'a, Theme, Renderer>
where
    Renderer: core_text::Renderer,
{
    text_with_metrics(supporting_text, type_scale.size, type_scale.line_height)
        .wrapping(text::Wrapping::Word)
}

fn rich_surface<'a, Message, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
    title: Option<text::Fragment<'a>>,
    supporting_text: impl text::IntoFragment<'a>,
    action: Option<Element<'a, Message, Theme, Renderer>>,
    position: Position,
) -> RichTooltip<'a, Message, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
{
    let has_title = title.is_some();
    let has_action = action.is_some();
    let mut tooltip = iced_widget::Column::new()
        .width(Length::Fill)
        .padding(Padding {
            top: 0.0,
            right: tokens::component::tooltip::RICH_HORIZONTAL_SPACE,
            bottom: 0.0,
            left: tokens::component::tooltip::RICH_HORIZONTAL_SPACE,
        });

    if let Some(title) = title {
        tooltip = tooltip.push(
            Container::new(rich_title_text::<Renderer>(title))
                .padding(Padding {
                    top: rich_title_top_padding(),
                    right: 0.0,
                    bottom: 0.0,
                    left: 0.0,
                })
                .width(Length::Fill),
        );
    }

    tooltip = tooltip.push(
        Container::new(rich_supporting_text::<Renderer>(supporting_text))
            .padding(rich_supporting_text_padding(has_title, has_action))
            .width(Length::Fill),
    );

    if let Some(action) = action {
        tooltip = tooltip.push(
            Container::new(action)
                .padding(Padding {
                    top: 0.0,
                    right: 0.0,
                    bottom: tokens::component::tooltip::RICH_ACTION_LABEL_BOTTOM_PADDING,
                    left: 0.0,
                })
                .width(Length::Fill),
        );
    }

    let tooltip = Container::new(tooltip)
        .width(Length::Fill)
        .height(Length::Shrink)
        .max_width(tokens::component::tooltip::RICH_MAX_WIDTH);

    RichTooltip::new(content, tooltip, position)
        .gap(tokens::component::tooltip::SPACING_BETWEEN_TOOLTIP_AND_ANCHOR)
        .padding(0.0)
        .clip_padding(rich_tooltip_shadow_padding())
        .style(tooltip_style::rich)
}

/// A Material plain tooltip with Android platform tooltip show/hide animation.
pub type PlainTooltip<'a, Message, Renderer = iced_widget::Renderer> =
    RichTooltip<'a, Message, Renderer>;

/// A Material rich tooltip that remains interactive while the pointer moves
/// from its anchor to the tooltip surface.
pub struct RichTooltip<'a, Message, Renderer = iced_widget::Renderer>
where
    Renderer: core_text::Renderer,
{
    content: Element<'a, Message, Theme, Renderer>,
    tooltip: Element<'a, Message, Theme, Renderer>,
    position: Position,
    gap: f32,
    padding: f32,
    clip_padding: f32,
    snap_within_viewport: bool,
    interactive_surface: bool,
    class: <Theme as iced_container::Catalog>::Class<'a>,
}

impl<Message, Renderer> std::fmt::Debug for RichTooltip<'_, Message, Renderer>
where
    Renderer: core_text::Renderer,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RichTooltip")
            .field("position", &self.position)
            .field("gap", &self.gap)
            .field("padding", &self.padding)
            .field("clip_padding", &self.clip_padding)
            .field("snap_within_viewport", &self.snap_within_viewport)
            .field("interactive_surface", &self.interactive_surface)
            .finish_non_exhaustive()
    }
}

impl<'a, Message, Renderer> RichTooltip<'a, Message, Renderer>
where
    Message: 'a,
    Renderer: core_text::Renderer + 'a,
{
    pub fn new(
        content: impl Into<Element<'a, Message, Theme, Renderer>>,
        tooltip: impl Into<Element<'a, Message, Theme, Renderer>>,
        position: Position,
    ) -> Self {
        Self {
            content: content.into(),
            tooltip: tooltip.into(),
            position,
            gap: 0.0,
            padding: 0.0,
            clip_padding: 0.0,
            snap_within_viewport: true,
            interactive_surface: true,
            class: <Theme as iced_container::Catalog>::default(),
        }
    }

    pub fn gap(mut self, gap: impl Into<Pixels>) -> Self {
        self.gap = gap.into().0;
        self
    }

    pub fn padding(mut self, padding: impl Into<Pixels>) -> Self {
        self.padding = padding.into().0;
        self
    }

    fn clip_padding(mut self, padding: impl Into<Pixels>) -> Self {
        self.clip_padding = padding.into().0;
        self
    }

    pub fn snap_within_viewport(mut self, snap: bool) -> Self {
        self.snap_within_viewport = snap;
        self
    }

    fn interactive_surface(mut self, interactive: bool) -> Self {
        self.interactive_surface = interactive;
        self
    }

    #[must_use]
    pub fn style(mut self, style: impl Fn(&Theme) -> iced_container::Style + 'a) -> Self
    where
        <Theme as iced_container::Catalog>::Class<'a>: From<iced_container::StyleFn<'a, Theme>>,
    {
        self.class = (Box::new(style) as iced_container::StyleFn<'a, Theme>).into();
        self
    }
}

impl<Message, Renderer> Widget<Message, Theme, Renderer> for RichTooltip<'_, Message, Renderer>
where
    Renderer: core_text::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<RichTooltipState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(RichTooltipState::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content), Tree::new(&self.tooltip)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[self.content.as_widget(), self.tooltip.as_widget()]);
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.content.as_widget().size_hint()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        if let Event::Mouse(_) | Event::Window(window::Event::RedrawRequested(_)) = event {
            let state = tree.state.downcast_mut::<RichTooltipState>();
            let now = tooltip_event_time(event);
            let was_visible = state.is_visible();
            let cursor_position = cursor.position_over(layout.bounds());

            state.advance(now);

            if let Some(cursor_position) = cursor_position {
                state.show(now, cursor_position);
                shell.request_redraw();
            } else if rich_tooltip_anchor_exit_dismisses(self.interactive_surface, state) {
                state.dismiss(now);
            }

            if was_visible != state.is_visible() {
                shell.invalidate_layout();
            }

            request_tooltip_redraw(state, shell);
        }

        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        defaults: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            defaults,
            layout,
            cursor,
            viewport,
        );
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        let state = tree.state.downcast_mut::<RichTooltipState>();
        let mut children = tree.children.iter_mut();

        let content = self.content.as_widget_mut().overlay(
            children.next().unwrap(),
            layout,
            renderer,
            viewport,
            translation,
        );

        let content_bounds = translated_bounds(layout.bounds(), translation);
        let tooltip = if state.is_visible() {
            Some(overlay::Element::new(Box::new(RichTooltipOverlay {
                tooltip: &mut self.tooltip,
                tree: children.next().unwrap(),
                state,
                content_bounds,
                snap_within_viewport: self.snap_within_viewport,
                position: self.position,
                gap: self.gap,
                padding: self.padding,
                clip_padding: self.clip_padding,
                interactive_surface: self.interactive_surface,
                class: &self.class,
            })))
        } else {
            let _ = children.next();
            None
        };

        if content.is_some() || tooltip.is_some() {
            Some(
                overlay::Group::with_children(content.into_iter().chain(tooltip).collect())
                    .overlay(),
            )
        } else {
            None
        }
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn core_widget::Operation,
    ) {
        operation.container(None, layout.bounds());
        operation.traverse(&mut |operation| {
            self.content.as_widget_mut().operate(
                &mut tree.children[0],
                layout,
                renderer,
                operation,
            );
        });
    }
}

impl<'a, Message, Renderer> From<RichTooltip<'a, Message, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: core_text::Renderer + 'a,
{
    fn from(tooltip: RichTooltip<'a, Message, Renderer>) -> Self {
        Element::new(tooltip)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TooltipPhase {
    Hidden,
    Showing,
    Shown,
    Dismissing,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct RichTooltipState {
    phase: TooltipPhase,
    cursor_position: Point,
    started_at: Option<Instant>,
    reveal_from: f32,
}

impl Default for RichTooltipState {
    fn default() -> Self {
        Self {
            phase: TooltipPhase::Hidden,
            cursor_position: Point::ORIGIN,
            started_at: None,
            reveal_from: 0.0,
        }
    }
}

impl RichTooltipState {
    fn show(&mut self, now: Instant, cursor_position: Point) {
        self.cursor_position = cursor_position;

        match self.phase {
            TooltipPhase::Showing | TooltipPhase::Shown => {}
            TooltipPhase::Hidden | TooltipPhase::Dismissing => {
                self.reveal_from = self.reveal(now);
                self.phase = TooltipPhase::Showing;
                self.started_at = Some(now);
            }
        }
    }

    fn dismiss(&mut self, now: Instant) {
        match self.phase {
            TooltipPhase::Hidden | TooltipPhase::Dismissing => {}
            TooltipPhase::Showing | TooltipPhase::Shown => {
                self.reveal_from = self.reveal(now);
                self.phase = TooltipPhase::Dismissing;
                self.started_at = Some(now);
            }
        }
    }

    fn advance(&mut self, now: Instant) {
        match self.phase {
            TooltipPhase::Hidden | TooltipPhase::Shown => {}
            TooltipPhase::Showing if self.progress(now) >= 1.0 => {
                self.phase = TooltipPhase::Shown;
                self.started_at = None;
                self.reveal_from = 1.0;
            }
            TooltipPhase::Showing => {}
            TooltipPhase::Dismissing if self.progress(now) >= 1.0 => {
                *self = Self::default();
            }
            TooltipPhase::Dismissing => {}
        }
    }

    fn is_visible(&self) -> bool {
        self.phase != TooltipPhase::Hidden
    }

    fn is_animating(&self) -> bool {
        matches!(self.phase, TooltipPhase::Showing | TooltipPhase::Dismissing)
    }

    fn reveal(&self, now: Instant) -> f32 {
        match self.phase {
            TooltipPhase::Hidden => 0.0,
            TooltipPhase::Shown => 1.0,
            TooltipPhase::Showing => {
                let eased = decelerate_quad(self.progress(now));
                lerp(self.reveal_from, 1.0, eased)
            }
            TooltipPhase::Dismissing => {
                let eased = accelerate_quad(self.progress(now));
                lerp(self.reveal_from, 0.0, eased)
            }
        }
    }

    fn progress(&self, now: Instant) -> f32 {
        let Some(started_at) = self.started_at else {
            return match self.phase {
                TooltipPhase::Showing | TooltipPhase::Dismissing => 1.0,
                TooltipPhase::Hidden | TooltipPhase::Shown => 0.0,
            };
        };

        let duration = duration_ms(tokens::component::tooltip::ANIMATION_DURATION_MS);

        if duration.is_zero() {
            return 1.0;
        }

        (now.saturating_duration_since(started_at).as_secs_f32() / duration.as_secs_f32())
            .clamp(0.0, 1.0)
    }
}

struct RichTooltipOverlay<'a, 'b, Message, Renderer>
where
    Renderer: core_text::Renderer,
{
    tooltip: &'b mut Element<'a, Message, Theme, Renderer>,
    tree: &'b mut Tree,
    state: &'b mut RichTooltipState,
    content_bounds: Rectangle,
    snap_within_viewport: bool,
    position: Position,
    gap: f32,
    padding: f32,
    clip_padding: f32,
    interactive_surface: bool,
    class: &'b <Theme as iced_container::Catalog>::Class<'a>,
}

impl<Message, Renderer> overlay::Overlay<Message, Theme, Renderer>
    for RichTooltipOverlay<'_, '_, Message, Renderer>
where
    Renderer: core_text::Renderer,
{
    fn layout(&mut self, renderer: &Renderer, bounds: Size) -> layout::Node {
        let viewport = Rectangle::with_size(bounds);
        let total_padding = self.padding + self.clip_padding;
        let tooltip_layout = self.tooltip.as_widget_mut().layout(
            self.tree,
            renderer,
            &layout::Limits::new(
                Size::ZERO,
                if self.snap_within_viewport {
                    viewport.size()
                } else {
                    Size::INFINITE
                },
            )
            .shrink(Padding::new(total_padding)),
        );

        let tooltip_bounds = rich_tooltip_surface_bounds(
            self.content_bounds,
            tooltip_layout.bounds().size(),
            viewport,
            self.state.cursor_position,
            self.position,
            self.gap,
            self.padding,
            self.clip_padding,
            self.snap_within_viewport,
        );

        layout::Node::with_children(
            tooltip_bounds.size(),
            vec![tooltip_layout.translate(Vector::new(total_padding, total_padding))],
        )
        .translate(Vector::new(tooltip_bounds.x, tooltip_bounds.y))
    }

    fn update(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) {
        let tooltip_bounds = layout.bounds();
        let now = tooltip_event_time(event);
        let cursor_in_content = cursor.is_over(self.content_bounds);
        let cursor_in_tooltip = cursor.is_over(tooltip_bounds);
        let cursor_in_keep_alive = self.interactive_surface
            && rich_tooltip_keep_alive_contains(
                self.content_bounds,
                tooltip_bounds,
                self.position,
                cursor,
            );

        if cursor_in_tooltip && let Some(child_layout) = layout.children().next() {
            self.tooltip.as_widget_mut().update(
                self.tree,
                event,
                child_layout,
                cursor,
                renderer,
                clipboard,
                shell,
                &Rectangle::with_size(Size::INFINITE),
            );
        }

        if let Event::Mouse(_) | Event::Window(window::Event::RedrawRequested(_)) = event {
            let was_visible = self.state.is_visible();

            if cursor_in_content || cursor_in_keep_alive {
                if let Some(cursor_position) = cursor.position_over(self.content_bounds) {
                    self.state.show(now, cursor_position);
                } else if self.state.phase == TooltipPhase::Dismissing {
                    self.state.show(now, self.state.cursor_position);
                }
            } else {
                self.state.dismiss(now);
            }

            self.state.advance(now);

            if was_visible != self.state.is_visible() {
                shell.invalidate_layout();
            }

            request_tooltip_redraw(self.state, shell);
        }

        if self.interactive_surface
            && (cursor_in_tooltip || cursor_in_keep_alive)
            && !cursor_in_content
            && matches!(event, Event::Mouse(_))
        {
            shell.capture_event();
        }
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let tooltip_bounds = layout.bounds();

        if cursor.is_over(tooltip_bounds)
            && let Some(child_layout) = layout.children().next()
        {
            return self.tooltip.as_widget().mouse_interaction(
                self.tree,
                child_layout,
                cursor,
                &Rectangle::with_size(Size::INFINITE),
                renderer,
            );
        }

        mouse::Interaction::default()
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        inherited_style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        let reveal = self.state.reveal(Instant::now());
        let style = tooltip_container_style_alpha(
            iced_container::Catalog::style(theme, self.class),
            reveal,
        );
        let surface_bounds = rich_tooltip_visual_bounds(layout.bounds(), self.clip_padding);
        let transformation = tooltip_reveal_transformation(surface_bounds, reveal);

        renderer.with_layer(layout.bounds(), |renderer| {
            renderer.with_transformation(transformation, |renderer| {
                iced_container::draw_background(renderer, &style, surface_bounds);

                let defaults = renderer::Style {
                    text_color: style.text_color.unwrap_or(inherited_style.text_color),
                };

                self.tooltip.as_widget().draw(
                    self.tree,
                    renderer,
                    theme,
                    &defaults,
                    layout.children().next().unwrap(),
                    cursor,
                    &Rectangle::with_size(Size::INFINITE),
                );
            });
        });
    }

    fn overlay<'c>(
        &'c mut self,
        layout: Layout<'c>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'c, Message, Theme, Renderer>> {
        self.tooltip.as_widget_mut().overlay(
            self.tree,
            layout.children().next().unwrap(),
            renderer,
            &Rectangle::with_size(Size::INFINITE),
            Vector::ZERO,
        )
    }
}

fn translated_bounds(bounds: Rectangle, translation: Vector) -> Rectangle {
    Rectangle {
        x: bounds.x + translation.x,
        y: bounds.y + translation.y,
        ..bounds
    }
}

fn rich_tooltip_surface_bounds(
    content_bounds: Rectangle,
    tooltip_size: Size,
    viewport: Rectangle,
    cursor_position: Point,
    position: Position,
    gap: f32,
    padding: f32,
    clip_padding: f32,
    snap_within_viewport: bool,
) -> Rectangle {
    let surface_size = Size::new(
        tooltip_size.width + padding * 2.0,
        tooltip_size.height + padding * 2.0,
    );
    let x_center = content_bounds.x + (content_bounds.width - surface_size.width) / 2.0;
    let y_center = content_bounds.y + (content_bounds.height - surface_size.height) / 2.0;

    let offset = match position {
        Position::Top => Vector::new(x_center, content_bounds.y - surface_size.height - gap),
        Position::Bottom => Vector::new(x_center, content_bounds.y + content_bounds.height + gap),
        Position::Left => Vector::new(content_bounds.x - surface_size.width - gap, y_center),
        Position::Right => Vector::new(content_bounds.x + content_bounds.width + gap, y_center),
        Position::FollowCursor => {
            Vector::new(cursor_position.x, cursor_position.y - surface_size.height)
        }
    };

    let mut tooltip_bounds = Rectangle {
        x: offset.x - clip_padding,
        y: offset.y - clip_padding,
        width: surface_size.width + clip_padding * 2.0,
        height: surface_size.height + clip_padding * 2.0,
    };

    if snap_within_viewport {
        if tooltip_bounds.x < viewport.x {
            tooltip_bounds.x = viewport.x;
        } else if viewport.x + viewport.width < tooltip_bounds.x + tooltip_bounds.width {
            tooltip_bounds.x = viewport.x + viewport.width - tooltip_bounds.width;
        }

        if tooltip_bounds.y < viewport.y {
            tooltip_bounds.y = viewport.y;
        } else if viewport.y + viewport.height < tooltip_bounds.y + tooltip_bounds.height {
            tooltip_bounds.y = viewport.y + viewport.height - tooltip_bounds.height;
        }
    }

    tooltip_bounds
}

fn rich_tooltip_visual_bounds(tooltip_bounds: Rectangle, clip_padding: f32) -> Rectangle {
    Rectangle {
        x: tooltip_bounds.x + clip_padding,
        y: tooltip_bounds.y + clip_padding,
        width: (tooltip_bounds.width - clip_padding * 2.0).max(0.0),
        height: (tooltip_bounds.height - clip_padding * 2.0).max(0.0),
    }
}

fn rich_tooltip_shadow_padding() -> f32 {
    let shadow =
        tokens::elevation::shadow(tokens::component::tooltip::RICH_CONTAINER_ELEVATION_LEVEL)
            .ambient;

    (shadow.blur + shadow.y.abs()).ceil()
}

fn rich_tooltip_keep_alive_contains(
    content_bounds: Rectangle,
    tooltip_bounds: Rectangle,
    position: Position,
    cursor: mouse::Cursor,
) -> bool {
    let Some(cursor) = cursor.position() else {
        return false;
    };

    if content_bounds.contains(cursor) || tooltip_bounds.contains(cursor) {
        return true;
    }

    rich_tooltip_corridor_bounds(content_bounds, tooltip_bounds, position)
        .is_some_and(|bounds| bounds.contains(cursor))
}

fn rich_tooltip_anchor_exit_dismisses(interactive_surface: bool, state: &RichTooltipState) -> bool {
    !interactive_surface || !state.is_visible()
}

fn rich_tooltip_corridor_bounds(
    content_bounds: Rectangle,
    tooltip_bounds: Rectangle,
    position: Position,
) -> Option<Rectangle> {
    let content_right = content_bounds.x + content_bounds.width;
    let content_bottom = content_bounds.y + content_bounds.height;
    let tooltip_right = tooltip_bounds.x + tooltip_bounds.width;
    let tooltip_bottom = tooltip_bounds.y + tooltip_bounds.height;

    match position {
        Position::Top if tooltip_bottom <= content_bounds.y => Some(Rectangle {
            x: content_bounds.x,
            y: tooltip_bottom,
            width: content_bounds.width,
            height: content_bounds.y - tooltip_bottom,
        }),
        Position::Bottom if content_bottom <= tooltip_bounds.y => Some(Rectangle {
            x: content_bounds.x,
            y: content_bottom,
            width: content_bounds.width,
            height: tooltip_bounds.y - content_bottom,
        }),
        Position::Left if tooltip_right <= content_bounds.x => Some(Rectangle {
            x: tooltip_right,
            y: content_bounds.y,
            width: content_bounds.x - tooltip_right,
            height: content_bounds.height,
        }),
        Position::Right if content_right <= tooltip_bounds.x => Some(Rectangle {
            x: content_right,
            y: content_bounds.y,
            width: tooltip_bounds.x - content_right,
            height: content_bounds.height,
        }),
        _ => None,
    }
}

fn tooltip_container_style_alpha(
    mut style: iced_container::Style,
    alpha: f32,
) -> iced_container::Style {
    style.background = style
        .background
        .map(|background| background.scale_alpha(alpha));
    style.text_color = style.text_color.map(|color| alpha_color(color, alpha));
    style.border.color = alpha_color(style.border.color, alpha);
    style.shadow.color = alpha_color(style.shadow.color, alpha);

    style
}

fn tooltip_reveal_transformation(bounds: Rectangle, reveal: f32) -> Transformation {
    Transformation::translate(bounds.center_x(), bounds.center_y())
        * Transformation::scale(reveal)
        * Transformation::translate(-bounds.center_x(), -bounds.center_y())
}

fn tooltip_event_time(event: &Event) -> Instant {
    match event {
        Event::Window(window::Event::RedrawRequested(now)) => *now,
        _ => Instant::now(),
    }
}

fn request_tooltip_redraw<Message>(state: &RichTooltipState, shell: &mut Shell<'_, Message>) {
    if state.is_animating() {
        shell.request_redraw();
    }
}

fn decelerate_quad(progress: f32) -> f32 {
    let progress = progress.clamp(0.0, 1.0);

    1.0 - (1.0 - progress) * (1.0 - progress)
}

fn accelerate_quad(progress: f32) -> f32 {
    let progress = progress.clamp(0.0, 1.0);

    progress * progress
}

fn rich_title_text<'a, Renderer>(title: impl text::IntoFragment<'a>) -> Text<'a, Theme, Renderer>
where
    Renderer: core_text::Renderer,
{
    let scale = tokens::component::tooltip::RICH_SUBHEAD_TEXT;

    text_with_metrics(title, scale.size, scale.line_height)
        .wrapping(text::Wrapping::Word)
        .style(rich_title_style)
}

fn rich_supporting_text<'a, Renderer>(
    supporting_text: impl text::IntoFragment<'a>,
) -> Text<'a, Theme, Renderer>
where
    Renderer: core_text::Renderer,
{
    let scale = tokens::component::tooltip::RICH_SUPPORTING_TEXT;

    text_with_metrics(supporting_text, scale.size, scale.line_height)
        .wrapping(text::Wrapping::Word)
        .style(rich_supporting_text_style)
}

fn rich_title_top_padding() -> f32 {
    (tokens::component::tooltip::RICH_HEIGHT_TO_SUBHEAD_FIRST_LINE
        - tokens::component::tooltip::RICH_SUBHEAD_TEXT.line_height)
        .max(0.0)
}

fn rich_supporting_text_padding(has_title: bool, has_action: bool) -> Padding {
    if !has_title && !has_action {
        return Padding {
            top: tokens::component::tooltip::RICH_TEXT_VERTICAL_SPACE_WITHOUT_TITLE_OR_ACTION,
            right: 0.0,
            bottom: tokens::component::tooltip::RICH_TEXT_VERTICAL_SPACE_WITHOUT_TITLE_OR_ACTION,
            left: 0.0,
        };
    }

    Padding {
        top: (tokens::component::tooltip::RICH_HEIGHT_FROM_SUBHEAD_TO_TEXT_FIRST_LINE
            - tokens::component::tooltip::RICH_SUPPORTING_TEXT.line_height)
            .max(0.0),
        right: 0.0,
        bottom: tokens::component::tooltip::RICH_TEXT_BOTTOM_PADDING,
        left: 0.0,
    }
}

fn rich_title_style(theme: &Theme) -> iced_widget::text::Style {
    iced_widget::text::Style {
        color: Some(theme.colors().surface.text_variant),
    }
}

fn rich_supporting_text_style(theme: &Theme) -> iced_widget::text::Style {
    iced_widget::text::Style {
        color: Some(theme.colors().surface.text_variant),
    }
}

fn plain_tooltip_inner_horizontal_padding() -> f32 {
    (tokens::component::tooltip::PLAIN_HORIZONTAL_SPACE
        - tokens::component::tooltip::PLAIN_VERTICAL_SPACE)
        .max(0.0)
}

fn plain_tooltip_inner_max_width() -> f32 {
    tokens::component::tooltip::PLAIN_MAX_WIDTH
        - tokens::component::tooltip::PLAIN_VERTICAL_SPACE * 2.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_tooltip_text_shrinks_under_material_max_width() {
        let type_scale = tokens::component::tooltip::PLAIN_SUPPORTING_TEXT;
        let text: Text<'_, Theme, iced_widget::Renderer> =
            plain_supporting_text("Material 3 plain tooltip", type_scale);

        assert_eq!(
            Widget::<(), Theme, iced_widget::Renderer>::size(&text).width,
            Length::Shrink
        );
        assert_eq!(plain_tooltip_inner_horizontal_padding(), 4.0);
        assert_eq!(plain_tooltip_inner_max_width(), 192.0);
        assert_eq!(
            plain_tooltip_inner_max_width()
                + tokens::component::tooltip::PLAIN_VERTICAL_SPACE * 2.0,
            tokens::component::tooltip::PLAIN_MAX_WIDTH
        );
    }

    #[test]
    fn rich_tooltip_padding_matches_androidx_material_layout_constants() {
        assert_eq!(rich_title_top_padding(), 8.0);
        assert_eq!(rich_tooltip_shadow_padding(), 8.0);
        assert_eq!(
            rich_supporting_text_padding(false, false),
            Padding {
                top: 4.0,
                right: 0.0,
                bottom: 4.0,
                left: 0.0,
            }
        );
        assert_eq!(
            rich_supporting_text_padding(true, true),
            Padding {
                top: 4.0,
                right: 0.0,
                bottom: 16.0,
                left: 0.0,
            }
        );
    }

    #[test]
    fn rich_tooltip_surface_keeps_material_gap_from_anchor() {
        let content = Rectangle {
            x: 120.0,
            y: 160.0,
            width: 80.0,
            height: 32.0,
        };
        let clip_padding = rich_tooltip_shadow_padding();
        let tooltip = rich_tooltip_surface_bounds(
            content,
            Size::new(180.0, 96.0),
            Rectangle::new(Point::ORIGIN, Size::new(400.0, 400.0)),
            Point::ORIGIN,
            Position::Top,
            tokens::component::tooltip::SPACING_BETWEEN_TOOLTIP_AND_ANCHOR,
            0.0,
            clip_padding,
            true,
        );
        let surface = rich_tooltip_visual_bounds(tooltip, clip_padding);

        assert_eq!(
            content.y - (surface.y + surface.height),
            tokens::component::tooltip::SPACING_BETWEEN_TOOLTIP_AND_ANCHOR
        );
        assert_eq!(tooltip.width, 180.0 + clip_padding * 2.0);
        assert_eq!(surface.width, 180.0);
    }

    #[test]
    fn rich_tooltip_corridor_spans_anchor_surface_gap() {
        let content = Rectangle {
            x: 120.0,
            y: 160.0,
            width: 80.0,
            height: 32.0,
        };
        let tooltip = Rectangle {
            x: 70.0,
            y: 60.0,
            width: 180.0,
            height: 96.0,
        };
        let corridor = rich_tooltip_corridor_bounds(content, tooltip, Position::Top)
            .expect("top tooltip should have a corridor to its anchor");

        assert!(corridor.contains(Point::new(160.0, 158.0)));
        assert!(!corridor.contains(Point::new(160.0, 120.0)));
        assert!(!corridor.contains(Point::new(80.0, 158.0)));
    }

    #[test]
    fn rich_tooltip_keep_alive_does_not_cover_adjacent_anchor() {
        let rich_anchor = Rectangle {
            x: 289.0,
            y: 290.0,
            width: 118.0,
            height: 62.0,
        };
        let rich_tooltip = Rectangle {
            x: 28.0,
            y: 9.0,
            width: 640.0,
            height: 272.0,
        };

        assert!(rich_tooltip_keep_alive_contains(
            rich_anchor,
            rich_tooltip,
            Position::Top,
            mouse::Cursor::Available(Point::new(348.0, 286.0)),
        ));
        assert!(!rich_tooltip_keep_alive_contains(
            rich_anchor,
            rich_tooltip,
            Position::Top,
            mouse::Cursor::Available(Point::new(200.0, 322.0)),
        ));
    }

    #[test]
    fn rich_tooltip_anchor_exit_defers_dismissal_to_overlay() {
        let start = Instant::now();
        let mut state = RichTooltipState::default();

        state.show(start, Point::new(10.0, 20.0));

        assert!(!rich_tooltip_anchor_exit_dismisses(true, &state));
        assert!(rich_tooltip_anchor_exit_dismisses(false, &state));
    }

    #[test]
    fn rich_tooltip_transition_uses_android_tooltip_easing() {
        let start = Instant::now();
        let mut state = RichTooltipState::default();

        state.show(start, Point::new(10.0, 20.0));
        assert_eq!(state.phase, TooltipPhase::Showing);
        assert_eq!(state.reveal(start), 0.0);

        let halfway = start + duration_ms(tokens::component::tooltip::ANIMATION_DURATION_MS / 2);
        assert!((state.reveal(halfway) - 0.75).abs() < 0.001);

        let shown = start + duration_ms(tokens::component::tooltip::ANIMATION_DURATION_MS);
        state.advance(shown);
        assert_eq!(state.phase, TooltipPhase::Shown);
        assert_eq!(state.reveal(shown), 1.0);

        state.dismiss(shown);
        assert_eq!(state.phase, TooltipPhase::Dismissing);
        assert_eq!(state.reveal(shown), 1.0);
        assert!(
            (state
                .reveal(halfway + duration_ms(tokens::component::tooltip::ANIMATION_DURATION_MS))
                - 0.75)
                .abs()
                < 0.001
        );

        let hidden = shown + duration_ms(tokens::component::tooltip::ANIMATION_DURATION_MS);
        state.advance(hidden);
        assert_eq!(state.phase, TooltipPhase::Hidden);
        assert_eq!(state.reveal(hidden), 0.0);
    }

    #[test]
    fn tooltip_alpha_style_scales_container_colors() {
        let theme = Theme::Light;
        let style = tooltip_container_style_alpha(tooltip_style::rich(&theme), 0.5);

        assert_eq!(
            style.background,
            Some(Background::Color(alpha_color(
                theme.colors().surface.container.base,
                0.5
            )))
        );
        assert_eq!(
            style.text_color,
            Some(alpha_color(theme.colors().surface.text_variant, 0.5))
        );
    }
}
