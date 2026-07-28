[package]
name = "{{crate_name}}-android"
version = "0.1.0"
edition = "2024"

[lib]
name = "{{crate_name}}_android"
crate-type = ["cdylib"]
path = "src/lib.rs"

[dependencies]
{{crate_name}} = { package = "{{package_name}}", path = ".." }
iced_winit = { version = "=0.14.0", default-features = false }
winit = { version = "=0.30.13", default-features = false, features = ["android-native-activity"] }

[package.metadata.android]
package = "{{app_id}}"
apk_name = "{{package_name}}"
build_targets = [{{android_targets}}]
resources = "res"
strip = "strip"

[package.metadata.android.sdk]
min_sdk_version = {{min_sdk}}
target_sdk_version = {{target_sdk}}

[package.metadata.android.application]
label = {{label_toml}}
theme = "@style/MaterialUiTheme"
has_code = false

[package.metadata.android.application.activity]
config_changes = "orientation|keyboardHidden|screenSize|screenLayout|uiMode|density"
exported = true
