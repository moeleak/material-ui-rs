//! Material 3 list item constructors with token-backed layout defaults.

use iced_widget::core::text as core_text;
use iced_widget::core::{alignment, Length, Padding};
use iced_widget::text;
use iced_widget::{Column, Container, Row, Text};

use super::absolute_line_height;
use crate::{list as list_style, text as text_style, tokens, Theme};

pub fn one_line<'a, Message, Renderer>(
    label: impl text::IntoFragment<'a>,
) -> Container<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
{
    item(
        Row::<Message, Theme, Renderer>::new()
            .push(label_text(label).width(Length::Fill))
            .align_y(alignment::Vertical::Center),
        tokens::component::list::ONE_LINE_CONTAINER_HEIGHT,
    )
}

pub fn one_line_with_leading_icon<'a, Message, Renderer>(
    icon: impl text::IntoFragment<'a>,
    label: impl text::IntoFragment<'a>,
) -> Container<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
{
    item(
        Row::<Message, Theme, Renderer>::new()
            .push(leading_icon(icon))
            .push(label_text(label).width(Length::Fill))
            .spacing(tokens::component::list::LEADING_SPACE)
            .align_y(alignment::Vertical::Center),
        tokens::component::list::ONE_LINE_CONTAINER_HEIGHT,
    )
}

pub fn two_line<'a, Message, Renderer>(
    label: impl text::IntoFragment<'a>,
    supporting: impl text::IntoFragment<'a>,
) -> Container<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
{
    item(
        Row::<Message, Theme, Renderer>::new()
            .push(text_column(label, supporting).width(Length::Fill))
            .align_y(alignment::Vertical::Center),
        tokens::component::list::TWO_LINE_CONTAINER_HEIGHT,
    )
}

pub fn two_line_with_trailing<'a, Message, Renderer>(
    label: impl text::IntoFragment<'a>,
    supporting: impl text::IntoFragment<'a>,
    trailing: impl text::IntoFragment<'a>,
) -> Container<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
{
    item(
        Row::<Message, Theme, Renderer>::new()
            .push(text_column(label, supporting).width(Length::Fill))
            .push(trailing_supporting_text(trailing))
            .spacing(tokens::component::list::TRAILING_SPACE)
            .align_y(alignment::Vertical::Center),
        tokens::component::list::TWO_LINE_CONTAINER_HEIGHT,
    )
}

pub fn three_line<'a, Message, Renderer>(
    label: impl text::IntoFragment<'a>,
    supporting: impl text::IntoFragment<'a>,
    supporting_second_line: impl text::IntoFragment<'a>,
) -> Container<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
{
    let supporting_text = tokens::component::list::SUPPORTING_TEXT;

    item(
        Row::<Message, Theme, Renderer>::new()
            .push(
                Column::<Message, Theme, Renderer>::new()
                    .push(label_text(label))
                    .push(
                        Text::new(supporting)
                            .size(supporting_text.size)
                            .line_height(absolute_line_height(supporting_text.line_height))
                            .style(text_style::surface_variant),
                    )
                    .push(
                        Text::new(supporting_second_line)
                            .size(supporting_text.size)
                            .line_height(absolute_line_height(supporting_text.line_height))
                            .style(text_style::surface_variant),
                    )
                    .width(Length::Fill),
            )
            .align_y(alignment::Vertical::Center),
        tokens::component::list::THREE_LINE_CONTAINER_HEIGHT,
    )
}

fn item<'a, Message, Renderer>(
    content: Row<'a, Message, Theme, Renderer>,
    height: f32,
) -> Container<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
{
    Container::new(content)
        .height(Length::Fixed(height))
        .width(Length::Fill)
        .padding(Padding {
            top: 0.0,
            right: tokens::component::list::TRAILING_SPACE,
            bottom: 0.0,
            left: tokens::component::list::LEADING_SPACE,
        })
        .align_y(alignment::Vertical::Center)
        .style(list_style::item)
}

fn text_column<'a, Message, Renderer>(
    label: impl text::IntoFragment<'a>,
    supporting: impl text::IntoFragment<'a>,
) -> Column<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: core_text::Renderer + 'a,
{
    let supporting_text = tokens::component::list::SUPPORTING_TEXT;

    Column::<Message, Theme, Renderer>::new()
        .push(label_text(label))
        .push(
            Text::new(supporting)
                .size(supporting_text.size)
                .line_height(absolute_line_height(supporting_text.line_height))
                .style(text_style::surface_variant),
        )
}

fn label_text<'a, Renderer>(
    label: impl text::IntoFragment<'a>,
) -> Text<'a, Theme, Renderer>
where
    Renderer: core_text::Renderer,
{
    let label_text = tokens::component::list::LABEL_TEXT;

    Text::new(label)
        .size(label_text.size)
        .line_height(absolute_line_height(label_text.line_height))
        .style(text_style::surface)
}

fn leading_icon<'a, Renderer>(
    icon: impl text::IntoFragment<'a>,
) -> Text<'a, Theme, Renderer>
where
    Renderer: core_text::Renderer,
{
    Text::new(icon)
        .size(tokens::component::list::LEADING_ICON_SIZE)
        .line_height(absolute_line_height(
            tokens::component::list::LEADING_ICON_SIZE,
        ))
        .style(text_style::surface_variant)
}

fn trailing_supporting_text<'a, Renderer>(
    trailing: impl text::IntoFragment<'a>,
) -> Text<'a, Theme, Renderer>
where
    Renderer: core_text::Renderer,
{
    let trailing_text = tokens::component::list::TRAILING_SUPPORTING_TEXT;

    Text::new(trailing)
        .size(trailing_text.size)
        .line_height(absolute_line_height(trailing_text.line_height))
        .style(text_style::surface_variant)
}
