use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail};
use serde::Deserialize;

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
    if config.build.backend == Backend::Nix && !in_nix_environment() {
        return run_doctor_in_nix(&root);
    }
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
    if platforms.contains(&Platform::Windows)
        && !cfg!(target_os = "windows")
        && !command_exists("x86_64-w64-mingw32-gcc")
    {
        problems.push(
            "cross-compiling Windows requires an x86_64-w64-mingw32-gcc toolchain".to_owned(),
        );
    }
    if platforms.contains(&Platform::Web) && !command_exists("trunk") {
        problems.push("Web build requires `trunk`".to_owned());
    }
    if platforms.contains(&Platform::Android) {
        if !command_help_exists("cargo-apk") && !cargo_subcommand_exists("apk") {
            problems.push("Android build requires `cargo-apk`".to_owned());
        }
        if std::env::var_os("ANDROID_SDK_ROOT").is_none()
            && std::env::var_os("ANDROID_HOME").is_none()
        {
            problems.push("Android build requires ANDROID_SDK_ROOT or ANDROID_HOME".to_owned());
        }
        if !command_exists("patch") {
            problems.push("Android compatibility overlay requires `patch`".to_owned());
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

fn run_doctor_in_nix(root: &Path) -> Result<()> {
    let executable = std::env::current_exe().context("resolve current CLI executable")?;
    let status = Command::new("nix")
        .args(["develop", "--command"])
        .arg(executable)
        .arg("doctor")
        .arg(root)
        .current_dir(root)
        .status()
        .context("check the generated Nix environment")?;
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
            label = escape_xml(&config.project.label),
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
    let source = if cfg!(target_os = "windows") {
        cargo_build(root, release, None)?;
        root.join("target")
            .join(profile(release))
            .join(format!("{}.exe", config.project.name))
    } else {
        let target = "x86_64-pc-windows-gnu";
        cargo_build(root, release, Some(target))?;
        root.join("target")
            .join(target)
            .join(profile(release))
            .join(format!("{}.exe", config.project.name))
    };
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
    let cargo_home = prepare_android_overrides(root)?;
    let android_package = android_package_name(&config.project.name);
    let android_manifest = root.join("android/Cargo.toml");
    let mut command = Command::new("cargo");
    let _ = command
        .args(["apk", "build", "--lib", "-p"])
        .arg(android_package)
        .arg("--manifest-path")
        .arg(android_manifest)
        .args(["--target-dir", "target/material-ui/android"]);
    let _ = command.env("CARGO_HOME", cargo_home);
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

fn android_package_name(project_name: &str) -> String {
    format!("{}-android", project_name.replace('-', "_"))
}

#[derive(Debug, Deserialize)]
struct CargoMetadata {
    packages: Vec<CargoPackage>,
}

#[derive(Debug, Deserialize)]
struct CargoPackage {
    name: String,
    version: String,
    manifest_path: PathBuf,
    source: Option<String>,
}

struct AndroidPatch {
    crate_name: &'static str,
    version: &'static str,
    patch: &'static str,
}

const ANDROID_PATCHES: [AndroidPatch; 3] = [
    AndroidPatch {
        crate_name: "iced",
        version: "0.14.0",
        patch: include_str!("../patches/iced-disable-wayland-on-android.patch"),
    },
    AndroidPatch {
        crate_name: "iced_winit",
        version: "0.14.0",
        patch: include_str!("../patches/iced-winit-android-lifecycle.patch"),
    },
    AndroidPatch {
        crate_name: "winit",
        version: "0.30.13",
        patch: include_str!("../patches/winit-android-ime.patch"),
    },
];

fn prepare_android_overrides(root: &Path) -> Result<PathBuf> {
    let output = Command::new("cargo")
        .args([
            "metadata",
            "--format-version",
            "1",
            "--manifest-path",
            "android/Cargo.toml",
        ])
        .current_dir(root)
        .output()
        .context("resolve Android dependency graph")?;
    if !output.status.success() {
        bail!(
            "resolve Android dependency graph: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let metadata: CargoMetadata =
        serde_json::from_slice(&output.stdout).context("parse Cargo metadata")?;
    let override_root = root.join("target/material-ui/android-overrides");
    if override_root.exists() {
        fs::remove_dir_all(&override_root)
            .with_context(|| format!("clear Android overrides {}", override_root.display()))?;
    }
    fs::create_dir_all(&override_root)?;

    let mut overrides = Vec::new();
    for specification in ANDROID_PATCHES {
        let package = metadata
            .packages
            .iter()
            .find(|package| {
                package.name == specification.crate_name && package.version == specification.version
            })
            .with_context(|| {
                format!(
                    "Android support requires {} {}; resolved dependency is incompatible",
                    specification.crate_name, specification.version
                )
            })?;
        if !package
            .source
            .as_deref()
            .is_some_and(|source| source.starts_with("registry+"))
        {
            bail!(
                "Android overlay only supports the crates.io {} {} source",
                package.name,
                package.version
            );
        }
        let source = package
            .manifest_path
            .parent()
            .context("dependency manifest has no parent")?;
        let destination = override_root.join(format!("{}-{}", package.name, package.version));
        copy_directory(source, &destination)?;
        apply_patch(&destination, specification.patch)?;
        overrides.push((package.name.as_str(), destination));
    }

    let cargo_home = root.join("target/material-ui/cargo-home");
    fs::create_dir_all(&cargo_home)?;
    link_cargo_cache(&cargo_home)?;
    let mut config = String::from("[patch.crates-io]\n");
    for (name, path) in overrides {
        let path = path.to_string_lossy().replace('\\', "\\\\");
        writeln!(config, "{name} = {{ path = \"{path}\" }}")
            .expect("writing to a String cannot fail");
    }
    fs::write(cargo_home.join("config.toml"), config)?;
    Ok(cargo_home)
}

fn copy_directory(source: &Path, destination: &Path) -> Result<()> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let target = destination.join(entry.file_name());
        if file_type.is_dir() {
            copy_directory(&entry.path(), &target)?;
        } else if file_type.is_file() {
            let _ = fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}

fn apply_patch(directory: &Path, patch: &str) -> Result<()> {
    let mut child = Command::new("patch")
        .args(["--batch", "-p1"])
        .current_dir(directory)
        .stdin(Stdio::piped())
        .spawn()
        .context("start `patch` for Android compatibility overlay")?;
    child
        .stdin
        .take()
        .context("open patch stdin")?
        .write_all(patch.as_bytes())?;
    let status = child.wait()?;
    if !status.success() {
        bail!(
            "Android compatibility patch failed in {}",
            directory.display()
        );
    }
    Ok(())
}

fn link_cargo_cache(cargo_home: &Path) -> Result<()> {
    let Some(original) = native_cargo_home() else {
        return Ok(());
    };
    for name in ["registry", "git"] {
        let source = original.join(name);
        let destination = cargo_home.join(name);
        if source.exists() && !destination.exists() {
            #[cfg(unix)]
            std::os::unix::fs::symlink(&source, &destination).with_context(|| {
                format!(
                    "link Cargo cache {} to {}",
                    source.display(),
                    destination.display()
                )
            })?;
        }
    }
    Ok(())
}

fn native_cargo_home() -> Option<PathBuf> {
    std::env::var_os("CARGO_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".cargo")))
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
        .arg("--help")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

fn command_help_exists(command: &str) -> bool {
    Command::new(command)
        .arg("--help")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

fn in_nix_environment() -> bool {
    std::env::var_os("IN_NIX_SHELL").is_some()
}

fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
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

    use super::{android_package_name, escape_xml, selected_platforms};

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

    #[test]
    fn escapes_macos_plist_values() {
        assert_eq!(
            escape_xml("A & <B> \"C\""),
            "A &amp; &lt;B&gt; &quot;C&quot;"
        );
    }

    #[test]
    fn derives_android_workspace_package_name() {
        assert_eq!(
            android_package_name("material-ui-smoke"),
            "material_ui_smoke-android"
        );
    }
}
