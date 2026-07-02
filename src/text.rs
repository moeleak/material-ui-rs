#![allow(dead_code)]
use iced_widget::Text;
use iced_widget::core::text as core_text;
use iced_widget::text::{self as iced_text, Catalog, LineHeight, Style, StyleFn};

use crate::{Theme, tokens};

pub fn line_height(scale: tokens::typography::TypeScale) -> LineHeight {
    LineHeight::Absolute(scale.line_height.into())
}

pub fn type_scale<'a, Renderer>(
    content: impl iced_text::IntoFragment<'a>,
    scale: tokens::typography::TypeScale,
) -> Text<'a, Theme, Renderer>
where
    Renderer: core_text::Renderer + 'a,
{
    Text::new(content)
        .size(scale.size)
        .line_height(line_height(scale))
}

pub fn headline_large<'a, Renderer>(
    content: impl iced_text::IntoFragment<'a>,
) -> Text<'a, Theme, Renderer>
where
    Renderer: core_text::Renderer + 'a,
{
    type_scale(content, tokens::typography::HEADLINE_LARGE)
}

pub fn headline_medium<'a, Renderer>(
    content: impl iced_text::IntoFragment<'a>,
) -> Text<'a, Theme, Renderer>
where
    Renderer: core_text::Renderer + 'a,
{
    type_scale(content, tokens::typography::HEADLINE_MEDIUM)
}

pub fn title_medium<'a, Renderer>(
    content: impl iced_text::IntoFragment<'a>,
) -> Text<'a, Theme, Renderer>
where
    Renderer: core_text::Renderer + 'a,
{
    type_scale(content, tokens::typography::TITLE_MEDIUM)
}

pub fn body_large<'a, Renderer>(
    content: impl iced_text::IntoFragment<'a>,
) -> Text<'a, Theme, Renderer>
where
    Renderer: core_text::Renderer + 'a,
{
    type_scale(content, tokens::typography::BODY_LARGE)
}

pub fn body_medium<'a, Renderer>(
    content: impl iced_text::IntoFragment<'a>,
) -> Text<'a, Theme, Renderer>
where
    Renderer: core_text::Renderer + 'a,
{
    type_scale(content, tokens::typography::BODY_MEDIUM)
}

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(none)
    }

    fn style(&self, class: &Self::Class<'_>) -> Style {
        class(self)
    }
}

pub fn none(_: &Theme) -> Style {
    Style { color: None }
}

pub fn primary(theme: &Theme) -> Style {
    Style {
        color: Some(theme.colors().primary.text),
    }
}

pub fn primary_container(theme: &Theme) -> Style {
    Style {
        color: Some(theme.colors().primary.container_text),
    }
}

pub fn secondary(theme: &Theme) -> Style {
    Style {
        color: Some(theme.colors().secondary.text),
    }
}

pub fn secondary_container(theme: &Theme) -> Style {
    Style {
        color: Some(theme.colors().secondary.container_text),
    }
}

pub fn tertiary(theme: &Theme) -> Style {
    Style {
        color: Some(theme.colors().tertiary.text),
    }
}

pub fn tertiary_container(theme: &Theme) -> Style {
    Style {
        color: Some(theme.colors().tertiary.container_text),
    }
}

pub fn error(theme: &Theme) -> Style {
    Style {
        color: Some(theme.colors().error.text),
    }
}

pub fn error_container(theme: &Theme) -> Style {
    Style {
        color: Some(theme.colors().error.container_text),
    }
}

pub fn surface(theme: &Theme) -> Style {
    Style {
        color: Some(theme.colors().surface.text),
    }
}

pub fn surface_variant(theme: &Theme) -> Style {
    Style {
        color: Some(theme.colors().surface.text_variant),
    }
}

pub fn inverse_surface(theme: &Theme) -> Style {
    Style {
        color: Some(theme.colors().inverse.inverse_surface_text),
    }
}
