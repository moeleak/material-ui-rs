mod build;
mod cli;
mod config;
mod flake;
mod generate;
mod ui;

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
    let cli = Cli::parse();

    match cli.command {
        Command::Init(args) => generate::init(args),
        Command::Configure(args) => generate::configure(args),
        Command::Build(args) => build::run(&args),
        Command::Doctor(args) => build::doctor(&args),
    }
}
