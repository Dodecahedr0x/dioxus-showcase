//! `dioxus-showcase check` — the diagnostic command.
//!
//! Everything here is a static read of the config and the sources. Nothing is
//! compiled and nothing is written, which is what makes `check` cheap enough to run
//! on every save and is the whole reason static discovery still exists now that the
//! runtime reads a link-time registry instead.
//!
//! `check` is also the one place discovery problems are *errors*. `build` warns and
//! carries on, because a scan that disagrees with the macros must not stop a build
//! that would have worked; but a command whose only output is diagnostics has to
//! exit non-zero when it finds something, or nothing in CI would ever notice.
use std::path::Path;

use dioxus_showcase_core::StoryDefinition;

use crate::{
    commands::load_config,
    discovery::{discover_components, discover_providers, duplicate_story_ids},
    scaffold::showcase_app_dir,
};

/// Runs a lightweight validation pass without generating new artifacts.
pub fn cmd_check() -> Result<(), String> {
    // Unknown keys are rejected here, by the config parser's `deny_unknown_fields`:
    // a typo'd key would otherwise be silently ignored and the user would be left
    // wondering why their setting had no effect.
    let config = load_config()?;

    let mut stories = discover_components(Path::new("."), &config)?;
    stories.sort_by(|a, b| a.title.cmp(&b.title));
    let providers = discover_providers(Path::new("."), &config)?;

    let app_dir = showcase_app_dir(&config);
    if !app_dir.exists() {
        return Err(format!(
            "showcase app crate not found at {}. Run `dioxus-showcase init` first.",
            app_dir.display()
        ));
    }

    println!("Config file found: DioxusShowcase.toml");
    println!("Showcase crate: {}", app_dir.display());
    println!("Discovered {} annotated components.", stories.len());
    println!("Discovered {} providers.", providers.len());

    for profile in profiles_missing_lto(&app_dir) {
        eprintln!(
            "warning: [profile.{profile}] in {}/Cargo.toml does not set `lto`. \
             On wasm the component crate's registrations are dropped at link time \
             without it, and the showcase renders EMPTY with no error. \
             Add `lto = \"thin\"` under [profile.dev] and `lto = true` under [profile.release].",
            app_dir.display()
        );
    }

    let duplicates = duplicate_story_ids(&stories);
    if !duplicates.is_empty() {
        return Err(describe_duplicates(&duplicates, &stories));
    }

    println!("Checks passed.");
    Ok(())
}

/// Returns the build profiles in the showcase app's manifest that do not set `lto`.
///
/// This exists because of a failure with no other symptom. `main.rs` names the
/// component crate only through `use showcase_entry as _;`, and on wasm32 that
/// import does not cause the linker to select the crate's member out of its rlib
/// archive — nothing in the binary references a symbol inside it. Every story then
/// disappears, and the app still builds, still launches, and shows nothing. LTO
/// merges the crate graph before that selection happens, so the generated manifest
/// pins it; a user who removes those lines gets this warning instead of silence.
///
/// A missing or unreadable manifest yields nothing: `check` is not the command that
/// reports a missing scaffold, and a scan this crude must never invent a problem.
fn profiles_missing_lto(app_dir: &Path) -> Vec<&'static str> {
    let Ok(manifest) = std::fs::read_to_string(app_dir.join("Cargo.toml")) else {
        return Vec::new();
    };

    ["dev", "release"].into_iter().filter(|profile| !profile_sets_lto(&manifest, profile)).collect()
}

/// Reports whether `[profile.<name>]` in a manifest assigns `lto`.
fn profile_sets_lto(manifest: &str, profile: &str) -> bool {
    let header = format!("[profile.{profile}]");
    let mut inside = false;

    for raw_line in manifest.lines() {
        let line = raw_line.trim();
        if line.starts_with('[') && line.ends_with(']') {
            inside = line == header;
            continue;
        }
        if inside && line.split('=').next().is_some_and(|key| key.trim() == "lto") {
            return true;
        }
    }

    false
}

/// Renders one diagnostic line per colliding id, naming every story involved.
///
/// The id alone is not actionable — two stories collide because their *titles*
/// slugify to the same thing, and the user has to see which titles those are.
fn describe_duplicates(duplicates: &[String], stories: &[StoryDefinition]) -> String {
    let mut message = format!(
        "{} duplicate showcase id(s) found. Story ids come from the title, so two \
         titles that differ only in punctuation or case collide:",
        duplicates.len()
    );

    for id in duplicates {
        let claimants: Vec<String> = stories
            .iter()
            .filter(|story| &story.id == id)
            .map(|story| format!("'{}' ({})", story.title, story.module_path))
            .collect();
        message.push_str(&format!("\n  duplicate id '{id}' claimed by {}", claimants.join(", ")));
    }

    message
}

#[cfg(test)]
mod tests {
    use super::{describe_duplicates, profile_sets_lto, profiles_missing_lto};
    use dioxus_showcase_core::StoryDefinition;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(prefix: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be monotonic")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("{prefix}-{unique}"));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    #[test]
    fn lto_is_detected_only_inside_the_profile_that_declares_it() {
        let manifest = "[package]\n\
                        name = \"app\"\n\
                        lto = true\n\
                        \n\
                        [profile.dev]\n\
                        lto = \"thin\"\n\
                        \n\
                        [profile.release]\n\
                        strip = true\n";

        assert!(profile_sets_lto(manifest, "dev"));
        // `lto` in `[package]` must not be mistaken for a profile setting, and the
        // release profile here really is missing it.
        assert!(!profile_sets_lto(manifest, "release"));
        assert!(!profile_sets_lto("", "dev"));
    }

    #[test]
    fn a_manifest_without_lto_names_both_profiles() {
        let dir = temp_dir("dioxus-showcase-check-lto");
        std::fs::write(dir.join("Cargo.toml"), "[package]\nname = \"app\"\n")
            .expect("write manifest");

        assert_eq!(profiles_missing_lto(&dir), vec!["dev", "release"]);

        std::fs::write(
            dir.join("Cargo.toml"),
            "[profile.dev]\nlto = \"thin\"\n\n[profile.release]\nlto = true\nstrip = true\n",
        )
        .expect("rewrite manifest");
        assert!(profiles_missing_lto(&dir).is_empty());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_missing_manifest_is_not_reported_as_a_missing_profile() {
        let dir = temp_dir("dioxus-showcase-check-no-manifest");

        assert!(profiles_missing_lto(&dir).is_empty());

        let _ = std::fs::remove_dir_all(&dir);
    }

    fn story(id: &str, title: &str, module_path: &str) -> StoryDefinition {
        StoryDefinition {
            id: id.to_owned(),
            title: title.to_owned(),
            source_path: "/tmp/lib.rs".to_owned(),
            module_path: module_path.to_owned(),
            renderer_symbol: "__dioxus_showcase_render__demo".to_owned(),
            tags: vec![],
        }
    }

    #[test]
    fn duplicate_diagnostics_name_every_colliding_story() {
        let stories = vec![
            story("atoms-button", "Atoms/Button", "ui::Button"),
            story("atoms-button", "Atoms Button", "ui::button_alt"),
            story("atoms-card", "Atoms/Card", "ui::Card"),
        ];

        let message = describe_duplicates(&["atoms-button".to_owned()], &stories);

        assert!(message.contains("1 duplicate showcase id(s) found"), "{message}");
        assert!(message.contains("duplicate id 'atoms-button'"), "{message}");
        assert!(message.contains("'Atoms/Button' (ui::Button)"), "{message}");
        assert!(message.contains("'Atoms Button' (ui::button_alt)"), "{message}");
        assert!(!message.contains("atoms-card"), "{message}");
    }
}
