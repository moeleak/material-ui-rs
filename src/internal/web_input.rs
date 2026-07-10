#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::wasm_bindgen;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(module = "/src/internal/web_input.js")]
extern "C" {
    #[wasm_bindgen(js_name = registerTextRegion)]
    fn register_text_region_js(x: f64, y: f64, width: f64, height: f64);

    #[wasm_bindgen(js_name = showMobileKeyboard)]
    fn show_mobile_keyboard_js();

    #[wasm_bindgen(js_name = hideMobileKeyboard)]
    fn hide_mobile_keyboard_js();
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn register_text_region(bounds: iced_widget::core::Rectangle) {
    register_text_region_js(
        f64::from(bounds.x),
        f64::from(bounds.y),
        f64::from(bounds.width),
        f64::from(bounds.height),
    );
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn register_text_region(_bounds: iced_widget::core::Rectangle) {}

#[cfg(target_arch = "wasm32")]
pub(crate) fn show_mobile_keyboard() {
    show_mobile_keyboard_js();
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn show_mobile_keyboard() {}

#[cfg(target_arch = "wasm32")]
pub(crate) fn hide_mobile_keyboard() {
    hide_mobile_keyboard_js();
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn hide_mobile_keyboard() {}
