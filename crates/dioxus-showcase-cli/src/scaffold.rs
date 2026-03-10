use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use dioxus_showcase_core::{ShowcaseConfig, StoryDefinition, StoryManifest};

use crate::templates;

pub fn showcase_app_dir(config: &ShowcaseConfig) -> PathBuf {
    PathBuf::from(&config.project.showcase_crate)
}

pub fn write_artifacts(
    config: &ShowcaseConfig,
    stories: &[StoryDefinition],
) -> Result<PathBuf, String> {
    let out_dir = PathBuf::from(&config.build.out_dir);
    fs::create_dir_all(&out_dir)
        .map_err(|err| format!("failed to create {}: {err}", out_dir.display()))?;

    let mut manifest = StoryManifest::new(1);
    for story in stories {
        manifest.add_story(story.clone());
    }

    let manifest_path = out_dir.join("showcase.manifest.json");
    fs::write(&manifest_path, manifest.to_json())
        .map_err(|err| format!("failed to create {}: {err}", manifest_path.display()))?;

    ensure_showcase_app_scaffold(config)?;
    let stylesheets = sync_entry_assets_and_collect_stylesheets(config)?;
    let main_path = showcase_app_dir(config).join("src/main.rs");
    let main_rs = templates::render_showcase_app_main_rs(&stylesheets)?;
    fs::write(&main_path, main_rs)
        .map_err(|err| format!("failed to create {}: {err}", main_path.display()))?;

    let generated_path = showcase_app_dir(config).join("src/generated.rs");
    let generation = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|err| format!("system clock error while generating showcase runtime: {err}"))?
        .as_nanos()
        .to_string();
    let generated_runtime = templates::render_generated_runtime_rs(stories, &generation)?;
    fs::write(&generated_path, generated_runtime)
        .map_err(|err| format!("failed to create {}: {err}", generated_path.display()))?;

    Ok(out_dir)
}

pub fn ensure_showcase_app_scaffold(config: &ShowcaseConfig) -> Result<bool, String> {
    let app_dir = showcase_app_dir(config);
    let src_dir = app_dir.join("src");
    fs::create_dir_all(&src_dir)
        .map_err(|err| format!("failed to create {}: {err}", src_dir.display()))?;

    let mut created = false;

    let cargo_toml_path = app_dir.join("Cargo.toml");
    let cargo_toml = templates::render_showcase_app_cargo_toml(config)?;
    let should_write_cargo = !cargo_toml_path.exists();
    if should_write_cargo {
        fs::write(&cargo_toml_path, cargo_toml)
            .map_err(|err| format!("failed to create {}: {err}", cargo_toml_path.display()))?;
        created = true;
    }

    let dioxus_toml_path = app_dir.join("Dioxus.toml");
    if !dioxus_toml_path.exists() {
        let dioxus_toml = templates::render_showcase_app_dioxus_toml(config)?;
        fs::write(&dioxus_toml_path, dioxus_toml)
            .map_err(|err| format!("failed to create {}: {err}", dioxus_toml_path.display()))?;
        created = true;
    }

    let main_rs_path = src_dir.join("main.rs");
    if !main_rs_path.exists() {
        let main_rs = templates::render_showcase_app_main_rs(&[])?;
        fs::write(&main_rs_path, main_rs)
            .map_err(|err| format!("failed to create {}: {err}", main_rs_path.display()))?;
        created = true;
    }

    let generated_rs_path = src_dir.join("generated.rs");
    if !generated_rs_path.exists() {
        let generated_rs = templates::render_generated_runtime_rs(&[], "initial")?;
        fs::write(&generated_rs_path, generated_rs)
            .map_err(|err| format!("failed to create {}: {err}", generated_rs_path.display()))?;
        created = true;
    }

    Ok(created)
}

fn sync_entry_assets_and_collect_stylesheets(
    config: &ShowcaseConfig,
) -> Result<Vec<String>, String> {
    let entry_assets_dir = Path::new(&config.project.entry_crate).join("assets");
    let showcase_assets_dir = showcase_app_dir(config).join("assets");

    if !entry_assets_dir.exists() {
        return Ok(Vec::new());
    }

    copy_dir_recursive(&entry_assets_dir, &showcase_assets_dir)?;

    let mut stylesheets = Vec::new();
    collect_stylesheets(&showcase_assets_dir, &showcase_assets_dir, &mut stylesheets)?;
    stylesheets.sort();
    Ok(stylesheets)
}

fn copy_dir_recursive(from: &Path, to: &Path) -> Result<(), String> {
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

    use dioxus_showcase_core::{ShowcaseConfig, StoryDefinition};

    use super::write_artifacts;

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
    fn write_artifacts_overwrites_existing_main_rs() {
        let dir = temp_dir("dioxus-showcase-write-artifacts");
        let entry_dir = dir.join("web");
        let showcase_dir = dir.join("showcase");
        let src_dir = showcase_dir.join("src");
        std::fs::create_dir_all(&entry_dir).expect("create entry dir");
        std::fs::create_dir_all(entry_dir.join("assets/styles")).expect("create entry assets");
        std::fs::create_dir_all(&src_dir).expect("create showcase src");
        std::fs::write(
            entry_dir.join("Cargo.toml"),
            "[package]\nname = \"web\"\nversion = \"0.1.0\"\n",
        )
        .expect("write entry cargo");
        std::fs::write(entry_dir.join("assets/app.css"), "body { color: red; }")
            .expect("write app css");
        std::fs::write(entry_dir.join("assets/styles/tailwind.css"), ".btn { display: flex; }")
            .expect("write tailwind css");

        let main_rs_path = src_dir.join("main.rs");
        std::fs::write(&main_rs_path, "stale main").expect("write stale main");

        let mut config = ShowcaseConfig::default();
        config.project.entry_crate = entry_dir.to_string_lossy().to_string();
        config.project.showcase_crate = showcase_dir.to_string_lossy().to_string();
        config.build.out_dir = dir.join("target/showcase").to_string_lossy().to_string();

        write_artifacts(
            &config,
            &[StoryDefinition {
                id: "atoms-button".to_owned(),
                title: "Atoms/Button".to_owned(),
                source_path: "src/button.rs".to_owned(),
                module_path: "button::Button".to_owned(),
                renderer_symbol: "__dioxus_showcase_render__Button".to_owned(),
                tags: vec![],
            }],
        )
        .expect("write artifacts");

        let updated_main = std::fs::read_to_string(&main_rs_path).expect("read updated main");
        assert!(updated_main.contains("fn Sidebar"));
        assert!(updated_main.contains("asset!(\"/assets/app.css\")"));
        assert!(updated_main.contains("asset!(\"/assets/styles/tailwind.css\")"));
        assert!(!updated_main.contains("stale main"));
        assert!(showcase_dir.join("assets/app.css").exists());
        assert!(showcase_dir.join("assets/styles/tailwind.css").exists());

        let _ = std::fs::remove_dir_all(&dir);
    }
}
