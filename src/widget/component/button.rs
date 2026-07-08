//! Material 3 button constructors with token-backed layout defaults.

use super::*;
use iced_widget::button::{Catalog, Status, Style, StyleFn};
use iced_widget::core::overlay;
use iced_widget::graphics::geometry;
use iced_widget::renderer::wgpu::primitive;

use super::ripple::{PressRippleState, RippleConfig, RippleStart, RippleStyle, draw_ripples};
use super::support::{AnimatedScalar, duration_ms};
use crate::utils::state_layer;

#[cfg(test)]
use super::ripple::{
    PressRipple as Ripple, RIPPLE_CLIP_MAX_SAMPLES, RIPPLE_CLIP_MIN_SAMPLES, clamped_ripple_origin,
    get_ripple_start_radius, ripple_clip_sample_count, ripple_noise_phases, ripple_target_radius,
    rounded_rect_span_at_y, unbounded_ripple_target_radius,
};

const TOUCH_CLICK_SLOP: f32 = 8.0;

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
    is_hovered: bool,
    state_layer_opacity: AnimatedScalar,
    touch_press_position: Option<Point>,
    ripples: PressRippleState,
    last_status: Option<Status>,
    now: Option<Instant>,
}

impl Default for ButtonState {
    fn default() -> Self {
        Self {
            is_pressed: false,
            is_hovered: false,
            state_layer_opacity: AnimatedScalar::new(0.0),
            touch_press_position: None,
            ripples: PressRippleState::default(),
            last_status: None,
            now: None,
        }
    }
}

impl ButtonState {
    fn press(&mut self, origin: Point, now: Instant) {
        self.is_pressed = true;
        self.ripples.press(
            origin,
            now,
            RippleStart::Additive,
            RippleStyle::material_patterned(),
        );
        self.now = Some(now);
    }

    fn release(&mut self, now: Instant) {
        self.is_pressed = false;
        self.touch_press_position = None;

        self.ripples.release(now);

        self.now = Some(now);
    }

    fn cancel(&mut self, now: Instant) {
        self.release(now);
    }

    fn sync_hover(&mut self, is_hovered: bool, now: Instant) -> bool {
        if self.is_hovered == is_hovered {
            return false;
        }

        self.is_hovered = is_hovered;
        self.animate_state_layer(now);

        true
    }

    fn snap_state_layer_to_hover_target(&mut self) {
        self.state_layer_opacity
            .snap_to(button_hover_state_layer_target(self.is_hovered));
    }

    fn animate_state_layer(&mut self, now: Instant) {
        self.state_layer_opacity.set_target(
            button_hover_state_layer_target(self.is_hovered),
            now,
            duration_ms(tokens::state::STATE_LAYER_TRANSITION_DURATION_MS),
            tokens::motion::EASING_LINEAR,
        );
    }

    fn advance(&mut self, now: Instant) -> bool {
        self.now = Some(now);
        self.prune(now);

        self.state_layer_opacity.advance(now) || self.has_visible_ripples(now)
    }

    fn state_layer_opacity(&self) -> f32 {
        self.state_layer_opacity.value
    }

    fn prune(&mut self, now: Instant) {
        self.ripples.prune(now);
    }

    fn has_visible_ripples(&self, now: Instant) -> bool {
        self.ripples.has_visible_ripples(now)
    }

    #[cfg(test)]
    fn ripple_opacity(&self, now: Instant) -> f32 {
        self.ripples.ripple_opacity(now)
    }
}

impl<Message, Renderer> Widget<Message, Theme, Renderer> for Button<'_, Message, Renderer>
where
    Message: Clone,
    Renderer: geometry::Renderer + primitive::Renderer,
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
        let is_touch_event = matches!(event, Event::Touch(_));
        let is_hovered = self.on_press.is_some() && !is_touch_event && cursor.is_over(bounds);
        let should_snap_initial_redraw_hover =
            button_should_snap_initial_redraw_hover(event, state, is_hovered);

        if button_should_sync_hover(event, cursor) && state.sync_hover(is_hovered, now_or_current())
        {
            if should_snap_initial_redraw_hover {
                state.snap_state_layer_to_hover_target();
            }

            shell.request_redraw();
        }

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. })
                if self.on_press.is_some() =>
            {
                if let Some(origin) = press_origin(event, bounds, cursor) {
                    state.press(origin, now_or_current());
                    state.touch_press_position = touch_position(event, cursor);
                    shell.capture_event();
                    shell.request_redraw();
                }
            }
            Event::Touch(touch::Event::FingerMoved { .. })
                if state.is_pressed
                    && touch_moved_beyond_click_slop(state.touch_press_position, event, cursor) =>
            {
                state.cancel(now_or_current());
                shell.request_redraw();
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. })
                if state.is_pressed =>
            {
                state.release(now_or_current());

                if release_is_over(event, bounds, cursor)
                    && let Some(on_press) = &self.on_press
                {
                    shell.publish(on_press.get());
                }

                shell.capture_event();
                shell.request_redraw();
            }
            Event::Touch(touch::Event::FingerLost { .. }) if state.is_pressed => {
                state.cancel(now_or_current());
                shell.request_redraw();
            }
            _ => {}
        }

        let current_status =
            button_status(self.on_press.is_some(), state.is_pressed, bounds, cursor);

        if let Some(now) = now {
            if state.advance(now) {
                shell.request_redraw();
            }

            self.status = Some(current_status);
            state.last_status = Some(current_status);
        } else if self.status.is_some_and(|status| status != current_status)
            || state.state_layer_opacity.is_animating()
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
        let status = button_status(self.on_press.is_some(), state.is_pressed, bounds, cursor);
        let now = state.now.unwrap_or_else(Instant::now);
        let style = button_draw_style(theme, &self.class, status);
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

        draw_button_state_layer(
            renderer,
            bounds,
            &style,
            button_state_layer_opacity_for_draw(state, status),
        );

        draw_ripples(
            renderer,
            bounds,
            &state.ripples,
            style.text_color,
            RippleConfig::bounded(style.border.radius),
            now,
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
    Renderer: geometry::Renderer + primitive::Renderer + 'a,
{
    fn from(button: Button<'a, Message, Renderer>) -> Self {
        Element::new(button)
    }
}

fn button_hover_state_layer_target(is_hovered: bool) -> f32 {
    if is_hovered {
        tokens::state::HOVER_STATE_LAYER_OPACITY
    } else {
        0.0
    }
}

fn button_should_sync_hover(event: &Event, cursor: mouse::Cursor) -> bool {
    match event {
        Event::Window(window::Event::RedrawRequested(_)) => {
            !matches!(cursor, mouse::Cursor::Unavailable)
        }
        Event::Mouse(_) | Event::Touch(_) => true,
        _ => false,
    }
}

fn button_should_snap_initial_redraw_hover(
    event: &Event,
    state: &ButtonState,
    is_hovered: bool,
) -> bool {
    matches!(event, Event::Window(window::Event::RedrawRequested(_)))
        && state.last_status.is_none()
        && is_hovered
}

fn button_state_layer_opacity_for_draw(state: &ButtonState, status: Status) -> f32 {
    if matches!(status, Status::Hovered) && state.last_status.is_none() {
        button_hover_state_layer_target(true)
    } else {
        state.state_layer_opacity()
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
) -> Style {
    if matches!(status, Status::Pressed | Status::Hovered) {
        return theme.style(class, Status::Active);
    }

    theme.style(class, status)
}

fn draw_button_state_layer<Renderer>(
    renderer: &mut Renderer,
    bounds: Rectangle,
    style: &Style,
    opacity: f32,
) where
    Renderer: geometry::Renderer,
{
    if opacity <= 0.0 {
        return;
    }

    renderer.fill_quad(
        renderer::Quad {
            bounds,
            border: Border {
                radius: style.border.radius,
                ..Border::default()
            },
            snap: style.snap,
            ..renderer::Quad::default()
        },
        state_layer(style.text_color, opacity),
    );
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

fn touch_position(event: &Event, cursor: mouse::Cursor) -> Option<Point> {
    if cursor.position().is_some() {
        return cursor.position();
    }

    if cursor.is_levitating() {
        return None;
    }

    match event {
        Event::Touch(
            touch::Event::FingerPressed { position, .. }
            | touch::Event::FingerMoved { position, .. }
            | touch::Event::FingerLifted { position, .. }
            | touch::Event::FingerLost { position, .. },
        ) => Some(*position),
        _ => None,
    }
}

fn touch_moved_beyond_click_slop(
    press_position: Option<Point>,
    event: &Event,
    cursor: mouse::Cursor,
) -> bool {
    let Some(press_position) = press_position else {
        return false;
    };
    let Some(position) = touch_position(event, cursor) else {
        return false;
    };
    let dx = position.x - press_position.x;
    let dy = position.y - press_position.y;

    dx * dx + dy * dy > TOUCH_CLICK_SLOP * TOUCH_CLICK_SLOP
}

fn relative_position(position: Point, bounds: Rectangle) -> Option<Point> {
    bounds
        .contains(position)
        .then(|| position - iced_widget::core::Vector::new(bounds.x, bounds.y))
}

#[cfg(test)]
#[path = "../../../tests/widget/component/button.rs"]
mod tests;

type ButtonStyle = fn(&Theme, Status) -> Style;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonVariant {
    Elevated,
    Filled,
    FilledTonal,
    Outlined,
    Text,
}

impl ButtonVariant {
    const fn style(self) -> ButtonStyle {
        match self {
            Self::Elevated => button_style::elevated,
            Self::Filled => button_style::filled,
            Self::FilledTonal => button_style::filled_tonal,
            Self::Outlined => button_style::outlined,
            Self::Text => button_style::text,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconButtonVariant {
    Standard,
    Filled,
    FilledTonal,
    Outlined,
}

impl IconButtonVariant {
    const fn style(self) -> ButtonStyle {
        match self {
            Self::Standard => button_style::icon,
            Self::Filled => button_style::filled_icon,
            Self::FilledTonal => button_style::filled_tonal_icon,
            Self::Outlined => button_style::outlined_icon,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FabVariant {
    Primary,
    Secondary,
    Tertiary,
    Surface,
}

impl FabVariant {
    const fn standard_style(self) -> ButtonStyle {
        match self {
            Self::Primary => button_style::fab_primary,
            Self::Secondary => button_style::fab_secondary,
            Self::Tertiary => button_style::fab_tertiary,
            Self::Surface => button_style::fab_surface,
        }
    }

    const fn small_style(self) -> ButtonStyle {
        match self {
            Self::Primary => button_style::fab_primary_small,
            Self::Secondary => button_style::fab_secondary_small,
            Self::Tertiary => button_style::fab_tertiary_small,
            Self::Surface => button_style::fab_surface_small,
        }
    }

    const fn large_style(self) -> ButtonStyle {
        match self {
            Self::Primary => button_style::fab_primary_large,
            Self::Secondary => button_style::fab_secondary_large,
            Self::Tertiary => button_style::fab_tertiary_large,
            Self::Surface => button_style::fab_surface_large,
        }
    }

    const fn extended_style(self) -> ButtonStyle {
        match self {
            Self::Primary => button_style::extended_fab_primary,
            Self::Secondary => button_style::extended_fab_secondary,
            Self::Tertiary => button_style::extended_fab_tertiary,
            Self::Surface => button_style::extended_fab_surface,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FabSize {
    Small,
    Standard,
    Large,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChipVariant {
    Assist,
    ElevatedAssist,
    Suggestion,
    ElevatedSuggestion,
    Filter,
    SelectedFilter,
    Input,
    SelectedInput,
}

impl ChipVariant {
    const fn style(self) -> ButtonStyle {
        match self {
            Self::Assist => button_style::assist_chip,
            Self::ElevatedAssist => button_style::elevated_assist_chip,
            Self::Suggestion => button_style::suggestion_chip,
            Self::ElevatedSuggestion => button_style::elevated_suggestion_chip,
            Self::Filter => button_style::filter_chip,
            Self::SelectedFilter => button_style::selected_filter_chip,
            Self::Input => button_style::input_chip,
            Self::SelectedInput => button_style::selected_input_chip,
        }
    }
}

fn text_button_content<'a, Message, Renderer>(
    label: impl text::IntoFragment<'a>,
    label_size: f32,
    label_line_height: f32,
    height: f32,
    horizontal_padding: f32,
) -> Container<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
{
    Container::new(text_with_metrics(label, label_size, label_line_height))
        .height(Length::Fixed(height))
        .padding(Padding::from([0.0, horizontal_padding]))
        .align_y(alignment::Vertical::Center)
}

fn icon_button_content<'a, Message, Renderer>(
    icon: impl text::IntoFragment<'a>,
) -> Container<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    let icon = centered_icon_text(icon, tokens::component::icon_button::ICON_SIZE);

    Container::new(icon)
        .center_x(Length::Fixed(
            tokens::component::icon_button::CONTAINER_WIDTH,
        ))
        .center_y(Length::Fixed(
            tokens::component::icon_button::CONTAINER_HEIGHT,
        ))
}

fn fab_content<'a, Message, Renderer>(
    icon: impl text::IntoFragment<'a>,
) -> Container<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    sized_fab_content(
        icon,
        tokens::component::fab::CONTAINER_WIDTH,
        tokens::component::fab::CONTAINER_HEIGHT,
        tokens::component::fab::ICON_SIZE,
    )
}

fn sized_fab_content<'a, Message, Renderer>(
    icon: impl text::IntoFragment<'a>,
    width: f32,
    height: f32,
    icon_size: f32,
) -> Container<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    let icon = centered_icon_text(icon, icon_size);

    Container::new(icon)
        .center_x(Length::Fixed(width))
        .center_y(Length::Fixed(height))
}

fn extended_fab_content<'a, Message, Renderer>(
    label: impl text::IntoFragment<'a>,
) -> Container<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
{
    let label_text = tokens::component::fab::EXTENDED_LABEL_TEXT;

    Container::new(text_with_metrics(
        label,
        label_text.size,
        label_text.line_height,
    ))
    .height(Length::Fixed(
        tokens::component::fab::EXTENDED_CONTAINER_HEIGHT,
    ))
    .padding(Padding {
        top: 0.0,
        right: tokens::component::fab::EXTENDED_TRAILING_SPACE,
        bottom: 0.0,
        left: tokens::component::fab::EXTENDED_LEADING_SPACE,
    })
    .align_y(alignment::Vertical::Center)
}

fn extended_fab_icon_content<'a, Message, Renderer>(
    icon: impl text::IntoFragment<'a>,
    label: impl text::IntoFragment<'a>,
) -> Container<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    let label_text = tokens::component::fab::EXTENDED_LABEL_TEXT;
    let content = Row::new()
        .push(centered_icon_text(
            icon,
            tokens::component::fab::EXTENDED_ICON_SIZE,
        ))
        .push(text_with_metrics(
            label,
            label_text.size,
            label_text.line_height,
        ))
        .spacing(tokens::component::fab::EXTENDED_ICON_LABEL_SPACE)
        .align_y(alignment::Vertical::Center);

    Container::new(content)
        .height(Length::Fixed(
            tokens::component::fab::EXTENDED_CONTAINER_HEIGHT,
        ))
        .padding(Padding {
            top: 0.0,
            right: tokens::component::fab::EXTENDED_TRAILING_SPACE,
            bottom: 0.0,
            left: tokens::component::fab::EXTENDED_LEADING_SPACE,
        })
        .align_y(alignment::Vertical::Center)
}

fn standard_button<'a, Message, Renderer>(
    label: impl text::IntoFragment<'a>,
    style: ButtonStyle,
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

fn chip_button<'a, Message, Renderer>(
    label: impl text::IntoFragment<'a>,
    style: ButtonStyle,
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

fn icon_button_with_style<'a, Message, Renderer>(
    icon: impl text::IntoFragment<'a>,
    style: ButtonStyle,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
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
    style: ButtonStyle,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    Button::new(sized_fab_content(icon_content, width, height, icon_size))
        .width(Length::Fixed(width))
        .height(Length::Fixed(height))
        .padding(Padding::ZERO)
        .style(style)
}

fn standard_fab<'a, Message, Renderer>(
    icon_content: impl text::IntoFragment<'a>,
    style: ButtonStyle,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    Button::new(fab_content(icon_content))
        .width(Length::Fixed(tokens::component::fab::CONTAINER_WIDTH))
        .height(Length::Fixed(tokens::component::fab::CONTAINER_HEIGHT))
        .padding(Padding::ZERO)
        .style(style)
}

fn small_fab<'a, Message, Renderer>(
    icon_content: impl text::IntoFragment<'a>,
    style: ButtonStyle,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
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
    style: ButtonStyle,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    sized_fab(
        icon_content,
        tokens::component::fab::LARGE_CONTAINER_WIDTH,
        tokens::component::fab::LARGE_CONTAINER_HEIGHT,
        tokens::component::fab::LARGE_ICON_SIZE,
        style,
    )
}

fn extended_fab_button<'a, Message, Renderer>(
    label: impl text::IntoFragment<'a>,
    style: ButtonStyle,
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

fn extended_fab_button_with_icon<'a, Message, Renderer>(
    icon_content: impl text::IntoFragment<'a>,
    label: impl text::IntoFragment<'a>,
    style: ButtonStyle,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    Button::new(extended_fab_icon_content(icon_content, label))
        .height(Length::Fixed(
            tokens::component::fab::EXTENDED_CONTAINER_HEIGHT,
        ))
        .padding(Padding::ZERO)
        .style(style)
}

pub fn button<'a, Message, Renderer>(
    label: impl text::IntoFragment<'a>,
    variant: ButtonVariant,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
{
    standard_button(label, variant.style())
}

pub fn action<'a, Message, Renderer>(
    button: Button<'a, Message, Renderer>,
    on_press: Message,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + primitive::Renderer + 'a,
{
    button.on_press(on_press).into()
}

pub fn optional_action<'a, Message, Renderer>(
    button: Button<'a, Message, Renderer>,
    on_press: Option<Message>,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + primitive::Renderer + 'a,
{
    button.on_press_maybe(on_press).into()
}

pub fn enabled_actions<'a, Message, Renderer>(
    enabled: bool,
    on_press: Message,
    buttons: impl IntoIterator<Item = Button<'a, Message, Renderer>>,
) -> Vec<Element<'a, Message, Theme, Renderer>>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + primitive::Renderer + 'a,
{
    buttons
        .into_iter()
        .map(|button| optional_action(button, enabled.then_some(on_press.clone())))
        .collect()
}

pub fn icon_button<'a, Message, Renderer>(
    icon_content: impl text::IntoFragment<'a>,
    variant: IconButtonVariant,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    icon_button_with_style(icon_content, variant.style())
}

pub fn fab<'a, Message, Renderer>(
    icon_content: impl text::IntoFragment<'a>,
    variant: FabVariant,
    size: FabSize,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    match size {
        FabSize::Small => small_fab(icon_content, variant.small_style()),
        FabSize::Standard => standard_fab(icon_content, variant.standard_style()),
        FabSize::Large => large_fab(icon_content, variant.large_style()),
    }
}

pub fn extended_fab<'a, Message, Renderer>(
    label: impl text::IntoFragment<'a>,
    variant: FabVariant,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
{
    extended_fab_button(label, variant.extended_style())
}

pub fn extended_fab_with_icon<'a, Message, Renderer>(
    icon_content: impl text::IntoFragment<'a>,
    label: impl text::IntoFragment<'a>,
    variant: FabVariant,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    extended_fab_button_with_icon(icon_content, label, variant.extended_style())
}

pub fn chip<'a, Message, Renderer>(
    label: impl text::IntoFragment<'a>,
    variant: ChipVariant,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
{
    chip_button(label, variant.style())
}
