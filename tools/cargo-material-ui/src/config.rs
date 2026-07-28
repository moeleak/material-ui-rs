use std::collections::BTreeSet;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};

pub const CONFIG_DIR: &str = ".material-ui";
pub const CONFIG_FILE: &str = "project.toml";
pub const STATE_FILE: &str = "generated-state.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum Platform {
    Macos,
    Linux,
    Windows,
    Web,
    Android,
}

impl Platform {
    pub const ALL: [Self; 5] = [
        Self::Macos,
        Self::Linux,
        Self::Windows,
        Self::Web,
        Self::Android,
    ];

    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Macos => "macOS (.app + .dmg)",
            Self::Linux => "Linux executable",
            Self::Windows => "Windows executable",
            Self::Web => "Web (WASM)",
            Self::Android => "Android APK",
        }
    }
}

impl fmt::Display for Platform {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Macos => "macos",
            Self::Linux => "linux",
            Self::Windows => "windows",
            Self::Web => "web",
            Self::Android => "android",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum Backend {
    Native,
    Nix,
}

impl fmt::Display for Backend {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Native => "native",
            Self::Nix => "nix",
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub project: Project,
    pub build: BuildConfig,
    #[serde(default)]
    pub web: WebConfig,
    #[serde(default)]
    pub android: AndroidConfig,
    #[serde(default)]
    pub macos: MacosConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub label: String,
    pub app_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildConfig {
    pub backend: Backend,
    pub platforms: BTreeSet<Platform>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct WebConfig {
    pub public_url: String,
}

impl Default for WebConfig {
    fn default() -> Self {
        Self {
            public_url: "/".to_owned(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AndroidConfig {
    pub targets: BTreeSet<String>,
    pub min_sdk: u32,
    pub target_sdk: u32,
}

impl Default for AndroidConfig {
    fn default() -> Self {
        Self {
            targets: BTreeSet::from(["aarch64-linux-android".to_owned()]),
            min_sdk: 26,
            target_sdk: 35,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct MacosConfig {
    pub minimum_version: String,
}

impl Default for MacosConfig {
    fn default() -> Self {
        Self {
            minimum_version: "12.0".to_owned(),
        }
    }
}

impl ProjectConfig {
    pub fn new(
        name: String,
        label: String,
        app_id: String,
        backend: Backend,
        platforms: BTreeSet<Platform>,
    ) -> Self {
        Self {
            project: Project {
                name,
                label,
                app_id,
            },
            build: BuildConfig { backend, platforms },
            web: WebConfig::default(),
            android: AndroidConfig::default(),
            macos: MacosConfig::default(),
        }
    }

    pub fn load(root: &Path) -> Result<Self> {
        let path = config_path(root);
        let source = fs::read_to_string(&path)
            .with_context(|| format!("read project configuration {}", path.display()))?;
        let config = toml::from_str(&source)
            .with_context(|| format!("parse project configuration {}", path.display()))?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<()> {
        validate_package_name(&self.project.name)?;
        validate_app_id(&self.project.app_id)?;
        if self.build.platforms.is_empty() {
            bail!("at least one platform must be configured");
        }
        Ok(())
    }
}

pub fn config_path(root: &Path) -> PathBuf {
    root.join(CONFIG_DIR).join(CONFIG_FILE)
}

pub fn state_path(root: &Path) -> PathBuf {
    root.join(CONFIG_DIR).join(STATE_FILE)
}

pub fn validate_package_name(name: &str) -> Result<()> {
    let mut characters = name.chars();
    let Some(first) = characters.next() else {
        bail!("package name cannot be empty");
    };
    if !first.is_ascii_alphabetic() && first != '_' {
        bail!("package name must start with an ASCII letter or underscore");
    }
    if !characters.all(|character| character.is_ascii_alphanumeric() || "_-".contains(character)) {
        bail!("package name may only contain ASCII letters, digits, '-' and '_'");
    }
    Ok(())
}

pub fn validate_app_id(app_id: &str) -> Result<()> {
    let segments = app_id.split('.').collect::<Vec<_>>();
    if segments.len() < 3 {
        bail!("application id must contain at least three reverse-DNS segments");
    }
    for segment in segments {
        let mut characters = segment.chars();
        let Some(first) = characters.next() else {
            bail!("application id cannot contain empty segments");
        };
        if !first.is_ascii_alphabetic() {
            bail!("each application id segment must begin with an ASCII letter");
        }
        if !characters.all(|character| character.is_ascii_alphanumeric() || character == '_') {
            bail!("application id segments may only contain ASCII letters, digits and '_'");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{validate_app_id, validate_package_name};

    #[test]
    fn validates_identifiers() {
        assert!(validate_package_name("material_app").is_ok());
        assert!(validate_package_name("2material").is_err());
        assert!(validate_app_id("dev.example.material").is_ok());
        assert!(validate_app_id("example").is_err());
        assert!(validate_app_id("dev.example.bad-value").is_err());
    }
}
