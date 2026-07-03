//! Small animation helpers for Material themed state.

use iced_widget::core::time::{Duration, Instant};
use iced_widget::core::{Point, Size};

use crate::{ColorScheme, tokens};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorSchemeTransition {
    from: ColorScheme,
    to: ColorScheme,
    started_at: Instant,
    duration: Duration,
    easing: tokens::motion::CubicBezier,
}

impl ColorSchemeTransition {
    pub fn new(
        from: ColorScheme,
        to: ColorScheme,
        started_at: Instant,
        duration: Duration,
        easing: tokens::motion::CubicBezier,
    ) -> Self {
        Self {
            from,
            to,
            started_at,
            duration,
            easing,
        }
    }

    pub fn material_theme(from: ColorScheme, to: ColorScheme, started_at: Instant) -> Self {
        Self::new(
            from,
            to,
            started_at,
            Duration::from_millis(u64::from(tokens::motion::DURATION_MEDIUM4_MS)),
            tokens::motion::EASING_EMPHASIZED_DECELERATE,
        )
    }

    pub fn value_at(self, now: Instant) -> ColorScheme {
        ColorScheme::interpolate(self.from, self.to, self.eased_progress_at(now))
    }

    pub fn is_finished_at(self, now: Instant) -> bool {
        self.progress_at(now) >= 1.0
    }

    pub fn progress_at(self, now: Instant) -> f32 {
        if self.duration.is_zero() {
            return 1.0;
        }

        (now.saturating_duration_since(self.started_at).as_secs_f32() / self.duration.as_secs_f32())
            .clamp(0.0, 1.0)
    }

    pub fn eased_progress_at(self, now: Instant) -> f32 {
        self.easing.transform(self.progress_at(now))
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ThemeRevealTransition {
    color_scheme: ColorSchemeTransition,
    origin: Point,
}

impl ThemeRevealTransition {
    pub fn new(
        from: ColorScheme,
        to: ColorScheme,
        origin: Point,
        started_at: Instant,
        duration: Duration,
        easing: tokens::motion::CubicBezier,
    ) -> Self {
        Self {
            color_scheme: ColorSchemeTransition::new(from, to, started_at, duration, easing),
            origin,
        }
    }

    pub fn material_theme(
        from: ColorScheme,
        to: ColorScheme,
        origin: Point,
        started_at: Instant,
    ) -> Self {
        Self::new(
            from,
            to,
            origin,
            started_at,
            Duration::from_millis(u64::from(tokens::motion::DURATION_MEDIUM4_MS)),
            tokens::motion::EASING_EMPHASIZED_DECELERATE,
        )
    }

    pub fn value_at(self, now: Instant) -> ColorScheme {
        self.color_scheme.value_at(now)
    }

    pub fn target(self) -> ColorScheme {
        self.color_scheme.to
    }

    pub fn origin(self) -> Point {
        self.origin
    }

    pub fn is_finished_at(self, now: Instant) -> bool {
        self.color_scheme.is_finished_at(now)
    }

    pub fn progress_at(self, now: Instant) -> f32 {
        self.color_scheme.progress_at(now)
    }

    pub fn eased_progress_at(self, now: Instant) -> f32 {
        self.color_scheme.eased_progress_at(now)
    }

    pub fn reveal_radius_at(self, viewport: Size, now: Instant) -> f32 {
        max_radius_from_origin(self.origin, viewport) * self.eased_progress_at(now)
    }
}

pub fn max_radius_from_origin(origin: Point, viewport: Size) -> f32 {
    [
        Point::ORIGIN,
        Point::new(viewport.width, 0.0),
        Point::new(0.0, viewport.height),
        Point::new(viewport.width, viewport.height),
    ]
    .into_iter()
    .map(|corner| distance(origin, corner))
    .fold(0.0, f32::max)
}

fn distance(a: Point, b: Point) -> f32 {
    let x = a.x - b.x;
    let y = a.y - b.y;

    x.hypot(y)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Theme;

    #[test]
    fn material_theme_transition_uses_m3_easing_and_duration() {
        let start = Instant::now();
        let transition = ColorSchemeTransition::material_theme(
            Theme::Dark.colors(),
            Theme::Light.colors(),
            start,
        );

        assert_eq!(transition.duration, Duration::from_millis(400));
        assert_eq!(
            transition.easing,
            tokens::motion::EASING_EMPHASIZED_DECELERATE
        );
        assert_eq!(transition.progress_at(start), 0.0);
        assert!(!transition.is_finished_at(start + Duration::from_millis(200)));
        assert!(transition.is_finished_at(start + Duration::from_millis(400)));
    }

    #[test]
    fn color_scheme_transition_reaches_target() {
        let start = Instant::now();
        let target = Theme::Light.colors();
        let transition = ColorSchemeTransition::material_theme(Theme::Dark.colors(), target, start);

        assert_eq!(
            transition.value_at(start + Duration::from_millis(500)),
            target
        );
    }

    #[test]
    fn theme_reveal_transition_tracks_origin_and_radius() {
        let start = Instant::now();
        let origin = Point::new(3.0, 4.0);
        let target = Theme::Light.colors();
        let transition =
            ThemeRevealTransition::material_theme(Theme::Dark.colors(), target, origin, start);

        assert_eq!(transition.origin(), origin);
        assert_eq!(transition.reveal_radius_at(Size::new(6.0, 8.0), start), 0.0);
        assert!(
            transition.reveal_radius_at(Size::new(6.0, 8.0), start + Duration::from_millis(200))
                > 0.0
        );
        assert_eq!(transition.target(), target);
    }
}
