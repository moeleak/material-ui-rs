use iced_widget::core::text as core_text;
use iced_widget::core::time::{Duration, Instant};
use iced_widget::core::widget as core_widget;
use iced_widget::core::{Background, Border, Color, Point, Rectangle, touch};

use crate::tokens;

const MILLIS_PER_SECOND: f64 = 1000.0;

pub(super) fn duration_ms(milliseconds: u16) -> Duration {
    Duration::from_millis(u64::from(milliseconds))
}

pub(super) fn lerp(from: f32, to: f32, progress: f32) -> f32 {
    from + (to - from) * progress
}

pub(super) fn bool_value(value: bool) -> f32 {
    if value { 1.0 } else { 0.0 }
}

pub(super) fn solid_color(background: Background) -> Color {
    match background {
        Background::Color(color) => color,
        Background::Gradient(_) => Color::TRANSPARENT,
    }
}

pub(super) fn alpha_color(mut color: Color, alpha: f32) -> Color {
    color.a *= alpha.clamp(0.0, 1.0);
    color
}

pub(super) fn alpha_border(mut border: Border, alpha: f32) -> Border {
    border.color = alpha_color(border.color, alpha);
    border
}

pub(super) fn scaled_rect(bounds: Rectangle, width: f32, height: f32) -> Rectangle {
    Rectangle {
        x: bounds.center_x() - width / 2.0,
        y: bounds.center_y() - height / 2.0,
        width,
        height,
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct AnimatedScalar {
    pub(super) value: f32,
    velocity: f32,
    from: f32,
    initial_velocity: f32,
    pub(super) to: f32,
    started_at: Option<Instant>,
    spec: AnimationSpec,
}

#[derive(Debug, Clone, Copy)]
enum AnimationSpec {
    Cubic {
        duration: Duration,
        easing: tokens::motion::CubicBezier,
    },
    Spring {
        spring: tokens::motion::Spring,
        duration: Duration,
    },
}

impl AnimatedScalar {
    pub(super) fn new(value: f32) -> Self {
        Self {
            value,
            velocity: 0.0,
            from: value,
            initial_velocity: 0.0,
            to: value,
            started_at: None,
            spec: AnimationSpec::Cubic {
                duration: Duration::ZERO,
                easing: tokens::motion::EASING_LINEAR,
            },
        }
    }

    pub(super) fn set_target(
        &mut self,
        to: f32,
        now: Instant,
        duration: Duration,
        easing: tokens::motion::CubicBezier,
    ) {
        if (self.to - to).abs() <= f32::EPSILON {
            return;
        }

        self.from = self.value;
        self.initial_velocity = 0.0;
        self.velocity = 0.0;
        self.to = to;
        self.started_at = Some(now);
        self.spec = AnimationSpec::Cubic { duration, easing };
    }

    pub(super) fn set_spring_target(
        &mut self,
        to: f32,
        now: Instant,
        spring: tokens::motion::Spring,
    ) {
        self.set_spring_target_with_threshold(
            to,
            now,
            spring,
            tokens::motion::SPRING_DEFAULT_DISPLACEMENT_THRESHOLD,
        );
    }

    pub(super) fn set_spring_target_with_threshold(
        &mut self,
        to: f32,
        now: Instant,
        spring: tokens::motion::Spring,
        visibility_threshold: f32,
    ) {
        if (self.to - to).abs() <= f32::EPSILON {
            return;
        }

        let _ = self.advance(now);

        self.from = self.value;
        self.initial_velocity = self.velocity;
        self.to = to;
        self.started_at = Some(now);
        self.spec = AnimationSpec::Spring {
            spring,
            duration: spring_duration(
                spring,
                self.from,
                self.to,
                self.initial_velocity,
                visibility_threshold,
            ),
        };
    }

    pub(super) fn advance(&mut self, now: Instant) -> bool {
        let Some(started_at) = self.started_at else {
            self.value = self.to;
            return false;
        };

        match self.spec {
            AnimationSpec::Cubic { duration, easing } => {
                if duration.is_zero() {
                    self.value = self.to;
                    self.velocity = 0.0;
                    self.started_at = None;
                    return false;
                }

                let progress = (now.duration_since(started_at).as_secs_f32()
                    / duration.as_secs_f32())
                .clamp(0.0, 1.0);

                self.value = lerp(self.from, self.to, easing.transform(progress));

                if progress >= 1.0 {
                    self.value = self.to;
                    self.velocity = 0.0;
                    self.started_at = None;
                    false
                } else {
                    true
                }
            }
            AnimationSpec::Spring { spring, duration } => {
                let elapsed = now.duration_since(started_at);

                if elapsed >= duration {
                    self.value = self.to;
                    self.velocity = 0.0;
                    self.started_at = None;
                    return false;
                }

                let (value, velocity) = spring_value_and_velocity(
                    self.from,
                    self.initial_velocity,
                    self.to,
                    elapsed,
                    spring,
                );
                self.value = value;
                self.velocity = velocity;

                true
            }
        }
    }

    pub(super) fn is_animating(&self) -> bool {
        self.started_at.is_some()
    }
}

fn spring_duration(
    spring: tokens::motion::Spring,
    initial_value: f32,
    target_value: f32,
    initial_velocity: f32,
    visibility_threshold: f32,
) -> Duration {
    let threshold = visibility_threshold.abs().max(f32::EPSILON);
    let millis = estimate_spring_duration_millis(
        f64::from(spring.stiffness),
        f64::from(spring.damping_ratio),
        f64::from(initial_velocity / threshold),
        f64::from((initial_value - target_value) / threshold),
        1.0,
    );

    Duration::from_millis(millis)
}

fn spring_value_and_velocity(
    initial_value: f32,
    initial_velocity: f32,
    target_value: f32,
    elapsed: Duration,
    spring: tokens::motion::Spring,
) -> (f32, f32) {
    let millis = elapsed.as_nanos() / 1_000_000;
    let delta_t = millis as f64 / MILLIS_PER_SECOND;
    let damping_ratio = f64::from(spring.damping_ratio.max(0.0));
    let natural_frequency = f64::from(spring.stiffness.sqrt());
    let adjusted_displacement = f64::from(initial_value - target_value);
    let initial_velocity = f64::from(initial_velocity);

    let (displacement, velocity) = if damping_ratio > 1.0 {
        let damping_ratio_squared = damping_ratio * damping_ratio;
        let damped_frequency = natural_frequency * (damping_ratio_squared - 1.0).sqrt();
        let gamma_plus = -damping_ratio * natural_frequency + damped_frequency;
        let gamma_minus = -damping_ratio * natural_frequency - damped_frequency;
        let coeff_b =
            (gamma_plus * adjusted_displacement - initial_velocity) / (gamma_plus - gamma_minus);
        let coeff_a = adjusted_displacement - coeff_b;
        let displacement =
            coeff_a * (gamma_minus * delta_t).exp() + coeff_b * (gamma_plus * delta_t).exp();
        let velocity = coeff_a * gamma_minus * (gamma_minus * delta_t).exp()
            + coeff_b * gamma_plus * (gamma_plus * delta_t).exp();

        (displacement, velocity)
    } else if (damping_ratio - 1.0).abs() <= f64::EPSILON {
        let coeff_a = adjusted_displacement;
        let coeff_b = initial_velocity + natural_frequency * adjusted_displacement;
        let n_fdt = -natural_frequency * delta_t;
        let exp = n_fdt.exp();
        let displacement = (coeff_a + coeff_b * delta_t) * exp;
        let velocity = displacement * -natural_frequency + coeff_b * exp;

        (displacement, velocity)
    } else {
        let damping_ratio_squared = damping_ratio * damping_ratio;
        let damped_frequency = natural_frequency * (1.0 - damping_ratio_squared).sqrt();
        let r = -damping_ratio * natural_frequency;
        let cos_coeff = adjusted_displacement;
        let sin_coeff = ((-r * adjusted_displacement) + initial_velocity) / damped_frequency;
        let d_fdt = damped_frequency * delta_t;
        let exp = (r * delta_t).exp();
        let displacement = exp * (cos_coeff * d_fdt.cos() + sin_coeff * d_fdt.sin());
        let velocity = displacement * r
            + exp
                * (-damped_frequency * cos_coeff * d_fdt.sin()
                    + damped_frequency * sin_coeff * d_fdt.cos());

        (displacement, velocity)
    };

    (
        (displacement + f64::from(target_value)) as f32,
        velocity as f32,
    )
}

fn estimate_spring_duration_millis(
    stiffness: f64,
    damping_ratio: f64,
    initial_velocity: f64,
    initial_displacement: f64,
    delta: f64,
) -> u64 {
    if damping_ratio == 0.0 {
        return u64::MAX / 1_000_000;
    }

    let damping_coefficient = 2.0 * damping_ratio * stiffness.sqrt();
    let partial_root = damping_coefficient * damping_coefficient - 4.0 * stiffness;
    let partial_root_real = if partial_root < 0.0 {
        0.0
    } else {
        partial_root.sqrt()
    };
    let partial_root_imaginary = if partial_root < 0.0 {
        partial_root.abs().sqrt()
    } else {
        0.0
    };

    let first_root_real = (-damping_coefficient + partial_root_real) * 0.5;
    let first_root_imaginary = partial_root_imaginary * 0.5;
    let second_root_real = (-damping_coefficient - partial_root_real) * 0.5;

    estimate_duration_internal(
        first_root_real,
        first_root_imaginary,
        second_root_real,
        damping_ratio,
        initial_velocity,
        initial_displacement,
        delta,
    )
}

fn estimate_duration_internal(
    first_root_real: f64,
    first_root_imaginary: f64,
    second_root_real: f64,
    damping_ratio: f64,
    initial_velocity: f64,
    initial_position: f64,
    delta: f64,
) -> u64 {
    if initial_position == 0.0 && initial_velocity == 0.0 {
        return 0;
    }

    let velocity = if initial_position < 0.0 {
        -initial_velocity
    } else {
        initial_velocity
    };
    let position = initial_position.abs();

    let seconds = if damping_ratio > 1.0 {
        estimate_over_damped(first_root_real, second_root_real, position, velocity, delta)
    } else if damping_ratio < 1.0 {
        estimate_under_damped(
            first_root_real,
            first_root_imaginary,
            position,
            velocity,
            delta,
        )
    } else {
        estimate_critically_damped(first_root_real, position, velocity, delta)
    };

    (seconds.max(0.0) * MILLIS_PER_SECOND) as u64
}

fn estimate_under_damped(
    first_root_real: f64,
    first_root_imaginary: f64,
    position: f64,
    velocity: f64,
    delta: f64,
) -> f64 {
    let c1 = position;
    let c2 = (velocity - first_root_real * c1) / first_root_imaginary;
    let c = (c1 * c1 + c2 * c2).sqrt();

    (delta / c).ln() / first_root_real
}

fn estimate_critically_damped(
    first_root_real: f64,
    position: f64,
    velocity: f64,
    delta: f64,
) -> f64 {
    let r = first_root_real;
    let c1 = position;
    let c2 = velocity - r * c1;

    let t1 = (delta / c1).abs().ln() / r;
    let t2 = {
        let guess = (delta / c2).abs().ln();
        let mut t = guess;

        for _ in 0..=5 {
            t = guess - (t / r).abs().ln();
        }

        t / r
    };

    let mut t_curr = match (t1.is_finite(), t2.is_finite()) {
        (false, true) => t2,
        (true, false) => t1,
        _ => t1.max(t2),
    };

    let t_inflection = -(r * c1 + c2) / (r * c2);
    let x_inflection = c1 * (r * t_inflection).exp() + c2 * t_inflection * (r * t_inflection).exp();

    let signed_delta = if t_inflection.is_nan() || t_inflection <= 0.0 {
        -delta
    } else if t_inflection > 0.0 && -x_inflection < delta {
        if c2 < 0.0 && c1 > 0.0 {
            t_curr = 0.0;
        }

        -delta
    } else {
        t_curr = -(2.0 / r) - (c1 / c2);
        delta
    };

    let mut t_delta = f64::MAX;
    let mut iterations = 0;
    while t_delta > 0.001 && iterations < 100 {
        iterations += 1;
        let t_last = t_curr;
        t_curr = iterate_newtons_method(
            t_curr,
            |t| (c1 + c2 * t) * (r * t).exp() + signed_delta,
            |t| (c2 * (r * t + 1.0) + c1 * r) * (r * t).exp(),
        );
        t_delta = (t_last - t_curr).abs();
    }

    t_curr
}

fn estimate_over_damped(
    first_root_real: f64,
    second_root_real: f64,
    position: f64,
    velocity: f64,
    delta: f64,
) -> f64 {
    let r1 = first_root_real;
    let r2 = second_root_real;
    let c2 = (r1 * position - velocity) / (r1 - r2);
    let c1 = position - c2;

    let t1 = (delta / c1).abs().ln() / r1;
    let t2 = (delta / c2).abs().ln() / r2;

    let mut t_curr = match (t1.is_finite(), t2.is_finite()) {
        (false, true) => t2,
        (true, false) => t1,
        _ => t1.max(t2),
    };

    let t_inflection = ((c1 * r1) / (-c2 * r2)).ln() / (r2 - r1);
    let x_inflection = || c1 * (r1 * t_inflection).exp() + c2 * (r2 * t_inflection).exp();

    let signed_delta = if t_inflection.is_nan() || t_inflection <= 0.0 {
        -delta
    } else if t_inflection > 0.0 && -x_inflection() < delta {
        if c2 > 0.0 && c1 < 0.0 {
            t_curr = 0.0;
        }

        -delta
    } else {
        t_curr = (-(c2 * r2 * r2) / (c1 * r1 * r1)).ln() / (r1 - r2);
        delta
    };

    if (c1 * r1 * (r1 * t_curr).exp() + c2 * r2 * (r2 * t_curr).exp()).abs() < 0.0001 {
        return t_curr;
    }

    let mut t_delta = f64::MAX;
    let mut iterations = 0;
    while t_delta > 0.001 && iterations < 100 {
        iterations += 1;
        let t_last = t_curr;
        t_curr = iterate_newtons_method(
            t_curr,
            |t| c1 * (r1 * t).exp() + c2 * (r2 * t).exp() + signed_delta,
            |t| c1 * r1 * (r1 * t).exp() + c2 * r2 * (r2 * t).exp(),
        );
        t_delta = (t_last - t_curr).abs();
    }

    t_curr
}

fn iterate_newtons_method(x: f64, f: impl Fn(f64) -> f64, f_prime: impl Fn(f64) -> f64) -> f64 {
    x - f(x) / f_prime(x)
}

pub(super) struct SelectionState<Paragraph: core_text::Paragraph, Status> {
    pub(super) text: core_widget::text::State<Paragraph>,
    pub(super) target: bool,
    pub(super) position: AnimatedScalar,
    pub(super) color: AnimatedScalar,
    pub(super) size: AnimatedScalar,
    pub(super) icon: AnimatedScalar,
    pub(super) icon_opacity: AnimatedScalar,
    pub(super) is_pressed: bool,
    pub(super) press_origin: Option<Point>,
    pub(super) last_status: Option<Status>,
}

impl<Paragraph: core_text::Paragraph, Status> SelectionState<Paragraph, Status> {
    pub(super) fn new(target: bool) -> Self {
        let value = bool_value(target);

        Self {
            text: core_widget::text::State::<Paragraph>::default(),
            target,
            position: AnimatedScalar::new(value),
            color: AnimatedScalar::new(value),
            size: AnimatedScalar::new(value),
            icon: AnimatedScalar::new(value),
            icon_opacity: AnimatedScalar::new(value),
            is_pressed: false,
            press_origin: None,
            last_status: None,
        }
    }

    pub(super) fn is_animating(&self) -> bool {
        self.position.is_animating()
            || self.color.is_animating()
            || self.size.is_animating()
            || self.icon.is_animating()
            || self.icon_opacity.is_animating()
    }

    pub(super) fn advance(&mut self, now: Instant) -> bool {
        self.position.advance(now)
            | self.color.advance(now)
            | self.size.advance(now)
            | self.icon.advance(now)
            | self.icon_opacity.advance(now)
    }
}

pub(super) struct TextFieldState<Paragraph: core_text::Paragraph> {
    pub(super) label: core_widget::text::State<Paragraph>,
    pub(super) floating_label: core_widget::text::State<Paragraph>,
    pub(super) label_float: AnimatedScalar,
    pub(super) is_focused: bool,
    pub(super) ime_preedit_active: bool,
    pub(super) touch_activation: Option<TextFieldTouchActivation>,
}

impl<Paragraph: core_text::Paragraph> TextFieldState<Paragraph> {
    pub(super) fn new(is_populated: bool) -> Self {
        Self {
            label: core_widget::text::State::<Paragraph>::default(),
            floating_label: core_widget::text::State::<Paragraph>::default(),
            label_float: AnimatedScalar::new(bool_value(is_populated)),
            is_focused: false,
            ime_preedit_active: false,
            touch_activation: None,
        }
    }

    pub(super) fn is_animating(&self) -> bool {
        self.label_float.is_animating()
    }

    pub(super) fn set_ime_preedit(&mut self, content: &str) -> bool {
        let active = !content.is_empty();
        let changed = self.ime_preedit_active != active;

        self.ime_preedit_active = active;

        changed
    }

    pub(super) fn clear_ime_preedit(&mut self) -> bool {
        let changed = self.ime_preedit_active;

        self.ime_preedit_active = false;

        changed
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct TextFieldTouchActivation {
    finger: touch::Finger,
    start: Point,
}

impl TextFieldTouchActivation {
    pub(super) fn new(finger: touch::Finger, start: Point) -> Self {
        Self { finger, start }
    }

    pub(super) fn matches(self, finger: touch::Finger) -> bool {
        self.finger == finger
    }

    pub(super) fn moved_beyond_slop(self, position: Point, slop: f32) -> bool {
        let dx = position.x - self.start.x;
        let dy = position.y - self.start.y;

        dx * dx + dy * dy > slop * slop
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type TestParagraph = <iced_widget::Renderer as core_text::Renderer>::Paragraph;

    #[test]
    fn text_field_state_tracks_active_ime_preedit() {
        let mut state = TextFieldState::<TestParagraph>::new(false);

        assert!(!state.ime_preedit_active);
        assert!(state.set_ime_preedit("pin yin"));
        assert!(state.ime_preedit_active);
        assert!(!state.set_ime_preedit("more"));
        assert!(state.clear_ime_preedit());
        assert!(!state.ime_preedit_active);
        assert!(!state.clear_ime_preedit());
    }
}
