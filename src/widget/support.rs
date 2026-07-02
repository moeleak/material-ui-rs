use iced_widget::core::text as core_text;
use iced_widget::core::time::{Duration, Instant};
use iced_widget::core::widget as core_widget;
use iced_widget::core::{Background, Border, Color, Rectangle};

use crate::tokens;

pub(super) fn duration_ms(milliseconds: u16) -> Duration {
    Duration::from_millis(u64::from(milliseconds))
}

pub(super) fn lerp(from: f32, to: f32, progress: f32) -> f32 {
    from + (to - from) * progress
}

fn bezier_axis(t: f32, p1: f32, p2: f32) -> f32 {
    let one_minus_t = 1.0 - t;

    3.0 * one_minus_t * one_minus_t * t * p1 + 3.0 * one_minus_t * t * t * p2 + t * t * t
}

fn cubic_bezier(progress: f32, easing: tokens::motion::CubicBezier) -> f32 {
    let target_x = progress.clamp(0.0, 1.0);
    let mut start = 0.0;
    let mut end = 1.0;

    for _ in 0..20 {
        let midpoint = (start + end) / 2.0;

        if bezier_axis(midpoint, easing.x1, easing.x2) < target_x {
            start = midpoint;
        } else {
            end = midpoint;
        }
    }

    bezier_axis((start + end) / 2.0, easing.y1, easing.y2)
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
    from: f32,
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
        settle_after: Duration,
    },
}

impl AnimatedScalar {
    pub(super) fn new(value: f32) -> Self {
        Self {
            value,
            from: value,
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
        if (self.to - to).abs() <= f32::EPSILON {
            return;
        }

        self.from = self.value;
        self.to = to;
        self.started_at = Some(now);
        self.spec = AnimationSpec::Spring {
            spring,
            settle_after: spring_settle_duration(spring),
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
                    self.started_at = None;
                    return false;
                }

                let progress = (now.duration_since(started_at).as_secs_f32()
                    / duration.as_secs_f32())
                .clamp(0.0, 1.0);

                self.value = lerp(self.from, self.to, cubic_bezier(progress, easing));

                if progress >= 1.0 {
                    self.value = self.to;
                    self.started_at = None;
                    false
                } else {
                    true
                }
            }
            AnimationSpec::Spring {
                spring,
                settle_after,
            } => {
                let elapsed = now.duration_since(started_at);

                if elapsed >= settle_after {
                    self.value = self.to;
                    self.started_at = None;
                    return false;
                }

                self.value = lerp(
                    self.from,
                    self.to,
                    spring_progress(elapsed.as_secs_f32(), spring),
                );

                true
            }
        }
    }

    fn is_animating(&self) -> bool {
        self.started_at.is_some()
    }
}

fn spring_settle_duration(spring: tokens::motion::Spring) -> Duration {
    let natural_frequency = spring.stiffness.sqrt();
    let damping = (spring.damping_ratio * natural_frequency).max(1.0);
    let seconds = (1000.0_f32.ln() / damping).clamp(0.05, 1.2);

    Duration::from_secs_f32(seconds)
}

fn spring_progress(seconds: f32, spring: tokens::motion::Spring) -> f32 {
    let damping_ratio = spring.damping_ratio.max(0.0);
    let natural_frequency = spring.stiffness.sqrt();

    if damping_ratio < 1.0 {
        let damped_frequency = natural_frequency * (1.0 - damping_ratio * damping_ratio).sqrt();
        let envelope = (-damping_ratio * natural_frequency * seconds).exp();
        let oscillation = (damped_frequency * seconds).cos()
            + (damping_ratio / (1.0 - damping_ratio * damping_ratio).sqrt())
                * (damped_frequency * seconds).sin();

        1.0 - envelope * oscillation
    } else {
        let envelope = (-natural_frequency * seconds).exp();

        1.0 - envelope * (1.0 + natural_frequency * seconds)
    }
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
    pub(super) label_float: AnimatedScalar,
    pub(super) is_focused: bool,
}

impl<Paragraph: core_text::Paragraph> TextFieldState<Paragraph> {
    pub(super) fn new(is_populated: bool) -> Self {
        Self {
            label: core_widget::text::State::<Paragraph>::default(),
            label_float: AnimatedScalar::new(bool_value(is_populated)),
            is_focused: false,
        }
    }

    pub(super) fn is_animating(&self) -> bool {
        self.label_float.is_animating()
    }
}
