//! Material 3 dialog surface constructors.

use iced_widget::core::text as core_text;
use iced_widget::core::{Background, Color, Element, Length, Padding, alignment, border};
use iced_widget::graphics::geometry;
use iced_widget::text;
use iced_widget::{Column, Container, Row, Space, Stack, Text, opaque};

use super::absolute_line_height;
use super::support::alpha_color;
use crate::utils::shadow_from_level;
use crate::{Theme, fonts, tokens};

/// Creates a Material 3 basic dialog surface around custom content.
pub fn basic<'a, Message, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Container<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
{
    Container::new(content)
        .width(Length::Fill)
        .max_width(tokens::component::dialog::CONTAINER_MAX_WIDTH)
        .padding(tokens::component::dialog::CONTAINER_PADDING)
        .style(container_style)
}

/// Creates a Material 3 alert dialog with title, supporting text, and actions.
pub fn alert<'a, Message, Renderer>(
    title: impl text::IntoFragment<'a>,
    supporting_text: impl text::IntoFragment<'a>,
    actions: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Container<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    alert_content(None, title, supporting_text, actions)
}

/// Creates a Material 3 alert dialog with the optional hero icon slot populated.
pub fn alert_with_icon<'a, Message, Renderer>(
    icon: impl text::IntoFragment<'a>,
    title: impl text::IntoFragment<'a>,
    supporting_text: impl text::IntoFragment<'a>,
    actions: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Container<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    alert_content(Some(icon.into_fragment()), title, supporting_text, actions)
}

/// Creates a right-aligned Material 3 dialog actions row.
pub fn actions<'a, Message, Renderer>(
    buttons: impl IntoIterator<Item = Element<'a, Message, Theme, Renderer>>,
) -> Container<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + 'a,
{
    Container::new(
        Row::with_children(buttons.into_iter())
            .spacing(tokens::component::dialog::ACTIONS_HORIZONTAL_SPACING)
            .align_y(alignment::Vertical::Center),
    )
    .width(Length::Fill)
    .align_x(alignment::Horizontal::Right)
}

/// Creates a Material 3 dialog text action.
pub fn action<'a, Message, Renderer>(
    label: impl text::IntoFragment<'a>,
) -> super::button::Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
{
    super::button::text(label)
}

/// Creates a Material 3 dialog text action with an on-press message.
pub fn action_button<'a, Message, Renderer>(
    label: impl text::IntoFragment<'a>,
    on_press: Message,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
{
    action(label).on_press(on_press).into()
}

/// Creates a Material 3 modal dialog scrim behind overlay content.
pub fn scrim<'a, Message, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Container<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + 'a,
{
    Container::new(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(scrim_style)
}

/// Centers a Material 3 dialog surface over a modal scrim.
pub fn modal_overlay<'a, Message, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Container<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + 'a,
{
    scrim(
        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center),
    )
}

/// Creates an event-blocking Material 3 modal dialog layer.
///
/// The scrim and dialog surface both absorb mouse presses, so clicks outside
/// the dialog do not pass through to content underneath and do not require a
/// no-op application message.
pub fn modal_layer<'a, Message, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + 'a,
{
    let scrim = opaque(scrim(Space::new().width(Length::Fill).height(Length::Fill)));
    let dialog = opaque(Container::new(content).center(Length::Fill));

    Stack::with_children([scrim, dialog])
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// Places a Material 3 modal dialog layer over existing content.
pub fn modal<'a, Message, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
    dialog: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + 'a,
{
    Stack::with_children([content.into(), modal_layer(dialog)])
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn alert_content<'a, Message, Renderer>(
    icon: Option<text::Fragment<'a>>,
    title: impl text::IntoFragment<'a>,
    supporting_text: impl text::IntoFragment<'a>,
    actions: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Container<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    let title_alignment = title_alignment(icon.is_some());
    let mut content = Column::new().width(Length::Fill);

    if let Some(icon) = icon {
        content = content.push(
            Container::new(icon_text(icon))
                .width(Length::Fill)
                .padding(Padding {
                    top: 0.0,
                    right: 0.0,
                    bottom: tokens::component::dialog::ICON_BOTTOM_PADDING,
                    left: 0.0,
                })
                .align_x(alignment::Horizontal::Center),
        );
    }

    content = content.push(
        Container::new(title_text(title, title_alignment))
            .width(Length::Fill)
            .padding(Padding {
                top: 0.0,
                right: 0.0,
                bottom: tokens::component::dialog::TITLE_BOTTOM_PADDING,
                left: 0.0,
            }),
    );

    content = content.push(
        Container::new(supporting_text_view(supporting_text))
            .width(Length::Fill)
            .padding(Padding {
                top: 0.0,
                right: 0.0,
                bottom: tokens::component::dialog::SUPPORTING_TEXT_BOTTOM_PADDING,
                left: 0.0,
            }),
    );

    content = content.push(actions.into());

    basic(content)
}

fn icon_text<'a, Renderer>(icon: text::Fragment<'a>) -> Text<'a, Theme, Renderer>
where
    Renderer: core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    fonts::filled_icon(icon, tokens::component::dialog::ICON_SIZE)
        .width(Length::Fixed(tokens::component::dialog::ICON_SIZE))
        .height(Length::Fixed(tokens::component::dialog::ICON_SIZE))
        .center()
        .style(icon_style)
}

fn title_text<'a, Renderer>(
    title: impl text::IntoFragment<'a>,
    alignment: alignment::Horizontal,
) -> Text<'a, Theme, Renderer>
where
    Renderer: core_text::Renderer + 'a,
{
    let scale = tokens::component::dialog::HEADLINE_TEXT;

    Text::new(title)
        .size(scale.size)
        .line_height(absolute_line_height(scale.line_height))
        .width(Length::Fill)
        .align_x(alignment)
        .color_maybe(None::<iced_widget::core::Color>)
        .style(title_style)
}

fn title_alignment(has_icon: bool) -> alignment::Horizontal {
    if has_icon {
        alignment::Horizontal::Center
    } else {
        alignment::Horizontal::Left
    }
}

fn supporting_text_view<'a, Renderer>(
    supporting_text: impl text::IntoFragment<'a>,
) -> Text<'a, Theme, Renderer>
where
    Renderer: core_text::Renderer + 'a,
{
    let scale = tokens::component::dialog::SUPPORTING_TEXT;

    Text::new(supporting_text)
        .size(scale.size)
        .line_height(absolute_line_height(scale.line_height))
        .width(Length::Fill)
        .color_maybe(None::<iced_widget::core::Color>)
        .style(supporting_text_style)
}

fn container_style(theme: &Theme) -> iced_widget::container::Style {
    let colors = theme.colors();

    iced_widget::container::Style {
        background: Some(Background::Color(colors.surface.container.high)),
        text_color: Some(colors.surface.text_variant),
        border: border::rounded(tokens::component::dialog::CONTAINER_SHAPE),
        shadow: shadow_from_level(
            tokens::component::dialog::CONTAINER_ELEVATION_LEVEL,
            colors.shadow,
        ),
        snap: cfg!(feature = "crisp"),
    }
}

fn icon_style(theme: &Theme) -> iced_widget::text::Style {
    iced_widget::text::Style {
        color: Some(theme.colors().secondary.color),
    }
}

fn title_style(theme: &Theme) -> iced_widget::text::Style {
    iced_widget::text::Style {
        color: Some(theme.colors().surface.text),
    }
}

fn supporting_text_style(theme: &Theme) -> iced_widget::text::Style {
    iced_widget::text::Style {
        color: Some(theme.colors().surface.text_variant),
    }
}

fn scrim_style(theme: &Theme) -> iced_widget::container::Style {
    iced_widget::container::Style {
        background: Some(Background::Color(alpha_color(
            Color {
                a: 1.0,
                ..theme.colors().scrim
            },
            tokens::component::dialog::SCRIM_OPACITY,
        ))),
        text_color: Some(theme.colors().surface.text),
        ..iced_widget::container::Style::default()
    }
}

#[cfg(test)]
mod tests {
    use iced_widget::core::Widget;

    use super::*;

    #[derive(Debug, Clone)]
    enum Message {}

    #[test]
    fn dialog_container_style_uses_material_tokens() {
        let theme = Theme::Light;
        let colors = theme.colors();
        let style = container_style(&theme);

        assert_eq!(
            style.background,
            Some(Background::Color(colors.surface.container.high))
        );
        assert_eq!(
            style.border.radius.top_left,
            tokens::component::dialog::CONTAINER_SHAPE
        );
        assert_eq!(style.shadow.offset.y, 4.0);
        assert_eq!(style.shadow.blur_radius, 8.0);
    }

    #[test]
    fn dialog_content_styles_use_material_color_roles() {
        let theme = Theme::Light;
        let colors = theme.colors();

        assert_eq!(icon_style(&theme).color, Some(colors.secondary.color));
        assert_eq!(title_style(&theme).color, Some(colors.surface.text));
        assert_eq!(
            supporting_text_style(&theme).color,
            Some(colors.surface.text_variant)
        );
    }

    #[test]
    fn dialog_title_alignment_follows_icon_presence() {
        assert_eq!(title_alignment(true), alignment::Horizontal::Center);
        assert_eq!(title_alignment(false), alignment::Horizontal::Left);
    }

    #[test]
    fn dialog_title_text_fills_width_for_alignment() {
        let title: Text<'_, Theme, iced_widget::Renderer> =
            title_text("Discard draft?", alignment::Horizontal::Center);

        assert_eq!(
            Widget::<Message, Theme, iced_widget::Renderer>::size(&title).width,
            Length::Fill
        );
    }

    #[test]
    fn dialog_scrim_uses_material_scrim_opacity() {
        let theme = Theme::Light;
        let style = scrim_style(&theme);
        let Some(Background::Color(color)) = style.background else {
            panic!("expected solid scrim background");
        };

        assert_eq!(color.a, tokens::component::dialog::SCRIM_OPACITY);
        assert_eq!(style.text_color, Some(theme.colors().surface.text));
    }
}
