//! Entry point for the `dioxus-showcase` binary.

/// Runs the CLI and exits non-zero when a command fails.
fn main() {
    if let Err(err) = dioxus_showcase_cli::run() {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}
