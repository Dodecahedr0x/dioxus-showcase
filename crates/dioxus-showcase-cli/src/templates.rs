use std::fs;
use std::path::{Component, Path, PathBuf};

use dioxus_showcase_core::ShowcaseConfig;
use handlebars::{no_escape, Handlebars};
use serde::Serialize;

use crate::discovery::slugify_title;

const GENERATED_RUNTIME_TEMPLATE: &str = include_str!("templates/generated_runtime.rs.hbs");
const SHOWCASE_CARGO_TEMPLATE: &str = include_str!("templates/showcase_cargo.toml.hbs");
const SHOWCASE_DIOXUS_TEMPLATE: &str = include_str!("templates/showcase_dioxus.toml.hbs");

/// The generated showcase app's `Cargo.toml` renames the user's component crate to a
/// fixed alias, so the entry point can name it without knowing the package name.
pub const ENTRY_CRATE_ALIAS: &str = "showcase_entry";

/// The showcase application's entry point, written **once** and never regenerated.
///
/// It is deliberately inline rather than a `.hbs` file: the twelve-kilobyte shell
/// template this replaced now lives in `dioxus-showcase-ui` as compiled components,
/// and what remains is small enough that keeping it beside the code that renders it
/// is clearer than a separate file.
///
/// Two lines are load-bearing and must not be reordered away:
///
/// - `use {{entry_crate}} as _;` is the *entire* reason any story appears. It forces
///   the component crate's rlib to be linked, which is what carries the `inventory`
///   registrations into the binary. Nothing calls into that crate, so it looks like
///   dead code — hence the comment that ships with it.
/// - Each `document::Stylesheet` line exists here rather than in the shell library
///   because `asset!()` resolves relative to the crate it is compiled in. A library
///   can only link its own CSS; a user's stylesheet has to be named at the user's
///   compile time, which is this file.
const SHOWCASE_MAIN_TEMPLATE: &str = r#"// Generated once by dioxus-showcase. Safe to edit; never regenerated.
use dioxus::prelude::*;
use {{entry_crate}} as _; // LOAD-BEARING: forces rlib linkage so registrations survive

fn main() {
    launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
{{#each stylesheets}}
        document::Stylesheet { href: asset!("{{this}}") }
{{/each}}
        dioxus_showcase_ui::ShowcaseApp { base_path: "{{base_path}}" }
    }
}
"#;

#[derive(Serialize)]
struct RuntimeContext {
    generation: String,
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
    app_title: String,
    base_path: String,
}

#[derive(Serialize)]
struct MainTemplateContext {
    entry_crate: String,
    base_path: String,
    stylesheets: Vec<String>,
}

/// Renders the generated runtime module.
///
/// This used to expand every discovered story into glue that named the macros'
/// `__dioxus_showcase_*` symbols, which coupled the CLI's string conventions to the
/// macro crate's output. Stories register themselves at link time now, so all that
/// is left is a token identifying which discovery run produced the current tree.
pub fn render_generated_runtime_rs(generation: &str) -> Result<String, String> {
    render_template(
        GENERATED_RUNTIME_TEMPLATE,
        &RuntimeContext { generation: escape_rust_string(generation) },
    )
}

/// Renders the showcase shell application's `main.rs`.
///
/// `stylesheets` are asset URLs relative to the generated app, as collected from the
/// entry crate's `assets/` directory.
pub fn render_showcase_app_main_rs(
    base_path: &str,
    stylesheets: &[String],
) -> Result<String, String> {
    render_template(
        SHOWCASE_MAIN_TEMPLATE,
        &MainTemplateContext {
            entry_crate: ENTRY_CRATE_ALIAS.to_owned(),
            base_path: escape_rust_string(&normalize_app_base_path(base_path)),
            stylesheets: stylesheets.iter().map(|href| escape_rust_string(href)).collect(),
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
    let app_title = escape_toml_string(&format!("{} Showcase", config.project.name));
    // The Dioxus CLI trims slashes itself, so a root base path renders as an empty string.
    let base_path = escape_toml_string(config.build.base_path.trim().trim_matches('/'));
    render_template(
        SHOWCASE_DIOXUS_TEMPLATE,
        &DioxusTemplateContext { app_name, app_title, base_path },
    )
}

/// Normalizes a configured base path into the form `ShowcaseApp` documents.
///
/// The prop is specified as `"/"` or `"/my-repo"` — a leading slash, and no trailing
/// slash unless the whole value is one. Users write it every other way.
fn normalize_app_base_path(base_path: &str) -> String {
    let trimmed = base_path.trim().trim_matches('/');
    if trimmed.is_empty() {
        "/".to_owned()
    } else {
        format!("/{trimmed}")
    }
}

/// Renders a Handlebars template with pre-escaped context values.
fn render_template<T: Serialize>(template: &str, context: &T) -> Result<String, String> {
    let mut handlebars = Handlebars::new();
    handlebars.register_escape_fn(no_escape);
    handlebars
        .render_template(template, context)
        .map_err(|err| format!("failed to render template: {err}"))
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

    const GOLDEN_MAIN_RS: &str = include_str!("testdata/build_golden_main.rs");

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
    fn generated_runtime_is_a_generation_token_and_nothing_else() {
        let runtime = render_generated_runtime_rs("gen-1").expect("render runtime");

        assert!(runtime.contains("pub const SHOWCASE_GENERATION: &str = \"gen-1\";"));
        // The whole point of link-time registration: the CLI stops emitting glue
        // that names symbols the macro crate invented.
        assert!(!runtime.contains("__dioxus_showcase_"));
        assert!(!runtime.contains("ShowcaseComponentDefinition"));
        assert!(!runtime.contains("showcase_components"));
        assert!(!runtime.contains("story_providers"));
        assert_eq!(runtime.lines().count(), 2);
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
        // The shell is a library now, so the generated app has to depend on it.
        assert!(cargo_toml.contains("dioxus-showcase-ui = "));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn showcase_cargo_pins_the_profiles_that_keep_registrations_linked() {
        let dir = temp_dir("dioxus-showcase-templates-profile");
        let entry_dir = dir.join("ui");
        std::fs::create_dir_all(&entry_dir).expect("create entry dir");
        std::fs::write(
            entry_dir.join("Cargo.toml"),
            "[package]\nname = \"ui\"\nversion = \"0.1.0\"\n",
        )
        .expect("write entry cargo");

        let mut config = ShowcaseConfig::default();
        config.project.entry_crate = entry_dir.to_string_lossy().to_string();
        config.project.showcase_crate = dir.join("showcase").to_string_lossy().to_string();

        let cargo_toml = render_showcase_app_cargo_toml(&config).expect("render cargo");
        // On wasm32 the `use showcase_entry as _;` line in the generated entry point
        // does not, on its own, pull the component crate's object out of its rlib —
        // nothing references a symbol inside it, so the linker never selects the
        // archive member and every registration in it is silently dropped. LTO
        // merges the crate graph before that selection happens. Both profiles need
        // it: `dx serve` builds through `dev`, `dx bundle --release` through
        // `release`. Removing either line yields an empty showcase and no error.
        assert!(cargo_toml.contains("[profile.dev]"));
        assert!(cargo_toml.contains("lto = \"thin\""));
        assert!(cargo_toml.contains("[profile.release]"));
        assert!(cargo_toml.contains("lto = true"));
        // `wasm-opt` aborts on release wasm that still carries DWARF.
        assert!(cargo_toml.contains("strip = true"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn showcase_dioxus_toml_carries_title_and_base_path() {
        let mut config = ShowcaseConfig::default();
        config.project.name = "Demo".to_owned();
        config.build.base_path = "/".to_owned();

        let root = render_showcase_app_dioxus_toml(&config).expect("render dioxus toml");
        assert!(root.contains("name = \"Demo showcase\""));
        assert!(root.contains("title = \"Demo Showcase\""));
        assert!(root.contains("base_path = \"\""));

        config.build.base_path = "/my-repo/".to_owned();
        let nested = render_showcase_app_dioxus_toml(&config).expect("render dioxus toml");
        assert!(nested.contains("base_path = \"my-repo\""));
    }

    #[test]
    fn showcase_main_matches_the_golden_entry_point() {
        let main_rs = render_showcase_app_main_rs("/", &[]).expect("render main");

        assert_eq!(main_rs, GOLDEN_MAIN_RS);
    }

    #[test]
    fn showcase_main_keeps_the_linkage_line_and_its_warning() {
        let main_rs = render_showcase_app_main_rs("/", &[]).expect("render main");

        // Deleting this line yields an empty showcase with no error at all, so the
        // comment explaining it is as load-bearing as the line itself.
        assert!(main_rs.contains("use showcase_entry as _;"));
        assert!(main_rs.contains("LOAD-BEARING"));
        assert!(main_rs.contains("dioxus_showcase_ui::ShowcaseApp { base_path: \"/\" }"));
        // The shell reads the registry itself, so the entry point neither declares
        // nor consumes the generated module.
        assert!(!main_rs.contains("mod generated"));
        assert!(!main_rs.contains("__dioxus_showcase_"));
    }

    #[test]
    fn showcase_main_links_one_stylesheet_per_discovered_file() {
        let main_rs = render_showcase_app_main_rs(
            "/",
            &["/assets/app.css".to_owned(), "/assets/styles/tailwind.css".to_owned()],
        )
        .expect("render main");

        assert!(main_rs.contains("document::Stylesheet { href: asset!(\"/assets/app.css\") }"));
        assert!(main_rs
            .contains("document::Stylesheet { href: asset!(\"/assets/styles/tailwind.css\") }"));
        // `asset!()` needs a literal at the *user's* compile time, which is why these
        // live here and not in the shell library.
        assert_eq!(main_rs.matches("document::Stylesheet").count(), 2);
    }

    #[test]
    fn showcase_main_normalizes_the_base_path_prop() {
        let root = render_showcase_app_main_rs("/", &[]).expect("render root");
        assert!(root.contains("base_path: \"/\""));

        let nested = render_showcase_app_main_rs("/my-repo/", &[]).expect("render nested");
        assert!(nested.contains("base_path: \"/my-repo\""));

        let bare = render_showcase_app_main_rs("my-repo", &[]).expect("render bare");
        assert!(bare.contains("base_path: \"/my-repo\""));

        let blank = render_showcase_app_main_rs("   ", &[]).expect("render blank");
        assert!(blank.contains("base_path: \"/\""));
    }
}
