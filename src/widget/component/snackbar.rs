//! Material 3 snackbar surface constructors.

use iced_widget::button::{Status, Style};
use iced_widget::core::text as core_text;
use iced_widget::core::time::{Duration, Instant};
use iced_widget::core::widget::{self, Tree, tree};
use iced_widget::core::{
    Background, Border, Clipboard, Color, Element, Event, Layout, Length, Padding, Rectangle,
    Shadow, Shell, Size, Vector, Widget, alignment, border, layout, mouse, overlay, renderer,
};
use iced_widget::graphics::geometry;
use iced_widget::renderer::wgpu::primitive;
use iced_widget::text;
use iced_widget::{Container, Row, Stack, Text};

use super::support::{alpha_color, duration_ms, lerp};
use super::{absolute_line_height, button::Button};
use crate::utils::{shadow_from_level, state_layer};
use crate::{Theme, fonts, tokens};

/// Android snackbar visibility animation state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transition {
    phase: TransitionPhase,
    started_at: Option<Instant>,
    shown_at: Option<Instant>,
    duration: Duration,
}

/// Android snackbar transition phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransitionPhase {
    Hidden,
    Showing,
    Shown,
    Dismissing,
}

impl Default for Transition {
    fn default() -> Self {
        Self {
            phase: TransitionPhase::Hidden,
            started_at: None,
            shown_at: None,
            duration: duration_ms(tokens::component::snackbar::LONG_DURATION_MS),
        }
    }
}

impl Transition {
    /// Starts showing the snackbar using Android's default long snackbar duration.
    pub fn show(&mut self, now: Instant) {
        self.show_for(
            now,
            duration_ms(tokens::component::snackbar::LONG_DURATION_MS),
        );
    }

    /// Starts showing the snackbar using a custom visible duration.
    pub fn show_for(&mut self, now: Instant, duration: Duration) {
        match self.phase {
            TransitionPhase::Showing | TransitionPhase::Shown => {
                self.duration = duration;
                self.shown_at = Some(now);
            }
            TransitionPhase::Hidden | TransitionPhase::Dismissing => {
                self.phase = TransitionPhase::Showing;
                self.started_at = Some(now);
                self.shown_at = None;
                self.duration = duration;
            }
        }
    }

    /// Starts dismissing the snackbar.
    pub fn dismiss(&mut self, now: Instant) {
        if self.is_active() && self.phase != TransitionPhase::Dismissing {
            self.phase = TransitionPhase::Dismissing;
            self.started_at = Some(now);
            self.shown_at = None;
        }
    }

    /// Advances timers and hides the snackbar after its Android timeout.
    pub fn advance(&mut self, now: Instant) -> bool {
        match self.phase {
            TransitionPhase::Hidden => {}
            TransitionPhase::Showing => {
                if self.slide_progress(now) >= 1.0 {
                    self.phase = TransitionPhase::Shown;
                    self.started_at = None;
                    self.shown_at = Some(now);
                }
            }
            TransitionPhase::Shown => {
                if self.shown_at.is_some_and(|shown_at| {
                    now.saturating_duration_since(shown_at) >= self.duration
                }) {
                    self.dismiss(now);
                }
            }
            TransitionPhase::Dismissing => {
                if self.slide_progress(now) >= 1.0 {
                    *self = Self::default();
                }
            }
        }

        self.is_active()
    }

    /// Returns whether the snackbar should remain in the view tree.
    pub fn is_active(&self) -> bool {
        self.phase != TransitionPhase::Hidden
    }

    /// Returns whether the snackbar is currently running an enter or exit animation.
    pub fn is_animating(&self) -> bool {
        matches!(
            self.phase,
            TransitionPhase::Showing | TransitionPhase::Dismissing
        )
    }

    /// Returns the current transition phase.
    pub fn phase(&self) -> TransitionPhase {
        self.phase
    }

    /// Computes the Android slide translation for the provided hidden distance.
    pub fn translation_y(&self, now: Instant, hidden_distance: f32) -> f32 {
        let eased =
            tokens::component::snackbar::SLIDE_ANIMATION_EASING.transform(self.slide_progress(now));

        match self.phase {
            TransitionPhase::Hidden => hidden_distance,
            TransitionPhase::Showing => lerp(hidden_distance, 0.0, eased),
            TransitionPhase::Shown => 0.0,
            TransitionPhase::Dismissing => lerp(0.0, hidden_distance, eased),
        }
    }

    /// Computes the Android content fade alpha.
    pub fn content_alpha(&self, now: Instant) -> f32 {
        match self.phase {
            TransitionPhase::Hidden => 0.0,
            TransitionPhase::Showing => {
                let Some(started_at) = self.started_at else {
                    return 1.0;
                };

                let elapsed = now.saturating_duration_since(started_at);
                let delay = duration_ms(
                    tokens::component::snackbar::SLIDE_ANIMATION_DURATION_MS
                        - tokens::component::snackbar::CONTENT_FADE_ANIMATION_DURATION_MS,
                );

                if elapsed <= delay {
                    return 0.0;
                }

                let fade_duration =
                    duration_ms(tokens::component::snackbar::CONTENT_FADE_ANIMATION_DURATION_MS);
                let progress =
                    ((elapsed - delay).as_secs_f32() / fade_duration.as_secs_f32()).clamp(0.0, 1.0);

                tokens::component::snackbar::CONTENT_FADE_ANIMATION_EASING.transform(progress)
            }
            TransitionPhase::Shown => 1.0,
            TransitionPhase::Dismissing => {
                let Some(started_at) = self.started_at else {
                    return 0.0;
                };

                let fade_duration =
                    duration_ms(tokens::component::snackbar::CONTENT_FADE_ANIMATION_DURATION_MS);
                let progress = (now.saturating_duration_since(started_at).as_secs_f32()
                    / fade_duration.as_secs_f32())
                .clamp(0.0, 1.0);

                1.0 - tokens::component::snackbar::CONTENT_FADE_ANIMATION_EASING.transform(progress)
            }
        }
    }

    fn slide_progress(&self, now: Instant) -> f32 {
        let Some(started_at) = self.started_at else {
            return match self.phase {
                TransitionPhase::Showing | TransitionPhase::Dismissing => 1.0,
                TransitionPhase::Hidden | TransitionPhase::Shown => 0.0,
            };
        };

        let duration = duration_ms(tokens::component::snackbar::SLIDE_ANIMATION_DURATION_MS);

        (now.saturating_duration_since(started_at).as_secs_f32() / duration.as_secs_f32())
            .clamp(0.0, 1.0)
    }
}

/// Snackbar text layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lines {
    Single,
    Two,
}

impl Lines {
    const fn container_height(self) -> f32 {
        match self {
            Self::Single => tokens::component::snackbar::WITH_SINGLE_LINE_CONTAINER_HEIGHT,
            Self::Two => tokens::component::snackbar::WITH_TWO_LINES_CONTAINER_HEIGHT,
        }
    }
}

/// Visual options for snackbar content.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Options {
    pub lines: Lines,
    pub content_alpha: f32,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            lines: Lines::Single,
            content_alpha: 1.0,
        }
    }
}

impl Options {
    /// Sets the snackbar text layout.
    pub fn lines(mut self, lines: Lines) -> Self {
        self.lines = lines;
        self
    }

    /// Sets the Android content fade alpha.
    pub fn content_alpha(mut self, content_alpha: f32) -> Self {
        self.content_alpha = content_alpha;
        self
    }
}

/// Visual options for snackbar action buttons.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ActionOptions {
    pub content_alpha: f32,
}

impl Default for ActionOptions {
    fn default() -> Self {
        Self { content_alpha: 1.0 }
    }
}

impl ActionOptions {
    /// Sets the Android content fade alpha.
    pub fn content_alpha(mut self, content_alpha: f32) -> Self {
        self.content_alpha = content_alpha;
        self
    }
}

/// Layout options for a snackbar host.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HostOptions {
    /// Distance between the snackbar bottom edge and the host bottom edge.
    pub bottom_margin: f32,
}

impl Default for HostOptions {
    fn default() -> Self {
        Self {
            bottom_margin: tokens::component::snackbar::BOTTOM_MARGIN,
        }
    }
}

impl HostOptions {
    /// Sets the distance between the snackbar and the host bottom edge.
    pub fn bottom_margin(mut self, bottom_margin: f32) -> Self {
        self.bottom_margin = bottom_margin.max(0.0);
        self
    }

    /// Places the snackbar above a floating action button, matching the
    /// vertical stacking used by Material 3 Compose `Scaffold` while
    /// preserving the snackbar's standard outer margin.
    pub fn above_fab(mut self, fab_height: f32, fab_bottom_margin: f32) -> Self {
        self.bottom_margin = fab_height.max(0.0)
            + fab_bottom_margin.max(0.0)
            + tokens::component::snackbar::BOTTOM_MARGIN;
        self
    }
}

/// Creates a snackbar surface.
pub fn surface<'a, Message, Renderer>(
    message: impl text::IntoFragment<'a>,
    action: Option<Element<'a, Message, Theme, Renderer>>,
    options: Options,
) -> Container<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
{
    surface_container(
        message,
        action,
        options.lines.container_height(),
        options.content_alpha,
    )
}

/// Creates a snackbar text action button.
pub fn action<'a, Message, Renderer>(
    label: impl text::IntoFragment<'a>,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'a,
{
    action_with(label, ActionOptions::default())
}

/// Creates a snackbar text action with an on-press message.
pub fn action_button<'a, Message, Renderer>(
    label: impl text::IntoFragment<'a>,
    on_press: Message,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'a,
{
    action_button_with(label, on_press, ActionOptions::default())
}

/// Creates a snackbar text action button with custom visual options.
pub fn action_with<'a, Message, Renderer>(
    label: impl text::IntoFragment<'a>,
    options: ActionOptions,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'a,
{
    let label_text = tokens::component::snackbar::ACTION_LABEL_TEXT;

    Button::new(
        Container::new(
            Text::new(label)
                .size(label_text.size)
                .line_height(absolute_line_height(label_text.line_height)),
        )
        .height(Length::Fixed(tokens::component::button::CONTAINER_HEIGHT))
        .padding(Padding {
            top: 0.0,
            right: tokens::component::button::TRAILING_SPACE,
            bottom: 0.0,
            left: tokens::component::button::LEADING_SPACE,
        })
        .align_y(alignment::Vertical::Center),
    )
    .height(Length::Fixed(tokens::component::button::CONTAINER_HEIGHT))
    .padding(Padding::ZERO)
    .style(move |theme, status| action_style_alpha(theme, status, options.content_alpha))
}

/// Creates a snackbar text action button with custom visual options.
pub fn action_button_with<'a, Message, Renderer>(
    label: impl text::IntoFragment<'a>,
    on_press: Message,
    options: ActionOptions,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'a,
{
    action_with(label, options).on_press(on_press).into()
}

/// Creates a snackbar icon action, typically used for dismiss.
pub fn icon_action<'a, Message, Renderer>(
    icon_name: impl text::IntoFragment<'a>,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    Button::new(
        Container::new(fonts::icon(
            icon_name,
            tokens::component::snackbar::ICON_SIZE,
        ))
        .center_x(Length::Fixed(
            tokens::component::icon_button::CONTAINER_WIDTH,
        ))
        .center_y(Length::Fixed(
            tokens::component::icon_button::CONTAINER_HEIGHT,
        )),
    )
    .width(Length::Fixed(
        tokens::component::icon_button::CONTAINER_WIDTH,
    ))
    .height(Length::Fixed(
        tokens::component::icon_button::CONTAINER_HEIGHT,
    ))
    .padding(Padding::ZERO)
    .style(icon_action_style)
}

/// Creates a snackbar icon action with an on-press message.
pub fn icon_action_button<'a, Message, Renderer>(
    icon_name: impl text::IntoFragment<'a>,
    on_press: Message,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    icon_action(icon_name).on_press(on_press).into()
}

/// Places snackbar content above the app content and translates it from the bottom edge.
pub fn overlay<'a, Message, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
    snackbar: impl Into<Element<'a, Message, Theme, Renderer>>,
    translation_y: f32,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + 'a,
{
    overlay_with(content, snackbar, translation_y, HostOptions::default())
}

/// Places snackbar content above app content with custom host layout options.
pub fn overlay_with<'a, Message, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
    snackbar: impl Into<Element<'a, Message, Theme, Renderer>>,
    translation_y: f32,
    options: HostOptions,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + 'a,
{
    Stack::with_children([
        content.into(),
        floating_layer(snackbar, translation_y, options.bottom_margin).into(),
    ])
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

/// Places an Android-animated single-line snackbar with one action over content.
pub fn host<'a, Message, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
    transition: &Transition,
    now: Instant,
    message: impl text::IntoFragment<'a>,
    action_label: impl text::IntoFragment<'a>,
    on_action: Message,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'a,
{
    host_with(
        content,
        transition,
        now,
        message,
        action_label,
        on_action,
        HostOptions::default(),
    )
}

/// Places an Android-animated single-line snackbar over content with custom
/// host layout options.
pub fn host_with<'a, Message, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
    transition: &Transition,
    now: Instant,
    message: impl text::IntoFragment<'a>,
    action_label: impl text::IntoFragment<'a>,
    on_action: Message,
    options: HostOptions,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'a,
{
    if !transition.is_active() {
        return content.into();
    }

    let alpha = transition.content_alpha(now);
    let hidden_distance =
        tokens::component::snackbar::WITH_SINGLE_LINE_CONTAINER_HEIGHT + options.bottom_margin;
    let translation_y = transition.translation_y(now, hidden_distance);
    let snackbar = surface(
        message,
        Some(action_button_with(
            action_label,
            on_action,
            ActionOptions::default().content_alpha(alpha),
        )),
        Options::default().content_alpha(alpha),
    );

    overlay_with(content, snackbar, translation_y, options)
}

fn surface_container<'a, Message, Renderer>(
    message: impl text::IntoFragment<'a>,
    action: Option<Element<'a, Message, Theme, Renderer>>,
    height: f32,
    content_alpha: f32,
) -> Container<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
{
    let supporting_text = tokens::component::snackbar::SUPPORTING_TEXT;
    let mut content = Row::new()
        .push(
            Text::new(message)
                .size(supporting_text.size)
                .line_height(absolute_line_height(supporting_text.line_height))
                .wrapping(text::Wrapping::Word)
                .style(move |theme: &Theme| text::Style {
                    color: Some(alpha_color(
                        theme.colors().inverse.inverse_surface_text,
                        content_alpha,
                    )),
                })
                .width(Length::Fill),
        )
        .spacing(8)
        .align_y(alignment::Vertical::Center);

    if let Some(action) = action {
        content = content.push(action);
    }

    Container::new(content)
        .height(Length::Fixed(height))
        .width(Length::Fill)
        .padding(Padding {
            top: 0.0,
            right: 8.0,
            bottom: 0.0,
            left: 16.0,
        })
        .align_y(alignment::Vertical::Center)
        .style(container_style)
}

fn floating_layer<'a, Message, Renderer>(
    snackbar: impl Into<Element<'a, Message, Theme, Renderer>>,
    translation_y: f32,
    bottom_margin: f32,
) -> Container<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + 'a,
{
    let snackbar = Container::new(snackbar)
        .width(Length::Fill)
        .max_width(tokens::component::snackbar::MAX_WIDTH);

    Container::new(translated(snackbar, Vector::new(0.0, translation_y)))
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(Padding {
            top: 0.0,
            right: tokens::component::snackbar::HORIZONTAL_MARGIN,
            bottom: bottom_margin,
            left: tokens::component::snackbar::HORIZONTAL_MARGIN,
        })
        .align_x(alignment::Horizontal::Center)
        .align_y(alignment::Vertical::Bottom)
}

fn translated<'a, Message, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
    translation: Vector,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + 'a,
{
    Element::new(Translated {
        content: content.into(),
        translation,
    })
}

struct Translated<'a, Message, Renderer> {
    content: Element<'a, Message, Theme, Renderer>,
    translation: Vector,
}

impl<Message, Renderer> std::fmt::Debug for Translated<'_, Message, Renderer> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Translated")
            .field("translation", &self.translation)
            .finish_non_exhaustive()
    }
}

impl<Message, Renderer> Widget<Message, Theme, Renderer> for Translated<'_, Message, Renderer>
where
    Renderer: iced_widget::core::Renderer,
{
    fn tag(&self) -> tree::Tag {
        self.content.as_widget().tag()
    }

    fn state(&self) -> tree::State {
        self.content.as_widget().state()
    }

    fn children(&self) -> Vec<Tree> {
        self.content.as_widget().children()
    }

    fn diff(&self, tree: &mut Tree) {
        self.content.as_widget().diff(tree);
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
        self.content.as_widget_mut().layout(tree, renderer, limits)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        self.content
            .as_widget_mut()
            .operate(tree, layout, renderer, operation);
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
        let translation = self.translation;

        self.content.as_widget_mut().update(
            tree,
            event,
            layout,
            cursor - translation,
            renderer,
            clipboard,
            shell,
            &(*viewport - translation),
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
        let translation = self.translation;

        self.content.as_widget().mouse_interaction(
            tree,
            layout,
            cursor - translation,
            &(*viewport - translation),
            renderer,
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let Some(viewport) = layout.bounds().intersection(viewport) else {
            return;
        };
        let translation = self.translation;

        renderer.with_layer(viewport, |renderer| {
            renderer.with_translation(translation, |renderer| {
                self.content.as_widget().draw(
                    tree,
                    renderer,
                    theme,
                    style,
                    layout,
                    cursor - translation,
                    &(viewport - translation),
                );
            });
        });
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        self.content.as_widget_mut().overlay(
            tree,
            layout,
            renderer,
            viewport,
            translation + self.translation,
        )
    }
}

fn container_style(theme: &Theme) -> iced_widget::container::Style {
    let colors = theme.colors();

    // M3 snackbars use inverse roles, so dark themes intentionally get
    // a light snackbar surface for contrast against the app surface.
    iced_widget::container::Style {
        background: Some(Background::Color(colors.inverse.inverse_surface)),
        text_color: Some(colors.inverse.inverse_surface_text),
        border: border::rounded(tokens::component::snackbar::CONTAINER_SHAPE),
        shadow: shadow_from_level(
            tokens::component::snackbar::CONTAINER_ELEVATION_LEVEL,
            colors.shadow,
        ),
        snap: cfg!(feature = "crisp"),
    }
}

fn action_style_alpha(theme: &Theme, status: Status, content_alpha: f32) -> Style {
    let colors = theme.colors();
    let foreground = alpha_color(colors.inverse.inverse_primary, content_alpha);
    let active = Style {
        background: None,
        text_color: foreground,
        border: border::rounded(tokens::component::button::CONTAINER_SHAPE),
        shadow: Shadow::default(),
        snap: cfg!(feature = "crisp"),
    };

    match status {
        Status::Active => active,
        Status::Hovered => Style {
            background: Some(Background::Color(state_layer(
                foreground,
                tokens::state::HOVER_STATE_LAYER_OPACITY,
            ))),
            ..active
        },
        Status::Pressed => Style {
            background: Some(Background::Color(state_layer(
                foreground,
                tokens::state::PRESSED_STATE_LAYER_OPACITY,
            ))),
            ..active
        },
        Status::Disabled => Style {
            text_color: Color {
                a: tokens::state::DISABLED_LABEL_TEXT_OPACITY,
                ..foreground
            },
            ..active
        },
    }
}

fn icon_action_style(theme: &Theme, status: Status) -> Style {
    let colors = theme.colors();
    let foreground = colors.inverse.inverse_surface_text;
    let active = Style {
        background: None,
        text_color: foreground,
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: tokens::component::icon_button::CONTAINER_SHAPE.into(),
        },
        shadow: Shadow::default(),
        snap: cfg!(feature = "crisp"),
    };

    match status {
        Status::Active => active,
        Status::Hovered => Style {
            background: Some(Background::Color(state_layer(
                foreground,
                tokens::state::HOVER_STATE_LAYER_OPACITY,
            ))),
            ..active
        },
        Status::Pressed => Style {
            background: Some(Background::Color(state_layer(
                foreground,
                tokens::state::PRESSED_STATE_LAYER_OPACITY,
            ))),
            ..active
        },
        Status::Disabled => Style {
            text_color: Color {
                a: tokens::state::DISABLED_LABEL_TEXT_OPACITY,
                ..foreground
            },
            ..active
        },
    }
}

#[cfg(test)]
#[path = "../../../tests/widget/component/snackbar.rs"]
mod tests;
