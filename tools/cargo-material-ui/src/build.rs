use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail};

use crate::cli::{BuildArgs, DoctorArgs};
use crate::config::{Backend, Platform, ProjectConfig};

pub fn run(args: &BuildArgs) -> Result<()> {
    let root = absolute_path(&args.path)?;
    let config = ProjectConfig::load(&root)?;
    let platforms = selected_platforms(&config, &args.platform)?;

    if config.build.backend == Backend::Nix && !args.prepared {
        return run_in_nix(&root, args);
    }

    preflight(&config, &platforms)?;
    let profile = if args.release { "release" } else { "debug" };

    for platform in platforms {
        match platform {
            Platform::Macos => build_macos(&root, &config, args.release)?,
            Platform::Linux => build_native_binary(&root, &config, args.release, "linux")?,
            Platform::Windows => build_windows(&root, &config, args.release)?,
            Platform::Web => build_web(&root, &config, args.release)?,
            Platform::Android => build_android(&root, &config, args.release)?,
        }
    }

    eprintln!(
        "Built configured {profile} artifacts in {}",
        root.join("dist").display()
    );
    Ok(())
}

pub fn doctor(args: &DoctorArgs) -> Result<()> {
    let root = absolute_path(&args.path)?;
    let config = ProjectConfig::load(&root)?;
    let platforms = config.build.platforms.clone();
    preflight(&config, &platforms)?;
    eprintln!("All configured build tools are available.");
    Ok(())
}

fn selected_platforms(
    config: &ProjectConfig,
    requested: &[Platform],
) -> Result<BTreeSet<Platform>> {
    if requested.is_empty() {
        return Ok(config.build.platforms.clone());
    }
    let requested = requested.iter().copied().collect::<BTreeSet<_>>();
    let missing = requested
        .difference(&config.build.platforms)
        .copied()
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        bail!(
            "platform(s) {} are not configured; run `cargo material-ui configure --add-platform ...`",
            missing
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    Ok(requested)
}

fn preflight(config: &ProjectConfig, platforms: &BTreeSet<Platform>) -> Result<()> {
    let mut problems = Vec::new();
    if !command_exists("cargo") {
        problems.push("Cargo is not available".to_owned());
    }
    if config.build.backend == Backend::Nix && !in_nix_environment() && !command_exists("nix") {
        problems.push("Nix backend is configured, but `nix` is not available".to_owned());
    }
    if platforms.contains(&Platform::Macos) && !cfg!(target_os = "macos") {
        problems.push("macOS DMG requires a macOS host".to_owned());
    }
    if platforms.contains(&Platform::Linux) && !cfg!(target_os = "linux") {
        problems.push("Linux executable requires a Linux host in the first release".to_owned());
    }
    if platforms.contains(&Platform::Web) && !command_exists("trunk") {
        problems.push("Web build requires `trunk`".to_owned());
    }
    if platforms.contains(&Platform::Android) {
        if !command_exists("cargo-apk") && !cargo_subcommand_exists("apk") {
            problems.push("Android build requires `cargo-apk`".to_owned());
        }
        if std::env::var_os("ANDROID_SDK_ROOT").is_none()
            && std::env::var_os("ANDROID_HOME").is_none()
        {
            problems.push("Android build requires ANDROID_SDK_ROOT or ANDROID_HOME".to_owned());
        }
    }
    if !problems.is_empty() {
        bail!("build preflight failed:\n- {}", problems.join("\n- "));
    }
    Ok(())
}

fn run_in_nix(root: &Path, args: &BuildArgs) -> Result<()> {
    let executable = std::env::current_exe().context("resolve current CLI executable")?;
    let mut command = Command::new("nix");
    let _ = command
        .args(["develop", "--command"])
        .arg(executable)
        .arg("build")
        .arg(root)
        .arg("--prepared");
    if args.release {
        let _ = command.arg("--release");
    }
    for platform in &args.platform {
        let _ = command.arg("--platform").arg(platform.to_string());
    }
    let status = command
        .current_dir(root)
        .status()
        .context("enter Nix environment")?;
    if !status.success() {
        bail!("Nix build environment exited with {status}");
    }
    Ok(())
}

fn build_native_binary(
    root: &Path,
    config: &ProjectConfig,
    release: bool,
    directory: &str,
) -> Result<()> {
    cargo_build(root, release, None)?;
    let source = root
        .join("target")
        .join(profile(release))
        .join(&config.project.name);
    let destination = root.join("dist").join(directory).join(&config.project.name);
    copy_artifact(&source, &destination)
}

fn build_macos(root: &Path, config: &ProjectConfig, release: bool) -> Result<()> {
    cargo_build(root, release, None)?;
    let bundle = root
        .join("target")
        .join("material-ui")
        .join("macos")
        .join(format!("{}.app", config.project.label));
    let contents = bundle.join("Contents");
    let executable_dir = contents.join("MacOS");
    fs::create_dir_all(&executable_dir)?;
    let binary = root
        .join("target")
        .join(profile(release))
        .join(&config.project.name);
    copy_artifact(&binary, &executable_dir.join(&config.project.name))?;
    fs::write(
        contents.join("Info.plist"),
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>CFBundleExecutable</key><string>{name}</string>
<key>CFBundleIdentifier</key><string>{app_id}</string>
<key>CFBundleName</key><string>{label}</string>
<key>CFBundlePackageType</key><string>APPL</string>
<key>LSMinimumSystemVersion</key><string>{minimum}</string>
</dict></plist>
"#,
            name = config.project.name,
            app_id = config.project.app_id,
            label = config.project.label,
            minimum = config.macos.minimum_version,
        ),
    )?;

    if let Ok(identity) = std::env::var("MATERIAL_UI_APPLE_SIGNING_IDENTITY") {
        run_command(
            Command::new("codesign")
                .args(["--force", "--deep", "--options", "runtime", "--sign"])
                .arg(identity)
                .arg(&bundle),
            "codesign macOS application",
        )?;
    }

    let destination = root
        .join("dist")
        .join("macos")
        .join(format!("{}.dmg", config.project.name));
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }
    let _ = fs::remove_file(&destination);
    run_command(
        Command::new("hdiutil")
            .args(["create", "-volname"])
            .arg(&config.project.label)
            .args(["-srcfolder"])
            .arg(&bundle)
            .args(["-ov", "-format", "UDZO"])
            .arg(&destination),
        "create macOS DMG",
    )?;

    if let Ok(profile) = std::env::var("MATERIAL_UI_APPLE_NOTARY_PROFILE") {
        run_command(
            Command::new("xcrun")
                .args([
                    "notarytool",
                    "submit",
                    "--keychain-profile",
                    &profile,
                    "--wait",
                ])
                .arg(&destination),
            "notarize macOS DMG",
        )?;
        run_command(
            Command::new("xcrun")
                .args(["stapler", "staple"])
                .arg(&destination),
            "staple macOS DMG",
        )?;
    }
    Ok(())
}

fn build_windows(root: &Path, config: &ProjectConfig, release: bool) -> Result<()> {
    let target = "x86_64-pc-windows-gnu";
    cargo_build(root, release, Some(target))?;
    let source = root
        .join("target")
        .join(target)
        .join(profile(release))
        .join(format!("{}.exe", config.project.name));
    let destination = root
        .join("dist")
        .join("windows")
        .join(format!("{}.exe", config.project.name));
    copy_artifact(&source, &destination)
}

fn build_web(root: &Path, config: &ProjectConfig, release: bool) -> Result<()> {
    let destination = root.join("dist").join("web");
    let mut command = Command::new("trunk");
    let _ = command
        .args(["build", "web/index.html", "--dist"])
        .arg(&destination)
        .args(["--public-url", &config.web.public_url]);
    if release {
        let _ = command.arg("--release");
    }
    run_command(command.current_dir(root), "build Web application")
}

fn build_android(root: &Path, config: &ProjectConfig, release: bool) -> Result<()> {
    let mut command = Command::new("cargo");
    let _ = command
        .args([
            "apk",
            "build",
            "--lib",
            "--manifest-path",
            "android/Cargo.toml",
        ])
        .args(["--target-dir", "target/material-ui/android"]);
    if release {
        let _ = command.arg("--release");
    }
    run_command(command.current_dir(root), "build Android APK")?;

    let apk_dir = root
        .join("target/material-ui/android")
        .join(profile(release))
        .join("apk");
    let apk = find_file(&apk_dir, "apk")?
        .with_context(|| format!("no APK found below {}", apk_dir.display()))?;
    let destination = root.join("dist").join("android").join(format!(
        "{}-{}.apk",
        config.project.name,
        profile(release)
    ));
    copy_artifact(&apk, &destination)
}

fn cargo_build(root: &Path, release: bool, target: Option<&str>) -> Result<()> {
    let mut command = Command::new("cargo");
    let _ = command.arg("build");
    if release {
        let _ = command.arg("--release");
    }
    if let Some(target) = target {
        let _ = command.args(["--target", target]);
    }
    run_command(command.current_dir(root), "build Rust application")
}

fn copy_artifact(source: &Path, destination: &Path) -> Result<()> {
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }
    let _ = fs::copy(source, destination).with_context(|| {
        format!(
            "copy build artifact {} to {}",
            source.display(),
            destination.display()
        )
    })?;
    eprintln!("artifact: {}", destination.display());
    Ok(())
}

fn find_file(directory: &Path, extension: &str) -> Result<Option<PathBuf>> {
    if !directory.exists() {
        return Ok(None);
    }
    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        if path.is_dir() {
            if let Some(found) = find_file(&path, extension)? {
                return Ok(Some(found));
            }
        } else if path.extension().and_then(|value| value.to_str()) == Some(extension) {
            return Ok(Some(path));
        }
    }
    Ok(None)
}

fn run_command(command: &mut Command, description: &str) -> Result<()> {
    let status = command
        .status()
        .with_context(|| format!("{description}: failed to start command"))?;
    if !status.success() {
        bail!("{description}: command exited with {status}");
    }
    Ok(())
}

const fn profile(release: bool) -> &'static str {
    if release { "release" } else { "debug" }
}

fn command_exists(command: &str) -> bool {
    Command::new(command)
        .arg("--version")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

fn cargo_subcommand_exists(subcommand: &str) -> bool {
    Command::new("cargo")
        .arg(subcommand)
        .arg("--version")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

fn in_nix_environment() -> bool {
    std::env::var_os("IN_NIX_SHELL").is_some()
}

fn absolute_path(path: &Path) -> Result<PathBuf> {
    if path.is_absolute() {
        Ok(path.to_owned())
    } else {
        Ok(std::env::current_dir()?.join(path))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use crate::config::{Backend, Platform, ProjectConfig};

    use super::selected_platforms;

    #[test]
    fn rejects_unconfigured_build_platform() {
        let config = ProjectConfig::new(
            "app".into(),
            "App".into(),
            "dev.example.app".into(),
            Backend::Native,
            BTreeSet::from([Platform::Web]),
        );
        assert!(selected_platforms(&config, &[Platform::Android]).is_err());
        assert_eq!(
            selected_platforms(&config, &[]).unwrap(),
            BTreeSet::from([Platform::Web])
        );
    }
}
