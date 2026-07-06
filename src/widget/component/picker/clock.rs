fn clock_face<'a, Message, Renderer>(
    state: &'a TimePickerState,
    on_action: impl Fn(TimePickerAction) -> Message + Clone + 'a,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + 'static + 'a,
{
    Canvas::new(ClockFace {
        hour: state.hour,
        minute: state.minute,
        is_24_hour: state.is_24_hour,
        selection: state.selection,
        previous_selection: state.animation.previous_selection,
        selected_selection: state.animation.selected_selection,
        selection_progress: state.animation.selection.value.clamp(0.0, 1.0),
        auto_switch_to_minute: state.auto_switch_to_minute,
        selector_angle: state.animation.clock_angle(),
        on_action: Arc::new(on_action),
    })
    .width(Length::Fixed(
        tokens::component::time_picker::CLOCK_DIAL_SIZE,
    ))
    .height(Length::Fixed(
        tokens::component::time_picker::CLOCK_DIAL_SIZE,
    ))
    .into()
}

#[derive(Clone)]
struct ClockFace<F> {
    hour: u8,
    minute: u8,
    is_24_hour: bool,
    selection: TimePickerSelectionMode,
    previous_selection: TimePickerSelectionMode,
    selected_selection: TimePickerSelectionMode,
    selection_progress: f32,
    auto_switch_to_minute: bool,
    selector_angle: f32,
    on_action: Arc<F>,
}

impl<Message, F, Renderer> canvas::Program<Message, Theme, Renderer> for ClockFace<F>
where
    Message: Clone,
    F: Fn(TimePickerAction) -> Message,
    Renderer: geometry::Renderer + 'static,
{
    type State = ClockFaceState<Renderer>;

    fn update(
        &self,
        state: &mut Self::State,
        event: &canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        match event {
            event::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let position = event_position(event, bounds, cursor)?;
                if !local_point_is_in_bounds(position, bounds.size()) {
                    return None;
                }
                state.drag = Some(ClockFaceDrag::new(ClockFacePointer::Mouse, position));
                let action = self.action_at(position, bounds.size());
                Some(canvas::Action::publish((self.on_action)(action)).and_capture())
            }
            event::Event::Touch(touch::Event::FingerPressed { id, .. }) => {
                if state.drag.is_some() {
                    return None;
                }

                let position = event_position(event, bounds, cursor)?;
                if !local_point_is_in_bounds(position, bounds.size()) {
                    return None;
                }
                state.drag = Some(ClockFaceDrag::new(ClockFacePointer::Touch(*id), position));
                let action = self.action_at(position, bounds.size());
                Some(canvas::Action::publish((self.on_action)(action)).and_capture())
            }
            event::Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if !state
                    .drag
                    .is_some_and(|drag| drag.pointer == ClockFacePointer::Mouse)
                {
                    return None;
                }

                let position = event_position(event, bounds, cursor)?;
                if let Some(drag) = &mut state.drag {
                    drag.update(position);
                }
                let action = self.drag_action_at(position, bounds.size());
                Some(canvas::Action::publish((self.on_action)(action)).and_capture())
            }
            event::Event::Touch(touch::Event::FingerMoved { id, .. }) => {
                if !state
                    .drag
                    .is_some_and(|drag| drag.pointer == ClockFacePointer::Touch(*id))
                {
                    return None;
                }

                let position = event_position(event, bounds, cursor)?;
                if let Some(drag) = &mut state.drag {
                    drag.update(position);
                }
                let action = self.drag_action_at(position, bounds.size());
                Some(canvas::Action::publish((self.on_action)(action)).and_capture())
            }
            event::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                self.finish_drag(state, ClockFacePointer::Mouse)
            }
            event::Event::Touch(
                touch::Event::FingerLifted { id, .. } | touch::Event::FingerLost { id, .. },
            ) => self.finish_drag(state, ClockFacePointer::Touch(*id)),
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
        let render_key = self.render_key(theme);

        if state.render_key.get() != Some(render_key) {
            state.cache.clear();
            state.render_key.set(Some(render_key));
        }

        let geometry = state.cache.draw(renderer, bounds.size(), |frame| {
            let colors = theme.colors();
            let center = frame.center();
            let radius = frame.width().min(frame.height()) / 2.0;
            let dial = Path::circle(center, radius);

            frame.fill(&dial, colors.surface.container.highest);
            self.draw_labels(frame, theme, center, radius, ClockLabelPass::Background);
            self.draw_selector(frame, theme, center, radius);
            self.draw_labels(
                frame,
                theme,
                center,
                radius,
                ClockLabelPass::SelectedForeground,
            );
        });

        vec![geometry]
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

struct ClockFaceState<Renderer = iced_widget::Renderer>
where
    Renderer: geometry::Renderer,
{
    drag: Option<ClockFaceDrag>,
    cache: canvas::Cache<Renderer>,
    render_key: Cell<Option<ClockFaceRenderKey>>,
}

impl<Renderer> Default for ClockFaceState<Renderer>
where
    Renderer: geometry::Renderer,
{
    fn default() -> Self {
        Self {
            drag: None,
            cache: canvas::Cache::new(),
            render_key: Cell::new(None),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct ClockFaceRenderKey {
    hour: u8,
    minute: u8,
    is_24_hour: bool,
    selection: TimePickerSelectionMode,
    previous_selection: TimePickerSelectionMode,
    selected_selection: TimePickerSelectionMode,
    selection_progress: f32,
    selector_angle: f32,
    dial_color: Color,
    selector_color: Color,
    label_text_color: Color,
    selected_text_color: Color,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ClockFacePointer {
    Mouse,
    Touch(touch::Finger),
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct ClockFaceDrag {
    pointer: ClockFacePointer,
    current: Point,
}

impl ClockFaceDrag {
    fn new(pointer: ClockFacePointer, start: Point) -> Self {
        Self {
            pointer,
            current: start,
        }
    }

    fn update(&mut self, position: Point) {
        self.current = position;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ClockLabelPass {
    Background,
    SelectedForeground,
}

impl<F> ClockFace<F>
where
    F: Sized,
{
    fn finish_drag<Message, Renderer>(
        &self,
        state: &mut ClockFaceState<Renderer>,
        pointer: ClockFacePointer,
    ) -> Option<canvas::Action<Message>>
    where
        F: Fn(TimePickerAction) -> Message,
        Renderer: geometry::Renderer,
    {
        if state.drag.is_some_and(|drag| drag.pointer == pointer) {
            state.drag = None;
            let action =
                if self.auto_switch_to_minute && self.selection == TimePickerSelectionMode::Hour {
                    TimePickerAction::SetSelection(TimePickerSelectionMode::Minute)
                } else {
                    TimePickerAction::FinishDrag
                };

            Some(canvas::Action::publish((self.on_action)(action)).and_capture())
        } else {
            None
        }
    }

    fn action_at(&self, position: Point, size: Size) -> TimePickerAction {
        self.action_at_with_mode(position, size, false)
    }

    fn drag_action_at(&self, position: Point, size: Size) -> TimePickerAction {
        self.action_at_with_mode(position, size, true)
    }

    fn action_at_with_mode(&self, position: Point, size: Size, dragging: bool) -> TimePickerAction {
        let center = Point::new(size.width / 2.0, size.height / 2.0);
        let dx = position.x - center.x;
        let dy = position.y - center.y;
        let angle = dy.atan2(dx);

        if self.selection == TimePickerSelectionMode::Minute {
            let minute = angle_to_minute(angle);
            if dragging {
                TimePickerAction::DragMinuteAngle(minute, pack_angle(angle))
            } else {
                TimePickerAction::SelectMinute(minute)
            }
        } else {
            let action_angle = if dragging {
                angle
            } else {
                (angle / (TAU / 12.0)).round() * (TAU / 12.0)
            };
            let mut hour = angle_to_hour(action_angle);

            if self.is_24_hour {
                let distance = (dx * dx + dy * dy).sqrt();
                let max_distance = tokens::component::time_picker::MAX_DISTANCE;
                if distance < max_distance {
                    if hour == 12 {
                        hour = 0;
                    } else {
                        hour += 12;
                    }
                }
            }

            if dragging {
                TimePickerAction::DragHourAngle(hour, pack_angle(angle))
            } else {
                TimePickerAction::SelectHour(hour)
            }
        }
    }

    fn render_key(&self, theme: &Theme) -> ClockFaceRenderKey {
        let colors = theme.colors();

        ClockFaceRenderKey {
            hour: self.hour,
            minute: self.minute,
            is_24_hour: self.is_24_hour,
            selection: self.selection,
            previous_selection: self.previous_selection,
            selected_selection: self.selected_selection,
            selection_progress: self.selection_progress,
            selector_angle: self.selector_angle,
            dial_color: colors.surface.container.highest,
            selector_color: colors.primary.color,
            label_text_color: colors.surface.text,
            selected_text_color: colors.primary.text,
        }
    }

    fn draw_selector<Renderer>(
        &self,
        frame: &mut Frame<Renderer>,
        theme: &Theme,
        center: Point,
        radius: f32,
    ) where
        Renderer: geometry::Renderer,
    {
        let selector_color = theme.colors().primary.color;
        let angle = self.selector_angle;
        let (handle_center, handle_radius, selector_radius) =
            self.selector_handle_geometry(center, radius);
        let line_end = Point::new(
            center.x + (selector_radius - handle_radius) * angle.cos(),
            center.y + (selector_radius - handle_radius) * angle.sin(),
        );

        let line = Path::line(center, line_end);
        frame.stroke(
            &line,
            Stroke::default()
                .with_width(tokens::component::time_picker::CLOCK_DIAL_SELECTOR_TRACK_WIDTH)
                .with_color(selector_color),
        );
        frame.fill(&Path::circle(handle_center, handle_radius), selector_color);
        frame.fill(
            &Path::circle(
                center,
                tokens::component::time_picker::CLOCK_DIAL_SELECTOR_CENTER_SIZE / 2.0,
            ),
            selector_color,
        );
    }

    fn draw_labels<Renderer>(
        &self,
        frame: &mut Frame<Renderer>,
        theme: &Theme,
        center: Point,
        radius: f32,
        pass: ClockLabelPass,
    ) where
        Renderer: geometry::Renderer,
    {
        if self.previous_selection != self.selected_selection && self.selection_progress < 1.0 {
            self.draw_labels_for_selection(
                frame,
                theme,
                center,
                radius,
                self.previous_selection,
                1.0 - self.selection_progress,
                pass,
            );
            self.draw_labels_for_selection(
                frame,
                theme,
                center,
                radius,
                self.selected_selection,
                self.selection_progress,
                pass,
            );
        } else {
            self.draw_labels_for_selection(frame, theme, center, radius, self.selection, 1.0, pass);
        }
    }

    fn draw_labels_for_selection<Renderer>(
        &self,
        frame: &mut Frame<Renderer>,
        theme: &Theme,
        center: Point,
        radius: f32,
        selection: TimePickerSelectionMode,
        alpha: f32,
        pass: ClockLabelPass,
    ) where
        Renderer: geometry::Renderer,
    {
        if alpha <= 0.0 {
            return;
        }

        let label_radius = radius * 2.0 * tokens::component::time_picker::OUTER_CIRCLE_RADIUS_RATIO;
        let scale = tokens::component::time_picker::CLOCK_DIAL_LABEL_TEXT;

        match selection {
            TimePickerSelectionMode::Hour => {
                for hour in HOURS {
                    let value = if self.is_24_hour { hour % 12 } else { hour };
                    let angle = hour_angle(value);
                    self.draw_clock_label_for_pass(
                        frame,
                        theme,
                        center,
                        radius,
                        label_radius,
                        angle,
                        &value.to_string(),
                        scale,
                        alpha,
                        pass,
                    );
                }

                if self.is_24_hour {
                    let inner_radius =
                        radius * 2.0 * tokens::component::time_picker::INNER_CIRCLE_RADIUS_RATIO;
                    for hour in EXTRA_HOURS {
                        let angle = hour_angle(hour);
                        self.draw_clock_label_for_pass(
                            frame,
                            theme,
                            center,
                            radius,
                            inner_radius,
                            angle,
                            &hour.to_string(),
                            scale,
                            alpha,
                            pass,
                        );
                    }
                }
            }
            TimePickerSelectionMode::Minute => {
                for minute in MINUTES {
                    let angle = minute_angle(minute);
                    self.draw_clock_label_for_pass(
                        frame,
                        theme,
                        center,
                        radius,
                        label_radius,
                        angle,
                        &two_digit(minute),
                        scale,
                        alpha,
                        pass,
                    );
                }
            }
        }
    }

    fn draw_clock_label_for_pass<Renderer>(
        &self,
        frame: &mut Frame<Renderer>,
        theme: &Theme,
        center: Point,
        clock_radius: f32,
        label_radius: f32,
        angle: f32,
        label: &str,
        scale: tokens::typography::TypeScale,
        alpha: f32,
        pass: ClockLabelPass,
    ) where
        Renderer: geometry::Renderer,
    {
        match pass {
            ClockLabelPass::Background => {
                draw_clock_label(
                    frame,
                    theme,
                    center,
                    label_radius,
                    angle,
                    label,
                    false,
                    scale,
                    alpha,
                );
            }
            ClockLabelPass::SelectedForeground => {
                if !self.label_uses_selector_foreground(
                    center,
                    clock_radius,
                    label_radius,
                    angle,
                    scale,
                ) {
                    return;
                }

                let (handle_center, handle_radius, _) =
                    self.selector_handle_geometry(center, clock_radius);
                draw_clock_label_clipped_to_circle(
                    frame,
                    theme,
                    center,
                    label_radius,
                    angle,
                    label,
                    true,
                    scale,
                    alpha,
                    handle_center,
                    handle_radius,
                );
            }
        }
    }

    fn selector_handle_geometry(&self, center: Point, radius: f32) -> (Point, f32, f32) {
        let handle_radius = tokens::component::time_picker::CLOCK_DIAL_SELECTOR_HANDLE_SIZE / 2.0;
        let outer = radius * 2.0 * tokens::component::time_picker::OUTER_CIRCLE_RADIUS_RATIO;
        let inner = radius * 2.0 * tokens::component::time_picker::INNER_CIRCLE_RADIUS_RATIO;
        let selector_radius = if self.selection == TimePickerSelectionMode::Hour
            && self.is_24_hour
            && self.hour >= 12
        {
            inner
        } else {
            outer
        };
        let angle = self.selector_angle;
        let handle_center = clock_label_position(center, selector_radius, angle);

        (handle_center, handle_radius, selector_radius)
    }

    fn label_uses_selector_foreground(
        &self,
        center: Point,
        clock_radius: f32,
        label_radius: f32,
        label_angle: f32,
        scale: tokens::typography::TypeScale,
    ) -> bool {
        self.label_matches_selector_ring(clock_radius, label_radius)
            && self.label_intersects_selector(
                center,
                clock_radius,
                label_radius,
                label_angle,
                scale,
            )
    }

    fn label_matches_selector_ring(&self, clock_radius: f32, label_radius: f32) -> bool {
        let (_, _, selector_radius) = self.selector_handle_geometry(Point::ORIGIN, clock_radius);

        (label_radius - selector_radius).abs() <= 1.0
    }

    fn label_intersects_selector(
        &self,
        center: Point,
        radius: f32,
        label_radius: f32,
        label_angle: f32,
        scale: tokens::typography::TypeScale,
    ) -> bool {
        let (handle_center, handle_radius, _) = self.selector_handle_geometry(center, radius);
        let label_center = clock_label_position(center, label_radius, label_angle);
        let dx = label_center.x - handle_center.x;
        let dy = label_center.y - handle_center.y;
        let label_extent = scale.line_height.max(scale.size * 1.5) / 2.0;
        let threshold = handle_radius + label_extent;

        dx * dx + dy * dy <= threshold * threshold
    }
}

fn event_position(
    event: &canvas::Event,
    bounds: Rectangle,
    cursor: mouse::Cursor,
) -> Option<Point> {
    if cursor.position().is_some() {
        return cursor
            .position()
            .map(|position| Point::new(position.x - bounds.x, position.y - bounds.y));
    }

    if cursor.is_levitating() {
        return None;
    }

    match event {
        event::Event::Mouse(mouse::Event::CursorMoved { position }) => {
            Some(Point::new(position.x - bounds.x, position.y - bounds.y))
        }
        event::Event::Touch(
            touch::Event::FingerPressed { position, .. }
            | touch::Event::FingerMoved { position, .. }
            | touch::Event::FingerLifted { position, .. }
            | touch::Event::FingerLost { position, .. },
        ) => Some(Point::new(position.x - bounds.x, position.y - bounds.y)),
        _ => None,
    }
}

fn local_point_is_in_bounds(position: Point, size: Size) -> bool {
    position.x >= 0.0 && position.y >= 0.0 && position.x <= size.width && position.y <= size.height
}

fn draw_clock_label<Renderer>(
    frame: &mut Frame<Renderer>,
    theme: &Theme,
    center: Point,
    radius: f32,
    angle: f32,
    label: &str,
    selected: bool,
    scale: tokens::typography::TypeScale,
    alpha: f32,
) where
    Renderer: geometry::Renderer,
{
    let text = clock_label_text(theme, center, radius, angle, label, selected, scale, alpha);
    frame.fill_text(text);
}

fn draw_clock_label_clipped_to_circle<Renderer>(
    frame: &mut Frame<Renderer>,
    theme: &Theme,
    center: Point,
    radius: f32,
    angle: f32,
    label: &str,
    selected: bool,
    scale: tokens::typography::TypeScale,
    alpha: f32,
    clip_center: Point,
    clip_radius: f32,
) where
    Renderer: geometry::Renderer,
{
    let text = clock_label_text(theme, center, radius, angle, label, selected, scale, alpha);
    let strip_height = clip_radius * 2.0 / CLOCK_LABEL_CLIP_STRIPS as f32;

    for index in 0..CLOCK_LABEL_CLIP_STRIPS {
        let top = clip_center.y - clip_radius + index as f32 * strip_height;
        let middle = top + strip_height / 2.0;
        let dy = middle - clip_center.y;
        let half_width = (clip_radius * clip_radius - dy * dy).max(0.0).sqrt();

        if half_width <= 0.0 {
            continue;
        }

        frame.with_clip(
            Rectangle {
                x: clip_center.x - half_width,
                y: top,
                width: half_width * 2.0,
                height: strip_height,
            },
            |frame| frame.fill_text(text.clone()),
        );
    }
}

fn clock_label_text(
    theme: &Theme,
    center: Point,
    radius: f32,
    angle: f32,
    label: &str,
    selected: bool,
    scale: tokens::typography::TypeScale,
    alpha: f32,
) -> CanvasText {
    let colors = theme.colors();
    let color = if selected {
        colors.primary.text
    } else {
        colors.surface.text
    };
    let position = clock_label_position(center, radius, angle);

    CanvasText {
        content: label.to_owned(),
        position,
        max_width: 48.0,
        color: alpha_color(color, alpha),
        size: scale.size.into(),
        line_height: absolute_line_height(scale.line_height),
        font: fonts::roboto_for_type_scale(scale),
        align_x: core_text::Alignment::Center,
        align_y: alignment::Vertical::Center,
        shaping: text::Shaping::Advanced,
    }
}

fn clock_label_position(center: Point, radius: f32, angle: f32) -> Point {
    Point::new(
        center.x + radius * angle.cos(),
        center.y + radius * angle.sin(),
    )
}
