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

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let cli = Cli::parse();
    if cli.command.is_none() {
        Cli::command().print_help().map_err(|err| format!("failed to print help: {err}"))?;
        println!();
        return Ok(());
    }

    commands::run(cli.command)
}
