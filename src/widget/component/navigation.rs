//! Material 3 navigation bar, rail, drawer, and adaptive layout helpers.

use std::f32::consts::PI;

use iced_widget::canvas::{self, Canvas, LineCap, Path, Stroke};
use iced_widget::core::layout;
use iced_widget::core::mouse;
use iced_widget::core::overlay;
use iced_widget::core::renderer;
use iced_widget::core::text as core_text;
use iced_widget::core::time::Instant;
use iced_widget::core::touch;
use iced_widget::core::widget::Operation;
use iced_widget::core::widget::tree::{self, Tree};
use iced_widget::core::{
    Background, Clipboard, Color, Element, Event, Font, Layout, Length, Padding, Point, Rectangle,
    Shell, Size, Vector, Widget, alignment, border, window,
};
use iced_widget::graphics::geometry;
use iced_widget::renderer::wgpu::primitive;
use iced_widget::text::{self, LineHeight};
use iced_widget::{Column, Container, Row, Space, Stack, Text};

use super::badge as badge_widget;
use super::button::Button;
use super::ripple::{PressRippleState, RippleConfig, RippleStart, RippleStyle, draw_ripples};
use super::support::{AnimatedScalar, alpha_color, duration_ms, lerp};
use crate::style::button as button_style;
use crate::utils::{HOVERED_LAYER_OPACITY, mix, shadow_from_level, state_layer};
use crate::{Theme, fonts, tokens};

#[cfg(test)]
use super::ripple::{ripple_target_radius, rounded_rect_span_at_y};

const NAVIGATION_MENU_ICON_VIEWPORT_SIZE: f32 = 24.0;
const NAVIGATION_MENU_ICON_START_X: f32 = 5.0;
const NAVIGATION_MENU_ICON_END_X: f32 = 19.0;
const NAVIGATION_MENU_ICON_CENTER_X: f32 = 12.0;
const NAVIGATION_MENU_ICON_TOP_Y: f32 = 7.0;
const NAVIGATION_MENU_ICON_CENTER_Y: f32 = 12.0;
const NAVIGATION_MENU_ICON_BOTTOM_Y: f32 = 17.0;
const NAVIGATION_MENU_ICON_ARROW_TOP_Y: f32 = 5.0;
const NAVIGATION_MENU_ICON_ARROW_BOTTOM_Y: f32 = 19.0;
const NAVIGATION_MENU_ICON_STROKE_WIDTH: f32 = 2.4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdaptiveLayout {
    NavigationBar,
    NavigationRail,
}

impl AdaptiveLayout {
    pub fn from_size(width: f32, height: f32) -> Self {
        adaptive_layout(width, height)
    }

    pub fn item_animation_duration_ms(self) -> u16 {
        match self {
            Self::NavigationBar => tokens::component::navigation_bar::ITEM_ANIMATION_DURATION_MS,
            Self::NavigationRail => tokens::component::navigation_rail::ITEM_ANIMATION_DURATION_MS,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowWidthClass {
    Compact,
    Medium,
    Expanded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowHeightClass {
    Compact,
    Medium,
    Expanded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowSizeClass {
    pub width: WindowWidthClass,
    pub height: WindowHeightClass,
}

impl WindowSizeClass {
    pub fn from_size(width: f32, height: f32) -> Self {
        Self {
            width: width_class(width),
            height: height_class(height),
        }
    }

    pub fn adaptive_navigation_layout(self) -> AdaptiveLayout {
        if matches!(self.width, WindowWidthClass::Compact)
            || matches!(self.height, WindowHeightClass::Compact)
        {
            AdaptiveLayout::NavigationBar
        } else {
            AdaptiveLayout::NavigationRail
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Selection<Id> {
    selected: Id,
    previous: Option<Id>,
    selected_size_start: f32,
    previous_size_start: f32,
    size_progress: f32,
    selected_alpha_start: f32,
    previous_alpha_start: f32,
    alpha_progress: f32,
}

impl<Id: Copy + Eq> Selection<Id> {
    pub fn new(selected: Id) -> Self {
        Self {
            selected,
            previous: None,
            selected_size_start: 1.0,
            previous_size_start: 0.0,
            size_progress: 1.0,
            selected_alpha_start: 1.0,
            previous_alpha_start: 0.0,
            alpha_progress: 1.0,
        }
    }

    pub fn transitioning(selected: Id, previous: Id, progress: f32) -> Self {
        Self::transitioning_from(selected, previous, 0.0, 1.0, progress)
    }

    pub fn transitioning_from(
        selected: Id,
        previous: Id,
        selected_start: f32,
        previous_start: f32,
        progress: f32,
    ) -> Self {
        Self::transitioning_from_tracks(
            selected,
            previous,
            TrackProgress::new(selected_start, previous_start, progress),
            TrackProgress::new(selected_start, previous_start, progress),
        )
    }

    fn transitioning_from_tracks(
        selected: Id,
        previous: Id,
        size: TrackProgress,
        alpha: TrackProgress,
    ) -> Self {
        Self {
            selected,
            previous: Some(previous),
            selected_size_start: size.selected_start,
            previous_size_start: size.previous_start,
            size_progress: size.progress,
            selected_alpha_start: alpha.selected_start,
            previous_alpha_start: alpha.previous_start,
            alpha_progress: alpha.progress,
        }
    }

    pub fn selected(self) -> Id {
        self.selected
    }

    pub fn progress(self, id: Id) -> f32 {
        self.size_progress(id)
    }

    pub fn size_progress(self, id: Id) -> f32 {
        if id == self.selected {
            lerp(self.selected_size_start, 1.0, self.size_progress)
        } else if self.previous.is_some_and(|previous| previous == id) {
            lerp(self.previous_size_start, 0.0, self.size_progress)
        } else {
            0.0
        }
    }

    pub fn alpha_progress(self, id: Id) -> f32 {
        if id == self.selected {
            lerp(self.selected_alpha_start, 1.0, self.alpha_progress)
        } else if self.previous.is_some_and(|previous| previous == id) {
            lerp(self.previous_alpha_start, 0.0, self.alpha_progress)
        } else {
            0.0
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct NavigationState<Id> {
    selected: Id,
    previous: Option<Id>,
    selected_size_start: f32,
    previous_size_start: f32,
    selected_alpha_start: f32,
    previous_alpha_start: f32,
    size_progress: AnimatedScalar,
    alpha_progress: AnimatedScalar,
    rail_expansion: NavigationRailExpansionState,
}

impl<Id: Copy + Eq> NavigationState<Id> {
    pub fn new(selected: Id) -> Self {
        Self {
            selected,
            previous: None,
            selected_size_start: 1.0,
            previous_size_start: 0.0,
            selected_alpha_start: 1.0,
            previous_alpha_start: 0.0,
            size_progress: AnimatedScalar::new(1.0),
            alpha_progress: AnimatedScalar::new(1.0),
            rail_expansion: NavigationRailExpansionState::new(false),
        }
    }

    pub fn selected(&self) -> Id {
        self.selected
    }

    pub fn selection(&self) -> Selection<Id> {
        if let Some(previous) = self.previous {
            Selection::transitioning_from_tracks(
                self.selected,
                previous,
                TrackProgress::new(
                    self.selected_size_start,
                    self.previous_size_start,
                    self.size_progress.value,
                ),
                TrackProgress::new(
                    self.selected_alpha_start,
                    self.previous_alpha_start,
                    self.alpha_progress.value,
                ),
            )
        } else {
            Selection::new(self.selected)
        }
    }

    pub fn select(&mut self, selected: Id, now: Instant, layout: AdaptiveLayout) {
        if selected == self.selected {
            return;
        }

        let current = self.selection();
        let previous = self.selected;
        let selected_size_start = current.size_progress(selected);
        let previous_size_start = current.size_progress(previous);
        let selected_alpha_start = current.alpha_progress(selected);
        let previous_alpha_start = current.alpha_progress(previous);

        self.selected = selected;
        self.previous = Some(previous);
        self.selected_size_start = selected_size_start;
        self.previous_size_start = previous_size_start;
        self.selected_alpha_start = selected_alpha_start;
        self.previous_alpha_start = previous_alpha_start;
        self.size_progress = AnimatedScalar::new(0.0);
        self.alpha_progress = AnimatedScalar::new(0.0);
        let duration = duration_ms(layout.item_animation_duration_ms());
        self.size_progress
            .set_target(1.0, now, duration, tokens::motion::EASING_LEGACY);
        self.alpha_progress
            .set_target(1.0, now, duration, tokens::motion::EASING_LEGACY);
    }

    pub fn select_for_size(&mut self, selected: Id, now: Instant, size: Size) {
        self.select(selected, now, adaptive_layout(size.width, size.height));
    }

    pub fn select_now_for_size(&mut self, selected: Id, size: Size) {
        self.select_for_size(selected, Instant::now(), size);
    }

    pub fn toggle_menu(&mut self, now: Instant) {
        self.rail_expansion.toggle(now);
    }

    pub fn toggle_menu_now(&mut self) {
        self.toggle_menu(Instant::now());
    }

    pub fn is_menu_open(&self) -> bool {
        self.rail_expansion.is_open()
    }

    pub fn is_menu_visible(&self) -> bool {
        self.rail_expansion.is_visible()
    }

    pub fn menu_progress(&self) -> f32 {
        self.rail_expansion.progress()
    }

    pub fn is_animating(&self) -> bool {
        self.previous.is_some() || self.rail_expansion.is_animating()
    }

    pub fn subscription<Message, F>(&self, on_frame: F) -> iced::Subscription<Message>
    where
        Message: 'static,
        F: Fn(Instant) -> Message + Send + Clone + 'static,
    {
        if self.is_animating() {
            iced::window::frames().map(on_frame)
        } else {
            iced::Subscription::none()
        }
    }

    pub fn advance(&mut self, now: Instant) -> bool {
        let navigation_animating =
            self.size_progress.advance(now) | self.alpha_progress.advance(now);
        let menu_animating = self.rail_expansion.advance(now);

        if !navigation_animating {
            self.size_progress.value = 1.0;
            self.alpha_progress.value = 1.0;
            self.previous = None;
            self.selected_size_start = 1.0;
            self.previous_size_start = 0.0;
            self.selected_alpha_start = 1.0;
            self.previous_alpha_start = 0.0;
        }

        navigation_animating | menu_animating
    }

    pub fn advance_frame(&mut self, now: Instant) {
        let _ = self.advance(now);
    }
}

#[derive(Debug, Clone, Copy)]
pub struct NavigationRailExpansionState {
    open: bool,
    progress: AnimatedScalar,
}

impl NavigationRailExpansionState {
    pub fn new(open: bool) -> Self {
        Self {
            open,
            progress: AnimatedScalar::new(if open { 1.0 } else { 0.0 }),
        }
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn progress(&self) -> f32 {
        self.progress.value.clamp(0.0, 1.0)
    }

    pub fn is_visible(&self) -> bool {
        self.open || self.progress() > 0.0 || self.is_animating()
    }

    pub fn is_animating(&self) -> bool {
        (self.progress.value - self.progress.to).abs() > 0.001
    }

    pub fn open(&mut self, now: Instant) {
        self.open = true;
        self.progress
            .set_spring_target(1.0, now, rail_expansion_spring());
    }

    pub fn close(&mut self, now: Instant) {
        self.open = false;
        self.progress
            .set_spring_target(0.0, now, rail_expansion_spring());
    }

    pub fn toggle(&mut self, now: Instant) {
        if self.open {
            self.close(now);
        } else {
            self.open(now);
        }
    }

    pub fn advance(&mut self, now: Instant) -> bool {
        let animating = self.progress.advance(now);
        self.progress.value = self.progress.value.clamp(0.0, 1.0);
        animating
    }
}

fn rail_expansion_spring() -> tokens::motion::Spring {
    tokens::motion::Spring {
        damping_ratio: 1.0,
        stiffness: tokens::motion::EXPRESSIVE_FAST_SPATIAL.stiffness,
    }
}

#[derive(Debug, Clone, Copy)]
struct TrackProgress {
    selected_start: f32,
    previous_start: f32,
    progress: f32,
}

impl TrackProgress {
    fn new(selected_start: f32, previous_start: f32, progress: f32) -> Self {
        Self {
            selected_start: selected_start.clamp(0.0, 1.0),
            previous_start: previous_start.clamp(0.0, 1.0),
            progress,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Destination<Id> {
    pub id: Id,
    pub icon: &'static str,
    pub label: &'static str,
    pub badge: Option<Badge>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Badge {
    Small,
    Large(&'static str),
}

impl<Id> Destination<Id> {
    pub const fn new(id: Id, icon: &'static str, label: &'static str) -> Self {
        Self {
            id,
            icon,
            label,
            badge: None,
        }
    }

    pub const fn small_badge(mut self) -> Self {
        self.badge = Some(Badge::Small);
        self
    }

    pub const fn badge(mut self, label: &'static str) -> Self {
        self.badge = Some(Badge::Large(label));
        self
    }
}

pub fn width_class(width: f32) -> WindowWidthClass {
    if width < tokens::component::adaptive_navigation::WIDTH_COMPACT_MAX {
        WindowWidthClass::Compact
    } else if width < tokens::component::adaptive_navigation::WIDTH_MEDIUM_MAX {
        WindowWidthClass::Medium
    } else {
        WindowWidthClass::Expanded
    }
}

pub fn height_class(height: f32) -> WindowHeightClass {
    if height < tokens::component::adaptive_navigation::HEIGHT_COMPACT_MAX {
        WindowHeightClass::Compact
    } else if height < tokens::component::adaptive_navigation::HEIGHT_MEDIUM_MAX {
        WindowHeightClass::Medium
    } else {
        WindowHeightClass::Expanded
    }
}

pub fn adaptive_layout(width: f32, height: f32) -> AdaptiveLayout {
    WindowSizeClass::from_size(width, height).adaptive_navigation_layout()
}

pub fn item_animation_duration_ms(layout: AdaptiveLayout) -> u16 {
    layout.item_animation_duration_ms()
}

pub fn rail_min_height(destination_count: usize, has_header: bool) -> f32 {
    RailMetrics::min_height(destination_count, has_header)
}

/// Starts a builder for an adaptive navigation suite.
///
/// Use [`Suite::layout`] or [`Suite::window_size`] to select the adaptive
/// layout, then call [`Suite::view`] or [`Suite::with_menu`].
pub fn suite<'a, Id>(
    destinations: &'a [Destination<Id>],
    state: &'a NavigationState<Id>,
) -> Suite<'a, Id> {
    Suite::new(destinations, state)
}

/// A builder for a navigation bar/rail shell around page content.
#[derive(Debug, Clone, Copy)]
pub struct Suite<'a, Id> {
    destinations: &'a [Destination<Id>],
    state: &'a NavigationState<Id>,
    layout: AdaptiveLayout,
}

impl<'a, Id> Suite<'a, Id> {
    /// Creates a navigation suite builder.
    ///
    /// The default layout is [`AdaptiveLayout::NavigationRail`]. Call
    /// [`layout`](Self::layout), [`window_size`](Self::window_size), or
    /// [`dimensions`](Self::dimensions) for adaptive behavior.
    pub fn new(destinations: &'a [Destination<Id>], state: &'a NavigationState<Id>) -> Self {
        Self {
            destinations,
            state,
            layout: AdaptiveLayout::NavigationRail,
        }
    }

    /// Uses the provided adaptive layout.
    pub fn layout(mut self, layout: AdaptiveLayout) -> Self {
        self.layout = layout;
        self
    }

    /// Chooses the adaptive layout for the provided window size.
    pub fn window_size(mut self, size: Size) -> Self {
        self.layout = adaptive_layout(size.width, size.height);
        self
    }

    /// Chooses the adaptive layout for the provided dimensions.
    pub fn dimensions(mut self, width: f32, height: f32) -> Self {
        self.layout = adaptive_layout(width, height);
        self
    }

    /// Adds a menu button and expandable rail behavior.
    pub fn with_menu<Message>(
        self,
        headline: &'static str,
        on_menu: Message,
    ) -> SuiteWithMenu<'a, Id, Message> {
        SuiteWithMenu {
            suite: self,
            headline,
            on_menu,
        }
    }

    /// Builds the navigation suite around the provided content.
    pub fn view<Message, Renderer, F>(
        self,
        on_select: F,
        content: impl Into<Element<'a, Message, Theme, Renderer>>,
    ) -> Element<'a, Message, Theme, Renderer>
    where
        Id: Copy + Eq + 'a,
        Message: Clone + 'a,
        Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'a,
        Font: Into<Renderer::Font>,
        F: Fn(Id) -> Message + Clone + 'a,
    {
        view_for_layout(
            self.layout,
            self.destinations,
            self.state.selection(),
            on_select,
            content,
        )
    }
}

/// A navigation suite builder with menu behavior enabled.
#[derive(Debug, Clone, Copy)]
pub struct SuiteWithMenu<'a, Id, Message> {
    suite: Suite<'a, Id>,
    headline: &'static str,
    on_menu: Message,
}

impl<'a, Id, Message> SuiteWithMenu<'a, Id, Message> {
    /// Uses the provided adaptive layout.
    pub fn layout(mut self, layout: AdaptiveLayout) -> Self {
        self.suite = self.suite.layout(layout);
        self
    }

    /// Chooses the adaptive layout for the provided window size.
    pub fn window_size(mut self, size: Size) -> Self {
        self.suite = self.suite.window_size(size);
        self
    }

    /// Chooses the adaptive layout for the provided dimensions.
    pub fn dimensions(mut self, width: f32, height: f32) -> Self {
        self.suite = self.suite.dimensions(width, height);
        self
    }

    /// Builds the navigation suite around the provided content.
    pub fn view<Renderer, F>(
        self,
        on_select: F,
        content: impl Into<Element<'a, Message, Theme, Renderer>>,
    ) -> Element<'a, Message, Theme, Renderer>
    where
        Id: Copy + Eq + 'a,
        Message: Clone + 'a,
        Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'a,
        Font: Into<Renderer::Font>,
        F: Fn(Id) -> Message + Clone + 'a,
    {
        view_menu_for_layout(
            self.headline,
            self.suite.layout,
            self.suite.destinations,
            self.suite.state,
            on_select,
            self.on_menu,
            content,
        )
    }
}

pub fn view<'a, Id, Message, Renderer, F>(
    width: f32,
    height: f32,
    destinations: &'a [Destination<Id>],
    selection: Selection<Id>,
    on_select: F,
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Element<'a, Message, Theme, Renderer>
where
    Id: Copy + Eq + 'a,
    Message: Clone + 'a,
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'a,
    Font: Into<Renderer::Font>,
    F: Fn(Id) -> Message + Clone + 'a,
{
    view_for_layout(
        adaptive_layout(width, height),
        destinations,
        selection,
        on_select,
        content,
    )
}

pub fn view_for_layout<'a, Id, Message, Renderer, F>(
    layout: AdaptiveLayout,
    destinations: &'a [Destination<Id>],
    selection: Selection<Id>,
    on_select: F,
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Element<'a, Message, Theme, Renderer>
where
    Id: Copy + Eq + 'a,
    Message: Clone + 'a,
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'a,
    Font: Into<Renderer::Font>,
    F: Fn(Id) -> Message + Clone + 'a,
{
    let content = content.into();

    match layout {
        AdaptiveLayout::NavigationBar => Column::new()
            .width(Length::Fill)
            .height(Length::Fill)
            .push(content)
            .push(bar(destinations, selection, on_select))
            .into(),
        AdaptiveLayout::NavigationRail => Row::new()
            .width(Length::Fill)
            .height(Length::Fill)
            .push(rail(destinations, selection, on_select))
            .push(content)
            .into(),
    }
}

pub fn view_with_menu<'a, Id, Message, Renderer, F>(
    headline: &'static str,
    window_size: Size,
    destinations: &'a [Destination<Id>],
    state: &NavigationState<Id>,
    on_select: F,
    on_menu: Message,
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Element<'a, Message, Theme, Renderer>
where
    Id: Copy + Eq + 'a,
    Message: Clone + 'a,
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'a,
    Font: Into<Renderer::Font>,
    F: Fn(Id) -> Message + Clone + 'a,
{
    view_menu_for_layout(
        headline,
        adaptive_layout(window_size.width, window_size.height),
        destinations,
        state,
        on_select,
        on_menu,
        content,
    )
}

fn view_menu_for_layout<'a, Id, Message, Renderer, F>(
    headline: &'static str,
    layout: AdaptiveLayout,
    destinations: &'a [Destination<Id>],
    state: &NavigationState<Id>,
    on_select: F,
    on_menu: Message,
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Element<'a, Message, Theme, Renderer>
where
    Id: Copy + Eq + 'a,
    Message: Clone + 'a,
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'a,
    Font: Into<Renderer::Font>,
    F: Fn(Id) -> Message + Clone + 'a,
{
    let menu_progress = state.menu_progress();
    let content = content.into();
    let selection = state.selection();

    match layout {
        AdaptiveLayout::NavigationBar => Column::new()
            .width(Length::Fill)
            .height(Length::Fill)
            .push(content)
            .push(bar(destinations, selection, on_select))
            .into(),
        AdaptiveLayout::NavigationRail => Row::new()
            .width(Length::Fill)
            .height(Length::Fill)
            .push(if state.is_menu_visible() {
                expanded_rail_with(
                    headline,
                    destinations,
                    selection,
                    on_select,
                    on_menu,
                    ExpandedRailOptions::default().width(expanded_rail_width(menu_progress)),
                )
            } else {
                rail_with_menu_at_progress(
                    destinations,
                    selection,
                    on_select,
                    on_menu,
                    menu_progress,
                )
            })
            .push(content)
            .into(),
    }
}

pub fn bar<'a, Id, Message, Renderer, F>(
    destinations: &'a [Destination<Id>],
    selection: Selection<Id>,
    on_select: F,
) -> Container<'a, Message, Theme, Renderer>
where
    Id: Copy + Eq + 'a,
    Message: Clone + 'a,
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'a,
    Font: Into<Renderer::Font>,
    F: Fn(Id) -> Message + Clone + 'a,
{
    let mut items = Row::new()
        .width(Length::Fill)
        .height(Length::Fixed(
            tokens::component::navigation_bar::CONTAINER_HEIGHT,
        ))
        .spacing(tokens::component::navigation_bar::ITEM_HORIZONTAL_PADDING);

    for destination in destinations {
        items = items.push(
            Container::new(navigation_bar_item(
                *destination,
                selection,
                on_select.clone(),
            ))
            .width(Length::FillPortion(1)),
        );
    }

    Container::new(items)
        .width(Length::Fill)
        .height(Length::Fixed(
            tokens::component::navigation_bar::CONTAINER_HEIGHT,
        ))
        .padding(Padding {
            top: 0.0,
            right: tokens::component::navigation_bar::ITEM_HORIZONTAL_PADDING,
            bottom: 0.0,
            left: tokens::component::navigation_bar::ITEM_HORIZONTAL_PADDING,
        })
        .style(bar_container)
}

pub struct NavigationRailOptions<'a, Message, Renderer> {
    header: Option<Element<'a, Message, Theme, Renderer>>,
    fit_content: bool,
}

impl<Message, Renderer> std::fmt::Debug for NavigationRailOptions<'_, Message, Renderer> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NavigationRailOptions")
            .field("has_header", &self.header.is_some())
            .field("fit_content", &self.fit_content)
            .finish()
    }
}

impl<'a, Message, Renderer> Default for NavigationRailOptions<'a, Message, Renderer> {
    fn default() -> Self {
        Self {
            header: None,
            fit_content: false,
        }
    }
}

impl<'a, Message, Renderer> NavigationRailOptions<'a, Message, Renderer> {
    pub fn fit_content(mut self) -> Self {
        self.fit_content = true;
        self
    }

    pub fn header(mut self, header: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        self.header = Some(header.into());
        self
    }

    pub fn menu(self, on_menu: Message) -> Self
    where
        Message: Clone + 'a,
        Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'a,
        Font: Into<Renderer::Font>,
    {
        self.menu_progress(on_menu, 0.0)
    }

    fn menu_progress(mut self, on_menu: Message, progress: f32) -> Self
    where
        Message: Clone + 'a,
        Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'a,
        Font: Into<Renderer::Font>,
    {
        self.header = Some(navigation_menu_button(on_menu, progress).into());
        self
    }
}

pub fn rail<'a, Id, Message, Renderer, F>(
    destinations: &'a [Destination<Id>],
    selection: Selection<Id>,
    on_select: F,
) -> Container<'a, Message, Theme, Renderer>
where
    Id: Copy + Eq + 'a,
    Message: Clone + 'a,
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'a,
    Font: Into<Renderer::Font>,
    F: Fn(Id) -> Message + Clone + 'a,
{
    rail_with(
        destinations,
        selection,
        on_select,
        NavigationRailOptions::default(),
    )
}

pub fn rail_with<'a, Id, Message, Renderer, F>(
    destinations: &'a [Destination<Id>],
    selection: Selection<Id>,
    on_select: F,
    options: NavigationRailOptions<'a, Message, Renderer>,
) -> Container<'a, Message, Theme, Renderer>
where
    Id: Copy + Eq + 'a,
    Message: Clone + 'a,
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'a,
    Font: Into<Renderer::Font>,
    F: Fn(Id) -> Message + Clone + 'a,
{
    let has_header = options.header.is_some();
    let rail = rail_with_optional_header(destinations, selection, on_select, options.header);

    if options.fit_content {
        rail.height(Length::Fixed(rail_min_height(
            destinations.len(),
            has_header,
        )))
    } else {
        rail
    }
}

pub fn rail_with_header<'a, Id, Message, Renderer, F>(
    destinations: &'a [Destination<Id>],
    selection: Selection<Id>,
    on_select: F,
    header: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Container<'a, Message, Theme, Renderer>
where
    Id: Copy + Eq + 'a,
    Message: Clone + 'a,
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'a,
    Font: Into<Renderer::Font>,
    F: Fn(Id) -> Message + Clone + 'a,
{
    rail_with(
        destinations,
        selection,
        on_select,
        NavigationRailOptions::default().header(header),
    )
}

pub fn rail_with_menu<'a, Id, Message, Renderer, F>(
    destinations: &'a [Destination<Id>],
    selection: Selection<Id>,
    on_select: F,
    on_menu: Message,
) -> Container<'a, Message, Theme, Renderer>
where
    Id: Copy + Eq + 'a,
    Message: Clone + 'a,
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'a,
    Font: Into<Renderer::Font>,
    F: Fn(Id) -> Message + Clone + 'a,
{
    rail_with(
        destinations,
        selection,
        on_select,
        NavigationRailOptions::default().menu(on_menu),
    )
}

fn rail_with_menu_at_progress<'a, Id, Message, Renderer, F>(
    destinations: &'a [Destination<Id>],
    selection: Selection<Id>,
    on_select: F,
    on_menu: Message,
    menu_progress: f32,
) -> Container<'a, Message, Theme, Renderer>
where
    Id: Copy + Eq + 'a,
    Message: Clone + 'a,
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'a,
    Font: Into<Renderer::Font>,
    F: Fn(Id) -> Message + Clone + 'a,
{
    rail_with(
        destinations,
        selection,
        on_select,
        NavigationRailOptions::default().menu_progress(on_menu, menu_progress),
    )
}

pub fn expanded_rail<'a, Id, Message, Renderer, F>(
    headline: &'static str,
    destinations: &'a [Destination<Id>],
    selection: Selection<Id>,
    on_select: F,
    on_menu: Message,
) -> Container<'a, Message, Theme, Renderer>
where
    Id: Copy + Eq + 'a,
    Message: Clone + 'a,
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'a,
    Font: Into<Renderer::Font>,
    F: Fn(Id) -> Message + Clone + 'a,
{
    expanded_rail_with(
        headline,
        destinations,
        selection,
        on_select,
        on_menu,
        ExpandedRailOptions::default(),
    )
}

#[derive(Debug, Clone, Copy)]
pub struct ExpandedRailOptions {
    width: f32,
    fit_content: bool,
}

impl Default for ExpandedRailOptions {
    fn default() -> Self {
        Self {
            width: tokens::component::navigation_rail::EXPANDED_CONTAINER_WIDTH,
            fit_content: false,
        }
    }
}

impl ExpandedRailOptions {
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub fn fit_content(mut self) -> Self {
        self.fit_content = true;
        self
    }
}

pub fn expanded_rail_with<'a, Id, Message, Renderer, F>(
    headline: &'static str,
    destinations: &'a [Destination<Id>],
    selection: Selection<Id>,
    on_select: F,
    on_menu: Message,
    options: ExpandedRailOptions,
) -> Container<'a, Message, Theme, Renderer>
where
    Id: Copy + Eq + 'a,
    Message: Clone + 'a,
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'a,
    Font: Into<Renderer::Font>,
    F: Fn(Id) -> Message + Clone + 'a,
{
    let metrics = ExpandedRailMetrics::new(options.width);
    let mut items = Column::new()
        .width(Length::Fixed(metrics.width()))
        .height(Length::Fill)
        .spacing(tokens::component::navigation_rail::VERTICAL_PADDING)
        .align_x(alignment::Horizontal::Center)
        .push(expanded_rail_header(headline, on_menu, metrics));

    for destination in destinations {
        items = items.push(expanded_rail_item(
            *destination,
            selection,
            on_select.clone(),
            metrics,
        ));
    }

    let rail = Container::new(items)
        .width(Length::Fixed(metrics.width()))
        .height(Length::Fill)
        .padding(Padding {
            top: tokens::component::navigation_rail::CONTENT_TOP_MARGIN,
            right: 0.0,
            bottom: tokens::component::navigation_rail::VERTICAL_PADDING,
            left: 0.0,
        })
        .style(rail_container);

    if options.fit_content {
        rail.height(Length::Fixed(rail_min_height(destinations.len(), true)))
    } else {
        rail
    }
}

fn rail_with_optional_header<'a, Id, Message, Renderer, F>(
    destinations: &'a [Destination<Id>],
    selection: Selection<Id>,
    on_select: F,
    header: Option<Element<'a, Message, Theme, Renderer>>,
) -> Container<'a, Message, Theme, Renderer>
where
    Id: Copy + Eq + 'a,
    Message: Clone + 'a,
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'a,
    Font: Into<Renderer::Font>,
    F: Fn(Id) -> Message + Clone + 'a,
{
    let mut items = Column::new()
        .width(Length::Fixed(
            tokens::component::navigation_rail::CONTAINER_WIDTH,
        ))
        .height(Length::Fill)
        .spacing(tokens::component::navigation_rail::VERTICAL_PADDING)
        .align_x(alignment::Horizontal::Center);

    if let Some(header) = header {
        items = items.push(rail_header(header));
    }

    for destination in destinations {
        items = items.push(navigation_rail_item(
            *destination,
            selection,
            on_select.clone(),
        ));
    }

    Container::new(items)
        .width(Length::Fixed(
            tokens::component::navigation_rail::CONTAINER_WIDTH,
        ))
        .height(Length::Fill)
        .padding(Padding {
            top: tokens::component::navigation_rail::CONTENT_TOP_MARGIN,
            right: 0.0,
            bottom: tokens::component::navigation_rail::VERTICAL_PADDING,
            left: 0.0,
        })
        .style(rail_container)
}

pub fn drawer<'a, Id, Message, Renderer, F>(
    headline: &'static str,
    destinations: &'a [Destination<Id>],
    selection: Selection<Id>,
    on_select: F,
) -> Container<'a, Message, Theme, Renderer>
where
    Id: Copy + Eq + 'a,
    Message: Clone + 'a,
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'a,
    Font: Into<Renderer::Font>,
    F: Fn(Id) -> Message + Clone + 'a,
{
    drawer_with(
        headline,
        destinations,
        selection,
        on_select,
        NavigationDrawerOptions::default(),
    )
}

#[derive(Debug, Clone, Copy)]
pub struct NavigationDrawerOptions {
    width: f32,
}

impl Default for NavigationDrawerOptions {
    fn default() -> Self {
        Self {
            width: tokens::component::navigation_drawer::CONTAINER_WIDTH,
        }
    }
}

impl NavigationDrawerOptions {
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }
}

pub fn drawer_with<'a, Id, Message, Renderer, F>(
    headline: &'static str,
    destinations: &'a [Destination<Id>],
    selection: Selection<Id>,
    on_select: F,
    options: NavigationDrawerOptions,
) -> Container<'a, Message, Theme, Renderer>
where
    Id: Copy + Eq + 'a,
    Message: Clone + 'a,
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'a,
    Font: Into<Renderer::Font>,
    F: Fn(Id) -> Message + Clone + 'a,
{
    drawer_with_optional_header(
        headline,
        destinations,
        selection,
        on_select,
        options.width,
        None,
    )
}

pub fn drawer_menu<'a, Id, Message, Renderer, F>(
    headline: &'static str,
    destinations: &'a [Destination<Id>],
    selection: Selection<Id>,
    on_select: F,
    on_menu: Message,
) -> Container<'a, Message, Theme, Renderer>
where
    Id: Copy + Eq + 'a,
    Message: Clone + 'a,
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'a,
    Font: Into<Renderer::Font>,
    F: Fn(Id) -> Message + Clone + 'a,
{
    drawer_menu_with(
        headline,
        destinations,
        selection,
        on_select,
        on_menu,
        NavigationDrawerOptions::default(),
    )
}

pub fn drawer_menu_with<'a, Id, Message, Renderer, F>(
    headline: &'static str,
    destinations: &'a [Destination<Id>],
    selection: Selection<Id>,
    on_select: F,
    on_menu: Message,
    options: NavigationDrawerOptions,
) -> Container<'a, Message, Theme, Renderer>
where
    Id: Copy + Eq + 'a,
    Message: Clone + 'a,
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'a,
    Font: Into<Renderer::Font>,
    F: Fn(Id) -> Message + Clone + 'a,
{
    drawer_with_optional_header(
        headline,
        destinations,
        selection,
        on_select,
        options.width,
        Some(drawer_menu_header(headline, on_menu).into()),
    )
}

fn drawer_with_optional_header<'a, Id, Message, Renderer, F>(
    headline: &'static str,
    destinations: &'a [Destination<Id>],
    selection: Selection<Id>,
    on_select: F,
    width: f32,
    header: Option<Element<'a, Message, Theme, Renderer>>,
) -> Container<'a, Message, Theme, Renderer>
where
    Id: Copy + Eq + 'a,
    Message: Clone + 'a,
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'a,
    Font: Into<Renderer::Font>,
    F: Fn(Id) -> Message + Clone + 'a,
{
    let metrics = DrawerMetrics::new(width);
    let headline_scale = tokens::component::navigation_drawer::HEADLINE_TEXT;
    let mut items = Column::new()
        .width(Length::Fixed(metrics.width()))
        .height(Length::Fill)
        .spacing(0);

    if let Some(header) = header {
        items = items.push(header);
    } else {
        items = items.push(
            Container::new(type_text(headline, headline_scale).style(headline_text_style))
                .height(Length::Fixed(
                    tokens::component::navigation_drawer::ACTIVE_INDICATOR_HEIGHT,
                ))
                .padding(Padding {
                    top: 0.0,
                    right: tokens::component::navigation_drawer::ITEM_HORIZONTAL_PADDING
                        + tokens::component::navigation_drawer::ITEM_CONTENT_TRAILING_SPACE,
                    bottom: 0.0,
                    left: tokens::component::navigation_drawer::ITEM_HORIZONTAL_PADDING
                        + tokens::component::navigation_drawer::ITEM_CONTENT_LEADING_SPACE,
                })
                .align_y(alignment::Vertical::Center),
        );
    }

    for destination in destinations {
        items = items.push(drawer_item(
            *destination,
            selection,
            on_select.clone(),
            metrics.indicator_width(),
        ));
    }

    Container::new(items)
        .width(Length::Fixed(metrics.width()))
        .height(Length::Fill)
        .padding(Padding {
            top: tokens::component::navigation_drawer::ITEM_HORIZONTAL_PADDING,
            right: tokens::component::navigation_drawer::ITEM_HORIZONTAL_PADDING,
            bottom: tokens::component::navigation_drawer::ITEM_HORIZONTAL_PADDING,
            left: tokens::component::navigation_drawer::ITEM_HORIZONTAL_PADDING,
        })
        .style(drawer_container)
}

pub fn drawer_width(progress: f32) -> f32 {
    let progress = progress.clamp(0.0, 1.0);

    if progress <= f32::EPSILON {
        0.0
    } else {
        lerp(
            tokens::component::navigation_drawer::MINIMUM_CONTAINER_WIDTH,
            tokens::component::navigation_drawer::CONTAINER_WIDTH,
            progress,
        )
    }
}

fn navigation_bar_item<'a, Id, Message, Renderer, F>(
    destination: Destination<Id>,
    selection: Selection<Id>,
    on_select: F,
) -> Element<'a, Message, Theme, Renderer>
where
    Id: Copy + Eq + 'a,
    Message: Clone + 'a,
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'a,
    Font: Into<Renderer::Font>,
    F: Fn(Id) -> Message + Clone + 'a,
{
    let size_progress = selection.size_progress(destination.id);
    let alpha_progress = selection.alpha_progress(destination.id);
    let scale = tokens::component::navigation_bar::LABEL_TEXT;
    let message = on_select(destination.id);
    let indicator = indicator_icon_stack(IndicatorIconSpec {
        icon: destination.icon,
        icon_size: tokens::component::navigation_bar::ICON_SIZE,
        indicator_size: Size::new(
            tokens::component::navigation_bar::ACTIVE_INDICATOR_WIDTH,
            tokens::component::navigation_bar::ACTIVE_INDICATOR_HEIGHT,
        ),
        size_progress,
        alpha_progress,
        badge: destination.badge,
        drawer: false,
    });
    let label = type_text(destination.label, scale).style(move |theme| text::Style {
        color: Some(bar_or_rail_label_color(theme, alpha_progress)),
    });
    let content = Column::new()
        .width(Length::Fill)
        .spacing(tokens::component::navigation_bar::INDICATOR_TO_LABEL_PADDING)
        .align_x(alignment::Horizontal::Center)
        .push(indicator)
        .push(label);

    press_surface(
        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fixed(
                tokens::component::navigation_bar::CONTAINER_HEIGHT,
            ))
            .padding(Padding {
                top: tokens::component::navigation_bar::INDICATOR_VERTICAL_OFFSET,
                right: 0.0,
                bottom: BarMetrics::item_bottom_padding(),
                left: 0.0,
            })
            .align_y(alignment::Vertical::Center),
        message,
        NavigationStateLayer::BarOrRail,
        NavigationIndicatorPlacement::TopCenter {
            top: tokens::component::navigation_bar::INDICATOR_VERTICAL_OFFSET,
            width: tokens::component::navigation_bar::ACTIVE_INDICATOR_WIDTH,
            height: tokens::component::navigation_bar::ACTIVE_INDICATOR_HEIGHT,
        },
    )
    .into()
}

fn navigation_rail_item<'a, Id, Message, Renderer, F>(
    destination: Destination<Id>,
    selection: Selection<Id>,
    on_select: F,
) -> Element<'a, Message, Theme, Renderer>
where
    Id: Copy + Eq + 'a,
    Message: Clone + 'a,
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'a,
    Font: Into<Renderer::Font>,
    F: Fn(Id) -> Message + Clone + 'a,
{
    let size_progress = selection.size_progress(destination.id);
    let alpha_progress = selection.alpha_progress(destination.id);
    let scale = tokens::component::navigation_rail::LABEL_TEXT;
    let message = on_select(destination.id);
    let indicator = indicator_icon_stack(IndicatorIconSpec {
        icon: destination.icon,
        icon_size: tokens::component::navigation_rail::ICON_SIZE,
        indicator_size: Size::new(
            tokens::component::navigation_rail::ACTIVE_INDICATOR_WIDTH,
            tokens::component::navigation_rail::ACTIVE_INDICATOR_HEIGHT,
        ),
        size_progress,
        alpha_progress,
        badge: destination.badge,
        drawer: false,
    });
    let label = type_text(destination.label, scale).style(move |theme| text::Style {
        color: Some(bar_or_rail_label_color(theme, alpha_progress)),
    });
    let content = Column::new()
        .width(Length::Fixed(
            tokens::component::navigation_rail::ITEM_WIDTH,
        ))
        .spacing(tokens::component::navigation_rail::ITEM_VERTICAL_PADDING)
        .align_x(alignment::Horizontal::Center)
        .push(indicator)
        .push(label);

    press_surface(
        Container::new(content)
            .width(Length::Fixed(
                tokens::component::navigation_rail::ITEM_WIDTH,
            ))
            .height(Length::Fixed(
                tokens::component::navigation_rail::ITEM_HEIGHT,
            ))
            .padding(Padding {
                top: RailMetrics::item_content_top_padding(),
                right: 0.0,
                bottom: 0.0,
                left: 0.0,
            }),
        message,
        NavigationStateLayer::BarOrRail,
        NavigationIndicatorPlacement::TopCenter {
            top: RailMetrics::item_content_top_padding(),
            width: tokens::component::navigation_rail::ACTIVE_INDICATOR_WIDTH,
            height: tokens::component::navigation_rail::ACTIVE_INDICATOR_HEIGHT,
        },
    )
    .into()
}

fn rail_header<'a, Message, Renderer>(
    header: Element<'a, Message, Theme, Renderer>,
) -> Container<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: geometry::Renderer + primitive::Renderer + 'a,
{
    Container::new(header)
        .width(Length::Fixed(
            tokens::component::navigation_rail::CONTAINER_WIDTH,
        ))
        .padding(Padding {
            top: 0.0,
            right: 0.0,
            bottom: RailMetrics::header_bottom_padding(),
            left: 0.0,
        })
        .align_x(alignment::Horizontal::Center)
}

fn navigation_menu_button<'a, Message, Renderer>(
    on_press: Message,
    progress: f32,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'a,
    Font: Into<Renderer::Font>,
{
    let icon = Canvas::new(NavigationMenuIcon { progress })
        .width(Length::Fixed(tokens::component::icon_button::ICON_SIZE))
        .height(Length::Fixed(tokens::component::icon_button::ICON_SIZE));

    Button::new(
        Container::new(icon)
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
    .style(button_style::icon)
    .on_press(on_press)
}

#[derive(Debug, Clone, Copy)]
struct NavigationMenuIcon {
    progress: f32,
}

impl<Message, Renderer> canvas::Program<Message, Theme, Renderer> for NavigationMenuIcon
where
    Renderer: geometry::Renderer,
{
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry<Renderer>> {
        let size = bounds.width.min(bounds.height);

        if size <= 0.0 {
            return Vec::new();
        }

        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let offset = Vector::new((bounds.width - size) / 2.0, (bounds.height - size) / 2.0);
        let center = Point::new(bounds.width / 2.0, bounds.height / 2.0);
        let stroke = Stroke::default()
            .with_width(NavigationMenuIcon::stroke_width(size))
            .with_color(theme.colors().surface.text_variant)
            .with_line_cap(LineCap::Round);

        frame.with_save(|frame| {
            frame.translate(Vector::new(center.x, center.y));
            frame.rotate(self.rotation_radians());
            frame.translate(Vector::new(-center.x, -center.y));

            for (from, to) in navigation_menu_icon_segments(self.progress, size) {
                frame.stroke(
                    &Path::line(
                        Point::new(from.x + offset.x, from.y + offset.y),
                        Point::new(to.x + offset.x, to.y + offset.y),
                    ),
                    stroke,
                );
            }
        });

        vec![frame.into_geometry()]
    }
}

impl NavigationMenuIcon {
    fn rotation_radians(self) -> f32 {
        PI * self.progress.clamp(0.0, 1.0)
    }

    fn stroke_width(size: f32) -> f32 {
        NAVIGATION_MENU_ICON_STROKE_WIDTH / NAVIGATION_MENU_ICON_VIEWPORT_SIZE * size
    }
}

fn navigation_menu_icon_segments(progress: f32, size: f32) -> [(Point, Point); 3] {
    let progress = progress.clamp(0.0, 1.0);

    [
        (
            navigation_menu_icon_point(
                lerp(
                    NAVIGATION_MENU_ICON_START_X,
                    NAVIGATION_MENU_ICON_CENTER_X,
                    progress,
                ),
                lerp(
                    NAVIGATION_MENU_ICON_TOP_Y,
                    NAVIGATION_MENU_ICON_ARROW_TOP_Y,
                    progress,
                ),
                size,
            ),
            navigation_menu_icon_point(
                NAVIGATION_MENU_ICON_END_X,
                lerp(
                    NAVIGATION_MENU_ICON_TOP_Y,
                    NAVIGATION_MENU_ICON_CENTER_Y,
                    progress,
                ),
                size,
            ),
        ),
        (
            navigation_menu_icon_point(
                NAVIGATION_MENU_ICON_START_X,
                NAVIGATION_MENU_ICON_CENTER_Y,
                size,
            ),
            navigation_menu_icon_point(
                NAVIGATION_MENU_ICON_END_X,
                NAVIGATION_MENU_ICON_CENTER_Y,
                size,
            ),
        ),
        (
            navigation_menu_icon_point(
                lerp(
                    NAVIGATION_MENU_ICON_START_X,
                    NAVIGATION_MENU_ICON_CENTER_X,
                    progress,
                ),
                lerp(
                    NAVIGATION_MENU_ICON_BOTTOM_Y,
                    NAVIGATION_MENU_ICON_ARROW_BOTTOM_Y,
                    progress,
                ),
                size,
            ),
            navigation_menu_icon_point(
                NAVIGATION_MENU_ICON_END_X,
                lerp(
                    NAVIGATION_MENU_ICON_BOTTOM_Y,
                    NAVIGATION_MENU_ICON_CENTER_Y,
                    progress,
                ),
                size,
            ),
        ),
    ]
}

fn navigation_menu_icon_point(x: f32, y: f32, size: f32) -> Point {
    Point::new(
        x / NAVIGATION_MENU_ICON_VIEWPORT_SIZE * size,
        y / NAVIGATION_MENU_ICON_VIEWPORT_SIZE * size,
    )
}

fn expanded_rail_header<'a, Message, Renderer>(
    headline: &'static str,
    on_menu: Message,
    metrics: ExpandedRailMetrics,
) -> Container<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'a,
    Font: Into<Renderer::Font>,
{
    let headline_scale = tokens::component::navigation_drawer::HEADLINE_TEXT;
    let headline =
        single_line_type_text(headline, headline_scale).style(move |theme| text::Style {
            color: Some(alpha_color(
                theme.colors().surface.text_variant,
                metrics.label_alpha(),
            )),
        });
    let headline = Container::new(headline)
        .width(Length::Fill)
        .height(Length::Fixed(
            tokens::component::icon_button::CONTAINER_HEIGHT,
        ))
        .align_y(alignment::Vertical::Center)
        .clip(true);
    let content = Row::new()
        .width(Length::Fill)
        .height(Length::Fixed(
            tokens::component::icon_button::CONTAINER_HEIGHT,
        ))
        .spacing(metrics.header_title_spacing())
        .align_y(alignment::Vertical::Center)
        .push(navigation_menu_button(on_menu, metrics.progress()))
        .push(headline);

    Container::new(content)
        .height(Length::Fixed(RailMetrics::header_slot_height()))
        .width(Length::Fill)
        .padding(Padding {
            top: 0.0,
            right: tokens::component::navigation_rail::EXPANDED_ACTIVE_INDICATOR_MARGIN_HORIZONTAL,
            bottom: RailMetrics::header_bottom_padding(),
            left: metrics.header_leading_space(),
        })
        .align_y(alignment::Vertical::Center)
}

fn expanded_rail_item<'a, Id, Message, Renderer, F>(
    destination: Destination<Id>,
    selection: Selection<Id>,
    on_select: F,
    metrics: ExpandedRailMetrics,
) -> Element<'a, Message, Theme, Renderer>
where
    Id: Copy + Eq + 'a,
    Message: Clone + 'a,
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'a,
    Font: Into<Renderer::Font>,
    F: Fn(Id) -> Message + Clone + 'a,
{
    let size_progress = selection.size_progress(destination.id);
    let alpha_progress = selection.alpha_progress(destination.id);
    let indicator_width = metrics.indicator_width();
    let indicator_height = metrics.indicator_height();
    let vertical_inset = metrics.item_vertical_inset();
    let scale = tokens::component::navigation_drawer::LABEL_TEXT;
    let message = on_select(destination.id);
    let badge_on_icon = metrics.badge_uses_icon_anchor();
    let trailing_badge_alpha = metrics.trailing_badge_alpha();
    let collapsed_label_alpha = metrics.collapsed_label_alpha();
    let icon = expanded_rail_icon_layer(
        destination.icon,
        alpha_progress,
        indicator_height,
        badge_on_icon.then_some(destination.badge).flatten(),
    );
    let label = type_text(destination.label, scale).style(move |theme| text::Style {
        color: Some(alpha_color(
            drawer_content_color(theme, alpha_progress),
            metrics.label_alpha(),
        )),
    });
    let content = Row::new()
        .width(Length::Fill)
        .height(Length::Fixed(indicator_height))
        .align_y(alignment::Vertical::Center)
        .push(Container::new(label).width(Length::Fill));
    let content = if let Some(badge) = destination.badge.filter(|_| !badge_on_icon) {
        content
            .push(Space::new().width(Length::Fixed(DrawerMetrics::badge_space())))
            .push(destination_badge_with_alpha::<Message, Renderer>(
                badge,
                trailing_badge_alpha,
            ))
    } else {
        content
    };
    let content = Container::new(content)
        .width(Length::Fixed(indicator_width))
        .height(Length::Fixed(indicator_height))
        .padding(Padding {
            top: 0.0,
            right: tokens::component::navigation_rail::EXPANDED_ACTIVE_INDICATOR_PADDING_END,
            bottom: 0.0,
            left: metrics.label_leading_padding(),
        })
        .align_y(alignment::Vertical::Center);
    let expanded_indicator = Stack::new()
        .width(Length::Fixed(indicator_width))
        .height(Length::Fixed(indicator_height))
        .push(
            Space::new()
                .width(Length::Fixed(indicator_width))
                .height(Length::Fixed(indicator_height)),
        )
        .push(indicator_layer(
            indicator_width,
            indicator_height,
            size_progress,
            alpha_progress,
        ))
        .push(content)
        .push(icon);
    let collapsed_label = collapsed_rail_label::<Message, Renderer>(
        destination.label,
        alpha_progress,
        collapsed_label_alpha,
    )
    .width(Length::Fixed(RailMetrics::collapsed_label_width()))
    .height(Length::Fixed(RailMetrics::item_slot_height()));
    let item = Stack::new()
        .width(Length::Fixed(indicator_width))
        .height(Length::Fixed(RailMetrics::item_slot_height()))
        .push(
            Container::new(expanded_indicator)
                .width(Length::Fixed(indicator_width))
                .height(Length::Fixed(RailMetrics::item_slot_height()))
                .padding(Padding {
                    top: vertical_inset,
                    right: 0.0,
                    bottom: vertical_inset,
                    left: 0.0,
                })
                .align_y(alignment::Vertical::Top),
        )
        .push(collapsed_label);

    press_surface(
        Container::new(item)
            .width(Length::Fixed(indicator_width))
            .height(Length::Fixed(RailMetrics::item_slot_height())),
        message,
        NavigationStateLayer::Drawer {
            progress: alpha_progress,
        },
        NavigationIndicatorPlacement::Inset {
            x: 0.0,
            y: vertical_inset,
            width: indicator_width,
            height: indicator_height,
        },
    )
    .into()
}

fn drawer_menu_header<'a, Message, Renderer>(
    headline: &'static str,
    on_menu: Message,
) -> Container<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'a,
    Font: Into<Renderer::Font>,
{
    let headline_scale = tokens::component::navigation_drawer::HEADLINE_TEXT;
    let content = Row::new()
        .height(Length::Fixed(
            tokens::component::navigation_drawer::ACTIVE_INDICATOR_HEIGHT,
        ))
        .spacing(DrawerMetrics::menu_header_title_spacing())
        .align_y(alignment::Vertical::Center)
        .push(navigation_menu_button(on_menu, 0.0))
        .push(type_text(headline, headline_scale).style(headline_text_style));

    Container::new(content)
        .height(Length::Fixed(
            tokens::component::navigation_drawer::ACTIVE_INDICATOR_HEIGHT,
        ))
        .padding(Padding {
            top: 0.0,
            right: tokens::component::navigation_drawer::ITEM_HORIZONTAL_PADDING
                + tokens::component::navigation_drawer::ITEM_CONTENT_TRAILING_SPACE,
            bottom: 0.0,
            left: DrawerMetrics::menu_header_leading_space(),
        })
        .align_y(alignment::Vertical::Center)
}

fn drawer_item<'a, Id, Message, Renderer, F>(
    destination: Destination<Id>,
    selection: Selection<Id>,
    on_select: F,
    indicator_width: f32,
) -> Element<'a, Message, Theme, Renderer>
where
    Id: Copy + Eq + 'a,
    Message: Clone + 'a,
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'a,
    Font: Into<Renderer::Font>,
    F: Fn(Id) -> Message + Clone + 'a,
{
    let size_progress = selection.size_progress(destination.id);
    let alpha_progress = selection.alpha_progress(destination.id);
    let scale = tokens::component::navigation_drawer::LABEL_TEXT;
    let message = on_select(destination.id);
    let icon = destination_icon::<Message, Renderer>(
        destination.icon,
        tokens::component::navigation_drawer::ICON_SIZE,
        alpha_progress,
        true,
    );
    let label = type_text(destination.label, scale).style(move |theme| text::Style {
        color: Some(drawer_content_color(theme, alpha_progress)),
    });
    let content = Row::new()
        .width(Length::Fill)
        .height(Length::Fixed(
            tokens::component::navigation_drawer::ACTIVE_INDICATOR_HEIGHT,
        ))
        .spacing(tokens::component::navigation_drawer::ICON_LABEL_SPACE)
        .align_y(alignment::Vertical::Center)
        .push(icon)
        .push(Container::new(label).width(Length::Fill));
    let content = if let Some(badge) = destination.badge {
        content
            .push(Space::new().width(Length::Fixed(DrawerMetrics::badge_space())))
            .push(destination_badge::<Message, Renderer>(badge))
    } else {
        content
    };
    let content = Container::new(content)
        .width(Length::Fixed(indicator_width))
        .height(Length::Fixed(
            tokens::component::navigation_drawer::ACTIVE_INDICATOR_HEIGHT,
        ))
        .padding(Padding {
            top: 0.0,
            right: tokens::component::navigation_drawer::ITEM_CONTENT_TRAILING_SPACE,
            bottom: 0.0,
            left: tokens::component::navigation_drawer::ITEM_CONTENT_LEADING_SPACE,
        })
        .align_y(alignment::Vertical::Center);
    let indicator = Stack::new()
        .width(Length::Fixed(indicator_width))
        .height(Length::Fixed(
            tokens::component::navigation_drawer::ACTIVE_INDICATOR_HEIGHT,
        ))
        .push(
            Space::new()
                .width(Length::Fixed(indicator_width))
                .height(Length::Fixed(
                    tokens::component::navigation_drawer::ACTIVE_INDICATOR_HEIGHT,
                )),
        )
        .push(indicator_layer(
            indicator_width,
            tokens::component::navigation_drawer::ACTIVE_INDICATOR_HEIGHT,
            size_progress,
            alpha_progress,
        ))
        .push(content);

    press_surface(
        indicator,
        message,
        NavigationStateLayer::Drawer {
            progress: alpha_progress,
        },
        NavigationIndicatorPlacement::Full,
    )
    .into()
}

#[derive(Debug, Clone, Copy)]
struct IndicatorIconSpec {
    icon: &'static str,
    icon_size: f32,
    indicator_size: Size,
    size_progress: f32,
    alpha_progress: f32,
    badge: Option<Badge>,
    drawer: bool,
}

fn indicator_icon_stack<'a, Message, Renderer>(
    spec: IndicatorIconSpec,
) -> Stack<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'a,
    Font: Into<Renderer::Font>,
{
    let indicator_width = spec.indicator_size.width;
    let indicator_height = spec.indicator_size.height;

    Stack::new()
        .width(Length::Fixed(indicator_width))
        .height(Length::Fixed(indicator_height))
        .push(
            Space::new()
                .width(Length::Fixed(indicator_width))
                .height(Length::Fixed(indicator_height)),
        )
        .push(indicator_layer(
            indicator_width,
            indicator_height,
            spec.size_progress,
            spec.alpha_progress,
        ))
        .push(
            destination_icon_anchor::<Message, Renderer>(
                spec.icon,
                spec.icon_size,
                spec.alpha_progress,
                spec.badge,
                spec.drawer,
            )
            .width(Length::Fixed(indicator_width))
            .height(Length::Fixed(indicator_height)),
        )
}

fn press_surface<'a, Message, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
    on_press: Message,
    layer: NavigationStateLayer,
    indicator: NavigationIndicatorPlacement,
) -> NavigationPressSurface<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + primitive::Renderer + 'a,
{
    NavigationPressSurface {
        content: content.into(),
        on_press,
        layer,
        indicator,
    }
}

struct NavigationPressSurface<'a, Message, Renderer>
where
    Renderer: geometry::Renderer + primitive::Renderer,
{
    content: Element<'a, Message, Theme, Renderer>,
    on_press: Message,
    layer: NavigationStateLayer,
    indicator: NavigationIndicatorPlacement,
}

#[derive(Debug, Clone, Copy)]
enum NavigationIndicatorPlacement {
    Full,
    TopCenter {
        top: f32,
        width: f32,
        height: f32,
    },
    Inset {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    },
}

impl NavigationIndicatorPlacement {
    fn bounds(self, bounds: Rectangle) -> Rectangle {
        match self {
            Self::Full => bounds,
            Self::TopCenter { top, width, height } => Rectangle {
                x: bounds.x + (bounds.width - width) / 2.0,
                y: bounds.y + top,
                width,
                height,
            },
            Self::Inset {
                x,
                y,
                width,
                height,
            } => Rectangle {
                x: bounds.x + x,
                y: bounds.y + y,
                width,
                height,
            },
        }
    }
}

#[derive(Debug)]
struct NavigationPressSurfaceState {
    is_hovered: bool,
    is_pressed: bool,
    state_layer_opacity: AnimatedScalar,
    ripples: PressRippleState,
    now: Option<Instant>,
}

impl Default for NavigationPressSurfaceState {
    fn default() -> Self {
        Self {
            is_hovered: false,
            is_pressed: false,
            state_layer_opacity: AnimatedScalar::new(0.0),
            ripples: PressRippleState::default(),
            now: None,
        }
    }
}

impl NavigationPressSurfaceState {
    fn sync_hover(&mut self, is_hovered: bool, now: Instant) -> bool {
        if self.is_hovered == is_hovered {
            return false;
        }

        self.is_hovered = is_hovered;

        if !self.is_pressed {
            if !is_hovered {
                self.clear_ripples();
            }

            self.animate_to_interaction_target(now);
        }

        true
    }

    fn press(&mut self, origin: Point, now: Instant) {
        self.is_pressed = true;
        self.ripples.press(
            origin,
            now,
            RippleStart::Replace,
            RippleStyle::material_patterned(),
        );
        self.now = Some(now);
        self.animate_to_interaction_target(now);
    }

    fn release(&mut self, is_hovered: bool, now: Instant) {
        self.release_with_hover(is_hovered, is_hovered, now);
    }

    fn release_with_hover(&mut self, keep_ripple: bool, is_hovered: bool, now: Instant) {
        self.is_pressed = false;
        self.is_hovered = is_hovered;

        if keep_ripple {
            self.ripples.release_replacing(now);
        } else {
            self.clear_ripples();
        }

        self.now = Some(now);
        self.animate_to_interaction_target(now);
    }

    fn snap_to_interaction_target(&mut self) {
        self.state_layer_opacity
            .snap_to(NavigationLayer::target(self.is_hovered, self.is_pressed));
    }

    fn cancel(&mut self, now: Instant) {
        self.is_pressed = false;
        self.is_hovered = false;
        self.clear_ripples();

        self.now = Some(now);
        self.animate_to_interaction_target(now);
    }

    fn advance(&mut self, now: Instant) -> bool {
        self.now = Some(now);
        self.prune(now);

        self.state_layer_opacity.advance(now) || self.has_visible_ripples(now)
    }

    fn opacity(&self) -> f32 {
        NavigationLayer::opacity(self.state_layer_opacity.value)
    }

    fn animate_to_interaction_target(&mut self, now: Instant) {
        self.state_layer_opacity.set_target(
            NavigationLayer::target(self.is_hovered, self.is_pressed),
            now,
            duration_ms(tokens::motion::DURATION_SHORT2_MS),
            tokens::motion::EASING_STANDARD,
        );
    }

    fn clear_ripples(&mut self) {
        self.ripples.clear();
    }

    fn prune(&mut self, now: Instant) {
        self.ripples.prune(now);
    }

    fn has_visible_ripples(&self, now: Instant) -> bool {
        self.ripples.has_visible_ripples(now)
    }
}

impl<Message, Renderer> Widget<Message, Theme, Renderer>
    for NavigationPressSurface<'_, Message, Renderer>
where
    Message: Clone,
    Renderer: geometry::Renderer + primitive::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<NavigationPressSurfaceState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(NavigationPressSurfaceState::default())
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
        operation: &mut dyn Operation,
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

        if shell.is_event_captured() {
            return;
        }

        let state = tree.state.downcast_mut::<NavigationPressSurfaceState>();
        let now = match event {
            Event::Window(window::Event::RedrawRequested(now)) => Some(*now),
            _ => None,
        };
        let is_touch_event = matches!(event, Event::Touch(_));
        let is_hovered = !is_touch_event && cursor.is_over(layout.bounds());
        let interaction = NavigationInteraction {
            event,
            cursor,
            is_hovered,
        };
        let pointer = NavigationPointer { event, cursor };
        let should_snap_initial_redraw_hover = interaction.should_snap_initial_redraw(state);

        if interaction.should_sync_hover()
            && state.sync_hover(is_hovered, now.unwrap_or_else(Instant::now))
        {
            if should_snap_initial_redraw_hover {
                state.snap_to_interaction_target();
            }

            shell.request_redraw();
        }

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. })
                if pointer.is_over(layout.bounds()) =>
            {
                let indicator_bounds = self.indicator.bounds(layout.bounds());

                if let Some(origin) = pointer.press_origin(indicator_bounds) {
                    state.press(origin, now.unwrap_or_else(Instant::now));
                    shell.request_redraw();
                    shell.capture_event();
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. })
                if state.is_pressed =>
            {
                let is_released_over = pointer.is_over(layout.bounds());
                let is_touch_release = matches!(event, Event::Touch(_));

                if is_touch_release {
                    state.release_with_hover(
                        is_released_over,
                        false,
                        now.unwrap_or_else(Instant::now),
                    );
                } else {
                    state.release(is_released_over, now.unwrap_or_else(Instant::now));
                }
                shell.request_redraw();

                if is_released_over {
                    shell.publish(self.on_press.clone());
                }

                shell.capture_event();
            }
            Event::Touch(touch::Event::FingerLost { .. }) if state.is_pressed => {
                state.cancel(now.unwrap_or_else(Instant::now));
                shell.request_redraw();
            }
            _ => {}
        }

        if let Some(now) = now
            && state.advance(now)
        {
            shell.request_redraw();
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let content_interaction = self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        );

        if matches!(content_interaction, mouse::Interaction::None)
            && cursor.is_over(layout.bounds())
        {
            mouse::Interaction::Pointer
        } else {
            content_interaction
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        renderer_style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            renderer_style,
            layout,
            cursor,
            viewport,
        );

        let state = tree.state.downcast_ref::<NavigationPressSurfaceState>();
        let indicator_bounds = self.indicator.bounds(layout.bounds());
        let now = state.now.unwrap_or_else(Instant::now);
        let opacity = NavigationDrawState {
            state,
            cursor,
            bounds: layout.bounds(),
        }
        .opacity();
        let layer_color = layer_color(theme, self.layer);

        if opacity > 0.0 {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: indicator_bounds,
                    border: border::rounded(tokens::shape::CORNER_FULL),
                    snap: cfg!(feature = "crisp"),
                    ..renderer::Quad::default()
                },
                state_layer(layer_color, opacity),
            );
        }

        draw_ripples(
            renderer,
            indicator_bounds,
            &state.ripples,
            layer_color,
            RippleConfig::bounded(border::radius(tokens::shape::CORNER_FULL)),
            now,
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
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message, Renderer> From<NavigationPressSurface<'a, Message, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + primitive::Renderer + 'a,
{
    fn from(surface: NavigationPressSurface<'a, Message, Renderer>) -> Self {
        Element::new(surface)
    }
}

struct NavigationLayer;

impl NavigationLayer {
    fn target(is_hovered: bool, _is_pressed: bool) -> f32 {
        if is_hovered {
            HOVERED_LAYER_OPACITY
        } else {
            0.0
        }
    }

    fn opacity(interaction_opacity: f32) -> f32 {
        interaction_opacity
    }
}

#[derive(Debug, Clone, Copy)]
struct NavigationInteraction<'a> {
    event: &'a Event,
    cursor: mouse::Cursor,
    is_hovered: bool,
}

impl NavigationInteraction<'_> {
    fn should_sync_hover(self) -> bool {
        match self.event {
            Event::Window(window::Event::RedrawRequested(_)) => {
                !matches!(self.cursor, mouse::Cursor::Unavailable)
            }
            Event::Mouse(_) | Event::Touch(_) => true,
            _ => false,
        }
    }

    fn should_snap_initial_redraw(self, state: &NavigationPressSurfaceState) -> bool {
        matches!(self.event, Event::Window(window::Event::RedrawRequested(_)))
            && state.now.is_none()
            && self.is_hovered
    }
}

#[derive(Debug, Clone, Copy)]
struct NavigationDrawState<'a> {
    state: &'a NavigationPressSurfaceState,
    cursor: mouse::Cursor,
    bounds: Rectangle,
}

impl NavigationDrawState<'_> {
    fn opacity(self) -> f32 {
        if self.cursor.is_over(self.bounds) && self.state.now.is_none() {
            NavigationLayer::opacity(NavigationLayer::target(true, false))
        } else {
            self.state.opacity()
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct NavigationPointer<'a> {
    event: &'a Event,
    cursor: mouse::Cursor,
}

impl NavigationPointer<'_> {
    fn is_over(self, bounds: Rectangle) -> bool {
        if self.cursor.position().is_some() {
            return self.cursor.is_over(bounds);
        }

        if self.cursor.is_levitating() {
            return false;
        }

        self.position()
            .map(|position| bounds.contains(position))
            .unwrap_or_else(|| self.cursor.is_over(bounds))
    }

    fn press_origin(self, indicator_bounds: Rectangle) -> Option<Point> {
        let position = self.cursor.position().or_else(|| self.position())?;

        if self.cursor.is_levitating() {
            return None;
        }

        Some(position - Vector::new(indicator_bounds.x, indicator_bounds.y))
    }

    fn position(self) -> Option<Point> {
        match self.event {
            Event::Touch(touch::Event::FingerPressed { position, .. })
            | Event::Touch(touch::Event::FingerMoved { position, .. })
            | Event::Touch(touch::Event::FingerLifted { position, .. })
            | Event::Touch(touch::Event::FingerLost { position, .. }) => Some(*position),
            _ => None,
        }
    }
}

fn indicator_layer<'a, Message, Renderer>(
    target_width: f32,
    height: f32,
    size_progress: f32,
    alpha_progress: f32,
) -> Container<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: geometry::Renderer + primitive::Renderer + 'a,
{
    let indicator = Container::new(Space::new())
        .width(Length::Fixed(animated_indicator_width(
            target_width,
            size_progress,
        )))
        .height(Length::Fixed(height))
        .style(move |theme| active_indicator(theme, alpha_progress));

    Container::new(indicator)
        .width(Length::Fixed(target_width))
        .height(Length::Fixed(height))
        .align_x(alignment::Horizontal::Center)
        .align_y(alignment::Vertical::Center)
}

#[derive(Debug, Clone, Copy)]
enum NavigationStateLayer {
    BarOrRail,
    Drawer { progress: f32 },
}

fn animated_indicator_width(target_width: f32, progress: f32) -> f32 {
    // AndroidX Material3 measures the selected indicator width from animation progress.
    target_width * progress.max(0.0)
}

struct BarMetrics;

impl BarMetrics {
    fn item_bottom_padding() -> f32 {
        let label = tokens::component::navigation_bar::LABEL_TEXT;

        (tokens::component::navigation_bar::CONTAINER_HEIGHT
            - tokens::component::navigation_bar::INDICATOR_VERTICAL_OFFSET
            - tokens::component::navigation_bar::ACTIVE_INDICATOR_HEIGHT
            - tokens::component::navigation_bar::INDICATOR_TO_LABEL_PADDING
            - label.line_height)
            .max(0.0)
    }
}

struct RailMetrics;

impl RailMetrics {
    fn min_height(destination_count: usize, has_header: bool) -> f32 {
        let header_height = if has_header {
            Self::header_slot_height()
        } else {
            0.0
        };
        let child_count = destination_count + usize::from(has_header);
        let spacing_count = child_count.saturating_sub(1);

        tokens::component::navigation_rail::CONTENT_TOP_MARGIN
            + header_height
            + destination_count as f32 * Self::item_slot_height()
            + spacing_count as f32 * tokens::component::navigation_rail::VERTICAL_PADDING
            + tokens::component::navigation_rail::VERTICAL_PADDING
    }

    fn item_content_top_padding() -> f32 {
        tokens::component::navigation_rail::ITEM_TOP_PADDING
    }

    fn header_bottom_padding() -> f32 {
        tokens::component::navigation_rail::HEADER_PADDING
    }

    fn header_slot_height() -> f32 {
        tokens::component::icon_button::CONTAINER_HEIGHT + Self::header_bottom_padding()
    }

    fn item_slot_height() -> f32 {
        tokens::component::navigation_rail::ITEM_HEIGHT
    }

    fn collapsed_label_top_padding() -> f32 {
        Self::item_content_top_padding()
            + tokens::component::navigation_rail::ACTIVE_INDICATOR_HEIGHT
            + tokens::component::navigation_rail::ITEM_VERTICAL_PADDING
    }

    fn collapsed_label_width() -> f32 {
        tokens::component::navigation_rail::ACTIVE_INDICATOR_WIDTH
    }

    #[cfg(test)]
    fn collapsed_icon_center_x() -> f32 {
        tokens::component::navigation_rail::CONTAINER_WIDTH / 2.0
    }

    #[cfg(test)]
    fn collapsed_icon_center_y() -> f32 {
        Self::item_content_top_padding()
            + tokens::component::navigation_rail::ACTIVE_INDICATOR_HEIGHT / 2.0
    }

    #[cfg(test)]
    fn first_item_y_after_header() -> f32 {
        tokens::component::navigation_rail::CONTENT_TOP_MARGIN
            + Self::header_slot_height()
            + tokens::component::navigation_rail::VERTICAL_PADDING
    }
}

#[derive(Debug, Clone, Copy)]
struct ExpandedRailMetrics {
    width: f32,
}

impl ExpandedRailMetrics {
    fn new(width: f32) -> Self {
        Self {
            width: width.clamp(
                tokens::component::navigation_rail::CONTAINER_WIDTH,
                tokens::component::navigation_drawer::CONTAINER_WIDTH,
            ),
        }
    }

    fn width(self) -> f32 {
        self.width
    }

    fn indicator_width(self) -> f32 {
        (self.width
            - tokens::component::navigation_rail::EXPANDED_ACTIVE_INDICATOR_MARGIN_HORIZONTAL * 2.0)
            .max(0.0)
    }

    fn progress(self) -> f32 {
        let range = tokens::component::navigation_rail::EXPANDED_CONTAINER_WIDTH
            - tokens::component::navigation_rail::CONTAINER_WIDTH;

        if range <= f32::EPSILON {
            1.0
        } else {
            ((self.width - tokens::component::navigation_rail::CONTAINER_WIDTH) / range)
                .clamp(0.0, 1.0)
        }
    }

    fn label_alpha(self) -> f32 {
        ((self.progress() - 0.6) / 0.4).clamp(0.0, 1.0)
    }

    fn item_vertical_inset(self) -> f32 {
        Self::item_vertical_inset_for(self.progress())
    }

    fn item_vertical_inset_for(progress: f32) -> f32 {
        lerp(
            RailMetrics::item_content_top_padding(),
            Self::expanded_item_vertical_inset(),
            progress.clamp(0.0, 1.0),
        )
    }

    fn indicator_height(self) -> f32 {
        Self::indicator_height_for(self.progress())
    }

    fn indicator_height_for(progress: f32) -> f32 {
        lerp(
            tokens::component::navigation_rail::ACTIVE_INDICATOR_HEIGHT,
            tokens::component::navigation_rail::EXPANDED_ACTIVE_INDICATOR_HEIGHT,
            progress.clamp(0.0, 1.0),
        )
    }

    fn label_leading_padding(self) -> f32 {
        Self::icon_anchor_width() + tokens::component::navigation_rail::ICON_LABEL_HORIZONTAL_SPACE
    }

    fn badge_uses_icon_anchor(self) -> bool {
        Self::badge_uses_icon_anchor_for(self.label_alpha())
    }

    fn badge_uses_icon_anchor_for(label_alpha: f32) -> bool {
        label_alpha <= 0.0
    }

    fn trailing_badge_alpha(self) -> f32 {
        Self::trailing_badge_alpha_for(self.label_alpha())
    }

    fn trailing_badge_alpha_for(label_alpha: f32) -> f32 {
        label_alpha.clamp(0.0, 1.0)
    }

    fn collapsed_label_alpha(self) -> f32 {
        Self::collapsed_label_alpha_for(self.label_alpha())
    }

    fn collapsed_label_alpha_for(label_alpha: f32) -> f32 {
        (1.0 - label_alpha).clamp(0.0, 1.0)
    }

    fn header_leading_space(self) -> f32 {
        tokens::component::navigation_rail::EXPANDED_ACTIVE_INDICATOR_MARGIN_HORIZONTAL
            + tokens::component::navigation_rail::EXPANDED_ACTIVE_INDICATOR_PADDING_START
            - (tokens::component::icon_button::CONTAINER_WIDTH
                - tokens::component::navigation_rail::ICON_SIZE)
                / 2.0
    }

    fn header_title_spacing(self) -> f32 {
        let label_start =
            tokens::component::navigation_rail::EXPANDED_ACTIVE_INDICATOR_MARGIN_HORIZONTAL
                + tokens::component::navigation_rail::EXPANDED_ACTIVE_INDICATOR_PADDING_START
                + tokens::component::navigation_rail::ICON_SIZE
                + tokens::component::navigation_rail::ICON_LABEL_HORIZONTAL_SPACE;

        (label_start
            - self.header_leading_space()
            - tokens::component::icon_button::CONTAINER_WIDTH)
            .max(0.0)
    }

    #[cfg(test)]
    fn expanded_icon_center_x(self) -> f32 {
        tokens::component::navigation_rail::EXPANDED_ACTIVE_INDICATOR_MARGIN_HORIZONTAL
            + tokens::component::navigation_rail::EXPANDED_ACTIVE_INDICATOR_PADDING_START
            + tokens::component::navigation_rail::ICON_SIZE / 2.0
    }

    #[cfg(test)]
    fn expanded_icon_center_y(self) -> f32 {
        self.item_vertical_inset() + self.indicator_height() / 2.0
    }

    fn icon_anchor_width() -> f32 {
        tokens::component::navigation_rail::EXPANDED_ACTIVE_INDICATOR_PADDING_START
            + tokens::component::navigation_rail::ICON_SIZE
    }

    fn expanded_item_vertical_inset() -> f32 {
        ((tokens::component::navigation_rail::ITEM_HEIGHT
            - tokens::component::navigation_rail::EXPANDED_ACTIVE_INDICATOR_HEIGHT)
            / 2.0)
            .max(0.0)
    }
}

pub fn expanded_rail_width(progress: f32) -> f32 {
    lerp(
        tokens::component::navigation_rail::CONTAINER_WIDTH,
        tokens::component::navigation_rail::EXPANDED_CONTAINER_WIDTH,
        progress.clamp(0.0, 1.0),
    )
}

#[derive(Debug, Clone, Copy)]
struct DrawerMetrics {
    width: f32,
}

impl DrawerMetrics {
    fn new(width: f32) -> Self {
        Self {
            width: width.clamp(
                tokens::component::navigation_drawer::MINIMUM_CONTAINER_WIDTH,
                tokens::component::navigation_drawer::CONTAINER_WIDTH,
            ),
        }
    }

    fn width(self) -> f32 {
        self.width
    }

    fn indicator_width(self) -> f32 {
        (self.width - tokens::component::navigation_drawer::ITEM_HORIZONTAL_PADDING * 2.0).max(0.0)
    }

    fn menu_header_leading_space() -> f32 {
        tokens::component::navigation_drawer::ITEM_HORIZONTAL_PADDING
            + tokens::component::navigation_drawer::ITEM_CONTENT_LEADING_SPACE
            - (tokens::component::icon_button::CONTAINER_WIDTH
                - tokens::component::navigation_drawer::ICON_SIZE)
                / 2.0
    }

    fn menu_header_title_spacing() -> f32 {
        let label_start = tokens::component::navigation_drawer::ITEM_HORIZONTAL_PADDING
            + tokens::component::navigation_drawer::ITEM_CONTENT_LEADING_SPACE
            + tokens::component::navigation_drawer::ICON_SIZE
            + tokens::component::navigation_drawer::ICON_LABEL_SPACE;

        (label_start
            - Self::menu_header_leading_space()
            - tokens::component::icon_button::CONTAINER_WIDTH)
            .max(0.0)
    }

    fn badge_space() -> f32 {
        tokens::component::navigation_drawer::LABEL_BADGE_SPACE
    }
}

fn expanded_rail_icon_layer<'a, Message, Renderer>(
    icon: &'static str,
    progress: f32,
    height: f32,
    badge: Option<Badge>,
) -> Container<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'a,
    Font: Into<Renderer::Font>,
{
    let icon = destination_icon_anchor::<Message, Renderer>(
        icon,
        tokens::component::navigation_rail::ICON_SIZE,
        progress,
        badge,
        true,
    )
    .width(Length::Fixed(tokens::component::navigation_rail::ICON_SIZE))
    .height(Length::Fixed(height));

    Container::new(icon)
        .width(Length::Fixed(ExpandedRailMetrics::icon_anchor_width()))
        .height(Length::Fixed(height))
        .align_x(alignment::Horizontal::Right)
        .align_y(alignment::Vertical::Center)
}

fn collapsed_rail_label<'a, Message, Renderer>(
    label: &'static str,
    progress: f32,
    alpha: f32,
) -> Container<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'a,
    Font: Into<Renderer::Font>,
{
    let alpha = alpha.clamp(0.0, 1.0);
    let scale = tokens::component::navigation_rail::LABEL_TEXT;
    let label = type_text(label, scale).style(move |theme| text::Style {
        color: Some(alpha_color(bar_or_rail_label_color(theme, progress), alpha)),
    });

    Container::new(label)
        .padding(Padding {
            top: RailMetrics::collapsed_label_top_padding(),
            right: 0.0,
            bottom: 0.0,
            left: 0.0,
        })
        .align_x(alignment::Horizontal::Center)
}

fn destination_icon_anchor<'a, Message, Renderer>(
    icon: &'static str,
    size: f32,
    progress: f32,
    badge: Option<Badge>,
    drawer: bool,
) -> Container<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'a,
    Font: Into<Renderer::Font>,
{
    let icon: Element<'a, Message, Theme, Renderer> =
        destination_icon::<Message, Renderer>(icon, size, progress, drawer).into();
    let anchor = if let Some(badge) = badge {
        badge_widget::badged_box(
            icon,
            destination_badge::<Message, Renderer>(badge),
            destination_badge_placement(badge),
        )
        .into()
    } else {
        icon
    };

    Container::new(anchor)
        .align_x(alignment::Horizontal::Center)
        .align_y(alignment::Vertical::Center)
}

fn destination_badge<'a, Message, Renderer>(badge: Badge) -> Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'a,
    Font: Into<Renderer::Font>,
{
    match badge {
        Badge::Small => badge_widget::small().into(),
        Badge::Large(label) => badge_widget::large(label).into(),
    }
}

fn destination_badge_with_alpha<'a, Message, Renderer>(
    badge: Badge,
    alpha: f32,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'a,
    Font: Into<Renderer::Font>,
{
    let alpha = alpha.clamp(0.0, 1.0);

    match badge {
        Badge::Small => badge_widget::small()
            .style(move |theme| alpha_badge_style(theme, alpha))
            .into(),
        Badge::Large(label) => badge_widget::large(label)
            .style(move |theme| alpha_badge_style(theme, alpha))
            .into(),
    }
}

fn alpha_badge_style(theme: &Theme, alpha: f32) -> iced_widget::container::Style {
    let mut style = crate::style::badge::default(theme);

    if let Some(Background::Color(color)) = style.background {
        style.background = Some(Background::Color(alpha_color(color, alpha)));
    }

    style.text_color = style.text_color.map(|color| alpha_color(color, alpha));
    style
}

fn destination_badge_placement(badge: Badge) -> badge_widget::BadgedBoxPlacement {
    match badge {
        Badge::Small => badge_widget::BadgedBoxPlacement::IconOnly,
        Badge::Large(_) => badge_widget::BadgedBoxPlacement::WithContent,
    }
}

fn destination_icon<'a, Message, Renderer>(
    icon: &'static str,
    size: f32,
    progress: f32,
    drawer: bool,
) -> Stack<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: geometry::Renderer + primitive::Renderer + core_text::Renderer + 'a,
    Font: Into<Renderer::Font>,
{
    let outline = fonts::icon(icon, size)
        .width(Length::Fixed(size))
        .height(Length::Fixed(size))
        .center()
        .style(move |theme| text::Style {
            color: Some(destination_icon_outline_color(theme, progress)),
        });
    let filled = fonts::filled_icon(icon, size)
        .width(Length::Fixed(size))
        .height(Length::Fixed(size))
        .center()
        .style(move |theme| text::Style {
            color: Some(destination_icon_filled_color(theme, progress, drawer)),
        });

    Stack::new()
        .width(Length::Fixed(size))
        .height(Length::Fixed(size))
        .push(outline)
        .push(filled)
}

fn destination_icon_outline_color(theme: &Theme, progress: f32) -> Color {
    alpha_color(
        theme.colors().surface.text_variant,
        1.0 - progress.clamp(0.0, 1.0),
    )
}

fn destination_icon_filled_color(theme: &Theme, progress: f32, drawer: bool) -> Color {
    let color = if drawer {
        drawer_content_color(theme, 1.0)
    } else {
        bar_or_rail_icon_color(theme, 1.0)
    };

    alpha_color(color, progress.clamp(0.0, 1.0))
}

fn type_text<'a, Renderer>(
    content: &'static str,
    scale: tokens::typography::TypeScale,
) -> Text<'a, Theme, Renderer>
where
    Renderer: core_text::Renderer + 'a,
    Font: Into<Renderer::Font>,
{
    Text::new(content)
        .font(fonts::roboto_for_type_scale(scale))
        .size(scale.size)
        .line_height(LineHeight::Absolute(scale.line_height.into()))
}

fn single_line_type_text<'a, Renderer>(
    content: &'static str,
    scale: tokens::typography::TypeScale,
) -> Text<'a, Theme, Renderer>
where
    Renderer: core_text::Renderer + 'a,
    Font: Into<Renderer::Font>,
{
    type_text(content, scale).wrapping(text::Wrapping::None)
}

fn bar_container(theme: &Theme) -> iced_widget::container::Style {
    let colors = theme.colors();

    iced_widget::container::Style {
        background: Some(Background::Color(colors.surface.color)),
        text_color: Some(colors.surface.text),
        border: border::rounded(tokens::shape::CORNER_NONE),
        shadow: shadow_from_level(
            tokens::component::navigation_bar::CONTAINER_ELEVATION_LEVEL,
            colors.shadow,
        ),
        ..iced_widget::container::Style::default()
    }
}

fn rail_container(theme: &Theme) -> iced_widget::container::Style {
    let colors = theme.colors();

    iced_widget::container::Style {
        background: Some(Background::Color(colors.surface.color)),
        text_color: Some(colors.surface.text),
        border: border::rounded(tokens::shape::CORNER_NONE),
        shadow: shadow_from_level(
            tokens::component::navigation_rail::CONTAINER_ELEVATION_LEVEL,
            colors.shadow,
        ),
        ..iced_widget::container::Style::default()
    }
}

fn drawer_container(theme: &Theme) -> iced_widget::container::Style {
    let colors = theme.colors();

    iced_widget::container::Style {
        background: Some(Background::Color(colors.surface.color)),
        text_color: Some(colors.surface.text),
        border: border::rounded(tokens::shape::CORNER_LARGE),
        shadow: shadow_from_level(
            tokens::component::navigation_drawer::STANDARD_CONTAINER_ELEVATION_LEVEL,
            colors.shadow,
        ),
        ..iced_widget::container::Style::default()
    }
}

fn active_indicator(theme: &Theme, alpha: f32) -> iced_widget::container::Style {
    let mut color = theme.colors().secondary.container;
    color.a *= alpha.clamp(0.0, 1.0);

    iced_widget::container::Style {
        background: Some(Background::Color(color)),
        text_color: Some(theme.colors().secondary.container_text),
        border: border::rounded(tokens::shape::CORNER_FULL),
        ..iced_widget::container::Style::default()
    }
}

fn headline_text_style(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(theme.colors().surface.text_variant),
    }
}

fn bar_or_rail_icon_color(theme: &Theme, progress: f32) -> Color {
    let colors = theme.colors();

    mix(
        colors.surface.text_variant,
        colors.secondary.container_text,
        progress,
    )
}

fn bar_or_rail_label_color(theme: &Theme, progress: f32) -> Color {
    let colors = theme.colors();

    mix(colors.surface.text_variant, colors.surface.text, progress)
}

fn drawer_content_color(theme: &Theme, progress: f32) -> Color {
    let colors = theme.colors();

    mix(
        colors.surface.text_variant,
        colors.secondary.container_text,
        progress,
    )
}

fn layer_color(theme: &Theme, layer: NavigationStateLayer) -> Color {
    let colors = theme.colors();

    match layer {
        // AndroidX Material3 applies the selectable interaction to the whole item, but remaps the
        // ripple onto the active-indicator pill. Its ripple color comes from the navigation
        // container content color, not from the active icon color.
        NavigationStateLayer::BarOrRail => colors.surface.text,
        NavigationStateLayer::Drawer { progress } => mix(
            colors.surface.text,
            colors.secondary.container_text,
            progress,
        ),
    }
}

#[cfg(test)]
#[path = "../../../tests/widget/component/navigation.rs"]
mod tests;
