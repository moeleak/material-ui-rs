use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};
use cliclack::{confirm, input, multiselect, select};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::cli::{ConfigureArgs, InitArgs, NewArgs, ProjectArgs};
use crate::config::{
    Backend, CONFIG_DIR, Platform, ProjectConfig, state_path, validate_app_id,
    validate_package_name,
};
use crate::flake;
use crate::ui::Ui;

const CARGO_TEMPLATE: &str = include_str!("../templates/Cargo.toml.tpl");
const MAIN_TEMPLATE: &str = include_str!("../templates/src/main.rs.tpl");
const APP_TEMPLATE: &str = include_str!("../templates/src/lib.rs.tpl");
const WEB_TEMPLATE: &str = include_str!("../templates/web/index.html.tpl");
const ANDROID_CARGO_TEMPLATE: &str = include_str!("../templates/android/Cargo.toml.tpl");
const ANDROID_MAIN_TEMPLATE: &str = include_str!("../templates/android/src/lib.rs.tpl");
const ANDROID_STYLES_TEMPLATE: &str = include_str!("../templates/android/res/values/styles.xml");

#[derive(Debug, Default, Serialize, Deserialize)]
struct GeneratedState {
    #[serde(default)]
    files: BTreeMap<String, String>,
}

#[derive(Clone, Copy)]
struct ApplyOptions {
    interactive: bool,
    force: bool,
    force_flake: bool,
}

pub fn init(args: InitArgs) -> Result<()> {
    let root = std::env::current_dir().context("resolve the current directory")?;
    let args = args.project;
    let ui = Ui::new(args.no_animations);
    ui.intro("Initialize a material-ui-rs application");

    let interactive = interaction_mode(args.non_interactive, "init")?;
    generate_project(&root, args, interactive, &ui, "Initialize this project?")
}

pub fn new(args: NewArgs) -> Result<()> {
    let project = args.project;
    let ui = Ui::new(project.no_animations);
    ui.intro("Create a material-ui-rs application");

    let interactive = interaction_mode(project.non_interactive, "new")?;
    let root = resolve_new_path(args.path, interactive)?;
    ensure_new_project_root(&root)?;
    generate_project(&root, project, interactive, &ui, "Create this project?")
}

fn generate_project(
    root: &Path,
    args: ProjectArgs,
    interactive: bool,
    ui: &Ui,
    confirmation: &str,
) -> Result<()> {
    let name = resolve_name(args.name, root, interactive)?;
    let label = resolve_label(args.label, &name, interactive)?;
    let app_id = resolve_app_id(args.app_id, &name, interactive)?;
    let platforms = resolve_platforms(args.platform, interactive)?;
    let backend = resolve_backend(args.backend, interactive)?;

    let config = ProjectConfig::new(name, label, app_id, backend, platforms);
    config.validate()?;

    if interactive {
        print_summary(root, &config);
        if !confirm(confirmation).initial_value(true).interact()? {
            bail!("initialization cancelled");
        }
    }

    let spinner = ui.spinner("Generating project files");
    apply_project(
        root,
        &config,
        ApplyOptions {
            interactive,
            force: false,
            force_flake: args.force_flake_overwrite,
        },
    )?;
    spinner.finish("Project files generated");

    if backend == Backend::Nix {
        refresh_flake_lock(root, ui)?;
    }

    ui.success("Project ready");
    Ok(())
}

pub fn configure(args: ConfigureArgs) -> Result<()> {
    let root = absolute_path(&args.path)?;
    let ui = Ui::new(args.no_animations);
    ui.intro("Configure material-ui-rs platforms");

    let mut config = ProjectConfig::load(&root)?;
    let interactive = !args.non_interactive && Ui::is_interactive();
    if !interactive && !args.non_interactive {
        bail!("configure requires a terminal; pass --non-interactive");
    }

    if interactive {
        let defaults = Platform::ALL
            .iter()
            .map(|platform| config.build.platforms.contains(platform))
            .collect::<Vec<_>>();
        let initial = Platform::ALL
            .iter()
            .copied()
            .zip(defaults)
            .filter_map(|(platform, enabled)| enabled.then_some(platform))
            .collect::<Vec<_>>();
        let mut prompt = multiselect("Build platforms");
        for platform in Platform::ALL {
            prompt = prompt.item(platform, platform.display_name(), "");
        }
        config.build.platforms = prompt
            .initial_values(initial)
            .interact()?
            .into_iter()
            .collect();

        let backends = available_backends();
        let mut prompt = select("Build environment");
        for backend in &backends {
            prompt = prompt.item(*backend, backend.to_string(), "");
        }
        config.build.backend = prompt.initial_value(config.build.backend).interact()?;
    } else {
        for platform in args.add_platform {
            let _ = config.build.platforms.insert(platform);
        }
        for platform in args.remove_platform {
            let _ = config.build.platforms.remove(&platform);
        }
        if let Some(backend) = args.backend {
            config.build.backend = backend;
        }
    }

    config.validate()?;
    if interactive {
        print_summary(&root, &config);
        if !confirm("Apply these changes?")
            .initial_value(true)
            .interact()?
        {
            bail!("configuration cancelled");
        }
    }

    let spinner = ui.spinner("Updating platform files");
    apply_project(
        &root,
        &config,
        ApplyOptions {
            interactive,
            force: args.force,
            force_flake: args.force_flake_overwrite,
        },
    )?;
    spinner.finish("Platform files updated");

    if config.build.backend == Backend::Nix {
        refresh_flake_lock(&root, &ui)?;
    }
    ui.success("Configuration updated");
    Ok(())
}

fn interaction_mode(non_interactive: bool, command: &str) -> Result<bool> {
    let interactive = !non_interactive && Ui::is_interactive();
    if !interactive && !non_interactive {
        bail!("{command} requires a terminal; pass --non-interactive with all required values");
    }
    Ok(interactive)
}

fn resolve_new_path(path: Option<PathBuf>, interactive: bool) -> Result<PathBuf> {
    let path = if let Some(path) = path {
        path
    } else if interactive {
        let value: String = input("Project directory")
            .default_input("material-app")
            .interact()?;
        PathBuf::from(value)
    } else {
        bail!("--non-interactive requires a project path");
    };
    absolute_path(&path)
}

fn ensure_new_project_root(root: &Path) -> Result<()> {
    if root.exists() {
        bail!(
            "project directory {} already exists; run `cargo material-ui init` from that directory",
            root.display()
        );
    }
    Ok(())
}

fn resolve_name(name: Option<String>, root: &Path, interactive: bool) -> Result<String> {
    let default = root
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("material-app")
        .replace(' ', "-");
    let name = if let Some(name) = name {
        name
    } else if interactive {
        input("Cargo package name")
            .default_input(&default)
            .validate(|input: &String| validate_package_name(input).map_err(|e| e.to_string()))
            .interact()?
    } else {
        bail!("--non-interactive requires --name");
    };
    validate_package_name(&name)?;
    Ok(name)
}

fn resolve_label(label: Option<String>, name: &str, interactive: bool) -> Result<String> {
    if let Some(label) = label {
        return Ok(label);
    }
    let default = title_case(name);
    if interactive {
        Ok(input("Application name")
            .default_input(&default)
            .interact()?)
    } else {
        Ok(default)
    }
}

fn resolve_app_id(app_id: Option<String>, name: &str, interactive: bool) -> Result<String> {
    let default = format!("dev.example.{}", crate_name(name));
    let app_id = if let Some(app_id) = app_id {
        app_id
    } else if interactive {
        input("Application ID")
            .default_input(&default)
            .validate(|input: &String| validate_app_id(input).map_err(|e| e.to_string()))
            .interact()?
    } else {
        bail!("--non-interactive requires --app-id");
    };
    validate_app_id(&app_id)?;
    Ok(app_id)
}

fn resolve_platforms(platforms: Vec<Platform>, interactive: bool) -> Result<BTreeSet<Platform>> {
    if !platforms.is_empty() {
        return Ok(platforms.into_iter().collect());
    }
    if !interactive {
        bail!("--non-interactive requires at least one --platform");
    }
    let initial = default_init_platforms(host_platform());
    let mut prompt = multiselect("Build platforms");
    for platform in Platform::ALL {
        prompt = prompt.item(platform, platform.display_name(), "");
    }
    Ok(prompt
        .initial_values(initial)
        .interact()?
        .into_iter()
        .collect())
}

fn resolve_backend(backend: Option<Backend>, interactive: bool) -> Result<Backend> {
    if let Some(backend) = backend {
        if backend == Backend::Nix && !command_exists("nix") {
            bail!("Nix backend selected, but `nix` was not found");
        }
        return Ok(backend);
    }
    if !interactive {
        bail!("--non-interactive requires --backend");
    }
    let available = available_backends();
    if available.len() == 1 {
        eprintln!("Nix was not detected; using the native Rust toolchain.");
        return Ok(Backend::Native);
    }
    Ok(select("Build environment")
        .item(Backend::Nix, "Nix Flake", "reproducible")
        .item(Backend::Native, "Native Rust", "use installed tools")
        .initial_value(Backend::Nix)
        .interact()?)
}

fn available_backends() -> Vec<Backend> {
    if command_exists("nix") {
        vec![Backend::Nix, Backend::Native]
    } else {
        vec![Backend::Native]
    }
}

fn print_summary(root: &Path, config: &ProjectConfig) {
    eprintln!("\nProject:  {}", config.project.label);
    eprintln!("Path:     {}", root.display());
    eprintln!("App ID:   {}", config.project.app_id);
    eprintln!("Backend:  {}", config.build.backend);
    eprintln!(
        "Platforms: {}\n",
        config
            .build
            .platforms
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ")
    );
}

fn apply_project(root: &Path, config: &ProjectConfig, options: ApplyOptions) -> Result<()> {
    fs::create_dir_all(root)
        .with_context(|| format!("create project directory {}", root.display()))?;
    let old_state = load_state(root)?;
    let desired = render_files(config)?;
    let mut overrides = BTreeSet::new();
    let mut removals = BTreeSet::new();

    for (relative, content) in &desired {
        let path = root.join(relative);
        if !path.exists() {
            continue;
        }
        let current_hash = content_hash(&fs::read(&path)?);
        let new_hash = content_hash(content.as_bytes());
        let owned = old_state
            .files
            .get(relative)
            .is_some_and(|hash| hash == &current_hash);
        if current_hash != new_hash && !owned {
            let is_flake = relative == "flake.nix";
            let forced = options.force || (is_flake && options.force_flake);
            if !forced && !confirm_override(&path, options.interactive)? {
                bail!("refusing to overwrite conflicting file {}", path.display());
            }
            let _ = overrides.insert(relative.clone());
        }
    }

    for relative in old_state.files.keys() {
        if desired.contains_key(relative) {
            continue;
        }
        let path = root.join(relative);
        if !path.exists() {
            continue;
        }
        let current_hash = content_hash(&fs::read(&path)?);
        let owned = old_state
            .files
            .get(relative)
            .is_some_and(|hash| hash == &current_hash);
        if !owned && !options.force && !confirm_remove(&path, options.interactive)? {
            bail!("refusing to remove conflicting file {}", path.display());
        }
        if !owned {
            let _ = overrides.insert(relative.clone());
        }
        let _ = removals.insert(relative.clone());
    }

    let mut next_state = GeneratedState::default();

    for (relative, content) in &desired {
        let path = root.join(relative);
        let new_hash = content_hash(content.as_bytes());
        if overrides.contains(relative) {
            backup_file(&path)?;
        }
        atomic_write(&path, content.as_bytes())?;
        let _ = next_state.files.insert(relative.clone(), new_hash);
    }

    for relative in removals {
        let path = root.join(&relative);
        if overrides.contains(&relative) {
            backup_file(&path)?;
        }
        fs::remove_file(&path)
            .with_context(|| format!("remove disabled platform file {}", path.display()))?;
    }

    let state = serde_json::to_vec_pretty(&next_state)?;
    atomic_write(&state_path(root), &state)?;
    Ok(())
}

fn render_files(config: &ProjectConfig) -> Result<BTreeMap<String, String>> {
    let mut files = BTreeMap::new();
    let crate_name = crate_name(&config.project.name);
    let label_rust = format!("{:?}", config.project.label);
    let workspace_members = if config.build.platforms.contains(&Platform::Android) {
        r#""android""#
    } else {
        ""
    };
    let cargo = render_template(
        CARGO_TEMPLATE,
        &[
            ("package_name", &config.project.name),
            ("crate_name", &crate_name),
            ("workspace_members", workspace_members),
        ],
    );
    let _ = files.insert("Cargo.toml".to_owned(), cargo);
    let _ = files.insert(
        "src/main.rs".to_owned(),
        render_template(MAIN_TEMPLATE, &[("crate_name", &crate_name)]),
    );
    let _ = files.insert(
        "src/lib.rs".to_owned(),
        render_template(APP_TEMPLATE, &[("label_rust", &label_rust)]),
    );
    let _ = files.insert(
        ".gitignore".to_owned(),
        "/target\n/dist\n/.direnv\n*.material-ui-backup-*\n".to_owned(),
    );
    let config_source = toml::to_string_pretty(config)?;
    let _ = files.insert(format!("{CONFIG_DIR}/project.toml"), config_source);

    if config.build.platforms.contains(&Platform::Web) {
        let _ = files.insert(
            "web/index.html".to_owned(),
            render_template(
                WEB_TEMPLATE,
                &[
                    ("label_html", &escape_html(&config.project.label)),
                    ("package_name", &config.project.name),
                ],
            ),
        );
    }
    if config.build.platforms.contains(&Platform::Android) {
        for (path, content) in render_android(config, &crate_name) {
            let _ = files.insert(path, content);
        }
    }
    if config.build.backend == Backend::Nix {
        let _ = files.insert("flake.nix".to_owned(), flake::render(config));
        let _ = files.insert(".envrc".to_owned(), "use flake\n".to_owned());
    }
    Ok(files)
}

fn render_android(config: &ProjectConfig, crate_name: &str) -> BTreeMap<String, String> {
    let mut files = BTreeMap::new();
    let targets = config
        .android
        .targets
        .iter()
        .map(|target| format!(r#""{target}""#))
        .collect::<Vec<_>>()
        .join(", ");
    let min_sdk = config.android.min_sdk.to_string();
    let target_sdk = config.android.target_sdk.to_string();
    let label_toml = toml::Value::String(config.project.label.clone()).to_string();
    let cargo = render_template(
        ANDROID_CARGO_TEMPLATE,
        &[
            ("crate_name", crate_name),
            ("package_name", &config.project.name),
            ("app_id", &config.project.app_id),
            ("android_targets", &targets),
            ("min_sdk", &min_sdk),
            ("target_sdk", &target_sdk),
            ("label_toml", &label_toml),
        ],
    );
    let _ = files.insert("android/Cargo.toml".to_owned(), cargo);
    let _ = files.insert(
        "android/src/lib.rs".to_owned(),
        render_template(ANDROID_MAIN_TEMPLATE, &[("crate_name", crate_name)]),
    );
    let _ = files.insert(
        "android/res/values/styles.xml".to_owned(),
        ANDROID_STYLES_TEMPLATE.to_owned(),
    );
    files
}

fn render_template(template: &str, replacements: &[(&str, &str)]) -> String {
    replacements
        .iter()
        .fold(template.to_owned(), |rendered, (key, value)| {
            rendered.replace(&format!("{{{{{key}}}}}"), value)
        })
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn load_state(root: &Path) -> Result<GeneratedState> {
    let path = state_path(root);
    if !path.exists() {
        return Ok(GeneratedState::default());
    }
    let bytes = fs::read(&path)?;
    serde_json::from_slice(&bytes).context("parse generated state")
}

fn confirm_override(path: &Path, interactive: bool) -> Result<bool> {
    if !interactive {
        return Ok(false);
    }
    Ok(
        confirm(format!("{} has user changes. Override it?", path.display()))
            .initial_value(false)
            .interact()?,
    )
}

fn confirm_remove(path: &Path, interactive: bool) -> Result<bool> {
    if !interactive {
        return Ok(false);
    }
    Ok(confirm(format!(
        "{} belongs to a disabled platform but has user changes. Remove it?",
        path.display()
    ))
    .initial_value(false)
    .interact()?)
}

fn backup_file(path: &Path) -> Result<()> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system clock is before Unix epoch")?
        .as_secs();
    let extension = format!("material-ui-backup-{timestamp}");
    let backup = path.with_extension(extension);
    let _ = fs::copy(path, &backup).with_context(|| {
        format!(
            "back up conflicting file {} to {}",
            path.display(),
            backup.display()
        )
    })?;
    eprintln!("backup: {}", backup.display());
    Ok(())
}

fn atomic_write(path: &Path, content: &[u8]) -> Result<()> {
    let parent = path
        .parent()
        .with_context(|| format!("{} has no parent directory", path.display()))?;
    fs::create_dir_all(parent)?;
    let temporary = parent.join(format!(
        ".{}.material-ui-tmp-{}",
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("generated"),
        std::process::id()
    ));
    {
        let mut file = fs::File::create(&temporary)?;
        file.write_all(content)?;
        file.sync_all()?;
    }
    fs::rename(&temporary, path)?;
    Ok(())
}

fn refresh_flake_lock(root: &Path, ui: &Ui) -> Result<()> {
    if !command_exists("nix") {
        bail!("Nix backend selected, but `nix` was not found");
    }
    let spinner = ui.spinner("Locking Nix inputs");
    let status = Command::new("nix")
        .args(["flake", "lock"])
        .current_dir(root)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .status()
        .context("run `nix flake lock`")?;
    if !status.success() {
        bail!("`nix flake lock` failed");
    }
    spinner.finish("Nix inputs locked");
    Ok(())
}

fn content_hash(content: &[u8]) -> String {
    let digest = Sha256::digest(content);
    digest.iter().fold(
        String::with_capacity(digest.len() * 2),
        |mut output, byte| {
            write!(output, "{byte:02x}").expect("writing to a String cannot fail");
            output
        },
    )
}

fn absolute_path(path: &Path) -> Result<PathBuf> {
    if path.is_absolute() {
        return Ok(path.to_owned());
    }
    Ok(std::env::current_dir()?.join(path))
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

fn crate_name(name: &str) -> String {
    name.replace('-', "_")
}

fn title_case(name: &str) -> String {
    name.split(['-', '_'])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut characters = part.chars();
            characters.next().map_or_else(String::new, |first| {
                first.to_uppercase().collect::<String>() + characters.as_str()
            })
        })
        .collect::<Vec<_>>()
        .join(" ")
}

const fn host_platform() -> Platform {
    if cfg!(target_os = "macos") {
        Platform::Macos
    } else if cfg!(target_os = "windows") {
        Platform::Windows
    } else {
        Platform::Linux
    }
}

fn default_init_platforms(host: Platform) -> Vec<Platform> {
    vec![host]
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::fs;

    use tempfile::tempdir;

    use crate::config::{Backend, Platform, ProjectConfig, state_path};

    use super::{
        ApplyOptions, apply_project, default_init_platforms, ensure_new_project_root,
        host_platform, render_files,
    };

    fn config(platforms: BTreeSet<Platform>) -> ProjectConfig {
        ProjectConfig::new(
            "sample-app".into(),
            "Sample App".into(),
            "dev.example.sample".into(),
            Backend::Native,
            platforms,
        )
    }

    #[test]
    fn init_defaults_to_the_current_platform_only() {
        assert_eq!(
            default_init_platforms(host_platform()),
            vec![host_platform()]
        );
        assert_eq!(
            default_init_platforms(Platform::Macos),
            vec![Platform::Macos]
        );
        assert_eq!(
            default_init_platforms(Platform::Linux),
            vec![Platform::Linux]
        );
        assert_eq!(
            default_init_platforms(Platform::Windows),
            vec![Platform::Windows]
        );
    }

    #[test]
    fn new_requires_a_path_that_does_not_exist() {
        let parent = tempdir().unwrap();
        assert!(ensure_new_project_root(parent.path()).is_err());
        assert!(ensure_new_project_root(&parent.path().join("material-app")).is_ok());
    }

    #[test]
    fn adds_and_removes_unmodified_platform_files() {
        let directory = tempdir().unwrap();
        let options = ApplyOptions {
            interactive: false,
            force: false,
            force_flake: false,
        };
        apply_project(
            directory.path(),
            &config(BTreeSet::from([Platform::Web, Platform::Android])),
            options,
        )
        .unwrap();
        assert!(directory.path().join("web/index.html").exists());
        assert!(directory.path().join("android/Cargo.toml").exists());

        apply_project(
            directory.path(),
            &config(BTreeSet::from([Platform::Web])),
            ApplyOptions {
                interactive: false,
                force: false,
                force_flake: false,
            },
        )
        .unwrap();
        assert!(directory.path().join("web/index.html").exists());
        assert!(!directory.path().join("android/Cargo.toml").exists());
        assert!(state_path(directory.path()).exists());
    }

    #[test]
    fn refuses_to_replace_user_changes_without_force() {
        let directory = tempdir().unwrap();
        apply_project(
            directory.path(),
            &config(BTreeSet::from([Platform::Web])),
            ApplyOptions {
                interactive: false,
                force: false,
                force_flake: false,
            },
        )
        .unwrap();
        fs::write(directory.path().join("web/index.html"), "custom").unwrap();
        let result = apply_project(
            directory.path(),
            &config(BTreeSet::from([Platform::Android])),
            ApplyOptions {
                interactive: false,
                force: false,
                force_flake: false,
            },
        );
        assert!(result.is_err());
        assert_eq!(
            fs::read_to_string(directory.path().join("web/index.html")).unwrap(),
            "custom"
        );
    }

    #[test]
    fn existing_flake_requires_explicit_overwrite() {
        let directory = tempdir().unwrap();
        fs::write(directory.path().join("flake.nix"), "custom flake").unwrap();
        let mut nix_config = config(BTreeSet::from([Platform::Web]));
        nix_config.build.backend = Backend::Nix;

        let result = apply_project(
            directory.path(),
            &nix_config,
            ApplyOptions {
                interactive: false,
                force: false,
                force_flake: false,
            },
        );
        assert!(result.is_err());
        assert_eq!(
            fs::read_to_string(directory.path().join("flake.nix")).unwrap(),
            "custom flake"
        );

        apply_project(
            directory.path(),
            &nix_config,
            ApplyOptions {
                interactive: false,
                force: false,
                force_flake: true,
            },
        )
        .unwrap();
        assert!(
            fs::read_to_string(directory.path().join("flake.nix"))
                .unwrap()
                .contains("rust-overlay")
        );
        assert!(
            fs::read_dir(directory.path())
                .unwrap()
                .filter_map(Result::ok)
                .any(|entry| entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with("flake.material-ui-backup-"))
        );
    }

    #[test]
    fn escapes_labels_for_each_generated_language() {
        let mut config = config(BTreeSet::from([Platform::Web, Platform::Android]));
        config.project.label = "A \"quoted\" <app>".into();
        let files = render_files(&config).unwrap();

        assert!(
            files["src/lib.rs"].contains(r#""A \"quoted\" <app>""#),
            "{}",
            files["src/lib.rs"]
        );
        assert!(files["web/index.html"].contains("A &quot;quoted&quot; &lt;app&gt;"));
        let android_manifest: toml::Value = toml::from_str(&files["android/Cargo.toml"]).unwrap();
        assert_eq!(
            android_manifest["package"]["metadata"]["android"]["application"]["label"].as_str(),
            Some("A \"quoted\" <app>")
        );
    }

    #[test]
    fn loads_android_system_fonts() {
        let files = render_files(&config(BTreeSet::from([Platform::Android]))).unwrap();

        assert!(files["src/lib.rs"].contains("material::android::system_fonts()"));
    }

    #[test]
    fn generates_navigation_menu_demo() {
        let files = render_files(&config(BTreeSet::from([Platform::Linux]))).unwrap();
        let app = &files["src/lib.rs"];

        assert!(app.contains("navigation::suite"));
        assert!(app.contains(".window_size(app.window_size)"));
        assert!(app.contains("iced::window::resize_events()"));
        assert!(app.contains(".toggle_menu_for_size("));
        assert!(!app.contains(".layout(navigation::AdaptiveLayout::NavigationRail)"));
        assert!(app.contains(".with_menu("));
        assert!(app.contains(".badge(\"3\")"));
    }
}
