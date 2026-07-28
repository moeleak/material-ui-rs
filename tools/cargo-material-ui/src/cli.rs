use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

use crate::config::{Backend, Platform};

#[derive(Debug, Parser)]
#[command(
    name = "cargo-material-ui",
    bin_name = "cargo material-ui",
    version,
    about = "Create and build multi-platform material-ui-rs applications"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Create a material-ui-rs application in a new directory.
    New(NewArgs),
    /// Initialize the current directory as a material-ui-rs application.
    Init(InitArgs),
    /// Change the platforms and toolchain of an existing project.
    Configure(ConfigureArgs),
    /// Build every configured platform, or a selected subset.
    Build(BuildArgs),
    /// Check the configured build environment.
    Doctor(DoctorArgs),
}

#[derive(Debug, Clone, Args)]
pub struct InitArgs {
    #[command(flatten)]
    pub project: ProjectArgs,
}

#[derive(Debug, Clone, Args)]
pub struct NewArgs {
    /// Directory to create. Defaults to a prompt in interactive terminals.
    pub path: Option<PathBuf>,
    #[command(flatten)]
    pub project: ProjectArgs,
}

#[derive(Debug, Clone, Args)]
pub struct ProjectArgs {
    /// Disable prompts. All required values must be supplied as flags.
    #[arg(long)]
    pub non_interactive: bool,
    /// Cargo package name.
    #[arg(long)]
    pub name: Option<String>,
    /// Human-readable application name.
    #[arg(long)]
    pub label: Option<String>,
    /// Reverse-DNS application identifier.
    #[arg(long)]
    pub app_id: Option<String>,
    /// Build environment.
    #[arg(long, value_enum)]
    pub backend: Option<Backend>,
    /// Platforms to configure.
    #[arg(long, value_enum, value_delimiter = ',')]
    pub platform: Vec<Platform>,
    /// Disable spinners and transition effects.
    #[arg(long)]
    pub no_animations: bool,
    /// Override an existing flake.nix after making a timestamped backup.
    #[arg(long)]
    pub force_flake_overwrite: bool,
}

#[derive(Debug, Clone, Args)]
#[allow(clippy::struct_excessive_bools)]
pub struct ConfigureArgs {
    /// Project directory.
    #[arg(default_value = ".")]
    pub path: PathBuf,
    /// Disable prompts.
    #[arg(long)]
    pub non_interactive: bool,
    /// Replace the configured backend.
    #[arg(long, value_enum)]
    pub backend: Option<Backend>,
    /// Add a platform.
    #[arg(long, value_enum, value_delimiter = ',')]
    pub add_platform: Vec<Platform>,
    /// Remove a platform.
    #[arg(long, value_enum, value_delimiter = ',')]
    pub remove_platform: Vec<Platform>,
    /// Disable spinners and transition effects.
    #[arg(long)]
    pub no_animations: bool,
    /// Override conflicting generated files.
    #[arg(long)]
    pub force: bool,
    /// Override an existing flake.nix after making a timestamped backup.
    #[arg(long)]
    pub force_flake_overwrite: bool,
}

#[derive(Debug, Clone, Args)]
pub struct BuildArgs {
    /// Project directory.
    #[arg(default_value = ".")]
    pub path: PathBuf,
    /// Build only these configured platforms.
    #[arg(long, value_enum, value_delimiter = ',')]
    pub platform: Vec<Platform>,
    /// Build optimized artifacts.
    #[arg(long)]
    pub release: bool,
    /// Internal flag used after entering a generated Nix environment.
    #[arg(long, hide = true)]
    pub prepared: bool,
}

#[derive(Debug, Clone, Args)]
pub struct DoctorArgs {
    /// Project directory.
    #[arg(default_value = ".")]
    pub path: PathBuf,
}

#[cfg(test)]
mod tests {
    use super::{Cli, Command};
    use clap::Parser;
    use std::path::PathBuf;

    #[test]
    fn new_accepts_a_project_directory() {
        let cli = Cli::try_parse_from(["cargo-material-ui", "new", "material-app"]).unwrap();

        let Command::New(args) = cli.command else {
            panic!("expected the new command");
        };
        assert_eq!(args.path, Some(PathBuf::from("material-app")));
    }

    #[test]
    fn init_does_not_accept_a_project_directory() {
        assert!(Cli::try_parse_from(["cargo-material-ui", "init"]).is_ok());
        assert!(Cli::try_parse_from(["cargo-material-ui", "init", "material-app"]).is_err());
    }
}
