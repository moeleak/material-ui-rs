[package]
name = "{{package_name}}"
version = "0.1.0"
edition = "2024"
rust-version = "1.88"

[workspace]
members = [{{workspace_members}}]
resolver = "3"

[lib]
name = "{{crate_name}}"
path = "src/lib.rs"

[[bin]]
name = "{{package_name}}"
path = "src/main.rs"

[dependencies]
iced = { version = "=0.14.0", default-features = false, features = [
  "wgpu",
  "tiny-skia",
  "crisp",
  "web-colors",
  "thread-pool",
] }
material-ui-rs = { git = "https://github.com/moeleak/material-ui-rs", branch = "main" }

[target.'cfg(not(target_os = "android"))'.dependencies]
iced = { version = "=0.14.0", default-features = false, features = [
  "linux-theme-detection",
  "x11",
  "wayland",
] }

[profile.release]
lto = "fat"
codegen-units = 1
strip = true
