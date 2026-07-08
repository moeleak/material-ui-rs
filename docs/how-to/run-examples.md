# Run the Examples

Use the examples to verify the crate on your platform and to find complete
composition patterns.

## Quick Start

Run the compact tutorial app:

```sh
cargo run --example quickstart
```

This example shows:

- `material::application`
- `window_with_min_size`
- adaptive navigation
- page surfaces
- filled and outlined buttons
- Material typography helpers

## Showcase

Run the full component showcase:

```sh
cargo run --example showcase
```

The showcase is the best source for composing more stateful widgets. Its pages
are grouped by component families under `examples/showcase/pages/`.

## With Nix

If you use the repository flake, enter the shell before running examples:

```sh
nix develop
cargo run --example showcase
```

The shell provides Rust, Trunk, WebAssembly build tools, Node.js, and mdBook.
