//! Material 3 dialog surface constructors.

use iced_widget::button::{Status, Style};
use iced_widget::core::text as core_text;
use iced_widget::core::time::Instant;
use iced_widget::core::widget::{self, Tree, tree};
use iced_widget::core::{
    Background, Clipboard, Color, Element, Event, Layout, Length, Padding, Rectangle, Shell, Size,
    Transformation, Vector, Widget, alignment, border, layout, mouse, overlay, renderer, touch,
};
use iced_widget::graphics::geometry;
use iced_widget::renderer::wgpu::primitive;
use iced_widget::text;
use iced_widget::{Column, Container, Row, Space, Stack, Text, opaque};

use super::absolute_line_height;
use super::support::{alpha_color, duration_ms, lerp};
use crate::utils::shadow_from_level;
use crate::{Theme, fonts, tokens};

/// Android dialog visibility animation state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transition {
    phase: TransitionPhase,
    started_at: Option<Instant>,
}

/// Android dialog transition phase.
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
        }
    }
}

impl Transition {
    /// Starts showing the dialog using Android's platform dialog window animation.
    pub fn show(&mut self, now: Instant) {
        match self.phase {
            TransitionPhase::Showing | TransitionPhase::Shown => {}
            TransitionPhase::Hidden | TransitionPhase::Dismissing => {
                self.phase = TransitionPhase::Showing;
                self.started_at = Some(now);
            }
        }
    }

    /// Starts dismissing the dialog.
    pub fn dismiss(&mut self, now: Instant) {
        if self.is_active() && self.phase != TransitionPhase::Dismissing {
            self.phase = TransitionPhase::Dismissing;
            self.started_at = Some(now);
        }
    }

    /// Advances timers and hides the dialog once Android's exit animation ends.
    pub fn advance(&mut self, now: Instant) -> bool {
        match self.phase {
            TransitionPhase::Hidden => {}
            TransitionPhase::Showing => {
                if self.scale_progress(now) >= 1.0 {
                    self.phase = TransitionPhase::Shown;
                    self.started_at = None;
                }
            }
            TransitionPhase::Shown => {}
            TransitionPhase::Dismissing => {
                if self.scale_progress(now) >= 1.0 {
                    *self = Self::default();
                }
            }
        }

        self.is_animating()
    }

    /// Returns whether the dialog should remain in the view tree.
    pub fn is_active(&self) -> bool {
        self.phase != TransitionPhase::Hidden
    }

    /// Returns whether the dialog is currently running an enter or exit animation.
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

    /// Computes the Android dialog surface scale.
    pub fn scale(&self, now: Instant) -> f32 {
        let eased = android_decelerate(
            self.scale_progress(now),
            tokens::component::dialog::DECELERATE_QUINT_FACTOR,
        );

        match self.phase {
            TransitionPhase::Hidden => tokens::component::dialog::ENTER_SCALE_FROM,
            TransitionPhase::Showing => {
                lerp(tokens::component::dialog::ENTER_SCALE_FROM, 1.0, eased)
            }
            TransitionPhase::Shown => 1.0,
            TransitionPhase::Dismissing => {
                lerp(1.0, tokens::component::dialog::EXIT_SCALE_TO, eased)
            }
        }
    }

    /// Computes the Android dialog window alpha.
    pub fn alpha(&self, now: Instant) -> f32 {
        let eased = android_decelerate(
            self.alpha_progress(now),
            tokens::component::dialog::DECELERATE_CUBIC_FACTOR,
        );

        match self.phase {
            TransitionPhase::Hidden => 0.0,
            TransitionPhase::Showing => eased,
            TransitionPhase::Shown => 1.0,
            TransitionPhase::Dismissing => 1.0 - eased,
        }
    }

    /// Computes the modal scrim fade progress.
    pub fn scrim_alpha(&self, now: Instant) -> f32 {
        let eased = android_decelerate(
            self.scrim_progress(now),
            tokens::component::dialog::DECELERATE_CUBIC_FACTOR,
        );

        match self.phase {
            TransitionPhase::Hidden => 0.0,
            TransitionPhase::Showing => eased,
            TransitionPhase::Shown => 1.0,
            TransitionPhase::Dismissing => 1.0 - eased,
        }
    }

    fn scale_progress(&self, now: Instant) -> f32 {
        self.progress(now, tokens::component::dialog::SCALE_ANIMATION_DURATION_MS)
    }

    fn alpha_progress(&self, now: Instant) -> f32 {
        self.progress(now, tokens::component::dialog::ALPHA_ANIMATION_DURATION_MS)
    }

    fn scrim_progress(&self, now: Instant) -> f32 {
        self.progress(now, tokens::component::dialog::SCRIM_ANIMATION_DURATION_MS)
    }

    fn progress(&self, now: Instant, duration_ms_value: u16) -> f32 {
        let Some(started_at) = self.started_at else {
            return match self.phase {
                TransitionPhase::Showing | TransitionPhase::Dismissing => 1.0,
                TransitionPhase::Hidden | TransitionPhase::Shown => 0.0,
            };
        };

        let duration = duration_ms(duration_ms_value);

        if duration.is_zero() {
            return 1.0;
        }

        (now.saturating_duration_since(started_at).as_secs_f32() / duration.as_secs_f32())
            .clamp(0.0, 1.0)
    }
}

/// Visual options for dialog pieces that only need an alpha override.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AlphaOptions {
    pub alpha: f32,
}

impl Default for AlphaOptions {
    fn default() -> Self {
        Self { alpha: 1.0 }
    }
}

impl AlphaOptions {
    /// Sets the Android window alpha applied to the dialog piece.
    pub fn alpha(mut self, alpha: f32) -> Self {
        self.alpha = alpha;
        self
    }
}

/// Visual options for alert dialog content.
#[derive(Debug, Clone, PartialEq)]
pub struct AlertOptions<'a> {
    pub icon: Option<text::Fragment<'a>>,
    pub alpha: f32,
}

impl Default for AlertOptions<'_> {
    fn default() -> Self {
        Self {
            icon: None,
            alpha: 1.0,
        }
    }
}

impl<'a> AlertOptions<'a> {
    /// Sets the optional hero icon slot.
    pub fn icon(mut self, icon: impl text::IntoFragment<'a>) -> Self {
        self.icon = Some(icon.into_fragment());
        self
    }

    /// Sets the Android window alpha applied to the alert content.
    pub fn alpha(mut self, alpha: f32) -> Self {
        self.alpha = alpha;
        self
    }
}

/// Creates a Material 3 basic dialog surface around custom content.
pub fn basic<'a, Message, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Container<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
{
    basic_with(content, AlphaOptions::default())
}

/// Creates a Material 3 basic dialog surface with custom visual options.
pub fn basic_with<'a, Message, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
    options: AlphaOptions,
) -> Container<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
{
    Container::new(content)
        .width(Length::Fill)
        .max_width(tokens::component::dialog::CONTAINER_MAX_WIDTH)
        .padding(tokens::component::dialog::CONTAINER_PADDING)
        .style(move |theme| container_style_alpha(theme, options.alpha))
}

/// Creates a Material 3 dialog with a title, arbitrary interactive content,
/// and actions.
pub fn content<'a, Message, Renderer>(
    title: impl text::IntoFragment<'a>,
    body: impl Into<Element<'a, Message, Theme, Renderer>>,
    actions: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Container<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
{
    content_with(title, body, actions, AlphaOptions::default())
}

/// Creates a Material 3 dialog with arbitrary interactive content and custom
/// visual options.
///
/// The alpha applies to the dialog-owned surface and title. Custom body and
/// action widgets should use the same alpha when participating in a dialog
/// transition.
pub fn content_with<'a, Message, Renderer>(
    title: impl text::IntoFragment<'a>,
    body: impl Into<Element<'a, Message, Theme, Renderer>>,
    actions: impl Into<Element<'a, Message, Theme, Renderer>>,
    options: AlphaOptions,
) -> Container<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
{
    dialog_content(
        None,
        title_text(title, alignment::Horizontal::Left, options.alpha),
        body.into(),
        actions,
        options.alpha,
    )
}

/// Creates a Material 3 alert dialog with title, supporting text, and actions.
pub fn alert<'a, Message, Renderer>(
    title: impl text::IntoFragment<'a>,
    supporting_text: impl text::IntoFragment<'a>,
    actions: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Container<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    alert_with(title, supporting_text, actions, AlertOptions::default())
}

/// Creates a Material 3 alert dialog with custom visual options.
pub fn alert_with<'a, Message, Renderer>(
    title: impl text::IntoFragment<'a>,
    supporting_text: impl text::IntoFragment<'a>,
    actions: impl Into<Element<'a, Message, Theme, Renderer>>,
    options: AlertOptions<'a>,
) -> Container<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    let alpha = options.alpha;
    let has_icon = options.icon.is_some();
    let icon = options.icon.map(|icon| icon_text(icon, alpha));
    let body = supporting_text_view(supporting_text, alpha).into();

    dialog_content(
        icon,
        title_text(title, title_alignment(has_icon), alpha),
        body,
        actions,
        alpha,
    )
}

/// Creates a right-aligned Material 3 dialog actions row.
pub fn actions<'a, Message, Renderer>(
    buttons: impl IntoIterator<Item = Element<'a, Message, Theme, Renderer>>,
) -> Container<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + 'a,
{
    Container::new(
        Row::with_children(buttons)
            .spacing(tokens::component::dialog::ACTIONS_HORIZONTAL_SPACING)
            .align_y(alignment::Vertical::Center),
    )
    .width(Length::Fill)
    .align_x(alignment::Horizontal::Right)
}

/// Creates a Material 3 dialog text action.
pub fn action<'a, Message, Renderer>(
    label: impl text::IntoFragment<'a>,
) -> super::button::Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'a,
{
    action_with(label, AlphaOptions::default())
}

/// Creates a Material 3 dialog text action with an on-press message.
pub fn action_button<'a, Message, Renderer>(
    label: impl text::IntoFragment<'a>,
    on_press: Message,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'a,
{
    action_button_with(label, on_press, AlphaOptions::default())
}

/// Creates a Material 3 dialog text action with custom visual options.
pub fn action_with<'a, Message, Renderer>(
    label: impl text::IntoFragment<'a>,
    options: AlphaOptions,
) -> super::button::Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'a,
{
    super::button::button(label, super::button::ButtonVariant::Text)
        .style(move |theme, status| action_style_alpha(theme, status, options.alpha))
}

/// Creates a Material 3 dialog text action button with custom visual options.
pub fn action_button_with<'a, Message, Renderer>(
    label: impl text::IntoFragment<'a>,
    on_press: Message,
    options: AlphaOptions,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'a,
{
    action_with(label, options).on_press(on_press).into()
}

/// Creates a Material 3 modal dialog scrim behind overlay content.
pub fn scrim<'a, Message, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Container<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + 'a,
{
    scrim_with(content, AlphaOptions::default())
}

/// Creates a Material 3 modal dialog scrim with custom visual options.
pub fn scrim_with<'a, Message, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
    options: AlphaOptions,
) -> Container<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + 'a,
{
    Container::new(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |theme| scrim_style_alpha(theme, options.alpha))
}

/// Centers a Material 3 dialog surface over a modal scrim.
pub fn modal_overlay<'a, Message, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Container<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + 'a,
{
    scrim(
        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center),
    )
}

/// Creates an event-blocking Material 3 modal dialog layer.
///
/// The scrim and dialog surface both absorb mouse presses, so clicks outside
/// the dialog do not pass through to content underneath and do not require a
/// no-op application message.
pub fn modal_layer<'a, Message, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + 'a,
{
    let scrim = opaque(scrim(Space::new().width(Length::Fill).height(Length::Fill)));
    let dialog = opaque(Container::new(content).center(Length::Fill));

    Stack::with_children([scrim, dialog])
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// Creates an event-blocking Material 3 modal dialog layer animated like Android.
pub fn modal_layer_animated<'a, Message, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
    transition: &Transition,
    now: Instant,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + 'a,
{
    let scrim = opaque(scrim_with(
        Space::new().width(Length::Fill).height(Length::Fill),
        AlphaOptions::default().alpha(transition.scrim_alpha(now)),
    ));
    let dialog = opaque(
        Container::new(scaled(
            content,
            transition.scale(now),
            transition.phase() != TransitionPhase::Dismissing,
        ))
        .center(Length::Fill),
    );

    Stack::with_children([scrim, dialog])
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// Places a Material 3 modal dialog layer over existing content.
pub fn modal<'a, Message, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
    dialog: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + 'a,
{
    Stack::with_children([content.into(), modal_layer(dialog)])
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// Places an Android-animated Material 3 modal dialog layer over existing content.
pub fn modal_animated<'a, Message, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
    transition: &Transition,
    now: Instant,
    dialog: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + 'a,
{
    let content = content.into();

    if !transition.is_active() {
        return content;
    }

    Stack::with_children([content, modal_layer_animated(dialog, transition, now)])
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn dialog_content<'a, Message, Renderer>(
    icon: Option<Text<'a, Theme, Renderer>>,
    title: Text<'a, Theme, Renderer>,
    body: Element<'a, Message, Theme, Renderer>,
    actions: impl Into<Element<'a, Message, Theme, Renderer>>,
    alpha: f32,
) -> Container<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
{
    let mut content = Column::new().width(Length::Fill);

    if let Some(icon) = icon {
        content = content.push(
            Container::new(icon)
                .width(Length::Fill)
                .padding(Padding {
                    top: 0.0,
                    right: 0.0,
                    bottom: tokens::component::dialog::ICON_BOTTOM_PADDING,
                    left: 0.0,
                })
                .align_x(alignment::Horizontal::Center),
        );
    }

    content = content.push(Container::new(title).width(Length::Fill).padding(Padding {
        top: 0.0,
        right: 0.0,
        bottom: tokens::component::dialog::TITLE_BOTTOM_PADDING,
        left: 0.0,
    }));

    content = content.push(Container::new(body).width(Length::Fill).padding(Padding {
        top: 0.0,
        right: 0.0,
        bottom: tokens::component::dialog::SUPPORTING_TEXT_BOTTOM_PADDING,
        left: 0.0,
    }));

    content = content.push(actions.into());

    basic_with(content, AlphaOptions::default().alpha(alpha))
}

fn icon_text<'a, Renderer>(icon: text::Fragment<'a>, alpha: f32) -> Text<'a, Theme, Renderer>
where
    Renderer: core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    fonts::filled_icon(icon, tokens::component::dialog::ICON_SIZE)
        .width(Length::Fixed(tokens::component::dialog::ICON_SIZE))
        .height(Length::Fixed(tokens::component::dialog::ICON_SIZE))
        .center()
        .style(move |theme| icon_style_alpha(theme, alpha))
}

fn title_text<'a, Renderer>(
    title: impl text::IntoFragment<'a>,
    alignment: alignment::Horizontal,
    alpha: f32,
) -> Text<'a, Theme, Renderer>
where
    Renderer: core_text::Renderer + 'a,
{
    let scale = tokens::component::dialog::HEADLINE_TEXT;

    Text::new(title)
        .size(scale.size)
        .line_height(absolute_line_height(scale.line_height))
        .width(Length::Fill)
        .align_x(alignment)
        .color_maybe(None::<iced_widget::core::Color>)
        .style(move |theme| title_style_alpha(theme, alpha))
}

fn title_alignment(has_icon: bool) -> alignment::Horizontal {
    if has_icon {
        alignment::Horizontal::Center
    } else {
        alignment::Horizontal::Left
    }
}

fn supporting_text_view<'a, Renderer>(
    supporting_text: impl text::IntoFragment<'a>,
    alpha: f32,
) -> Text<'a, Theme, Renderer>
where
    Renderer: core_text::Renderer + 'a,
{
    let scale = tokens::component::dialog::SUPPORTING_TEXT;

    Text::new(supporting_text)
        .size(scale.size)
        .line_height(absolute_line_height(scale.line_height))
        .width(Length::Fill)
        .color_maybe(None::<iced_widget::core::Color>)
        .style(move |theme| supporting_text_style_alpha(theme, alpha))
}

fn container_style(theme: &Theme) -> iced_widget::container::Style {
    let colors = theme.colors();

    iced_widget::container::Style {
        background: Some(Background::Color(colors.surface.container.high)),
        text_color: Some(colors.surface.text_variant),
        border: border::rounded(tokens::component::dialog::CONTAINER_SHAPE),
        shadow: shadow_from_level(
            tokens::component::dialog::CONTAINER_ELEVATION_LEVEL,
            colors.shadow,
        ),
        snap: cfg!(feature = "crisp"),
    }
}

fn container_style_alpha(theme: &Theme, alpha: f32) -> iced_widget::container::Style {
    let mut style = container_style(theme);

    style.background = style
        .background
        .map(|background| background.scale_alpha(alpha));
    style.text_color = style.text_color.map(|color| alpha_color(color, alpha));
    style.border.color = alpha_color(style.border.color, alpha);
    style.shadow.color = alpha_color(style.shadow.color, alpha);

    style
}

fn icon_style(theme: &Theme) -> iced_widget::text::Style {
    iced_widget::text::Style {
        color: Some(theme.colors().secondary.color),
    }
}

fn icon_style_alpha(theme: &Theme, alpha: f32) -> iced_widget::text::Style {
    iced_widget::text::Style {
        color: icon_style(theme)
            .color
            .map(|color| alpha_color(color, alpha)),
    }
}

fn title_style(theme: &Theme) -> iced_widget::text::Style {
    iced_widget::text::Style {
        color: Some(theme.colors().surface.text),
    }
}

fn title_style_alpha(theme: &Theme, alpha: f32) -> iced_widget::text::Style {
    iced_widget::text::Style {
        color: title_style(theme)
            .color
            .map(|color| alpha_color(color, alpha)),
    }
}

fn supporting_text_style(theme: &Theme) -> iced_widget::text::Style {
    iced_widget::text::Style {
        color: Some(theme.colors().surface.text_variant),
    }
}

fn supporting_text_style_alpha(theme: &Theme, alpha: f32) -> iced_widget::text::Style {
    iced_widget::text::Style {
        color: supporting_text_style(theme)
            .color
            .map(|color| alpha_color(color, alpha)),
    }
}

fn action_style_alpha(theme: &Theme, status: Status, alpha: f32) -> Style {
    let mut style = crate::style::button::text(theme, status);

    style.background = style
        .background
        .map(|background| background.scale_alpha(alpha));
    style.text_color = alpha_color(style.text_color, alpha);
    style.border.color = alpha_color(style.border.color, alpha);
    style.shadow.color = alpha_color(style.shadow.color, alpha);

    style
}

fn scrim_style_alpha(theme: &Theme, alpha: f32) -> iced_widget::container::Style {
    iced_widget::container::Style {
        background: Some(Background::Color(alpha_color(
            Color {
                a: 1.0,
                ..theme.colors().scrim
            },
            tokens::component::dialog::SCRIM_OPACITY * alpha,
        ))),
        text_color: Some(theme.colors().surface.text),
        ..iced_widget::container::Style::default()
    }
}

fn scaled<'a, Message, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
    scale: f32,
    interactive: bool,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + 'a,
{
    Element::new(Scaled {
        content: content.into(),
        scale,
        interactive,
    })
}

struct Scaled<'a, Message, Renderer> {
    content: Element<'a, Message, Theme, Renderer>,
    scale: f32,
    interactive: bool,
}

struct ScaledTag;

impl<Message, Renderer> std::fmt::Debug for Scaled<'_, Message, Renderer> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Scaled")
            .field("scale", &self.scale)
            .field("interactive", &self.interactive)
            .finish_non_exhaustive()
    }
}

impl<Message, Renderer> Scaled<'_, Message, Renderer> {
    fn transformation(&self, bounds: Rectangle) -> Transformation {
        Transformation::translate(bounds.center_x(), bounds.center_y())
            * Transformation::scale(self.scale)
            * Transformation::translate(-bounds.center_x(), -bounds.center_y())
    }
}

impl<Message, Renderer> Widget<Message, Theme, Renderer> for Scaled<'_, Message, Renderer>
where
    Renderer: iced_widget::core::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<ScaledTag>()
    }

    fn state(&self) -> tree::State {
        tree::State::None
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
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

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
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
        if !self.interactive
            && !matches!(
                event,
                Event::Window(iced_widget::core::window::Event::RedrawRequested(_))
            )
        {
            return;
        }

        let inverse = self.transformation(layout.bounds()).inverse();
        let cursor = if self.interactive {
            cursor * inverse
        } else {
            mouse::Cursor::Unavailable
        };
        let transformed_event = transform_pointer_event(event, inverse);
        let event = transformed_event.as_ref().unwrap_or(event);

        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            &(*viewport * inverse),
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
        if !self.interactive {
            return mouse::Interaction::None;
        }

        let inverse = self.transformation(layout.bounds()).inverse();

        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor * inverse,
            &(*viewport * inverse),
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
        let Some(layer_bounds) = scaled_layer_bounds(layout.bounds(), viewport) else {
            return;
        };
        let transformation = self.transformation(layout.bounds());
        let inverse = transformation.inverse();
        let cursor = if self.interactive {
            cursor * inverse
        } else {
            mouse::Cursor::Unavailable
        };

        renderer.with_layer(layer_bounds, |renderer| {
            renderer.with_transformation(transformation, |renderer| {
                self.content.as_widget().draw(
                    &tree.children[0],
                    renderer,
                    theme,
                    style,
                    layout,
                    cursor,
                    &(*viewport * inverse),
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
        if !self.interactive {
            return None;
        }

        let transformation = self.transformation(layout.bounds());
        let inverse = transformation.inverse();

        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            &(*viewport * inverse),
            translation + transformation.translation(),
        )
    }
}

fn transform_pointer_event(event: &Event, transformation: Transformation) -> Option<Event> {
    match event {
        Event::Mouse(mouse::Event::CursorMoved { position }) => {
            Some(Event::Mouse(mouse::Event::CursorMoved {
                position: *position * transformation,
            }))
        }
        Event::Touch(touch::Event::FingerPressed { id, position }) => {
            Some(Event::Touch(touch::Event::FingerPressed {
                id: *id,
                position: *position * transformation,
            }))
        }
        Event::Touch(touch::Event::FingerMoved { id, position }) => {
            Some(Event::Touch(touch::Event::FingerMoved {
                id: *id,
                position: *position * transformation,
            }))
        }
        Event::Touch(touch::Event::FingerLifted { id, position }) => {
            Some(Event::Touch(touch::Event::FingerLifted {
                id: *id,
                position: *position * transformation,
            }))
        }
        Event::Touch(touch::Event::FingerLost { id, position }) => {
            Some(Event::Touch(touch::Event::FingerLost {
                id: *id,
                position: *position * transformation,
            }))
        }
        _ => None,
    }
}

fn scaled_layer_bounds(bounds: Rectangle, viewport: &Rectangle) -> Option<Rectangle> {
    bounds.intersection(viewport).map(|_| *viewport)
}

fn android_decelerate(progress: f32, factor: f32) -> f32 {
    let progress = progress.clamp(0.0, 1.0);

    // Mirrors Android's DecelerateInterpolator: 1 - (1 - t) ^ (2 * factor).
    if (factor - 1.0).abs() <= f32::EPSILON {
        1.0 - (1.0 - progress) * (1.0 - progress)
    } else {
        1.0 - (1.0 - progress).powf(2.0 * factor)
    }
}

#[cfg(test)]
#[path = "../../../tests/widget/component/dialog.rs"]
mod tests;
