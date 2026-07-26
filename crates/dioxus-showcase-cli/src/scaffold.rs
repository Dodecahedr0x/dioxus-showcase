use std::fs;
use std::path::{Path, PathBuf};

use dioxus_showcase_core::{
    ProviderDefinition, ShowcaseConfig, StoryDefinition, StoryManifest, MANIFEST_SCHEMA_VERSION,
};

use crate::templates;

/// The stylesheet older versions of the CLI generated into the showcase app.
///
/// The shell's own CSS ships inside `dioxus-showcase-ui` now. A copy left over from
/// an earlier build would otherwise be picked up as if the user had written it and
/// linked into the entry point, where it would fight the shell's real stylesheet.
const LEGACY_SHELL_STYLESHEET: &str = "showcase.css";

/// Returns the configured path of the generated showcase application crate.
pub fn showcase_app_dir(config: &ShowcaseConfig) -> PathBuf {
    PathBuf::from(&config.project.showcase_crate)
}

/// Writes the advisory manifest and the generated runtime, scaffolding the app first.
pub fn write_artifacts(
    config: &ShowcaseConfig,
    stories: &[StoryDefinition],
    providers: &[ProviderDefinition],
) -> Result<PathBuf, String> {
    let out_dir = PathBuf::from(&config.build.out_dir);
    fs::create_dir_all(&out_dir)
        .map_err(|err| format!("failed to create {}: {err}", out_dir.display()))?;

    let manifest_path = out_dir.join("showcase.manifest.json");
    fs::write(&manifest_path, story_manifest(stories).to_json())
        .map_err(|err| format!("failed to create {}: {err}", manifest_path.display()))?;

    ensure_showcase_app_scaffold(config)?;

    let generated_path = showcase_app_dir(config).join("src/generated.rs");
    let generation = stable_generation_token(stories, providers);
    let generated_runtime = templates::render_generated_runtime_rs(&generation)?;
    fs::write(&generated_path, generated_runtime)
        .map_err(|err| format!("failed to create {}: {err}", generated_path.display()))?;

    Ok(out_dir)
}

/// Ensures the generated showcase app directory and seed files exist.
///
/// Everything here is idempotent, and two of the files are **write-once**:
///
/// - `Cargo.toml`, because a user adds their own dependencies to it.
/// - `src/main.rs`, because it is the application's entry point and the shell it
///   launches is a library now. Regenerating it would silently discard whatever the
///   user put there, which is the failure this whole refactor exists to remove.
///
/// A consequence worth knowing: a project scaffolded before a stylesheet was added
/// to the entry crate will not pick up the new `document::Stylesheet` line, because
/// `main.rs` is never rewritten. Adding the line by hand is the fix.
pub fn ensure_showcase_app_scaffold(config: &ShowcaseConfig) -> Result<(), String> {
    let app_dir = showcase_app_dir(config);
    let src_dir = app_dir.join("src");
    let assets_dir = app_dir.join("assets");
    fs::create_dir_all(&src_dir)
        .map_err(|err| format!("failed to create {}: {err}", src_dir.display()))?;
    fs::create_dir_all(&assets_dir)
        .map_err(|err| format!("failed to create {}: {err}", assets_dir.display()))?;

    let cargo_toml_path = app_dir.join("Cargo.toml");
    if !cargo_toml_path.exists() {
        let cargo_toml = templates::render_showcase_app_cargo_toml(config)?;
        fs::write(&cargo_toml_path, cargo_toml)
            .map_err(|err| format!("failed to create {}: {err}", cargo_toml_path.display()))?;
    }

    let dioxus_toml_path = app_dir.join("Dioxus.toml");
    let dioxus_toml = templates::render_showcase_app_dioxus_toml(config)?;
    fs::write(&dioxus_toml_path, dioxus_toml)
        .map_err(|err| format!("failed to create {}: {err}", dioxus_toml_path.display()))?;

    let stylesheets = sync_entry_assets_and_collect_stylesheets(config)?;

    let main_rs_path = src_dir.join("main.rs");
    if !main_rs_path.exists() {
        let main_rs =
            templates::render_showcase_app_main_rs(&config.build.base_path, &stylesheets)?;
        fs::write(&main_rs_path, main_rs)
            .map_err(|err| format!("failed to create {}: {err}", main_rs_path.display()))?;
    }

    let generated_rs_path = src_dir.join("generated.rs");
    if !generated_rs_path.exists() {
        let generated_rs = templates::render_generated_runtime_rs("initial")?;
        fs::write(&generated_rs_path, generated_rs)
            .map_err(|err| format!("failed to create {}: {err}", generated_rs_path.display()))?;
    }

    Ok(())
}

/// Builds the advisory manifest for the discovered stories.
fn story_manifest(stories: &[StoryDefinition]) -> StoryManifest {
    let mut manifest = StoryManifest::new(MANIFEST_SCHEMA_VERSION);
    for story in stories {
        manifest.add_story(story.clone());
    }
    manifest
}

/// Copies entry assets into the showcase app and returns all CSS asset URLs.
fn sync_entry_assets_and_collect_stylesheets(
    config: &ShowcaseConfig,
) -> Result<Vec<String>, String> {
    let entry_assets_dir = Path::new(&config.project.entry_crate).join("assets");
    let showcase_assets_dir = showcase_app_dir(config).join("assets");

    fs::create_dir_all(&showcase_assets_dir)
        .map_err(|err| format!("failed to create {}: {err}", showcase_assets_dir.display()))?;

    remove_legacy_shell_stylesheet(&entry_assets_dir, &showcase_assets_dir)?;

    if entry_assets_dir.exists() {
        copy_dir_recursive(&entry_assets_dir, &showcase_assets_dir)?;
    }

    let mut stylesheets = Vec::new();
    collect_stylesheets(&showcase_assets_dir, &showcase_assets_dir, &mut stylesheets)?;
    stylesheets.sort();
    Ok(stylesheets)
}

/// Deletes the shell stylesheet earlier CLI versions generated, if the user has none.
///
/// Only a file the CLI itself would have written is removed: if the entry crate ships
/// its own `assets/showcase.css`, that one is the user's and is copied over as usual.
fn remove_legacy_shell_stylesheet(
    entry_assets_dir: &Path,
    showcase_assets_dir: &Path,
) -> Result<(), String> {
    if entry_assets_dir.join(LEGACY_SHELL_STYLESHEET).exists() {
        return Ok(());
    }

    let stale = showcase_assets_dir.join(LEGACY_SHELL_STYLESHEET);
    if !stale.exists() {
        return Ok(());
    }

    fs::remove_file(&stale).map_err(|err| format!("failed to remove {}: {err}", stale.display()))
}

/// Computes a deterministic token representing the current generated runtime inputs.
fn stable_generation_token(
    stories: &[StoryDefinition],
    providers: &[ProviderDefinition],
) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in story_manifest(stories).to_json().bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    for provider in providers {
        for byte in provider.module_path.bytes() {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x100000001b3);
        }
        for byte in provider.wrap_symbol.bytes() {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x100000001b3);
        }
        for byte in provider.order.to_le_bytes() {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x100000001b3);
        }
    }

    format!("manifest-{hash:016x}")
}

/// Recursively copies one directory tree into another.
pub fn copy_dir_recursive(from: &Path, to: &Path) -> Result<(), String> {
    fs::create_dir_all(to).map_err(|err| format!("failed to create {}: {err}", to.display()))?;

    for entry in
        fs::read_dir(from).map_err(|err| format!("failed to read {}: {err}", from.display()))?
    {
        let entry =
            entry.map_err(|err| format!("failed to read {} entry: {err}", from.display()))?;
        let source_path = entry.path();
        let destination_path = to.join(entry.file_name());
        let file_type = entry.file_type().map_err(|err| {
            format!("failed to determine file type for {}: {err}", source_path.display())
        })?;

        if file_type.is_dir() {
            copy_dir_recursive(&source_path, &destination_path)?;
        } else if file_type.is_file() {
            if let Some(parent) = destination_path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
            }
            fs::copy(&source_path, &destination_path).map_err(|err| {
                format!(
                    "failed to copy {} to {}: {err}",
                    source_path.display(),
                    destination_path.display()
                )
            })?;
        }
    }

    Ok(())
}

/// Collects stylesheet asset URLs relative to the generated app `assets/` directory.
fn collect_stylesheets(
    root: &Path,
    current: &Path,
    stylesheets: &mut Vec<String>,
) -> Result<(), String> {
    for entry in fs::read_dir(current)
        .map_err(|err| format!("failed to read {}: {err}", current.display()))?
    {
        let entry =
            entry.map_err(|err| format!("failed to read {} entry: {err}", current.display()))?;
        let path = entry.path();
        let file_type = entry.file_type().map_err(|err| {
            format!("failed to determine file type for {}: {err}", path.display())
        })?;

        if file_type.is_dir() {
            collect_stylesheets(root, &path, stylesheets)?;
            continue;
        }

        if !file_type.is_file() || path.extension().and_then(|ext| ext.to_str()) != Some("css") {
            continue;
        }

        let relative = path.strip_prefix(root).map_err(|err| {
            format!(
                "failed to compute stylesheet path for {} relative to {}: {err}",
                path.display(),
                root.display()
            )
        })?;

        let relative = relative.to_string_lossy().replace(std::path::MAIN_SEPARATOR, "/");
        stylesheets.push(format!("/assets/{relative}"));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use dioxus_showcase_core::{
        ProviderDefinition, ShowcaseConfig, StoryDefinition, MANIFEST_SCHEMA_VERSION,
    };

    use super::{ensure_showcase_app_scaffold, stable_generation_token, write_artifacts};

    const GOLDEN_MANIFEST: &str = include_str!("testdata/build_golden_manifest.json");
    const GOLDEN_GENERATED_RS: &str = include_str!("testdata/build_golden_generated.rs");

    fn temp_dir(prefix: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be monotonic")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("{prefix}-{unique}"));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    /// Builds a project on disk and returns `(root, config)`.
    fn project(prefix: &str) -> (PathBuf, ShowcaseConfig) {
        let dir = temp_dir(prefix);
        let entry_dir = dir.join("web");
        let showcase_dir = dir.join("showcase");
        std::fs::create_dir_all(entry_dir.join("src")).expect("create entry src");
        std::fs::create_dir_all(&showcase_dir).expect("create showcase dir");
        std::fs::write(
            entry_dir.join("Cargo.toml"),
            "[package]\nname = \"web\"\nversion = \"0.1.0\"\n",
        )
        .expect("write entry cargo");

        let mut config = ShowcaseConfig::default();
        config.project.entry_crate = entry_dir.to_string_lossy().to_string();
        config.project.showcase_crate = showcase_dir.to_string_lossy().to_string();
        config.build.out_dir = dir.join("target/showcase").to_string_lossy().to_string();

        (dir, config)
    }

    fn button_story() -> StoryDefinition {
        StoryDefinition {
            id: "atoms-button".to_owned(),
            title: "Atoms/Button".to_owned(),
            source_path: "/workspace/src/button.rs".to_owned(),
            module_path: "button_variants::Button".to_owned(),
            renderer_symbol: "__dioxus_showcase_render__Button".to_owned(),
            tags: vec!["atoms".to_owned()],
        }
    }

    #[test]
    fn write_artifacts_never_overwrites_an_existing_main_rs() {
        let (dir, config) = project("dioxus-showcase-write-once");
        let main_rs_path = PathBuf::from(&config.project.showcase_crate).join("src/main.rs");
        std::fs::create_dir_all(main_rs_path.parent().expect("src dir")).expect("create src");
        std::fs::write(&main_rs_path, "// mine, thanks").expect("write user main");

        write_artifacts(&config, &[button_story()], &[]).expect("write artifacts");

        assert_eq!(std::fs::read_to_string(&main_rs_path).expect("read main"), "// mine, thanks");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn write_artifacts_seeds_main_rs_with_the_entry_crate_stylesheets() {
        let (dir, config) = project("dioxus-showcase-seed-main");
        let entry_dir = PathBuf::from(&config.project.entry_crate);
        std::fs::create_dir_all(entry_dir.join("assets/styles")).expect("create entry assets");
        std::fs::write(entry_dir.join("assets/app.css"), "body { color: red; }")
            .expect("write app css");
        std::fs::write(entry_dir.join("assets/styles/tailwind.css"), ".btn { display: flex; }")
            .expect("write tailwind css");

        write_artifacts(&config, &[button_story()], &[]).expect("write artifacts");

        let showcase_dir = PathBuf::from(&config.project.showcase_crate);
        let main_rs =
            std::fs::read_to_string(showcase_dir.join("src/main.rs")).expect("read main.rs");
        assert!(main_rs.contains("use showcase_entry as _;"));
        assert!(main_rs.contains("document::Stylesheet { href: asset!(\"/assets/app.css\") }"));
        assert!(main_rs
            .contains("document::Stylesheet { href: asset!(\"/assets/styles/tailwind.css\") }"));
        assert!(showcase_dir.join("assets/app.css").exists());
        assert!(showcase_dir.join("assets/styles/tailwind.css").exists());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn scaffolding_drops_the_shell_stylesheet_earlier_versions_generated() {
        let (dir, config) = project("dioxus-showcase-legacy-css");
        let showcase_assets = PathBuf::from(&config.project.showcase_crate).join("assets");
        std::fs::create_dir_all(&showcase_assets).expect("create showcase assets");
        std::fs::write(showcase_assets.join("showcase.css"), ":root { --stale: 1; }")
            .expect("write stale stylesheet");

        ensure_showcase_app_scaffold(&config).expect("scaffold");

        assert!(!showcase_assets.join("showcase.css").exists());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_user_supplied_showcase_css_is_not_treated_as_legacy_output() {
        let (dir, config) = project("dioxus-showcase-user-css");
        let entry_assets = PathBuf::from(&config.project.entry_crate).join("assets");
        std::fs::create_dir_all(&entry_assets).expect("create entry assets");
        std::fs::write(entry_assets.join("showcase.css"), ".mine {}").expect("write user css");

        ensure_showcase_app_scaffold(&config).expect("scaffold");

        let showcase_assets = PathBuf::from(&config.project.showcase_crate).join("assets");
        assert_eq!(
            std::fs::read_to_string(showcase_assets.join("showcase.css")).expect("read css"),
            ".mine {}"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn write_artifacts_matches_golden_outputs_and_is_stable() {
        let (dir, config) = project("dioxus-showcase-build-golden");
        let stories = vec![button_story()];

        write_artifacts(&config, &stories, &[]).expect("first build");

        let manifest_path = PathBuf::from(&config.build.out_dir).join("showcase.manifest.json");
        let showcase_dir = PathBuf::from(&config.project.showcase_crate);
        let generated_path = showcase_dir.join("src/generated.rs");
        let main_path = showcase_dir.join("src/main.rs");

        let manifest = std::fs::read_to_string(&manifest_path).expect("read manifest");
        let generated = std::fs::read_to_string(&generated_path).expect("read generated");
        let first_main = std::fs::read_to_string(&main_path).expect("read main");

        assert_eq!(manifest.trim_end(), GOLDEN_MANIFEST.trim_end());
        assert_eq!(generated.trim_end(), GOLDEN_GENERATED_RS.trim_end());

        write_artifacts(&config, &stories, &[]).expect("second build");
        let second_main = std::fs::read_to_string(&main_path).expect("read second main");
        let second_generated =
            std::fs::read_to_string(&generated_path).expect("read second generated");
        let second_manifest =
            std::fs::read_to_string(&manifest_path).expect("read second manifest");

        assert_eq!(first_main, second_main);
        assert_eq!(generated, second_generated);
        assert_eq!(manifest, second_manifest);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_manifest_declares_the_advisory_schema_version() {
        let (dir, config) = project("dioxus-showcase-schema-version");

        write_artifacts(&config, &[button_story()], &[]).expect("write artifacts");

        let manifest_path = PathBuf::from(&config.build.out_dir).join("showcase.manifest.json");
        let manifest = std::fs::read_to_string(&manifest_path).expect("read manifest");
        assert!(manifest.starts_with("{\"schema_version\":2,"), "{manifest}");
        // The CLI must not carry its own copy of the version: it reads core's.
        assert!(
            manifest.starts_with(&format!("{{\"schema_version\":{MANIFEST_SCHEMA_VERSION},")),
            "{manifest}"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn stable_generation_token_is_deterministic() {
        let stories = vec![button_story()];

        let first = stable_generation_token(&stories, &[]);
        let second = stable_generation_token(&stories, &[]);

        assert_eq!(first, second);
        assert!(first.starts_with("manifest-"));

        let providers = vec![ProviderDefinition {
            source_path: "src/provider.rs".to_owned(),
            module_path: "provider::Shell".to_owned(),
            wrap_symbol: "__dioxus_showcase_wrap__Shell".to_owned(),
            order: 1,
        }];
        assert_ne!(first, stable_generation_token(&stories, &providers));
    }

    /// Pins the token byte-for-byte across the `index` -> `order` rename.
    ///
    /// The rename changes the field's NAME, never the value hashed: the same `i32`
    /// still contributes the same `to_le_bytes()` in the same position. If a future
    /// change alters the hash INPUT rather than just its spelling, this test fails
    /// and CI's determinism assertion would have failed too — here it fails first,
    /// with a name that says why.
    #[test]
    fn the_generation_token_is_unchanged_by_the_index_to_order_rename() {
        let stories = vec![button_story()];

        assert_eq!(stable_generation_token(&stories, &[]), "manifest-ce76a41f02cc03a2");

        let providers = vec![ProviderDefinition {
            source_path: "src/provider.rs".to_owned(),
            module_path: "provider::Shell".to_owned(),
            wrap_symbol: "__dioxus_showcase_wrap__Shell".to_owned(),
            order: 1,
        }];
        assert_eq!(stable_generation_token(&stories, &providers), "manifest-6e737d1109715e39");
    }
}
