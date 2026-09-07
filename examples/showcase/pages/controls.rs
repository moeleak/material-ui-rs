use material::widget::page;
use material_ui_rs as material;

use super::super::{Message, RadioChoice, SegmentChoice, Showcase};

pub(super) fn view(state: &Showcase) -> material::Element<'_, Message> {
    page::sections([
        page::section("Counter", counter_controls(state)).into(),
        page::section("Actions", action_buttons(state)).into(),
        page::section("FABs", fabs(state)).into(),
        page::section("Chips", chips(state)).into(),
        page::section("Segmented buttons", segmented_buttons(state)).into(),
        page::section("Selection controls", selection_controls(state)).into(),
    ])
    .into()
}

fn counter_controls(state: &Showcase) -> material::Element<'_, Message> {
    use material::widget::button::{self, ButtonVariant};

    page::row([
        button::action(
            button::button("Minus", ButtonVariant::Outlined),
            Message::Decrement,
        ),
        material::text::headline_medium(state.count.to_string()).into(),
        button::action(
            button::button("Plus", ButtonVariant::Filled),
            Message::Increment,
        ),
    ])
    .wrap()
    .into()
}

fn action_buttons(state: &Showcase) -> material::Element<'_, Message> {
    use material::widget::button::{self, ButtonVariant};

    page::row(button::enabled_actions(
        state.enabled,
        Message::Increment,
        [
            button::button("Filled", ButtonVariant::Filled),
            button::button("Tonal", ButtonVariant::FilledTonal),
            button::button("Text", ButtonVariant::Text),
        ],
    ))
    .wrap()
    .into()
}

fn fabs(state: &Showcase) -> material::Element<'_, Message> {
    use material::widget::button::{self, FabSize, FabVariant};

    page::stack([
        page::row(button::enabled_actions(
            state.enabled,
            Message::Increment,
            [
                button::fab("add", FabVariant::Surface, FabSize::Small),
                button::fab("add", FabVariant::Surface, FabSize::Standard),
                button::fab("add", FabVariant::Surface, FabSize::Large),
                button::fab("add", FabVariant::Primary, FabSize::Standard),
                button::fab("add", FabVariant::Secondary, FabSize::Standard),
                button::fab("add", FabVariant::Tertiary, FabSize::Standard),
            ],
        ))
        .wrap()
        .into(),
        page::row(button::enabled_actions(
            state.enabled,
            Message::Increment,
            [
                button::extended_fab_with_icon("add", "Create", FabVariant::Primary),
                button::extended_fab_with_icon("share", "Share", FabVariant::Secondary),
                button::extended_fab_with_icon("add", "Add", FabVariant::Tertiary),
                button::extended_fab("Reroute", FabVariant::Surface),
            ],
        ))
        .wrap()
        .into(),
    ])
    .into()
}

fn chips(state: &Showcase) -> material::Element<'_, Message> {
    use material::widget::button::{self, ChipVariant};

    page::compact_row(button::enabled_actions(
        state.enabled,
        Message::Increment,
        [
            button::chip("Assist", ChipVariant::Assist),
            button::chip("Suggestion", ChipVariant::Suggestion),
            button::chip("Filter", ChipVariant::Filter),
            button::chip("Selected", ChipVariant::SelectedFilter),
        ],
    ))
    .wrap()
    .into()
}

fn segmented_buttons(state: &Showcase) -> material::Element<'_, Message> {
    use material::widget::segmented_button;

    segmented_button::group(segmented_button::animated_label_actions(
        &state.segment_state,
        [
            ("List", Message::SegmentSelected(SegmentChoice::List)),
            ("Grid", Message::SegmentSelected(SegmentChoice::Grid)),
            ("Map", Message::SegmentSelected(SegmentChoice::Map)),
        ],
    ))
    .into()
}

fn selection_controls(state: &Showcase) -> material::Element<'_, Message> {
    let switches = page::component_stack([
        material::widget::checkbox::standard(
            state.enabled,
            "Enable actions",
            Message::EnabledChanged,
        ),
        state
            .theme_controller
            .dark_mode_switch("Dark theme", Message::ThemeChanged),
    ]);

    let radios = page::row([
        material::widget::radio::standard(
            "Standard",
            RadioChoice::Standard,
            state.radio_choice,
            Message::ChoiceSelected,
        ),
        material::widget::radio::standard(
            "Expressive",
            RadioChoice::Expressive,
            state.radio_choice,
            Message::ChoiceSelected,
        ),
        material::widget::radio::standard(
            "Dense",
            RadioChoice::Dense,
            state.radio_choice,
            Message::ChoiceSelected,
        ),
    ])
    .wrap();

    page::spacious_stack([switches.into(), radios.into()]).into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced_widget::core::{
        Event, Layout, Pixels, Rectangle, Shell, Size, layout, mouse, touch, widget::Tree,
    };

    #[test]
    fn fab_and_chip_previews_wrap_without_shrinking_or_losing_click_targets() {
        let renderer = iced_widget::Renderer::Secondary(iced_tiny_skia::Renderer::new(
            material::fonts::ROBOTO,
            Pixels(16.0),
        ));
        for enabled in [true, false] {
            let state = Showcase {
                enabled,
                ..Showcase::default()
            };
            for is_fab in [true, false] {
                let mut content = if is_fab { fabs(&state) } else { chips(&state) };
                let mut tree = Tree::new(content.as_widget());
                let mut natural_sizes = Vec::new();
                let mut wide_height = 0.0;
                // 320/360 dp phones leave 264/304 dp after page padding.
                // Resize back to desktop too, using the same widget state.
                for width in [720.0, 304.0, 264.0, 720.0] {
                    let viewport = Rectangle::with_size(Size::new(width, 1000.0));
                    let node = content.as_widget_mut().layout(
                        &mut tree,
                        &renderer,
                        &layout::Limits::new(Size::ZERO, viewport.size()),
                    );
                    let layout = Layout::new(&node);
                    let buttons: Vec<_> = if is_fab {
                        layout
                            .children()
                            .flat_map(|row| row.children().map(|button| button.bounds()))
                            .collect()
                    } else {
                        layout.children().map(|button| button.bounds()).collect()
                    };
                    assert_eq!(buttons.len(), if is_fab { 10 } else { 4 });
                    let sizes: Vec<_> = buttons.iter().map(|bounds| bounds.size()).collect();
                    if natural_sizes.is_empty() {
                        natural_sizes = sizes.clone();
                        wide_height = node.size().height;
                    }
                    assert_eq!(sizes, natural_sizes, "a preview shrank at width {width}");
                    if width < 720.0 {
                        assert!(node.size().height > wide_height, "previews did not wrap");
                    } else {
                        assert_eq!(node.size().height, wide_height);
                    }
                    for (index, bounds) in buttons.iter().enumerate() {
                        assert!(bounds.x >= 0.0 && bounds.x + bounds.width <= width);
                        assert!(bounds.y >= 0.0 && bounds.y + bounds.height <= node.size().height);
                        for other in &buttons[index + 1..] {
                            assert!(bounds.intersection(other).is_none(), "previews overlap");
                        }
                    }

                    let mut messages = Vec::new();
                    for button in &buttons {
                        let position = button.center();
                        let id = touch::Finger(1);
                        for event in [
                            touch::Event::FingerPressed { id, position },
                            touch::Event::FingerLifted { id, position },
                        ] {
                            content.as_widget_mut().update(
                                &mut tree,
                                &Event::Touch(event),
                                layout,
                                mouse::Cursor::Available(position),
                                &renderer,
                                &mut iced_widget::core::clipboard::Null,
                                &mut Shell::new(&mut messages),
                                &viewport,
                            );
                        }
                    }
                    assert_eq!(messages.len(), if enabled { buttons.len() } else { 0 });
                    assert!(
                        messages
                            .iter()
                            .all(|message| matches!(message, Message::Increment))
                    );
                }
            }
        }
    }
}
