use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

use dioxus_showcase_core::{
    ShowcaseBuildConfig, ShowcaseConfig, ShowcaseDevConfig, ShowcaseProjectConfig,
};

use crate::build::cmd_build;
use crate::check::cmd_check;
use crate::cli::Command;
use crate::dev::cmd_dev;
use crate::scaffold::ensure_showcase_app_scaffold;
use crate::scaffold::showcase_app_dir;

pub fn run(command: Option<Command>) -> Result<(), String> {
    match command {
        Some(Command::Init) => cmd_init(),
        Some(Command::Dev) => cmd_dev(),
        Some(Command::Build(args)) => cmd_build(args),
        Some(Command::Check) => cmd_check(),
        Some(Command::Doctor) => {
            println!("dioxus-showcase doctor");
            println!("platform: {} {}", env::consts::OS, env::consts::ARCH);
            println!("cwd: {}", env::current_dir().map_err(|err| err.to_string())?.display());
            Ok(())
        }
        None => Ok(()),
    }
}

pub fn load_config() -> Result<ShowcaseConfig, String> {
    let config_path = Path::new("DioxusShowcase.toml");
    if !config_path.exists() {
        return Err("DioxusShowcase.toml not found. Run `dioxus-showcase init` first.".to_owned());
    }
    ShowcaseConfig::from_toml_file(config_path)
}

pub fn cmd_init() -> Result<(), String> {
    let existing = load_config().ok();
    let config = prompt_for_config(existing)?;

    fs::write("DioxusShowcase.toml", config.as_toml_string())
        .map_err(|err| format!("failed to write DioxusShowcase.toml: {err}"))?;

    let app_dir = showcase_app_dir(&config);
    ensure_showcase_app_scaffold(&config)?;

    println!("Wrote DioxusShowcase.toml.");
    println!("Created showcase app crate at {}", app_dir.display());

    println!("Run `dioxus-showcase dev` to launch with live updates.");
    Ok(())
}

fn prompt_for_config(existing: Option<ShowcaseConfig>) -> Result<ShowcaseConfig, String> {
    let cwd_name = env::current_dir()
        .ok()
        .and_then(|path| path.file_name().map(|s| s.to_string_lossy().to_string()))
        .unwrap_or_else(|| "my-ui".to_owned());

    let defaults = existing.unwrap_or_else(|| ShowcaseConfig {
        project: ShowcaseProjectConfig {
            name: cwd_name,
            entry_crate: default_entry_crate(),
            showcase_crate: "showcase".to_owned(),
        },
        dev: ShowcaseDevConfig { port: 6111, host: "127.0.0.1".to_owned() },
        build: ShowcaseBuildConfig {
            out_dir: "target/showcase".to_owned(),
            base_path: "/".to_owned(),
        },
    });

    println!("Configure DioxusShowcase.toml (press Enter to keep default):");
    let project_name = prompt("Project name", &defaults.project.name)?;
    let entry_crate = prompt("Component/UI crate path", &defaults.project.entry_crate)?;
    let showcase_crate = prompt("Showcase crate path", &defaults.project.showcase_crate)?;
    let host = prompt("Dev host", &defaults.dev.host)?;
    let port = prompt("Dev port", &defaults.dev.port.to_string())?
        .parse::<u16>()
        .map_err(|_| "invalid port provided".to_owned())?;
    let out_dir = prompt("Build output dir", &defaults.build.out_dir)?;

    Ok(ShowcaseConfig {
        project: ShowcaseProjectConfig { name: project_name, entry_crate, showcase_crate },
        dev: ShowcaseDevConfig { port, host },
        build: ShowcaseBuildConfig { out_dir, base_path: defaults.build.base_path },
    })
}

fn default_entry_crate() -> String {
    if Path::new("src").exists() && Path::new("Cargo.toml").exists() {
        return ".".to_owned();
    }

    if Path::new("examples/basic/src").exists() {
        return "examples/basic".to_owned();
    }

    "web".to_owned()
}

fn prompt(label: &str, default: &str) -> Result<String, String> {
    print!("- {label} [{default}]: ");
    io::stdout().flush().map_err(|err| format!("failed to flush stdout: {err}"))?;

    let mut line = String::new();
    io::stdin().read_line(&mut line).map_err(|err| format!("failed to read stdin: {err}"))?;
    let trimmed = line.trim();
    if trimmed.is_empty() {
        Ok(default.to_owned())
    } else {
        Ok(trimmed.to_owned())
    }
}
