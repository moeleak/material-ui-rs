use std::hash::Hash;
use std::time::Instant;

use iced::futures::{FutureExt, SinkExt, StreamExt, channel::mpsc, future};
use iced_winit::futures::{BoxStream, subscription};

use super::{Event, invalidate, queue_refresh, runtime};

pub(super) struct AndroidEvents;

impl subscription::Recipe for AndroidEvents {
    type Output = Event;

    fn hash(&self, state: &mut subscription::Hasher) {
        std::any::TypeId::of::<Self>().hash(state);
    }

    fn stream(self: Box<Self>, mut input: subscription::EventStream) -> BoxStream<Event> {
        iced::stream::channel(1, move |mut output: mpsc::Sender<Event>| async move {
            let (sender, mut wake) = mpsc::channel(1);
            let id = {
                let mut runtime = runtime();
                runtime.dirty = true;
                runtime.listeners.register(sender)
            };
            let _registration = Registration(id);
            let mut delivered = None;
            loop {
                let _ = queue_refresh();
                let (active, snapshot, initialized, next_poll) = {
                    let runtime = runtime();
                    (
                        runtime.state.active,
                        runtime.state.snapshot,
                        runtime.state.initialized,
                        runtime.next_poll,
                    )
                };
                if initialized && delivered != Some(snapshot) {
                    if output
                        .send(Event::InsetsChanged(snapshot.raw))
                        .await
                        .is_err()
                    {
                        break;
                    }
                    delivered = Some(snapshot);
                }
                let delay = if active {
                    future::Either::Left(futures_timer::Delay::new(
                        next_poll.saturating_duration_since(Instant::now()),
                    ))
                } else {
                    // Do not keep a background timer running while paused.
                    future::Either::Right(future::pending::<()>())
                }
                .fuse();
                iced::futures::pin_mut!(delay);
                iced::futures::select! {
                    event = input.next().fuse() => {
                        let Some(event) = event else { break; };
                        if let subscription::Event::Interaction { event, .. } = event {
                            match event {
                                iced::Event::Window(iced::window::Event::Opened { .. }
                                    | iced::window::Event::Focused) => invalidate(true),
                                iced::Event::Window(iced::window::Event::Resized(_)
                                    | iced::window::Event::Rescaled(_)
                                    | iced::window::Event::Unfocused)
                                | iced::Event::InputMethod(_)
                                | iced::Event::Touch(iced::touch::Event::FingerPressed { .. })
                                | iced::Event::Touch(iced::touch::Event::FingerLifted { .. })
                                | iced::Event::Keyboard(_) => invalidate(false),
                                _ => {}
                            }
                        }
                    }
                    signal = wake.next().fuse() => {
                        if signal.is_none() { break; }
                    }
                    _ = delay => {
                        let mut runtime = runtime();
                        if Instant::now() >= runtime.next_poll {
                            runtime.dirty = true;
                            // Advance the deadline even while a slow UI callback
                            // is pending, preventing an expired-timer busy loop.
                            runtime.next_poll = Instant::now() + std::time::Duration::from_millis(100);
                        }
                    }
                }
            }
        })
        .boxed()
    }
}

struct Registration(u64);

impl Drop for Registration {
    fn drop(&mut self) {
        runtime().listeners.unregister(self.0);
    }
}
