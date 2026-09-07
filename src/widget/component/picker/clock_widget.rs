// Keep touch scrolling available over the dial. A touch can drag the current
// selector immediately; elsewhere, defer selection until it is a confirmed tap.
impl<Message, F, Renderer> Widget<Message, Theme, Renderer> for ClockFace<F>
where
    Message: Clone,
    F: Fn(TimePickerAction) -> Message,
    Renderer: geometry::Renderer + 'static,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<ClockFaceState<Renderer>>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(ClockFaceState::<Renderer>::default())
    }

    fn size(&self) -> Size<Length> {
        Size::new(
            Length::Fixed(tokens::component::time_picker::CLOCK_DIAL_SIZE),
            Length::Fixed(tokens::component::time_picker::CLOCK_DIAL_SIZE),
        )
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let size = <Self as Widget<Message, Theme, Renderer>>::size(self);
        layout::atomic(limits, size.width, size.height)
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
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<ClockFaceState<Renderer>>();
        let bounds = layout.bounds();
        let position = event_position(event, bounds, cursor);
        let visible = position.is_some_and(|position| {
            let center = Point::new(bounds.width / 2.0, bounds.height / 2.0);
            position.distance(center) <= bounds.width.min(bounds.height) / 2.0
                && viewport.contains(position + Vector::new(bounds.x, bounds.y))
        });

        if matches!(event, Event::Window(window::Event::Unfocused)) {
            let _ = state.tap.update(event, cursor, bounds, false);
            if state.drag.take().is_some() {
                shell.publish((self.on_action)(TimePickerAction::FinishDrag));
                shell.request_redraw();
            }
            return;
        }

        if let Event::Touch(touch::Event::FingerLost { id, .. }) = event
            && state
                .drag
                .is_some_and(|drag| drag.pointer == ClockFacePointer::Touch(*id))
        {
            state.drag = None;
            shell.publish((self.on_action)(TimePickerAction::FinishDrag));
            shell.request_redraw();
            shell.capture_event();
            return;
        }

        if matches!(event, Event::Touch(_)) && state.drag.is_none() {
            if matches!(event, Event::Touch(touch::Event::FingerPressed { .. })) && !visible {
                return;
            }
            let tap = state.tap.update(event, cursor, bounds, true);
            match tap {
                super::click::Update::Pressed => {
                    let center = Point::new(bounds.width / 2.0, bounds.height / 2.0);
                    let (handle, radius, _) = self
                        .selector_handle_geometry(center, bounds.width.min(bounds.height) / 2.0);
                    if !position.is_some_and(|position| position.distance(handle) <= radius) {
                        return;
                    }
                    let _ = state.tap.update(event, cursor, bounds, false);
                }
                super::click::Update::Released(true) if visible => {
                    let position = position.unwrap() + Vector::new(bounds.x, bounds.y);
                    // Replay both events in this update so selecting an hour
                    // and auto-switching to minutes keep their existing order.
                    for event in [
                        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
                        Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
                    ] {
                        self.update_canvas(
                            state,
                            &event,
                            bounds,
                            mouse::Cursor::Available(position),
                            shell,
                        );
                    }
                    return;
                }
                _ => return,
            }
        }

        if matches!(event, Event::Mouse(mouse::Event::ButtonPressed(_))) && !visible {
            return;
        }
        self.update_canvas(state, event, bounds, cursor, shell);
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        if bounds.width < 1.0 || bounds.height < 1.0 {
            return;
        }
        let Some(clip) = bounds.intersection(viewport) else {
            return;
        };
        let state = tree.state.downcast_ref::<ClockFaceState<Renderer>>();
        renderer.with_layer(clip, |renderer| {
            renderer.with_translation(Vector::new(bounds.x, bounds.y), |renderer| {
                for geometry in <Self as canvas::Program<Message, Theme, Renderer>>::draw(
                    self, state, renderer, theme, bounds, cursor,
                ) {
                    renderer.draw_geometry(geometry);
                }
            });
        });
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        if !cursor.is_over(*viewport) {
            return mouse::Interaction::None;
        }
        <Self as canvas::Program<Message, Theme, Renderer>>::mouse_interaction(
            self,
            tree.state.downcast_ref::<ClockFaceState<Renderer>>(),
            layout.bounds(),
            cursor,
        )
    }
}

impl<F> ClockFace<F> {
    fn update_canvas<Message, Renderer>(
        &self,
        state: &mut ClockFaceState<Renderer>,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
        shell: &mut Shell<'_, Message>,
    ) where
        Message: Clone,
        F: Fn(TimePickerAction) -> Message,
        Renderer: geometry::Renderer + 'static,
    {
        if let Some(action) = <Self as canvas::Program<Message, Theme, Renderer>>::update(
            self, state, event, bounds, cursor,
        ) {
            let (message, redraw, status) = action.into_inner();
            shell.request_redraw_at(redraw);
            if let Some(message) = message {
                shell.publish(message);
            }
            if status == event::Status::Captured {
                shell.capture_event();
            }
        }
    }
}
