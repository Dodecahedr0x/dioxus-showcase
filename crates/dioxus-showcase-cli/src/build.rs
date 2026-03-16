use std::{
    fs,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::{Duration, SystemTime},
};

use dioxus_showcase_core::ShowcaseConfig;

use crate::{
    cli::BuildArgs,
    commands::load_config,
    discovery::{
        discover_component_source_files, discover_components, discover_providers,
        validate_component_ids,
    },
    scaffold::{showcase_app_dir, write_artifacts},
};

pub fn cmd_build(args: BuildArgs) -> Result<(), String> {
    let config = load_config()?;
    let component_count = rebuild_showcase_artifacts(&config)?;

    let out_dir = PathBuf::from(&config.build.out_dir);
    println!("Wrote showcase artifacts to {}", out_dir.display());
    println!("Wrote manifest to {}", out_dir.join("showcase.manifest.json").display());
    println!(
        "Wrote generated routes/runtime to {}",
        showcase_app_dir(&config).join("src/generated.rs").display()
    );
    println!("Discovered {} annotated components.", component_count);

    if args.watch {
        println!(
            "Watching component crate for changes (auto-regenerates showcase routes/runtime)."
        );
        println!("Press Ctrl+C to stop.");
        watch_and_rebuild(config, Arc::new(AtomicBool::new(false)));
    }

    Ok(())
}

pub fn rebuild_showcase_artifacts(config: &ShowcaseConfig) -> Result<usize, String> {
    let mut components = discover_components(Path::new("."), config)?;
    let providers = discover_providers(Path::new("."), config)?;
    components.sort_by(|a, b| a.title.cmp(&b.title));
    validate_component_ids(&components)?;
    write_artifacts(config, &components, &providers)?;
    Ok(components.len())
}

pub fn watch_and_rebuild(config: ShowcaseConfig, stop: Arc<AtomicBool>) {
    let mut last_stamp = latest_source_stamp(&config).ok().flatten();

    while !stop.load(Ordering::Relaxed) {
        thread::sleep(Duration::from_millis(700));
        let current_stamp = latest_source_stamp(&config).ok().flatten();
        if current_stamp <= last_stamp {
            continue;
        }

        match rebuild_showcase_artifacts(&config) {
            Ok(count) => {
                println!("showcase updated: {} components", count);
                last_stamp = current_stamp;
            }
            Err(err) => {
                eprintln!("showcase update failed: {err}");
                last_stamp = current_stamp;
            }
        }
    }
}

fn latest_source_stamp(config: &ShowcaseConfig) -> Result<Option<SystemTime>, String> {
    let mut files = discover_component_source_files(Path::new("."), config)?;
    files.push(PathBuf::from("DioxusShowcase.toml"));

    let mut latest: Option<SystemTime> = None;
    for file in files {
        let Ok(metadata) = fs::metadata(&file) else {
            continue;
        };
        let Ok(modified) = metadata.modified() else {
            continue;
        };

        latest = match latest {
            Some(current) if current >= modified => Some(current),
            _ => Some(modified),
        };
    }

    Ok(latest)
}
