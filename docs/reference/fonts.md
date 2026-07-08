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

## Helpers

- `roboto_for_type_scale`
- `noto_sans_cjk_sc_for_type_scale`
- `font_for_content_type_scale`
- `contains_cjk`
- `icon`
- `filled_icon`

Use `icon` and `filled_icon` for Material Symbols text widgets, and prefer the
button/app-bar constructors when an icon is part of a Material control.
