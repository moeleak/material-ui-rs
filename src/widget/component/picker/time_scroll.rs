fn time_scroll_body<'a, Message, Renderer>(
    state: &'a TimePickerState,
    on_action: impl Fn(TimePickerAction) -> Message + Clone + 'a,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    let hour = time_scroll_field(state, on_action.clone(), TimePickerSelectionMode::Hour);
    let minute = time_scroll_field(state, on_action.clone(), TimePickerSelectionMode::Minute);
    let mut content = Row::new()
        .push(hour)
        .push(display_separator_with_size::<Message, Renderer>(
            tokens::component::time_picker::TIME_SCROLL_SEPARATOR_WIDTH,
            tokens::component::time_picker::TIME_SCROLL_FIELD_HEIGHT,
        ))
        .push(minute)
        .align_y(alignment::Vertical::Top);

    if !state.is_24_hour {
        content = content
            .push(Space::new().width(Length::Fixed(
                tokens::component::time_picker::RICH_PERIOD_SELECTOR_START_SPACE,
            )))
            .push(period_toggle_sized(
                state,
                on_action,
                true,
                tokens::component::time_picker::RICH_PERIOD_SELECTOR_WIDTH,
                tokens::component::time_picker::RICH_PERIOD_SELECTOR_HEIGHT,
            ));
    }

    content.into()
}

fn time_scroll_field<'a, Message, Renderer>(
    state: &'a TimePickerState,
    on_action: impl Fn(TimePickerAction) -> Message + Clone + 'a,
    selection: TimePickerSelectionMode,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    let scroll_field = Canvas::new(TimeScrollField {
        selection,
        is_24_hour: state.is_24_hour,
        selected_value: time_scroll_selected_value(state, selection),
        anchor_value: time_scroll_anchor_value(state, selection),
        option_count: time_scroll_option_count(state, selection),
        on_action: Arc::new(on_action),
    })
    .width(Length::Fixed(
        tokens::component::time_picker::TIME_SCROLL_FIELD_WIDTH,
    ))
    .height(Length::Fixed(
        tokens::component::time_picker::TIME_SCROLL_FIELD_HEIGHT,
    ));

    Container::new(
        scroll_field
            .width(Length::Fixed(
                tokens::component::time_picker::TIME_SCROLL_FIELD_WIDTH,
            ))
            .height(Length::Fixed(
                tokens::component::time_picker::TIME_SCROLL_FIELD_HEIGHT,
            )),
    )
    .width(Length::Fixed(
        tokens::component::time_picker::TIME_SCROLL_FIELD_WIDTH,
    ))
    .height(Length::Fixed(
        tokens::component::time_picker::TIME_SCROLL_FIELD_HEIGHT,
    ))
    .clip(true)
    .style(time_scroll_field_container_style)
    .into()
}

struct TimeScrollField<F> {
    selection: TimePickerSelectionMode,
    is_24_hour: bool,
    selected_value: u8,
    anchor_value: u8,
    option_count: u8,
    on_action: Arc<F>,
}

#[derive(Debug, Clone, Default)]
struct TimeScrollFieldState {
    offset_y: f32,
    velocity_y: f32,
    last_frame: Option<Instant>,
    last_reported_value: Option<u8>,
    drag: Option<TimeScrollDrag>,
    selection: Option<TimePickerSelectionMode>,
    anchor_value: Option<u8>,
    option_count: u8,
    is_24_hour: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TimeScrollPointer {
    Mouse,
    Touch(touch::Finger),
}

impl TimeScrollPointer {
    fn is_touch(self) -> bool {
        matches!(self, Self::Touch(_))
    }
}

#[derive(Debug, Clone, Copy)]
struct TimeScrollDrag {
    pointer: TimeScrollPointer,
    start: Point,
    last: Point,
    last_at: Instant,
    has_scrolled: bool,
}

impl TimeScrollDrag {
    fn new(pointer: TimeScrollPointer, position: Point, now: Instant) -> Self {
        Self {
            pointer,
            start: position,
            last: position,
            last_at: now,
            has_scrolled: false,
        }
    }

    fn moved_beyond_slop(self, position: Point) -> bool {
        let dx = position.x - self.start.x;
        let dy = position.y - self.start.y;

        dx * dx + dy * dy > TIME_SCROLL_TOUCH_SLOP * TIME_SCROLL_TOUCH_SLOP
    }
}

impl<Message, F, Renderer> canvas::Program<Message, Theme, Renderer> for TimeScrollField<F>
where
    Message: Clone,
    F: Fn(TimePickerAction) -> Message,
    Renderer: geometry::Renderer,
{
    type State = TimeScrollFieldState;

    fn update(
        &self,
        state: &mut Self::State,
        event: &canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        self.sync_state(state);

        match event {
            event::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                self.press(state, event, bounds, cursor, TimeScrollPointer::Mouse)
            }
            event::Event::Touch(touch::Event::FingerPressed { id, .. }) => {
                self.press(state, event, bounds, cursor, TimeScrollPointer::Touch(*id))
            }
            event::Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                self.drag(state, event, bounds, cursor, TimeScrollPointer::Mouse)
            }
            event::Event::Touch(touch::Event::FingerMoved { id, .. }) => {
                self.drag(state, event, bounds, cursor, TimeScrollPointer::Touch(*id))
            }
            event::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                self.release(state, event, bounds, cursor, TimeScrollPointer::Mouse)
            }
            event::Event::Touch(
                touch::Event::FingerLifted { id, .. } | touch::Event::FingerLost { id, .. },
            ) => self.release(state, event, bounds, cursor, TimeScrollPointer::Touch(*id)),
            event::Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                if cursor.position_over(bounds).is_none() {
                    return None;
                }

                state.velocity_y = 0.0;
                state.last_frame = None;
                let delta_y = match *delta {
                    mouse::ScrollDelta::Lines { y, .. } => -y * 60.0,
                    mouse::ScrollDelta::Pixels { y, .. } => -y,
                };

                if self.scroll_by(state, delta_y) {
                    Some(self.scroll_or_redraw(state))
                } else {
                    Some(canvas::Action::capture())
                }
            }
            event::Event::Window(window::Event::RedrawRequested(now)) => {
                self.advance_inertia(state, *now)
            }
            _ => None,
        }
    }

    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry<Renderer>> {
        let mut frame = Frame::new(renderer, bounds.size());
        let width = tokens::component::time_picker::TIME_SCROLL_FIELD_WIDTH;
        let height = tokens::component::time_picker::TIME_SCROLL_FIELD_HEIGHT;
        let item_height = tokens::component::time_picker::TIME_SCROLL_ITEM_HEIGHT;
        let offset_y = self.effective_offset(state);
        let selected_row = time_scroll_row_offset_for_offset(self.option_count, offset_y);

        let layer_style = time_scroll_selection_layer_style(theme);
        if let Some(Background::Color(color)) = layer_style.background {
            let layer_top = (height - item_height) / 2.0;
            let layer = Path::rounded_rectangle(
                Point::new(0.0, layer_top),
                Size::new(width, item_height),
                layer_style.border.radius,
            );
            frame.fill(&layer, color);
        }

        for row_offset in -1..=i16::from(self.option_count) {
            let row_index = row_offset + 1;
            let top = f32::from(row_index) * item_height - offset_y;

            if top + item_height < 0.0 || top > height {
                continue;
            }

            let value = self.value_for_row_offset(row_offset);
            let selected = row_offset == selected_row;
            let scale = if selected {
                tokens::component::time_picker::TIME_SELECTOR_LABEL_TEXT
            } else {
                tokens::component::time_picker::CLOCK_DIAL_LABEL_TEXT
            };
            let style = time_scroll_item_style(theme, Status::Active, selected);
            let text = CanvasText {
                content: time_scroll_label(value, self.selection),
                position: Point::new(width / 2.0, top + item_height / 2.0),
                max_width: width,
                color: style.text_color,
                size: scale.size.into(),
                line_height: LineHeight::Absolute(scale.size.into()),
                font: fonts::roboto_for_type_scale(scale),
                align_x: core_text::Alignment::Center,
                align_y: alignment::Vertical::Center,
                shaping: text::Shaping::Advanced,
            };

            frame.fill_text(text);
        }

        vec![frame.into_geometry()]
    }

    fn mouse_interaction(
        &self,
        _state: &Self::State,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if cursor.is_over(bounds) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }
}

impl<F> TimeScrollField<F> {
    fn sync_state(&self, state: &mut TimeScrollFieldState) {
        if state.selection != Some(self.selection)
            || state.anchor_value != Some(self.anchor_value)
            || state.option_count != self.option_count
            || state.is_24_hour != self.is_24_hour
        {
            state.offset_y = 0.0;
            state.velocity_y = 0.0;
            state.last_frame = None;
            state.last_reported_value = Some(self.selected_value);
            state.drag = None;
            state.selection = Some(self.selection);
            state.anchor_value = Some(self.anchor_value);
            state.option_count = self.option_count;
            state.is_24_hour = self.is_24_hour;
        } else {
            state.offset_y = state.offset_y.clamp(0.0, self.max_offset());
        }
    }

    fn effective_offset(&self, state: &TimeScrollFieldState) -> f32 {
        if state.selection == Some(self.selection)
            && state.anchor_value == Some(self.anchor_value)
            && state.option_count == self.option_count
            && state.is_24_hour == self.is_24_hour
        {
            state.offset_y.clamp(0.0, self.max_offset())
        } else {
            0.0
        }
    }

    fn press<Message>(
        &self,
        state: &mut TimeScrollFieldState,
        event: &canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
        pointer: TimeScrollPointer,
    ) -> Option<canvas::Action<Message>> {
        let position = event_position(event, bounds, cursor)?;
        if !local_point_is_in_bounds(position, bounds.size()) {
            return None;
        }

        state.velocity_y = 0.0;
        state.last_frame = None;
        state.drag = Some(TimeScrollDrag::new(pointer, position, Instant::now()));

        Some(canvas::Action::capture())
    }

    fn drag<Message>(
        &self,
        state: &mut TimeScrollFieldState,
        event: &canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
        pointer: TimeScrollPointer,
    ) -> Option<canvas::Action<Message>>
    where
        F: Fn(TimePickerAction) -> Message,
    {
        let position = event_position(event, bounds, cursor)?;
        let now = Instant::now();
        let Some(drag) = state.drag.as_mut() else {
            return None;
        };

        if drag.pointer != pointer {
            return None;
        }

        let delta_y = if drag.has_scrolled {
            drag.last.y - position.y
        } else if drag.moved_beyond_slop(position) {
            drag.has_scrolled = true;
            drag.start.y - position.y
        } else {
            drag.last = position;
            drag.last_at = now;
            return Some(canvas::Action::capture());
        };
        let elapsed = now.duration_since(drag.last_at).as_secs_f32();

        drag.last = position;
        drag.last_at = now;

        if elapsed > 0.0 {
            state.velocity_y = (delta_y / elapsed).clamp(
                -TIME_SCROLL_FLING_MAX_VELOCITY,
                TIME_SCROLL_FLING_MAX_VELOCITY,
            );
        }

        if self.scroll_by(state, delta_y) {
            Some(self.scroll_or_redraw(state))
        } else {
            Some(canvas::Action::capture())
        }
    }

    fn release<Message>(
        &self,
        state: &mut TimeScrollFieldState,
        event: &canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
        pointer: TimeScrollPointer,
    ) -> Option<canvas::Action<Message>>
    where
        F: Fn(TimePickerAction) -> Message,
    {
        let position = event_position(event, bounds, cursor);
        let drag = state.drag?;

        if drag.pointer != pointer {
            return None;
        }

        state.drag = None;

        if !drag.has_scrolled {
            let position = position.unwrap_or(drag.last);
            let tap_offset = self.effective_offset(state);
            state.offset_y = 0.0;
            state.velocity_y = 0.0;
            state.last_frame = None;
            state.last_reported_value = Some(self.selected_value);
            let action = self.select_action_for_position(position.y, tap_offset);

            return Some(canvas::Action::publish((self.on_action)(action)).and_capture());
        }

        if pointer.is_touch() && state.velocity_y.abs() >= TIME_SCROLL_FLING_MIN_VELOCITY {
            state.velocity_y = state.velocity_y.clamp(
                -TIME_SCROLL_FLING_MAX_VELOCITY,
                TIME_SCROLL_FLING_MAX_VELOCITY,
            );
            state.last_frame = None;

            Some(canvas::Action::request_redraw().and_capture())
        } else {
            state.velocity_y = 0.0;
            state.last_frame = None;
            self.settle(state);
            Some(self.force_scroll_action(state))
        }
    }

    fn advance_inertia<Message>(
        &self,
        state: &mut TimeScrollFieldState,
        now: Instant,
    ) -> Option<canvas::Action<Message>>
    where
        F: Fn(TimePickerAction) -> Message,
    {
        if state.velocity_y.abs() < f32::EPSILON {
            return None;
        }

        let elapsed = state
            .last_frame
            .map(|last_frame| now.duration_since(last_frame).as_secs_f32())
            .unwrap_or(1.0 / 60.0)
            .clamp(0.0, 1.0 / 15.0);

        if elapsed <= 0.0 {
            return Some(canvas::Action::request_redraw().and_capture());
        }

        state.last_frame = Some(now);
        let velocity = state.velocity_y;
        let moved = self.scroll_by(state, velocity * elapsed);
        let speed = (velocity.abs() - TIME_SCROLL_FLING_DECELERATION * elapsed).max(0.0);

        state.velocity_y = velocity.signum() * speed;

        if !moved || speed < TIME_SCROLL_FLING_MIN_VELOCITY {
            state.velocity_y = 0.0;
            state.last_frame = None;
            self.settle(state);

            return Some(self.force_scroll_action(state));
        }

        Some(self.scroll_or_redraw(state))
    }

    fn scroll_or_redraw<Message>(&self, state: &mut TimeScrollFieldState) -> canvas::Action<Message>
    where
        F: Fn(TimePickerAction) -> Message,
    {
        let value = self.value_for_offset(state.offset_y);

        if state.last_reported_value != Some(value) {
            state.last_reported_value = Some(value);
            canvas::Action::publish((self.on_action)(self.scroll_action(value))).and_capture()
        } else {
            canvas::Action::request_redraw().and_capture()
        }
    }

    fn force_scroll_action<Message>(
        &self,
        state: &mut TimeScrollFieldState,
    ) -> canvas::Action<Message>
    where
        F: Fn(TimePickerAction) -> Message,
    {
        let value = self.value_for_offset(state.offset_y);
        state.last_reported_value = Some(value);

        canvas::Action::publish((self.on_action)(self.scroll_action(value))).and_capture()
    }

    fn scroll_by(&self, state: &mut TimeScrollFieldState, delta_y: f32) -> bool {
        let previous = state.offset_y;
        state.offset_y = (state.offset_y + delta_y).clamp(0.0, self.max_offset());

        (state.offset_y - previous).abs() > f32::EPSILON
    }

    fn settle(&self, state: &mut TimeScrollFieldState) {
        let row_offset = time_scroll_row_offset_for_offset(self.option_count, state.offset_y);
        state.offset_y =
            f32::from(row_offset) * tokens::component::time_picker::TIME_SCROLL_ITEM_HEIGHT;
    }

    fn max_offset(&self) -> f32 {
        time_scroll_max_offset(self.option_count)
    }

    fn value_for_offset(&self, offset_y: f32) -> u8 {
        let row_offset = time_scroll_row_offset_for_offset(self.option_count, offset_y);

        self.value_for_row_offset(row_offset)
    }

    fn value_for_row_offset(&self, row_offset: i16) -> u8 {
        time_scroll_value_for_anchor(
            self.anchor_value,
            self.selection,
            self.is_24_hour,
            row_offset,
        )
    }

    fn scroll_action(&self, value: u8) -> TimePickerAction {
        match self.selection {
            TimePickerSelectionMode::Hour => TimePickerAction::ScrollHour(value),
            TimePickerSelectionMode::Minute => TimePickerAction::ScrollMinute(value),
        }
    }

    fn select_action_for_position(&self, y: f32, offset_y: f32) -> TimePickerAction {
        let item_height = tokens::component::time_picker::TIME_SCROLL_ITEM_HEIGHT;
        let row_offset = ((y + offset_y) / item_height).floor() as i16 - 1;
        let value = self.value_for_row_offset(row_offset.clamp(-1, i16::from(self.option_count)));

        match self.selection {
            TimePickerSelectionMode::Hour => TimePickerAction::SelectHour(value),
            TimePickerSelectionMode::Minute => TimePickerAction::SelectMinute(value),
        }
    }
}
