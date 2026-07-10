# Native and WebAssembly

The same widget APIs are intended to work on native and WebAssembly targets, but
the integration points differ.

## Native

Native examples run through Cargo:

```sh
cargo run --example quickstart
cargo run --example showcase
```

Use `material::window`, `material::window_with_min_size`, or
`material::window_settings` to keep window sizing consistent with the examples.

## WebAssembly

The web showcase is built by Trunk:

```sh
trunk build web/index.html --release --dist dist --public-url /
```

Serve the generated `dist/` directory over HTTP. Direct file loading does not
provide reliable MIME types for JavaScript modules and WASM.

On touch browsers, iced/winit does not attach an editable DOM control to its
canvas. `material-ui-rs` therefore ships an internal wasm-bindgen JavaScript
adapter that keeps a visually hidden input focused while a Material text field
is active. The adapter is included and initialized lazily by the crate; host
pages do not need a script tag, hidden input, or to define JavaScript hooks.

The adapter uses composition and `beforeinput` events so Android and iOS
keyboards can commit CJK text, delete in either direction, and submit Enter
without leaking candidate-navigation keys into the application. It activates
only for touch or coarse-pointer environments, leaving desktop input paths
unchanged.

This bridge provides committed-text input. The current sentinel model does not
mirror surrounding application text or selection into the DOM, so browser IME
preedit, reconversion, native selection handles, and surrounding-text
autocorrection are not exposed through iced 0.14.

## Platform-Specific Behavior

Platform-specific behavior should stay behind narrow adapters or `cfg`
sections. For example, input method normalization is isolated so native and WASM
behavior remain predictable for text fields and overlays.

When changing shared widgets, verify both native examples and the WebAssembly
showcase if the change touches navigation, overlays, input handling, or
animation behavior.
