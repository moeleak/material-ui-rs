# Fonts

Font helpers live under `material::fonts`.

## Bundled Font Bytes

The crate bundles:

- Roboto Regular
- Roboto Medium
- Roboto Bold
- Material Symbols Rounded Regular
- Material Symbols Rounded Filled

`fonts::all()` returns the bundled byte slices for application loading.

## Font Constants

Common font constants include:

- `ROBOTO`
- `ROBOTO_MEDIUM`
- `ROBOTO_BOLD`
- `MATERIAL_SYMBOLS_ROUNDED`
- `MATERIAL_SYMBOLS_ROUNDED_FILLED`
- `NOTO_SANS_CJK_SC`
- `NOTO_SANS_CJK_SC_MEDIUM`
- `NOTO_SANS_CJK_SC_BOLD`

The Noto constants name the family only. Applications that use them must load
that font family themselves.

## Runtime Web Fonts

`fonts::load_web_font(url)` returns an iced task that fetches a raw TTF, OTF, or
TTC file and loads it into the renderer at runtime. This keeps large font bytes
out of the WASM module. Map the task result to an application message and return
it from boot or update; the request starts only when that task is returned. On
native targets the task returns
`WebFontError::UnsupportedPlatform`.

## Helpers

- `roboto_for_type_scale`
- `noto_sans_cjk_sc_for_type_scale`
- `font_for_content_type_scale`
- `contains_cjk`
- `load_web_font`
- `icon`
- `filled_icon`

Use `icon` and `filled_icon` for Material Symbols text widgets, and prefer the
button/app-bar constructors when an icon is part of a Material control.
