use iced_widget::core::{Background, border};
use iced_widget::pane_grid::{Catalog, Highlight, Line, Style, StyleFn};

use super::Theme;
use crate::utils::{HOVERED_LAYER_OPACITY, mix};

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> <Self as Catalog>::Class<'a> {
        Box::new(default)
    }

    fn style(&self, class: &<Self as Catalog>::Class<'_>) -> Style {
        class(self)
    }
}

pub fn default(theme: &Theme) -> Style {
    Style {
        hovered_region: Highlight {
            background: Background::Color(mix(
                theme.colors().tertiary.container,
                theme.colors().surface.text,
                HOVERED_LAYER_OPACITY,
            )),
            border: border::rounded(12),
        },
        picked_split: Line {
            color: theme.colors().outline.variant,
            width: 2.0,
        },
        hovered_split: Line {
            color: theme.colors().surface.text,
            width: 6.0,
        },
    }
}
