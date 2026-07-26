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

use dioxus_showcase_core::{ProviderDefinition, ShowcaseConfig, StoryDefinition};

use crate::{
    cli::BuildArgs,
    commands::load_config,
    discovery::{
        discover_component_source_files, discover_components, discover_providers,
        duplicate_story_ids,
    },
    scaffold::{showcase_app_dir, write_artifacts},
};

/// Runs the build command and optionally enters artifact watch mode.
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

/// Re-discovers stories/providers and rewrites all generated showcase artifacts.
pub fn rebuild_showcase_artifacts(config: &ShowcaseConfig) -> Result<usize, String> {
    let advisory = discover_advisory(config);
    write_artifacts(config, &advisory.stories, &advisory.providers)?;
    Ok(advisory.stories.len())
}

/// What a static scan of the entry crate found. Never authoritative.
pub struct AdvisoryDiscovery {
    pub stories: Vec<StoryDefinition>,
    pub providers: Vec<ProviderDefinition>,
}

/// Runs discovery for the manifest, downgrading every failure to a warning.
///
/// Discovery feeds diagnostics and the manifest, and nothing else: the application
/// gets its stories from the link-time registry. So a source file this scanner
/// cannot parse, or a story id it sees twice, must leave the manifest less useful —
/// never stop the build that would have worked anyway. `check` is where the same
/// conditions are reported as failures.
fn discover_advisory(config: &ShowcaseConfig) -> AdvisoryDiscovery {
    let mut stories = match discover_components(Path::new("."), config) {
        Ok(stories) => stories,
        Err(err) => {
            eprintln!("warning: showcase discovery failed, the manifest will be incomplete: {err}");
            Vec::new()
        }
    };
    stories.sort_by(|a, b| a.title.cmp(&b.title));

    let providers = match discover_providers(Path::new("."), config) {
        Ok(providers) => providers,
        Err(err) => {
            eprintln!("warning: showcase provider discovery failed: {err}");
            Vec::new()
        }
    };

    for id in duplicate_story_ids(&stories) {
        eprintln!(
            "warning: more than one story claims the id '{id}'. \
             The showcase will render a warning banner and route '/component/{id}' to \
             the first of them. Run `dioxus-showcase check` for details."
        );
    }

    AdvisoryDiscovery { stories, providers }
}

/// Polls source timestamps and rebuilds artifacts whenever the newest file changes.
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

/// Returns the latest modification time across discovered sources and the config file.
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
