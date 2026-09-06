//! Distinguishes a click from a pointer drag without taking over scrolling.

use iced_widget::core::{Event, Point, Rectangle, mouse, touch, window};

const DRAG_SLOP: f32 = 8.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Source {
    Mouse,
    Touch(touch::Finger),
}

#[derive(Debug, Clone, Copy)]
struct Press {
    source: Source,
    local_start: Point,
    raw_start: Option<Point>,
}

#[derive(Debug, Clone, Copy, Default)]
pub(super) struct ClickGesture {
    press: Option<Press>,
    mouse_position: Option<Point>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Update {
    None,
    Pressed,
    Released(bool),
    Cancelled,
}

impl ClickGesture {
    pub(super) fn update(
        &mut self,
        event: &Event,
        cursor: mouse::Cursor,
        bounds: Rectangle,
        enabled: bool,
    ) -> Update {
        if let Event::Mouse(mouse::Event::CursorMoved { position }) = event {
            self.mouse_position = Some(*position);
        }

        if !enabled
            || matches!(
                event,
                Event::Window(window::Event::Unfocused) | Event::Mouse(mouse::Event::CursorLeft)
            )
        {
            self.mouse_position = None;
            return self.cancel();
        }

        let source = source(event);
        if self
            .press
            .is_some_and(|press| source.is_some_and(|source| source != press.source))
        {
            return Update::None;
        }

        let position = if cursor.is_levitating() {
            None
        } else {
            cursor.position().or_else(|| touch_position(event))
        };
        let raw = match source {
            Some(Source::Mouse) => self.mouse_position,
            Some(Source::Touch(_)) => touch_position(event),
            None => None,
        };

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. })
                if self.press.is_none() =>
            {
                if let Some(local_start) =
                    position.filter(|_| super::press_is_over(event, bounds, cursor))
                {
                    self.press = Some(Press {
                        source: source.unwrap(),
                        local_start,
                        raw_start: raw,
                    });
                    Update::Pressed
                } else {
                    Update::None
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. })
            | Event::Touch(touch::Event::FingerMoved { .. })
                if self.press.is_some() =>
            {
                let press = self.press.as_mut().unwrap();
                // A mouse down has no coordinates. Usually the preceding move
                // supplies the raw anchor. If it did not, establish the mapping
                // from the translated cursor on the first move. Keeping raw
                // positions catches a move out and back in the same event batch.
                if press.raw_start.is_none() {
                    if let (Some(raw), Some(local)) = (raw, position) {
                        press.raw_start = Some(raw - (local - press.local_start));
                    }
                }
                if position.is_none() || moved(*press, raw, position) {
                    self.cancel()
                } else {
                    Update::None
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. }) => {
                if let Some(press) = self.press.take() {
                    Update::Released(
                        super::release_is_over(event, bounds, cursor)
                            && !moved(press, raw, position),
                    )
                } else {
                    Update::None
                }
            }
            Event::Touch(touch::Event::FingerLost { .. })
            | Event::Mouse(mouse::Event::WheelScrolled { .. }) => self.cancel(),
            _ => Update::None,
        }
    }

    fn cancel(&mut self) -> Update {
        if self.press.take().is_some() {
            Update::Cancelled
        } else {
            Update::None
        }
    }
}

fn moved(press: Press, raw: Option<Point>, local: Option<Point>) -> bool {
    let far = |start: Point, current: Point| {
        let delta = current - start;
        delta.x * delta.x + delta.y * delta.y > DRAG_SLOP * DRAG_SLOP
    };
    local.is_none_or(|current| far(press.local_start, current))
        || press
            .raw_start
            .zip(raw)
            .is_some_and(|(start, current)| far(start, current))
}

fn source(event: &Event) -> Option<Source> {
    match event {
        Event::Mouse(_) => Some(Source::Mouse),
        Event::Touch(
            touch::Event::FingerPressed { id, .. }
            | touch::Event::FingerMoved { id, .. }
            | touch::Event::FingerLifted { id, .. }
            | touch::Event::FingerLost { id, .. },
        ) => Some(Source::Touch(*id)),
        _ => None,
    }
}

fn touch_position(event: &Event) -> Option<Point> {
    match event {
        Event::Touch(
            touch::Event::FingerPressed { position, .. }
            | touch::Event::FingerMoved { position, .. }
            | touch::Event::FingerLifted { position, .. }
            | touch::Event::FingerLost { position, .. },
        ) => Some(*position),
        _ => None,
    }
}

#[cfg(test)]
#[path = "../../../tests/widget/internal/click.rs"]
mod tests;
