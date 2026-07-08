//! Material 3 switch/toggler constructors with token-backed size and motion defaults.

use super::*;

type StyleFn<'a> = Box<dyn Fn(&Theme, iced_toggler::Status) -> iced_toggler::Style + 'a>;

/// A Material 3 switch with animated handle motion and color transitions.
pub struct Toggler<'a, Message, Renderer = iced_widget::Renderer>
where
    Renderer: core_text::Renderer,
{
    is_toggled: bool,
    on_toggle: Option<Box<dyn Fn(bool) -> Message + 'a>>,
    on_toggle_with_origin: Option<Box<dyn Fn(bool, Point) -> Message + 'a>>,
    label: Option<text::Fragment<'a>>,
    width: Length,
    track_height: f32,
    spacing: f32,
    text_size: Option<Pixels>,
    text_line_height: LineHeight,
    text_alignment: text::Alignment,
    text_shaping: text::Shaping,
    text_wrapping: text::Wrapping,
    font: Option<Renderer::Font>,
    icons: bool,
    show_only_selected_icon: bool,
    style: StyleFn<'a>,
}

impl<Message, Renderer> std::fmt::Debug for Toggler<'_, Message, Renderer>
where
    Renderer: core_text::Renderer,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Toggler")
            .field("is_toggled", &self.is_toggled)
            .field("has_on_toggle", &self.on_toggle.is_some())
            .field(
                "has_on_toggle_with_origin",
                &self.on_toggle_with_origin.is_some(),
            )
            .field("has_label", &self.label.is_some())
            .field("width", &self.width)
            .field("track_height", &self.track_height)
            .field("spacing", &self.spacing)
            .field("text_size", &self.text_size)
            .field("text_line_height", &self.text_line_height)
            .field("text_alignment", &self.text_alignment)
            .field("icons", &self.icons)
            .field("show_only_selected_icon", &self.show_only_selected_icon)
            .finish_non_exhaustive()
    }
}

impl<'a, Message, Renderer> Toggler<'a, Message, Renderer>
where
    Renderer: core_text::Renderer,
{
    pub fn new(is_toggled: bool) -> Self {
        Self {
            is_toggled,
            on_toggle: None,
            on_toggle_with_origin: None,
            label: None,
            width: Length::Shrink,
            track_height: tokens::component::switch::TRACK_HEIGHT,
            spacing: f32::from(tokens::component::divider::LIST_ITEM_LEADING_SPACE),
            text_size: Some(Pixels(tokens::component::switch::LABEL_TEXT_SIZE)),
            text_line_height: absolute_line_height(
                tokens::component::switch::LABEL_TEXT_LINE_HEIGHT,
            ),
            text_alignment: text::Alignment::Default,
            text_shaping: text::Shaping::default(),
            text_wrapping: text::Wrapping::default(),
            font: None,
            icons: false,
            show_only_selected_icon: false,
            style: Box::new(toggler_style::default),
        }
    }

    pub fn label(mut self, label: impl text::IntoFragment<'a>) -> Self {
        self.label = Some(label.into_fragment());
        self
    }

    pub fn on_toggle(mut self, on_toggle: impl Fn(bool) -> Message + 'a) -> Self {
        self.on_toggle = Some(Box::new(on_toggle));
        self.on_toggle_with_origin = None;
        self
    }

    pub fn on_toggle_with_origin(
        mut self,
        on_toggle: impl Fn(bool, Point) -> Message + 'a,
    ) -> Self {
        self.on_toggle = None;
        self.on_toggle_with_origin = Some(Box::new(on_toggle));
        self
    }

    pub fn on_toggle_maybe(mut self, on_toggle: Option<impl Fn(bool) -> Message + 'a>) -> Self {
        self.on_toggle = on_toggle.map(|on_toggle| Box::new(on_toggle) as _);
        self.on_toggle_with_origin = None;
        self
    }

    pub fn size(mut self, size: impl Into<Pixels>) -> Self {
        self.track_height = size.into().0;
        self
    }

    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    pub fn spacing(mut self, spacing: impl Into<Pixels>) -> Self {
        self.spacing = spacing.into().0;
        self
    }

    pub fn text_size(mut self, text_size: impl Into<Pixels>) -> Self {
        self.text_size = Some(text_size.into());
        self
    }

    pub fn text_line_height(mut self, line_height: impl Into<LineHeight>) -> Self {
        self.text_line_height = line_height.into();
        self
    }

    pub fn text_alignment(mut self, alignment: impl Into<text::Alignment>) -> Self {
        self.text_alignment = alignment.into();
        self
    }

    pub fn text_shaping(mut self, shaping: text::Shaping) -> Self {
        self.text_shaping = shaping;
        self
    }

    pub fn text_wrapping(mut self, wrapping: text::Wrapping) -> Self {
        self.text_wrapping = wrapping;
        self
    }

    pub fn font(mut self, font: impl Into<Renderer::Font>) -> Self {
        self.font = Some(font.into());
        self
    }

    pub fn icons(mut self, icons: bool) -> Self {
        self.icons = icons;
        self
    }

    pub fn show_only_selected_icon(mut self, show_only_selected_icon: bool) -> Self {
        self.show_only_selected_icon = show_only_selected_icon;
        self
    }

    pub fn style(
        mut self,
        style: impl Fn(&Theme, iced_toggler::Status) -> iced_toggler::Style + 'a,
    ) -> Self {
        self.style = Box::new(style);
        self
    }
}

impl<Message, Renderer> Toggler<'_, Message, Renderer>
where
    Renderer: core_text::Renderer,
{
    fn track_size(&self) -> Size {
        let scale = self.track_height / tokens::component::switch::TRACK_HEIGHT;

        Size::new(
            tokens::component::switch::TRACK_WIDTH * scale,
            self.track_height,
        )
    }

    fn handle_size_for(&self, is_toggled: bool, is_pressed: bool) -> f32 {
        if is_pressed {
            tokens::component::switch::PRESSED_HANDLE_SIZE
        } else if self.icons || (self.show_only_selected_icon && is_toggled) {
            tokens::component::switch::WITH_ICON_HANDLE_SIZE
        } else if is_toggled {
            tokens::component::switch::SELECTED_HANDLE_SIZE
        } else {
            tokens::component::switch::UNSELECTED_HANDLE_SIZE
        }
    }

    fn shows_icons(&self) -> bool {
        self.icons || self.show_only_selected_icon
    }

    fn shows_off_icon(&self) -> bool {
        self.icons && !self.show_only_selected_icon
    }

    fn current_status(&self, bounds: Rectangle, cursor: mouse::Cursor) -> iced_toggler::Status {
        if !self.has_on_toggle() {
            iced_toggler::Status::Disabled {
                is_toggled: self.is_toggled,
            }
        } else if cursor.is_over(bounds) {
            iced_toggler::Status::Hovered {
                is_toggled: self.is_toggled,
            }
        } else {
            iced_toggler::Status::Active {
                is_toggled: self.is_toggled,
            }
        }
    }

    fn has_on_toggle(&self) -> bool {
        self.on_toggle.is_some() || self.on_toggle_with_origin.is_some()
    }

    fn toggle_message(&self, is_toggled: bool, origin: Point) -> Option<Message> {
        if let Some(on_toggle) = &self.on_toggle_with_origin {
            Some((on_toggle)(is_toggled, origin))
        } else {
            self.on_toggle
                .as_ref()
                .map(|on_toggle| (on_toggle)(is_toggled))
        }
    }
}

fn toggler_event_origin(event: &Event, bounds: Rectangle, cursor: mouse::Cursor) -> Point {
    cursor
        .position()
        .or(match event {
            Event::Touch(
                touch::Event::FingerPressed { position, .. }
                | touch::Event::FingerLifted { position, .. },
            ) => Some(*position),
            _ => None,
        })
        .unwrap_or_else(|| bounds.center())
}

impl<Message, Renderer> Widget<Message, Theme, Renderer> for Toggler<'_, Message, Renderer>
where
    Renderer: core_text::Renderer + core_svg::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<SelectionState<Renderer::Paragraph, iced_toggler::Status>>()
    }

    fn state(&self) -> tree::State {
        let mut state =
            SelectionState::<Renderer::Paragraph, iced_toggler::Status>::new(self.is_toggled);

        state.size = AnimatedScalar::new(self.handle_size_for(self.is_toggled, false));

        tree::State::new(state)
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: Length::Shrink,
        }
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let track_size = self.track_size();

        layout::next_to_each_other(
            &limits.width(self.width),
            if self.label.is_some() {
                self.spacing
            } else {
                0.0
            },
            |_| layout::Node::new(track_size),
            |limits| {
                if let Some(label) = self.label.as_deref() {
                    let state = tree
                        .state
                        .downcast_mut::<SelectionState<Renderer::Paragraph, iced_toggler::Status>>(
                        );

                    core_widget::text::layout(
                        &mut state.text,
                        renderer,
                        limits,
                        label,
                        core_widget::text::Format {
                            width: self.width,
                            height: Length::Shrink,
                            line_height: self.text_line_height,
                            size: self.text_size,
                            font: self.font,
                            align_x: self.text_alignment,
                            align_y: alignment::Vertical::Top,
                            shaping: self.text_shaping,
                            wrapping: self.text_wrapping,
                        },
                    )
                } else {
                    layout::Node::new(Size::ZERO)
                }
            },
        )
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let state = tree
            .state
            .downcast_mut::<SelectionState<Renderer::Paragraph, iced_toggler::Status>>();
        let hit_bounds =
            selection_control_hit_bounds(layout, tokens::component::switch::STATE_LAYER_SIZE);

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. })
                if self.has_on_toggle() && press_is_over(event, hit_bounds, cursor) =>
            {
                state.is_pressed = true;
                state.press_origin = Some(toggler_event_origin(event, layout.bounds(), cursor));
                shell.capture_event();
                shell.request_redraw();
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. })
                if state.is_pressed =>
            {
                let is_released_over = release_is_over(event, hit_bounds, cursor);
                let origin = state
                    .press_origin
                    .unwrap_or_else(|| toggler_event_origin(event, layout.bounds(), cursor));

                state.is_pressed = false;
                state.press_origin = None;

                if is_released_over
                    && let Some(message) = self.toggle_message(!self.is_toggled, origin)
                {
                    shell.publish(message);
                }

                shell.capture_event();
                shell.request_redraw();
            }
            Event::Touch(touch::Event::FingerLost { .. }) if state.is_pressed => {
                state.is_pressed = false;
                state.press_origin = None;
                shell.request_redraw();
            }
            _ => {}
        }

        let now = match event {
            Event::Window(window::Event::RedrawRequested(now)) => Some(*now),
            _ => None,
        };

        if state.target != self.is_toggled {
            let now = now.unwrap_or_else(Instant::now);

            state.target = self.is_toggled;
            state.position.set_target(
                bool_value(self.is_toggled),
                now,
                duration_ms(tokens::component::switch::HANDLE_POSITION_TRANSITION_DURATION_MS),
                tokens::component::switch::HANDLE_POSITION_TRANSITION_EASING,
            );
            state.color.set_target(
                bool_value(self.is_toggled),
                now,
                duration_ms(tokens::component::switch::TRACK_COLOR_TRANSITION_DURATION_MS),
                tokens::motion::EASING_LINEAR,
            );
            state.size.set_target(
                self.handle_size_for(self.is_toggled, state.is_pressed),
                now,
                if state.is_pressed {
                    duration_ms(
                        tokens::component::switch::PRESSED_HANDLE_SIZE_TRANSITION_DURATION_MS,
                    )
                } else {
                    duration_ms(tokens::component::switch::HANDLE_SIZE_TRANSITION_DURATION_MS)
                },
                if state.is_pressed {
                    tokens::motion::EASING_LINEAR
                } else {
                    tokens::motion::EASING_STANDARD
                },
            );
            state.icon.set_target(
                bool_value(self.is_toggled),
                now,
                duration_ms(tokens::component::switch::ICON_TRANSFORM_TRANSITION_DURATION_MS),
                tokens::motion::EASING_STANDARD,
            );
            state.icon_opacity.set_target(
                bool_value(self.is_toggled),
                now,
                duration_ms(tokens::component::switch::ICON_OPACITY_TRANSITION_DURATION_MS),
                tokens::motion::EASING_LINEAR,
            );
            shell.request_redraw();
        }

        let target_handle_size = self.handle_size_for(self.is_toggled, state.is_pressed);

        if (state.size.to - target_handle_size).abs() > f32::EPSILON {
            let now = now.unwrap_or_else(Instant::now);

            state.size.set_target(
                target_handle_size,
                now,
                if state.is_pressed {
                    duration_ms(
                        tokens::component::switch::PRESSED_HANDLE_SIZE_TRANSITION_DURATION_MS,
                    )
                } else {
                    duration_ms(tokens::component::switch::HANDLE_SIZE_TRANSITION_DURATION_MS)
                },
                if state.is_pressed {
                    tokens::motion::EASING_LINEAR
                } else {
                    tokens::motion::EASING_STANDARD
                },
            );
            shell.request_redraw();
        }

        let current_status = self.current_status(hit_bounds, cursor);

        if let Some(now) = now {
            if state.advance(now) {
                shell.request_redraw();
            }

            state.last_status = Some(current_status);
        } else if state
            .last_status
            .is_some_and(|status| status != current_status)
            || state.is_animating()
        {
            shell.request_redraw();
        }
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        let hit_bounds =
            selection_control_hit_bounds(layout, tokens::component::switch::STATE_LAYER_SIZE);

        if cursor.is_over(hit_bounds) {
            if self.has_on_toggle() {
                mouse::Interaction::Pointer
            } else {
                mouse::Interaction::NotAllowed
            }
        } else {
            mouse::Interaction::default()
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        defaults: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree
            .state
            .downcast_ref::<SelectionState<Renderer::Paragraph, iced_toggler::Status>>();

        let mut children = layout.children();
        let toggler_layout = children.next().unwrap();
        let bounds = toggler_layout.bounds();

        let status = state.last_status.unwrap_or(iced_toggler::Status::Disabled {
            is_toggled: self.is_toggled,
        });
        let unselected_status = match status {
            iced_toggler::Status::Active { .. } => {
                iced_toggler::Status::Active { is_toggled: false }
            }
            iced_toggler::Status::Hovered { .. } => {
                iced_toggler::Status::Hovered { is_toggled: false }
            }
            iced_toggler::Status::Disabled { .. } => {
                iced_toggler::Status::Disabled { is_toggled: false }
            }
        };
        let selected_status = match status {
            iced_toggler::Status::Active { .. } => {
                iced_toggler::Status::Active { is_toggled: true }
            }
            iced_toggler::Status::Hovered { .. } => {
                iced_toggler::Status::Hovered { is_toggled: true }
            }
            iced_toggler::Status::Disabled { .. } => {
                iced_toggler::Status::Disabled { is_toggled: true }
            }
        };

        let current_style = (self.style)(theme, status);
        let unselected_style = (self.style)(theme, unselected_status);
        let selected_style = (self.style)(theme, selected_status);

        let color_progress = state.color.value.clamp(0.0, 1.0);
        let scale = bounds.height / tokens::component::switch::TRACK_HEIGHT;
        let colors = theme.colors();
        let is_disabled = matches!(status, iced_toggler::Status::Disabled { .. });
        let track_radius = current_style
            .border_radius
            .unwrap_or_else(|| border::Radius::new(bounds.height / 2.0));
        let track_border_width = lerp(
            unselected_style.background_border_width,
            selected_style.background_border_width,
            color_progress,
        );
        let track_border_color = mix(
            unselected_style.background_border_color,
            selected_style.background_border_color,
            color_progress,
        );
        let track_background = mix(
            solid_color(unselected_style.background),
            solid_color(selected_style.background),
            color_progress,
        );

        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: Border {
                    radius: track_radius,
                    width: track_border_width,
                    color: track_border_color,
                },
                ..renderer::Quad::default()
            },
            track_background,
        );

        let handle_size = state.size.value * scale;
        let center_x = bounds.x
            + scale
                * (tokens::component::switch::TRACK_HEIGHT / 2.0
                    + (tokens::component::switch::TRACK_WIDTH
                        - tokens::component::switch::TRACK_HEIGHT)
                        * state.position.value);
        let center_y = bounds.center_y();
        let handle_bounds = Rectangle {
            x: center_x - handle_size / 2.0,
            y: center_y - handle_size / 2.0,
            width: handle_size,
            height: handle_size,
        };
        let handle_background = mix(
            solid_color(unselected_style.foreground),
            solid_color(selected_style.foreground),
            color_progress,
        );
        let handle_border_width = lerp(
            unselected_style.foreground_border_width,
            selected_style.foreground_border_width,
            color_progress,
        );
        let handle_border_color = mix(
            unselected_style.foreground_border_color,
            selected_style.foreground_border_color,
            color_progress,
        );

        let state_layer_opacity = if is_disabled {
            0.0
        } else if state.is_pressed {
            tokens::state::PRESSED_STATE_LAYER_OPACITY
        } else if matches!(status, iced_toggler::Status::Hovered { .. }) {
            tokens::state::HOVER_STATE_LAYER_OPACITY
        } else {
            0.0
        };

        if state_layer_opacity > 0.0 {
            let state_layer_size = tokens::component::switch::STATE_LAYER_SIZE * scale;
            let state_layer_color = if self.is_toggled {
                colors.primary.color
            } else {
                colors.surface.text
            };

            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle {
                        x: center_x - state_layer_size / 2.0,
                        y: center_y - state_layer_size / 2.0,
                        width: state_layer_size,
                        height: state_layer_size,
                    },
                    border: Border {
                        radius: border::Radius::new(state_layer_size / 2.0),
                        ..Border::default()
                    },
                    ..renderer::Quad::default()
                },
                alpha_color(state_layer_color, state_layer_opacity),
            );
        }

        renderer.fill_quad(
            renderer::Quad {
                bounds: handle_bounds,
                border: Border {
                    radius: border::Radius::new(handle_size / 2.0),
                    width: handle_border_width,
                    color: handle_border_color,
                },
                ..renderer::Quad::default()
            },
            handle_background,
        );

        if self.shows_icons() {
            let icon_progress = state.icon.value.clamp(0.0, 1.0);
            let selected_icon_opacity = state.icon_opacity.value.clamp(0.0, 1.0);
            let unselected_icon_opacity = if self.shows_off_icon() {
                1.0 - selected_icon_opacity
            } else {
                0.0
            };
            let selected_icon_color = if is_disabled {
                alpha_color(
                    colors.surface.text,
                    tokens::component::switch::DISABLED_SELECTED_ICON_OPACITY,
                )
            } else {
                colors.primary.container_text
            };
            let unselected_icon_color = if is_disabled {
                alpha_color(
                    colors.surface.container.highest,
                    tokens::component::switch::DISABLED_UNSELECTED_ICON_OPACITY,
                )
            } else {
                colors.surface.container.highest
            };

            if selected_icon_opacity > 0.0 {
                let icon_scale = 0.82 + 0.18 * icon_progress;
                let icon_size = tokens::component::switch::SELECTED_ICON_SIZE * scale * icon_scale;

                renderer.draw_svg(
                    core_svg::Svg::new(core_svg::Handle::from_memory(SWITCH_ON_ICON_SVG))
                        .color(selected_icon_color)
                        .opacity(selected_icon_opacity),
                    scaled_rect(handle_bounds, icon_size, icon_size),
                    *viewport,
                );
            }

            if unselected_icon_opacity > 0.0 {
                let icon_size = tokens::component::switch::UNSELECTED_ICON_SIZE * scale;

                renderer.draw_svg(
                    core_svg::Svg::new(core_svg::Handle::from_memory(SWITCH_OFF_ICON_SVG))
                        .color(unselected_icon_color)
                        .opacity(unselected_icon_opacity),
                    scaled_rect(handle_bounds, icon_size, icon_size),
                    *viewport,
                );
            }
        }

        if self.label.is_none() {
            return;
        }

        let label_layout = children.next().unwrap();

        core_widget::text::draw(
            renderer,
            defaults,
            label_layout.bounds(),
            state.text.raw(),
            core_widget::text::Style {
                color: current_style.text_color,
            },
            viewport,
        );
    }

    fn operate(
        &mut self,
        _tree: &mut Tree,
        layout: Layout<'_>,
        _renderer: &Renderer,
        operation: &mut dyn core_widget::Operation,
    ) {
        if let Some(label) = self.label.as_deref() {
            operation.text(None, layout.bounds(), label);
        }
    }
}

impl<'a, Message, Renderer> From<Toggler<'a, Message, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: core_text::Renderer + core_svg::Renderer + 'a,
{
    fn from(toggler: Toggler<'a, Message, Renderer>) -> Self {
        Element::new(toggler)
    }
}

pub fn control<'a, Message, Renderer>(is_toggled: bool) -> Toggler<'a, Message, Renderer>
where
    Renderer: core_text::Renderer + core_svg::Renderer + 'a,
{
    Toggler::new(is_toggled)
        .size(tokens::component::switch::TRACK_HEIGHT)
        .spacing(f32::from(
            tokens::component::divider::LIST_ITEM_LEADING_SPACE,
        ))
        .text_size(tokens::component::switch::LABEL_TEXT_SIZE)
        .text_line_height(absolute_line_height(
            tokens::component::switch::LABEL_TEXT_LINE_HEIGHT,
        ))
        .show_only_selected_icon(true)
        .style(toggler_style::default)
}

pub fn standard<'a, Message, Renderer>(
    is_toggled: bool,
    label: impl text::IntoFragment<'a>,
    on_toggle: impl Fn(bool) -> Message + 'a,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + core_text::Renderer + core_svg::Renderer + 'a,
{
    Container::new(control(is_toggled).label(label).on_toggle(on_toggle))
        .center_y(Length::Fixed(tokens::component::switch::STATE_LAYER_SIZE))
        .into()
}

pub fn standard_with_origin<'a, Message, Renderer>(
    is_toggled: bool,
    label: impl text::IntoFragment<'a>,
    on_toggle: impl Fn(bool, Point) -> Message + 'a,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + core_text::Renderer + core_svg::Renderer + 'a,
{
    Container::new(
        control(is_toggled)
            .label(label)
            .on_toggle_with_origin(on_toggle),
    )
    .center_y(Length::Fixed(tokens::component::switch::STATE_LAYER_SIZE))
    .into()
}
