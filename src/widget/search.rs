//! Material 3 search bar and search view constructors.

use iced_widget::core::text as core_text;
use iced_widget::core::{Background, Border, Color, Element, Length, Padding, alignment, border};
use iced_widget::text_input::{Status, Style};
use iced_widget::{Column, Container, Row, TextInput as IcedTextInput};

use super::{absolute_line_height, mobile_text_input};
use crate::utils::{shadow_from_level, state_layer};
use crate::{Theme, fonts, tokens};

/// Creates a Material 3 search bar with a leading search icon.
pub fn bar<'a, Message, Renderer>(
    placeholder: impl Into<String>,
    value: &str,
    on_input: impl Fn(String) -> Message + 'a,
) -> Container<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    bar_with_trailing(placeholder, value, on_input, None)
}

/// Creates a Material 3 search bar with trailing content.
pub fn bar_with_trailing<'a, Message, Renderer>(
    placeholder: impl Into<String>,
    value: &str,
    on_input: impl Fn(String) -> Message + 'a,
    trailing: Option<Element<'a, Message, Theme, Renderer>>,
) -> Container<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    let placeholder = placeholder.into();
    let input_text = tokens::component::search_bar::INPUT_TEXT;
    let input = mobile_text_input(
        IcedTextInput::new(&placeholder, value)
            .on_input(on_input)
            .padding(Padding::ZERO)
            .size(input_text.size)
            .line_height(absolute_line_height(input_text.line_height))
            .style(input_style)
            .width(Length::Fill),
        true,
    );

    let mut content = Row::new()
        .push(fonts::icon(
            "search",
            tokens::component::search_bar::ICON_SIZE,
        ))
        .push(input)
        .spacing(tokens::component::search_bar::LEADING_ICON_LABEL_SPACE)
        .align_y(alignment::Vertical::Center);

    if let Some(trailing) = trailing {
        content = content
            .push(trailing)
            .spacing(tokens::component::search_bar::TRAILING_ICON_LABEL_SPACE);
    }

    Container::new(content)
        .height(Length::Fixed(
            tokens::component::search_bar::CONTAINER_HEIGHT,
        ))
        .width(Length::Fill)
        .padding(Padding {
            top: 0.0,
            right: tokens::component::search_bar::TRAILING_SPACE,
            bottom: 0.0,
            left: tokens::component::search_bar::LEADING_SPACE,
        })
        .align_y(alignment::Vertical::Center)
        .style(bar_container_style)
}

/// Creates a docked Material 3 search view with a header and result content.
pub fn docked_view<'a, Message, Renderer>(
    placeholder: impl Into<String>,
    value: &str,
    on_input: impl Fn(String) -> Message + 'a,
    results: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Container<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    search_view(
        placeholder,
        value,
        on_input,
        results,
        tokens::component::search_view::DOCKED_HEADER_CONTAINER_HEIGHT,
        tokens::component::search_view::DOCKED_CONTAINER_SHAPE,
    )
}

/// Creates a full-screen-style Material 3 search view surface.
pub fn full_screen_view<'a, Message, Renderer>(
    placeholder: impl Into<String>,
    value: &str,
    on_input: impl Fn(String) -> Message + 'a,
    results: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Container<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    search_view(
        placeholder,
        value,
        on_input,
        results,
        tokens::component::search_view::FULL_SCREEN_HEADER_CONTAINER_HEIGHT,
        tokens::component::search_view::FULL_SCREEN_CONTAINER_SHAPE,
    )
}

fn search_view<'a, Message, Renderer>(
    placeholder: impl Into<String>,
    value: &str,
    on_input: impl Fn(String) -> Message + 'a,
    results: impl Into<Element<'a, Message, Theme, Renderer>>,
    header_height: f32,
    shape: f32,
) -> Container<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    let placeholder = placeholder.into();
    let input_text = tokens::component::search_view::HEADER_INPUT_TEXT;
    let input = mobile_text_input(
        IcedTextInput::new(&placeholder, value)
            .on_input(on_input)
            .padding(Padding::ZERO)
            .size(input_text.size)
            .line_height(absolute_line_height(input_text.line_height))
            .style(input_style)
            .width(Length::Fill),
        true,
    );

    let header = Row::new()
        .push(fonts::icon(
            "arrow_back",
            tokens::component::search_bar::ICON_SIZE,
        ))
        .push(input)
        .spacing(tokens::component::search_view::LEADING_ICON_LABEL_SPACE)
        .align_y(alignment::Vertical::Center);

    let content = Column::new()
        .push(
            Container::new(header)
                .height(Length::Fixed(header_height))
                .padding(Padding {
                    top: 0.0,
                    right: tokens::component::search_view::TRAILING_SPACE,
                    bottom: 0.0,
                    left: tokens::component::search_view::LEADING_SPACE,
                })
                .align_y(alignment::Vertical::Center),
        )
        .push(super::rule::horizontal_full_width())
        .push(results)
        .width(Length::Fill);

    Container::new(content)
        .width(Length::Fill)
        .style(move |theme| view_container_style(theme, shape))
}

fn input_style(theme: &Theme, _status: Status) -> Style {
    let colors = theme.colors();

    Style {
        background: Background::Color(Color::TRANSPARENT),
        border: Border::default(),
        icon: colors.surface.text,
        placeholder: colors.surface.text_variant,
        value: colors.surface.text,
        selection: state_layer(
            colors.primary.color,
            tokens::state::FOCUS_STATE_LAYER_OPACITY,
        ),
    }
}

fn bar_container_style(theme: &Theme) -> iced_widget::container::Style {
    let colors = theme.colors();

    iced_widget::container::Style {
        background: Some(Background::Color(colors.surface.container.high)),
        text_color: Some(colors.surface.text),
        border: border::rounded(tokens::component::search_bar::CONTAINER_SHAPE),
        shadow: shadow_from_level(
            tokens::component::search_bar::CONTAINER_ELEVATION_LEVEL,
            colors.shadow,
        ),
        snap: cfg!(feature = "crisp"),
    }
}

fn view_container_style(theme: &Theme, shape: f32) -> iced_widget::container::Style {
    let colors = theme.colors();

    iced_widget::container::Style {
        background: Some(Background::Color(colors.surface.container.high)),
        text_color: Some(colors.surface.text),
        border: border::rounded(shape),
        shadow: shadow_from_level(
            tokens::component::search_view::CONTAINER_ELEVATION_LEVEL,
            colors.shadow,
        ),
        snap: cfg!(feature = "crisp"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_bar_uses_surface_container_high_and_level3() {
        let theme = Theme::Light;
        let colors = theme.colors();
        let style = bar_container_style(&theme);

        assert_eq!(
            style.background,
            Some(Background::Color(colors.surface.container.high))
        );
        assert_eq!(
            style.border.radius.top_left,
            tokens::component::search_bar::CONTAINER_SHAPE
        );
        assert_eq!(style.shadow.offset.y, 4.0);
        assert_eq!(style.shadow.blur_radius, 8.0);
    }

    #[test]
    fn search_input_style_uses_body_surface_roles() {
        let theme = Theme::Light;
        let colors = theme.colors();
        let style = input_style(&theme, Status::Active);

        assert_eq!(style.value, colors.surface.text);
        assert_eq!(style.placeholder, colors.surface.text_variant);
        assert_eq!(style.background, Background::Color(Color::TRANSPARENT));
    }
}
