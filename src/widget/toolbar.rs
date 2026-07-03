//! Material 3 docked and floating toolbar constructors.

use iced_widget::button::{Status, Style};
use iced_widget::core::text as core_text;
use iced_widget::core::{Background, Color, Element, Length, Padding, alignment, border};
use iced_widget::graphics::geometry;
use iced_widget::text;
use iced_widget::{Column, Container, Row};

use super::{absolute_line_height, button::Button};
use crate::utils::{HOVERED_LAYER_OPACITY, PRESSED_LAYER_OPACITY, mix, shadow_from_level};
use crate::{Theme, fonts, tokens};

/// The Material color configuration used by a toolbar and its actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorMode {
    /// Low-emphasis toolbar color for keeping attention on page content.
    Standard,
    /// High-emphasis toolbar color for prominent or temporary page modes.
    Vibrant,
}

/// The layout direction used by a floating toolbar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

/// Creates a standard docked toolbar.
pub fn docked<'a, Message, Renderer>(
    actions: impl IntoIterator<Item = Element<'a, Message, Theme, Renderer>>,
) -> Container<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + 'a,
{
    docked_with_color(actions, ColorMode::Standard)
}

/// Creates a vibrant docked toolbar.
pub fn docked_vibrant<'a, Message, Renderer>(
    actions: impl IntoIterator<Item = Element<'a, Message, Theme, Renderer>>,
) -> Container<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + 'a,
{
    docked_with_color(actions, ColorMode::Vibrant)
}

/// Creates a docked toolbar with an explicit color configuration.
pub fn docked_with_color<'a, Message, Renderer>(
    actions: impl IntoIterator<Item = Element<'a, Message, Theme, Renderer>>,
    color: ColorMode,
) -> Container<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + 'a,
{
    let content = actions
        .into_iter()
        .fold(Row::new(), |row, action| row.push(action))
        .spacing(tokens::component::toolbar::ACTION_SPACE)
        .align_y(alignment::Vertical::Center)
        .width(Length::Fill);

    Container::new(content)
        .height(Length::Fixed(
            tokens::component::toolbar::DOCKED_CONTAINER_HEIGHT,
        ))
        .width(Length::Fill)
        .padding(Padding {
            top: 0.0,
            right: tokens::component::toolbar::DOCKED_TRAILING_SPACE,
            bottom: 0.0,
            left: tokens::component::toolbar::DOCKED_LEADING_SPACE,
        })
        .align_y(alignment::Vertical::Center)
        .style(move |theme| docked_style(theme, color))
}

/// Creates a standard horizontal floating toolbar.
pub fn floating<'a, Message, Renderer>(
    actions: impl IntoIterator<Item = Element<'a, Message, Theme, Renderer>>,
) -> Container<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + 'a,
{
    floating_with_options(actions, Orientation::Horizontal, ColorMode::Standard)
}

/// Creates a vibrant horizontal floating toolbar.
pub fn floating_vibrant<'a, Message, Renderer>(
    actions: impl IntoIterator<Item = Element<'a, Message, Theme, Renderer>>,
) -> Container<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + 'a,
{
    floating_with_options(actions, Orientation::Horizontal, ColorMode::Vibrant)
}

/// Creates a standard vertical floating toolbar.
pub fn vertical_floating<'a, Message, Renderer>(
    actions: impl IntoIterator<Item = Element<'a, Message, Theme, Renderer>>,
) -> Container<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + 'a,
{
    floating_with_options(actions, Orientation::Vertical, ColorMode::Standard)
}

/// Creates a vibrant vertical floating toolbar.
pub fn vertical_floating_vibrant<'a, Message, Renderer>(
    actions: impl IntoIterator<Item = Element<'a, Message, Theme, Renderer>>,
) -> Container<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + 'a,
{
    floating_with_options(actions, Orientation::Vertical, ColorMode::Vibrant)
}

/// Creates a floating toolbar with explicit orientation and color configuration.
pub fn floating_with_options<'a, Message, Renderer>(
    actions: impl IntoIterator<Item = Element<'a, Message, Theme, Renderer>>,
    orientation: Orientation,
    color: ColorMode,
) -> Container<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + 'a,
{
    let content: Element<'a, Message, Theme, Renderer> = match orientation {
        Orientation::Horizontal => actions
            .into_iter()
            .fold(Row::new(), |row, action| row.push(action))
            .spacing(tokens::component::toolbar::ACTION_SPACE)
            .align_y(alignment::Vertical::Center)
            .into(),
        Orientation::Vertical => actions
            .into_iter()
            .fold(Column::new(), |column, action| column.push(action))
            .spacing(tokens::component::toolbar::ACTION_SPACE)
            .align_x(alignment::Horizontal::Center)
            .into(),
    };

    let mut container = Container::new(content)
        .padding(Padding::from([
            tokens::component::toolbar::FLOATING_CONTAINER_LEADING_SPACE,
            tokens::component::toolbar::FLOATING_CONTAINER_TRAILING_SPACE,
        ]))
        .style(move |theme| floating_style(theme, color));

    match orientation {
        Orientation::Horizontal => {
            container = container
                .height(Length::Fixed(
                    tokens::component::toolbar::FLOATING_HORIZONTAL_CONTAINER_HEIGHT,
                ))
                .align_y(alignment::Vertical::Center);
        }
        Orientation::Vertical => {
            container = container
                .width(Length::Fixed(
                    tokens::component::toolbar::FLOATING_VERTICAL_CONTAINER_WIDTH,
                ))
                .align_x(alignment::Horizontal::Center);
        }
    }

    container
}

/// Places a floating toolbar next to a floating action button.
pub fn floating_with_fab<'a, Message, Renderer>(
    toolbar: impl Into<Element<'a, Message, Theme, Renderer>>,
    fab: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Row<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + 'a,
{
    Row::new()
        .push(toolbar)
        .push(fab)
        .spacing(tokens::component::toolbar::FAB_SPACE)
        .align_y(alignment::Vertical::Center)
}

/// Places a vertical floating toolbar above a floating action button.
pub fn vertical_floating_with_fab<'a, Message, Renderer>(
    toolbar: impl Into<Element<'a, Message, Theme, Renderer>>,
    fab: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Column<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + 'a,
{
    Column::new()
        .push(toolbar)
        .push(fab)
        .spacing(tokens::component::toolbar::FAB_SPACE)
        .align_x(alignment::Horizontal::Center)
}

/// Creates a toolbar icon button using the standard color configuration.
pub fn icon_button<'a, Message, Renderer>(
    icon_name: impl text::IntoFragment<'a>,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    icon_button_with_color(icon_name, ColorMode::Standard, false)
}

/// Creates a selected toolbar icon button using the standard color configuration.
pub fn selected_icon_button<'a, Message, Renderer>(
    icon_name: impl text::IntoFragment<'a>,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    icon_button_with_color(icon_name, ColorMode::Standard, true)
}

/// Creates a toolbar icon button using the vibrant color configuration.
pub fn vibrant_icon_button<'a, Message, Renderer>(
    icon_name: impl text::IntoFragment<'a>,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    icon_button_with_color(icon_name, ColorMode::Vibrant, false)
}

/// Creates a selected toolbar icon button using the vibrant color configuration.
pub fn selected_vibrant_icon_button<'a, Message, Renderer>(
    icon_name: impl text::IntoFragment<'a>,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    icon_button_with_color(icon_name, ColorMode::Vibrant, true)
}

/// Creates a toolbar icon action using the standard color configuration.
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

/// Creates a selected toolbar icon action using the standard color configuration.
pub fn selected_icon_action<'a, Message, Renderer>(
    icon_name: impl text::IntoFragment<'a>,
    on_press: Message,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    selected_icon_button(icon_name).on_press(on_press).into()
}

/// Creates a toolbar icon action using the vibrant color configuration.
pub fn vibrant_icon_action<'a, Message, Renderer>(
    icon_name: impl text::IntoFragment<'a>,
    on_press: Message,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    vibrant_icon_button(icon_name).on_press(on_press).into()
}

/// Creates a selected toolbar icon action using the vibrant color configuration.
pub fn selected_vibrant_icon_action<'a, Message, Renderer>(
    icon_name: impl text::IntoFragment<'a>,
    on_press: Message,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    selected_vibrant_icon_button(icon_name)
        .on_press(on_press)
        .into()
}

/// Creates standard toolbar icon actions.
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

/// Creates vibrant toolbar icon actions.
pub fn vibrant_icon_actions<'a, Message, Renderer, Icon>(
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
        .map(|(icon_name, on_press)| vibrant_icon_action(icon_name, on_press))
        .collect()
}

fn icon_button_with_color<'a, Message, Renderer>(
    icon_name: impl text::IntoFragment<'a>,
    color: ColorMode,
    selected: bool,
) -> Button<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: geometry::Renderer + core_text::Renderer + 'a,
    iced_widget::core::Font: Into<Renderer::Font>,
{
    Button::new(
        Container::new(
            fonts::icon(icon_name, tokens::component::toolbar::ACTION_ICON_SIZE).line_height(
                absolute_line_height(tokens::component::toolbar::ACTION_ICON_SIZE),
            ),
        )
        .center_x(Length::Fixed(
            tokens::component::toolbar::ACTION_CONTAINER_WIDTH,
        ))
        .center_y(Length::Fixed(
            tokens::component::toolbar::ACTION_CONTAINER_HEIGHT,
        )),
    )
    .width(Length::Fixed(
        tokens::component::toolbar::ACTION_CONTAINER_WIDTH,
    ))
    .height(Length::Fixed(
        tokens::component::toolbar::ACTION_CONTAINER_HEIGHT,
    ))
    .padding(Padding::ZERO)
    .style(move |theme, status| action_style(theme, status, color, selected))
}

fn docked_style(theme: &Theme, color: ColorMode) -> iced_widget::container::Style {
    let colors = theme.colors();

    iced_widget::container::Style {
        background: Some(Background::Color(container_color(theme, color))),
        text_color: Some(action_icon_color(theme, color, false)),
        border: border::rounded(tokens::component::toolbar::DOCKED_CONTAINER_SHAPE),
        shadow: shadow_from_level(0, colors.shadow),
        snap: cfg!(feature = "crisp"),
    }
}

fn floating_style(theme: &Theme, color: ColorMode) -> iced_widget::container::Style {
    let colors = theme.colors();

    iced_widget::container::Style {
        background: Some(Background::Color(container_color(theme, color))),
        text_color: Some(action_icon_color(theme, color, false)),
        border: border::rounded(tokens::component::toolbar::FLOATING_CONTAINER_SHAPE),
        shadow: shadow_from_level(
            tokens::component::toolbar::FLOATING_CONTAINER_ELEVATION_LEVEL,
            colors.shadow,
        ),
        snap: cfg!(feature = "crisp"),
    }
}

fn action_style(theme: &Theme, status: Status, color: ColorMode, selected: bool) -> Style {
    let colors = theme.colors();
    let container = action_container_color(theme, color, selected);
    let foreground = action_icon_color(theme, color, selected);
    let state_layer = action_state_layer_color(theme, color, selected);
    let shape = if selected {
        tokens::component::toolbar::ACTION_SELECTED_CONTAINER_SHAPE
    } else {
        tokens::component::toolbar::ACTION_CONTAINER_SHAPE
    };

    let active = Style {
        background: Some(Background::Color(container)),
        text_color: foreground,
        border: border::rounded(shape),
        shadow: shadow_from_level(0, Color::TRANSPARENT),
        snap: cfg!(feature = "crisp"),
    };

    match status {
        Status::Active => active,
        Status::Hovered => Style {
            background: Some(Background::Color(mix(
                container,
                state_layer,
                HOVERED_LAYER_OPACITY,
            ))),
            ..active
        },
        Status::Pressed => Style {
            background: Some(Background::Color(mix(
                container,
                state_layer,
                PRESSED_LAYER_OPACITY,
            ))),
            ..active
        },
        Status::Disabled => Style {
            background: Some(Background::Color(container)),
            text_color: Color {
                a: tokens::component::toolbar::DISABLED_ICON_OPACITY,
                ..colors.surface.text
            },
            border: border::rounded(shape),
            shadow: shadow_from_level(0, Color::TRANSPARENT),
            snap: cfg!(feature = "crisp"),
        },
    }
}

fn container_color(theme: &Theme, color: ColorMode) -> Color {
    let colors = theme.colors();

    match color {
        ColorMode::Standard => colors.surface.container.base,
        ColorMode::Vibrant => colors.primary.container,
    }
}

fn action_container_color(theme: &Theme, color: ColorMode, selected: bool) -> Color {
    let colors = theme.colors();

    match (color, selected) {
        (ColorMode::Standard, false) => colors.surface.container.base,
        (ColorMode::Standard, true) => colors.secondary.container,
        (ColorMode::Vibrant, false) => colors.primary.container,
        (ColorMode::Vibrant, true) => colors.surface.container.base,
    }
}

fn action_icon_color(theme: &Theme, color: ColorMode, selected: bool) -> Color {
    let colors = theme.colors();

    match (color, selected) {
        (ColorMode::Standard, false) => colors.surface.text_variant,
        (ColorMode::Standard, true) => colors.secondary.container_text,
        (ColorMode::Vibrant, false) => colors.primary.container_text,
        (ColorMode::Vibrant, true) => colors.surface.text,
    }
}

fn action_state_layer_color(theme: &Theme, color: ColorMode, selected: bool) -> Color {
    action_icon_color(theme, color, selected)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toolbar_styles_use_m3_color_roles() {
        let theme = Theme::Light;
        let colors = theme.colors();

        let standard = floating_style(&theme, ColorMode::Standard);
        assert_eq!(
            standard.background,
            Some(Background::Color(colors.surface.container.base))
        );
        assert_eq!(standard.shadow.offset.y, 4.0);

        let vibrant = docked_style(&theme, ColorMode::Vibrant);
        assert_eq!(
            vibrant.background,
            Some(Background::Color(colors.primary.container))
        );
    }

    #[test]
    fn toolbar_action_styles_follow_standard_and_vibrant_selectors() {
        let theme = Theme::Light;
        let colors = theme.colors();

        let standard = action_style(&theme, Status::Active, ColorMode::Standard, false);
        assert_eq!(
            standard.background,
            Some(Background::Color(colors.surface.container.base))
        );
        assert_eq!(standard.text_color, colors.surface.text_variant);

        let selected_standard = action_style(&theme, Status::Active, ColorMode::Standard, true);
        assert_eq!(
            selected_standard.background,
            Some(Background::Color(colors.secondary.container))
        );
        assert_eq!(
            selected_standard.text_color,
            colors.secondary.container_text
        );

        let selected_vibrant = action_style(&theme, Status::Active, ColorMode::Vibrant, true);
        assert_eq!(
            selected_vibrant.background,
            Some(Background::Color(colors.surface.container.base))
        );
        assert_eq!(selected_vibrant.text_color, colors.surface.text);
    }
}
