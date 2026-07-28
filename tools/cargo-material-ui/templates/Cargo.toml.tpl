[package]
name = "{{package_name}}"
version = "0.1.0"
edition = "2024"

[workspace]
members = [{{workspace_members}}]
resolver = "2"

[lib]
name = "{{crate_name}}"
path = "src/lib.rs"

[[bin]]
name = "{{package_name}}"
path = "src/main.rs"

[dependencies]
iced = "0.14"
material-ui-rs = { git = "https://github.com/moeleak/material-ui-rs", branch = "main" }

[profile.release]
lto = "fat"
codegen-units = 1
strip = true
