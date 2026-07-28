mod build;
mod cli;
mod config;
mod flake;
mod generate;
mod ui;

use std::ffi::OsString;
use std::process::ExitCode;

use anyhow::Result;
use clap::Parser;

use crate::cli::{Cli, Command};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error:#}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse_from(normalize_cargo_subcommand_args(
        std::env::args_os().collect(),
    ));

    match cli.command {
        Command::Init(args) => generate::init(args),
        Command::Configure(args) => generate::configure(args),
        Command::Build(args) => build::run(&args),
        Command::Doctor(args) => build::doctor(&args),
    }
}

fn normalize_cargo_subcommand_args(mut args: Vec<OsString>) -> Vec<OsString> {
    if args.get(1).is_some_and(|arg| arg == "material-ui") {
        let _ = args.remove(1);
    }

    args
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_name_passed_by_cargo_external_subcommand() {
        let args = normalize_cargo_subcommand_args(
            ["cargo-material-ui", "material-ui", "doctor"]
                .into_iter()
                .map(OsString::from)
                .collect(),
        );

        assert_eq!(
            args,
            ["cargo-material-ui", "doctor"].map(OsString::from).to_vec()
        );
    }

    #[test]
    fn preserves_direct_binary_invocation() {
        let args = normalize_cargo_subcommand_args(
            ["cargo-material-ui", "doctor"]
                .into_iter()
                .map(OsString::from)
                .collect(),
        );

        assert_eq!(
            args,
            ["cargo-material-ui", "doctor"].map(OsString::from).to_vec()
        );
    }
}
