//! Floating Material color theme picker.

use iced_widget::button::{Status, Style};
use iced_widget::canvas::{self, Canvas, Path};
use iced_widget::core::text as core_text;
use iced_widget::core::time::Instant;
use iced_widget::core::{
    Background, Color, Element, Length, Padding, Point, Rectangle, Size, alignment, border, mouse,
};
use iced_widget::graphics::geometry;
use iced_widget::{Column, Container, Row, Space, Stack};

use super::{absolute_line_height, button::Button};
use crate::animation::{ThemeRevealTransition, max_radius_from_origin};
use crate::utils::{HOVERED_LAYER_OPACITY, PRESSED_LAYER_OPACITY, mix, shadow_from_level};
use crate::{ColorQuartet, ColorScheme, Surface, SurfaceContainer, Theme, fonts, tokens};

pub const FLOATING_MARGIN: f32 = 24.0;

const PICKER_PANEL_PADDING: f32 = 12.0;
const PICKER_PANEL_SPACING: f32 = 8.0;
const PICKER_PANEL_SHAPE: f32 = tokens::shape::CORNER_EXTRA_LARGE;
const PICKER_PANEL_ELEVATION_LEVEL: u8 = 3;
const SWATCH_SIZE: f32 = 40.0;
const SWATCH_TARGET_SIZE: f32 = 48.0;
const SWATCH_SHAPE: f32 = tokens::shape::CORNER_FULL;
const SELECTED_SWATCH_OUTLINE_WIDTH: f32 = 3.0;
const SWATCH_OUTLINE_WIDTH: f32 = 1.0;
const SWATCH_COLUMNS: usize = 4;
const SWATCH_ROWS: usize = 2;
const PALETTE_BUTTON_SIZE: f32 = 56.0;
const PALETTE_BUTTON_SHAPE: f32 = tokens::shape::CORNER_LARGE;
const PALETTE_BUTTON_ELEVATION_LEVEL: u8 = 3;
const THEME_REVEAL_CORE_RADIUS_FACTOR: f32 = 0.36;
const THEME_REVEAL_MAX_ALPHA: f32 = 0.42;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct State {
    is_open: bool,
}

impl State {
    pub const fn new() -> Self {
        Self { is_open: false }
    }

    pub const fn is_open(self) -> bool {
        self.is_open
    }

    pub fn toggle(&mut self) {
        self.is_open = !self.is_open;
    }

    pub fn open(&mut self) {
        self.is_open = true;
    }

    pub fn close(&mut self) {
        self.is_open = false;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaterialColor {
    Purple,
    Blue,
    Teal,
    Green,
    Yellow,
    Orange,
    Red,
    Pink,
}

impl MaterialColor {
    pub const ALL: [Self; 8] = [
        Self::Purple,
        Self::Blue,
        Self::Teal,
        Self::Green,
        Self::Yellow,
        Self::Orange,
        Self::Red,
        Self::Pink,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Purple => "Purple",
            Self::Blue => "Blue",
            Self::Teal => "Teal",
            Self::Green => "Green",
            Self::Yellow => "Yellow",
            Self::Orange => "Orange",
            Self::Red => "Red",
            Self::Pink => "Pink",
        }
    }

    pub fn color_scheme(self, dark: bool) -> ColorScheme {
        let mut scheme = if dark {
            Theme::Dark.colors()
        } else {
            Theme::Light.colors()
        };

        let primary = self.primary(dark);

        scheme.primary = primary;
        scheme.secondary = tint_quartet(scheme.secondary, primary, 0.55);
        scheme.tertiary = tint_quartet(scheme.tertiary, primary, 0.32);
        scheme.surface = tint_surface(scheme.surface, primary, dark);
        scheme.inverse.inverse_primary = self.primary(!dark).color;
        scheme.inverse.inverse_surface = mix(
            scheme.inverse.inverse_surface,
            self.primary(!dark).container,
            0.08,
        );
        scheme.outline.color = mix(scheme.outline.color, primary.color, 0.08);
        scheme.outline.variant = mix(scheme.outline.variant, primary.container, 0.10);
        scheme
    }

    pub const fn swatch(self) -> Color {
        self.primary(false).color
    }

    const fn index(self) -> usize {
        match self {
            Self::Purple => 0,
            Self::Blue => 1,
            Self::Teal => 2,
            Self::Green => 3,
            Self::Yellow => 4,
            Self::Orange => 5,
            Self::Red => 6,
            Self::Pink => 7,
        }
    }

    const fn primary(self, dark: bool) -> ColorQuartet {
        match (self, dark) {
            (Self::Purple, false) => ColorQuartet {
                color: rgb(0x67, 0x50, 0xa4),
                text: rgb(0xff, 0xff, 0xff),
                container: rgb(0xea, 0xdd, 0xff),
                container_text: rgb(0x21, 0x00, 0x5d),
            },
            (Self::Purple, true) => ColorQuartet {
                color: rgb(0xd0, 0xbc, 0xff),
                text: rgb(0x38, 0x1e, 0x72),
                container: rgb(0x4f, 0x37, 0x8b),
                container_text: rgb(0xea, 0xdd, 0xff),
            },
            (Self::Blue, false) => ColorQuartet {
                color: rgb(0x00, 0x61, 0xa4),
                text: rgb(0xff, 0xff, 0xff),
                container: rgb(0xd1, 0xe4, 0xff),
                container_text: rgb(0x00, 0x1d, 0x36),
            },
            (Self::Blue, true) => ColorQuartet {
                color: rgb(0x9e, 0xca, 0xff),
                text: rgb(0x00, 0x32, 0x58),
                container: rgb(0x00, 0x49, 0x7d),
                container_text: rgb(0xd1, 0xe4, 0xff),
            },
            (Self::Teal, false) => ColorQuartet {
                color: rgb(0x00, 0x6a, 0x60),
                text: rgb(0xff, 0xff, 0xff),
                container: rgb(0x74, 0xf8, 0xe6),
                container_text: rgb(0x00, 0x20, 0x1c),
            },
            (Self::Teal, true) => ColorQuartet {
                color: rgb(0x53, 0xdb, 0xc9),
                text: rgb(0x00, 0x37, 0x31),
                container: rgb(0x00, 0x50, 0x48),
                container_text: rgb(0x74, 0xf8, 0xe6),
            },
            (Self::Green, false) => ColorQuartet {
                color: rgb(0x00, 0x6d, 0x3b),
                text: rgb(0xff, 0xff, 0xff),
                container: rgb(0x8f, 0xf7, 0xb3),
                container_text: rgb(0x00, 0x21, 0x0d),
            },
            (Self::Green, true) => ColorQuartet {
                color: rgb(0x73, 0xdb, 0x99),
                text: rgb(0x00, 0x39, 0x1c),
                container: rgb(0x00, 0x52, 0x2b),
                container_text: rgb(0x8f, 0xf7, 0xb3),
            },
            (Self::Yellow, false) => ColorQuartet {
                color: rgb(0x6d, 0x5e, 0x00),
                text: rgb(0xff, 0xff, 0xff),
                container: rgb(0xfb, 0xe5, 0x60),
                container_text: rgb(0x21, 0x1c, 0x00),
            },
            (Self::Yellow, true) => ColorQuartet {
                color: rgb(0xde, 0xc8, 0x48),
                text: rgb(0x39, 0x31, 0x00),
                container: rgb(0x52, 0x46, 0x00),
                container_text: rgb(0xfb, 0xe5, 0x60),
            },
            (Self::Orange, false) => ColorQuartet {
                color: rgb(0x8b, 0x50, 0x00),
                text: rgb(0xff, 0xff, 0xff),
                container: rgb(0xff, 0xdc, 0xbe),
                container_text: rgb(0x2d, 0x16, 0x00),
            },
            (Self::Orange, true) => ColorQuartet {
                color: rgb(0xff, 0xb8, 0x70),
                text: rgb(0x4a, 0x28, 0x00),
                container: rgb(0x69, 0x3c, 0x00),
                container_text: rgb(0xff, 0xdc, 0xbe),
            },
            (Self::Red, false) => ColorQuartet {
                color: rgb(0xba, 0x1a, 0x1a),
                text: rgb(0xff, 0xff, 0xff),
                container: rgb(0xff, 0xda, 0xd6),
                container_text: rgb(0x41, 0x00, 0x02),
            },
            (Self::Red, true) => ColorQuartet {
                color: rgb(0xff, 0xb4, 0xab),
                text: rgb(0x69, 0x00, 0x05),
                container: rgb(0x93, 0x00, 0x0a),
                container_text: rgb(0xff, 0xda, 0xd6),
            },
            (Self::Pink, false) => ColorQuartet {
                color: rgb(0x98, 0x40, 0x61),
                text: rgb(0xff, 0xff, 0xff),
                container: rgb(0xff, 0xd9, 0xe3),
                container_text: rgb(0x3e, 0x00, 0x1d),
            },
            (Self::Pink, true) => ColorQuartet {
                color: rgb(0xff, 0xb1, 0xc8),
                text: rgb(0x5e, 0x11, 0x32),
                container: rgb(0x7b, 0x29, 0x49),
                container_text: rgb(0xff, 0xd9, 0xe3),
            },
        }
    }
}

pub fn palette_center(viewport: Size, bottom_margin: f32) -> Point {
    let right = viewport.width - FLOATING_MARGIN;
    let bottom = viewport.height - bottom_margin;

    Point::new(
        right - PALETTE_BUTTON_SIZE / 2.0,
        bottom - PALETTE_BUTTON_SIZE / 2.0,
    )
}

pub fn swatch_center(viewport: Size, bottom_margin: f32, color: MaterialColor) -> Point {
    let index = color.index();
    let column = index % SWATCH_COLUMNS;
    let row = index / SWATCH_COLUMNS;
    let panel_right = viewport.width - FLOATING_MARGIN;
    let panel_bottom = viewport.height - bottom_margin - PALETTE_BUTTON_SIZE - PICKER_PANEL_SPACING;
    let panel_left = panel_right - picker_panel_width();
    let panel_top = panel_bottom - picker_panel_height();

    Point::new(
        panel_left
            + PICKER_PANEL_PADDING
            + column as f32 * (SWATCH_TARGET_SIZE + PICKER_PANEL_SPACING)
            + SWATCH_TARGET_SIZE / 2.0,
        panel_top
            + PICKER_PANEL_PADDING
            + row as f32 * (SWATCH_TARGET_SIZE + PICKER_PANEL_SPACING)
            + SWATCH_TARGET_SIZE / 2.0,
    )
}

pub fn floating_over<'a, Message, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
    state: &State,
    selected: MaterialColor,
    bottom_margin: f32,
    on_toggle: Message,
    on_select: impl Fn(MaterialColor) -> Message + 'a,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: iced_widget::core::Renderer + core_text::Renderer + geometry::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    Stack::with_children([
        content.into(),
        floating_layer(state, selected, bottom_margin, on_toggle, on_select),
    ])
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

pub fn reveal_over<'a, Message, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
    transition: Option<ThemeRevealTransition>,
    now: Instant,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + geometry::Renderer + 'a,
{
    let content = content.into();

    if let Some(transition) = transition {
        Stack::with_children([content, reveal_overlay(transition, now).into()])
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    } else {
        content
    }
}

pub fn reveal_overlay<'a, Message, Renderer>(
    transition: ThemeRevealTransition,
    now: Instant,
) -> Canvas<ThemeRevealOverlay, Message, Theme, Renderer>
where
    Renderer: geometry::Renderer + 'a,
{
    Canvas::new(ThemeRevealOverlay {
        origin: transition.origin(),
        target: transition.target(),
        progress: transition.eased_progress_at(now),
    })
    .width(Length::Fill)
    .height(Length::Fill)
}

pub fn floating_layer<'a, Message, Renderer>(
    state: &State,
    selected: MaterialColor,
    bottom_margin: f32,
    on_toggle: Message,
    on_select: impl Fn(MaterialColor) -> Message + 'a,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: iced_widget::core::Renderer + core_text::Renderer + geometry::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    Container::new(floating_picker(state, selected, on_toggle, on_select))
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(Padding {
            top: 0.0,
            right: FLOATING_MARGIN,
            bottom: bottom_margin,
            left: 0.0,
        })
        .align_x(alignment::Horizontal::Right)
        .align_y(alignment::Vertical::Bottom)
        .into()
}

fn floating_picker<'a, Message, Renderer>(
    state: &State,
    selected: MaterialColor,
    on_toggle: Message,
    on_select: impl Fn(MaterialColor) -> Message + 'a,
) -> Column<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: iced_widget::core::Renderer + core_text::Renderer + geometry::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    let mut content = Column::new()
        .spacing(PICKER_PANEL_SPACING)
        .align_x(alignment::Horizontal::Right);

    if state.is_open() {
        content = content.push(picker_panel(selected, on_select));
    }

    content.push(palette_button(state.is_open(), on_toggle))
}

fn picker_panel<'a, Message, Renderer>(
    selected: MaterialColor,
    on_select: impl Fn(MaterialColor) -> Message + 'a,
) -> Container<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: iced_widget::core::Renderer + geometry::Renderer + 'a,
{
    let mut rows = Column::new().spacing(PICKER_PANEL_SPACING);

    for colors in MaterialColor::ALL.chunks(SWATCH_COLUMNS) {
        let mut row = Row::new().spacing(PICKER_PANEL_SPACING);

        for color in colors {
            row = row.push(swatch_button(*color, *color == selected, on_select(*color)));
        }

        rows = rows.push(row);
    }

    Container::new(rows)
        .padding(PICKER_PANEL_PADDING)
        .style(picker_panel_style)
}

fn palette_button<'a, Message, Renderer>(
    is_open: bool,
    on_press: Message,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: iced_widget::core::Renderer + core_text::Renderer + geometry::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    Button::new(
        Container::new(
            fonts::icon("palette", tokens::component::toolbar::ACTION_ICON_SIZE).line_height(
                absolute_line_height(tokens::component::toolbar::ACTION_ICON_SIZE),
            ),
        )
        .center_x(Length::Fixed(PALETTE_BUTTON_SIZE))
        .center_y(Length::Fixed(PALETTE_BUTTON_SIZE)),
    )
    .width(Length::Fixed(PALETTE_BUTTON_SIZE))
    .height(Length::Fixed(PALETTE_BUTTON_SIZE))
    .padding(Padding::ZERO)
    .on_press(on_press)
    .style(move |theme, status| palette_button_style(theme, status, is_open))
}

fn swatch_button<'a, Message, Renderer>(
    color: MaterialColor,
    selected: bool,
    on_press: Message,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: iced_widget::core::Renderer + geometry::Renderer + 'a,
{
    Button::new(
        Container::new(Space::new())
            .width(Length::Fixed(SWATCH_SIZE))
            .height(Length::Fixed(SWATCH_SIZE)),
    )
    .width(Length::Fixed(SWATCH_TARGET_SIZE))
    .height(Length::Fixed(SWATCH_TARGET_SIZE))
    .padding(Padding::from([
        (SWATCH_TARGET_SIZE - SWATCH_SIZE) / 2.0,
        (SWATCH_TARGET_SIZE - SWATCH_SIZE) / 2.0,
    ]))
    .on_press(on_press)
    .style(move |theme, status| swatch_style(theme, status, color, selected))
}

fn picker_panel_style(theme: &Theme) -> iced_widget::container::Style {
    let colors = theme.colors();

    iced_widget::container::Style {
        background: Some(Background::Color(colors.surface.container.high)),
        text_color: Some(colors.surface.text),
        border: border::rounded(PICKER_PANEL_SHAPE),
        shadow: shadow_from_level(PICKER_PANEL_ELEVATION_LEVEL, colors.shadow),
        snap: cfg!(feature = "crisp"),
    }
}

fn palette_button_style(theme: &Theme, status: Status, is_open: bool) -> Style {
    let colors = theme.colors();
    let container = if is_open {
        colors.primary.container
    } else {
        colors.surface.container.high
    };
    let foreground = if is_open {
        colors.primary.container_text
    } else {
        colors.primary.color
    };
    let background = match status {
        Status::Active | Status::Disabled => container,
        Status::Hovered => mix(container, foreground, HOVERED_LAYER_OPACITY),
        Status::Pressed => mix(container, foreground, PRESSED_LAYER_OPACITY),
    };

    Style {
        background: Some(Background::Color(background)),
        text_color: foreground,
        border: border::rounded(PALETTE_BUTTON_SHAPE),
        shadow: shadow_from_level(PALETTE_BUTTON_ELEVATION_LEVEL, colors.shadow),
        snap: cfg!(feature = "crisp"),
    }
}

fn swatch_style(theme: &Theme, status: Status, color: MaterialColor, selected: bool) -> Style {
    let colors = theme.colors();
    let base = color.swatch();
    let background = match status {
        Status::Active | Status::Disabled => base,
        Status::Hovered => mix(base, colors.surface.text, HOVERED_LAYER_OPACITY),
        Status::Pressed => mix(base, colors.surface.text, PRESSED_LAYER_OPACITY),
    };

    let outline = if selected {
        colors.surface.text
    } else {
        colors.outline.variant
    };

    Style {
        background: Some(Background::Color(background)),
        text_color: colors.surface.text,
        border: iced_widget::core::Border {
            color: outline,
            width: if selected {
                SELECTED_SWATCH_OUTLINE_WIDTH
            } else {
                SWATCH_OUTLINE_WIDTH
            },
            radius: SWATCH_SHAPE.into(),
        },
        shadow: shadow_from_level(0, Color::TRANSPARENT),
        snap: cfg!(feature = "crisp"),
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ThemeRevealOverlay {
    origin: Point,
    target: ColorScheme,
    progress: f32,
}

impl<Message, Renderer> canvas::Program<Message, Theme, Renderer> for ThemeRevealOverlay
where
    Renderer: geometry::Renderer,
{
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry<Renderer>> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let progress = self.progress.clamp(0.0, 1.0);
        let origin = Point::new(self.origin.x - bounds.x, self.origin.y - bounds.y);
        let radius = max_radius_from_origin(origin, bounds.size()) * progress;

        if radius <= 0.0 {
            return vec![frame.into_geometry()];
        }

        let alpha = reveal_overlay_alpha(progress);
        let mut surface = self.target.surface.color;
        surface.a *= alpha;
        let mut core = mix(
            self.target.primary.container,
            self.target.surface.color,
            0.20,
        );
        core.a *= alpha * 0.75;

        frame.fill(&Path::circle(origin, radius), surface);
        frame.fill(
            &Path::circle(origin, radius * THEME_REVEAL_CORE_RADIUS_FACTOR),
            core,
        );

        vec![frame.into_geometry()]
    }
}

fn reveal_overlay_alpha(progress: f32) -> f32 {
    THEME_REVEAL_MAX_ALPHA * (1.0 - progress.clamp(0.0, 1.0)).sqrt()
}

fn picker_panel_width() -> f32 {
    PICKER_PANEL_PADDING * 2.0
        + SWATCH_COLUMNS as f32 * SWATCH_TARGET_SIZE
        + (SWATCH_COLUMNS - 1) as f32 * PICKER_PANEL_SPACING
}

fn picker_panel_height() -> f32 {
    PICKER_PANEL_PADDING * 2.0
        + SWATCH_ROWS as f32 * SWATCH_TARGET_SIZE
        + (SWATCH_ROWS - 1) as f32 * PICKER_PANEL_SPACING
}

fn tint_quartet(base: ColorQuartet, primary: ColorQuartet, amount: f32) -> ColorQuartet {
    ColorQuartet {
        color: mix(base.color, primary.color, amount),
        text: base.text,
        container: mix(base.container, primary.container, amount),
        container_text: base.container_text,
    }
}

fn tint_surface(base: Surface, primary: ColorQuartet, dark: bool) -> Surface {
    let anchor = primary.container;
    let [surface, lowest, low, container, high, highest] = if dark {
        [0.08, 0.05, 0.09, 0.12, 0.15, 0.18]
    } else {
        [0.20, 0.10, 0.18, 0.24, 0.30, 0.36]
    };

    Surface {
        color: mix(base.color, anchor, surface),
        text: base.text,
        text_variant: base.text_variant,
        container: SurfaceContainer {
            lowest: mix(base.container.lowest, anchor, lowest),
            low: mix(base.container.low, anchor, low),
            base: mix(base.container.base, anchor, container),
            high: mix(base.container.high, anchor, high),
            highest: mix(base.container.highest, anchor, highest),
        },
    }
}

const fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::from_rgb8(r, g, b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_toggles_open_closed() {
        let mut state = State::new();

        assert!(!state.is_open());

        state.toggle();
        assert!(state.is_open());

        state.close();
        assert!(!state.is_open());
    }

    #[test]
    fn material_colors_generate_distinct_primary_roles() {
        assert_eq!(MaterialColor::ALL.len(), 8);
        assert_eq!(
            MaterialColor::Purple.color_scheme(false).primary,
            Theme::Light.colors().primary
        );
        assert_ne!(
            MaterialColor::Blue.color_scheme(false).primary,
            Theme::Light.colors().primary
        );
        assert_ne!(
            MaterialColor::Green.color_scheme(true).primary,
            Theme::Dark.colors().primary
        );
        assert_ne!(
            MaterialColor::Blue
                .color_scheme(false)
                .surface
                .container
                .base,
            Theme::Light.colors().surface.container.base
        );
        assert_ne!(
            MaterialColor::Blue.color_scheme(false).secondary.container,
            Theme::Light.colors().secondary.container
        );
    }

    #[test]
    fn material_colors_tint_menu_backgrounds() {
        let baseline = crate::menu::default(&Theme::Light);
        let blue = Theme::new("Blue", MaterialColor::Blue.color_scheme(false));
        let tinted = crate::menu::default(&blue);

        assert_ne!(tinted.background, baseline.background);
        assert_ne!(tinted.selected_background, baseline.selected_background);
    }

    #[test]
    fn selected_swatch_uses_stronger_outline() {
        let theme = Theme::Light;
        let selected = swatch_style(&theme, Status::Active, MaterialColor::Blue, true);
        let unselected = swatch_style(&theme, Status::Active, MaterialColor::Blue, false);

        assert_eq!(selected.border.width, SELECTED_SWATCH_OUTLINE_WIDTH);
        assert_eq!(unselected.border.width, SWATCH_OUTLINE_WIDTH);
    }

    #[test]
    fn palette_button_uses_rounded_square_without_circular_toolbar_base() {
        let style = palette_button_style(&Theme::Dark, Status::Active, false);

        assert_eq!(style.border.radius.top_left, PALETTE_BUTTON_SHAPE);
        assert_ne!(style.border.radius.top_left, tokens::shape::CORNER_FULL);
    }

    #[test]
    fn swatch_centers_track_picker_layout() {
        let viewport = Size::new(1080.0, 980.0);
        let bottom_margin = FLOATING_MARGIN;
        let purple = swatch_center(viewport, bottom_margin, MaterialColor::Purple);
        let blue = swatch_center(viewport, bottom_margin, MaterialColor::Blue);
        let yellow = swatch_center(viewport, bottom_margin, MaterialColor::Yellow);

        assert_eq!(blue.x - purple.x, SWATCH_TARGET_SIZE + PICKER_PANEL_SPACING);
        assert_eq!(
            yellow.y - purple.y,
            SWATCH_TARGET_SIZE + PICKER_PANEL_SPACING
        );
        assert!(purple.x < palette_center(viewport, bottom_margin).x);
    }

    #[test]
    fn reveal_overlay_fades_as_reveal_completes() {
        assert_eq!(reveal_overlay_alpha(1.0), 0.0);
        assert!(reveal_overlay_alpha(0.25) > reveal_overlay_alpha(0.75));
    }
}
