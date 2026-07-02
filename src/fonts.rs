//! Bundled Material typefaces and icon fonts.

use iced_widget::core::font::{Family, Stretch, Style, Weight};
use iced_widget::core::text as core_text;
use iced_widget::core::Font;
use iced_widget::text::{self, LineHeight};
use iced_widget::Text;

use crate::{tokens, Theme};

use std::borrow::Cow;

pub const ROBOTO_FAMILY: &str = "Roboto";
pub const NOTO_SANS_CJK_SC_FAMILY: &str = "Noto Sans CJK SC";
pub const MATERIAL_SYMBOLS_ROUNDED_FAMILY: &str = "Material Symbols Rounded";

pub const ROBOTO_REGULAR_BYTES: &[u8] = include_bytes!("fonts/Roboto-Regular.ttf");
pub const ROBOTO_MEDIUM_BYTES: &[u8] = include_bytes!("fonts/Roboto-Medium.ttf");
pub const ROBOTO_BOLD_BYTES: &[u8] = include_bytes!("fonts/Roboto-Bold.ttf");
pub const NOTO_SANS_CJK_SC_REGULAR_BYTES: &[u8] = include_bytes!("fonts/NotoSansCJKsc-Regular.otf");
pub const NOTO_SANS_CJK_SC_MEDIUM_BYTES: &[u8] = include_bytes!("fonts/NotoSansCJKsc-Medium.otf");
pub const NOTO_SANS_CJK_SC_BOLD_BYTES: &[u8] = include_bytes!("fonts/NotoSansCJKsc-Bold.otf");
pub const MATERIAL_SYMBOLS_ROUNDED_BYTES: &[u8] =
    include_bytes!("fonts/MaterialSymbolsRounded-Regular.ttf");

pub const ROBOTO: Font = roboto_for_weight(tokens::typography::WEIGHT_REGULAR);
pub const ROBOTO_MEDIUM: Font = roboto_for_weight(tokens::typography::WEIGHT_MEDIUM);
pub const ROBOTO_BOLD: Font = roboto_for_weight(tokens::typography::WEIGHT_BOLD);
pub const NOTO_SANS_CJK_SC: Font = noto_sans_cjk_sc_for_weight(tokens::typography::WEIGHT_REGULAR);
pub const NOTO_SANS_CJK_SC_MEDIUM: Font =
    noto_sans_cjk_sc_for_weight(tokens::typography::WEIGHT_MEDIUM);
pub const NOTO_SANS_CJK_SC_BOLD: Font =
    noto_sans_cjk_sc_for_weight(tokens::typography::WEIGHT_BOLD);
pub const MATERIAL_SYMBOLS_ROUNDED: Font = Font {
    family: Family::Name(MATERIAL_SYMBOLS_ROUNDED_FAMILY),
    weight: Weight::Normal,
    stretch: Stretch::Normal,
    style: Style::Normal,
};

pub fn all() -> [Cow<'static, [u8]>; 7] {
    [
        Cow::Borrowed(ROBOTO_REGULAR_BYTES),
        Cow::Borrowed(ROBOTO_MEDIUM_BYTES),
        Cow::Borrowed(ROBOTO_BOLD_BYTES),
        Cow::Borrowed(NOTO_SANS_CJK_SC_REGULAR_BYTES),
        Cow::Borrowed(NOTO_SANS_CJK_SC_MEDIUM_BYTES),
        Cow::Borrowed(NOTO_SANS_CJK_SC_BOLD_BYTES),
        Cow::Borrowed(MATERIAL_SYMBOLS_ROUNDED_BYTES),
    ]
}

pub const fn roboto_for_type_scale(scale: tokens::typography::TypeScale) -> Font {
    roboto_for_weight(scale.weight)
}

pub const fn noto_sans_cjk_sc_for_type_scale(scale: tokens::typography::TypeScale) -> Font {
    noto_sans_cjk_sc_for_weight(scale.weight)
}

pub fn font_for_content_type_scale(content: &str, scale: tokens::typography::TypeScale) -> Font {
    if contains_cjk(content) {
        noto_sans_cjk_sc_for_type_scale(scale)
    } else {
        roboto_for_type_scale(scale)
    }
}

pub const fn roboto_for_weight(weight: u16) -> Font {
    Font {
        family: Family::Name(ROBOTO_FAMILY),
        weight: match weight {
            tokens::typography::WEIGHT_BOLD => Weight::Bold,
            tokens::typography::WEIGHT_MEDIUM => Weight::Medium,
            _ => Weight::Normal,
        },
        stretch: Stretch::Normal,
        style: Style::Normal,
    }
}

pub const fn noto_sans_cjk_sc_for_weight(weight: u16) -> Font {
    Font {
        family: Family::Name(NOTO_SANS_CJK_SC_FAMILY),
        weight: match weight {
            tokens::typography::WEIGHT_BOLD => Weight::Bold,
            tokens::typography::WEIGHT_MEDIUM => Weight::Medium,
            _ => Weight::Normal,
        },
        stretch: Stretch::Normal,
        style: Style::Normal,
    }
}

pub fn contains_cjk(content: &str) -> bool {
    content.chars().any(is_cjk_codepoint)
}

pub fn material_symbol_codepoint(name: &str) -> Option<char> {
    let codepoint = match name {
        "info" => 0xe88e,
        "input" => 0xe890,
        "layers" => 0xe53b,
        "menu" => 0xe5d2,
        "navigation" => 0xe55d,
        "tune" => 0xe429,
        _ => return None,
    };

    char::from_u32(codepoint)
}

fn material_symbol_fragment<'a>(name: impl text::IntoFragment<'a>) -> text::Fragment<'a> {
    let fragment = name.into_fragment();

    material_symbol_codepoint(fragment.as_ref())
        .map(|codepoint| text::Fragment::Owned(codepoint.to_string()))
        .unwrap_or(fragment)
}

fn is_cjk_codepoint(character: char) -> bool {
    matches!(
        character,
        '\u{2E80}'..='\u{2EFF}'
            | '\u{3000}'..='\u{303F}'
            | '\u{3040}'..='\u{30FF}'
            | '\u{3100}'..='\u{312F}'
            | '\u{31A0}'..='\u{31BF}'
            | '\u{31F0}'..='\u{31FF}'
            | '\u{3400}'..='\u{4DBF}'
            | '\u{4E00}'..='\u{9FFF}'
            | '\u{AC00}'..='\u{D7AF}'
            | '\u{F900}'..='\u{FAFF}'
            | '\u{20000}'..='\u{2A6DF}'
            | '\u{2A700}'..='\u{2B73F}'
            | '\u{2B740}'..='\u{2B81F}'
            | '\u{2B820}'..='\u{2CEAF}'
            | '\u{2CEB0}'..='\u{2EBEF}'
            | '\u{30000}'..='\u{323AF}'
    )
}

pub fn icon<'a, Renderer>(name: impl text::IntoFragment<'a>, size: f32) -> Text<'a, Theme, Renderer>
where
    Renderer: core_text::Renderer,
    Font: Into<Renderer::Font>,
{
    Text::new(material_symbol_fragment(name))
        .font(MATERIAL_SYMBOLS_ROUNDED)
        .size(size)
        .line_height(LineHeight::Absolute(size.into()))
        .shaping(text::Shaping::Advanced)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn is_font_asset(bytes: &[u8]) -> bool {
        bytes.starts_with(&[0x00, 0x01, 0x00, 0x00]) || bytes.starts_with(b"OTTO")
    }

    #[test]
    fn bundled_fonts_are_font_assets() {
        assert!(is_font_asset(ROBOTO_REGULAR_BYTES));
        assert!(is_font_asset(ROBOTO_MEDIUM_BYTES));
        assert!(is_font_asset(ROBOTO_BOLD_BYTES));
        assert!(is_font_asset(NOTO_SANS_CJK_SC_REGULAR_BYTES));
        assert!(is_font_asset(NOTO_SANS_CJK_SC_MEDIUM_BYTES));
        assert!(is_font_asset(NOTO_SANS_CJK_SC_BOLD_BYTES));
        assert!(is_font_asset(MATERIAL_SYMBOLS_ROUNDED_BYTES));
        assert_eq!(all().len(), 7);
    }

    #[test]
    fn material_fonts_expose_expected_families_and_weights() {
        assert_eq!(ROBOTO.family, Family::Name(ROBOTO_FAMILY));
        assert_eq!(ROBOTO.weight, Weight::Normal);
        assert_eq!(ROBOTO_MEDIUM.weight, Weight::Medium);
        assert_eq!(ROBOTO_BOLD.weight, Weight::Bold);
        assert_eq!(
            NOTO_SANS_CJK_SC.family,
            Family::Name(NOTO_SANS_CJK_SC_FAMILY)
        );
        assert_eq!(NOTO_SANS_CJK_SC.weight, Weight::Normal);
        assert_eq!(NOTO_SANS_CJK_SC_MEDIUM.weight, Weight::Medium);
        assert_eq!(NOTO_SANS_CJK_SC_BOLD.weight, Weight::Bold);
        assert_eq!(
            MATERIAL_SYMBOLS_ROUNDED.family,
            Family::Name(MATERIAL_SYMBOLS_ROUNDED_FAMILY)
        );
    }

    #[test]
    fn material_symbol_names_resolve_to_google_codepoints() {
        assert_eq!(material_symbol_codepoint("input"), Some('\u{e890}'));
        assert_eq!(material_symbol_codepoint("tune"), Some('\u{e429}'));
        assert_eq!(material_symbol_codepoint("info"), Some('\u{e88e}'));
        assert_eq!(material_symbol_codepoint("layers"), Some('\u{e53b}'));
        assert_eq!(material_symbol_codepoint("navigation"), Some('\u{e55d}'));
        assert_eq!(material_symbol_codepoint("menu"), Some('\u{e5d2}'));
        assert_eq!(material_symbol_codepoint("unknown_symbol"), None);
    }

    #[test]
    fn material_symbol_fragment_falls_back_to_ligature_text_for_unknown_names() {
        assert_eq!(material_symbol_fragment("input").as_ref(), "\u{e890}");
        assert_eq!(
            material_symbol_fragment("unknown_symbol").as_ref(),
            "unknown_symbol"
        );
    }

    #[test]
    fn type_scale_weights_select_roboto_faces() {
        assert_eq!(
            roboto_for_type_scale(tokens::typography::BODY_LARGE),
            ROBOTO
        );
        assert_eq!(
            roboto_for_type_scale(tokens::typography::LABEL_LARGE),
            ROBOTO_MEDIUM
        );
        assert_eq!(
            roboto_for_weight(tokens::typography::WEIGHT_BOLD),
            ROBOTO_BOLD
        );
    }

    #[test]
    fn cjk_content_selects_noto_sans_cjk_sc() {
        assert!(!contains_cjk("Material 3 typography"));
        assert!(contains_cjk("中文字体"));
        assert!(contains_cjk("かな"));
        assert!(contains_cjk("한글"));
        assert_eq!(
            font_for_content_type_scale("中文字体", tokens::typography::BODY_LARGE),
            NOTO_SANS_CJK_SC
        );
        assert_eq!(
            font_for_content_type_scale("English", tokens::typography::LABEL_LARGE),
            ROBOTO_MEDIUM
        );
    }
}
