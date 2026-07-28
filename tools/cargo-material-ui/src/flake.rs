use std::collections::BTreeSet;
use std::fmt::Write as _;

use crate::config::{Platform, ProjectConfig};

const FLAKE_TEMPLATE: &str = include_str!("../templates/flake.nix.tpl");

pub fn render(config: &ProjectConfig) -> String {
    let platforms = &config.build.platforms;
    let android = platforms.contains(&Platform::Android);
    let web = platforms.contains(&Platform::Web);
    let windows = platforms.contains(&Platform::Windows);
    let linux = platforms.contains(&Platform::Linux);

    let mut platform_packages = Vec::new();
    let mut rust_targets = BTreeSet::new();
    if web {
        platform_packages.extend(["trunk", "binaryen", "wasm-bindgen-cli"]);
        let _ = rust_targets.insert("wasm32-unknown-unknown");
    }
    if android {
        platform_packages.extend(["cargo-apk", "jdk_headless", "patch"]);
        rust_targets.extend(config.android.targets.iter().map(String::as_str));
    }
    if windows {
        platform_packages.push("pkgsCross.mingwW64.stdenv.cc");
        let _ = rust_targets.insert("x86_64-pc-windows-gnu");
    }
    if linux {
        platform_packages.extend(["pkg-config", "wayland", "libxkbcommon"]);
    }
    let mut package_lines = String::new();
    for package in platform_packages {
        let _ = writeln!(package_lines, "            {package}");
    }
    let mut rust_target_lines = String::new();
    for target in rust_targets {
        let _ = writeln!(rust_target_lines, "            \"{target}\"");
    }

    let android_bindings = if android {
        r#"
        androidComposition = pkgs.androidenv.composeAndroidPackages {
          platformVersions = [ "35" ];
          buildToolsVersions = [ "35.0.0" ];
          includeNDK = true;
          ndkVersions = [ "27.2.12479018" ];
        };
"#
    } else {
        ""
    };
    let android_package = if android {
        "            androidComposition.androidsdk\n"
    } else {
        ""
    };
    let android_hook = if android {
        r#"
            export ANDROID_HOME="${androidComposition.androidsdk}/libexec/android-sdk"
            export ANDROID_SDK_ROOT="$ANDROID_HOME"
            export ANDROID_NDK_ROOT="$ANDROID_HOME/ndk-bundle"
            export ANDROID_NDK_HOME="$ANDROID_NDK_ROOT"
            export JAVA_HOME="${pkgs.jdk_headless}"
"#
    } else {
        ""
    };
    let web_hook = if web {
        r#"
            export CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_LINKER="${pkgs.lld}/bin/wasm-ld"
"#
    } else {
        ""
    };

    FLAKE_TEMPLATE
        .replace("{{android_binding}}", android_bindings)
        .replace("{{rust_targets}}", &rust_target_lines)
        .replace("{{platform_packages}}", &package_lines)
        .replace("{{android_package}}", android_package)
        .replace("{{android_hook}}", android_hook)
        .replace("{{web_hook}}", web_hook)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use crate::config::{Backend, Platform, ProjectConfig};

    use super::render;

    #[test]
    fn only_includes_enabled_platform_tools() {
        let config = ProjectConfig::new(
            "app".into(),
            "App".into(),
            "dev.example.app".into(),
            Backend::Nix,
            BTreeSet::from([Platform::Web]),
        );
        let flake = render(&config);
        assert!(flake.contains("trunk"));
        assert!(flake.contains("\"wasm32-unknown-unknown\""));
        assert!(!flake.contains("androidComposition"));
        assert!(!flake.contains("cargo-apk"));
        assert!(!flake.contains("mingwW64"));
        assert!(!flake.contains("\"aarch64-linux-android\""));
        assert!(!flake.contains("{{"));
    }

    #[test]
    fn shell_includes_cargo_material_ui_from_project_flake() {
        let config = ProjectConfig::new(
            "app".into(),
            "App".into(),
            "dev.example.app".into(),
            Backend::Nix,
            BTreeSet::from([Platform::Macos]),
        );
        let flake = render(&config);

        assert!(flake.contains(r#"url = "github:moeleak/material-ui-rs";"#));
        assert!(
            flake.contains("materialUiCli = material-ui-rs.packages.${system}.cargo-material-ui;")
        );
        assert!(flake.contains("            materialUiCli\n"));
    }

    #[test]
    fn android_shell_contains_only_android_cross_tools() {
        let config = ProjectConfig::new(
            "app".into(),
            "App".into(),
            "dev.example.app".into(),
            Backend::Nix,
            BTreeSet::from([Platform::Android]),
        );
        let flake = render(&config);
        assert!(flake.contains("androidComposition"));
        assert!(flake.contains("cargo-apk"));
        assert!(flake.contains("patch"));
        assert!(flake.contains("\"aarch64-linux-android\""));
        assert!(!flake.contains("\"wasm32-unknown-unknown\""));
        assert!(!flake.contains("trunk"));
        assert!(!flake.contains("mingwW64"));
    }
}
