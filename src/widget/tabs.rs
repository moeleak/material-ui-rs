//! Material 3 primary and secondary tab constructors.

use iced_widget::button::{Status, Style};
use iced_widget::core::border::Radius;
use iced_widget::core::text as core_text;
use iced_widget::core::time::Instant;
use iced_widget::core::{
    Background, Border, Color, Element, Layout, Length, Padding, Rectangle, Size, Widget,
    alignment, border, layout, mouse, renderer,
};
use iced_widget::graphics::geometry;
use iced_widget::text;
use iced_widget::{Column, Container, Row, Space, Text};

use super::absolute_line_height;
use super::button::Button;
use super::support::{AnimatedScalar, duration_ms};
use crate::utils::{mix, shadow_from_level};
use crate::{Theme, fonts, tokens};

/// Animated tab selection state.
#[derive(Debug, Clone)]
pub struct State {
    selected_index: usize,
    indicator_position: AnimatedScalar,
}

impl State {
    /// Creates tab selection state with the initial selected index.
    pub fn new(selected_index: usize) -> Self {
        Self {
            selected_index,
            indicator_position: AnimatedScalar::new(selected_index as f32),
        }
    }

    /// Returns the selected tab index.
    pub const fn selected_index(&self) -> usize {
        self.selected_index
    }

    /// Starts the Material tab indicator transition to `selected_index`.
    pub fn select(&mut self, selected_index: usize, now: Instant, variant: Variant) {
        if self.selected_index == selected_index {
            return;
        }

        self.selected_index = selected_index;
        self.indicator_position.set_target(
            selected_index as f32,
            now,
            duration_ms(variant.indicator_animation_duration_ms()),
            variant.indicator_animation_easing(),
        );
    }

    /// Advances the running transition.
    pub fn advance(&mut self, now: Instant) -> bool {
        self.indicator_position.advance(now)
    }

    /// Returns whether the indicator transition is still running.
    pub fn is_animating(&self) -> bool {
        self.indicator_position.is_animating()
    }

    fn indicator_position(&self) -> f32 {
        self.indicator_position.value
    }
}

/// The Material tab variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Variant {
    Primary,
    Secondary,
}

impl Variant {
    const fn container_height(self) -> f32 {
        match self {
            Self::Primary => tokens::component::primary_tab::CONTAINER_HEIGHT,
            Self::Secondary => tokens::component::secondary_tab::CONTAINER_HEIGHT,
        }
    }

    const fn indicator_height(self) -> f32 {
        match self {
            Self::Primary => tokens::component::primary_tab::ACTIVE_INDICATOR_HEIGHT,
            Self::Secondary => tokens::component::secondary_tab::ACTIVE_INDICATOR_HEIGHT,
        }
    }

    const fn label_text(self) -> tokens::typography::TypeScale {
        match self {
            Self::Primary => tokens::component::primary_tab::LABEL_TEXT,
            Self::Secondary => tokens::component::secondary_tab::LABEL_TEXT,
        }
    }

    const fn icon_size(self) -> f32 {
        match self {
            Self::Primary => tokens::component::primary_tab::ICON_SIZE,
            Self::Secondary => tokens::component::secondary_tab::ICON_SIZE,
        }
    }

    const fn indicator_animation_duration_ms(self) -> u16 {
        match self {
            Self::Primary => tokens::component::primary_tab::INDICATOR_ANIMATION_DURATION_MS,
            Self::Secondary => tokens::component::secondary_tab::INDICATOR_ANIMATION_DURATION_MS,
        }
    }

    const fn indicator_animation_easing(self) -> tokens::motion::CubicBezier {
        match self {
            Self::Primary => tokens::component::primary_tab::INDICATOR_ANIMATION_EASING,
            Self::Secondary => tokens::component::secondary_tab::INDICATOR_ANIMATION_EASING,
        }
    }
}

/// Creates an equal-width Material tab bar.
pub fn bar<'a, Message, Renderer>(
    tabs: impl IntoIterator<Item = Element<'a, Message, Theme, Renderer>>,
) -> Row<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + 'a,
{
    Row::with_children(tabs.into_iter())
        .spacing(0)
        .align_y(alignment::Vertical::Bottom)
        .width(Length::Fill)
}

/// Creates an equal-width Material tab bar with an animated shared indicator.
pub fn animated_bar<'a, Message, Renderer>(
    variant: Variant,
    tab_count: usize,
    state: &State,
    tabs: impl IntoIterator<Item = Element<'a, Message, Theme, Renderer>>,
) -> Column<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + 'a,
{
    Column::new()
        .push(bar(tabs))
        .push(MovingIndicator {
            variant,
            tab_count,
            position: state.indicator_position(),
        })
        .spacing(0)
        .width(Length::Fill)
}

/// Creates a primary animated tab bar from icon-label action items.
pub fn animated_primary_icon_label_bar<'a, Message, Renderer, Icon, Label>(
    state: &State,
    tabs: impl IntoIterator<Item = (Icon, Label, Message)>,
) -> Column<'a, Message, Theme, Renderer>
where
    Icon: text::IntoFragment<'a>,
    Label: text::IntoFragment<'a>,
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    let tabs: Vec<_> = tabs
        .into_iter()
        .enumerate()
        .map(|(index, (icon, label, on_press))| {
            primary_icon_label_action_for_animated_bar(
                icon,
                label,
                state.selected_index() == index,
                on_press,
            )
        })
        .collect();

    animated_bar(Variant::Primary, tabs.len(), state, tabs)
}

/// Creates a secondary animated tab bar from label action items.
pub fn animated_secondary_label_bar<'a, Message, Renderer, Label>(
    state: &State,
    tabs: impl IntoIterator<Item = (Label, Message)>,
) -> Column<'a, Message, Theme, Renderer>
where
    Label: text::IntoFragment<'a>,
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
{
    let tabs: Vec<_> = tabs
        .into_iter()
        .enumerate()
        .map(|(index, (label, on_press))| {
            secondary_label_action_for_animated_bar(
                label,
                state.selected_index() == index,
                on_press,
            )
        })
        .collect();

    animated_bar(Variant::Secondary, tabs.len(), state, tabs)
}

/// Creates a primary label tab.
pub fn primary_label<'a, Message, Renderer>(
    label: impl text::IntoFragment<'a>,
    active: bool,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
{
    label_tab(Variant::Primary, label, active)
}

/// Creates a stacked primary tab with icon and label.
pub fn primary_icon_label<'a, Message, Renderer>(
    icon_name: impl text::IntoFragment<'a>,
    label: impl text::IntoFragment<'a>,
    active: bool,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    let variant = Variant::Primary;
    let label_text = variant.label_text();
    let content = Column::<Message, Theme, Renderer>::new()
        .push(fonts::filled_icon(icon_name, variant.icon_size()))
        .push(
            Text::new(label)
                .size(label_text.size)
                .line_height(absolute_line_height(label_text.line_height)),
        )
        .spacing(tokens::component::primary_tab::STACKED_ICON_LABEL_SPACE)
        .align_x(alignment::Horizontal::Center);

    tab_button(
        variant,
        content.into(),
        active,
        tokens::component::primary_tab::WITH_ICON_AND_LABEL_TEXT_CONTAINER_HEIGHT,
    )
}

/// Creates a stacked primary tab for an [`animated_bar`].
pub fn primary_icon_label_for_animated_bar<'a, Message, Renderer>(
    icon_name: impl text::IntoFragment<'a>,
    label: impl text::IntoFragment<'a>,
    active: bool,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    let variant = Variant::Primary;
    let label_text = variant.label_text();
    let content = Column::<Message, Theme, Renderer>::new()
        .push(fonts::filled_icon(icon_name, variant.icon_size()))
        .push(
            Text::new(label)
                .size(label_text.size)
                .line_height(absolute_line_height(label_text.line_height)),
        )
        .spacing(tokens::component::primary_tab::STACKED_ICON_LABEL_SPACE)
        .align_x(alignment::Horizontal::Center);

    animated_tab_button(
        variant,
        content.into(),
        active,
        tokens::component::primary_tab::WITH_ICON_AND_LABEL_TEXT_CONTAINER_HEIGHT,
    )
}

/// Creates a stacked primary tab with an action message for an [`animated_bar`].
pub fn primary_icon_label_action_for_animated_bar<'a, Message, Renderer>(
    icon_name: impl text::IntoFragment<'a>,
    label: impl text::IntoFragment<'a>,
    active: bool,
    on_press: Message,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    primary_icon_label_for_animated_bar(icon_name, label, active)
        .on_press(on_press)
        .into()
}

/// Creates an inline primary tab with icon and label.
pub fn primary_inline_icon_label<'a, Message, Renderer>(
    icon_name: impl text::IntoFragment<'a>,
    label: impl text::IntoFragment<'a>,
    active: bool,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    inline_icon_label_tab(Variant::Primary, icon_name, label, active)
}

/// Creates a secondary label tab.
pub fn secondary_label<'a, Message, Renderer>(
    label: impl text::IntoFragment<'a>,
    active: bool,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
{
    label_tab(Variant::Secondary, label, active)
}

/// Creates a primary label tab for an [`animated_bar`].
pub fn primary_label_for_animated_bar<'a, Message, Renderer>(
    label: impl text::IntoFragment<'a>,
    active: bool,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
{
    animated_label_tab(Variant::Primary, label, active)
}

/// Creates an inline primary tab with icon and label for an [`animated_bar`].
pub fn primary_inline_icon_label_for_animated_bar<'a, Message, Renderer>(
    icon_name: impl text::IntoFragment<'a>,
    label: impl text::IntoFragment<'a>,
    active: bool,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    animated_inline_icon_label_tab(Variant::Primary, icon_name, label, active)
}

/// Creates a secondary label tab for an [`animated_bar`].
pub fn secondary_label_for_animated_bar<'a, Message, Renderer>(
    label: impl text::IntoFragment<'a>,
    active: bool,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
{
    animated_label_tab(Variant::Secondary, label, active)
}

/// Creates a secondary label tab with an action message for an [`animated_bar`].
pub fn secondary_label_action_for_animated_bar<'a, Message, Renderer>(
    label: impl text::IntoFragment<'a>,
    active: bool,
    on_press: Message,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
{
    secondary_label_for_animated_bar(label, active)
        .on_press(on_press)
        .into()
}

/// Creates an inline secondary tab with icon and label.
pub fn secondary_icon_label<'a, Message, Renderer>(
    icon_name: impl text::IntoFragment<'a>,
    label: impl text::IntoFragment<'a>,
    active: bool,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    inline_icon_label_tab(Variant::Secondary, icon_name, label, active)
}

/// Creates an inline secondary tab with icon and label for an [`animated_bar`].
pub fn secondary_icon_label_for_animated_bar<'a, Message, Renderer>(
    icon_name: impl text::IntoFragment<'a>,
    label: impl text::IntoFragment<'a>,
    active: bool,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    animated_inline_icon_label_tab(Variant::Secondary, icon_name, label, active)
}

fn animated_label_tab<'a, Message, Renderer>(
    variant: Variant,
    label: impl text::IntoFragment<'a>,
    active: bool,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
{
    let label_text = variant.label_text();
    animated_tab_button(
        variant,
        Text::new(label)
            .size(label_text.size)
            .line_height(absolute_line_height(label_text.line_height))
            .into(),
        active,
        variant.container_height(),
    )
}

fn animated_inline_icon_label_tab<'a, Message, Renderer>(
    variant: Variant,
    icon_name: impl text::IntoFragment<'a>,
    label: impl text::IntoFragment<'a>,
    active: bool,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    let label_text = variant.label_text();
    let gap = match variant {
        Variant::Primary => tokens::component::primary_tab::INLINE_ICON_LABEL_SPACE,
        Variant::Secondary => tokens::component::secondary_tab::ICON_LABEL_SPACE,
    };
    let content = Row::<Message, Theme, Renderer>::new()
        .push(fonts::filled_icon(icon_name, variant.icon_size()))
        .push(
            Text::new(label)
                .size(label_text.size)
                .line_height(absolute_line_height(label_text.line_height)),
        )
        .spacing(gap)
        .align_y(alignment::Vertical::Center);

    animated_tab_button(variant, content.into(), active, variant.container_height())
}

fn label_tab<'a, Message, Renderer>(
    variant: Variant,
    label: impl text::IntoFragment<'a>,
    active: bool,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
{
    let label_text = variant.label_text();
    tab_button(
        variant,
        Text::new(label)
            .size(label_text.size)
            .line_height(absolute_line_height(label_text.line_height))
            .into(),
        active,
        variant.container_height(),
    )
}

fn inline_icon_label_tab<'a, Message, Renderer>(
    variant: Variant,
    icon_name: impl text::IntoFragment<'a>,
    label: impl text::IntoFragment<'a>,
    active: bool,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    let label_text = variant.label_text();
    let gap = match variant {
        Variant::Primary => tokens::component::primary_tab::INLINE_ICON_LABEL_SPACE,
        Variant::Secondary => tokens::component::secondary_tab::ICON_LABEL_SPACE,
    };
    let content = Row::<Message, Theme, Renderer>::new()
        .push(fonts::filled_icon(icon_name, variant.icon_size()))
        .push(
            Text::new(label)
                .size(label_text.size)
                .line_height(absolute_line_height(label_text.line_height)),
        )
        .spacing(gap)
        .align_y(alignment::Vertical::Center);

    tab_button(variant, content.into(), active, variant.container_height())
}

fn tab_button<'a, Message, Renderer>(
    variant: Variant,
    content: Element<'a, Message, Theme, Renderer>,
    active: bool,
    height: f32,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
{
    tab_button_with_indicator(variant, content, active, height, true)
}

fn animated_tab_button<'a, Message, Renderer>(
    variant: Variant,
    content: Element<'a, Message, Theme, Renderer>,
    active: bool,
    height: f32,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
{
    tab_button_with_indicator(
        variant,
        content,
        active,
        height - variant.indicator_height(),
        false,
    )
}

fn tab_button_with_indicator<'a, Message, Renderer>(
    variant: Variant,
    content: Element<'a, Message, Theme, Renderer>,
    active: bool,
    height: f32,
    show_indicator: bool,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
{
    let tab_content = Column::new().push(
        Container::new(content)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .padding(Padding {
                top: 0.0,
                right: horizontal_space(variant),
                bottom: 0.0,
                left: horizontal_space(variant),
            }),
    );
    let tab_content = if show_indicator {
        tab_content.push(indicator(variant, active))
    } else {
        tab_content
    }
    .width(Length::Fill)
    .height(Length::Fixed(height));

    Button::new(tab_content)
        .width(Length::Fill)
        .height(Length::Fixed(height))
        .padding(Padding::ZERO)
        .style(move |theme, status| tab_style(theme, status, variant, active))
}

fn indicator<'a, Message, Renderer>(
    variant: Variant,
    active: bool,
) -> Container<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + 'a,
{
    Container::new(Space::new())
        .width(Length::Fill)
        .height(Length::Fixed(variant.indicator_height()))
        .style(move |theme| indicator_style(theme, variant, active))
}

const fn horizontal_space(variant: Variant) -> f32 {
    match variant {
        Variant::Primary => tokens::component::primary_tab::HORIZONTAL_SPACE,
        Variant::Secondary => tokens::component::secondary_tab::HORIZONTAL_SPACE,
    }
}

/// Returns the container style for a Material tab.
pub fn tab_style(theme: &Theme, status: Status, variant: Variant, active: bool) -> Style {
    let colors = theme.colors();
    let surface = colors.surface;
    let content = tab_content_color(theme, variant, active, status);
    let layer = tab_state_layer_color(theme, variant, active, status);
    let container = surface.color;

    let active_style = Style {
        background: Some(Background::Color(container)),
        text_color: content,
        border: border::rounded(tab_container_shape(variant)),
        shadow: shadow_from_level(tab_container_elevation(variant), colors.shadow),
        snap: cfg!(feature = "crisp"),
    };

    match status {
        Status::Active => active_style,
        Status::Hovered => Style {
            background: Some(Background::Color(mix(
                container,
                layer,
                tab_hover_opacity(variant, active),
            ))),
            ..active_style
        },
        Status::Pressed => Style {
            background: Some(Background::Color(mix(
                container,
                layer,
                tab_pressed_opacity(variant, active),
            ))),
            ..active_style
        },
        Status::Disabled => Style {
            background: Some(Background::Color(container)),
            text_color: Color {
                a: tokens::state::DISABLED_LABEL_TEXT_OPACITY,
                ..surface.text
            },
            ..active_style
        },
    }
}

fn tab_content_color(theme: &Theme, variant: Variant, active: bool, status: Status) -> Color {
    let colors = theme.colors();

    match (variant, active, status) {
        (Variant::Primary, true, _) => colors.primary.color,
        (Variant::Primary, false, Status::Active) => colors.surface.text_variant,
        (Variant::Primary, false, _) => colors.surface.text,
        (Variant::Secondary, true, _) => colors.surface.text,
        (Variant::Secondary, false, Status::Active) => colors.surface.text_variant,
        (Variant::Secondary, false, _) => colors.surface.text,
    }
}

fn tab_state_layer_color(theme: &Theme, variant: Variant, active: bool, status: Status) -> Color {
    let colors = theme.colors();

    match (variant, active, status) {
        (Variant::Primary, true, _) => colors.primary.color,
        (Variant::Primary, false, Status::Pressed) => colors.primary.color,
        (Variant::Primary, false, _) => colors.surface.text,
        (Variant::Secondary, _, _) => colors.surface.text,
    }
}

const fn tab_hover_opacity(variant: Variant, active: bool) -> f32 {
    match (variant, active) {
        (Variant::Primary, true) => {
            tokens::component::primary_tab::ACTIVE_HOVER_STATE_LAYER_OPACITY
        }
        (Variant::Primary, false) => {
            tokens::component::primary_tab::INACTIVE_HOVER_STATE_LAYER_OPACITY
        }
        (Variant::Secondary, _) => tokens::component::secondary_tab::HOVER_STATE_LAYER_OPACITY,
    }
}

const fn tab_pressed_opacity(variant: Variant, active: bool) -> f32 {
    match (variant, active) {
        (Variant::Primary, true) => {
            tokens::component::primary_tab::ACTIVE_PRESSED_STATE_LAYER_OPACITY
        }
        (Variant::Primary, false) => {
            tokens::component::primary_tab::INACTIVE_PRESSED_STATE_LAYER_OPACITY
        }
        (Variant::Secondary, _) => tokens::component::secondary_tab::PRESSED_STATE_LAYER_OPACITY,
    }
}

const fn tab_container_shape(variant: Variant) -> f32 {
    match variant {
        Variant::Primary => tokens::component::primary_tab::CONTAINER_SHAPE,
        Variant::Secondary => tokens::component::secondary_tab::CONTAINER_SHAPE,
    }
}

const fn tab_container_elevation(variant: Variant) -> u8 {
    match variant {
        Variant::Primary => tokens::component::primary_tab::CONTAINER_ELEVATION_LEVEL,
        Variant::Secondary => tokens::component::secondary_tab::CONTAINER_ELEVATION_LEVEL,
    }
}

fn indicator_style(theme: &Theme, variant: Variant, active: bool) -> iced_widget::container::Style {
    let colors = theme.colors();
    let background = if active {
        colors.primary.color
    } else {
        Color::TRANSPARENT
    };

    iced_widget::container::Style {
        background: Some(Background::Color(background)),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: indicator_radius(variant),
        },
        snap: cfg!(feature = "crisp"),
        ..Default::default()
    }
}

fn indicator_radius(variant: Variant) -> Radius {
    match variant {
        Variant::Primary => Radius {
            top_left: tokens::component::primary_tab::ACTIVE_INDICATOR_SHAPE_TOP,
            top_right: tokens::component::primary_tab::ACTIVE_INDICATOR_SHAPE_TOP,
            bottom_right: tokens::component::primary_tab::ACTIVE_INDICATOR_SHAPE_BOTTOM,
            bottom_left: tokens::component::primary_tab::ACTIVE_INDICATOR_SHAPE_BOTTOM,
        },
        Variant::Secondary => Radius::new(tokens::component::secondary_tab::ACTIVE_INDICATOR_SHAPE),
    }
}

#[derive(Debug, Clone, Copy)]
struct MovingIndicator {
    variant: Variant,
    tab_count: usize,
    position: f32,
}

impl<Message, Renderer> Widget<Message, Theme, Renderer> for MovingIndicator
where
    Renderer: iced_widget::core::Renderer,
{
    fn size(&self) -> Size<Length> {
        Size {
            width: Length::Fill,
            height: Length::Fixed(self.variant.indicator_height()),
        }
    }

    fn layout(
        &mut self,
        _tree: &mut iced_widget::core::widget::Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::Node::new(limits.resolve(
            Length::Fill,
            Length::Fixed(self.variant.indicator_height()),
            Size::ZERO,
        ))
    }

    fn draw(
        &self,
        _tree: &iced_widget::core::widget::Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _defaults: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        if self.tab_count == 0 {
            return;
        }

        let bounds = layout.bounds();
        let tab_width = bounds.width / self.tab_count as f32;

        if tab_width <= 0.0 {
            return;
        }

        let position = self
            .position
            .clamp(0.0, self.tab_count.saturating_sub(1) as f32);
        let indicator_width = moving_indicator_width(self.variant, tab_width);
        let x = bounds.x + tab_width * position + (tab_width - indicator_width) / 2.0;
        let indicator_bounds = Rectangle {
            x,
            y: bounds.y,
            width: indicator_width,
            height: self.variant.indicator_height(),
        };

        renderer.fill_quad(
            renderer::Quad {
                bounds: indicator_bounds,
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: indicator_radius(self.variant),
                },
                snap: cfg!(feature = "crisp"),
                ..renderer::Quad::default()
            },
            Background::Color(theme.colors().primary.color),
        );
    }
}

impl<'a, Message, Renderer> From<MovingIndicator> for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + 'a,
{
    fn from(indicator: MovingIndicator) -> Self {
        Element::new(indicator)
    }
}

fn moving_indicator_width(variant: Variant, tab_width: f32) -> f32 {
    match variant {
        Variant::Primary => (tab_width - horizontal_space(variant) * 2.0).max(0.0),
        Variant::Secondary => tab_width,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primary_tab_active_style_uses_primary_tokens() {
        let theme = Theme::Light;
        let colors = theme.colors();
        let style = tab_style(&theme, Status::Active, Variant::Primary, true);

        assert_eq!(
            style.background,
            Some(Background::Color(colors.surface.color))
        );
        assert_eq!(style.text_color, colors.primary.color);
        assert_eq!(style.shadow.offset.y, 0.0);
    }

    #[test]
    fn inactive_tabs_use_on_surface_variant_until_interaction() {
        let theme = Theme::Light;
        let colors = theme.colors();
        let active = tab_style(&theme, Status::Active, Variant::Secondary, false);
        let hovered = tab_style(&theme, Status::Hovered, Variant::Secondary, false);

        assert_eq!(active.text_color, colors.surface.text_variant);
        assert_eq!(hovered.text_color, colors.surface.text);
        assert_eq!(
            hovered.background,
            Some(Background::Color(mix(
                colors.surface.color,
                colors.surface.text,
                tokens::component::secondary_tab::HOVER_STATE_LAYER_OPACITY
            )))
        );
    }

    #[test]
    fn indicator_uses_primary_shape_and_secondary_square_shape() {
        let primary = indicator_radius(Variant::Primary);
        assert_eq!(
            primary.top_left,
            tokens::component::primary_tab::ACTIVE_INDICATOR_SHAPE_TOP
        );
        assert_eq!(primary.bottom_left, 0.0);
        assert_eq!(
            indicator_radius(Variant::Secondary),
            Radius::new(tokens::component::secondary_tab::ACTIVE_INDICATOR_SHAPE)
        );
    }

    #[test]
    fn inactive_indicator_is_transparent_but_keeps_height() {
        let theme = Theme::Light;
        let style = indicator_style(&theme, Variant::Primary, false);

        assert_eq!(
            style.background,
            Some(Background::Color(Color::TRANSPARENT))
        );
        assert_eq!(
            Variant::Primary.indicator_height(),
            tokens::component::primary_tab::ACTIVE_INDICATOR_HEIGHT
        );
    }

    #[test]
    fn tab_state_animates_indicator_with_material_timing() {
        let now = Instant::now();
        let mut state = State::new(0);

        state.select(2, now, Variant::Primary);

        assert_eq!(state.selected_index(), 2);
        assert_eq!(state.indicator_position.to, 2.0);

        let _ = state.advance(now + duration_ms(125));
        assert!(state.indicator_position() > 0.0);
        assert!(state.indicator_position() < 2.0);

        let _ = state.advance(now + duration_ms(250));
        assert_eq!(state.indicator_position(), 2.0);
    }

    #[test]
    fn moving_indicator_width_matches_primary_inset_and_secondary_full_width() {
        assert_eq!(
            moving_indicator_width(Variant::Primary, 120.0),
            120.0 - tokens::component::primary_tab::HORIZONTAL_SPACE * 2.0
        );
        assert_eq!(moving_indicator_width(Variant::Secondary, 120.0), 120.0);
    }
}
