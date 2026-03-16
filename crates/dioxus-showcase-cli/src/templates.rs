use std::fs;
use std::path::{Component, Path, PathBuf};

use dioxus_showcase_core::{ProviderDefinition, ShowcaseConfig, StoryDefinition};
use handlebars::{no_escape, Handlebars};
use serde::Serialize;

use crate::discovery::{showcase_story_symbol, slugify_title};

const GENERATED_RUNTIME_TEMPLATE: &str = include_str!("templates/generated_runtime.rs.hbs");
const SHOWCASE_MAIN_TEMPLATE: &str = include_str!("templates/showcase_main.rs.hbs");
const SHOWCASE_CARGO_TEMPLATE: &str = include_str!("templates/showcase_cargo.toml.hbs");
const SHOWCASE_DIOXUS_TEMPLATE: &str = include_str!("templates/showcase_dioxus.toml.hbs");
const SHOWCASE_APP_CSS_TEMPLATE: &str = include_str!("templates/showcase_app.css");

#[derive(Serialize)]
struct RuntimeContext {
    generation: String,
    components: Vec<RuntimeComponent>,
    providers: Vec<String>,
}

#[derive(Serialize)]
struct RuntimeComponent {
    source_path: String,
    module_path: String,
    story_path: String,
}

#[derive(Serialize)]
struct CargoTemplateContext {
    package_name: String,
    package_version: String,
    entry_crate_package_name: String,
    entry_crate_dependency_path: String,
}

#[derive(Serialize)]
struct DioxusTemplateContext {
    app_name: String,
}

#[derive(Serialize)]
struct MainTemplateContext {
    route_root: String,
    route_component: String,
    route_component_prefix: String,
    route_not_found: String,
    stylesheets: Vec<String>,
}

/// Renders the generated runtime module that imports all discovered story constructors.
pub fn render_generated_runtime_rs(
    stories: &[StoryDefinition],
    providers: &[ProviderDefinition],
    generation: &str,
) -> Result<String, String> {
    let entry_crate_alias = "showcase_entry".to_owned();
    let components = stories
        .iter()
        .map(|story| RuntimeComponent {
            source_path: escape_rust_string(&story.source_path),
            module_path: escape_rust_string(&story.module_path),
            story_path: render_story_path(
                &entry_crate_alias,
                &story.module_path,
                &story.renderer_symbol,
            ),
        })
        .collect();
    let providers = render_provider_paths(&entry_crate_alias, providers);

    render_template(
        GENERATED_RUNTIME_TEMPLATE,
        &RuntimeContext { generation: escape_rust_string(generation), components, providers },
    )
}

/// Renders the showcase shell application's `main.rs`.
pub fn render_showcase_app_main_rs(
    _base_path: &str,
    stylesheets: &[String],
) -> Result<String, String> {
    render_template(
        SHOWCASE_MAIN_TEMPLATE,
        &MainTemplateContext {
            route_root: "/".to_owned(),
            route_component: "/component/:id".to_owned(),
            route_component_prefix: "/component/".to_owned(),
            route_not_found: "/:..route".to_owned(),
            stylesheets: stylesheets.to_vec(),
        },
    )
}

/// Renders the generated showcase app `Cargo.toml`.
pub fn render_showcase_app_cargo_toml(config: &ShowcaseConfig) -> Result<String, String> {
    let package_name = slugify_title(&format!("{}-showcase", config.project.name));
    let entry_crate_package_name = discover_entry_crate_package_name(config)?;
    let entry_crate_dependency_path = relative_dependency_path(
        &showcase_app_dir(config),
        Path::new(&config.project.entry_crate),
    )?;

    render_template(
        SHOWCASE_CARGO_TEMPLATE,
        &CargoTemplateContext {
            package_name,
            package_version: env!("CARGO_PKG_VERSION").to_owned(),
            entry_crate_package_name,
            entry_crate_dependency_path: escape_toml_string(&entry_crate_dependency_path),
        },
    )
}

/// Renders the generated showcase app `Dioxus.toml`.
pub fn render_showcase_app_dioxus_toml(config: &ShowcaseConfig) -> Result<String, String> {
    let app_name = escape_toml_string(&format!("{} showcase", config.project.name));
    render_template(SHOWCASE_DIOXUS_TEMPLATE, &DioxusTemplateContext { app_name })
}

/// Returns the static CSS used by the generated showcase shell.
pub fn render_showcase_app_css() -> &'static str {
    SHOWCASE_APP_CSS_TEMPLATE
}

/// Renders a Handlebars template with pre-escaped context values.
fn render_template<T: Serialize>(template: &str, context: &T) -> Result<String, String> {
    let mut handlebars = Handlebars::new();
    handlebars.register_escape_fn(no_escape);
    handlebars
        .render_template(template, context)
        .map_err(|err| format!("failed to render template: {err}"))
}

/// Builds the fully qualified path to a generated story constructor in the entry crate.
fn render_story_path(entry_crate_alias: &str, module_path: &str, renderer_symbol: &str) -> String {
    let story_symbol = showcase_story_symbol(renderer_symbol);
    let mut segments: Vec<&str> =
        module_path.split("::").filter(|segment| !segment.is_empty()).collect();
    if segments.is_empty() {
        return format!("{entry_crate_alias}::{story_symbol}");
    }
    segments.pop();
    segments.push(story_symbol.as_str());
    format!("{entry_crate_alias}::{}", segments.join("::"))
}

/// Builds fully qualified provider wrapper paths ordered by provider index.
fn render_provider_paths(entry_crate_alias: &str, providers: &[ProviderDefinition]) -> Vec<String> {
    let mut ordered = providers.to_vec();
    ordered.sort_by_key(|provider| provider.index);
    ordered
        .into_iter()
        .map(|provider| {
            render_provider_path(entry_crate_alias, &provider.module_path, &provider.wrap_symbol)
        })
        .collect()
}

/// Builds the fully qualified path to a provider wrapper function in the entry crate.
fn render_provider_path(entry_crate_alias: &str, module_path: &str, wrap_symbol: &str) -> String {
    let mut segments: Vec<&str> =
        module_path.split("::").filter(|segment| !segment.is_empty()).collect();
    if segments.is_empty() {
        return format!("{entry_crate_alias}::{wrap_symbol}");
    }
    segments.pop();
    segments.push(wrap_symbol);
    format!("{entry_crate_alias}::{}", segments.join("::"))
}

/// Reads the configured entry crate package name from its `Cargo.toml`.
fn discover_entry_crate_package_name(config: &ShowcaseConfig) -> Result<String, String> {
    let cargo_toml_path = Path::new(&config.project.entry_crate).join("Cargo.toml");
    let content = fs::read_to_string(&cargo_toml_path)
        .map_err(|err| format!("failed to read {}: {err}", cargo_toml_path.display()))?;

    let mut section: Option<&str> = None;
    let mut package_name: Option<String> = None;

    for raw_line in content.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            section = Some(&line[1..line.len() - 1]);
            continue;
        }

        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        let value = value.trim();

        match section {
            Some("package") if key == "name" => package_name = Some(parse_toml_string(value)?),
            _ => {}
        }
    }

    package_name.ok_or_else(|| format!("missing [package].name in {}", cargo_toml_path.display()))
}

/// Parses a bare TOML string literal value from a `key = "value"` line.
fn parse_toml_string(value: &str) -> Result<String, String> {
    if !(value.starts_with('"') && value.ends_with('"')) {
        return Err(format!("expected quoted string, got {value}"));
    }
    Ok(value[1..value.len() - 1].to_owned())
}

/// Computes a relative dependency path from the generated showcase app to the entry crate.
fn relative_dependency_path(from_dir: &Path, to_dir: &Path) -> Result<String, String> {
    let from_components: Vec<Component<'_>> = from_dir.components().collect();
    let to_components: Vec<Component<'_>> = to_dir.components().collect();

    let shared_len = from_components.iter().zip(&to_components).take_while(|(a, b)| a == b).count();

    let mut relative = PathBuf::new();
    for _ in shared_len..from_components.len() {
        relative.push("..");
    }
    for component in &to_components[shared_len..] {
        relative.push(component.as_os_str());
    }

    if relative.as_os_str().is_empty() {
        return Ok(".".to_owned());
    }

    relative.to_str().map(|path| path.to_owned()).ok_or_else(|| {
        format!(
            "failed to render relative path from {} to {}",
            from_dir.display(),
            to_dir.display()
        )
    })
}

/// Mirrors `scaffold::showcase_app_dir` for template rendering helpers.
fn showcase_app_dir(config: &ShowcaseConfig) -> PathBuf {
    PathBuf::from(&config.project.showcase_crate)
}

/// Escapes a string for inclusion in generated TOML source.
fn escape_toml_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Escapes a string for inclusion in generated Rust source.
fn escape_rust_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::*;
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
    fn generated_runtime_includes_components() {
        let stories = vec![StoryDefinition {
            id: "atoms-button".to_owned(),
            title: "Atoms/Button".to_owned(),
            source_path: "/workspace/src/button.rs".to_owned(),
            module_path: "button_variants::Button".to_owned(),
            renderer_symbol: "__dioxus_showcase_render__Button".to_owned(),
            tags: vec!["atoms".to_owned()],
        }];

        let runtime = render_generated_runtime_rs(&stories, &[], "gen-1").expect("render runtime");
        assert!(
            runtime.contains("pub fn showcase_components() -> Vec<ShowcaseComponentDefinition>")
        );
        assert!(runtime.contains("pub const SHOWCASE_GENERATION: &str = \"gen-1\";"));
        assert!(
            runtime.contains("showcase_entry::button_variants::__dioxus_showcase_story__Button")
        );
        assert!(runtime.contains("duplicate showcase id '{}'"));
        assert!(runtime.contains("r#\"button_variants::Button\"#"));
        assert!(
            runtime.contains("pub fn story_providers() -> Vec<::dioxus_showcase::StoryProvider>")
        );
    }

    #[test]
    fn generated_runtime_wraps_stories_with_providers() {
        let stories = vec![StoryDefinition {
            id: "atoms-button".to_owned(),
            title: "Atoms/Button".to_owned(),
            source_path: "/workspace/src/button.rs".to_owned(),
            module_path: "button_variants::Button".to_owned(),
            renderer_symbol: "__dioxus_showcase_render__Button".to_owned(),
            tags: vec!["atoms".to_owned()],
        }];
        let providers = vec![
            ProviderDefinition {
                source_path: "/workspace/src/provider_a.rs".to_owned(),
                module_path: "providers::Outer".to_owned(),
                wrap_symbol: "__dioxus_showcase_wrap__Outer".to_owned(),
                index: 0,
            },
            ProviderDefinition {
                source_path: "/workspace/src/provider_b.rs".to_owned(),
                module_path: "providers::Inner".to_owned(),
                wrap_symbol: "__dioxus_showcase_wrap__Inner".to_owned(),
                index: 1,
            },
        ];

        let runtime =
            render_generated_runtime_rs(&stories, &providers, "gen-2").expect("render runtime");
        assert!(runtime.contains("showcase_entry::providers::__dioxus_showcase_wrap__Outer,"));
        assert!(runtime.contains("showcase_entry::providers::__dioxus_showcase_wrap__Inner,"));
    }

    #[test]
    fn showcase_cargo_includes_entry_crate_dependency() {
        let dir = temp_dir("dioxus-showcase-templates");
        let entry_dir = dir.join("examples/basic");
        let showcase_dir = dir.join("examples/basic/showcase");
        std::fs::create_dir_all(&entry_dir).expect("create entry dir");
        std::fs::create_dir_all(&showcase_dir).expect("create showcase dir");
        std::fs::write(
            entry_dir.join("Cargo.toml"),
            "[package]\nname = \"basic-example\"\nversion = \"0.1.0\"\n",
        )
        .expect("write entry cargo");

        let mut config = ShowcaseConfig::default();
        config.project.name = "Demo".to_owned();
        config.project.entry_crate = entry_dir.to_string_lossy().to_string();
        config.project.showcase_crate = showcase_dir.to_string_lossy().to_string();

        let cargo_toml = render_showcase_app_cargo_toml(&config).expect("render cargo");
        assert!(cargo_toml.contains(&format!("version = \"{}\"", env!("CARGO_PKG_VERSION"))));
        assert!(cargo_toml.contains("[workspace]"));
        assert!(cargo_toml.contains("showcase_entry = { package = \"basic-example\""));
        assert!(cargo_toml.contains("path = \"..\""));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn showcase_main_renders_routes() {
        let app = render_showcase_app_main_rs(
            "/",
            &["/assets/app.css".to_owned(), "/assets/styles/tailwind.css".to_owned()],
        )
        .expect("render app");
        assert!(app.contains("#[route(\"/\")"));
        assert!(app.contains("#[route(\"/component/:id\")"));
        assert!(app.contains("#[route(\"/:..route\")"));
        assert!(app.contains("fn Component(id: String) -> Element"));
        assert!(app.contains("fn story_canvas(component: generated::ShowcaseComponentDefinition)"));
        assert!(app.contains("section { class: \"canvas\", {story_canvas(component)} }"));
        assert!(app.contains("ErrorBoundary {"));
        assert!(app.contains("errors.clear_errors()"));
        assert!(app.contains("Story render failed"));
        assert!(app.contains("enum ThemeMode"));
        assert!(app.contains("\"data-theme\": theme.read().as_str()"));
        assert!(app.contains("class: \"theme-toggle\""));
        assert!(app.contains("class: \"theme-toggle-track\""));
        assert!(!app.contains("Back to list"));
        assert!(!app.contains("components\" }"));
        assert!(app.contains("document::Stylesheet { href: asset!(\"/assets/app.css\") }"));
        assert!(
            app.contains("document::Stylesheet { href: asset!(\"/assets/styles/tailwind.css\") }")
        );
    }

    #[test]
    fn showcase_main_ignores_base_path_for_routes() {
        let app = render_showcase_app_main_rs(
            "/showcase/",
            &["/assets/app.css".to_owned(), "/assets/styles/tailwind.css".to_owned()],
        )
        .expect("render app");
        assert!(app.contains("#[route(\"/\")"));
        assert!(app.contains("#[route(\"/component/:id\")"));
        assert!(app.contains("#[route(\"/:..route\")"));
        assert!(app.contains("document::Stylesheet { href: asset!(\"/assets/app.css\") }"));
        assert!(
            app.contains("document::Stylesheet { href: asset!(\"/assets/styles/tailwind.css\") }")
        );
    }
}
