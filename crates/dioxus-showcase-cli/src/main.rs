mod build;
mod check;
mod cli;
mod commands;
mod dev;
mod discovery;
mod scaffold;
mod templates;

use clap::{CommandFactory, Parser};

use crate::cli::Cli;

/// Starts the CLI process and exits non-zero when command execution fails.
fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

/// Parses arguments, prints help when needed, and dispatches to the command layer.
fn run() -> Result<(), String> {
    let cli = Cli::parse();
    if cli.command.is_none() {
        Cli::command().print_help().map_err(|err| format!("failed to print help: {err}"))?;
        println!();
        return Ok(());
    }

    commands::run(cli.command)
}
