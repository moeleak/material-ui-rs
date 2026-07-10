//! Bundled Material typefaces and icon fonts.

use iced_widget::Text;
use iced_widget::core::Font;
use iced_widget::core::font::{Family, Stretch, Style, Weight};
use iced_widget::core::text as core_text;
use iced_widget::text::{self, LineHeight};

use crate::{Theme, tokens};

use std::borrow::Cow;
use std::fmt;

#[cfg(target_arch = "wasm32")]
#[path = "web_font.rs"]
mod web_font;

pub const ROBOTO_FAMILY: &str = "Roboto";
pub const NOTO_SANS_CJK_SC_FAMILY: &str = "Noto Sans CJK SC";
pub const MATERIAL_SYMBOLS_ROUNDED_FAMILY: &str = "Material Symbols Rounded";
pub const MATERIAL_SYMBOLS_ROUNDED_FILLED_FAMILY: &str = "Material Symbols Rounded Filled";

pub const ROBOTO_REGULAR_BYTES: &[u8] = include_bytes!("../fonts/Roboto-Regular.ttf");
pub const ROBOTO_MEDIUM_BYTES: &[u8] = include_bytes!("../fonts/Roboto-Medium.ttf");
pub const ROBOTO_BOLD_BYTES: &[u8] = include_bytes!("../fonts/Roboto-Bold.ttf");
pub const MATERIAL_SYMBOLS_ROUNDED_BYTES: &[u8] =
    include_bytes!("../fonts/MaterialSymbolsRounded-Regular.ttf");
pub const MATERIAL_SYMBOLS_ROUNDED_FILLED_BYTES: &[u8] =
    include_bytes!("../fonts/MaterialSymbolsRounded-Filled.ttf");

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
pub const MATERIAL_SYMBOLS_ROUNDED_FILLED: Font = Font {
    family: Family::Name(MATERIAL_SYMBOLS_ROUNDED_FILLED_FAMILY),
    weight: Weight::Normal,
    stretch: Stretch::Normal,
    style: Style::Normal,
};

/// An error produced while loading a web font from a URL.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebFontError {
    /// URL loading is only available in WebAssembly builds.
    UnsupportedPlatform,
    /// A browser API needed to fetch the font was not available.
    MissingBrowserApi(&'static str),
    /// The browser rejected the fetch request.
    RequestFailed,
    /// The server returned an unsuccessful HTTP status.
    HttpStatus(u16),
    /// The response body could not be read.
    ReadFailed,
    /// The response was not a TrueType, OpenType, or TrueType Collection font.
    UnsupportedFormat,
    /// The renderer could not load the downloaded font.
    FontLoad(iced::font::Error),
}

impl fmt::Display for WebFontError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedPlatform => {
                formatter.write_str("web fonts can only be fetched on WebAssembly")
            }
            Self::MissingBrowserApi(api) => write!(formatter, "browser API `{api}` is unavailable"),
            Self::RequestFailed => formatter.write_str("the browser rejected the font request"),
            Self::HttpStatus(status) => {
                write!(formatter, "the font server returned HTTP status {status}")
            }
            Self::ReadFailed => formatter.write_str("the font response body could not be read"),
            Self::UnsupportedFormat => formatter.write_str(
                "the downloaded file is not a TrueType, OpenType, or TrueType Collection font",
            ),
            Self::FontLoad(_) => formatter.write_str("the renderer could not load the web font"),
        }
    }
}

impl std::error::Error for WebFontError {}

pub fn all() -> [Cow<'static, [u8]>; 5] {
    [
        Cow::Borrowed(ROBOTO_REGULAR_BYTES),
        Cow::Borrowed(ROBOTO_MEDIUM_BYTES),
        Cow::Borrowed(ROBOTO_BOLD_BYTES),
        Cow::Borrowed(MATERIAL_SYMBOLS_ROUNDED_BYTES),
        Cow::Borrowed(MATERIAL_SYMBOLS_ROUNDED_FILLED_BYTES),
    ]
}

/// Downloads and loads a font without embedding its bytes in the WASM binary.
///
/// The returned task starts the request when it is returned from application
/// boot or update. The URL must serve a TrueType (`.ttf`), OpenType (`.otf`),
/// or TrueType Collection (`.ttc`) file and must permit a browser CORS request.
/// Web-only font formats such as WOFF2 are not accepted by the iced renderer.
///
/// On non-WASM targets, the task resolves to
/// [`WebFontError::UnsupportedPlatform`].
///
/// ```no_run
/// # use material_ui_rs::fonts;
/// # #[derive(Debug, Clone)]
/// # enum Message { FontLoaded(Result<(), fonts::WebFontError>) }
/// let task = fonts::load_web_font("fonts/NotoSansCJKsc-Regular.otf")
///     .map(Message::FontLoaded);
/// # let _ = task;
/// ```
pub fn load_web_font(url: impl Into<String>) -> iced::Task<Result<(), WebFontError>> {
    #[cfg(target_arch = "wasm32")]
    {
        web_font::load(url.into())
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = url.into();

        iced::Task::done(Err(WebFontError::UnsupportedPlatform))
    }
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

#[cfg(any(target_arch = "wasm32", test))]
fn is_supported_web_font(bytes: &[u8]) -> bool {
    bytes.starts_with(&[0x00, 0x01, 0x00, 0x00])
        || bytes.starts_with(b"OTTO")
        || bytes.starts_with(b"ttcf")
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

pub fn filled_icon<'a, Renderer>(
    name: impl text::IntoFragment<'a>,
    size: f32,
) -> Text<'a, Theme, Renderer>
where
    Renderer: core_text::Renderer,
    Font: Into<Renderer::Font>,
{
    Text::new(material_symbol_fragment(name))
        .font(MATERIAL_SYMBOLS_ROUNDED_FILLED)
        .size(size)
        .line_height(LineHeight::Absolute(size.into()))
        .shaping(text::Shaping::Advanced)
}

#[cfg(test)]
#[path = "../../tests/design/fonts.rs"]
mod tests;
