//! Material 3 radio constructors with token-backed size and motion defaults.

use super::*;

type StyleFn<'a> = Box<dyn Fn(&Theme, iced_radio::Status) -> iced_radio::Style + 'a>;

/// A Material 3 radio button with animated selected state transitions.
pub struct Radio<'a, Message, Renderer = iced_widget::Renderer>
where
    Renderer: core_text::Renderer,
{
    is_selected: bool,
    on_click: Option<Message>,
    label: String,
    width: Length,
    size: f32,
    spacing: f32,
    text_size: Option<Pixels>,
    text_line_height: LineHeight,
    text_shaping: text::Shaping,
    text_wrapping: text::Wrapping,
    font: Option<Renderer::Font>,
    style: StyleFn<'a>,
}

impl<Message, Renderer> std::fmt::Debug for Radio<'_, Message, Renderer>
where
    Message: std::fmt::Debug,
    Renderer: core_text::Renderer,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Radio")
            .field("is_selected", &self.is_selected)
            .field("on_click", &self.on_click)
            .field("label", &self.label)
            .field("width", &self.width)
            .field("size", &self.size)
            .field("spacing", &self.spacing)
            .field("text_size", &self.text_size)
            .field("text_line_height", &self.text_line_height)
            .finish_non_exhaustive()
    }
}

impl<'a, Message, Renderer> Radio<'a, Message, Renderer>
where
    Message: Clone,
    Renderer: core_text::Renderer,
{
    pub fn new<F, V>(label: impl Into<String>, value: V, selected: Option<V>, on_select: F) -> Self
    where
        V: Eq + Copy,
        F: FnOnce(V) -> Message,
    {
        Self {
            is_selected: Some(value) == selected,
            on_click: Some(on_select(value)),
            label: label.into(),
            width: Length::Shrink,
            size: tokens::component::radio::ICON_SIZE,
            spacing: f32::from(tokens::component::divider::LIST_ITEM_LEADING_SPACE),
            text_size: Some(Pixels(tokens::component::radio::LABEL_TEXT_SIZE)),
            text_line_height: absolute_line_height(
                tokens::component::radio::LABEL_TEXT_LINE_HEIGHT,
            ),
            text_shaping: text::Shaping::default(),
            text_wrapping: text::Wrapping::default(),
            font: None,
            style: Box::new(crate::style::radio::default),
        }
    }

    pub fn on_select_maybe(mut self, on_click: Option<Message>) -> Self {
        self.on_click = on_click;
        self
    }

    pub fn size(mut self, size: impl Into<Pixels>) -> Self {
        self.size = size.into().0;
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

    pub fn style(
        mut self,
        style: impl Fn(&Theme, iced_radio::Status) -> iced_radio::Style + 'a,
    ) -> Self {
        self.style = Box::new(style);
        self
    }
}

impl<Message, Renderer> Radio<'_, Message, Renderer>
where
    Renderer: core_text::Renderer,
{
    fn current_status(&self, bounds: Rectangle, cursor: mouse::Cursor) -> iced_radio::Status {
        if cursor.is_over(bounds) && self.on_click.is_some() {
            iced_radio::Status::Hovered {
                is_selected: self.is_selected,
            }
        } else {
            iced_radio::Status::Active {
                is_selected: self.is_selected,
            }
        }
    }

    fn state_layer_color(
        &self,
        theme: &Theme,
        state: &SelectionState<Renderer::Paragraph, iced_radio::Status>,
        status: iced_radio::Status,
    ) -> Option<Color> {
        let is_hovered = matches!(status, iced_radio::Status::Hovered { .. });

        if !state.is_pressed && !is_hovered {
            return None;
        }

        let colors = theme.colors();
        let (color, opacity) = if self.is_selected {
            if state.is_pressed {
                (
                    colors.surface.text,
                    tokens::state::PRESSED_STATE_LAYER_OPACITY,
                )
            } else {
                (
                    colors.primary.color,
                    tokens::state::HOVER_STATE_LAYER_OPACITY,
                )
            }
        } else if state.is_pressed {
            (
                colors.primary.color,
                tokens::state::PRESSED_STATE_LAYER_OPACITY,
            )
        } else {
            (
                colors.surface.text,
                tokens::state::HOVER_STATE_LAYER_OPACITY,
            )
        };

        Some(alpha_color(color, opacity))
    }
}

impl<Message, Renderer> Widget<Message, Theme, Renderer> for Radio<'_, Message, Renderer>
where
    Message: Clone,
    Renderer: core_text::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<SelectionState<Renderer::Paragraph, iced_radio::Status>>()
    }

    fn state(&self) -> tree::State {
        let mut state =
            SelectionState::<Renderer::Paragraph, iced_radio::Status>::new(self.is_selected);

        state.size = AnimatedScalar::new(bool_value(self.is_selected));

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
        layout::next_to_each_other(
            &limits.width(self.width),
            self.spacing,
            |_| layout::Node::new(Size::new(self.size, self.size)),
            |limits| {
                let state = tree
                    .state
                    .downcast_mut::<SelectionState<Renderer::Paragraph, iced_radio::Status>>();

                core_widget::text::layout(
                    &mut state.text,
                    renderer,
                    limits,
                    &self.label,
                    core_widget::text::Format {
                        width: self.width,
                        height: Length::Shrink,
                        line_height: self.text_line_height,
                        size: self.text_size,
                        font: self.font,
                        align_x: text::Alignment::Default,
                        align_y: alignment::Vertical::Top,
                        shaping: self.text_shaping,
                        wrapping: self.text_wrapping,
                    },
                )
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
            .downcast_mut::<SelectionState<Renderer::Paragraph, iced_radio::Status>>();
        let hit_bounds =
            selection_control_hit_bounds(layout, tokens::component::radio::TARGET_SIZE);

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. })
                if self.on_click.is_some() && press_is_over(event, hit_bounds, cursor) =>
            {
                state.is_pressed = true;
                state.press_origin = None;
                shell.capture_event();
                shell.request_redraw();
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. })
                if state.is_pressed =>
            {
                let is_released_over = release_is_over(event, hit_bounds, cursor);

                state.is_pressed = false;
                state.press_origin = None;

                if is_released_over && let Some(on_click) = &self.on_click {
                    shell.publish(on_click.clone());
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

        if state.target != self.is_selected {
            let now = now.unwrap_or_else(Instant::now);

            state.target = self.is_selected;
            state.color.set_target(
                bool_value(self.is_selected),
                now,
                duration_ms(tokens::component::radio::ICON_COLOR_TRANSITION_DURATION_MS),
                tokens::motion::EASING_LINEAR,
            );

            if self.is_selected {
                state.size = AnimatedScalar::new(0.0);
                state.size.set_target(
                    1.0,
                    now,
                    duration_ms(tokens::component::radio::SELECT_TRANSITION_DURATION_MS),
                    tokens::component::radio::SELECT_TRANSITION_EASING,
                );
            } else {
                state.size = AnimatedScalar::new(1.0);
            }

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
            selection_control_hit_bounds(layout, tokens::component::radio::TARGET_SIZE);

        if cursor.is_over(hit_bounds) {
            if self.on_click.is_some() {
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
            .downcast_ref::<SelectionState<Renderer::Paragraph, iced_radio::Status>>();

        let mut children = layout.children();
        let control_layout = children.next().unwrap();
        let bounds = control_layout.bounds();
        let status = state.last_status.unwrap_or(iced_radio::Status::Active {
            is_selected: self.is_selected,
        });
        let unchecked_status = match status {
            iced_radio::Status::Active { .. } => iced_radio::Status::Active { is_selected: false },
            iced_radio::Status::Hovered { .. } => {
                iced_radio::Status::Hovered { is_selected: false }
            }
        };
        let checked_status = match status {
            iced_radio::Status::Active { .. } => iced_radio::Status::Active { is_selected: true },
            iced_radio::Status::Hovered { .. } => iced_radio::Status::Hovered { is_selected: true },
        };
        let current_style = (self.style)(theme, status);
        let unchecked_style = (self.style)(theme, unchecked_status);
        let checked_style = (self.style)(theme, checked_status);

        if let Some(layer_color) = self.state_layer_color(theme, state, status) {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: scaled_rect(
                        bounds,
                        tokens::component::radio::STATE_LAYER_SIZE,
                        tokens::component::radio::STATE_LAYER_SIZE,
                    ),
                    border: border::rounded(tokens::component::radio::STATE_LAYER_SIZE / 2.0),
                    ..renderer::Quad::default()
                },
                Background::Color(layer_color),
            );
        }

        let color_progress = state.color.value.clamp(0.0, 1.0);
        let icon_color = mix(
            unchecked_style.border_color,
            checked_style.border_color,
            color_progress,
        );
        let radius = bounds.width.min(bounds.height) / 2.0;

        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: Border {
                    radius: radius.into(),
                    width: tokens::component::radio::OUTER_RING_WIDTH,
                    color: icon_color,
                },
                ..renderer::Quad::default()
            },
            Color::TRANSPARENT,
        );

        if color_progress > 0.0 {
            let dot_size =
                tokens::component::radio::INNER_DOT_SIZE * state.size.value.clamp(0.0, 1.0);

            if dot_size > 0.0 {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: scaled_rect(bounds, dot_size, dot_size),
                        border: border::rounded(dot_size / 2.0),
                        ..renderer::Quad::default()
                    },
                    Background::Color(alpha_color(checked_style.dot_color, color_progress)),
                );
            }
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
        operation.text(None, layout.bounds(), &self.label);
    }
}

impl<'a, Message, Renderer> From<Radio<'a, Message, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: core_text::Renderer + 'a,
{
    fn from(radio: Radio<'a, Message, Renderer>) -> Self {
        Element::new(radio)
    }
}

pub fn control<'a, Message, Renderer, V>(
    label: impl Into<String>,
    value: V,
    selected: Option<V>,
    on_select: impl FnOnce(V) -> Message,
) -> Radio<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: core_text::Renderer + 'a,
    V: Eq + Copy,
{
    Radio::new(label, value, selected, on_select)
        .size(tokens::component::radio::ICON_SIZE)
        .spacing(f32::from(
            tokens::component::divider::LIST_ITEM_LEADING_SPACE,
        ))
        .text_size(tokens::component::radio::LABEL_TEXT_SIZE)
        .text_line_height(absolute_line_height(
            tokens::component::radio::LABEL_TEXT_LINE_HEIGHT,
        ))
        .style(crate::style::radio::default)
}

pub fn standard<'a, Message, Renderer, V>(
    label: impl Into<String>,
    value: V,
    selected: Option<V>,
    on_select: impl FnOnce(V) -> Message,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
    V: Eq + Copy,
{
    Container::new(control(label, value, selected, on_select))
        .center_y(Length::Fixed(tokens::component::radio::TARGET_SIZE))
        .into()
}
