# Build a WebAssembly App with Trunk

`material-ui-rs` works with a normal Trunk host page. Mobile Android and iOS
IME support is shipped inside the crate as a wasm-bindgen JavaScript module, so
you do not need to copy a bridge script, add a hidden input, or define browser
globals. [Trunk includes wasm-bindgen JavaScript snippets automatically][trunk-js].

## Use a Minimal Host Page

For an application whose `Cargo.toml` is one directory above `web/`, create
`web/index.html`:

```html
<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <base data-trunk-public-url />
    <title>My app</title>
    <style>
      html, body { width: 100%; height: 100%; margin: 0; }
      body { overflow: hidden; }
      canvas { display: block; }
    </style>
    <link
      data-trunk
      rel="rust"
      href="../Cargo.toml"
      data-bin="my_app"
      data-wasm-opt="z"
    />
  </head>
  <body></body>
</html>
```

Omit `data-bin` when Cargo has only one unambiguous binary target. The
`data-trunk-public-url` base keeps generated assets working when the site is
deployed below a path prefix.

## Build and Serve

During development:

```sh
trunk serve web/index.html
```

For deployment:

```sh
trunk build web/index.html --release --dist dist --public-url /
python3 -m http.server 4173 --directory dist
```

Serve `dist/` over HTTP instead of opening `index.html` directly, so JavaScript
and WASM receive the correct MIME types.

For a small release binary, configure the application crate too:

```toml
[profile.release]
lto = "fat"
opt-level = "z"
codegen-units = 1
strip = true
panic = "abort"
```

## Add CJK Fonts Without Embedding Them

The mobile IME bridge and CJK fonts are separate concerns. IME input works
automatically; a CJK font should be fetched only when the content needs it.
Follow [Use bundled and CJK fonts](use-fonts.md) to load a raw font with
`fonts::load_web_font`. The downloaded font remains outside the `.wasm` file.

## Build This Repository's Showcase

The repository has multiple targets, so its host page includes showcase-only
feature flags:

```sh
trunk build web/index.html --release --dist dist --public-url /
```

Run the mobile input bridge regression tests with Node.js 24 or newer:

```sh
node --test web/mobile_ime.test.mjs
```

The minimal downstream page above should not copy the showcase-only
`__showcase_web` feature configuration.

Pages built from the previous integration guide can remove their inline mobile
IME bridge. The crate still recognizes the legacy `__icedMaterial*` hooks while
applications migrate, so upgrading does not install a second bridge.

[trunk-js]: https://trunk-rs.github.io/trunk/guide/assets/index.html#js-snippets
