# Create and build a multi-platform app

`cargo-material-ui` creates one shared Material application and the smallest
set of platform wrappers needed for the platforms you select.

## Install and initialize

The default Nix development shell already contains the Cargo subcommand:

```sh
nix develop
cargo material-ui new material-app
```

Without Nix, install the Cargo subcommand from this repository:

```sh
cargo install --path tools/cargo-material-ui
cargo material-ui new material-app
```

`cargo material-ui new [PATH]` creates a new directory. If `PATH` is omitted,
the interactive wizard asks for it. To initialize the current directory
instead, run:

```sh
cargo material-ui init
```

The interactive wizard uses `cliclack` prompts and progress animations. It asks
for the Cargo package name, display name, application ID, platforms, and build
environment. Only the current desktop platform is selected by default; Web,
Android, and other desktop targets are opt-in.

If `nix` is available, the environment prompt offers Nix Flake and Native Rust.
Without Nix, Native Rust is selected automatically. Disable animation with
`--no-animations`.

For scripts or CI, provide all values explicitly:

```sh
cargo material-ui new my-app \
  --non-interactive \
  --name my-app \
  --label "My App" \
  --app-id dev.example.my_app \
  --backend native \
  --platform macos,web
```

## Change platforms later

Run the wizard again from the generated project:

```sh
cargo material-ui configure
```

Non-interactive configuration can add or remove platforms:

```sh
cargo material-ui configure \
  --non-interactive \
  --add-platform android \
  --remove-platform web
```

When the backend is Nix, `flake.nix` contains only enabled platform tools and
Rust standard-library targets. For example, Android adds the Android SDK, NDK,
JDK, `cargo-apk`, patch utility, and configured Android Rust targets; it does
not add Web or Windows tools. The generated development shell also includes
`cargo-material-ui`, so `cargo material-ui configure`, `doctor`, and `build`
remain available after entering the project with `nix develop`.

Generated files are recorded in `.material-ui/generated-state.json`. Files that
still match their recorded hash are updated automatically. If a generated file
contains user changes, the interactive command asks before replacing or
removing it and creates a timestamped backup. Non-interactive commands refuse
the conflict unless `--force` is used. An existing `flake.nix` additionally
supports `--force-flake-overwrite`.

The source templates live under `tools/cargo-material-ui/templates/`, separate
from the Rust generator.

The starter UI tracks the actual window size. Compact windows use a bottom
navigation bar; wider windows use an expandable navigation rail. Android uses
three top-level destinations: Components, Navigation, and Structure. Components
has four secondary tabs and remembers its last selected tab. Desktop and Web
retain all six destinations. The note field and theme button exercise keyboard
avoidance and system-bar appearance on Android. A compact modal drawer remains
an explicit [adaptive-navigation option](adaptive-navigation.md).

## Build

Build every configured platform:

```sh
cargo material-ui build --release
```

Or build one configured platform:

```sh
cargo material-ui build --platform android --release
```

Artifacts are copied below `dist/`:

| Platform | Artifact |
| --- | --- |
| macOS | `dist/macos/<name>.dmg` |
| Linux | `dist/linux/<name>` |
| Windows | `dist/windows/<name>.exe` |
| Web | `dist/web/` |
| Android | `dist/android/<name>-<profile>.apk` |

Run `cargo material-ui doctor` to check the configured tools. Nix projects run
the check inside their generated development shell.

For macOS signing, set `MATERIAL_UI_APPLE_SIGNING_IDENTITY`. To submit and staple
the DMG, also set `MATERIAL_UI_APPLE_NOTARY_PROFILE` to a `notarytool` keychain
profile.

## Android runtime behavior

The Android wrapper uses NativeActivity with compatibility patches applied by
the CLI build command:

- suspend drops the graphics surface, resume recreates it, and the window is
  redrawn instead of continuing with a stale surface;
- status and navigation bars remain visible but transparent, with edge-to-edge
  content and Android's automatic contrast scrims disabled;
- status, navigation, cutout, and IME insets are queried separately and the
  generated navigation suite reacts to inset changes;
- Android text events, composing ranges, commits, and IME actions are forwarded
  to iced, with UTF-16 indices converted safely for Rust strings;
- generated applications load available Android system fonts so CJK input and
  other Unicode scripts render without embedding large font files in the APK.

The generated app passes `android::layout_insets().content()` to the navigation
suite, which distributes safe-area padding between content and navigation.
These insets exclude any area already consumed by the native window. The raw
`safe_area_insets()` values remain available for observing keyboard visibility.
The bottom navigation bar hides while the keyboard is visible and restores its
selection after the keyboard closes.

System-bar updates run asynchronously on Android's UI thread. Call
`android::set_system_bars(android::SystemBarsStyle::from_theme(&theme))` at
startup and when the application theme changes, as the starter does. A successful
return means the request was accepted; asynchronous failures are logged and
retried while the app is resumed, retaining the last valid insets. Subscribe to `android::events()`
for inset updates, including changes detected while the foreground app is idle.

The generated Android resources include API 28 cutout/divider settings and API
29 contrast settings. Both gesture navigation and three-button navigation use
the app's extended background. This follows Android's
[edge-to-edge layout guidance](https://developer.android.com/develop/ui/views/layout/edge-to-edge).
The default minimum SDK is 26 and target SDK is 35.

## Verify local Android changes

Run the current CLI directly from a repository checkout so validation uses its
latest templates and compatibility patches:

```sh
cargo run --manifest-path tools/cargo-material-ui/Cargo.toml -- new /tmp/material-android-check \
  --non-interactive --name material-android-check \
  --label "Material Android Check" --app-id dev.example.material_android_check \
  --backend native --platform android
```

In the generated `Cargo.toml`, replace the `material-ui-rs` Git dependency with
an absolute path to the checkout under test:

```toml
material-ui-rs = { path = "/absolute/path/to/material-ui-rs" }
```

With `ANDROID_HOME` (or `ANDROID_SDK_ROOT`), the NDK, JDK, `cargo-apk`, and the
configured Rust Android targets installed, build and install the resulting APK:

```sh
cargo run --manifest-path tools/cargo-material-ui/Cargo.toml -- doctor /tmp/material-android-check
cargo run --manifest-path tools/cargo-material-ui/Cargo.toml -- build /tmp/material-android-check --platform android
adb devices
adb -s emulator-5554 install -r /tmp/material-android-check/dist/android/material-android-check-debug.apk
adb -s emulator-5554 shell am start -n dev.example.material_android_check/android.app.NativeActivity
```

Use the serial reported by `adb devices` if it differs. Build through the CLI:
it prepares the lifecycle and IME overrides before invoking `cargo apk`.
Check all three destinations, Components tab restoration, light/dark themes,
the note field with the keyboard open and closed, rotation, and background /
foreground transitions. Repeat with gesture and three-button navigation, then
restore the emulator's original navigation and rotation settings.
