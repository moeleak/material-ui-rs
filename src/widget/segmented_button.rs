//! Material 3 outlined segmented button constructors.

use iced_widget::button::{Status, Style};
use iced_widget::core::border::Radius;
use iced_widget::core::text as core_text;
use iced_widget::core::time::Instant;
use iced_widget::core::{Background, Border, Color, Element, Length, Padding, alignment};
use iced_widget::graphics::geometry;
use iced_widget::text;
use iced_widget::{Container, Row, Text};

use super::absolute_line_height;
use super::button::Button;
use super::support::{AnimatedScalar, duration_ms};
use crate::utils::{mix, state_layer};
use crate::{Theme, fonts, tokens};

/// Animated segmented button selection state.
#[derive(Debug, Clone)]
pub struct State {
    selected_index: usize,
    previous_index: usize,
    progress: AnimatedScalar,
}

impl State {
    /// Creates segmented button selection state with the initial selected index.
    pub fn new(selected_index: usize) -> Self {
        Self {
            selected_index,
            previous_index: selected_index,
            progress: AnimatedScalar::new(1.0),
        }
    }

    /// Returns the selected segment index.
    pub const fn selected_index(&self) -> usize {
        self.selected_index
    }

    /// Starts the Material selection transition to `selected_index`.
    pub fn select(&mut self, selected_index: usize, now: Instant) {
        if self.selected_index == selected_index {
            return;
        }

        self.previous_index = self.selected_index;
        self.selected_index = selected_index;
        self.progress = AnimatedScalar::new(0.0);
        self.progress.set_target(
            1.0,
            now,
            duration_ms(tokens::component::segmented_button::SELECT_TRANSITION_DURATION_MS),
            tokens::component::segmented_button::SELECT_TRANSITION_EASING,
        );
    }

    /// Advances the running transition.
    pub fn advance(&mut self, now: Instant) -> bool {
        self.progress.advance(now)
    }

    /// Returns whether a transition is still running.
    pub fn is_animating(&self) -> bool {
        self.progress.is_animating()
    }

    /// Returns this segment's selected-state visual progress.
    pub fn progress_for(&self, index: usize) -> f32 {
        let progress = self.progress.value.clamp(0.0, 1.0);

        if index == self.selected_index {
            progress
        } else if index == self.previous_index {
            1.0 - progress
        } else {
            0.0
        }
    }
}

/// A segment's visual position inside a segmented button set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SegmentPosition {
    Only,
    First,
    Middle,
    Last,
}

impl SegmentPosition {
    /// Returns the segment position for an item in a segmented button set.
    pub fn for_index(index: usize, len: usize) -> Self {
        if len <= 1 {
            Self::Only
        } else if index == 0 {
            Self::First
        } else if index + 1 == len {
            Self::Last
        } else {
            Self::Middle
        }
    }

    fn radius(self) -> Radius {
        let full = tokens::component::segmented_button::CONTAINER_SHAPE;

        match self {
            Self::Only => Radius::new(full),
            Self::First => Radius {
                top_left: full,
                top_right: 0.0,
                bottom_right: 0.0,
                bottom_left: full,
            },
            Self::Middle => Radius::default(),
            Self::Last => Radius {
                top_left: 0.0,
                top_right: full,
                bottom_right: full,
                bottom_left: 0.0,
            },
        }
    }
}

fn segment_overlap_spacing() -> f32 {
    -tokens::component::segmented_button::OUTLINE_WIDTH
}

/// Creates a row that holds segmented buttons with overlapping outlines.
pub fn group<'a, Message, Renderer>(
    segments: impl IntoIterator<Item = Element<'a, Message, Theme, Renderer>>,
) -> Row<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + 'a,
{
    Row::with_children(segments.into_iter())
        .spacing(segment_overlap_spacing())
        .align_y(alignment::Vertical::Center)
}

/// Creates a label-only outlined segment.
pub fn label<'a, Message, Renderer>(
    label: impl text::IntoFragment<'a>,
    selected: bool,
    position: SegmentPosition,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
{
    segment_button(
        Text::new(label)
            .size(tokens::component::segmented_button::LABEL_TEXT.size)
            .line_height(absolute_line_height(
                tokens::component::segmented_button::LABEL_TEXT.line_height,
            ))
            .into(),
        selected,
        position,
    )
}

/// Creates a label segment that shows the Material selected check icon only when selected.
pub fn selectable_label<'a, Message, Renderer>(
    label_text: impl text::IntoFragment<'a>,
    selected: bool,
    position: SegmentPosition,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    animated_selectable_label(label_text, selected.then_some(1.0).unwrap_or(0.0), position)
}

/// Creates a label segment with animated selected check icon and fill progress.
pub fn animated_selectable_label<'a, Message, Renderer>(
    label: impl text::IntoFragment<'a>,
    selected_progress: f32,
    position: SegmentPosition,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    leading_icon_progress("check", label, selected_progress, position)
}

/// Creates an animated label segment with an action message.
pub fn animated_selectable_label_action<'a, Message, Renderer>(
    label: impl text::IntoFragment<'a>,
    selected_progress: f32,
    position: SegmentPosition,
    on_press: Message,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    animated_selectable_label(label, selected_progress, position)
        .on_press(on_press)
        .into()
}

/// Creates animated label segments from the given selection state and actions.
pub fn animated_selectable_label_actions<'a, Message, Renderer, Label>(
    state: &State,
    segments: impl IntoIterator<Item = (Label, Message)>,
) -> Vec<Element<'a, Message, Theme, Renderer>>
where
    Label: text::IntoFragment<'a>,
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    let segments: Vec<_> = segments.into_iter().collect();
    let len = segments.len();

    segments
        .into_iter()
        .enumerate()
        .map(|(index, (label, on_press))| {
            animated_selectable_label_action(
                label,
                state.progress_for(index),
                SegmentPosition::for_index(index, len),
                on_press,
            )
        })
        .collect()
}

/// Creates an outlined segment with a Material Symbols leading icon.
pub fn leading_icon<'a, Message, Renderer>(
    icon_name: impl text::IntoFragment<'a>,
    label: impl text::IntoFragment<'a>,
    selected: bool,
    position: SegmentPosition,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    leading_icon_progress(
        icon_name,
        label,
        selected.then_some(1.0).unwrap_or(0.0),
        position,
    )
}

fn leading_icon_progress<'a, Message, Renderer>(
    icon_name: impl text::IntoFragment<'a>,
    label: impl text::IntoFragment<'a>,
    selected_progress: f32,
    position: SegmentPosition,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    let progress = selected_progress.clamp(0.0, 1.0);
    let label_text = tokens::component::segmented_button::LABEL_TEXT;
    let icon = fonts::filled_icon(
        icon_name,
        tokens::component::segmented_button::WITH_ICON_ICON_SIZE * progress,
    )
    .width(Length::Fixed(
        tokens::component::segmented_button::WITH_ICON_ICON_SIZE * progress,
    ))
    .height(Length::Fixed(
        tokens::component::segmented_button::WITH_ICON_ICON_SIZE,
    ));

    let content = Row::<Message, Theme, Renderer>::new()
        .push(icon)
        .push(
            Text::new(label)
                .size(label_text.size)
                .line_height(absolute_line_height(label_text.line_height)),
        )
        .spacing(tokens::component::segmented_button::ICON_LABEL_SPACE * progress)
        .align_y(alignment::Vertical::Center);

    segment_button_progress(content.into(), progress, position)
}

fn segment_button<'a, Message, Renderer>(
    content: Element<'a, Message, Theme, Renderer>,
    selected: bool,
    position: SegmentPosition,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
{
    segment_button_progress(content, selected.then_some(1.0).unwrap_or(0.0), position)
}

fn segment_button_progress<'a, Message, Renderer>(
    content: Element<'a, Message, Theme, Renderer>,
    selected_progress: f32,
    position: SegmentPosition,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
{
    let progress = selected_progress.clamp(0.0, 1.0);

    Button::new(
        Container::new(content)
            .height(Length::Fixed(
                tokens::component::segmented_button::CONTAINER_HEIGHT,
            ))
            .padding(Padding {
                top: 0.0,
                right: tokens::component::segmented_button::TRAILING_SPACE,
                bottom: 0.0,
                left: tokens::component::segmented_button::LEADING_SPACE,
            })
            .align_y(alignment::Vertical::Center),
    )
    .height(Length::Fixed(
        tokens::component::segmented_button::CONTAINER_HEIGHT,
    ))
    .padding(Padding::ZERO)
    .style(move |theme, status| segmented_style_progress(theme, status, progress, position))
}

/// Returns the style for an outlined segmented button.
pub fn segmented_style(
    theme: &Theme,
    status: Status,
    selected: bool,
    position: SegmentPosition,
) -> Style {
    segmented_style_progress(
        theme,
        status,
        selected.then_some(1.0).unwrap_or(0.0),
        position,
    )
}

/// Returns the style for an outlined segmented button at a selected-state progress.
pub fn segmented_style_progress(
    theme: &Theme,
    status: Status,
    selected_progress: f32,
    position: SegmentPosition,
) -> Style {
    let colors = theme.colors();
    let radius = position.radius();
    let progress = selected_progress.clamp(0.0, 1.0);
    let outline = colors.outline.color;
    let container = if progress > 0.0 {
        Some(Color {
            a: colors.secondary.container.a * progress,
            ..colors.secondary.container
        })
    } else {
        None
    };
    let content = mix(
        colors.surface.text,
        colors.secondary.container_text,
        progress,
    );
    let layer = mix(
        colors.surface.text,
        colors.secondary.container_text,
        progress,
    );

    let border = Border {
        color: outline,
        width: tokens::component::segmented_button::OUTLINE_WIDTH,
        radius,
    };

    let active = Style {
        background: container.map(Background::Color),
        text_color: content,
        border,
        shadow: Default::default(),
        snap: cfg!(feature = "crisp"),
    };

    match status {
        Status::Active => active,
        Status::Hovered => Style {
            background: Some(Background::Color(state_background(
                container,
                layer,
                tokens::component::segmented_button::HOVER_STATE_LAYER_OPACITY,
            ))),
            ..active
        },
        Status::Pressed => Style {
            background: Some(Background::Color(state_background(
                container,
                layer,
                tokens::component::segmented_button::PRESSED_STATE_LAYER_OPACITY,
            ))),
            ..active
        },
        Status::Disabled => Style {
            background: (progress > 0.0).then_some(Background::Color(Color {
                a: tokens::state::DISABLED_CONTAINER_OPACITY,
                ..colors.surface.text
            })),
            text_color: Color {
                a: tokens::component::segmented_button::DISABLED_LABEL_TEXT_OPACITY,
                ..colors.surface.text
            },
            border: Border {
                color: Color {
                    a: tokens::component::segmented_button::DISABLED_OUTLINE_OPACITY,
                    ..colors.surface.text
                },
                ..border
            },
            ..active
        },
    }
}

fn state_background(container: Option<Color>, layer: Color, opacity: f32) -> Color {
    container.map_or_else(
        || state_layer(layer, opacity),
        |color| mix(color, layer, opacity),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn segment_position_sets_outer_radii_only() {
        let full = tokens::component::segmented_button::CONTAINER_SHAPE;

        assert_eq!(SegmentPosition::for_index(0, 1), SegmentPosition::Only);
        assert_eq!(SegmentPosition::for_index(0, 3), SegmentPosition::First);
        assert_eq!(SegmentPosition::for_index(1, 3), SegmentPosition::Middle);
        assert_eq!(SegmentPosition::for_index(2, 3), SegmentPosition::Last);
        assert_eq!(SegmentPosition::Only.radius(), Radius::new(full));
        assert_eq!(SegmentPosition::First.radius().top_left, full);
        assert_eq!(SegmentPosition::First.radius().top_right, 0.0);
        assert_eq!(SegmentPosition::Middle.radius(), Radius::default());
        assert_eq!(SegmentPosition::Last.radius().top_right, full);
        assert_eq!(SegmentPosition::Last.radius().bottom_left, 0.0);
    }

    #[test]
    fn group_overlaps_adjacent_outlines_by_border_width() {
        assert_eq!(
            segment_overlap_spacing(),
            -tokens::component::segmented_button::OUTLINE_WIDTH
        );
    }

    #[test]
    fn selected_segment_uses_secondary_container_tokens() {
        let theme = Theme::Light;
        let colors = theme.colors();
        let style = segmented_style(&theme, Status::Active, true, SegmentPosition::Only);

        assert_eq!(
            style.background,
            Some(Background::Color(colors.secondary.container))
        );
        assert_eq!(style.text_color, colors.secondary.container_text);
        assert_eq!(
            style.border.width,
            tokens::component::segmented_button::OUTLINE_WIDTH
        );
    }

    #[test]
    fn selection_state_crossfades_previous_and_selected_segments() {
        let now = Instant::now();
        let mut state = State::new(0);

        state.select(2, now);

        assert_eq!(state.selected_index(), 2);
        assert_eq!(state.progress_for(0), 1.0);
        assert_eq!(state.progress_for(2), 0.0);

        let _ = state.advance(now + duration_ms(100));
        assert!(state.progress_for(0) < 1.0);
        assert!(state.progress_for(2) > 0.0);
    }

    #[test]
    fn segmented_style_progress_interpolates_selected_fill_and_text() {
        let theme = Theme::Light;
        let colors = theme.colors();
        let style = segmented_style_progress(&theme, Status::Active, 0.5, SegmentPosition::Only);

        assert_eq!(
            style.background,
            Some(Background::Color(Color {
                a: colors.secondary.container.a * 0.5,
                ..colors.secondary.container
            }))
        );
        assert_eq!(
            style.text_color,
            mix(colors.surface.text, colors.secondary.container_text, 0.5)
        );
    }
}
