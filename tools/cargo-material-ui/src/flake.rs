use std::fmt::Write as _;

use crate::config::{Platform, ProjectConfig};

const FLAKE_TEMPLATE: &str = include_str!("../templates/flake.nix.tpl");

pub fn render(config: &ProjectConfig) -> String {
    let platforms = &config.build.platforms;
    let android = platforms.contains(&Platform::Android);
    let web = platforms.contains(&Platform::Web);
    let windows = platforms.contains(&Platform::Windows);
    let linux = platforms.contains(&Platform::Linux);
    let macos = platforms.contains(&Platform::Macos);

    let mut platform_packages = Vec::new();
    if web {
        platform_packages.extend(["trunk", "binaryen", "wasm-bindgen-cli"]);
    }
    if android {
        platform_packages.extend(["cargo-apk", "jdk_headless"]);
    }
    if windows {
        platform_packages.push("pkgsCross.mingwW64.stdenv.cc");
    }
    if linux {
        platform_packages.extend(["pkg-config", "wayland", "libxkbcommon"]);
    }
    if macos {
        platform_packages.push("create-dmg");
    }

    let mut package_lines = String::new();
    for package in platform_packages {
        let _ = writeln!(package_lines, "            {package}");
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
        assert!(!flake.contains("androidComposition"));
        assert!(!flake.contains("cargo-apk"));
        assert!(!flake.contains("mingwW64"));
    }
}
