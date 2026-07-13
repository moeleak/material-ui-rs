use iced_widget::core::time::{Duration, Instant};
use iced_widget::core::{Background, Color};

use super::*;

fn entries() -> Vec<LogEntry<u64>> {
    vec![
        LogEntry::new(10, LogLevel::Info, "[0001] first"),
        LogEntry::new(20, LogLevel::Error, "[0002] second"),
        LogEntry::new(30, LogLevel::Warn, "[0003] third"),
    ]
}

#[test]
fn levels_use_stable_uppercase_labels() {
    assert_eq!(LogLevel::Trace.label(), "TRACE");
    assert_eq!(LogLevel::Debug.label(), "DEBUG");
    assert_eq!(LogLevel::Info.label(), "INFO");
    assert_eq!(LogLevel::Warn.label(), "WARN");
    assert_eq!(LogLevel::Error.label(), "ERROR");
}

#[test]
fn entry_preserves_source_format_after_level() {
    let entry = LogEntry::new(1, LogLevel::Info, "[0005] connected");

    assert_eq!(entry.id(), &1);
    assert_eq!(entry.level(), LogLevel::Info);
    assert_eq!(entry.message(), "[0005] connected");
    assert_eq!(entry.line(), "INFO[0005] connected");
}

#[test]
fn log_text_uses_a_bundled_font_available_on_wasm() {
    assert_eq!(
        log_text_font(),
        fonts::roboto_for_type_scale(tokens::component::log_viewer::LOG_TEXT)
    );
    assert_ne!(log_text_font(), Font::MONOSPACE);
}

#[test]
fn log_list_opens_at_the_top() {
    assert_eq!(LOG_SCROLL_ANCHOR, scrollable::Anchor::Start);
}

#[test]
fn selection_toggles_and_closes_cleanly() {
    let logs = entries();
    let mut state = State::new();

    assert!(state.toggle(20));
    assert!(state.toggle(10));
    assert_eq!(state.selected_ids(), &[20, 10]);
    assert_eq!(state.selected_count(&logs), 2);

    assert!(!state.toggle(20));
    assert_eq!(state.selected_ids(), &[10]);

    let _: iced::Task<()> = state.update(Action::CloseSelection, &logs);
    assert!(state.selected_ids().is_empty());
    assert_eq!(state.selected_count(&logs), 0);
}

#[test]
fn copied_text_uses_visible_order_instead_of_selection_order() {
    let logs = entries();
    let mut state = State::new();

    let _ = state.toggle(30);
    let _ = state.toggle(10);

    assert_eq!(
        state.selected_text(&logs),
        "INFO[0001] first\nWARN[0003] third"
    );
}

#[test]
fn removed_entries_do_not_count_or_copy_and_can_be_pruned() {
    let logs = entries();
    let remaining = vec![logs[0].clone(), logs[2].clone()];
    let mut state = State::new();

    let _ = state.toggle(20);
    let _ = state.toggle(30);

    assert_eq!(state.selected_count(&remaining), 1);
    assert_eq!(state.selected_text(&remaining), "WARN[0003] third");

    state.retain_entries(&remaining);
    assert_eq!(state.selected_ids(), &[30]);
}

#[test]
fn selection_bar_animates_without_changing_selection_state() {
    let start = Instant::now();
    let mut state = State::new();

    assert!(state.toggle_at(10, start));
    assert!(state.is_animating());
    assert_eq!(state.selection_bar_progress(), 0.0);
    assert_eq!(state.selected_ids(), &[10]);

    assert!(state.advance(start + Duration::from_millis(125)));
    assert!(state.selection_bar_progress() > 0.0);
    assert!(state.selection_bar_progress() < 1.0);

    assert!(!state.advance(start + Duration::from_secs(2)));
    assert_eq!(state.selection_bar_progress(), 1.0);

    state.clear_selection_at(start + Duration::from_secs(2));
    assert!(state.is_animating());
    assert!(state.selected_ids().is_empty());
    assert!(state.advance(start + Duration::from_millis(2050)));
    assert!(state.selection_bar_progress() > 0.0);
    assert!(state.selection_bar_progress() < 1.0);
    assert!(!state.advance(start + Duration::from_secs(4)));
    assert_eq!(state.selection_bar_progress(), 0.0);
}

#[test]
fn selection_bar_reverses_from_its_current_progress() {
    let start = Instant::now();
    let mut state = State::new();

    assert!(state.toggle_at(10, start));
    assert!(state.advance(start + Duration::from_millis(100)));
    let entering = state.selection_bar_progress();

    state.clear_selection_at(start + Duration::from_millis(100));
    assert_eq!(state.selection_bar_progress(), entering);
    assert!(state.advance(start + Duration::from_millis(150)));
    let exiting = state.selection_bar_progress();
    assert!(exiting < entering);

    assert!(state.toggle_at(10, start + Duration::from_millis(150)));
    assert_eq!(state.selection_bar_progress(), exiting);
    assert!(!state.advance(start + Duration::from_secs(2)));
    assert_eq!(state.selection_bar_progress(), 1.0);
}

#[test]
fn selection_bar_reveals_from_the_top_like_a_selection_field() {
    let frame = |reveal| RevealFrame {
        reveal,
        alpha: reveal,
        is_closing: false,
    };

    assert_eq!(selection_bar_visible_height(frame(-1.0)), 0.0);
    assert_eq!(
        selection_bar_visible_height(frame(0.5)),
        tokens::component::log_viewer::SELECTION_BAR_HEIGHT / 2.0
    );
    assert_eq!(
        selection_bar_visible_height(frame(2.0)),
        tokens::component::log_viewer::SELECTION_BAR_HEIGHT
    );
}

#[test]
fn selection_bar_fades_surface_without_shadow_padding() {
    let theme = Theme::Dark;
    let hidden = selection_bar_style(&theme, 0.0);
    let shown = selection_bar_style(&theme, 1.0);
    let Some(Background::Color(hidden_background)) = hidden.background else {
        panic!("selection bar should use a solid background");
    };
    let Some(Background::Color(shown_background)) = shown.background else {
        panic!("selection bar should use a solid background");
    };

    assert_eq!(hidden_background.a, 0.0);
    assert_eq!(shown_background.a, 1.0);
    assert_eq!(shown.shadow, iced_widget::core::Shadow::default());
    assert_eq!(
        tokens::component::log_viewer::SELECTION_BAR_HEIGHT,
        tokens::component::app_bar::SMALL_SEARCH_CONTAINER_HEIGHT
    );
}

#[test]
fn selected_item_style_uses_outline_without_changing_surface() {
    let theme = Theme::Light;
    let colors = theme.colors();
    let selected = item_container_style(&theme, true);
    let unselected = item_container_style(&theme, false);

    assert_eq!(
        selected.background,
        Some(Background::Color(colors.surface.container.low))
    );
    assert_eq!(
        selected.border.width,
        tokens::component::log_viewer::SELECTED_OUTLINE_WIDTH
    );
    assert_eq!(selected.border.color, colors.outline.color);
    assert_eq!(
        unselected.background,
        Some(Background::Color(colors.surface.container.low))
    );
    assert_eq!(unselected.border.width, 0.0);
}

#[test]
fn log_checkbox_preserves_checked_and_unchecked_animation_endpoints() {
    let theme = Theme::Light;
    let unchecked = checkbox_visual_style(&theme, CheckboxStatus::Disabled { is_checked: false });
    let checked = checkbox_visual_style(&theme, CheckboxStatus::Disabled { is_checked: true });

    assert_eq!(
        unchecked,
        checkbox_style::default(&theme, CheckboxStatus::Active { is_checked: false })
    );
    assert_eq!(
        checked,
        checkbox_style::default(&theme, CheckboxStatus::Active { is_checked: true })
    );
    assert_ne!(unchecked.background, checked.background);
    assert_ne!(unchecked.border, checked.border);
}

#[test]
fn row_button_uses_one_rounded_state_layer() {
    let theme = Theme::Light;
    let style = item_button_style(&theme, ButtonStatus::Hovered);

    assert_eq!(style.background, None);
    assert_eq!(style.border.width, 0.0);
    assert_eq!(
        style.border.radius,
        border::radius(tokens::component::log_viewer::ITEM_SHAPE)
    );
}

#[test]
fn levels_use_theme_semantic_colors() {
    let theme = Theme::Light;
    let colors = theme.colors();

    assert_eq!(
        level_text_style(&theme, LogLevel::Info).color,
        Some(colors.primary.color)
    );
    assert_eq!(
        level_text_style(&theme, LogLevel::Error).color,
        Some(colors.error.color)
    );
    assert_ne!(
        level_text_style(&theme, LogLevel::Trace).color,
        Some(Color::TRANSPARENT)
    );
}
