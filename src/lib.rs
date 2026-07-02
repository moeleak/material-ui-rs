use std::borrow::Cow;

use iced_widget::core::{
    Color, color,
    theme::{Base, Mode, Style},
};
use utils::{lightness, mix};

pub mod animation;
pub mod application;
pub mod badge;
pub mod button;
pub mod checkbox;
pub mod combo_box;
pub mod container;
#[cfg(feature = "dialog")]
pub mod dialog;
pub mod fonts;
pub mod list;
#[cfg(feature = "markdown")]
pub mod markdown;
pub mod menu;
pub mod pane_grid;
pub mod pick_list;
pub mod progress_bar;
#[cfg(feature = "qr_code")]
pub mod qr_code;
pub mod radio;
pub mod rule;
pub mod scrollable;
#[cfg(feature = "selection")]
pub mod selection;
pub mod slider;
#[cfg(feature = "svg")]
pub mod svg;
pub mod table;
pub mod text;
pub mod text_editor;
pub mod text_input;
pub mod toggler;
pub mod tokens;
pub mod tooltip;
pub mod utils;
pub mod widget;

pub use application::{
    application, window, window_settings, window_with_min_size, with_material_fonts,
};

/// An [`iced::Element`] that uses the bundled [`Theme`] by default.
pub type Element<'a, Message, T = crate::Theme, Renderer = iced::Renderer> =
    iced::Element<'a, Message, T, Renderer>;

/// An [`iced::widget::Container`] that uses the bundled [`Theme`] by default.
pub type Container<'a, Message, T = crate::Theme, Renderer = iced::Renderer> =
    iced::widget::Container<'a, Message, T, Renderer>;

#[allow(clippy::cast_precision_loss)]
macro_rules! from_argb {
    ($hex:expr) => {{
        let hex = $hex as u32;

        let a = ((hex & 0xff000000) >> 24) as f32 / 255.0;
        let r = (hex & 0x00ff0000) >> 16;
        let g = (hex & 0x0000ff00) >> 8;
        let b = (hex & 0x000000ff);

        ::iced_widget::core::color!(r as u8, g as u8, b as u8, a)
    }};
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(from = "Custom", into = "Custom"))]
pub enum Theme {
    Dark,
    Light,
    Custom(Custom),
}

impl Theme {
    pub const ALL: &'static [Self] = &[Self::Dark, Self::Light];

    pub fn new(name: impl Into<Cow<'static, str>>, colorscheme: ColorScheme) -> Self {
        Self::Custom(Custom {
            name: name.into(),
            is_dark: lightness(colorscheme.surface.color) <= 0.5,
            colorscheme,
        })
    }

    pub const fn new_const(name: &'static str, colorscheme: ColorScheme) -> Self {
        Self::Custom(Custom {
            name: Cow::Borrowed(name),
            is_dark: lightness(colorscheme.surface.color) <= 0.5,
            colorscheme,
        })
    }

    pub fn name(&self) -> &str {
        match self {
            Self::Dark => "Dark",
            Self::Light => "Light",
            Self::Custom(custom) => &custom.name,
        }
    }

    pub fn is_dark(&self) -> bool {
        match self {
            Self::Dark => true,
            Self::Light => false,
            Self::Custom(custom) => custom.is_dark,
        }
    }

    pub fn colors(&self) -> ColorScheme {
        match self {
            Self::Dark => ColorScheme::DARK,
            Self::Light => ColorScheme::LIGHT,
            Self::Custom(custom) => custom.colorscheme,
        }
    }
}

impl std::fmt::Display for Theme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl Base for Theme {
    fn default(preference: Mode) -> Self {
        match preference {
            Mode::None | Mode::Dark => Self::Dark,
            Mode::Light => Self::Light,
        }
    }

    fn mode(&self) -> Mode {
        if self.is_dark() {
            Mode::Dark
        } else {
            Mode::Light
        }
    }

    fn base(&self) -> Style {
        Style {
            background_color: self.colors().surface.color,
            text_color: self.colors().surface.text,
        }
    }

    fn palette(&self) -> Option<iced_widget::theme::Palette> {
        let colors = self.colors();

        Some(iced_widget::theme::Palette {
            background: colors.surface.color,
            text: colors.surface.text,
            primary: colors.primary.color,
            success: colors.primary.container,
            warning: mix(from_argb!(0xffffff00), colors.primary.color, 0.25),
            danger: colors.error.color,
        })
    }

    fn name(&self) -> &str {
        self.name()
    }
}

#[cfg(feature = "animate")]
impl iced_anim::Animate for Theme {
    fn components() -> usize {
        ColorScheme::components()
    }

    fn update(&mut self, components: &mut impl Iterator<Item = f32>) {
        let mut colorscheme = self.colors();
        colorscheme.update(components);
        *self = Self::Custom(Custom {
            name: "Animating Theme".into(),
            is_dark: lightness(colorscheme.surface.color) <= 0.5,
            colorscheme,
        });
    }

    fn distance_to(&self, end: &Self) -> Vec<f32> {
        self.colors().distance_to(&end.colors())
    }

    fn lerp(&mut self, start: &Self, end: &Self, progress: f32) {
        let mut colorscheme = self.colors();
        colorscheme.lerp(&start.colors(), &end.colors(), progress);
        *self = Self::Custom(Custom {
            name: "Animating Theme".into(),
            is_dark: lightness(colorscheme.surface.color) <= 0.5,
            colorscheme,
        });
    }
}

/// A custom [`Theme`].
#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Custom {
    /// The [`Theme`]'s name.
    pub name: Cow<'static, str>,
    /// Whether the [`Theme`] is dark.
    pub is_dark: bool,
    /// The [`Theme`]'s [`ColorScheme`].
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub colorscheme: ColorScheme,
}

impl From<Custom> for Theme {
    fn from(custom: Custom) -> Self {
        Self::Custom(custom)
    }
}

impl From<Theme> for Custom {
    fn from(theme: Theme) -> Self {
        match theme {
            Theme::Custom(custom) => custom,
            theme => Self {
                name: theme.name().to_owned().into(),
                is_dark: theme.is_dark(),
                colorscheme: theme.colors(),
            },
        }
    }
}

impl Clone for Custom {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            is_dark: self.is_dark,
            colorscheme: self.colorscheme,
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.name.clone_from(&source.name);
        self.is_dark = source.is_dark;
        self.colorscheme = source.colorscheme;
    }
}

/// A [`Theme`]'s color scheme.
///
/// These color roles are based on Material Design 3. For more information about them, visit the
/// official [M3 documentation](https://m3.material.io/styles/color/roles).
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "animate", derive(iced_anim::Animate))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ColorScheme {
    /// The primary colors.
    pub primary: ColorQuartet,
    /// The secondary colors.
    pub secondary: ColorQuartet,
    /// The tertiary colors.
    pub tertiary: ColorQuartet,
    /// The error colors.
    pub error: ColorQuartet,
    /// The surface colors.
    pub surface: Surface,
    /// The inverse colors.
    pub inverse: Inverse,
    /// The outline colors.
    pub outline: Outline,
    /// The shadow color.
    #[cfg_attr(feature = "serde", serde(with = "color_serde"))]
    pub shadow: Color,
    /// The scrim color.
    #[cfg_attr(feature = "serde", serde(with = "color_serde"))]
    pub scrim: Color,
}

#[allow(clippy::cast_precision_loss)]
impl ColorScheme {
    const DARK: Self = Self {
        primary: ColorQuartet {
            color: color!(0xd0bcff),
            text: color!(0x381e72),
            container: color!(0x4f378b),
            container_text: color!(0xeaddff),
        },
        secondary: ColorQuartet {
            color: color!(0xccc2dc),
            text: color!(0x332d41),
            container: color!(0x4a4458),
            container_text: color!(0xe8def8),
        },
        tertiary: ColorQuartet {
            color: color!(0xefb8c8),
            text: color!(0x492532),
            container: color!(0x633b48),
            container_text: color!(0xffd8e4),
        },
        error: ColorQuartet {
            color: color!(0xf2b8b5),
            text: color!(0x601410),
            container: color!(0x8c1d18),
            container_text: color!(0xf9dedc),
        },
        surface: Surface {
            color: color!(0x141218),
            text: color!(0xe6e0e9),
            text_variant: color!(0xcac4d0),
            container: SurfaceContainer {
                lowest: color!(0x0f0d13),
                low: color!(0x1d1b20),
                base: color!(0x211f26),
                high: color!(0x2b2930),
                highest: color!(0x36343b),
            },
        },
        inverse: Inverse {
            inverse_surface: color!(0xe6e0e9),
            inverse_surface_text: color!(0x322f35),
            inverse_primary: color!(0x6750a4),
        },
        outline: Outline {
            color: color!(0x938f99),
            variant: color!(0x49454f),
        },
        shadow: color!(0x000000),
        scrim: from_argb!(0x4d000000),
    };

    const LIGHT: Self = Self {
        primary: ColorQuartet {
            color: color!(0x6750a4),
            text: color!(0xffffff),
            container: color!(0xeaddff),
            container_text: color!(0x21005d),
        },
        secondary: ColorQuartet {
            color: color!(0x625b71),
            text: color!(0xffffff),
            container: color!(0xe8def8),
            container_text: color!(0x1d192b),
        },
        tertiary: ColorQuartet {
            color: color!(0x7d5260),
            text: color!(0xffffff),
            container: color!(0xffd8e4),
            container_text: color!(0x31111d),
        },
        error: ColorQuartet {
            color: color!(0xb3261e),
            text: color!(0xffffff),
            container: color!(0xf9dedc),
            container_text: color!(0x410e0b),
        },
        surface: Surface {
            color: color!(0xfef7ff),
            text: color!(0x1d1b20),
            text_variant: color!(0x49454f),
            container: SurfaceContainer {
                lowest: color!(0xffffff),
                low: color!(0xf7f2fa),
                base: color!(0xf3edf7),
                high: color!(0xece6f0),
                highest: color!(0xe6e0e9),
            },
        },
        inverse: Inverse {
            inverse_surface: color!(0x322f35),
            inverse_surface_text: color!(0xf5eff7),
            inverse_primary: color!(0xd0bcff),
        },
        outline: Outline {
            color: color!(0x79747e),
            variant: color!(0xcac4d0),
        },
        shadow: color!(0x000000),
        scrim: from_argb!(0x4d000000),
    };
}

impl ColorScheme {
    pub fn interpolate(from: Self, to: Self, amount: f32) -> Self {
        Self {
            primary: interpolate_color_quartet(from.primary, to.primary, amount),
            secondary: interpolate_color_quartet(from.secondary, to.secondary, amount),
            tertiary: interpolate_color_quartet(from.tertiary, to.tertiary, amount),
            error: interpolate_color_quartet(from.error, to.error, amount),
            surface: interpolate_surface(from.surface, to.surface, amount),
            inverse: interpolate_inverse(from.inverse, to.inverse, amount),
            outline: interpolate_outline(from.outline, to.outline, amount),
            shadow: interpolate_color(from.shadow, to.shadow, amount),
            scrim: interpolate_color(from.scrim, to.scrim, amount),
        }
    }
}

fn interpolate_color_quartet(from: ColorQuartet, to: ColorQuartet, amount: f32) -> ColorQuartet {
    ColorQuartet {
        color: interpolate_color(from.color, to.color, amount),
        text: interpolate_color(from.text, to.text, amount),
        container: interpolate_color(from.container, to.container, amount),
        container_text: interpolate_color(from.container_text, to.container_text, amount),
    }
}

fn interpolate_surface(from: Surface, to: Surface, amount: f32) -> Surface {
    Surface {
        color: interpolate_color(from.color, to.color, amount),
        text: interpolate_color(from.text, to.text, amount),
        text_variant: interpolate_color(from.text_variant, to.text_variant, amount),
        container: interpolate_surface_container(from.container, to.container, amount),
    }
}

fn interpolate_surface_container(
    from: SurfaceContainer,
    to: SurfaceContainer,
    amount: f32,
) -> SurfaceContainer {
    SurfaceContainer {
        lowest: interpolate_color(from.lowest, to.lowest, amount),
        low: interpolate_color(from.low, to.low, amount),
        base: interpolate_color(from.base, to.base, amount),
        high: interpolate_color(from.high, to.high, amount),
        highest: interpolate_color(from.highest, to.highest, amount),
    }
}

fn interpolate_inverse(from: Inverse, to: Inverse, amount: f32) -> Inverse {
    Inverse {
        inverse_surface: interpolate_color(from.inverse_surface, to.inverse_surface, amount),
        inverse_surface_text: interpolate_color(
            from.inverse_surface_text,
            to.inverse_surface_text,
            amount,
        ),
        inverse_primary: interpolate_color(from.inverse_primary, to.inverse_primary, amount),
    }
}

fn interpolate_outline(from: Outline, to: Outline, amount: f32) -> Outline {
    Outline {
        color: interpolate_color(from.color, to.color, amount),
        variant: interpolate_color(from.variant, to.variant, amount),
    }
}

fn interpolate_color(from: Color, to: Color, amount: f32) -> Color {
    if amount <= 0.0 {
        return from;
    }

    if amount >= 1.0 {
        return to;
    }

    Color {
        r: interpolate_component(from.r, to.r, amount),
        g: interpolate_component(from.g, to.g, amount),
        b: interpolate_component(from.b, to.b, amount),
        a: interpolate_component(from.a, to.a, amount),
    }
}

fn interpolate_component(from: f32, to: f32, amount: f32) -> f32 {
    from + (to - from) * amount
}

#[cfg(test)]
mod color_scheme_tests {
    use super::{ColorScheme, color};

    #[test]
    fn light_scheme_matches_m3_baseline_roles() {
        let scheme = ColorScheme::LIGHT;

        assert_eq!(scheme.primary.color, color!(0x6750a4));
        assert_eq!(scheme.primary.text, color!(0xffffff));
        assert_eq!(scheme.primary.container, color!(0xeaddff));
        assert_eq!(scheme.surface.color, color!(0xfef7ff));
        assert_eq!(scheme.surface.container.low, color!(0xf7f2fa));
        assert_eq!(scheme.surface.container.highest, color!(0xe6e0e9));
        assert_eq!(scheme.outline.color, color!(0x79747e));
    }

    #[test]
    fn dark_scheme_matches_m3_baseline_roles() {
        let scheme = ColorScheme::DARK;

        assert_eq!(scheme.primary.color, color!(0xd0bcff));
        assert_eq!(scheme.primary.text, color!(0x381e72));
        assert_eq!(scheme.primary.container, color!(0x4f378b));
        assert_eq!(scheme.surface.color, color!(0x141218));
        assert_eq!(scheme.surface.container.lowest, color!(0x0f0d13));
        assert_eq!(scheme.surface.container.highest, color!(0x36343b));
        assert_eq!(scheme.outline.color, color!(0x938f99));
    }

    #[test]
    fn color_scheme_interpolation_clamps_to_endpoints() {
        assert_eq!(
            ColorScheme::interpolate(ColorScheme::DARK, ColorScheme::LIGHT, -1.0),
            ColorScheme::DARK
        );
        assert_eq!(
            ColorScheme::interpolate(ColorScheme::DARK, ColorScheme::LIGHT, 2.0),
            ColorScheme::LIGHT
        );
    }

    #[test]
    fn color_scheme_interpolation_moves_all_roles() {
        let midpoint = ColorScheme::interpolate(ColorScheme::DARK, ColorScheme::LIGHT, 0.5);

        assert_ne!(midpoint.primary.color, ColorScheme::DARK.primary.color);
        assert_ne!(midpoint.primary.color, ColorScheme::LIGHT.primary.color);
        assert_ne!(midpoint.surface.color, ColorScheme::DARK.surface.color);
        assert_ne!(midpoint.surface.color, ColorScheme::LIGHT.surface.color);
        assert_ne!(midpoint.outline.color, ColorScheme::DARK.outline.color);
        assert_ne!(midpoint.outline.color, ColorScheme::LIGHT.outline.color);
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "animate", derive(iced_anim::Animate))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ColorQuartet {
    #[cfg_attr(feature = "serde", serde(with = "color_serde"))]
    pub color: Color,
    #[cfg_attr(feature = "serde", serde(with = "color_serde"))]
    pub text: Color,
    #[cfg_attr(feature = "serde", serde(with = "color_serde"))]
    pub container: Color,
    #[cfg_attr(feature = "serde", serde(with = "color_serde"))]
    pub container_text: Color,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "animate", derive(iced_anim::Animate))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Surface {
    #[cfg_attr(feature = "serde", serde(with = "color_serde"))]
    pub color: Color,
    #[cfg_attr(feature = "serde", serde(with = "color_serde"))]
    pub text: Color,
    #[cfg_attr(feature = "serde", serde(with = "color_serde"))]
    pub text_variant: Color,
    pub container: SurfaceContainer,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "animate", derive(iced_anim::Animate))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SurfaceContainer {
    #[cfg_attr(feature = "serde", serde(with = "color_serde"))]
    pub lowest: Color,
    #[cfg_attr(feature = "serde", serde(with = "color_serde"))]
    pub low: Color,
    #[cfg_attr(feature = "serde", serde(with = "color_serde"))]
    pub base: Color,
    #[cfg_attr(feature = "serde", serde(with = "color_serde"))]
    pub high: Color,
    #[cfg_attr(feature = "serde", serde(with = "color_serde"))]
    pub highest: Color,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "animate", derive(iced_anim::Animate))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Inverse {
    #[cfg_attr(feature = "serde", serde(with = "color_serde"))]
    pub inverse_surface: Color,
    #[cfg_attr(feature = "serde", serde(with = "color_serde"))]
    pub inverse_surface_text: Color,
    #[cfg_attr(feature = "serde", serde(with = "color_serde"))]
    pub inverse_primary: Color,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "animate", derive(iced_anim::Animate))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Outline {
    #[cfg_attr(feature = "serde", serde(with = "color_serde"))]
    pub color: Color,
    #[cfg_attr(feature = "serde", serde(with = "color_serde"))]
    pub variant: Color,
}

#[cfg(feature = "serde")]
mod color_serde {
    use iced_widget::core::Color;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    use super::utils::{color_to_argb, parse_argb};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Color, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(String::deserialize(deserializer)
            .map(|hex| parse_argb(&hex))?
            .unwrap_or(Color::TRANSPARENT))
    }

    pub fn serialize<S>(color: &Color, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        color_to_argb(*color).serialize(serializer)
    }
}
