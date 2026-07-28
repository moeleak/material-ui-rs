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

The starter UI tracks the actual window size. Desktop and Web builds keep the
standard bottom navigation bar on compact windows and use an expandable
navigation rail on wider windows. Android explicitly selects a modal drawer for
compact windows. The drawer follows AndroidX Compose's fixed-width,
translated-surface model with a 256 ms FastOutSlowIn tween; its text and item
geometry are not remeasured while the drawer moves.

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

The Android wrapper uses NativeActivity and the same compatibility approach as
the sibling `rdict` application:

- suspend drops the graphics surface, resume recreates it, and the window is
  redrawn instead of continuing with a stale surface;
- status and navigation bars remain visible but transparent, with edge-to-edge
  content and Android's automatic contrast scrims disabled;
- status, navigation, cutout, and IME insets are queried separately and the
  generated page reacts to inset changes;
- Android text events, composing ranges, commits, and IME actions are forwarded
  to iced, with UTF-16 indices converted safely for Rust strings;
- generated applications load available Android system fonts so CJK input and
  other Unicode scripts render without embedding large font files in the APK.

The generated app applies system and IME-safe content padding. Use
`material_ui_rs::android::set_system_bars` when the app theme changes so icon
appearance and fallback colors stay synchronized.
