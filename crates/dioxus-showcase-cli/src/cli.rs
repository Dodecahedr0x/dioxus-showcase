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
    #[command(about = "Build a deployable static website of the showcased components")]
    Export(ExportArgs),
    Check,
    Doctor,
}

#[derive(Args, Debug, Clone, Default)]
pub struct BuildArgs {
    #[arg(long, help = "Rebuild showcase artifacts when annotated component sources change")]
    pub watch: bool,
}

#[derive(Args, Debug, Clone, Default)]
pub struct ExportArgs {
    #[arg(
        long,
        value_name = "DIR",
        help = "Directory to write the static site into [default: <build.out_dir>/site]"
    )]
    pub out_dir: Option<String>,

    #[arg(
        long,
        value_name = "PATH",
        help = "Public sub-path the site is served from, e.g. /my-repo [default: build.base_path]"
    )]
    pub base_path: Option<String>,

    #[arg(long, help = "Build the site in debug mode instead of release")]
    pub debug: bool,
}
