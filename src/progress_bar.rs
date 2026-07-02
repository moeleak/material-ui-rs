use iced_widget::core::{Background, border};
use iced_widget::progress_bar::{Catalog, Style, StyleFn};

use super::Theme;
use crate::tokens;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(default)
    }

    fn style(&self, class: &Self::Class<'_>) -> Style {
        class(self)
    }
}

pub fn default(theme: &Theme) -> Style {
    Style {
        background: Background::Color(theme.colors().surface.container.highest),
        bar: Background::Color(theme.colors().primary.color),
        border: border::rounded(tokens::component::linear_progress::TRACK_SHAPE),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_progress_bar_uses_m3_linear_indicator_tokens() {
        let theme = Theme::Light;
        let colors = theme.colors();
        let style = default(&theme);

        assert_eq!(
            style.background,
            Background::Color(colors.surface.container.highest)
        );
        assert_eq!(style.bar, Background::Color(colors.primary.color));
        assert_eq!(
            style.border.radius.top_left,
            tokens::component::linear_progress::TRACK_SHAPE
        );
    }
}
