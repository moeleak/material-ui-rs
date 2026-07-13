//! Selectable, copyable log viewer building blocks.

use iced_widget::button::{Status as ButtonStatus, Style as ButtonStyle};
use iced_widget::checkbox::{Status as CheckboxStatus, Style as CheckboxStyle};
use iced_widget::core::svg as core_svg;
use iced_widget::core::text as core_text;
use iced_widget::core::time::Instant;
use iced_widget::core::widget;
use iced_widget::core::{
    Background, Border, Element, Font, Length, Padding, Shadow, alignment, border,
};
use iced_widget::graphics::geometry;
use iced_widget::renderer::wgpu::primitive;
use iced_widget::text::{self, LineHeight};
use iced_widget::{Column, Container, Row, Scrollable, Stack, Text, opaque};

use super::app_bar;
use super::button::Button;
use super::support::{AnimatedScalar, alpha_color, duration_ms};
use crate::style::{button as button_style, checkbox as checkbox_style};
use crate::{Theme, text as text_style, tokens};

/// Severity attached to a structured log entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    /// Returns the uppercase label rendered and copied for this level.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Trace => "TRACE",
            Self::Debug => "DEBUG",
            Self::Info => "INFO",
            Self::Warn => "WARN",
            Self::Error => "ERROR",
        }
    }
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

/// A structured item displayed by the log viewer.
///
/// `message` is appended directly to the level label so callers can preserve
/// source formatting such as `INFO[0005] ...`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogEntry<Id> {
    id: Id,
    level: LogLevel,
    line: String,
}

impl<Id> LogEntry<Id> {
    /// Creates a log entry with a stable caller-provided identifier.
    pub fn new(id: Id, level: LogLevel, message: impl Into<String>) -> Self {
        let message = message.into();

        Self {
            id,
            level,
            line: format!("{level}{message}"),
        }
    }

    /// Returns the stable identifier used for selection.
    pub const fn id(&self) -> &Id {
        &self.id
    }

    /// Returns the log severity.
    pub const fn level(&self) -> LogLevel {
        self.level
    }

    /// Returns the source-formatted text following the severity label.
    pub fn message(&self) -> &str {
        &self.line[self.level.label().len()..]
    }

    /// Returns the complete line shown by the viewer and written on copy.
    pub fn line(&self) -> &str {
        &self.line
    }
}

/// User actions emitted by [`view`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action<Id> {
    /// Toggles one entry's selected state.
    Toggle(Id),
    /// Leaves selection mode and clears all selected entries.
    CloseSelection,
    /// Copies all selected entries in their visible order.
    CopySelection,
}

/// Selection and scroll identity for a log viewer.
#[derive(Debug)]
pub struct State<Id> {
    selected: Vec<Id>,
    scrollable_id: widget::Id,
    selection_bar_visibility: AnimatedScalar,
    selection_bar_count: usize,
}

impl<Id> Default for State<Id> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Id> State<Id> {
    /// Creates an empty viewer state with an independent scroll identity.
    pub fn new() -> Self {
        Self {
            selected: Vec::new(),
            scrollable_id: widget::Id::unique(),
            selection_bar_visibility: AnimatedScalar::new(0.0),
            selection_bar_count: 0,
        }
    }

    /// Returns the selected identifiers in selection order.
    pub fn selected_ids(&self) -> &[Id] {
        &self.selected
    }

    /// Clears the current selection.
    pub fn clear_selection(&mut self) {
        self.clear_selection_at(Instant::now());
    }

    /// Advances the contextual selection bar animation.
    ///
    /// Call this from window frame events while [`Self::is_animating`] is true.
    pub fn advance(&mut self, now: Instant) -> bool {
        let animating = self.selection_bar_visibility.advance(now);

        if !animating
            && self.selection_bar_visibility.value <= f32::EPSILON
            && self.selected.is_empty()
        {
            self.selection_bar_count = 0;
        }

        animating
    }

    /// Returns whether the contextual selection bar is animating.
    pub fn is_animating(&self) -> bool {
        self.selection_bar_visibility.is_animating()
    }

    /// Returns the contextual selection bar visibility progress.
    pub fn selection_bar_progress(&self) -> f32 {
        self.selection_bar_visibility.value.clamp(0.0, 1.0)
    }

    fn clear_selection_at(&mut self, now: Instant) {
        self.selected.clear();
        self.sync_selection_bar(now);
    }

    fn sync_selection_bar(&mut self, now: Instant) {
        let count = self.selected.len();

        if count > 0 {
            self.selection_bar_count = count;
            self.selection_bar_visibility.set_target(
                1.0,
                now,
                duration_ms(tokens::component::log_viewer::SELECTION_BAR_ENTER_DURATION_MS),
                tokens::component::log_viewer::SELECTION_BAR_ENTER_EASING,
            );
        } else {
            self.selection_bar_visibility.set_target(
                0.0,
                now,
                duration_ms(tokens::component::log_viewer::SELECTION_BAR_EXIT_DURATION_MS),
                tokens::component::log_viewer::SELECTION_BAR_EXIT_EASING,
            );
        }
    }
}

impl<Id: Eq> State<Id> {
    /// Returns whether an identifier is currently selected.
    pub fn is_selected(&self, id: &Id) -> bool {
        self.selected.iter().any(|selected| selected == id)
    }

    /// Toggles an identifier and returns its new selected state.
    pub fn toggle(&mut self, id: Id) -> bool {
        self.toggle_at(id, Instant::now())
    }

    fn toggle_at(&mut self, id: Id, now: Instant) -> bool {
        if let Some(index) = self.selected.iter().position(|selected| selected == &id) {
            let _ = self.selected.remove(index);
            self.sync_selection_bar(now);
            false
        } else {
            self.selected.push(id);
            self.sync_selection_bar(now);
            true
        }
    }

    /// Drops selections whose entries are no longer present.
    pub fn retain_entries(&mut self, entries: &[LogEntry<Id>]) {
        self.selected
            .retain(|selected| entries.iter().any(|entry| entry.id() == selected));
        self.sync_selection_bar(Instant::now());
    }

    /// Counts selected entries that are still present in the supplied list.
    pub fn selected_count(&self, entries: &[LogEntry<Id>]) -> usize {
        entries
            .iter()
            .filter(|entry| self.is_selected(entry.id()))
            .count()
    }

    /// Builds clipboard text in the current visible entry order.
    pub fn selected_text(&self, entries: &[LogEntry<Id>]) -> String {
        entries
            .iter()
            .filter(|entry| self.is_selected(entry.id()))
            .map(LogEntry::line)
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Applies a viewer action, including clipboard writes for copy actions.
    pub fn update<Message>(
        &mut self,
        action: Action<Id>,
        entries: &[LogEntry<Id>],
    ) -> iced::Task<Message> {
        match action {
            Action::Toggle(id) => {
                let _ = self.toggle(id);
                iced::Task::none()
            }
            Action::CloseSelection => {
                self.clear_selection();
                iced::Task::none()
            }
            Action::CopySelection => {
                let selected = self.selected_text(entries);

                if selected.is_empty() {
                    iced::Task::none()
                } else {
                    iced::clipboard::write(selected)
                }
            }
        }
    }
}

/// Builds a selectable log list and its contextual selection app bar.
///
/// Page titles are intentionally outside this component. Compose this viewer
/// below an existing top app bar such as [`super::app_bar::large`].
/// Advance [`State`] from frame events to animate the contextual bar.
pub fn view<'a, Id, Message, Renderer>(
    entries: &'a [LogEntry<Id>],
    state: &'a State<Id>,
    on_action: impl Fn(Action<Id>) -> Message,
) -> Column<'a, Message, Theme, Renderer>
where
    Id: Clone + Eq + 'a,
    Message: Clone + 'a,
    Renderer: iced_widget::core::Renderer
        + geometry::Renderer
        + primitive::Renderer
        + core_text::Renderer
        + core_svg::Renderer
        + 'a,
    Font: Into<Renderer::Font>,
{
    let selected_count = state.selected_count(entries);

    let items = Container::new(
        Column::with_children(
            entries
                .iter()
                .map(|entry| item(entry, state, on_action(Action::Toggle(entry.id().clone())))),
        )
        .spacing(tokens::component::log_viewer::ITEM_SPACING)
        .width(Length::Fill),
    )
    .padding(Padding {
        top: 0.0,
        right: tokens::component::log_viewer::LIST_HORIZONTAL_SPACE,
        bottom: 0.0,
        left: tokens::component::log_viewer::LIST_HORIZONTAL_SPACE,
    })
    .width(Length::Fill);

    let logs: Element<'a, Message, Theme, Renderer> = Scrollable::new(items)
        .id(state.scrollable_id.clone())
        .anchor_bottom()
        .width(Length::Fill)
        .height(Length::Fill)
        .into();
    let progress = state.selection_bar_progress();
    let mut layers = Stack::new()
        .push(logs)
        .width(Length::Fill)
        .height(Length::Fill);

    if selected_count > 0 || progress > f32::EPSILON {
        let count = if selected_count > 0 {
            selected_count
        } else {
            state.selection_bar_count
        };
        let bar = selection_bar(
            count,
            progress,
            on_action(Action::CloseSelection),
            on_action(Action::CopySelection),
        );
        layers = layers.push(opaque(bar));
    }

    Column::new()
        .push(layers)
        .width(Length::Fill)
        .height(Length::Fill)
}

fn selection_bar<'a, Message, Renderer>(
    selected_count: usize,
    progress: f32,
    close: Message,
    copy: Message,
) -> Container<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: iced_widget::core::Renderer
        + geometry::Renderer
        + primitive::Renderer
        + core_text::Renderer
        + 'a,
    Font: Into<Renderer::Font>,
{
    let progress = progress.clamp(0.0, 1.0);
    let title_text = tokens::component::app_bar::SMALL_TITLE_TEXT;
    let close = selection_icon_button("close", close, progress);
    let copy = selection_icon_button("content_copy", copy, progress);
    let content = Row::new()
        .push(close)
        .push(
            Text::new(format!("{selected_count} selected"))
                .size(title_text.size)
                .line_height(LineHeight::Absolute(title_text.line_height.into()))
                .width(Length::Fill)
                .style(move |theme: &Theme| iced_widget::text::Style {
                    color: Some(alpha_color(theme.colors().surface.text, progress)),
                }),
        )
        .push(copy)
        .spacing(tokens::component::app_bar::ICON_BUTTON_SPACE)
        .padding(Padding {
            top: 0.0,
            right: tokens::component::app_bar::TRAILING_SPACE,
            bottom: 0.0,
            left: tokens::component::app_bar::LEADING_SPACE,
        })
        .align_y(alignment::Vertical::Center)
        .width(Length::Fill);

    Container::new(content)
        .width(Length::Fill)
        .height(Length::Fixed(
            tokens::component::log_viewer::SELECTION_BAR_HEIGHT,
        ))
        .align_y(alignment::Vertical::Center)
        .style(move |theme| selection_bar_style(theme, progress))
}

fn selection_icon_button<'a, Message, Renderer>(
    icon: &'static str,
    on_press: Message,
    progress: f32,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'a,
    Font: Into<Renderer::Font>,
{
    app_bar::icon_button(icon)
        .style(move |theme, status| {
            let mut style = button_style::icon(theme, status);
            style.text_color = alpha_color(style.text_color, progress);
            style.background = style.background.map(|background| match background {
                Background::Color(color) => Background::Color(alpha_color(color, progress)),
                Background::Gradient(gradient) => Background::Gradient(gradient),
            });
            style
        })
        .on_press(on_press)
        .into()
}

fn selection_bar_style(theme: &Theme, progress: f32) -> iced_widget::container::Style {
    let colors = theme.colors();

    iced_widget::container::Style {
        background: Some(Background::Color(alpha_color(
            colors.surface.container.base,
            progress,
        ))),
        text_color: Some(alpha_color(colors.surface.text, progress)),
        border: border::rounded(tokens::component::app_bar::CONTAINER_SHAPE),
        snap: cfg!(feature = "crisp"),
        ..iced_widget::container::Style::default()
    }
}

fn item<'a, Id, Message, Renderer>(
    entry: &'a LogEntry<Id>,
    state: &State<Id>,
    toggle: Message,
) -> Element<'a, Message, Theme, Renderer>
where
    Id: Eq,
    Message: Clone + 'a,
    Renderer: iced_widget::core::Renderer
        + geometry::Renderer
        + primitive::Renderer
        + core_text::Renderer
        + core_svg::Renderer
        + 'a,
    Font: Into<Renderer::Font>,
{
    let selected = state.is_selected(entry.id());
    let checkbox = super::checkbox::control(selected)
        .style(move |theme, _status| checkbox_visual_style(theme, selected));
    let checkbox_button = Button::new(
        Container::new(checkbox)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center),
    )
    .width(Length::Fixed(
        tokens::component::log_viewer::CHECKBOX_SLOT_WIDTH,
    ))
    .height(Length::Fixed(
        tokens::component::log_viewer::ITEM_MIN_HEIGHT,
    ))
    .padding(Padding::ZERO)
    .style(item_button_style)
    .on_press(toggle.clone());

    let scale = tokens::component::log_viewer::LOG_TEXT;
    let line = Text::new(entry.line())
        .size(scale.size)
        .line_height(LineHeight::Absolute(scale.line_height.into()))
        .font(Font::MONOSPACE)
        .wrapping(text::Wrapping::WordOrGlyph)
        .width(Length::Fill)
        .style(text_style::surface);
    let level = entry.level();
    let colored_level = Text::new(level.label())
        .size(scale.size)
        .line_height(LineHeight::Absolute(scale.line_height.into()))
        .font(Font::MONOSPACE)
        .style(move |theme| level_text_style(theme, level));
    let log_text = Stack::new()
        .push(line)
        .push(colored_level)
        .width(Length::Fill);
    let text_button = Button::new(log_text)
        .width(Length::Fill)
        .padding(Padding {
            top: tokens::component::log_viewer::ITEM_VERTICAL_SPACE,
            right: tokens::component::log_viewer::ITEM_TRAILING_SPACE,
            bottom: tokens::component::log_viewer::ITEM_VERTICAL_SPACE,
            left: 0.0,
        })
        .style(item_button_style)
        .on_press(toggle);

    Container::new(
        Row::new()
            .push(checkbox_button)
            .push(text_button)
            .align_y(alignment::Vertical::Center)
            .width(Length::Fill),
    )
    .width(Length::Fill)
    .style(move |theme| item_container_style(theme, selected))
    .into()
}

fn checkbox_visual_style(theme: &Theme, selected: bool) -> CheckboxStyle {
    checkbox_style::default(
        theme,
        CheckboxStatus::Active {
            is_checked: selected,
        },
    )
}

fn item_button_style(theme: &Theme, _status: ButtonStatus) -> ButtonStyle {
    ButtonStyle {
        background: None,
        text_color: theme.colors().surface.text,
        border: Border::default(),
        shadow: Shadow::default(),
        snap: cfg!(feature = "crisp"),
    }
}

fn item_container_style(theme: &Theme, selected: bool) -> iced_widget::container::Style {
    let colors = theme.colors();

    iced_widget::container::Style {
        background: Some(Background::Color(colors.surface.container.low)),
        text_color: Some(colors.surface.text),
        border: if selected {
            Border {
                color: colors.outline.color,
                width: tokens::component::log_viewer::SELECTED_OUTLINE_WIDTH,
                radius: tokens::component::log_viewer::ITEM_SHAPE.into(),
            }
        } else {
            border::rounded(tokens::component::log_viewer::ITEM_SHAPE)
        },
        snap: cfg!(feature = "crisp"),
        ..iced_widget::container::Style::default()
    }
}

fn level_text_style(theme: &Theme, level: LogLevel) -> iced_widget::text::Style {
    let colors = theme.colors();
    let color = match level {
        LogLevel::Trace => colors.outline.color,
        LogLevel::Debug => colors.secondary.color,
        LogLevel::Info => colors.primary.color,
        LogLevel::Warn => colors.tertiary.color,
        LogLevel::Error => colors.error.color,
    };

    iced_widget::text::Style { color: Some(color) }
}

#[cfg(test)]
#[path = "../../../tests/widget/component/log_viewer.rs"]
mod tests;
