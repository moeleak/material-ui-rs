#[cfg(target_os = "android")]
pub mod android;
pub mod animation;
pub mod application;
#[path = "design/fonts.rs"]
pub mod fonts;
mod internal;
#[path = "design/style/mod.rs"]
pub mod style;
#[path = "design/text.rs"]
pub mod text;
#[path = "design/theme.rs"]
mod theme;
#[path = "design/tokens/mod.rs"]
pub mod tokens;
pub mod widget;

pub(crate) use internal::{utils, web_input};
pub use theme::{
    ColorQuartet, ColorScheme, Custom, Inverse, Outline, Surface, SurfaceContainer, Theme,
};

pub use application::{
    application, window, window_settings, window_with_min_size, with_material_fonts,
};

/// An [`iced::Element`] that uses the bundled [`Theme`] by default.
pub type Element<'a, Message, T = crate::Theme, Renderer = iced::Renderer> =
    iced::Element<'a, Message, T, Renderer>;

/// An [`iced::widget::Container`] that uses the bundled [`Theme`] by default.
pub type Container<'a, Message, T = crate::Theme, Renderer = iced::Renderer> =
    iced::widget::Container<'a, Message, T, Renderer>;
