//! Material 3 checkbox constructors with token-backed size and motion defaults.

use super::*;

type StyleFn<'a> = Box<dyn Fn(&Theme, iced_checkbox::Status) -> iced_checkbox::Style + 'a>;

/// A Material 3 checkbox with animated selected state transitions.
pub struct Checkbox<'a, Message, Renderer = iced_widget::Renderer>
where
    Renderer: core_text::Renderer,
{
    is_checked: bool,
    on_toggle: Option<Box<dyn Fn(bool) -> Message + 'a>>,
    label: Option<text::Fragment<'a>>,
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

impl<Message, Renderer> std::fmt::Debug for Checkbox<'_, Message, Renderer>
where
    Renderer: core_text::Renderer,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Checkbox")
            .field("is_checked", &self.is_checked)
            .field("has_on_toggle", &self.on_toggle.is_some())
            .field("has_label", &self.label.is_some())
            .field("width", &self.width)
            .field("size", &self.size)
            .field("spacing", &self.spacing)
            .field("text_size", &self.text_size)
            .field("text_line_height", &self.text_line_height)
            .finish_non_exhaustive()
    }
}

impl<'a, Message, Renderer> Checkbox<'a, Message, Renderer>
where
    Renderer: core_text::Renderer,
{
    pub fn new(is_checked: bool) -> Self {
        Self {
            is_checked,
            on_toggle: None,
            label: None,
            width: Length::Shrink,
            size: tokens::component::checkbox::CONTAINER_SIZE,
            spacing: f32::from(tokens::component::divider::LIST_ITEM_LEADING_SPACE),
            text_size: Some(Pixels(tokens::component::checkbox::LABEL_TEXT_SIZE)),
            text_line_height: absolute_line_height(
                tokens::component::checkbox::LABEL_TEXT_LINE_HEIGHT,
            ),
            text_shaping: text::Shaping::default(),
            text_wrapping: text::Wrapping::default(),
            font: None,
            style: Box::new(checkbox_style::default),
        }
    }

    pub fn label(mut self, label: impl text::IntoFragment<'a>) -> Self {
        self.label = Some(label.into_fragment());
        self
    }

    pub fn on_toggle(mut self, on_toggle: impl Fn(bool) -> Message + 'a) -> Self {
        self.on_toggle = Some(Box::new(on_toggle));
        self
    }

    pub fn on_toggle_maybe(mut self, on_toggle: Option<impl Fn(bool) -> Message + 'a>) -> Self {
        self.on_toggle = on_toggle.map(|on_toggle| Box::new(on_toggle) as _);
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
        style: impl Fn(&Theme, iced_checkbox::Status) -> iced_checkbox::Style + 'a,
    ) -> Self {
        self.style = Box::new(style);
        self
    }
}

impl<Message, Renderer> Checkbox<'_, Message, Renderer>
where
    Renderer: core_text::Renderer,
{
    fn current_status(&self, bounds: Rectangle, cursor: mouse::Cursor) -> iced_checkbox::Status {
        if self.on_toggle.is_none() {
            iced_checkbox::Status::Disabled {
                is_checked: self.is_checked,
            }
        } else if cursor.is_over(bounds) {
            iced_checkbox::Status::Hovered {
                is_checked: self.is_checked,
            }
        } else {
            iced_checkbox::Status::Active {
                is_checked: self.is_checked,
            }
        }
    }

    fn state_layer_color(
        &self,
        state: &SelectionState<Renderer::Paragraph, iced_checkbox::Status>,
        status: iced_checkbox::Status,
        unchecked_style: &iced_checkbox::Style,
        checked_style: &iced_checkbox::Style,
    ) -> Option<Color> {
        let is_hovered = matches!(status, iced_checkbox::Status::Hovered { .. });

        if !state.is_pressed && !is_hovered {
            return None;
        }

        let color = if self.is_checked {
            solid_color(checked_style.background)
        } else {
            unchecked_style.border.color
        };
        let opacity = if state.is_pressed {
            tokens::state::PRESSED_STATE_LAYER_OPACITY
        } else {
            tokens::state::HOVER_STATE_LAYER_OPACITY
        };

        Some(alpha_color(color, opacity))
    }
}

impl<Message, Renderer> Widget<Message, Theme, Renderer> for Checkbox<'_, Message, Renderer>
where
    Renderer: core_text::Renderer + core_svg::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<SelectionState<Renderer::Paragraph, iced_checkbox::Status>>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(
            SelectionState::<Renderer::Paragraph, iced_checkbox::Status>::new(self.is_checked),
        )
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
            if self.label.is_some() {
                self.spacing
            } else {
                0.0
            },
            |_| layout::Node::new(Size::new(self.size, self.size)),
            |limits| {
                if let Some(label) = self.label.as_deref() {
                    let state = tree
                        .state
                        .downcast_mut::<SelectionState<Renderer::Paragraph, iced_checkbox::Status>>(
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
                            align_x: text::Alignment::Default,
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
            .downcast_mut::<SelectionState<Renderer::Paragraph, iced_checkbox::Status>>();
        let hit_bounds =
            selection_control_hit_bounds(layout, tokens::component::checkbox::STATE_LAYER_SIZE);

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. })
                if self.on_toggle.is_some() && press_is_over(event, hit_bounds, cursor) =>
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

                if is_released_over && let Some(on_toggle) = &self.on_toggle {
                    shell.publish((on_toggle)(!self.is_checked));
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

        if state.target != self.is_checked {
            let now = now.unwrap_or_else(Instant::now);
            let (duration, easing) = if self.is_checked {
                (
                    duration_ms(tokens::component::checkbox::SELECT_TRANSITION_DURATION_MS),
                    tokens::component::checkbox::SELECT_TRANSITION_EASING,
                )
            } else {
                (
                    duration_ms(tokens::component::checkbox::UNSELECT_TRANSITION_DURATION_MS),
                    tokens::component::checkbox::UNSELECT_TRANSITION_EASING,
                )
            };

            state.target = self.is_checked;
            state
                .position
                .set_target(bool_value(self.is_checked), now, duration, easing);
            state.color.set_target(
                bool_value(self.is_checked),
                now,
                duration_ms(tokens::component::checkbox::OPACITY_TRANSITION_DURATION_MS),
                tokens::motion::EASING_LINEAR,
            );
            state
                .size
                .set_target(bool_value(self.is_checked), now, duration, easing);
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
            selection_control_hit_bounds(layout, tokens::component::checkbox::STATE_LAYER_SIZE);

        if cursor.is_over(hit_bounds) && self.on_toggle.is_some() {
            mouse::Interaction::Pointer
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
            .downcast_ref::<SelectionState<Renderer::Paragraph, iced_checkbox::Status>>();

        let mut children = layout.children();
        let control_layout = children.next().unwrap();
        let bounds = control_layout.bounds();

        let status = state
            .last_status
            .unwrap_or(iced_checkbox::Status::Disabled {
                is_checked: self.is_checked,
            });
        let unchecked_status = match status {
            iced_checkbox::Status::Active { .. } => {
                iced_checkbox::Status::Active { is_checked: false }
            }
            iced_checkbox::Status::Hovered { .. } => {
                iced_checkbox::Status::Hovered { is_checked: false }
            }
            iced_checkbox::Status::Disabled { .. } => {
                iced_checkbox::Status::Disabled { is_checked: false }
            }
        };
        let checked_status = match status {
            iced_checkbox::Status::Active { .. } => {
                iced_checkbox::Status::Active { is_checked: true }
            }
            iced_checkbox::Status::Hovered { .. } => {
                iced_checkbox::Status::Hovered { is_checked: true }
            }
            iced_checkbox::Status::Disabled { .. } => {
                iced_checkbox::Status::Disabled { is_checked: true }
            }
        };
        let current_style = (self.style)(theme, status);
        let unchecked_style = (self.style)(theme, unchecked_status);
        let checked_style = (self.style)(theme, checked_status);

        let selection = state.position.value.clamp(0.0, 1.0);
        let opacity = state.color.value.clamp(0.0, 1.0);
        let scale = 0.74 + 0.26 * state.size.value.clamp(0.0, 1.0);

        if let Some(layer_color) =
            self.state_layer_color(state, status, &unchecked_style, &checked_style)
        {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: scaled_rect(
                        bounds,
                        tokens::component::checkbox::STATE_LAYER_SIZE,
                        tokens::component::checkbox::STATE_LAYER_SIZE,
                    ),
                    border: border::rounded(tokens::component::checkbox::STATE_LAYER_SIZE / 2.0),
                    ..renderer::Quad::default()
                },
                Background::Color(layer_color),
            );
        }

        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: alpha_border(unchecked_style.border, 1.0 - selection),
                ..renderer::Quad::default()
            },
            unchecked_style.background.scale_alpha(1.0 - selection),
        );

        if selection > 0.0 {
            let selected_bounds = scaled_rect(bounds, bounds.width * scale, bounds.height * scale);

            renderer.fill_quad(
                renderer::Quad {
                    bounds: selected_bounds,
                    border: alpha_border(checked_style.border, selection),
                    ..renderer::Quad::default()
                },
                checked_style.background.scale_alpha(selection),
            );
        }

        if opacity > 0.0 {
            let icon_scale = 0.58 + 0.42 * selection;
            let icon_size = tokens::component::checkbox::ICON_SIZE * icon_scale;
            let mark_progress = if self.is_checked { selection } else { 1.0 };

            renderer.draw_svg(
                core_svg::Svg::new(core_svg::Handle::from_memory(checkbox_checkmark_svg(
                    mark_progress,
                )))
                .color(checked_style.icon_color)
                .opacity(opacity),
                scaled_rect(bounds, icon_size, icon_size),
                *viewport,
            );
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

impl<'a, Message, Renderer> From<Checkbox<'a, Message, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: core_text::Renderer + core_svg::Renderer + 'a,
{
    fn from(checkbox: Checkbox<'a, Message, Renderer>) -> Self {
        Element::new(checkbox)
    }
}

pub fn control<'a, Message, Renderer>(is_checked: bool) -> Checkbox<'a, Message, Renderer>
where
    Renderer: core_text::Renderer + core_svg::Renderer + 'a,
{
    Checkbox::new(is_checked)
        .size(tokens::component::checkbox::CONTAINER_SIZE)
        .spacing(f32::from(
            tokens::component::divider::LIST_ITEM_LEADING_SPACE,
        ))
        .text_size(tokens::component::checkbox::LABEL_TEXT_SIZE)
        .text_line_height(absolute_line_height(
            tokens::component::checkbox::LABEL_TEXT_LINE_HEIGHT,
        ))
        .style(checkbox_style::default)
}

pub fn standard<'a, Message, Renderer>(
    is_checked: bool,
    label: impl text::IntoFragment<'a>,
    on_toggle: impl Fn(bool) -> Message + 'a,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + core_text::Renderer + core_svg::Renderer + 'a,
{
    Container::new(control(is_checked).label(label).on_toggle(on_toggle))
        .center_y(Length::Fixed(tokens::component::checkbox::STATE_LAYER_SIZE))
        .into()
}
