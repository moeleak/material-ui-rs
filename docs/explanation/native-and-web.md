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

## Platform-Specific Behavior

Platform-specific behavior should stay behind narrow adapters or `cfg`
sections. For example, input method normalization is isolated so native and WASM
behavior remain predictable for text fields and overlays.

When changing shared widgets, verify both native examples and the WebAssembly
showcase if the change touches navigation, overlays, input handling, or
animation behavior.
