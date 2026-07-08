//! Canvas-based Material 3 progress and loading indicators.

use iced_widget::canvas::{self, Canvas, LineCap, LineJoin, Path, Stroke};
use iced_widget::core::time::{Duration, Instant};
use iced_widget::core::{Color, Length, Point, Rectangle, mouse};
use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, TAU};

use crate::{Theme, tokens};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LinearMode {
    Determinate,
    Indeterminate,
}

/// A clock-backed state for indeterminate canvas indicators.
#[derive(Debug, Clone)]
pub struct IndeterminateState {
    started_at: Instant,
    elapsed: Duration,
}

impl IndeterminateState {
    /// Creates a new indeterminate animation state.
    pub fn new(started_at: Instant) -> Self {
        Self {
            started_at,
            elapsed: Duration::ZERO,
        }
    }

    /// Advances the state to `now`.
    pub fn advance(&mut self, now: Instant) {
        self.elapsed = now.saturating_duration_since(self.started_at);
    }

    /// Returns the current phase for Material linear progress keyframes.
    pub fn linear_phase(&self) -> f32 {
        elapsed_phase(
            self.elapsed,
            tokens::component::linear_progress::INDETERMINATE_DURATION_MS,
        )
    }

    /// Returns the slower phase used by the four-color linear progress cycle.
    pub fn color_phase(&self) -> f32 {
        elapsed_phase(
            self.elapsed,
            tokens::component::linear_progress::INDETERMINATE_DURATION_MS * 2,
        )
    }

    /// Returns the current phase for expressive loading indicator rotation.
    pub fn loading_phase(&self) -> f32 {
        elapsed_phase(
            self.elapsed,
            tokens::component::loading_indicator::GLOBAL_ROTATION_DURATION_MS,
        )
    }

    /// Indeterminate indicators animate for as long as they are displayed.
    pub const fn is_animating(&self) -> bool {
        true
    }
}

impl Default for IndeterminateState {
    fn default() -> Self {
        Self::new(Instant::now())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LinearProgress {
    mode: LinearMode,
    progress: f32,
    phase: f32,
    color_phase: f32,
    four_color: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LinearProgressMode {
    Determinate { progress: f32, phase: f32 },
    Indeterminate { phase: f32 },
    FourColorIndeterminate { phase: f32, color_phase: f32 },
}

impl LinearProgressMode {
    pub const fn determinate(progress: f32, phase: f32) -> Self {
        Self::Determinate { progress, phase }
    }

    pub const fn indeterminate(phase: f32) -> Self {
        Self::Indeterminate { phase }
    }

    pub fn four_color_indeterminate(phase: f32) -> Self {
        Self::FourColorIndeterminate {
            phase,
            color_phase: phase * 0.5,
        }
    }

    pub const fn four_color_indeterminate_with_color_phase(phase: f32, color_phase: f32) -> Self {
        Self::FourColorIndeterminate { phase, color_phase }
    }
}

/// Creates a Material linear progress indicator.
pub fn linear<'a, Message, Renderer>(
    mode: LinearProgressMode,
) -> Canvas<LinearProgress, Message, Theme, Renderer>
where
    Renderer: iced_widget::graphics::geometry::Renderer + 'a,
{
    let (mode, progress, phase, color_phase, four_color) = match mode {
        LinearProgressMode::Determinate { progress, phase } => (
            LinearMode::Determinate,
            progress.clamp(0.0, 1.0),
            phase,
            phase,
            false,
        ),
        LinearProgressMode::Indeterminate { phase } => {
            (LinearMode::Indeterminate, 0.0, phase, phase, false)
        }
        LinearProgressMode::FourColorIndeterminate { phase, color_phase } => {
            (LinearMode::Indeterminate, 0.0, phase, color_phase, true)
        }
    };

    Canvas::new(LinearProgress {
        mode,
        progress,
        phase,
        color_phase,
        four_color,
    })
    .width(Length::Fill)
    .height(Length::Fixed(
        tokens::component::linear_progress::WAVE_HEIGHT,
    ))
}

impl<Message, Renderer> canvas::Program<Message, Theme, Renderer> for LinearProgress
where
    Renderer: iced_widget::graphics::geometry::Renderer,
{
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry<Renderer>> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let colors = theme.colors();

        let active = if self.four_color {
            four_color_indicator(
                colors.primary.color,
                colors.primary.container,
                colors.tertiary.color,
                colors.tertiary.container,
                self.color_phase,
            )
        } else {
            colors.primary.color
        };

        let track = colors.surface.container.highest;

        match self.mode {
            LinearMode::Determinate => {
                draw_linear_determinate_track(&mut frame, track, active, self.progress);
                draw_linear_determinate(&mut frame, active, self.progress, self.phase);
            }
            LinearMode::Indeterminate => {
                let bars = indeterminate_bars(self.phase);

                draw_linear_indeterminate_track(&mut frame, track, &bars);

                for (index, bar) in bars.into_iter().enumerate() {
                    draw_indeterminate_bar(
                        &mut frame,
                        active,
                        bar,
                        self.phase + index as f32 * 0.25,
                    );
                }
            }
        }

        vec![frame.into_geometry()]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LoadingMode {
    Uncontained,
    Contained,
}

#[derive(Debug, Clone, Copy)]
pub struct LoadingIndicator {
    mode: LoadingMode,
    progress: Option<f32>,
    phase: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LoadingIndicatorMode {
    Indeterminate { phase: f32 },
    ContainedIndeterminate { phase: f32 },
    Determinate { progress: f32 },
    ContainedDeterminate { progress: f32 },
}

impl LoadingIndicatorMode {
    pub const fn indeterminate(phase: f32) -> Self {
        Self::Indeterminate { phase }
    }

    pub const fn contained_indeterminate(phase: f32) -> Self {
        Self::ContainedIndeterminate { phase }
    }

    pub const fn determinate(progress: f32) -> Self {
        Self::Determinate { progress }
    }

    pub const fn contained_determinate(progress: f32) -> Self {
        Self::ContainedDeterminate { progress }
    }
}

/// Creates an expressive loading indicator.
pub fn loading<'a, Message, Renderer>(
    mode: LoadingIndicatorMode,
) -> Canvas<LoadingIndicator, Message, Theme, Renderer>
where
    Renderer: iced_widget::graphics::geometry::Renderer + 'a,
{
    let (mode, progress, phase) = match mode {
        LoadingIndicatorMode::Indeterminate { phase } => (LoadingMode::Uncontained, None, phase),
        LoadingIndicatorMode::ContainedIndeterminate { phase } => {
            (LoadingMode::Contained, None, phase)
        }
        LoadingIndicatorMode::Determinate { progress } => (
            LoadingMode::Uncontained,
            Some(progress.clamp(0.0, 1.0)),
            0.0,
        ),
        LoadingIndicatorMode::ContainedDeterminate { progress } => {
            (LoadingMode::Contained, Some(progress.clamp(0.0, 1.0)), 0.0)
        }
    };

    Canvas::new(LoadingIndicator {
        mode,
        progress,
        phase,
    })
    .width(Length::Fixed(
        tokens::component::loading_indicator::CONTAINER_WIDTH,
    ))
    .height(Length::Fixed(
        tokens::component::loading_indicator::CONTAINER_HEIGHT,
    ))
}

impl<Message, Renderer> canvas::Program<Message, Theme, Renderer> for LoadingIndicator
where
    Renderer: iced_widget::graphics::geometry::Renderer,
{
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry<Renderer>> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let colors = theme.colors();

        let (container, active) = match self.mode {
            LoadingMode::Uncontained => (None, colors.primary.color),
            LoadingMode::Contained => (
                Some(colors.primary.container),
                colors.primary.container_text,
            ),
        };

        if let Some(color) = container {
            let container = Path::circle(frame.center(), frame.width().min(frame.height()) / 2.0);
            frame.fill(&container, color);
        }

        let side = frame.width().min(frame.height());
        let path = if let Some(progress) = self.progress {
            determinate_loading_shape_path(frame.center(), side, progress)
        } else {
            loading_shape_path(frame.center(), side, self.phase)
        };
        frame.fill(&path, active);

        vec![frame.into_geometry()]
    }
}

fn elapsed_phase(elapsed: Duration, duration_ms: u16) -> f32 {
    let duration = f32::from(duration_ms) / 1000.0;

    if duration <= 0.0 {
        return 0.0;
    }

    (elapsed.as_secs_f32() / duration).rem_euclid(1.0)
}

fn draw_linear_determinate_track<Renderer>(
    frame: &mut canvas::Frame<Renderer>,
    track: Color,
    stop: Color,
    progress: f32,
) where
    Renderer: iced_widget::graphics::geometry::Renderer,
{
    let width = frame.width();
    let height = frame.height();
    let y = height / 2.0;
    let stroke_width = tokens::component::linear_progress::TRACK_THICKNESS;
    let left = stroke_width / 2.0;
    let stop_size = tokens::component::linear_progress::STOP_SIZE;
    let stop_center_x =
        width - tokens::component::linear_progress::STOP_TRAILING_SPACE - stop_size / 2.0;
    let right = (stop_center_x - stop_size / 2.0).max(left);
    let active_end = left + (right - left) * progress.clamp(0.0, 1.0);
    let track_start =
        (active_end + tokens::component::linear_progress::TRACK_ACTIVE_SPACE + stroke_width)
            .clamp(left, right);

    if track_start < right {
        frame.stroke(
            &Path::line(Point::new(track_start, y), Point::new(right, y)),
            round_stroke(track, stroke_width),
        );
    }

    let stop_radius = linear_stop_radius(progress, width);
    if stop_radius > 0.0 {
        frame.fill(
            &Path::circle(Point::new(stop_center_x, y), stop_radius),
            stop,
        );
    }
}

fn draw_linear_indeterminate_track<Renderer>(
    frame: &mut canvas::Frame<Renderer>,
    track: Color,
    bars: &[IndeterminateBar; 2],
) where
    Renderer: iced_widget::graphics::geometry::Renderer,
{
    let stroke_width = tokens::component::linear_progress::TRACK_THICKNESS;
    let left = stroke_width / 2.0;
    let right = frame.width() - stroke_width / 2.0;
    let y = frame.height() / 2.0;
    let gap = tokens::component::linear_progress::TRACK_ACTIVE_SPACE + stroke_width;
    let mut cursor = left;

    let mut ranges = [
        linear_bar_range(bars[0], left, right),
        linear_bar_range(bars[1], left, right),
    ];
    ranges.sort_by(|a, b| a.0.total_cmp(&b.0));

    for (start, end) in ranges {
        if end <= start {
            continue;
        }

        let track_end = (start - gap).clamp(left, right);
        if track_end > cursor {
            frame.stroke(
                &Path::line(Point::new(cursor, y), Point::new(track_end, y)),
                round_stroke(track, stroke_width),
            );
        }

        cursor = cursor.max((end + gap).clamp(left, right));
    }

    if cursor < right {
        frame.stroke(
            &Path::line(Point::new(cursor, y), Point::new(right, y)),
            round_stroke(track, stroke_width),
        );
    }
}

fn linear_stop_radius(progress: f32, width: f32) -> f32 {
    let stop_size = tokens::component::linear_progress::STOP_SIZE;
    let stroke_width = tokens::component::linear_progress::TRACK_THICKNESS;
    let stop_x = width - tokens::component::linear_progress::STOP_TRAILING_SPACE - stop_size;
    let progress_x = width * progress.clamp(0.0, 1.0) + stroke_width / 2.0;
    let size = if stop_x <= progress_x {
        (stop_size - (progress_x - stop_x)).max(0.0)
    } else {
        stop_size
    };

    size / 2.0
}

fn draw_linear_determinate<Renderer>(
    frame: &mut canvas::Frame<Renderer>,
    active: Color,
    progress: f32,
    phase: f32,
) where
    Renderer: iced_widget::graphics::geometry::Renderer,
{
    let stroke_width = tokens::component::linear_progress::ACTIVE_INDICATOR_HEIGHT;
    let left = stroke_width / 2.0;
    let right = frame.width()
        - tokens::component::linear_progress::STOP_TRAILING_SPACE
        - tokens::component::linear_progress::STOP_SIZE;
    let end = left + (right - left).max(0.0) * progress.clamp(0.0, 1.0);
    let amplitude = tokens::component::linear_progress::ACTIVE_WAVE_AMPLITUDE
        * determinate_wave_amplitude(progress);

    if end <= left {
        return;
    }

    let path = wave_path(
        left,
        end,
        frame.height() / 2.0,
        amplitude,
        tokens::component::linear_progress::ACTIVE_WAVE_WAVELENGTH,
        phase,
    );
    frame.stroke(&path, round_stroke(active, stroke_width));
}

fn draw_indeterminate_bar<Renderer>(
    frame: &mut canvas::Frame<Renderer>,
    active: Color,
    bar: IndeterminateBar,
    wave_phase: f32,
) where
    Renderer: iced_widget::graphics::geometry::Renderer,
{
    let stroke_width = tokens::component::linear_progress::ACTIVE_INDICATOR_HEIGHT;
    let left = stroke_width / 2.0;
    let right = frame.width() - stroke_width / 2.0;
    let (start, end) = linear_bar_range(bar, left, right);

    if end <= start {
        return;
    }

    let path = wave_path(
        start,
        end,
        frame.height() / 2.0,
        tokens::component::linear_progress::ACTIVE_WAVE_AMPLITUDE,
        tokens::component::linear_progress::INDETERMINATE_ACTIVE_WAVE_WAVELENGTH,
        wave_phase,
    );

    frame.stroke(&path, round_stroke(active, stroke_width));
}

fn determinate_wave_amplitude(progress: f32) -> f32 {
    let progress = progress.clamp(0.0, 1.0);

    if progress <= 0.1 || progress >= 0.95 {
        0.0
    } else {
        1.0
    }
}

fn linear_bar_range(bar: IndeterminateBar, left: f32, right: f32) -> (f32, f32) {
    let width = (right - left).max(0.0);
    let start = left + width * bar.tail.clamp(0.0, 1.0);
    let end = left + width * bar.head.clamp(0.0, 1.0);

    if end >= start {
        (start, end)
    } else {
        (end, start)
    }
}

fn round_stroke(color: Color, width: f32) -> Stroke<'static> {
    Stroke::default()
        .with_color(color)
        .with_width(width)
        .with_line_cap(LineCap::Round)
        .with_line_join(LineJoin::Round)
}

fn wave_path(start: f32, end: f32, y: f32, amplitude: f32, wavelength: f32, phase: f32) -> Path {
    let length = (end - start).max(0.0);
    let step = 3.0_f32.max(wavelength / 12.0);

    Path::new(|path| {
        path.move_to(Point::new(
            start,
            y + wave_offset(0.0, amplitude, wavelength, phase),
        ));

        let mut distance = step;
        while distance < length {
            let x = start + distance;
            path.line_to(Point::new(
                x,
                y + wave_offset(distance, amplitude, wavelength, phase),
            ));
            distance += step;
        }

        path.line_to(Point::new(
            end,
            y + wave_offset(length, amplitude, wavelength, phase),
        ));
    })
}

fn wave_offset(distance: f32, amplitude: f32, wavelength: f32, phase: f32) -> f32 {
    if wavelength <= 0.0 {
        return 0.0;
    }

    ((distance / wavelength) * TAU + phase.rem_euclid(1.0) * TAU).sin() * amplitude
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct IndeterminateBar {
    tail: f32,
    head: f32,
}

fn indeterminate_bars(phase: f32) -> [IndeterminateBar; 2] {
    [
        IndeterminateBar {
            tail: indeterminate_keyframe_progress(
                phase,
                tokens::component::linear_progress::FIRST_LINE_TAIL_DELAY_MS,
                tokens::component::linear_progress::FIRST_LINE_TAIL_DURATION_MS,
            ),
            head: indeterminate_keyframe_progress(
                phase,
                tokens::component::linear_progress::FIRST_LINE_HEAD_DELAY_MS,
                tokens::component::linear_progress::FIRST_LINE_HEAD_DURATION_MS,
            ),
        },
        IndeterminateBar {
            tail: indeterminate_keyframe_progress(
                phase,
                tokens::component::linear_progress::SECOND_LINE_TAIL_DELAY_MS,
                tokens::component::linear_progress::SECOND_LINE_TAIL_DURATION_MS,
            ),
            head: indeterminate_keyframe_progress(
                phase,
                tokens::component::linear_progress::SECOND_LINE_HEAD_DELAY_MS,
                tokens::component::linear_progress::SECOND_LINE_HEAD_DURATION_MS,
            ),
        },
    ]
}

fn indeterminate_keyframe_progress(phase: f32, delay_ms: u16, duration_ms: u16) -> f32 {
    let elapsed_ms = phase.rem_euclid(1.0)
        * f32::from(tokens::component::linear_progress::INDETERMINATE_DURATION_MS);
    let delay_ms = f32::from(delay_ms);
    let duration_ms = f32::from(duration_ms);

    if elapsed_ms <= delay_ms {
        return 0.0;
    }

    if elapsed_ms >= delay_ms + duration_ms {
        return 1.0;
    }

    tokens::motion::EASING_EMPHASIZED_ACCELERATE.transform((elapsed_ms - delay_ms) / duration_ms)
}

fn four_color_indicator(
    primary: Color,
    primary_container: Color,
    tertiary: Color,
    tertiary_container: Color,
    phase: f32,
) -> Color {
    let phase = phase.rem_euclid(1.0);

    if !(0.15..0.25).contains(&phase)
        && !(0.40..0.50).contains(&phase)
        && !(0.65..0.75).contains(&phase)
        && !(0.90..1.0).contains(&phase)
    {
        if phase < 0.25 || phase >= 0.90 {
            return primary;
        }
        if phase < 0.50 {
            return primary_container;
        }
        if phase < 0.75 {
            return tertiary;
        }

        return tertiary_container;
    }

    if phase < 0.25 {
        color_lerp(primary, primary_container, (phase - 0.15) / 0.10)
    } else if phase < 0.50 {
        color_lerp(primary_container, tertiary, (phase - 0.40) / 0.10)
    } else if phase < 0.75 {
        color_lerp(tertiary, tertiary_container, (phase - 0.65) / 0.10)
    } else {
        color_lerp(tertiary_container, primary, (phase - 0.90) / 0.10)
    }
}

fn color_lerp(from: Color, to: Color, progress: f32) -> Color {
    let progress = progress.clamp(0.0, 1.0);

    Color {
        r: from.r + (to.r - from.r) * progress,
        g: from.g + (to.g - from.g) * progress,
        b: from.b + (to.b - from.b) * progress,
        a: from.a + (to.a - from.a) * progress,
    }
}

fn loading_shape_path(center: Point, side: f32, phase: f32) -> Path {
    let phase = phase.rem_euclid(1.0);
    let polygons = indeterminate_loading_polygons();
    let morphs = morph_sequence(&polygons, true);
    let scale_factor = loading_shape_scale(&polygons);
    let morph_position = (phase
        * f32::from(tokens::component::loading_indicator::GLOBAL_ROTATION_DURATION_MS)
        / f32::from(tokens::component::loading_indicator::MORPH_INTERVAL_MS))
    .rem_euclid(morphs.len() as f32);
    let from_index = morph_position.floor() as usize;
    let local_progress = morph_position.fract();
    let morph_progress = loading_spring_progress(local_progress);
    let rotation = phase * TAU + (from_index as f32 + 1.0 + morph_progress) * FRAC_PI_2;

    morphed_loading_shape_path(
        &morphs[from_index],
        center,
        side,
        scale_factor,
        morph_progress,
        rotation,
    )
}

fn determinate_loading_shape_path(center: Point, side: f32, progress: f32) -> Path {
    let progress = progress.clamp(0.0, 1.0);
    let polygons = determinate_loading_polygons();
    let morphs = morph_sequence(&polygons, false);
    let scale_factor = loading_shape_scale(&polygons);
    let rotation = -progress * std::f32::consts::PI;

    morphed_loading_shape_path(&morphs[0], center, side, scale_factor, progress, rotation)
}

fn morphed_loading_shape_path(
    morph: &Morph,
    center: Point,
    side: f32,
    scale_factor: f32,
    morph_progress: f32,
    rotation: f32,
) -> Path {
    let cubics = morph.as_cubics(morph_progress);

    processed_cubic_path(&cubics, center, side, scale_factor, rotation)
}

fn loading_spring_progress(progress: f32) -> f32 {
    let seconds = progress.clamp(0.0, 1.0)
        * f32::from(tokens::component::loading_indicator::MORPH_INTERVAL_MS)
        / 1000.0;
    let damping_ratio = tokens::component::loading_indicator::MORPH_SPRING_DAMPING_RATIO;
    let stiffness = tokens::component::loading_indicator::MORPH_SPRING_STIFFNESS;
    let natural_frequency = stiffness.sqrt();

    if damping_ratio >= 1.0 {
        return (1.0 - (-natural_frequency * seconds).exp()).clamp(0.0, 1.0);
    }

    let damped_frequency = natural_frequency * (1.0 - damping_ratio * damping_ratio).sqrt();
    let envelope = (-damping_ratio * natural_frequency * seconds).exp();
    let phase = damped_frequency * seconds;
    let response = 1.0
        - envelope
            * (phase.cos()
                + damping_ratio / (1.0 - damping_ratio * damping_ratio).sqrt() * phase.sin());

    response.clamp(0.0, 1.0)
}

fn processed_cubic_path(
    cubics: &[Cubic],
    center: Point,
    side: f32,
    scale_factor: f32,
    rotation: f32,
) -> Path {
    if cubics.is_empty() {
        return Path::new(|_| {});
    }

    let transformed = processed_cubics(cubics, center, side, scale_factor, rotation);

    Path::new(|path| {
        path.move_to(Point::new(
            transformed[0].anchor0_x(),
            transformed[0].anchor0_y(),
        ));

        for cubic in &transformed {
            path.bezier_curve_to(
                Point::new(cubic.control0_x(), cubic.control0_y()),
                Point::new(cubic.control1_x(), cubic.control1_y()),
                Point::new(cubic.anchor1_x(), cubic.anchor1_y()),
            );
        }

        path.close();
    })
}

fn processed_cubics(
    cubics: &[Cubic],
    center: Point,
    side: f32,
    scale_factor: f32,
    rotation: f32,
) -> Vec<Cubic> {
    if cubics.is_empty() {
        return Vec::new();
    }

    let scale = side * scale_factor;
    let transformed: Vec<Cubic> = cubics
        .iter()
        .map(|cubic| cubic.transformed(|point| Point::new(point.x * scale, point.y * scale)))
        .collect();
    let bounds = cubics_bounds(&transformed, false);
    let bounds_center = bounds_center(bounds);
    let translation = point_sub(center, bounds_center);

    transformed
        .into_iter()
        .map(|cubic| {
            cubic.transformed(|point| {
                rotate_point_around(point_add(point, translation), center, rotation)
            })
        })
        .collect()
}

fn loading_shape_scale(polygons: &[RoundedPolygon]) -> f32 {
    let mut scale_factor = 1.0_f32;

    for polygon in polygons {
        let bounds = polygon.calculate_bounds(true);
        let max_bounds = polygon.calculate_max_bounds();
        let scale_x = bounds_width(bounds) / bounds_width(max_bounds);
        let scale_y = bounds_height(bounds) / bounds_height(max_bounds);

        scale_factor = scale_factor.min(scale_x.max(scale_y));
    }

    scale_factor * tokens::component::loading_indicator::ACTIVE_INDICATOR_SCALE
}

fn indeterminate_loading_polygons() -> Vec<RoundedPolygon> {
    vec![
        material_soft_burst(),
        material_cookie9(),
        material_pentagon(),
        material_pill(),
        material_sunny(),
        material_cookie4(),
        material_oval(),
    ]
}

fn determinate_loading_polygons() -> Vec<RoundedPolygon> {
    vec![
        material_circle().transformed(|point| rotate_point(point, TAU / 20.0)),
        material_soft_burst(),
    ]
}

fn morph_sequence(polygons: &[RoundedPolygon], circular_sequence: bool) -> Vec<Morph> {
    let mut morphs = Vec::new();

    for index in 0..polygons.len() {
        if index + 1 < polygons.len() {
            morphs.push(Morph::new(
                polygons[index].normalized(),
                polygons[index + 1].normalized(),
            ));
        } else if circular_sequence {
            morphs.push(Morph::new(
                polygons[index].normalized(),
                polygons[0].normalized(),
            ));
        }
    }

    morphs
}

fn material_circle() -> RoundedPolygon {
    rounded_polygon_circle(10, 1.0, Point::ORIGIN).normalized()
}

fn material_oval() -> RoundedPolygon {
    rounded_polygon_circle(8, 1.0, Point::ORIGIN)
        .transformed(|point| Point::new(point.x, point.y * 0.64))
        .transformed(|point| rotate_point(point, -FRAC_PI_4))
        .normalized()
}

fn material_pill() -> RoundedPolygon {
    custom_material_polygon(
        &[
            ShapeVertex::new(0.961, 0.039, CornerRounding::new(0.426)),
            ShapeVertex::new(1.001, 0.428, CornerRounding::UNROUNDED),
            ShapeVertex::new(1.000, 0.609, CornerRounding::new(1.0)),
        ],
        2,
        true,
    )
    .normalized()
}

fn material_pentagon() -> RoundedPolygon {
    custom_material_polygon(
        &[
            ShapeVertex::new(0.500, -0.009, CornerRounding::new(0.172)),
            ShapeVertex::new(1.030, 0.365, CornerRounding::new(0.164)),
            ShapeVertex::new(0.828, 0.970, CornerRounding::new(0.169)),
        ],
        1,
        true,
    )
    .normalized()
}

fn material_sunny() -> RoundedPolygon {
    rounded_polygon_star(8, 1.0, 0.8, CornerRounding::new(0.15), Point::ORIGIN).normalized()
}

fn material_cookie4() -> RoundedPolygon {
    custom_material_polygon(
        &[
            ShapeVertex::new(1.237, 1.236, CornerRounding::new(0.258)),
            ShapeVertex::new(0.500, 0.918, CornerRounding::new(0.233)),
        ],
        4,
        false,
    )
    .normalized()
}

fn material_cookie9() -> RoundedPolygon {
    rounded_polygon_star(9, 1.0, 0.8, CornerRounding::new(0.5), Point::ORIGIN)
        .transformed(|point| rotate_point(point, -FRAC_PI_2))
        .normalized()
}

fn material_soft_burst() -> RoundedPolygon {
    custom_material_polygon(
        &[
            ShapeVertex::new(0.193, 0.277, CornerRounding::new(0.053)),
            ShapeVertex::new(0.176, 0.055, CornerRounding::new(0.053)),
        ],
        10,
        false,
    )
    .normalized()
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct ShapeVertex {
    point: Point,
    rounding: CornerRounding,
}

impl ShapeVertex {
    fn new(x: f32, y: f32, rounding: CornerRounding) -> Self {
        Self {
            point: Point::new(x, y),
            rounding,
        }
    }
}

fn custom_material_polygon(points: &[ShapeVertex], reps: usize, mirroring: bool) -> RoundedPolygon {
    let center = Point::new(0.5, 0.5);
    let repeated = repeat_material_vertices(points, reps, center, mirroring);
    let vertices: Vec<Point> = repeated.iter().map(|vertex| vertex.point).collect();
    let roundings: Vec<CornerRounding> = repeated.iter().map(|vertex| vertex.rounding).collect();

    RoundedPolygon::from_vertices(&vertices, &roundings, Some(center))
}

fn repeat_material_vertices(
    points: &[ShapeVertex],
    reps: usize,
    center: Point,
    mirroring: bool,
) -> Vec<ShapeVertex> {
    if mirroring {
        let angles: Vec<f32> = points
            .iter()
            .map(|vertex| (vertex.point.y - center.y).atan2(vertex.point.x - center.x))
            .collect();
        let distances: Vec<f32> = points
            .iter()
            .map(|vertex| point_distance(point_sub(vertex.point, center)))
            .collect();
        let actual_reps = reps * 2;
        let section_angle = TAU / actual_reps as f32;
        let mut vertices = Vec::with_capacity(points.len() * actual_reps);

        for rep in 0..actual_reps {
            for index in 0..points.len() {
                let source = if rep % 2 == 0 {
                    index
                } else {
                    points.len() - 1 - index
                };

                if source > 0 || rep % 2 == 0 {
                    let angle = section_angle * rep as f32
                        + if rep % 2 == 0 {
                            angles[source]
                        } else {
                            section_angle - angles[source] + 2.0 * angles[0]
                        };

                    vertices.push(ShapeVertex::new(
                        center.x + angle.cos() * distances[source],
                        center.y + angle.sin() * distances[source],
                        points[source].rounding,
                    ));
                }
            }
        }

        vertices
    } else {
        let mut vertices = Vec::with_capacity(points.len() * reps);

        for index in 0..points.len() * reps {
            let source = index % points.len();
            let rep = index / points.len();
            let point =
                rotate_point_around(points[source].point, center, rep as f32 * TAU / reps as f32);

            vertices.push(ShapeVertex {
                point,
                rounding: points[source].rounding,
            });
        }

        vertices
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct CornerRounding {
    radius: f32,
    smoothing: f32,
}

impl CornerRounding {
    const UNROUNDED: Self = Self {
        radius: 0.0,
        smoothing: 0.0,
    };

    const fn new(radius: f32) -> Self {
        Self {
            radius,
            smoothing: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Cubic {
    points: [f32; 8],
}

impl Cubic {
    fn new(
        anchor0_x: f32,
        anchor0_y: f32,
        control0_x: f32,
        control0_y: f32,
        control1_x: f32,
        control1_y: f32,
        anchor1_x: f32,
        anchor1_y: f32,
    ) -> Self {
        Self {
            points: [
                anchor0_x, anchor0_y, control0_x, control0_y, control1_x, control1_y, anchor1_x,
                anchor1_y,
            ],
        }
    }

    fn from_points(anchor0: Point, control0: Point, control1: Point, anchor1: Point) -> Self {
        Self::new(
            anchor0.x, anchor0.y, control0.x, control0.y, control1.x, control1.y, anchor1.x,
            anchor1.y,
        )
    }

    fn straight_line(x0: f32, y0: f32, x1: f32, y1: f32) -> Self {
        Self::new(
            x0,
            y0,
            lerp(x0, x1, 1.0 / 3.0),
            lerp(y0, y1, 1.0 / 3.0),
            lerp(x0, x1, 2.0 / 3.0),
            lerp(y0, y1, 2.0 / 3.0),
            x1,
            y1,
        )
    }

    fn circular_arc(center_x: f32, center_y: f32, x0: f32, y0: f32, x1: f32, y1: f32) -> Self {
        let p0d = direction_vector(x0 - center_x, y0 - center_y);
        let p1d = direction_vector(x1 - center_x, y1 - center_y);
        let rotated_p0 = rotate90(p0d);
        let rotated_p1 = rotate90(p1d);
        let clockwise = point_dot(rotated_p0, Point::new(x1 - center_x, y1 - center_y)) >= 0.0;
        let cosa = point_dot(p0d, p1d);

        if cosa > 0.999 {
            return Self::straight_line(x0, y0, x1, y1);
        }

        let k = distance_components(x0 - center_x, y0 - center_y) * 4.0 / 3.0
            * ((2.0 * (1.0 - cosa)).sqrt() - (1.0 - cosa * cosa).sqrt())
            / (1.0 - cosa)
            * if clockwise { 1.0 } else { -1.0 };

        Self::new(
            x0,
            y0,
            x0 + rotated_p0.x * k,
            y0 + rotated_p0.y * k,
            x1 - rotated_p1.x * k,
            y1 - rotated_p1.y * k,
            x1,
            y1,
        )
    }

    fn anchor0_x(&self) -> f32 {
        self.points[0]
    }

    fn anchor0_y(&self) -> f32 {
        self.points[1]
    }

    fn control0_x(&self) -> f32 {
        self.points[2]
    }

    fn control0_y(&self) -> f32 {
        self.points[3]
    }

    fn control1_x(&self) -> f32 {
        self.points[4]
    }

    fn control1_y(&self) -> f32 {
        self.points[5]
    }

    fn anchor1_x(&self) -> f32 {
        self.points[6]
    }

    fn anchor1_y(&self) -> f32 {
        self.points[7]
    }

    fn point_on_curve(&self, t: f32) -> Point {
        let u = 1.0 - t;

        Point::new(
            self.anchor0_x() * (u * u * u)
                + self.control0_x() * (3.0 * t * u * u)
                + self.control1_x() * (3.0 * t * t * u)
                + self.anchor1_x() * (t * t * t),
            self.anchor0_y() * (u * u * u)
                + self.control0_y() * (3.0 * t * u * u)
                + self.control1_y() * (3.0 * t * t * u)
                + self.anchor1_y() * (t * t * t),
        )
    }

    fn split(&self, t: f32) -> (Self, Self) {
        let u = 1.0 - t;
        let point_on_curve = self.point_on_curve(t);

        (
            Self::new(
                self.anchor0_x(),
                self.anchor0_y(),
                self.anchor0_x() * u + self.control0_x() * t,
                self.anchor0_y() * u + self.control0_y() * t,
                self.anchor0_x() * (u * u)
                    + self.control0_x() * (2.0 * u * t)
                    + self.control1_x() * (t * t),
                self.anchor0_y() * (u * u)
                    + self.control0_y() * (2.0 * u * t)
                    + self.control1_y() * (t * t),
                point_on_curve.x,
                point_on_curve.y,
            ),
            Self::new(
                point_on_curve.x,
                point_on_curve.y,
                self.control0_x() * (u * u)
                    + self.control1_x() * (2.0 * u * t)
                    + self.anchor1_x() * (t * t),
                self.control0_y() * (u * u)
                    + self.control1_y() * (2.0 * u * t)
                    + self.anchor1_y() * (t * t),
                self.control1_x() * u + self.anchor1_x() * t,
                self.control1_y() * u + self.anchor1_y() * t,
                self.anchor1_x(),
                self.anchor1_y(),
            ),
        )
    }

    fn reverse(&self) -> Self {
        Self::new(
            self.anchor1_x(),
            self.anchor1_y(),
            self.control1_x(),
            self.control1_y(),
            self.control0_x(),
            self.control0_y(),
            self.anchor0_x(),
            self.anchor0_y(),
        )
    }

    fn transformed(&self, mut f: impl FnMut(Point) -> Point) -> Self {
        Self::from_points(
            f(Point::new(self.anchor0_x(), self.anchor0_y())),
            f(Point::new(self.control0_x(), self.control0_y())),
            f(Point::new(self.control1_x(), self.control1_y())),
            f(Point::new(self.anchor1_x(), self.anchor1_y())),
        )
    }

    fn zero_length(&self) -> bool {
        (self.anchor0_x() - self.anchor1_x()).abs() < DISTANCE_EPSILON
            && (self.anchor0_y() - self.anchor1_y()).abs() < DISTANCE_EPSILON
    }

    fn calculate_bounds(&self, approximate: bool) -> [f32; 4] {
        if self.zero_length() {
            return [
                self.anchor0_x(),
                self.anchor0_y(),
                self.anchor0_x(),
                self.anchor0_y(),
            ];
        }

        let mut min_x = self.anchor0_x().min(self.anchor1_x());
        let mut min_y = self.anchor0_y().min(self.anchor1_y());
        let mut max_x = self.anchor0_x().max(self.anchor1_x());
        let mut max_y = self.anchor0_y().max(self.anchor1_y());

        if approximate {
            return [
                min_x.min(self.control0_x().min(self.control1_x())),
                min_y.min(self.control0_y().min(self.control1_y())),
                max_x.max(self.control0_x().max(self.control1_x())),
                max_y.max(self.control0_y().max(self.control1_y())),
            ];
        }

        update_cubic_bounds_axis(
            self.anchor0_x(),
            self.control0_x(),
            self.control1_x(),
            self.anchor1_x(),
            |t| self.point_on_curve(t).x,
            &mut min_x,
            &mut max_x,
        );
        update_cubic_bounds_axis(
            self.anchor0_y(),
            self.control0_y(),
            self.control1_y(),
            self.anchor1_y(),
            |t| self.point_on_curve(t).y,
            &mut min_y,
            &mut max_y,
        );

        [min_x, min_y, max_x, max_y]
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Feature {
    Edge(Vec<Cubic>),
    Corner { cubics: Vec<Cubic>, convex: bool },
}

impl Feature {
    fn cubics(&self) -> &[Cubic] {
        match self {
            Self::Edge(cubics) | Self::Corner { cubics, .. } => cubics,
        }
    }

    fn transformed(&self, f: impl Fn(Point) -> Point + Copy) -> Self {
        match self {
            Self::Edge(cubics) => {
                Self::Edge(cubics.iter().map(|cubic| cubic.transformed(f)).collect())
            }
            Self::Corner { cubics, convex } => Self::Corner {
                cubics: cubics.iter().map(|cubic| cubic.transformed(f)).collect(),
                convex: *convex,
            },
        }
    }

    fn is_corner(&self) -> bool {
        matches!(self, Self::Corner { .. })
    }

    fn is_convex_corner(&self) -> bool {
        matches!(self, Self::Corner { convex: true, .. })
    }

    fn is_concave_corner(&self) -> bool {
        matches!(self, Self::Corner { convex: false, .. })
    }
}

#[derive(Debug, Clone, PartialEq)]
struct RoundedPolygon {
    features: Vec<Feature>,
    center: Point,
    cubics: Vec<Cubic>,
}

impl RoundedPolygon {
    fn from_features(features: Vec<Feature>, center: Point) -> Self {
        let cubics = polygon_cubics(&features, center);

        Self {
            features,
            center,
            cubics,
        }
    }

    fn from_vertices(
        vertices: &[Point],
        per_vertex_rounding: &[CornerRounding],
        center: Option<Point>,
    ) -> Self {
        assert!(vertices.len() >= 3);
        assert_eq!(vertices.len(), per_vertex_rounding.len());

        let rounded_corners: Vec<PolygonCorner> = (0..vertices.len())
            .map(|index| {
                PolygonCorner::new(
                    vertices[(index + vertices.len() - 1) % vertices.len()],
                    vertices[index],
                    vertices[(index + 1) % vertices.len()],
                    per_vertex_rounding[index],
                )
            })
            .collect();
        let cut_adjusts: Vec<(f32, f32)> = (0..vertices.len())
            .map(|index| {
                let expected_round_cut = rounded_corners[index].expected_round_cut
                    + rounded_corners[(index + 1) % vertices.len()].expected_round_cut;
                let expected_cut = rounded_corners[index].expected_cut()
                    + rounded_corners[(index + 1) % vertices.len()].expected_cut();
                let side_size = point_distance(point_sub(
                    vertices[index],
                    vertices[(index + 1) % vertices.len()],
                ));

                if expected_round_cut > side_size {
                    (side_size / expected_round_cut, 0.0)
                } else if expected_cut > side_size {
                    (
                        1.0,
                        (side_size - expected_round_cut) / (expected_cut - expected_round_cut),
                    )
                } else {
                    (1.0, 1.0)
                }
            })
            .collect();
        let corners: Vec<Vec<Cubic>> = (0..vertices.len())
            .map(|index| {
                let (round_cut_ratio0, cut_ratio0) =
                    cut_adjusts[(index + vertices.len() - 1) % vertices.len()];
                let (round_cut_ratio1, cut_ratio1) = cut_adjusts[index];
                let allowed_cut0 = rounded_corners[index].expected_round_cut * round_cut_ratio0
                    + (rounded_corners[index].expected_cut()
                        - rounded_corners[index].expected_round_cut)
                        * cut_ratio0;
                let allowed_cut1 = rounded_corners[index].expected_round_cut * round_cut_ratio1
                    + (rounded_corners[index].expected_cut()
                        - rounded_corners[index].expected_round_cut)
                        * cut_ratio1;

                rounded_corners[index].get_cubics(allowed_cut0, allowed_cut1)
            })
            .collect();
        let mut features = Vec::with_capacity(vertices.len() * 2);

        for index in 0..vertices.len() {
            let previous = vertices[(index + vertices.len() - 1) % vertices.len()];
            let current = vertices[index];
            let next = vertices[(index + 1) % vertices.len()];
            let convex = convex(previous, current, next);

            features.push(Feature::Corner {
                cubics: corners[index].clone(),
                convex,
            });
            features.push(Feature::Edge(vec![Cubic::straight_line(
                corners[index].last().unwrap().anchor1_x(),
                corners[index].last().unwrap().anchor1_y(),
                corners[(index + 1) % vertices.len()]
                    .first()
                    .unwrap()
                    .anchor0_x(),
                corners[(index + 1) % vertices.len()]
                    .first()
                    .unwrap()
                    .anchor0_y(),
            )]));
        }

        Self::from_features(
            features,
            center.unwrap_or_else(|| calculate_center(vertices)),
        )
    }

    fn transformed(&self, f: impl Fn(Point) -> Point + Copy) -> Self {
        Self::from_features(
            self.features
                .iter()
                .map(|feature| feature.transformed(f))
                .collect(),
            f(self.center),
        )
    }

    fn normalized(&self) -> Self {
        let bounds = self.calculate_bounds(true);
        let width = bounds_width(bounds);
        let height = bounds_height(bounds);
        let side = width.max(height);
        let offset_x = (side - width) / 2.0 - bounds[0];
        let offset_y = (side - height) / 2.0 - bounds[1];

        self.transformed(|point| {
            Point::new((point.x + offset_x) / side, (point.y + offset_y) / side)
        })
    }

    fn calculate_bounds(&self, approximate: bool) -> [f32; 4] {
        cubics_bounds(&self.cubics, approximate)
    }

    fn calculate_max_bounds(&self) -> [f32; 4] {
        let mut max_dist_squared = 0.0_f32;

        for cubic in &self.cubics {
            let anchor_distance = distance_squared(
                cubic.anchor0_x() - self.center.x,
                cubic.anchor0_y() - self.center.y,
            );
            let middle = cubic.point_on_curve(0.5);
            let middle_distance =
                distance_squared(middle.x - self.center.x, middle.y - self.center.y);

            max_dist_squared = max_dist_squared.max(anchor_distance.max(middle_distance));
        }

        let distance = max_dist_squared.sqrt();

        [
            self.center.x - distance,
            self.center.y - distance,
            self.center.x + distance,
            self.center.y + distance,
        ]
    }
}

fn rounded_polygon_circle(num_vertices: usize, radius: f32, center: Point) -> RoundedPolygon {
    let theta = std::f32::consts::PI / num_vertices as f32;
    let polygon_radius = radius / theta.cos();
    let vertices = vertices_from_num_verts(num_vertices, polygon_radius, center);
    let roundings = vec![CornerRounding::new(radius); num_vertices];

    RoundedPolygon::from_vertices(&vertices, &roundings, Some(center))
}

fn rounded_polygon_star(
    num_vertices_per_radius: usize,
    radius: f32,
    inner_radius: f32,
    rounding: CornerRounding,
    center: Point,
) -> RoundedPolygon {
    assert!(radius > 0.0 && inner_radius > 0.0 && inner_radius < radius);

    let vertices =
        star_vertices_from_num_verts(num_vertices_per_radius, radius, inner_radius, center);
    let roundings = vec![rounding; vertices.len()];

    RoundedPolygon::from_vertices(&vertices, &roundings, Some(center))
}

fn vertices_from_num_verts(num_vertices: usize, radius: f32, center: Point) -> Vec<Point> {
    (0..num_vertices)
        .map(|index| radial_to_cartesian(radius, TAU / num_vertices as f32 * index as f32, center))
        .collect()
}

fn star_vertices_from_num_verts(
    num_vertices_per_radius: usize,
    radius: f32,
    inner_radius: f32,
    center: Point,
) -> Vec<Point> {
    let mut vertices = Vec::with_capacity(num_vertices_per_radius * 2);

    for index in 0..num_vertices_per_radius {
        vertices.push(radial_to_cartesian(
            radius,
            TAU / num_vertices_per_radius as f32 * index as f32,
            center,
        ));
        vertices.push(radial_to_cartesian(
            inner_radius,
            std::f32::consts::PI / num_vertices_per_radius as f32 * (2 * index + 1) as f32,
            center,
        ));
    }

    vertices
}

fn polygon_cubics(features: &[Feature], center: Point) -> Vec<Cubic> {
    let mut cubics = Vec::new();
    let mut first_cubic = None;
    let mut last_cubic: Option<Cubic> = None;
    let mut first_feature_split_start = None;
    let mut first_feature_split_end = None;

    if !features.is_empty() && features[0].cubics().len() == 3 {
        let (start, end) = features[0].cubics()[1].split(0.5);
        first_feature_split_start = Some(vec![features[0].cubics()[0], start]);
        first_feature_split_end = Some(vec![end, features[0].cubics()[2]]);
    }

    for index in 0..=features.len() {
        let feature_cubics: Option<&[Cubic]> = if index == 0 {
            first_feature_split_end
                .as_deref()
                .or(Some(features[0].cubics()))
        } else if index == features.len() {
            first_feature_split_start.as_deref()
        } else {
            Some(features[index].cubics())
        };

        let Some(feature_cubics) = feature_cubics else {
            break;
        };

        for cubic in feature_cubics {
            if !cubic.zero_length() {
                if let Some(last) = last_cubic.take() {
                    cubics.push(last);
                }

                last_cubic = Some(*cubic);
                let _ = first_cubic.get_or_insert(*cubic);
            } else if let Some(last) = last_cubic.as_mut() {
                last.points[6] = cubic.anchor1_x();
                last.points[7] = cubic.anchor1_y();
            }
        }
    }

    if let (Some(last), Some(first)) = (last_cubic, first_cubic) {
        cubics.push(Cubic::new(
            last.anchor0_x(),
            last.anchor0_y(),
            last.control0_x(),
            last.control0_y(),
            last.control1_x(),
            last.control1_y(),
            first.anchor0_x(),
            first.anchor0_y(),
        ));
    } else {
        cubics.push(Cubic::new(
            center.x, center.y, center.x, center.y, center.x, center.y, center.x, center.y,
        ));
    }

    cubics
}

#[derive(Debug, Clone, Copy)]
struct PolygonCorner {
    p0: Point,
    p1: Point,
    p2: Point,
    d1: Point,
    d2: Point,
    corner_radius: f32,
    smoothing: f32,
    expected_round_cut: f32,
}

impl PolygonCorner {
    fn new(p0: Point, p1: Point, p2: Point, rounding: CornerRounding) -> Self {
        let v01 = point_sub(p0, p1);
        let v21 = point_sub(p2, p1);
        let d01 = point_distance(v01);
        let d21 = point_distance(v21);

        if d01 > 0.0 && d21 > 0.0 {
            let d1 = point_scale(v01, 1.0 / d01);
            let d2 = point_scale(v21, 1.0 / d21);
            let cos_angle = point_dot(d1, d2);
            let sin_angle = (1.0 - square(cos_angle)).sqrt();
            let expected_round_cut = if sin_angle > 1e-3 {
                rounding.radius * (cos_angle + 1.0) / sin_angle
            } else {
                0.0
            };

            Self {
                p0,
                p1,
                p2,
                d1,
                d2,
                corner_radius: rounding.radius,
                smoothing: rounding.smoothing,
                expected_round_cut,
            }
        } else {
            Self {
                p0,
                p1,
                p2,
                d1: Point::ORIGIN,
                d2: Point::ORIGIN,
                corner_radius: 0.0,
                smoothing: 0.0,
                expected_round_cut: 0.0,
            }
        }
    }

    fn expected_cut(&self) -> f32 {
        (1.0 + self.smoothing) * self.expected_round_cut
    }

    fn get_cubics(&self, allowed_cut0: f32, allowed_cut1: f32) -> Vec<Cubic> {
        let allowed_cut = allowed_cut0.min(allowed_cut1);

        if self.expected_round_cut < DISTANCE_EPSILON
            || allowed_cut < DISTANCE_EPSILON
            || self.corner_radius < DISTANCE_EPSILON
        {
            return vec![Cubic::straight_line(
                self.p1.x, self.p1.y, self.p1.x, self.p1.y,
            )];
        }

        let actual_round_cut = allowed_cut.min(self.expected_round_cut);
        let actual_smoothing0 = self.calculate_actual_smoothing_value(allowed_cut0);
        let actual_smoothing1 = self.calculate_actual_smoothing_value(allowed_cut1);
        let actual_radius = self.corner_radius * actual_round_cut / self.expected_round_cut;
        let center_distance = (square(actual_radius) + square(actual_round_cut)).sqrt();
        let circle_center = point_add(
            self.p1,
            point_scale(
                point_direction(point_scale(point_add(self.d1, self.d2), 0.5)),
                center_distance,
            ),
        );
        let circle_intersection0 = point_add(self.p1, point_scale(self.d1, actual_round_cut));
        let circle_intersection2 = point_add(self.p1, point_scale(self.d2, actual_round_cut));
        let flanking0 = self.compute_flanking_curve(
            actual_round_cut,
            actual_smoothing0,
            self.p1,
            self.p0,
            circle_intersection0,
            circle_intersection2,
            circle_center,
            actual_radius,
        );
        let flanking2 = self
            .compute_flanking_curve(
                actual_round_cut,
                actual_smoothing1,
                self.p1,
                self.p2,
                circle_intersection2,
                circle_intersection0,
                circle_center,
                actual_radius,
            )
            .reverse();

        vec![
            flanking0,
            Cubic::circular_arc(
                circle_center.x,
                circle_center.y,
                flanking0.anchor1_x(),
                flanking0.anchor1_y(),
                flanking2.anchor0_x(),
                flanking2.anchor0_y(),
            ),
            flanking2,
        ]
    }

    fn calculate_actual_smoothing_value(&self, allowed_cut: f32) -> f32 {
        if allowed_cut > self.expected_cut() {
            self.smoothing
        } else if allowed_cut > self.expected_round_cut {
            self.smoothing * (allowed_cut - self.expected_round_cut)
                / (self.expected_cut() - self.expected_round_cut)
        } else {
            0.0
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn compute_flanking_curve(
        &self,
        actual_round_cut: f32,
        actual_smoothing_value: f32,
        corner: Point,
        side_start: Point,
        circle_segment_intersection: Point,
        other_circle_segment_intersection: Point,
        circle_center: Point,
        actual_radius: f32,
    ) -> Cubic {
        let side_direction = point_direction(point_sub(side_start, corner));
        let curve_start = point_add(
            corner,
            point_scale(
                side_direction,
                actual_round_cut * (1.0 + actual_smoothing_value),
            ),
        );
        let p = point_lerp(
            circle_segment_intersection,
            point_scale(
                point_add(
                    circle_segment_intersection,
                    other_circle_segment_intersection,
                ),
                0.5,
            ),
            actual_smoothing_value,
        );
        let curve_end = point_add(
            circle_center,
            point_scale(
                direction_vector(p.x - circle_center.x, p.y - circle_center.y),
                actual_radius,
            ),
        );
        let circle_tangent = rotate90(point_sub(curve_end, circle_center));
        let anchor_end = line_intersection(side_start, side_direction, curve_end, circle_tangent)
            .unwrap_or(circle_segment_intersection);
        let anchor_start = point_scale(
            point_add(curve_start, point_scale(anchor_end, 2.0)),
            1.0 / 3.0,
        );

        Cubic::from_points(curve_start, anchor_start, anchor_end, curve_end)
    }
}

fn line_intersection(p0: Point, d0: Point, p1: Point, d1: Point) -> Option<Point> {
    let rotated_d1 = rotate90(d1);
    let denominator = point_dot(d0, rotated_d1);

    if denominator.abs() < DISTANCE_EPSILON {
        return None;
    }

    let numerator = point_dot(point_sub(p1, p0), rotated_d1);

    if denominator.abs() < DISTANCE_EPSILON * numerator.abs() {
        return None;
    }

    Some(point_add(p0, point_scale(d0, numerator / denominator)))
}

#[derive(Debug, Clone)]
struct Morph {
    pairs: Vec<(Cubic, Cubic)>,
}

impl Morph {
    fn new(start: RoundedPolygon, end: RoundedPolygon) -> Self {
        Self {
            pairs: match_polygons(&start, &end),
        }
    }

    fn as_cubics(&self, progress: f32) -> Vec<Cubic> {
        let mut cubics = Vec::with_capacity(self.pairs.len());
        let mut first_cubic = None;
        let mut last_cubic = None;

        for (start, end) in &self.pairs {
            let cubic = Cubic {
                points: std::array::from_fn(|index| {
                    lerp(start.points[index], end.points[index], progress)
                }),
            };

            let _ = first_cubic.get_or_insert(cubic);
            if let Some(last) = last_cubic.take() {
                cubics.push(last);
            }
            last_cubic = Some(cubic);
        }

        if let (Some(last), Some(first)) = (last_cubic, first_cubic) {
            cubics.push(Cubic::new(
                last.anchor0_x(),
                last.anchor0_y(),
                last.control0_x(),
                last.control0_y(),
                last.control1_x(),
                last.control1_y(),
                first.anchor0_x(),
                first.anchor0_y(),
            ));
        }

        cubics
    }
}

#[derive(Debug, Clone)]
struct ProgressableFeature {
    progress: f32,
    feature: Feature,
}

#[derive(Debug, Clone)]
struct MeasuredCubic {
    cubic: Cubic,
    start_outline_progress: f32,
    end_outline_progress: f32,
}

impl MeasuredCubic {
    fn cut_at_progress(&self, cut_outline_progress: f32) -> (Self, Self) {
        let bounded_cut_outline_progress =
            cut_outline_progress.clamp(self.start_outline_progress, self.end_outline_progress);
        let outline_progress_size = self.end_outline_progress - self.start_outline_progress;
        let progress_from_start = bounded_cut_outline_progress - self.start_outline_progress;
        let relative_progress = progress_from_start / outline_progress_size;
        let measured_size = measure_cubic(self.cubic);
        let t = find_cubic_cut_point(self.cubic, relative_progress * measured_size);
        let (first, second) = self.cubic.split(t);

        (
            Self {
                cubic: first,
                start_outline_progress: self.start_outline_progress,
                end_outline_progress: bounded_cut_outline_progress,
            },
            Self {
                cubic: second,
                start_outline_progress: bounded_cut_outline_progress,
                end_outline_progress: self.end_outline_progress,
            },
        )
    }
}

#[derive(Debug, Clone)]
struct MeasuredPolygon {
    features: Vec<ProgressableFeature>,
    cubics: Vec<MeasuredCubic>,
}

impl MeasuredPolygon {
    fn new(
        features: Vec<ProgressableFeature>,
        cubics: Vec<Cubic>,
        outline_progress: Vec<f32>,
    ) -> Self {
        assert_eq!(outline_progress.len(), cubics.len() + 1);
        assert!((outline_progress[0] - 0.0).abs() < DISTANCE_EPSILON);
        assert!((outline_progress[outline_progress.len() - 1] - 1.0).abs() < DISTANCE_EPSILON);

        let mut measured_cubics = Vec::new();
        let mut start_outline_progress = 0.0;

        for index in 0..cubics.len() {
            if outline_progress[index + 1] - outline_progress[index] > DISTANCE_EPSILON {
                measured_cubics.push(MeasuredCubic {
                    cubic: cubics[index],
                    start_outline_progress,
                    end_outline_progress: outline_progress[index + 1],
                });
                start_outline_progress = outline_progress[index + 1];
            }
        }

        if let Some(last) = measured_cubics.last_mut() {
            last.end_outline_progress = 1.0;
        }

        Self {
            features,
            cubics: measured_cubics,
        }
    }

    fn measure_polygon(polygon: &RoundedPolygon) -> Self {
        let mut cubics = Vec::new();
        let mut feature_to_cubic = Vec::new();

        for feature in &polygon.features {
            for (cubic_index, cubic) in feature.cubics().iter().enumerate() {
                if feature.is_corner() && cubic_index == feature.cubics().len() / 2 {
                    feature_to_cubic.push((feature.clone(), cubics.len()));
                }
                cubics.push(*cubic);
            }
        }

        let mut measures = Vec::with_capacity(cubics.len() + 1);
        let mut total = 0.0;
        measures.push(total);

        for cubic in &cubics {
            total += measure_cubic(*cubic);
            measures.push(total);
        }

        let outline_progress: Vec<f32> = measures.iter().map(|measure| measure / total).collect();
        let features = feature_to_cubic
            .into_iter()
            .map(|(feature, index)| ProgressableFeature {
                progress: positive_modulo(
                    (outline_progress[index] + outline_progress[index + 1]) / 2.0,
                    1.0,
                ),
                feature,
            })
            .collect();

        Self::new(features, cubics, outline_progress)
    }

    fn cut_and_shift(&self, cutting_point: f32) -> Self {
        assert!((0.0..=1.0).contains(&cutting_point));

        if cutting_point < DISTANCE_EPSILON {
            return self.clone();
        }

        let target_index = self
            .cubics
            .iter()
            .position(|cubic| {
                cutting_point >= cubic.start_outline_progress
                    && cutting_point <= cubic.end_outline_progress
            })
            .unwrap_or(self.cubics.len() - 1);
        let target = &self.cubics[target_index];
        let (first, second) = target.cut_at_progress(cutting_point);
        let mut cubics = Vec::with_capacity(self.cubics.len() + 1);

        cubics.push(second.cubic);
        for index in 1..self.cubics.len() {
            cubics.push(self.cubics[(index + target_index) % self.cubics.len()].cubic);
        }
        cubics.push(first.cubic);

        let mut outline_progress = Vec::with_capacity(self.cubics.len() + 2);

        for index in 0..self.cubics.len() + 2 {
            outline_progress.push(match index {
                0 => 0.0,
                n if n == self.cubics.len() + 1 => 1.0,
                _ => {
                    let cubic_index = (target_index + index - 1) % self.cubics.len();
                    positive_modulo(
                        self.cubics[cubic_index].end_outline_progress - cutting_point,
                        1.0,
                    )
                }
            });
        }

        let features = self
            .features
            .iter()
            .map(|feature| ProgressableFeature {
                progress: positive_modulo(feature.progress - cutting_point, 1.0),
                feature: feature.feature.clone(),
            })
            .collect();

        Self::new(features, cubics, outline_progress)
    }
}

fn match_polygons(start: &RoundedPolygon, end: &RoundedPolygon) -> Vec<(Cubic, Cubic)> {
    let measured_start = MeasuredPolygon::measure_polygon(start);
    let measured_end = MeasuredPolygon::measure_polygon(end);
    let mapper = feature_mapper(&measured_start.features, &measured_end.features);
    let end_cut_point = mapper.map(0.0);
    let shifted_start = measured_start;
    let shifted_end = measured_end.cut_and_shift(end_cut_point);
    let mut pairs = Vec::new();
    let mut start_index = 0;
    let mut end_index = 0;
    let mut start_cubic = shifted_start.cubics.get(start_index).cloned();
    start_index += 1;
    let mut end_cubic = shifted_end.cubics.get(end_index).cloned();
    end_index += 1;

    while let (Some(start), Some(end)) = (start_cubic.clone(), end_cubic.clone()) {
        let start_end_progress = if start_index == shifted_start.cubics.len() {
            1.0
        } else {
            start.end_outline_progress
        };
        let end_end_progress = if end_index == shifted_end.cubics.len() {
            1.0
        } else {
            mapper.map_back(positive_modulo(
                end.end_outline_progress + end_cut_point,
                1.0,
            ))
        };
        let min_progress = start_end_progress.min(end_end_progress);
        let (start_segment, new_start) = if start_end_progress > min_progress + ANGLE_EPSILON {
            let (segment, remainder) = start.cut_at_progress(min_progress);

            (segment, Some(remainder))
        } else {
            let next = shifted_start.cubics.get(start_index).cloned();
            start_index += 1;

            (start, next)
        };
        let (end_segment, new_end) = if end_end_progress > min_progress + ANGLE_EPSILON {
            let (segment, remainder) = end.cut_at_progress(positive_modulo(
                mapper.map(min_progress) - end_cut_point,
                1.0,
            ));

            (segment, Some(remainder))
        } else {
            let next = shifted_end.cubics.get(end_index).cloned();
            end_index += 1;

            (end, next)
        };

        pairs.push((start_segment.cubic, end_segment.cubic));
        start_cubic = new_start;
        end_cubic = new_end;
    }

    assert!(start_cubic.is_none() && end_cubic.is_none());

    pairs
}

#[derive(Debug, Clone)]
struct DoubleMapper {
    source_values: Vec<f32>,
    target_values: Vec<f32>,
}

impl DoubleMapper {
    fn new(mappings: &[(f32, f32)]) -> Self {
        let source_values = mappings.iter().map(|mapping| mapping.0).collect();
        let target_values = mappings.iter().map(|mapping| mapping.1).collect();

        Self {
            source_values,
            target_values,
        }
    }

    fn map(&self, progress: f32) -> f32 {
        linear_map(&self.source_values, &self.target_values, progress)
    }

    fn map_back(&self, progress: f32) -> f32 {
        linear_map(&self.target_values, &self.source_values, progress)
    }
}

fn feature_mapper(
    features1: &[ProgressableFeature],
    features2: &[ProgressableFeature],
) -> DoubleMapper {
    let filtered1: Vec<ProgressableFeature> = features1
        .iter()
        .filter(|feature| feature.feature.is_corner())
        .cloned()
        .collect();
    let filtered2: Vec<ProgressableFeature> = features2
        .iter()
        .filter(|feature| feature.feature.is_corner())
        .cloned()
        .collect();
    let mappings = feature_mapping(&filtered1, &filtered2);

    DoubleMapper::new(&mappings)
}

fn feature_mapping(
    features1: &[ProgressableFeature],
    features2: &[ProgressableFeature],
) -> Vec<(f32, f32)> {
    let mut distances = Vec::new();

    for (index1, feature1) in features1.iter().enumerate() {
        for (index2, feature2) in features2.iter().enumerate() {
            let distance = feature_distance_squared(&feature1.feature, &feature2.feature);

            if distance != f32::MAX {
                distances.push((distance, index1, index2));
            }
        }
    }

    distances.sort_by(|a, b| a.0.total_cmp(&b.0));

    if distances.is_empty() {
        return vec![(0.0, 0.0), (0.5, 0.5)];
    }

    if distances.len() == 1 {
        let (_, index1, index2) = distances[0];
        let f1 = features1[index1].progress;
        let f2 = features2[index2].progress;

        return vec![(f1, f2), ((f1 + 0.5) % 1.0, (f2 + 0.5) % 1.0)];
    }

    let mut helper = MappingHelper::new();

    for (_, index1, index2) in distances {
        helper.add_mapping(features1, features2, index1, index2);
    }

    helper.mapping
}

struct MappingHelper {
    mapping: Vec<(f32, f32)>,
    used1: Vec<usize>,
    used2: Vec<usize>,
}

impl MappingHelper {
    fn new() -> Self {
        Self {
            mapping: Vec::new(),
            used1: Vec::new(),
            used2: Vec::new(),
        }
    }

    fn add_mapping(
        &mut self,
        features1: &[ProgressableFeature],
        features2: &[ProgressableFeature],
        index1: usize,
        index2: usize,
    ) {
        if self.used1.contains(&index1) || self.used2.contains(&index2) {
            return;
        }

        let f1 = features1[index1].progress;
        let f2 = features2[index2].progress;
        let insertion_index = self
            .mapping
            .iter()
            .position(|mapping| mapping.0 > f1)
            .unwrap_or(self.mapping.len());
        let len = self.mapping.len();

        if len >= 1 {
            let before = self.mapping[(insertion_index + len - 1) % len];
            let after = self.mapping[insertion_index % len];

            if progress_distance(f1, before.0) < DISTANCE_EPSILON
                || progress_distance(f1, after.0) < DISTANCE_EPSILON
                || progress_distance(f2, before.1) < DISTANCE_EPSILON
                || progress_distance(f2, after.1) < DISTANCE_EPSILON
            {
                return;
            }

            if len > 1 && !progress_in_range(f2, before.1, after.1) {
                return;
            }
        }

        self.mapping.insert(insertion_index, (f1, f2));
        self.used1.push(index1);
        self.used2.push(index2);
    }
}

fn feature_distance_squared(first: &Feature, second: &Feature) -> f32 {
    if (first.is_convex_corner() && second.is_concave_corner())
        || (first.is_concave_corner() && second.is_convex_corner())
    {
        return f32::MAX;
    }

    distance_squared_point(point_sub(
        feature_representative_point(first),
        feature_representative_point(second),
    ))
}

fn feature_representative_point(feature: &Feature) -> Point {
    Point::new(
        (feature.cubics().first().unwrap().anchor0_x()
            + feature.cubics().last().unwrap().anchor1_x())
            / 2.0,
        (feature.cubics().first().unwrap().anchor0_y()
            + feature.cubics().last().unwrap().anchor1_y())
            / 2.0,
    )
}

fn linear_map(x_values: &[f32], y_values: &[f32], progress: f32) -> f32 {
    let progress = if progress >= 1.0 {
        0.0
    } else {
        positive_modulo(progress, 1.0)
    };
    let segment_start_index = (0..x_values.len())
        .find(|index| {
            progress_in_range(
                progress,
                x_values[*index],
                x_values[(*index + 1) % x_values.len()],
            )
        })
        .unwrap_or(0);
    let segment_end_index = (segment_start_index + 1) % x_values.len();
    let segment_size_x = positive_modulo(
        x_values[segment_end_index] - x_values[segment_start_index],
        1.0,
    );
    let segment_size_y = positive_modulo(
        y_values[segment_end_index] - y_values[segment_start_index],
        1.0,
    );
    let position = if segment_size_x < 0.001 {
        0.5
    } else {
        positive_modulo(progress - x_values[segment_start_index], 1.0) / segment_size_x
    };

    positive_modulo(
        y_values[segment_start_index] + segment_size_y * position,
        1.0,
    )
}

fn progress_in_range(progress: f32, from: f32, to: f32) -> bool {
    if to >= from {
        (from..=to).contains(&progress)
    } else {
        progress >= from || progress <= to
    }
}

fn progress_distance(first: f32, second: f32) -> f32 {
    let distance = (first - second).abs();

    distance.min(1.0 - distance)
}

fn measure_cubic(cubic: Cubic) -> f32 {
    closest_progress_to(cubic, f32::INFINITY).1
}

fn find_cubic_cut_point(cubic: Cubic, measure: f32) -> f32 {
    closest_progress_to(cubic, measure).0
}

fn closest_progress_to(cubic: Cubic, threshold: f32) -> (f32, f32) {
    const SEGMENTS: usize = 3;
    let mut total = 0.0;
    let mut remainder = threshold;
    let mut previous = Point::new(cubic.anchor0_x(), cubic.anchor0_y());

    for index in 1..=SEGMENTS {
        let progress = index as f32 / SEGMENTS as f32;
        let point = cubic.point_on_curve(progress);
        let segment = point_distance(point_sub(point, previous));

        if segment >= remainder {
            return (
                progress - (1.0 - remainder / segment) / SEGMENTS as f32,
                threshold,
            );
        }

        remainder -= segment;
        total += segment;
        previous = point;
    }

    (1.0, total)
}

fn update_cubic_bounds_axis(
    anchor0: f32,
    control0: f32,
    control1: f32,
    anchor1: f32,
    point: impl Fn(f32) -> f32,
    min_value: &mut f32,
    max_value: &mut f32,
) {
    let a = -anchor0 + 3.0 * control0 - 3.0 * control1 + anchor1;
    let b = 2.0 * anchor0 - 4.0 * control0 + 2.0 * control1;
    let c = -anchor0 + control0;

    if a.abs() < DISTANCE_EPSILON {
        if b != 0.0 {
            let t = 2.0 * c / (-2.0 * b);
            update_bounds_with_curve_point(t, &point, min_value, max_value);
        }
    } else {
        let discriminant = b * b - 4.0 * a * c;

        if discriminant >= 0.0 {
            update_bounds_with_curve_point(
                (-b + discriminant.sqrt()) / (2.0 * a),
                &point,
                min_value,
                max_value,
            );
            update_bounds_with_curve_point(
                (-b - discriminant.sqrt()) / (2.0 * a),
                &point,
                min_value,
                max_value,
            );
        }
    }
}

fn update_bounds_with_curve_point(
    t: f32,
    point: &impl Fn(f32) -> f32,
    min_value: &mut f32,
    max_value: &mut f32,
) {
    if (0.0..=1.0).contains(&t) {
        let value = point(t);
        *min_value = min_value.min(value);
        *max_value = max_value.max(value);
    }
}

fn cubics_bounds(cubics: &[Cubic], approximate: bool) -> [f32; 4] {
    let mut min_x = f32::INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut max_y = f32::NEG_INFINITY;

    for cubic in cubics {
        let bounds = cubic.calculate_bounds(approximate);

        min_x = min_x.min(bounds[0]);
        min_y = min_y.min(bounds[1]);
        max_x = max_x.max(bounds[2]);
        max_y = max_y.max(bounds[3]);
    }

    [min_x, min_y, max_x, max_y]
}

fn bounds_width(bounds: [f32; 4]) -> f32 {
    bounds[2] - bounds[0]
}

fn bounds_height(bounds: [f32; 4]) -> f32 {
    bounds[3] - bounds[1]
}

fn bounds_center(bounds: [f32; 4]) -> Point {
    Point::new((bounds[0] + bounds[2]) / 2.0, (bounds[1] + bounds[3]) / 2.0)
}

fn calculate_center(vertices: &[Point]) -> Point {
    let sum = vertices
        .iter()
        .fold(Point::ORIGIN, |sum, point| point_add(sum, *point));

    point_scale(sum, 1.0 / vertices.len() as f32)
}

fn radial_to_cartesian(radius: f32, angle: f32, center: Point) -> Point {
    point_add(
        point_scale(direction_vector_from_angle(angle), radius),
        center,
    )
}

fn convex(previous: Point, current: Point, next: Point) -> bool {
    point_clockwise(point_sub(current, previous), point_sub(next, current))
}

fn rotate_point(point: Point, rotation: f32) -> Point {
    let cos = rotation.cos();
    let sin = rotation.sin();

    Point::new(point.x * cos - point.y * sin, point.x * sin + point.y * cos)
}

fn rotate_point_around(point: Point, center: Point, rotation: f32) -> Point {
    point_add(rotate_point(point_sub(point, center), rotation), center)
}

fn point_add(first: Point, second: Point) -> Point {
    Point::new(first.x + second.x, first.y + second.y)
}

fn point_sub(first: Point, second: Point) -> Point {
    Point::new(first.x - second.x, first.y - second.y)
}

fn point_scale(point: Point, scale: f32) -> Point {
    Point::new(point.x * scale, point.y * scale)
}

fn point_lerp(first: Point, second: Point, progress: f32) -> Point {
    Point::new(
        lerp(first.x, second.x, progress),
        lerp(first.y, second.y, progress),
    )
}

fn point_distance(point: Point) -> f32 {
    distance_components(point.x, point.y)
}

fn point_direction(point: Point) -> Point {
    let distance = point_distance(point);

    assert!(distance > 0.0);
    point_scale(point, 1.0 / distance)
}

fn point_dot(first: Point, second: Point) -> f32 {
    first.x * second.x + first.y * second.y
}

fn point_clockwise(first: Point, second: Point) -> bool {
    first.x * second.y - first.y * second.x > 0.0
}

fn rotate90(point: Point) -> Point {
    Point::new(-point.y, point.x)
}

fn direction_vector(x: f32, y: f32) -> Point {
    let distance = distance_components(x, y);

    assert!(distance > 0.0);
    Point::new(x / distance, y / distance)
}

fn direction_vector_from_angle(angle: f32) -> Point {
    Point::new(angle.cos(), angle.sin())
}

fn distance_components(x: f32, y: f32) -> f32 {
    (x * x + y * y).sqrt()
}

fn distance_squared(x: f32, y: f32) -> f32 {
    x * x + y * y
}

fn distance_squared_point(point: Point) -> f32 {
    distance_squared(point.x, point.y)
}

fn square(value: f32) -> f32 {
    value * value
}

fn lerp(start: f32, end: f32, progress: f32) -> f32 {
    (1.0 - progress) * start + progress * end
}

fn positive_modulo(value: f32, modulus: f32) -> f32 {
    (value % modulus + modulus) % modulus
}

const DISTANCE_EPSILON: f32 = 1e-4;
const ANGLE_EPSILON: f32 = 1e-6;

#[cfg(test)]
#[path = "../../../tests/widget/component/progress_bar.rs"]
mod tests;
