use iced_widget::markdown::{Catalog, Settings, Style};
use iced_widget::theme::Base;

use super::{Theme, container};

impl Catalog for Theme {
    fn code_block<'a>() -> <Self as iced_widget::container::Catalog>::Class<'a> {
        Box::new(container::surface_container_highest)
    }
}

impl From<&Theme> for Settings {
    fn from(theme: &Theme) -> Self {
        Self::with_style(theme)
    }
}

impl From<Theme> for Settings {
    fn from(theme: Theme) -> Self {
        Self::with_style(theme)
    }
}

impl From<&Theme> for Style {
    fn from(theme: &Theme) -> Self {
        let palette = theme.palette().unwrap();

        Self::from_palette(palette)
    }
}

impl From<Theme> for Style {
    fn from(theme: Theme) -> Self {
        let palette = theme.palette().unwrap();

        Self::from_palette(palette)
    }
}
