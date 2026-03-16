use std::{
    net::TcpListener,
    process::Command,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

use crate::{
    build::{rebuild_showcase_artifacts, watch_and_rebuild},
    commands::load_config,
    scaffold::showcase_app_dir,
};

/// Rebuilds artifacts, starts a watcher thread, and launches `dx serve`.
pub fn cmd_dev() -> Result<(), String> {
    let config = load_config()?;
    let app_dir = showcase_app_dir(&config);
    let dev_port = find_available_port(&config.dev.host, config.dev.port, 20)?;

    let initial_count = rebuild_showcase_artifacts(&config)?;

    println!("Starting dioxus-showcase app...");
    println!("Discovered {} showcase components.", initial_count);
    println!("App directory: {}", app_dir.display());
    println!(
        "Launching in {}: dx serve --web --port {} --addr {}",
        app_dir.display(),
        dev_port,
        config.dev.host
    );
    println!("Watching component crate for changes (auto-regenerates showcase routes/runtime).");
    println!("Press Ctrl+C to stop.");

    let stop_flag = Arc::new(AtomicBool::new(false));
    let watcher_stop = Arc::clone(&stop_flag);
    let watcher_config = config.clone();
    let watcher = thread::spawn(move || watch_and_rebuild(watcher_config, watcher_stop));

    // wait for the watcher to start
    thread::sleep(Duration::from_secs(1));

    let status = Command::new("dx")
        .arg("serve")
        .arg("--web")
        .arg("--port")
        .arg(dev_port.to_string())
        .arg("--addr")
        .arg(&config.dev.host)
        .current_dir(&app_dir)
        .status()
        .map_err(|err| {
            format!(
                "failed to run `dx serve`. Ensure Dioxus CLI is installed and available in PATH: {err}"
            )
        })?;

    stop_flag.store(true, Ordering::Relaxed);
    let _ = watcher.join();

    if status.success() {
        Ok(())
    } else {
        Err(format!("`dx serve` exited with status {status}"))
    }
}

/// Finds the first bindable port within a bounded range on the requested host.
fn find_available_port(host: &str, preferred_port: u16, max_attempts: u16) -> Result<u16, String> {
    let upper = preferred_port.saturating_add(max_attempts);
    for port in preferred_port..=upper {
        let bind_addr = format!("{host}:{port}");
        if TcpListener::bind(&bind_addr).is_ok() {
            return Ok(port);
        }
    }

    Err(format!("no available port found for host {host} in range {preferred_port}-{upper}"))
}
