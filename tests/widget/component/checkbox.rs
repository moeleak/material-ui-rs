use super::*;

#[test]
fn checkbox_alpha_scales_all_visual_channels() {
    let theme = Theme::Light;
    let status = iced_checkbox::Status::Active { is_checked: true };
    let base = checkbox_style::default(&theme, status);
    let faded = checkbox_style_alpha(base, 0.5);

    assert_eq!(faded.background, base.background.scale_alpha(0.5));
    assert_eq!(faded.icon_color, alpha_color(base.icon_color, 0.5));
    assert_eq!(faded.border, alpha_border(base.border, 0.5));
    assert_eq!(
        faded.text_color,
        base.text_color.map(|color| alpha_color(color, 0.5))
    );
}

#[test]
fn checkbox_alpha_clamps_to_valid_opacity() {
    let transparent: Checkbox<'_, (), iced_widget::Renderer> = Checkbox::new(false).alpha(-1.0);
    let opaque: Checkbox<'_, (), iced_widget::Renderer> = Checkbox::new(false).alpha(2.0);

    assert_eq!(transparent.content_alpha, 0.0);
    assert_eq!(opaque.content_alpha, 1.0);
}
