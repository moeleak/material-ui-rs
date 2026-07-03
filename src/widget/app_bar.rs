//! Material 3 app bar constructors.

use iced_widget::core::text as core_text;
use iced_widget::core::{Background, Element, Length, Padding, alignment, border};
use iced_widget::graphics::geometry;
use iced_widget::text;
use iced_widget::{Column, Container, Row, Space, Text};

use super::{absolute_line_height, button::Button};
use crate::utils::shadow_from_level;
use crate::{Theme, button as button_style, fonts, tokens};

/// Status bar inset height used when previewing top app bars edge-to-edge.
pub const STATUS_BAR_HEIGHT: f32 = 24.0;

/// Creates a Material icon button suitable for app bars.
pub fn icon_button<'a, Message, Renderer>(
    icon_name: impl text::IntoFragment<'a>,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    Button::new(
        Container::new(fonts::icon(
            icon_name,
            tokens::component::app_bar::ICON_SIZE,
        ))
        .center_x(Length::Fixed(
            tokens::component::icon_button::CONTAINER_WIDTH,
        ))
        .center_y(Length::Fixed(
            tokens::component::icon_button::CONTAINER_HEIGHT,
        )),
    )
    .width(Length::Fixed(
        tokens::component::icon_button::CONTAINER_WIDTH,
    ))
    .height(Length::Fixed(
        tokens::component::icon_button::CONTAINER_HEIGHT,
    ))
    .padding(Padding::ZERO)
    .style(button_style::icon)
}

/// Creates a Material icon action suitable for app bars.
pub fn icon_action<'a, Message, Renderer>(
    icon_name: impl text::IntoFragment<'a>,
    on_press: Message,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    icon_button(icon_name).on_press(on_press).into()
}

/// Creates Material icon actions suitable for app bars.
pub fn icon_actions<'a, Message, Renderer, Icon>(
    actions: impl IntoIterator<Item = (Icon, Message)>,
) -> Vec<Element<'a, Message, Theme, Renderer>>
where
    Icon: text::IntoFragment<'a>,
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    actions
        .into_iter()
        .map(|(icon_name, on_press)| icon_action(icon_name, on_press))
        .collect()
}

/// Creates a small top app bar.
pub fn small<'a, Message, Renderer>(
    title: impl text::IntoFragment<'a>,
    leading: Option<Element<'a, Message, Theme, Renderer>>,
    actions: impl IntoIterator<Item = Element<'a, Message, Theme, Renderer>>,
) -> Container<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
{
    let title_text = tokens::component::app_bar::SMALL_TITLE_TEXT;
    let mut content = Row::new()
        .spacing(tokens::component::app_bar::ICON_BUTTON_SPACE)
        .padding(app_bar_padding())
        .align_y(alignment::Vertical::Center);

    if let Some(leading) = leading {
        content = content.push(leading);
    }

    content = content.push(
        Text::new(title)
            .size(title_text.size)
            .line_height(absolute_line_height(title_text.line_height))
            .width(Length::Fill),
    );

    for action in actions {
        content = content.push(action);
    }

    top_container(
        content,
        tokens::component::app_bar::SMALL_CONTAINER_HEIGHT,
        false,
    )
}

/// Creates a small top app bar with the on-scroll container color/elevation.
pub fn small_on_scroll<'a, Message, Renderer>(
    title: impl text::IntoFragment<'a>,
    leading: Option<Element<'a, Message, Theme, Renderer>>,
    actions: impl IntoIterator<Item = Element<'a, Message, Theme, Renderer>>,
) -> Container<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
{
    let title_text = tokens::component::app_bar::SMALL_TITLE_TEXT;
    let mut content = Row::new()
        .spacing(tokens::component::app_bar::ICON_BUTTON_SPACE)
        .padding(app_bar_padding())
        .align_y(alignment::Vertical::Center);

    if let Some(leading) = leading {
        content = content.push(leading);
    }

    content = content.push(
        Text::new(title)
            .size(title_text.size)
            .line_height(absolute_line_height(title_text.line_height))
            .width(Length::Fill),
    );

    for action in actions {
        content = content.push(action);
    }

    top_container(
        content,
        tokens::component::app_bar::SMALL_CONTAINER_HEIGHT,
        true,
    )
}

/// Creates a medium top app bar.
pub fn medium<'a, Message, Renderer>(
    title: impl text::IntoFragment<'a>,
    leading: Option<Element<'a, Message, Theme, Renderer>>,
    actions: impl IntoIterator<Item = Element<'a, Message, Theme, Renderer>>,
) -> Container<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
{
    flexible(title, leading, actions, FlexibleSize::Medium)
}

/// Creates a large top app bar.
pub fn large<'a, Message, Renderer>(
    title: impl text::IntoFragment<'a>,
    leading: Option<Element<'a, Message, Theme, Renderer>>,
    actions: impl IntoIterator<Item = Element<'a, Message, Theme, Renderer>>,
) -> Container<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
{
    flexible(title, leading, actions, FlexibleSize::Large)
}

/// Creates a bottom app bar.
pub fn bottom<'a, Message, Renderer>(
    actions: impl IntoIterator<Item = Element<'a, Message, Theme, Renderer>>,
    floating_action: Option<Element<'a, Message, Theme, Renderer>>,
) -> Container<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
{
    let mut content = Row::new()
        .spacing(tokens::component::app_bar::ICON_BUTTON_SPACE)
        .align_y(alignment::Vertical::Center)
        .width(Length::Fill);

    for action in actions {
        content = content.push(action);
    }

    content = content.push(Space::new().width(Length::Fill));

    if let Some(fab) = floating_action {
        content = content.push(fab);
    }

    Container::new(content)
        .height(Length::Fixed(
            tokens::component::bottom_app_bar::CONTAINER_HEIGHT,
        ))
        .width(Length::Fill)
        .padding(Padding {
            top: 0.0,
            right: 16.0,
            bottom: 0.0,
            left: 4.0,
        })
        .align_y(alignment::Vertical::Center)
        .style(bottom_style)
}

/// Creates a top app bar status inset using the top app bar surface color.
pub fn status_bar<'a, Message, Renderer>() -> Container<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + 'a,
{
    Container::new(Space::new())
        .height(Length::Fixed(STATUS_BAR_HEIGHT))
        .width(Length::Fill)
        .style(|theme| top_style(theme, false))
}

/// Prepends a status inset to a top app bar.
pub fn with_status_bar<'a, Message, Renderer>(
    app_bar: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Column<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + 'a,
{
    Column::new()
        .push(status_bar())
        .push(app_bar)
        .spacing(0)
        .width(Length::Fill)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FlexibleSize {
    Medium,
    Large,
}

impl FlexibleSize {
    fn container_height(self) -> f32 {
        match self {
            Self::Medium => tokens::component::app_bar::MEDIUM_CONTAINER_HEIGHT,
            Self::Large => tokens::component::app_bar::LARGE_CONTAINER_HEIGHT,
        }
    }

    fn title_text(self) -> tokens::typography::TypeScale {
        match self {
            Self::Medium => tokens::component::app_bar::MEDIUM_TITLE_TEXT,
            Self::Large => tokens::component::app_bar::LARGE_TITLE_TEXT,
        }
    }
}

fn flexible<'a, Message, Renderer>(
    title: impl text::IntoFragment<'a>,
    leading: Option<Element<'a, Message, Theme, Renderer>>,
    actions: impl IntoIterator<Item = Element<'a, Message, Theme, Renderer>>,
    size: FlexibleSize,
) -> Container<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
{
    let title_text = size.title_text();
    let content = Column::new()
        .push(navigation_row(leading, actions).height(Length::Fixed(
            tokens::component::app_bar::SMALL_CONTAINER_HEIGHT,
        )))
        .push(
            Container::new(
                Text::new(title)
                    .size(title_text.size)
                    .line_height(absolute_line_height(title_text.line_height)),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(Padding {
                top: 0.0,
                right: 16.0,
                bottom: 20.0,
                left: 16.0,
            })
            .align_y(alignment::Vertical::Bottom),
        )
        .width(Length::Fill);

    top_container(content, size.container_height(), false)
}

fn navigation_row<'a, Message, Renderer>(
    leading: Option<Element<'a, Message, Theme, Renderer>>,
    actions: impl IntoIterator<Item = Element<'a, Message, Theme, Renderer>>,
) -> Row<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + 'a,
{
    let mut content = Row::new()
        .spacing(tokens::component::app_bar::ICON_BUTTON_SPACE)
        .padding(app_bar_padding())
        .align_y(alignment::Vertical::Center)
        .width(Length::Fill);

    if let Some(leading) = leading {
        content = content.push(leading);
    }

    content = content.push(Space::new().width(Length::Fill));

    for action in actions {
        content = content.push(action);
    }

    content
}

fn app_bar_padding() -> Padding {
    Padding {
        top: 0.0,
        right: tokens::component::app_bar::TRAILING_SPACE,
        bottom: 0.0,
        left: tokens::component::app_bar::LEADING_SPACE,
    }
}

fn top_container<'a, Message, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
    height: f32,
    on_scroll: bool,
) -> Container<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
{
    Container::new(content)
        .height(Length::Fixed(height))
        .width(Length::Fill)
        .style(move |theme| top_style(theme, on_scroll))
}

fn top_style(theme: &Theme, on_scroll: bool) -> iced_widget::container::Style {
    let colors = theme.colors();
    let (background, elevation) = if on_scroll {
        (
            colors.surface.container.base,
            tokens::component::app_bar::ON_SCROLL_CONTAINER_ELEVATION_LEVEL,
        )
    } else {
        (
            colors.surface.color,
            tokens::component::app_bar::CONTAINER_ELEVATION_LEVEL,
        )
    };

    iced_widget::container::Style {
        background: Some(Background::Color(background)),
        text_color: Some(colors.surface.text),
        border: border::rounded(tokens::component::app_bar::CONTAINER_SHAPE),
        shadow: shadow_from_level(elevation, colors.shadow),
        snap: cfg!(feature = "crisp"),
    }
}

fn bottom_style(theme: &Theme) -> iced_widget::container::Style {
    let colors = theme.colors();

    iced_widget::container::Style {
        background: Some(Background::Color(colors.surface.container.base)),
        text_color: Some(colors.surface.text),
        border: border::rounded(tokens::component::bottom_app_bar::CONTAINER_SHAPE),
        shadow: shadow_from_level(
            tokens::component::bottom_app_bar::CONTAINER_ELEVATION_LEVEL,
            colors.shadow,
        ),
        snap: cfg!(feature = "crisp"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn top_app_bar_styles_use_surface_roles() {
        let theme = Theme::Light;
        let colors = theme.colors();

        let resting = top_style(&theme, false);
        assert_eq!(
            resting.background,
            Some(Background::Color(colors.surface.color))
        );
        assert_eq!(resting.shadow.offset.y, 0.0);

        let scrolled = top_style(&theme, true);
        assert_eq!(
            scrolled.background,
            Some(Background::Color(colors.surface.container.base))
        );
        assert_eq!(scrolled.shadow.offset.y, 2.0);
        assert_eq!(scrolled.shadow.blur_radius, 6.0);
    }

    #[test]
    fn bottom_app_bar_uses_surface_container_and_level2() {
        let theme = Theme::Light;
        let colors = theme.colors();
        let style = bottom_style(&theme);

        assert_eq!(
            style.background,
            Some(Background::Color(colors.surface.container.base))
        );
        assert_eq!(style.shadow.offset.y, 2.0);
        assert_eq!(style.shadow.blur_radius, 6.0);
    }

    #[test]
    fn status_bar_uses_fixed_edge_to_edge_inset() {
        let status: Container<'_, (), Theme, iced_widget::Renderer> = status_bar();

        assert_eq!(STATUS_BAR_HEIGHT, 24.0);
        assert_eq!(
            iced_widget::core::Widget::<(), Theme, iced_widget::Renderer>::size(&status).height,
            Length::Fixed(STATUS_BAR_HEIGHT)
        );
    }
}
