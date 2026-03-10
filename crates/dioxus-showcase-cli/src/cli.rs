use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "dioxus-showcase")]
#[command(about = "Dioxus showcase CLI")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    Init,
    Dev,
    Build(BuildArgs),
    Check,
    Doctor,
}

#[derive(Args, Debug, Clone, Default)]
pub struct BuildArgs {
    #[arg(long, help = "Rebuild showcase artifacts when annotated component sources change")]
    pub watch: bool,
}
