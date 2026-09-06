use iced::Color;

/// Insets on each edge, expressed in iced logical pixels.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Insets {
    /// Left inset.
    pub left: f32,
    /// Top inset.
    pub top: f32,
    /// Right inset.
    pub right: f32,
    /// Bottom inset.
    pub bottom: f32,
}

impl Insets {
    /// No inset on any edge.
    pub const ZERO: Self = Self {
        left: 0.0,
        top: 0.0,
        right: 0.0,
        bottom: 0.0,
    };

    /// Combines two sources by taking the largest value on every edge.
    #[must_use]
    pub fn max(self, other: Self) -> Self {
        Self {
            left: self.left.max(other.left),
            top: self.top.max(other.top),
            right: self.right.max(other.right),
            bottom: self.bottom.max(other.bottom),
        }
    }
}

/// Independently tracked Android window inset sources.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct SafeAreaInsets {
    /// Insets occupied by the status bar.
    pub status_bars: Insets,
    /// Insets occupied by gesture or button navigation.
    pub navigation_bars: Insets,
    /// Insets occupied by a display cutout.
    pub display_cutout: Insets,
    /// Insets occupied by the on-screen keyboard.
    pub ime: Insets,
}

impl SafeAreaInsets {
    /// An empty safe area.
    pub const ZERO: Self = Self {
        status_bars: Insets::ZERO,
        navigation_bars: Insets::ZERO,
        display_cutout: Insets::ZERO,
        ime: Insets::ZERO,
    };

    /// Insets suitable for persistent page padding.
    #[must_use]
    pub fn system(self) -> Insets {
        self.status_bars
            .max(self.navigation_bars)
            .max(self.display_cutout)
    }

    /// Insets suitable for content that must stay visible above the keyboard.
    #[must_use]
    pub fn content(self) -> Insets {
        self.system().max(self.ime)
    }
}

/// System-bar presentation synchronized with an application theme.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SystemBarsStyle {
    /// Draw application content behind system bars.
    pub edge_to_edge: bool,
    /// Use dark status-bar icons on a light background.
    pub light_status_icons: bool,
    /// Use dark navigation-bar icons on a light background.
    pub light_navigation_icons: bool,
    /// Status-bar fallback color.
    pub status_bar_color: Color,
    /// Navigation-bar fallback color.
    pub navigation_bar_color: Color,
}

impl Default for SystemBarsStyle {
    fn default() -> Self {
        Self {
            edge_to_edge: true,
            light_status_icons: false,
            light_navigation_icons: false,
            status_bar_color: Color::TRANSPARENT,
            navigation_bar_color: Color::TRANSPARENT,
        }
    }
}

impl SystemBarsStyle {
    /// Transparent system bars with icon contrast matching the Material theme.
    #[must_use]
    pub fn from_theme(theme: &crate::Theme) -> Self {
        Self {
            light_status_icons: !theme.is_dark(),
            light_navigation_icons: !theme.is_dark(),
            ..Self::default()
        }
    }
}

impl Insets {
    fn remaining(self, consumed: Self) -> Self {
        Self {
            left: (self.left - consumed.left).max(0.0),
            top: (self.top - consumed.top).max(0.0),
            right: (self.right - consumed.right).max(0.0),
            bottom: (self.bottom - consumed.bottom).max(0.0),
        }
    }
}

impl SafeAreaInsets {
    pub(crate) fn remaining(self, consumed: Insets) -> Self {
        Self {
            status_bars: self.status_bars.remaining(consumed),
            navigation_bars: self.navigation_bars.remaining(consumed),
            display_cutout: self.display_cutout.remaining(consumed),
            ime: self.ime.remaining(consumed),
        }
    }
}

pub(crate) fn consumed_edges(
    window: iced::Size,
    origin: iced::Point,
    surface: iced::Size,
    scale: f32,
) -> Insets {
    Insets {
        left: origin.x.max(0.0) / scale,
        top: origin.y.max(0.0) / scale,
        right: (window.width - origin.x - surface.width).max(0.0) / scale,
        bottom: (window.height - origin.y - surface.height).max(0.0) / scale,
    }
}

pub(crate) fn legacy_insets(system: Insets, stable: Insets, cutout: Insets) -> SafeAreaInsets {
    // Stable insets exclude the IME. Preserve lateral navigation in landscape.
    let navigation = Insets { top: 0.0, ..stable };
    SafeAreaInsets {
        status_bars: Insets {
            top: stable.top,
            ..Insets::ZERO
        },
        navigation_bars: navigation,
        display_cutout: cutout,
        ime: Insets {
            // IME insets include the navigation area, just like API 30's typed insets.
            bottom: if system.bottom > stable.bottom {
                system.bottom
            } else {
                0.0
            },
            ..Insets::ZERO
        },
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Snapshot {
    pub raw: SafeAreaInsets,
    pub layout: SafeAreaInsets,
}

impl Snapshot {
    pub const ZERO: Self = Self {
        raw: SafeAreaInsets::ZERO,
        layout: SafeAreaInsets::ZERO,
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Request {
    pub generation: u64,
    id: u64,
}

#[derive(Debug, Default)]
pub(crate) struct WakeListeners {
    next_id: u64,
    senders: Vec<(u64, iced::futures::channel::mpsc::Sender<()>)>,
}

impl WakeListeners {
    pub fn register(&mut self, sender: iced::futures::channel::mpsc::Sender<()>) -> u64 {
        self.next_id += 1;
        self.senders.push((self.next_id, sender));
        self.next_id
    }

    pub fn unregister(&mut self, id: u64) {
        self.senders.retain(|(key, _)| *key != id);
    }

    pub fn wake(&mut self) {
        self.senders.retain_mut(|(_, sender)| {
            sender.try_send(()).is_ok_and(|()| true) || !sender.is_closed()
        });
    }
}

#[derive(Debug)]
pub(crate) struct RefreshState {
    pub active: bool,
    pub generation: u64,
    pub snapshot: Snapshot,
    pub initialized: bool,
    next_request: u64,
    pending: Option<Request>,
}

impl RefreshState {
    pub const fn new() -> Self {
        Self {
            active: false,
            generation: 0,
            snapshot: Snapshot::ZERO,
            initialized: false,
            next_request: 0,
            pending: None,
        }
    }

    pub fn begin_session(&mut self) {
        // Never reset either counter: callbacks from an older Activity may
        // still be queued when the new Activity starts in the same process.
        self.generation += 1;
        self.active = false;
        self.snapshot = Snapshot::ZERO;
        self.initialized = false;
        self.pending = None;
    }

    pub fn begin_request(&mut self) -> Option<Request> {
        if !self.active || self.pending.is_some() {
            return None;
        }
        self.next_request += 1;
        let request = Request {
            generation: self.generation,
            id: self.next_request,
        };
        self.pending = Some(request);
        Some(request)
    }

    pub fn is_current(&self, request: Request) -> bool {
        self.active && self.generation == request.generation && self.pending == Some(request)
    }

    pub fn finish_request(&mut self, request: Request) -> bool {
        if self.pending != Some(request) {
            return false;
        }
        self.pending = None;
        true
    }

    pub fn set_active(&mut self, active: bool) {
        if self.active != active {
            self.active = active;
            self.generation += 1;
        }
    }

    pub fn accept(&mut self, generation: u64, snapshot: Snapshot) -> bool {
        if !self.active || self.generation != generation {
            return false;
        }
        let changed = !self.initialized || self.snapshot != snapshot;
        self.snapshot = snapshot;
        self.initialized = true;
        changed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bottom(value: f32) -> Insets {
        Insets {
            bottom: value,
            ..Insets::ZERO
        }
    }

    #[test]
    fn a_previous_activity_cannot_finish_a_new_activity_request() {
        let mut state = RefreshState::new();
        state.begin_session();
        state.set_active(true);
        let old = state.begin_request().unwrap();
        state.begin_session();
        state.set_active(true);
        let new = state.begin_request().unwrap();
        assert_ne!(old.generation, new.generation);
        assert!(!state.is_current(old));
        assert!(!state.accept(old.generation, Snapshot::ZERO));
        assert!(!state.finish_request(old));
        assert!(state.is_current(new));
        assert!(state.begin_request().is_none());
        assert!(state.finish_request(new));
        assert!(state.begin_request().is_some());
    }

    #[test]
    fn repeated_completion_cannot_clear_the_next_request() {
        let mut state = RefreshState::new();
        state.set_active(true);
        let old = state.begin_request().unwrap();
        assert!(state.finish_request(old));
        let new = state.begin_request().unwrap();
        assert!(!state.finish_request(old));
        assert!(state.is_current(new));
    }

    #[test]
    fn independently_mapped_subscriptions_both_receive_wakes() {
        use iced::futures::{StreamExt, channel::mpsc, executor::block_on};
        let mut listeners = WakeListeners::default();
        let (a, mut receive_a) = mpsc::channel(1);
        let (b, mut receive_b) = mpsc::channel(1);
        let old = listeners.register(a);
        let current = listeners.register(b);
        listeners.wake();
        assert_eq!(block_on(receive_a.next()), Some(()));
        assert_eq!(block_on(receive_b.next()), Some(()));
        listeners.unregister(old);
        assert_eq!(block_on(receive_a.next()), None);
        listeners.unregister(old); // A late Drop is harmless to the remaining stream.
        listeners.wake();
        assert_eq!(block_on(receive_b.next()), Some(()));
        listeners.unregister(current);
        assert_eq!(block_on(receive_b.next()), None);
    }

    #[test]
    fn window_geometry_accounts_for_resize_and_density_once() {
        let consumed = consumed_edges(
            iced::Size::new(1080.0, 2400.0),
            iced::Point::new(0.0, 72.0),
            iced::Size::new(1080.0, 1578.0),
            3.0,
        );
        assert_eq!(
            consumed,
            Insets {
                top: 24.0,
                bottom: 250.0,
                ..Insets::ZERO
            }
        );
    }

    #[test]
    fn window_geometry_accounts_for_lateral_system_navigation() {
        let consumed = consumed_edges(
            iced::Size::new(2400.0, 1080.0),
            iced::Point::ORIGIN,
            iced::Size::new(2256.0, 1080.0),
            3.0,
        );
        assert_eq!(
            consumed,
            Insets {
                right: 48.0,
                ..Insets::ZERO
            }
        );
    }

    #[test]
    fn legacy_ime_is_separate_from_stable_navigation() {
        let insets = legacy_insets(bottom(320.0), bottom(24.0), Insets::ZERO);
        assert_eq!(insets.navigation_bars.bottom, 24.0);
        assert_eq!(insets.ime.bottom, 320.0);
        assert_eq!(insets.content().bottom, 320.0);
        assert_eq!(
            legacy_insets(bottom(24.0), bottom(24.0), Insets::ZERO).ime,
            Insets::ZERO
        );
    }

    #[test]
    fn landscape_navigation_preserves_its_edge() {
        let stable = Insets {
            right: 48.0,
            top: 24.0,
            ..Insets::ZERO
        };
        let insets = legacy_insets(stable, stable, Insets::ZERO);
        assert_eq!(
            insets.navigation_bars,
            Insets {
                right: 48.0,
                ..Insets::ZERO
            }
        );
        assert_eq!(insets.status_bars.top, 24.0);
    }

    #[test]
    fn resized_window_consumes_ime_padding_without_losing_visibility() {
        let raw = SafeAreaInsets {
            ime: bottom(320.0),
            navigation_bars: bottom(24.0),
            ..SafeAreaInsets::ZERO
        };
        let layout = raw.remaining(bottom(320.0));
        assert_eq!(layout.content().bottom, 0.0);
        assert_eq!(raw.ime.bottom, 320.0);
        assert_eq!(raw.remaining(bottom(100.0)).content().bottom, 220.0);
    }

    #[test]
    fn transient_suspend_invalidates_queued_queries() {
        let mut state = RefreshState::new();
        state.set_active(true);
        let generation = state.generation;
        assert!(state.accept(generation, Snapshot::ZERO));
        assert!(!state.accept(generation, Snapshot::ZERO));
        state.set_active(false);
        assert!(!state.accept(generation, Snapshot::ZERO));
        state.set_active(true);
        assert!(!state.accept(generation, Snapshot::ZERO));
    }

    #[test]
    fn layout_only_changes_are_observable() {
        let mut state = RefreshState::new();
        state.set_active(true);
        let snapshot = Snapshot {
            raw: SafeAreaInsets {
                ime: bottom(320.0),
                ..SafeAreaInsets::ZERO
            },
            layout: SafeAreaInsets::ZERO,
        };
        assert!(state.accept(state.generation, snapshot));
        let changed = Snapshot {
            layout: snapshot.raw,
            ..snapshot
        };
        assert!(state.accept(state.generation, changed));
    }

    #[test]
    fn theme_icons_contrast_with_the_surface() {
        assert!(SystemBarsStyle::from_theme(&crate::Theme::Light).light_navigation_icons);
        assert!(!SystemBarsStyle::from_theme(&crate::Theme::Dark).light_status_icons);
    }
}
