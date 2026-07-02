use iced_widget::core::{Background, Border, Shadow, border};
use iced_widget::scrollable::{AutoScroll, Catalog, Rail, Scroller, Status, Style, StyleFn};

use super::Theme;
use super::container::surface_container;
use super::utils::mix;
use crate::utils::{
    HOVERED_LAYER_OPACITY, PRESSED_LAYER_OPACITY, disabled_container, disabled_text,
};

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(default)
    }

    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style {
        class(self, status)
    }
}

pub fn default(theme: &Theme, status: Status) -> Style {
    let surface = theme.colors().surface;

    let active_rail = Rail {
        background: None,
        scroller: Scroller {
            background: surface.text.into(),
            border: border::rounded(9999),
        },
        border: Border::default(),
    };

    let disabled_rail = Rail {
        background: Some(Background::Color(disabled_container(surface.text))),
        scroller: Scroller {
            background: disabled_text(surface.text).into(),
            border: border::rounded(9999),
        },
        ..active_rail
    };

    let active_auto_scroll = AutoScroll {
        background: surface.color.into(),
        border: border::rounded(500).width(1).color(surface.text),
        icon: surface.text,
        shadow: Shadow::default(),
    };

    let disabled_auto_scroll = AutoScroll {
        background: disabled_container(surface.text).into(),
        border: border::rounded(9999),
        icon: disabled_text(surface.text),
        shadow: Shadow::default(),
    };

    let style = Style {
        container: surface_container(theme),
        vertical_rail: active_rail,
        horizontal_rail: active_rail,
        gap: None,
        auto_scroll: active_auto_scroll,
    };

    match status {
        Status::Active {
            is_horizontal_scrollbar_disabled,
            is_vertical_scrollbar_disabled,
        } => Style {
            horizontal_rail: if is_horizontal_scrollbar_disabled {
                disabled_rail
            } else {
                active_rail
            },
            vertical_rail: if is_vertical_scrollbar_disabled {
                disabled_rail
            } else {
                active_rail
            },
            auto_scroll: if is_vertical_scrollbar_disabled && is_horizontal_scrollbar_disabled {
                disabled_auto_scroll
            } else {
                active_auto_scroll
            },
            ..style
        },
        Status::Hovered {
            is_horizontal_scrollbar_hovered,
            is_vertical_scrollbar_hovered,
            is_horizontal_scrollbar_disabled,
            is_vertical_scrollbar_disabled,
        } => {
            let hovered_rail = Rail {
                scroller: Scroller {
                    background: mix(surface.text, surface.color, HOVERED_LAYER_OPACITY).into(),
                    border: border::rounded(9999),
                },
                ..active_rail
            };

            Style {
                horizontal_rail: if is_horizontal_scrollbar_disabled {
                    disabled_rail
                } else if is_horizontal_scrollbar_hovered {
                    hovered_rail
                } else {
                    active_rail
                },
                vertical_rail: if is_vertical_scrollbar_disabled {
                    disabled_rail
                } else if is_vertical_scrollbar_hovered {
                    hovered_rail
                } else {
                    active_rail
                },
                auto_scroll: if is_vertical_scrollbar_disabled && is_horizontal_scrollbar_disabled {
                    disabled_auto_scroll
                } else {
                    active_auto_scroll
                },
                ..style
            }
        }
        Status::Dragged {
            is_horizontal_scrollbar_dragged,
            is_vertical_scrollbar_dragged,
            is_horizontal_scrollbar_disabled,
            is_vertical_scrollbar_disabled,
        } => {
            let dragged_rail = Rail {
                scroller: Scroller {
                    background: mix(surface.text, surface.color, PRESSED_LAYER_OPACITY).into(),
                    border: border::rounded(9999),
                },
                ..active_rail
            };

            Style {
                horizontal_rail: if is_horizontal_scrollbar_disabled {
                    disabled_rail
                } else if is_horizontal_scrollbar_dragged {
                    dragged_rail
                } else {
                    active_rail
                },
                vertical_rail: if is_vertical_scrollbar_disabled {
                    disabled_rail
                } else if is_vertical_scrollbar_dragged {
                    dragged_rail
                } else {
                    active_rail
                },
                auto_scroll: if is_vertical_scrollbar_disabled && is_horizontal_scrollbar_disabled {
                    disabled_auto_scroll
                } else {
                    active_auto_scroll
                },
                ..style
            }
        }
    }
}
