//! Command implementations for the `dioxus-showcase` CLI.
//!
//! The binary in `src/main.rs` only calls [`run`]. Everything else is private, so the
//! command surface is defined by `src/cli.rs` and nothing else.

mod build;
mod check;
mod cli;
mod commands;
mod dev;
mod discovery;
mod export;
mod scaffold;
mod templates;

use clap::{CommandFactory, Parser};

use crate::cli::Cli;

/// Parses arguments, prints help when needed, and dispatches to the command layer.
pub fn run() -> Result<(), String> {
    let cli = Cli::parse();
    if cli.command.is_none() {
        Cli::command().print_help().map_err(|err| format!("failed to print help: {err}"))?;
        println!();
        return Ok(());
    }

    commands::run(cli.command)
}
