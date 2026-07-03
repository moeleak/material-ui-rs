//! Material 3 button constructors with token-backed layout defaults.

use super::*;
use iced_widget::button::{Catalog, Status, Style, StyleFn};
use iced_widget::canvas;
use iced_widget::core::overlay;
use iced_widget::graphics::geometry;

use crate::utils::{PRESSED_LAYER_OPACITY, state_layer};

const RIPPLE_ENTER_DURATION_MS: u16 = 225;
const RIPPLE_ORIGIN_DURATION_MS: u16 = 225;
const RIPPLE_OPACITY_ENTER_DURATION_MS: u16 = 75;
const RIPPLE_OPACITY_EXIT_DURATION_MS: u16 = 150;
const RIPPLE_OPACITY_HOLD_DURATION_MS: u16 = RIPPLE_OPACITY_ENTER_DURATION_MS + 150;
const RIPPLE_START_RADIUS_FACTOR: f32 = 0.3;
const RIPPLE_CLIP_MIN_SAMPLES: usize = 24;
const RIPPLE_CLIP_MAX_SAMPLES: usize = 96;
const MAX_RIPPLES: usize = 10;

/// A Material 3 button with Android-style bounded press ripples.
pub struct Button<'a, Message, Renderer = iced_widget::Renderer>
where
    Renderer: geometry::Renderer,
{
    content: Element<'a, Message, Theme, Renderer>,
    on_press: Option<OnPress<'a, Message>>,
    width: Length,
    height: Length,
    padding: Padding,
    clip: bool,
    class: <Theme as Catalog>::Class<'a>,
    status: Option<Status>,
}

enum OnPress<'a, Message> {
    Direct(Message),
    Closure(Box<dyn Fn() -> Message + 'a>),
}

impl<Message, Renderer> std::fmt::Debug for Button<'_, Message, Renderer>
where
    Renderer: geometry::Renderer,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Button")
            .field("width", &self.width)
            .field("height", &self.height)
            .field("padding", &self.padding)
            .field("clip", &self.clip)
            .field("status", &self.status)
            .finish_non_exhaustive()
    }
}

impl<Message: Clone> OnPress<'_, Message> {
    fn get(&self) -> Message {
        match self {
            OnPress::Direct(message) => message.clone(),
            OnPress::Closure(f) => f(),
        }
    }
}

impl<'a, Message, Renderer> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + 'a,
{
    /// Creates a new [`Button`] with the given content.
    pub fn new(content: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        let content = content.into();
        let size = content.as_widget().size_hint();

        Self {
            content,
            on_press: None,
            width: size.width.fluid(),
            height: size.height.fluid(),
            padding: iced_widget::button::DEFAULT_PADDING,
            clip: false,
            class: <Theme as Catalog>::default(),
            status: None,
        }
    }

    /// Sets the width of the [`Button`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the [`Button`].
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Sets the [`Padding`] of the [`Button`].
    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets the message produced when the [`Button`] is pressed.
    pub fn on_press(mut self, on_press: Message) -> Self {
        self.on_press = Some(OnPress::Direct(on_press));
        self
    }

    /// Sets the message produced when the [`Button`] is pressed using a closure.
    pub fn on_press_with(mut self, on_press: impl Fn() -> Message + 'a) -> Self {
        self.on_press = Some(OnPress::Closure(Box::new(on_press)));
        self
    }

    /// Sets the message produced when the [`Button`] is pressed, if any.
    pub fn on_press_maybe(mut self, on_press: Option<Message>) -> Self {
        self.on_press = on_press.map(OnPress::Direct);
        self
    }

    /// Sets whether the button content should be clipped on overflow.
    pub fn clip(mut self, clip: bool) -> Self {
        self.clip = clip;
        self
    }

    /// Sets the style of the [`Button`].
    #[must_use]
    pub fn style(mut self, style: impl Fn(&Theme, Status) -> Style + 'a) -> Self {
        self.class = Box::new(style) as StyleFn<'a, Theme>;
        self
    }
}

#[derive(Debug, Clone)]
struct ButtonState {
    is_pressed: bool,
    active_ripple: Option<Ripple>,
    exiting_ripples: Vec<Ripple>,
    last_status: Option<Status>,
    now: Option<Instant>,
}

impl Default for ButtonState {
    fn default() -> Self {
        Self {
            is_pressed: false,
            active_ripple: None,
            exiting_ripples: Vec::new(),
            last_status: None,
            now: None,
        }
    }
}

impl ButtonState {
    fn press(&mut self, origin: Point, now: Instant) {
        if let Some(mut ripple) = self.active_ripple.take() {
            ripple.exit(now);
            self.push_exiting(ripple);
        }

        self.is_pressed = true;
        self.active_ripple = Some(Ripple::new(origin, now));
        self.now = Some(now);
    }

    fn release(&mut self, now: Instant) {
        self.is_pressed = false;

        if let Some(mut ripple) = self.active_ripple.take() {
            ripple.exit(now);
            self.push_exiting(ripple);
        }

        self.now = Some(now);
    }

    fn cancel(&mut self, now: Instant) {
        self.release(now);
    }

    fn push_exiting(&mut self, ripple: Ripple) {
        if self.exiting_ripples.len() >= MAX_RIPPLES {
            let _ = self.exiting_ripples.remove(0);
        }

        self.exiting_ripples.push(ripple);
    }

    fn prune(&mut self, now: Instant) {
        self.exiting_ripples
            .retain(|ripple| !ripple.has_finished_exit(now));
    }

    fn has_visible_ripples(&self, now: Instant) -> bool {
        self.visible_ripple_opacity(now) > 0.0
    }

    fn visible_ripple_opacity(&self, now: Instant) -> f32 {
        self.active_ripple
            .map(|ripple| ripple.opacity(now))
            .into_iter()
            .chain(
                self.exiting_ripples
                    .iter()
                    .map(|ripple| ripple.opacity(now)),
            )
            .fold(0.0, f32::max)
    }
}

#[derive(Debug, Clone, Copy)]
struct Ripple {
    origin: Point,
    started_at: Instant,
    exit_started_at: Option<Instant>,
    exit_delay: iced_widget::core::time::Duration,
}

impl Ripple {
    fn new(origin: Point, started_at: Instant) -> Self {
        Self {
            origin,
            started_at,
            exit_started_at: None,
            exit_delay: iced_widget::core::time::Duration::ZERO,
        }
    }

    fn exit(&mut self, now: Instant) {
        let hold = duration_ms(RIPPLE_OPACITY_HOLD_DURATION_MS);
        let elapsed = now.duration_since(self.started_at);

        self.exit_started_at = Some(now);
        self.exit_delay = hold.saturating_sub(elapsed);
    }

    fn circle(self, size: Size, now: Instant) -> RippleCircle {
        let target_radius = ripple_target_radius(size);
        let start_radius = size.width.max(size.height) * RIPPLE_START_RADIUS_FACTOR;
        let clamped_origin = clamped_ripple_origin(self.origin, size, target_radius, start_radius);
        let radius_progress = timed_progress(
            self.started_at,
            now,
            duration_ms(RIPPLE_ENTER_DURATION_MS),
            tokens::motion::EASING_LEGACY,
        );
        let origin_progress = timed_progress(
            self.started_at,
            now,
            duration_ms(RIPPLE_ORIGIN_DURATION_MS),
            tokens::motion::EASING_LEGACY,
        );
        let center = Point::new(size.width / 2.0, size.height / 2.0);

        RippleCircle {
            center: Point::new(
                lerp(clamped_origin.x, center.x, origin_progress),
                lerp(clamped_origin.y, center.y, origin_progress),
            ),
            radius: lerp(start_radius, target_radius, radius_progress),
            target_radius,
        }
    }

    fn opacity(self, now: Instant) -> f32 {
        let enter = timed_progress(
            self.started_at,
            now,
            duration_ms(RIPPLE_OPACITY_ENTER_DURATION_MS),
            tokens::motion::EASING_LINEAR,
        );

        let exit = self
            .exit_started_at
            .map(|exit_started_at| {
                let elapsed = now.duration_since(exit_started_at);

                if elapsed <= self.exit_delay {
                    1.0
                } else {
                    let fade = elapsed - self.exit_delay;
                    1.0 - (fade.as_secs_f32()
                        / duration_ms(RIPPLE_OPACITY_EXIT_DURATION_MS).as_secs_f32())
                    .clamp(0.0, 1.0)
                }
            })
            .unwrap_or(1.0);

        enter * exit
    }

    fn has_finished_exit(self, now: Instant) -> bool {
        self.exit_started_at.is_some_and(|exit_started_at| {
            now.duration_since(exit_started_at)
                >= self.exit_delay + duration_ms(RIPPLE_OPACITY_EXIT_DURATION_MS)
        })
    }
}

#[derive(Debug, Clone, Copy)]
struct RippleCircle {
    center: Point,
    radius: f32,
    target_radius: f32,
}

impl<Message, Renderer> Widget<Message, Theme, Renderer> for Button<'_, Message, Renderer>
where
    Message: Clone,
    Renderer: geometry::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<ButtonState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(ButtonState::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::padded(limits, self.width, self.height, self.padding, |limits| {
            self.content
                .as_widget_mut()
                .layout(&mut tree.children[0], renderer, limits)
        })
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
                layout.children().next().unwrap(),
                renderer,
                operation,
            );
        });
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
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout.children().next().unwrap(),
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );

        if shell.is_event_captured() {
            return;
        }

        let bounds = layout.bounds();
        let now = match event {
            Event::Window(window::Event::RedrawRequested(now)) => Some(*now),
            _ => None,
        };
        let now_or_current = || now.unwrap_or_else(Instant::now);
        let state = tree.state.downcast_mut::<ButtonState>();

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                if self.on_press.is_some() {
                    if let Some(origin) = press_origin(event, bounds, cursor) {
                        state.press(origin, now_or_current());
                        shell.capture_event();
                        shell.request_redraw();
                    }
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. }) => {
                if state.is_pressed {
                    state.release(now_or_current());

                    if release_is_over(event, bounds, cursor) {
                        if let Some(on_press) = &self.on_press {
                            shell.publish(on_press.get());
                        }
                    }

                    shell.capture_event();
                    shell.request_redraw();
                }
            }
            Event::Touch(touch::Event::FingerLost { .. }) => {
                if state.is_pressed {
                    state.cancel(now_or_current());
                    shell.request_redraw();
                }
            }
            _ => {}
        }

        let current_status =
            button_status(self.on_press.is_some(), state.is_pressed, bounds, cursor);

        if let Some(now) = now {
            state.now = Some(now);
            state.prune(now);

            if state.has_visible_ripples(now) {
                shell.request_redraw();
            }

            self.status = Some(current_status);
            state.last_status = Some(current_status);
        } else if self.status.is_some_and(|status| status != current_status)
            || state.has_visible_ripples(state.now.unwrap_or_else(Instant::now))
        {
            shell.request_redraw();
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();

        if bounds.width < 1.0 || bounds.height < 1.0 {
            return;
        }

        let state = tree.state.downcast_ref::<ButtonState>();
        let status = self.status.or(state.last_status).unwrap_or_else(|| {
            button_status(self.on_press.is_some(), state.is_pressed, bounds, cursor)
        });
        let now = state.now.unwrap_or_else(Instant::now);
        let style = button_draw_style(
            theme,
            &self.class,
            status,
            state.visible_ripple_opacity(now),
        );
        let content_layout = layout.children().next().unwrap();

        if style.background.is_some() || style.border.width > 0.0 || style.shadow.color.a > 0.0 {
            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border: style.border,
                    shadow: style.shadow,
                    snap: style.snap,
                },
                style
                    .background
                    .unwrap_or(Background::Color(Color::TRANSPARENT)),
            );
        }

        draw_ripples(
            renderer,
            bounds,
            state,
            style.text_color,
            style.border.radius,
        );

        let viewport = if self.clip {
            bounds.intersection(viewport).unwrap_or(*viewport)
        } else {
            *viewport
        };

        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            &renderer::Style {
                text_color: style.text_color,
            },
            content_layout,
            cursor,
            &viewport,
        );
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        if cursor.is_over(layout.bounds()) && self.on_press.is_some() {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: iced_widget::core::Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout.children().next().unwrap(),
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message, Renderer> From<Button<'a, Message, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + 'a,
{
    fn from(button: Button<'a, Message, Renderer>) -> Self {
        Element::new(button)
    }
}

fn button_status(
    is_enabled: bool,
    is_pressed: bool,
    bounds: Rectangle,
    cursor: mouse::Cursor,
) -> Status {
    if !is_enabled {
        Status::Disabled
    } else if cursor.is_over(bounds) {
        if is_pressed {
            Status::Pressed
        } else {
            Status::Hovered
        }
    } else {
        Status::Active
    }
}

fn button_draw_style(
    theme: &Theme,
    class: &<Theme as Catalog>::Class<'_>,
    status: Status,
    visible_ripple_opacity: f32,
) -> Style {
    let mut style = theme.style(class, status);
    let active = theme.style(class, Status::Active);

    match status {
        Status::Pressed => {
            style.background = active.background;
        }
        Status::Hovered if visible_ripple_opacity > 0.0 => {
            style.background = blend_background(
                active.background,
                style.background,
                1.0 - visible_ripple_opacity.clamp(0.0, 1.0),
            );
        }
        Status::Active | Status::Hovered | Status::Disabled => {}
    }

    style
}

fn blend_background(
    from: Option<Background>,
    to: Option<Background>,
    progress: f32,
) -> Option<Background> {
    let progress = progress.clamp(0.0, 1.0);

    if progress <= 0.0 {
        return from;
    } else if progress >= 1.0 {
        return to;
    }

    match (from, to) {
        (Some(Background::Color(from)), Some(Background::Color(to))) => {
            Some(Background::Color(mix(from, to, progress)))
        }
        (None, Some(to)) => Some(to.scale_alpha(progress)),
        (Some(from), None) => Some(from.scale_alpha(1.0 - progress)),
        (None, None) => None,
        (_, to) => to,
    }
}

fn press_origin(event: &Event, bounds: Rectangle, cursor: mouse::Cursor) -> Option<Point> {
    if cursor.position().is_some() {
        return cursor.position_in(bounds);
    }

    if cursor.is_levitating() {
        return None;
    }

    match event {
        Event::Touch(touch::Event::FingerPressed { position, .. }) => {
            relative_position(*position, bounds)
        }
        _ => cursor.position_in(bounds),
    }
}

fn release_is_over(event: &Event, bounds: Rectangle, cursor: mouse::Cursor) -> bool {
    if cursor.position().is_some() {
        return cursor.is_over(bounds);
    }

    if cursor.is_levitating() {
        return false;
    }

    match event {
        Event::Touch(touch::Event::FingerLifted { position, .. }) => bounds.contains(*position),
        _ => cursor.is_over(bounds),
    }
}

fn relative_position(position: Point, bounds: Rectangle) -> Option<Point> {
    bounds
        .contains(position)
        .then(|| position - iced_widget::core::Vector::new(bounds.x, bounds.y))
}

fn draw_ripples<Renderer>(
    renderer: &mut Renderer,
    bounds: Rectangle,
    state: &ButtonState,
    color: Color,
    clip_radius: border::Radius,
) where
    Renderer: geometry::Renderer,
{
    let now = state.now.unwrap_or_else(Instant::now);

    if !state.has_visible_ripples(now) {
        return;
    }

    let mut frame = canvas::Frame::new(renderer, bounds.size());
    let ripple_color = state_layer(color, PRESSED_LAYER_OPACITY);

    if let Some(ripple) = state.active_ripple {
        fill_ripple(
            &mut frame,
            ripple,
            bounds.size(),
            ripple_color,
            clip_radius,
            now,
        );
    }

    for ripple in &state.exiting_ripples {
        fill_ripple(
            &mut frame,
            *ripple,
            bounds.size(),
            ripple_color,
            clip_radius,
            now,
        );
    }

    let geometry = frame.into_geometry();

    renderer.with_layer(bounds, |renderer| {
        renderer.with_translation(
            iced_widget::core::Vector::new(bounds.x, bounds.y),
            |renderer| renderer.draw_geometry(geometry),
        );
    });
}

fn fill_ripple<Renderer>(
    frame: &mut canvas::Frame<Renderer>,
    ripple: Ripple,
    size: Size,
    mut color: Color,
    clip_radius: border::Radius,
    now: Instant,
) where
    Renderer: geometry::Renderer,
{
    let opacity = ripple.opacity(now);

    if opacity <= 0.0 {
        return;
    }

    let circle = ripple.circle(size, now);

    if circle.radius <= 0.0 {
        return;
    }

    color.a *= opacity;

    let path = bounded_ripple_path(size, clip_radius, circle);

    frame.fill(&path, color);
}

fn bounded_ripple_path(
    size: Size,
    clip_radius: border::Radius,
    circle: RippleCircle,
) -> canvas::Path {
    if circle.radius >= circle.target_radius - 0.5 {
        return canvas::Path::rounded_rectangle(Point::ORIGIN, size, clip_radius);
    }

    clipped_circle_path(size, clip_radius, circle)
        .unwrap_or_else(|| canvas::Path::circle(circle.center, circle.radius))
}

fn clipped_circle_path(
    size: Size,
    clip_radius: border::Radius,
    circle: RippleCircle,
) -> Option<canvas::Path> {
    let top = (circle.center.y - circle.radius).max(0.0);
    let bottom = (circle.center.y + circle.radius).min(size.height);

    if bottom <= top {
        return None;
    }

    let sample_count = ripple_clip_sample_count(circle.radius);
    let step = (bottom - top) / (sample_count.saturating_sub(1) as f32);
    let mut left_edge = Vec::with_capacity(sample_count);
    let mut right_edge = Vec::with_capacity(sample_count);

    for index in 0..sample_count {
        let y = if index + 1 == sample_count {
            bottom
        } else {
            top + step * index as f32
        };

        let Some((circle_left, circle_right)) = circle_span_at_y(circle, y) else {
            continue;
        };
        let Some((clip_left, clip_right)) = rounded_rect_span_at_y(size, clip_radius, y) else {
            continue;
        };

        let left = circle_left.max(clip_left);
        let right = circle_right.min(clip_right);

        if left <= right {
            left_edge.push(Point::new(left, y));
            right_edge.push(Point::new(right, y));
        }
    }

    if left_edge.len() < 2 || right_edge.len() < 2 {
        return None;
    }

    Some(canvas::Path::new(|path| {
        path.move_to(left_edge[0]);

        for point in left_edge.iter().skip(1) {
            path.line_to(*point);
        }

        for point in right_edge.iter().rev() {
            path.line_to(*point);
        }

        path.close();
    }))
}

fn ripple_clip_sample_count(radius: f32) -> usize {
    ((radius * std::f32::consts::TAU).ceil() as usize)
        .clamp(RIPPLE_CLIP_MIN_SAMPLES, RIPPLE_CLIP_MAX_SAMPLES)
}

fn circle_span_at_y(circle: RippleCircle, y: f32) -> Option<(f32, f32)> {
    let dy = y - circle.center.y;
    let distance_to_edge_squared = circle.radius * circle.radius - dy * dy;

    if distance_to_edge_squared < 0.0 {
        return None;
    }

    let dx = distance_to_edge_squared.sqrt();

    Some((circle.center.x - dx, circle.center.x + dx))
}

fn rounded_rect_span_at_y(size: Size, radius: border::Radius, y: f32) -> Option<(f32, f32)> {
    if y < 0.0 || y > size.height {
        return None;
    }

    let [top_left, top_right, bottom_right, bottom_left] = normalized_corner_radii(size, radius);
    let mut left: f32 = 0.0;
    let mut right = size.width;

    if top_left > 0.0 && y < top_left {
        left = left.max(corner_left_bound(top_left, y, top_left));
    }

    if bottom_left > 0.0 && y > size.height - bottom_left {
        left = left.max(corner_left_bound(bottom_left, y, size.height - bottom_left));
    }

    if top_right > 0.0 && y < top_right {
        right = right.min(corner_right_bound(size.width, top_right, y, top_right));
    }

    if bottom_right > 0.0 && y > size.height - bottom_right {
        right = right.min(corner_right_bound(
            size.width,
            bottom_right,
            y,
            size.height - bottom_right,
        ));
    }

    (left <= right).then_some((left, right))
}

fn normalized_corner_radii(size: Size, radius: border::Radius) -> [f32; 4] {
    let max_radius = size.width.min(size.height) / 2.0;
    let [top_left, top_right, bottom_right, bottom_left] = radius.into();

    [
        top_left.min(max_radius),
        top_right.min(max_radius),
        bottom_right.min(max_radius),
        bottom_left.min(max_radius),
    ]
}

fn corner_left_bound(radius: f32, y: f32, center_y: f32) -> f32 {
    radius - circle_axis_delta(radius, y - center_y)
}

fn corner_right_bound(width: f32, radius: f32, y: f32, center_y: f32) -> f32 {
    width - radius + circle_axis_delta(radius, y - center_y)
}

fn circle_axis_delta(radius: f32, offset: f32) -> f32 {
    (radius * radius - offset * offset).max(0.0).sqrt()
}

fn timed_progress(
    started_at: Instant,
    now: Instant,
    duration: iced_widget::core::time::Duration,
    easing: tokens::motion::CubicBezier,
) -> f32 {
    if duration.is_zero() {
        return 1.0;
    }

    let progress =
        (now.duration_since(started_at).as_secs_f32() / duration.as_secs_f32()).clamp(0.0, 1.0);

    easing.transform(progress)
}

fn ripple_target_radius(size: Size) -> f32 {
    let half_width = size.width / 2.0;
    let half_height = size.height / 2.0;

    (half_width * half_width + half_height * half_height).sqrt()
}

fn clamped_ripple_origin(
    origin: Point,
    size: Size,
    target_radius: f32,
    start_radius: f32,
) -> Point {
    let center = Point::new(size.width / 2.0, size.height / 2.0);
    let dx = origin.x - center.x;
    let dy = origin.y - center.y;
    let radius = (target_radius - start_radius).max(0.0);
    let distance_squared = dx * dx + dy * dy;

    if radius > 0.0 && distance_squared > radius * radius {
        let angle = dy.atan2(dx);

        Point::new(
            center.x + angle.cos() * radius,
            center.y + angle.sin() * radius,
        )
    } else {
        origin
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_close(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() < 0.001,
            "expected {expected}, got {actual}",
        );
    }

    #[test]
    fn ripple_radius_matches_android_auto_radius_for_non_round_bounds() {
        let radius = ripple_target_radius(Size::new(100.0, 40.0));

        assert_close(radius, (50.0_f32 * 50.0 + 20.0 * 20.0).sqrt());
    }

    #[test]
    fn ripple_radius_uses_android_auto_radius_for_round_bounds() {
        let radius = ripple_target_radius(Size::new(40.0, 40.0));

        assert_close(radius, (20.0_f32 * 20.0 + 20.0 * 20.0).sqrt());
    }

    #[test]
    fn partial_round_bounds_use_android_auto_radius() {
        let radius = ripple_target_radius(Size::new(80.0, 40.0));

        assert_close(radius, (40.0_f32 * 40.0 + 20.0 * 20.0).sqrt());
    }

    #[test]
    fn ripple_starts_at_android_foreground_start_radius() {
        let start = Instant::now();
        let ripple = Ripple::new(Point::new(50.0, 20.0), start);
        let circle = ripple.circle(Size::new(100.0, 40.0), start);

        assert_close(circle.radius, 30.0);
    }

    #[test]
    fn ripple_enter_opacity_is_linear() {
        let start = Instant::now();
        let ripple = Ripple::new(Point::new(20.0, 20.0), start);

        assert_close(ripple.opacity(start), 0.0);
        assert_close(ripple.opacity(start + duration_ms(75)), 1.0);
    }

    #[test]
    fn rounded_rect_span_clips_full_round_corners() {
        let size = Size::new(40.0, 40.0);
        let radius = border::radius(9999.0);

        let top = rounded_rect_span_at_y(size, radius, 0.0).unwrap();
        assert_close(top.0, 20.0);
        assert_close(top.1, 20.0);

        let middle = rounded_rect_span_at_y(size, radius, 20.0).unwrap();
        assert_close(middle.0, 0.0);
        assert_close(middle.1, 40.0);

        let upper = rounded_rect_span_at_y(size, radius, 10.0).unwrap();
        assert_close(upper.0, 20.0 - (20.0_f32 * 20.0 - 10.0 * 10.0).sqrt());
        assert_close(upper.1, 20.0 + (20.0_f32 * 20.0 - 10.0 * 10.0).sqrt());
    }

    #[test]
    fn rounded_rect_span_keeps_square_bounds_without_radius() {
        let span =
            rounded_rect_span_at_y(Size::new(80.0, 40.0), border::Radius::default(), 8.0).unwrap();

        assert_close(span.0, 0.0);
        assert_close(span.1, 80.0);
    }

    #[test]
    fn ripple_clip_sampling_is_bounded_for_runtime_cost() {
        assert_eq!(ripple_clip_sample_count(1.0), RIPPLE_CLIP_MIN_SAMPLES);
        assert_eq!(ripple_clip_sample_count(100.0), RIPPLE_CLIP_MAX_SAMPLES);
    }

    #[test]
    fn short_press_ripple_holds_before_fade_out() {
        let start = Instant::now();
        let mut ripple = Ripple::new(Point::new(20.0, 20.0), start);
        let release = start + duration_ms(50);

        ripple.exit(release);

        assert_eq!(ripple.exit_delay, duration_ms(175));
        assert_close(ripple.opacity(release + duration_ms(174)), 1.0);
        assert_close(ripple.opacity(release + duration_ms(250)), 0.5);
        assert!(ripple.has_finished_exit(release + duration_ms(325)));
    }

    #[test]
    fn pressing_again_moves_existing_active_ripple_to_exiting() {
        let start = Instant::now();
        let mut state = ButtonState::default();

        state.press(Point::new(10.0, 10.0), start);
        state.press(Point::new(20.0, 20.0), start + duration_ms(20));

        assert!(state.active_ripple.is_some());
        assert_eq!(state.exiting_ripples.len(), 1);
        assert!(state.has_visible_ripples(start + duration_ms(75)));
    }

    #[test]
    fn visible_ripple_opacity_tracks_max_ripple_alpha() {
        let start = Instant::now();
        let release = start + duration_ms(50);
        let mut state = ButtonState::default();

        state.press(Point::new(10.0, 10.0), start);
        state.release(release);

        assert_close(state.visible_ripple_opacity(release + duration_ms(25)), 1.0);
        assert_close(
            state.visible_ripple_opacity(release + duration_ms(250)),
            0.5,
        );
        assert_close(
            state.visible_ripple_opacity(release + duration_ms(325)),
            0.0,
        );
    }

    #[test]
    fn visible_ripple_fades_hover_state_layer_background() {
        let theme = Theme::Light;
        let class: StyleFn<'_, Theme> = Box::new(crate::button::text);
        let active = button_draw_style(&theme, &class, Status::Active, 0.0);
        let hovered = button_draw_style(&theme, &class, Status::Hovered, 0.0);
        let covered = button_draw_style(&theme, &class, Status::Hovered, 1.0);
        let fading = button_draw_style(&theme, &class, Status::Hovered, 0.5);

        assert!(hovered.background.is_some());
        assert_eq!(covered.background, active.background);

        let Some(Background::Color(hovered_color)) = hovered.background else {
            panic!("expected hovered color background");
        };
        let Some(Background::Color(fading_color)) = fading.background else {
            panic!("expected fading color background");
        };

        assert!(fading_color.a > 0.0);
        assert!(fading_color.a < hovered_color.a);
    }

    #[test]
    fn ripple_origin_clamps_to_android_foreground_radius() {
        let size = Size::new(100.0, 40.0);
        let target_radius = ripple_target_radius(size);
        let start_radius = size.width.max(size.height) * RIPPLE_START_RADIUS_FACTOR;
        let clamped =
            clamped_ripple_origin(Point::new(500.0, 500.0), size, target_radius, start_radius);
        let center = Point::new(50.0, 20.0);
        let dx = clamped.x - center.x;
        let dy = clamped.y - center.y;

        assert_close(
            (dx * dx + dy * dy).sqrt(),
            (target_radius - start_radius).max(0.0),
        );
    }

    #[test]
    fn press_origin_ignores_mouse_outside_bounds() {
        let bounds = Rectangle {
            x: 10.0,
            y: 20.0,
            width: 40.0,
            height: 30.0,
        };
        let event = Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left));

        assert_eq!(
            press_origin(
                &event,
                bounds,
                mouse::Cursor::Available(Point::new(80.0, 80.0))
            ),
            None
        );
    }

    #[test]
    fn press_origin_returns_mouse_position_relative_to_bounds() {
        let bounds = Rectangle {
            x: 10.0,
            y: 20.0,
            width: 40.0,
            height: 30.0,
        };
        let event = Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left));

        assert_eq!(
            press_origin(
                &event,
                bounds,
                mouse::Cursor::Available(Point::new(25.0, 35.0))
            ),
            Some(Point::new(15.0, 15.0))
        );
    }

    #[test]
    fn touch_press_origin_prefers_translated_cursor_position() {
        let bounds = Rectangle {
            x: 10.0,
            y: 120.0,
            width: 40.0,
            height: 30.0,
        };
        let event = Event::Touch(touch::Event::FingerPressed {
            id: touch::Finger(0),
            position: Point::new(25.0, 35.0),
        });

        assert_eq!(
            press_origin(
                &event,
                bounds,
                mouse::Cursor::Available(Point::new(25.0, 135.0))
            ),
            Some(Point::new(15.0, 15.0))
        );
    }

    #[test]
    fn touch_press_origin_does_not_fallback_to_raw_position_when_cursor_is_available() {
        let bounds = Rectangle {
            x: 10.0,
            y: 20.0,
            width: 40.0,
            height: 30.0,
        };
        let event = Event::Touch(touch::Event::FingerPressed {
            id: touch::Finger(0),
            position: Point::new(25.0, 35.0),
        });

        assert_eq!(
            press_origin(
                &event,
                bounds,
                mouse::Cursor::Available(Point::new(25.0, 135.0))
            ),
            None
        );
    }

    #[test]
    fn touch_press_origin_does_not_fallback_to_raw_position_when_cursor_is_levitating() {
        let bounds = Rectangle {
            x: 10.0,
            y: 20.0,
            width: 40.0,
            height: 30.0,
        };
        let event = Event::Touch(touch::Event::FingerPressed {
            id: touch::Finger(0),
            position: Point::new(25.0, 35.0),
        });

        assert_eq!(
            press_origin(
                &event,
                bounds,
                mouse::Cursor::Levitating(Point::new(25.0, 135.0))
            ),
            None
        );
    }

    #[test]
    fn touch_release_uses_translated_cursor_position() {
        let bounds = Rectangle {
            x: 10.0,
            y: 120.0,
            width: 40.0,
            height: 30.0,
        };
        let event = Event::Touch(touch::Event::FingerLifted {
            id: touch::Finger(0),
            position: Point::new(25.0, 35.0),
        });

        assert!(release_is_over(
            &event,
            bounds,
            mouse::Cursor::Available(Point::new(25.0, 135.0))
        ));
    }

    #[test]
    fn touch_release_does_not_fallback_to_raw_position_when_cursor_is_levitating() {
        let bounds = Rectangle {
            x: 10.0,
            y: 20.0,
            width: 40.0,
            height: 30.0,
        };
        let event = Event::Touch(touch::Event::FingerLifted {
            id: touch::Finger(0),
            position: Point::new(25.0, 35.0),
        });

        assert!(!release_is_over(
            &event,
            bounds,
            mouse::Cursor::Levitating(Point::new(25.0, 135.0))
        ));
    }

    #[test]
    fn touch_press_origin_falls_back_to_raw_position_without_cursor() {
        let bounds = Rectangle {
            x: 10.0,
            y: 20.0,
            width: 40.0,
            height: 30.0,
        };
        let event = Event::Touch(touch::Event::FingerPressed {
            id: touch::Finger(0),
            position: Point::new(25.0, 35.0),
        });

        assert_eq!(
            press_origin(&event, bounds, mouse::Cursor::Unavailable),
            Some(Point::new(15.0, 15.0))
        );
    }
}

fn standard<'a, Message, Renderer>(
    label: impl text::IntoFragment<'a>,
    style: fn(&Theme, iced_widget::button::Status) -> iced_widget::button::Style,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
{
    Button::new(text_button_content(
        label,
        tokens::component::button::LABEL_TEXT_SIZE,
        tokens::component::button::LABEL_TEXT_LINE_HEIGHT,
        tokens::component::button::CONTAINER_HEIGHT,
        tokens::component::button::LEADING_SPACE,
    ))
    .height(Length::Fixed(tokens::component::button::CONTAINER_HEIGHT))
    .padding(Padding::ZERO)
    .style(style)
}

fn chip<'a, Message, Renderer>(
    label: impl text::IntoFragment<'a>,
    style: fn(&Theme, iced_widget::button::Status) -> iced_widget::button::Style,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
{
    Button::new(text_button_content(
        label,
        tokens::component::chip::LABEL_TEXT_SIZE,
        tokens::component::chip::LABEL_TEXT_LINE_HEIGHT,
        tokens::component::chip::CONTAINER_HEIGHT,
        tokens::component::chip::LEADING_SPACE,
    ))
    .height(Length::Fixed(tokens::component::chip::CONTAINER_HEIGHT))
    .padding(Padding::ZERO)
    .style(style)
}

fn icon<'a, Message, Renderer>(
    icon: impl text::IntoFragment<'a>,
    style: fn(&Theme, iced_widget::button::Status) -> iced_widget::button::Style,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
{
    Button::new(icon_button_content(icon))
        .width(Length::Fixed(
            tokens::component::icon_button::CONTAINER_WIDTH,
        ))
        .height(Length::Fixed(
            tokens::component::icon_button::CONTAINER_HEIGHT,
        ))
        .padding(Padding::ZERO)
        .style(style)
}

fn sized_fab<'a, Message, Renderer>(
    icon_content: impl text::IntoFragment<'a>,
    width: f32,
    height: f32,
    icon_size: f32,
    style: fn(&Theme, iced_widget::button::Status) -> iced_widget::button::Style,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
{
    Button::new(sized_fab_content(icon_content, width, height, icon_size))
        .width(Length::Fixed(width))
        .height(Length::Fixed(height))
        .padding(Padding::ZERO)
        .style(style)
}

fn fab<'a, Message, Renderer>(
    icon_content: impl text::IntoFragment<'a>,
    style: fn(&Theme, iced_widget::button::Status) -> iced_widget::button::Style,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
{
    Button::new(fab_content(icon_content))
        .width(Length::Fixed(tokens::component::fab::CONTAINER_WIDTH))
        .height(Length::Fixed(tokens::component::fab::CONTAINER_HEIGHT))
        .padding(Padding::ZERO)
        .style(style)
}

fn small_fab<'a, Message, Renderer>(
    icon_content: impl text::IntoFragment<'a>,
    style: fn(&Theme, iced_widget::button::Status) -> iced_widget::button::Style,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
{
    sized_fab(
        icon_content,
        tokens::component::fab::SMALL_CONTAINER_WIDTH,
        tokens::component::fab::SMALL_CONTAINER_HEIGHT,
        tokens::component::fab::SMALL_ICON_SIZE,
        style,
    )
}

fn large_fab<'a, Message, Renderer>(
    icon_content: impl text::IntoFragment<'a>,
    style: fn(&Theme, iced_widget::button::Status) -> iced_widget::button::Style,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
{
    sized_fab(
        icon_content,
        tokens::component::fab::LARGE_CONTAINER_WIDTH,
        tokens::component::fab::LARGE_CONTAINER_HEIGHT,
        tokens::component::fab::LARGE_ICON_SIZE,
        style,
    )
}

fn extended_fab<'a, Message, Renderer>(
    label: impl text::IntoFragment<'a>,
    style: fn(&Theme, iced_widget::button::Status) -> iced_widget::button::Style,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
{
    Button::new(extended_fab_content(label))
        .height(Length::Fixed(
            tokens::component::fab::EXTENDED_CONTAINER_HEIGHT,
        ))
        .padding(Padding::ZERO)
        .style(style)
}

fn extended_fab_with_icon<'a, Message, Renderer>(
    icon_content: impl text::IntoFragment<'a>,
    label: impl text::IntoFragment<'a>,
    style: fn(&Theme, iced_widget::button::Status) -> iced_widget::button::Style,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
{
    Button::new(extended_fab_icon_content(icon_content, label))
        .height(Length::Fixed(
            tokens::component::fab::EXTENDED_CONTAINER_HEIGHT,
        ))
        .padding(Padding::ZERO)
        .style(style)
}

pub fn elevated<'a, Message, Renderer>(
    label: impl text::IntoFragment<'a>,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
{
    standard(label, button_style::elevated)
}

pub fn filled<'a, Message, Renderer>(
    label: impl text::IntoFragment<'a>,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
{
    standard(label, button_style::filled)
}

pub fn filled_action<'a, Message, Renderer>(
    label: impl text::IntoFragment<'a>,
    on_press: Message,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
{
    filled(label).on_press(on_press).into()
}

/// Converts a Material button into an element with an optional action.
pub fn maybe_action<'a, Message, Renderer>(
    button: Button<'a, Message, Renderer>,
    enabled: bool,
    on_press: Message,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + 'a,
{
    button.on_press_maybe(enabled.then_some(on_press)).into()
}

/// Converts a group of Material buttons into elements sharing an enabled action.
pub fn enabled_actions<'a, Message, Renderer>(
    enabled: bool,
    on_press: Message,
    buttons: impl IntoIterator<Item = Button<'a, Message, Renderer>>,
) -> Vec<Element<'a, Message, Theme, Renderer>>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + 'a,
{
    buttons
        .into_iter()
        .map(|button| maybe_action(button, enabled, on_press.clone()))
        .collect()
}

pub fn filled_tonal<'a, Message, Renderer>(
    label: impl text::IntoFragment<'a>,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
{
    standard(label, button_style::filled_tonal)
}

pub fn outlined<'a, Message, Renderer>(
    label: impl text::IntoFragment<'a>,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
{
    standard(label, button_style::outlined)
}

pub fn outlined_action<'a, Message, Renderer>(
    label: impl text::IntoFragment<'a>,
    on_press: Message,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
{
    outlined(label).on_press(on_press).into()
}

pub fn text<'a, Message, Renderer>(
    label: impl text::IntoFragment<'a>,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
{
    standard(label, button_style::text)
}

pub fn text_action<'a, Message, Renderer>(
    label: impl text::IntoFragment<'a>,
    on_press: Message,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
{
    text(label).on_press(on_press).into()
}

pub fn icon_button<'a, Message, Renderer>(
    icon_content: impl text::IntoFragment<'a>,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
{
    icon(icon_content, button_style::icon)
}

pub fn filled_icon<'a, Message, Renderer>(
    icon_content: impl text::IntoFragment<'a>,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
{
    icon(icon_content, button_style::filled_icon)
}

pub fn filled_tonal_icon<'a, Message, Renderer>(
    icon_content: impl text::IntoFragment<'a>,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
{
    icon(icon_content, button_style::filled_tonal_icon)
}

pub fn outlined_icon<'a, Message, Renderer>(
    icon_content: impl text::IntoFragment<'a>,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
{
    icon(icon_content, button_style::outlined_icon)
}

pub fn primary_fab<'a, Message, Renderer>(
    icon_content: impl text::IntoFragment<'a>,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
{
    fab(icon_content, button_style::fab_primary)
}

pub fn primary_fab_action<'a, Message, Renderer>(
    icon_content: impl text::IntoFragment<'a>,
    on_press: Message,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
{
    primary_fab(icon_content).on_press(on_press).into()
}

pub fn primary_small_fab<'a, Message, Renderer>(
    icon_content: impl text::IntoFragment<'a>,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
{
    small_fab(icon_content, button_style::fab_primary_small)
}

pub fn primary_large_fab<'a, Message, Renderer>(
    icon_content: impl text::IntoFragment<'a>,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
{
    large_fab(icon_content, button_style::fab_primary_large)
}

pub fn secondary_fab<'a, Message, Renderer>(
    icon_content: impl text::IntoFragment<'a>,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
{
    fab(icon_content, button_style::fab_secondary)
}

pub fn secondary_small_fab<'a, Message, Renderer>(
    icon_content: impl text::IntoFragment<'a>,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
{
    small_fab(icon_content, button_style::fab_secondary_small)
}

pub fn secondary_large_fab<'a, Message, Renderer>(
    icon_content: impl text::IntoFragment<'a>,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
{
    large_fab(icon_content, button_style::fab_secondary_large)
}

pub fn tertiary_fab<'a, Message, Renderer>(
    icon_content: impl text::IntoFragment<'a>,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
{
    fab(icon_content, button_style::fab_tertiary)
}

pub fn tertiary_small_fab<'a, Message, Renderer>(
    icon_content: impl text::IntoFragment<'a>,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
{
    small_fab(icon_content, button_style::fab_tertiary_small)
}

pub fn tertiary_large_fab<'a, Message, Renderer>(
    icon_content: impl text::IntoFragment<'a>,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
{
    large_fab(icon_content, button_style::fab_tertiary_large)
}

pub fn surface_fab<'a, Message, Renderer>(
    icon_content: impl text::IntoFragment<'a>,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
{
    fab(icon_content, button_style::fab_surface)
}

pub fn surface_small_fab<'a, Message, Renderer>(
    icon_content: impl text::IntoFragment<'a>,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
{
    small_fab(icon_content, button_style::fab_surface_small)
}

pub fn surface_large_fab<'a, Message, Renderer>(
    icon_content: impl text::IntoFragment<'a>,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
{
    large_fab(icon_content, button_style::fab_surface_large)
}

pub fn primary_extended_fab<'a, Message, Renderer>(
    label: impl text::IntoFragment<'a>,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
{
    extended_fab(label, button_style::extended_fab_primary)
}

pub fn primary_extended_fab_with_icon<'a, Message, Renderer>(
    icon_content: impl text::IntoFragment<'a>,
    label: impl text::IntoFragment<'a>,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
{
    extended_fab_with_icon(icon_content, label, button_style::extended_fab_primary)
}

pub fn secondary_extended_fab<'a, Message, Renderer>(
    label: impl text::IntoFragment<'a>,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
{
    extended_fab(label, button_style::extended_fab_secondary)
}

pub fn secondary_extended_fab_with_icon<'a, Message, Renderer>(
    icon_content: impl text::IntoFragment<'a>,
    label: impl text::IntoFragment<'a>,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
{
    extended_fab_with_icon(icon_content, label, button_style::extended_fab_secondary)
}

pub fn tertiary_extended_fab<'a, Message, Renderer>(
    label: impl text::IntoFragment<'a>,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
{
    extended_fab(label, button_style::extended_fab_tertiary)
}

pub fn tertiary_extended_fab_with_icon<'a, Message, Renderer>(
    icon_content: impl text::IntoFragment<'a>,
    label: impl text::IntoFragment<'a>,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
{
    extended_fab_with_icon(icon_content, label, button_style::extended_fab_tertiary)
}

pub fn surface_extended_fab<'a, Message, Renderer>(
    label: impl text::IntoFragment<'a>,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
{
    extended_fab(label, button_style::extended_fab_surface)
}

pub fn surface_extended_fab_with_icon<'a, Message, Renderer>(
    icon_content: impl text::IntoFragment<'a>,
    label: impl text::IntoFragment<'a>,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
{
    extended_fab_with_icon(icon_content, label, button_style::extended_fab_surface)
}

pub fn assist_chip<'a, Message, Renderer>(
    label: impl text::IntoFragment<'a>,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
{
    chip(label, button_style::assist_chip)
}

pub fn elevated_assist_chip<'a, Message, Renderer>(
    label: impl text::IntoFragment<'a>,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
{
    chip(label, button_style::elevated_assist_chip)
}

pub fn suggestion_chip<'a, Message, Renderer>(
    label: impl text::IntoFragment<'a>,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
{
    chip(label, button_style::suggestion_chip)
}

pub fn elevated_suggestion_chip<'a, Message, Renderer>(
    label: impl text::IntoFragment<'a>,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
{
    chip(label, button_style::elevated_suggestion_chip)
}

pub fn filter_chip<'a, Message, Renderer>(
    label: impl text::IntoFragment<'a>,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
{
    chip(label, button_style::filter_chip)
}

pub fn selected_filter_chip<'a, Message, Renderer>(
    label: impl text::IntoFragment<'a>,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
{
    chip(label, button_style::selected_filter_chip)
}

pub fn input_chip<'a, Message, Renderer>(
    label: impl text::IntoFragment<'a>,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
{
    chip(label, button_style::input_chip)
}

pub fn selected_input_chip<'a, Message, Renderer>(
    label: impl text::IntoFragment<'a>,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
{
    chip(label, button_style::selected_input_chip)
}
