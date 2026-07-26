//! Static discovery of annotated items — **advisory only**.
//!
//! Nothing at runtime depends on this module any more. Stories and providers
//! register themselves at link time through `inventory`, so what a source scan
//! finds only ever feeds two things: the diagnostics `dioxus-showcase check`
//! prints, and `target/showcase/showcase.manifest.json`.
//!
//! Two rules follow from that, and both are load-bearing:
//!
//! - **Discovery must never fail a build.** If this module and the macros drift
//!   apart — a `#[showcase]` behind a `cfg`, a module it cannot resolve, a file it
//!   cannot parse — the result is worse diagnostics, not a broken build. `build`
//!   downgrades every error here to a warning; `check`, whose entire job is
//!   diagnostics, still reports them as failures.
//! - **Discovery must never compile anything.** `check` is expected to be fast
//!   enough to run on every save, which is the only reason a second, static
//!   implementation of "what stories exist" is worth keeping at all.
use std::fs;
use std::path::{Path, PathBuf};

use dioxus_showcase_core::{ProviderDefinition, ShowcaseConfig, StoryDefinition};
use syn::{Attribute, Item};

/// Discovers all story definitions reachable from the configured entry crate.
pub fn discover_components(
    root: &Path,
    config: &ShowcaseConfig,
) -> Result<Vec<StoryDefinition>, String> {
    let entry_src_dir = entry_crate_src_dir(root, config)?;
    let files = discover_component_source_files(root, config)?;
    let mut components = Vec::new();
    for file in files {
        components.extend(discover_components_in_file(&entry_src_dir, &file)?);
    }

    Ok(components)
}

/// Discovers all provider definitions reachable from the configured entry crate.
pub fn discover_providers(
    root: &Path,
    config: &ShowcaseConfig,
) -> Result<Vec<ProviderDefinition>, String> {
    let entry_src_dir = entry_crate_src_dir(root, config)?;
    let files = discover_component_source_files(root, config)?;
    let mut providers = Vec::new();
    for file in files {
        providers.extend(discover_providers_in_file(&entry_src_dir, &file)?);
    }

    // Ascending `order`, matching the runtime rule that the lowest order wraps
    // outermost. The runtime breaks ties by module path and this does not; that is
    // acceptable because this ordering is advisory, but it must not contradict.
    providers.sort_by_key(|provider| provider.order);
    Ok(providers)
}

/// Returns the Rust source files reachable from the entry crate root modules.
pub fn discover_component_source_files(
    root: &Path,
    config: &ShowcaseConfig,
) -> Result<Vec<PathBuf>, String> {
    let entry_src_dir = entry_crate_src_dir(root, config)?;
    let mut files = Vec::new();
    let mut visited = std::collections::BTreeSet::new();

    for crate_root in discover_crate_root_files(&entry_src_dir) {
        collect_reachable_rust_files(&crate_root, &mut visited, &mut files)?;
    }

    Ok(files)
}

/// Returns every route id claimed by more than one discovered story.
///
/// Reporting all of them, rather than erroring on the first, is what lets `check`
/// print one diagnostic per collision. The result is sorted and deduplicated, so it
/// does not depend on the order files happened to be walked in.
pub fn duplicate_story_ids(stories: &[StoryDefinition]) -> Vec<String> {
    let mut ids: Vec<&str> = stories.iter().map(|story| story.id.as_str()).collect();
    ids.sort_unstable();

    let mut duplicates: Vec<String> =
        ids.windows(2).filter(|pair| pair[0] == pair[1]).map(|pair| pair[0].to_owned()).collect();
    duplicates.dedup();
    duplicates
}

/// Resolves the configured entry crate `src/` directory.
fn entry_crate_src_dir(root: &Path, config: &ShowcaseConfig) -> Result<PathBuf, String> {
    let entry_crate_dir = root.join(&config.project.entry_crate);
    if !entry_crate_dir.exists() {
        return Err(format!(
            "entry crate path '{}' does not exist. Update project.entry_crate in DioxusShowcase.toml.",
            entry_crate_dir.display()
        ));
    }

    let src_dir = entry_crate_dir.join("src");
    if !src_dir.exists() {
        return Err(format!(
            "entry crate source directory '{}' does not exist.",
            src_dir.display()
        ));
    }

    Ok(src_dir)
}

/// Returns the entry crate root files that can seed discovery traversal.
fn discover_crate_root_files(entry_src_dir: &Path) -> Vec<PathBuf> {
    ["lib.rs", "main.rs"]
        .into_iter()
        .map(|name| entry_src_dir.join(name))
        .filter(|path| path.exists())
        .collect()
}

/// Recursively follows reachable external modules and collects Rust source files once each.
fn collect_reachable_rust_files(
    path: &Path,
    visited: &mut std::collections::BTreeSet<PathBuf>,
    files: &mut Vec<PathBuf>,
) -> Result<(), String> {
    let canonical = path
        .canonicalize()
        .map_err(|err| format!("failed to canonicalize {}: {err}", path.display()))?;
    if !visited.insert(canonical) {
        return Ok(());
    }

    if !is_rust_source_file(path) {
        return Ok(());
    }

    files.push(path.to_path_buf());

    let source = fs::read_to_string(path)
        .map_err(|err| format!("failed to read source file {}: {err}", path.display()))?;
    let file = syn::parse_file(&source)
        .map_err(|err| format!("failed to parse Rust source {}: {err}", path.display()))?;

    for nested_path in discover_external_module_files(path, &file.items)? {
        collect_reachable_rust_files(&nested_path, visited, files)?;
    }

    Ok(())
}

/// Checks whether a path ends with the `.rs` extension.
fn is_rust_source_file(path: &Path) -> bool {
    path.extension().and_then(std::ffi::OsStr::to_str).is_some_and(|ext| ext == "rs")
}

/// Extracts story definitions from one Rust source file.
fn discover_components_in_file(
    entry_src_dir: &Path,
    path: &Path,
) -> Result<Vec<StoryDefinition>, String> {
    let content = fs::read_to_string(path)
        .map_err(|err| format!("failed to read source file {}: {err}", path.display()))?;
    let file = syn::parse_file(&content)
        .map_err(|err| format!("failed to parse Rust source {}: {err}", path.display()))?;
    let source_path = canonical_source_path(path)?;

    let mut stories = Vec::new();
    collect_stories_from_items(entry_src_dir, path, &source_path, &[], &file.items, &mut stories)?;
    Ok(stories)
}

/// Extracts provider definitions from one Rust source file.
fn discover_providers_in_file(
    entry_src_dir: &Path,
    path: &Path,
) -> Result<Vec<ProviderDefinition>, String> {
    let content = fs::read_to_string(path)
        .map_err(|err| format!("failed to read source file {}: {err}", path.display()))?;
    let file = syn::parse_file(&content)
        .map_err(|err| format!("failed to parse Rust source {}: {err}", path.display()))?;
    let source_path = canonical_source_path(path)?;

    let mut providers = Vec::new();
    collect_providers_from_items(
        entry_src_dir,
        path,
        &source_path,
        &[],
        &file.items,
        &mut providers,
    )?;
    Ok(providers)
}

/// Finds file-backed nested modules declared by `mod foo;`.
fn discover_external_module_files(path: &Path, items: &[Item]) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();

    for item in items {
        let Item::Mod(item_mod) = item else {
            continue;
        };
        if item_mod.content.is_some() {
            continue;
        }

        let nested = resolve_external_module_path(path, item_mod)?;
        files.push(nested);
    }

    Ok(files)
}

/// Walks parsed items recursively and appends discovered story definitions.
fn collect_stories_from_items(
    entry_src_dir: &Path,
    file_path: &Path,
    source_path: &str,
    module_segments: &[String],
    items: &[Item],
    out: &mut Vec<StoryDefinition>,
) -> Result<(), String> {
    for item in items {
        match item {
            Item::Fn(item_fn) => {
                let Some(showcase_attr) = find_story_attribute(&item_fn.attrs) else {
                    continue;
                };

                let metadata = parse_showcase_component_attribute(showcase_attr, file_path)?;
                let component_name = item_fn.sig.ident.to_string();
                let title = metadata.title.unwrap_or_else(|| component_name.clone());
                let mut module_path_segments = module_segments.to_vec();
                module_path_segments.push(component_name.clone());

                out.push(StoryDefinition {
                    id: slugify_title(&title),
                    title,
                    source_path: source_path.to_owned(),
                    module_path: render_module_path(
                        entry_src_dir,
                        file_path,
                        &module_path_segments,
                    )?,
                    renderer_symbol: showcase_renderer_symbol(&component_name),
                    tags: metadata.tags,
                });
            }
            Item::Mod(item_mod) => {
                let Some((_, nested_items)) = &item_mod.content else {
                    continue;
                };
                let mut nested_segments = module_segments.to_vec();
                nested_segments.push(item_mod.ident.to_string());
                collect_stories_from_items(
                    entry_src_dir,
                    file_path,
                    source_path,
                    &nested_segments,
                    nested_items,
                    out,
                )?;
            }
            _ => {}
        }
    }

    Ok(())
}

/// Walks parsed items recursively and appends discovered provider definitions.
fn collect_providers_from_items(
    entry_src_dir: &Path,
    file_path: &Path,
    source_path: &str,
    module_segments: &[String],
    items: &[Item],
    out: &mut Vec<ProviderDefinition>,
) -> Result<(), String> {
    for item in items {
        match item {
            Item::Fn(item_fn) => {
                let Some(provider_attr) = find_provider_attribute(&item_fn.attrs) else {
                    continue;
                };

                let metadata = parse_provider_attribute(provider_attr, file_path)?;
                let component_name = item_fn.sig.ident.to_string();
                let mut module_path_segments = module_segments.to_vec();
                module_path_segments.push(component_name.clone());

                out.push(ProviderDefinition {
                    source_path: source_path.to_owned(),
                    module_path: render_module_path(
                        entry_src_dir,
                        file_path,
                        &module_path_segments,
                    )?,
                    wrap_symbol: showcase_provider_symbol(&component_name),
                    order: metadata.order.unwrap_or(0),
                });
            }
            Item::Mod(item_mod) => {
                let Some((_, nested_items)) = &item_mod.content else {
                    continue;
                };
                let mut nested_segments = module_segments.to_vec();
                nested_segments.push(item_mod.ident.to_string());
                collect_providers_from_items(
                    entry_src_dir,
                    file_path,
                    source_path,
                    &nested_segments,
                    nested_items,
                    out,
                )?;
            }
            _ => {}
        }
    }

    Ok(())
}

/// Returns the first story-producing attribute on a function, if any.
fn find_story_attribute(attrs: &[Attribute]) -> Option<&Attribute> {
    attrs.iter().find(|attr| {
        attr.path()
            .segments
            .last()
            .is_some_and(|segment| segment.ident == "showcase" || segment.ident == "story")
    })
}

/// Returns the provider attribute on a function, if present.
fn find_provider_attribute(attrs: &[Attribute]) -> Option<&Attribute> {
    attrs
        .iter()
        .find(|attr| attr.path().segments.last().is_some_and(|segment| segment.ident == "provider"))
}

/// Resolves a `mod` declaration to its backing source file.
fn resolve_external_module_path(
    current_file: &Path,
    item_mod: &syn::ItemMod,
) -> Result<PathBuf, String> {
    if let Some(path_override) = module_path_override(&item_mod.attrs)? {
        let resolved = current_file.parent().unwrap_or_else(|| Path::new("")).join(path_override);
        if resolved.exists() {
            return Ok(resolved);
        }

        return Err(format!(
            "module '{}' declared in {} points to missing path {}",
            item_mod.ident,
            current_file.display(),
            resolved.display()
        ));
    }

    let module_name = item_mod.ident.to_string();
    let search_root = external_module_search_dir(current_file)?;
    let file_candidate = search_root.join(format!("{module_name}.rs"));
    if file_candidate.exists() {
        return Ok(file_candidate);
    }

    let mod_candidate = search_root.join(&module_name).join("mod.rs");
    if mod_candidate.exists() {
        return Ok(mod_candidate);
    }

    Err(format!(
        "failed to resolve module '{}' declared in {}",
        module_name,
        current_file.display()
    ))
}

/// Determines the directory Rust will search for sibling module files.
fn external_module_search_dir(current_file: &Path) -> Result<PathBuf, String> {
    let parent = current_file.parent().ok_or_else(|| {
        format!("failed to resolve parent directory for module source {}", current_file.display())
    })?;
    let stem = current_file
        .file_stem()
        .and_then(std::ffi::OsStr::to_str)
        .ok_or_else(|| format!("failed to derive module stem from {}", current_file.display()))?;

    Ok(match stem {
        "lib" | "main" | "mod" => parent.to_path_buf(),
        other => parent.join(other),
    })
}

/// Reads a `#[path = \"...\"]` override from a module declaration, if one exists.
fn module_path_override(attrs: &[Attribute]) -> Result<Option<String>, String> {
    for attr in attrs {
        if !attr.path().is_ident("path") {
            continue;
        }

        let meta = &attr.meta;
        let syn::Meta::NameValue(named) = meta else {
            return Err("invalid #[path = ...] attribute: expected name-value syntax".to_owned());
        };
        let syn::Expr::Lit(expr_lit) = &named.value else {
            return Err("invalid #[path = ...] attribute: expected string literal".to_owned());
        };
        let syn::Lit::Str(value) = &expr_lit.lit else {
            return Err("invalid #[path = ...] attribute: expected string literal".to_owned());
        };
        return Ok(Some(value.value()));
    }

    Ok(None)
}

struct ShowcaseAttrMeta {
    title: Option<String>,
    tags: Vec<String>,
    order: Option<i32>,
}

/// Parses the supported `#[showcase(...)]` and `#[story(...)]` metadata fields.
fn parse_showcase_component_attribute(
    attribute: &Attribute,
    path: &Path,
) -> Result<ShowcaseAttrMeta, String> {
    // `#[showcase]` and `#[story]` are both valid with no arguments at all, in which
    // case there is no nested meta to walk.
    if !has_arguments(attribute) {
        return Ok(ShowcaseAttrMeta { title: None, tags: Vec::new(), order: None });
    }

    let mut title = None;
    let mut component = None;
    let mut name = None;
    let mut tags = Vec::new();
    let is_story = attribute.path().is_ident("story");

    attribute
        .parse_nested_meta(|meta| {
            if meta.path.is_ident("title") {
                let value: syn::LitStr = meta.value()?.parse()?;
                title = Some(value.value());
                return Ok(());
            }

            if meta.path.is_ident("component") {
                if is_story {
                    return Err(meta.error(
                        "story attributes no longer accept component = ...; use title = \"...\"",
                    ));
                }
                let value: syn::ExprPath = meta.value()?.parse()?;
                component = Some(
                    component_name_from_path(&value)
                        .map_err(|err| syn::Error::new_spanned(&value, err))?,
                );
                return Ok(());
            }

            if meta.path.is_ident("name") {
                if is_story {
                    return Err(meta.error(
                        "story attributes no longer accept name = ...; include the full path in title = \"...\"",
                    ));
                }
                let value: syn::LitStr = meta.value()?.parse()?;
                name = Some(value.value());
                return Ok(());
            }

            if meta.path.is_ident("tags") {
                let value: syn::Expr = meta.value()?.parse()?;
                tags = parse_tags_array(&value)?;
            }

            Ok(())
        })
        .map_err(|err| format!("invalid showcase attribute in {}: {err}", path.display()))?;

    let title = title;

    Ok(ShowcaseAttrMeta { title, tags, order: None })
}

/// Parses `#[provider(order = ...)]` metadata.
///
/// `order` is the only accepted spelling. The pre-0.1.0 name was `index`, and it is
/// rejected here on purpose rather than aliased: two names for one concept is
/// exactly what pinning a single spelling before the first release was meant to
/// avoid.
fn parse_provider_attribute(
    attribute: &Attribute,
    path: &Path,
) -> Result<ShowcaseAttrMeta, String> {
    // A bare `#[provider]` takes the default order, like the macro does.
    if !has_arguments(attribute) {
        return Ok(ShowcaseAttrMeta { title: None, tags: Vec::new(), order: None });
    }

    let mut order = None;

    attribute
        .parse_nested_meta(|meta| {
            if meta.path.is_ident("order") {
                let value: syn::LitInt = meta.value()?.parse()?;
                order =
                    Some(value.base10_parse::<i32>().map_err(|err| {
                        meta.error(format!("provider order must fit in i32: {err}"))
                    })?);
                return Ok(());
            }

            Err(meta.error("provider attributes only support order = <integer>"))
        })
        .map_err(|err| format!("invalid provider attribute in {}: {err}", path.display()))?;

    Ok(ShowcaseAttrMeta { title: None, tags: Vec::new(), order })
}

/// Reports whether an attribute was written with a parenthesised argument list.
///
/// `syn`'s `parse_nested_meta` treats a bare `#[provider]` as a syntax error, but
/// every showcase attribute is legal without arguments, so the bare form has to be
/// recognised before parsing rather than reported as invalid.
fn has_arguments(attribute: &Attribute) -> bool {
    matches!(attribute.meta, syn::Meta::List(_))
}

/// Extracts the final identifier from a component path expression.
fn component_name_from_path(expr_path: &syn::ExprPath) -> Result<String, String> {
    expr_path
        .path
        .segments
        .last()
        .map(|segment| segment.ident.to_string())
        .ok_or_else(|| "component path must not be empty".to_owned())
}

/// Parses a tags expression into owned strings while enforcing string literals.
fn parse_tags_array(expr: &syn::Expr) -> Result<Vec<String>, syn::Error> {
    let array = match expr {
        syn::Expr::Array(array) => array,
        _ => {
            return Err(syn::Error::new_spanned(
                expr,
                "showcase tags must be an array of string literals",
            ));
        }
    };

    array
        .elems
        .iter()
        .map(|expr| {
            let expr_lit = match expr {
                syn::Expr::Lit(expr_lit) => expr_lit,
                _ => {
                    return Err(syn::Error::new_spanned(
                        expr,
                        "showcase tags must contain only string literals",
                    ));
                }
            };

            let lit = match &expr_lit.lit {
                syn::Lit::Str(lit) => lit,
                _ => {
                    return Err(syn::Error::new_spanned(
                        expr,
                        "showcase tags must contain only string literals",
                    ));
                }
            };
            Ok(lit.value())
        })
        .collect()
}

/// Converts a source path into an absolute canonical string for manifest output.
fn canonical_source_path(path: &Path) -> Result<String, String> {
    path.canonicalize().map(|absolute| absolute.display().to_string()).map_err(|err| {
        format!("failed to canonicalize showcase source path {}: {err}", path.display())
    })
}

/// Builds the Rust module path for one discovered item.
fn render_module_path(
    entry_src_dir: &Path,
    path: &Path,
    module_segments: &[String],
) -> Result<String, String> {
    let mut segments = module_prefix_from_path(entry_src_dir, path)?;
    segments.extend(module_segments.iter().cloned());
    Ok(segments.join("::"))
}

/// Derives the module prefix implied by a source file's location under `src/`.
fn module_prefix_from_path(entry_src_dir: &Path, path: &Path) -> Result<Vec<String>, String> {
    let relative = path.strip_prefix(entry_src_dir).map_err(|err| {
        format!(
            "failed to compute module path for {} relative to {}: {err}",
            path.display(),
            entry_src_dir.display()
        )
    })?;

    let mut segments = Vec::new();
    let parent = relative.parent().unwrap_or_else(|| Path::new(""));
    for component in parent.components() {
        let value = component.as_os_str().to_string_lossy();
        if !value.is_empty() {
            segments.push(value.to_string());
        }
    }

    let stem = relative
        .file_stem()
        .and_then(std::ffi::OsStr::to_str)
        .ok_or_else(|| format!("failed to derive module name from {}", path.display()))?;
    if stem != "lib" && stem != "main" && stem != "mod" {
        segments.push(stem.to_owned());
    }

    Ok(segments)
}

/// Returns the renderer helper name the macros are expected to generate.
///
/// **Advisory.** Nothing links against this string any more — it is recorded in
/// `StoryDefinition::renderer_symbol` so the manifest still describes what a scan
/// believes the macros emitted. If the macro crate ever changes its naming, this
/// goes stale and the only consequence is a manifest field that no longer matches.
fn showcase_renderer_symbol(component_name: &str) -> String {
    format!("__dioxus_showcase_render__{component_name}")
}

/// Returns the provider wrapper helper name the macros are expected to generate.
///
/// Advisory, for the same reason as [`showcase_renderer_symbol`].
fn showcase_provider_symbol(component_name: &str) -> String {
    format!("__dioxus_showcase_wrap__{component_name}")
}

/// Normalizes a title into the route id format used by generated stories.
pub fn slugify_title(title: &str) -> String {
    let mut out = String::with_capacity(title.len());
    let mut prev_dash = false;

    for ch in title.chars() {
        let normalized = ch.to_ascii_lowercase();
        if normalized.is_ascii_alphanumeric() {
            out.push(normalized);
            prev_dash = false;
            continue;
        }

        if !prev_dash {
            out.push('-');
            prev_dash = true;
        }
    }

    out.trim_matches('-').to_owned()
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
    fn rust_source_file_detection_matches_suffix() {
        assert!(is_rust_source_file(Path::new("button.rs")));
        assert!(!is_rust_source_file(Path::new("button.txt")));
    }

    #[test]
    fn renderer_symbol_is_derived_from_component_name() {
        assert_eq!(showcase_renderer_symbol("Button"), "__dioxus_showcase_render__Button");
        assert_eq!(showcase_provider_symbol("Shell"), "__dioxus_showcase_wrap__Shell");
    }

    #[test]
    fn slugify_title_normalizes_separator_runs() {
        assert_eq!(slugify_title("Atoms/Button/Default"), "atoms-button-default");
        assert_eq!(slugify_title("  Forms   Input  "), "forms-input");
        assert_eq!(slugify_title("Hello___World"), "hello-world");
    }

    fn story_with_id(id: &str, module_path: &str) -> StoryDefinition {
        StoryDefinition {
            id: id.to_owned(),
            title: id.to_owned(),
            source_path: format!("/tmp/{module_path}.rs"),
            module_path: module_path.to_owned(),
            renderer_symbol: "__dioxus_showcase_render__demo".to_owned(),
            tags: vec![],
        }
    }

    #[test]
    fn duplicate_story_ids_are_reported_once_each() {
        let stories = vec![
            story_with_id("atoms-button", "a::button"),
            story_with_id("atoms-button", "b::button_alt"),
            story_with_id("atoms-button", "c::button_third"),
            story_with_id("atoms-badge", "d::badge"),
            story_with_id("atoms-badge", "e::badge_alt"),
            story_with_id("atoms-card", "f::card"),
        ];

        assert_eq!(
            duplicate_story_ids(&stories),
            vec!["atoms-badge".to_owned(), "atoms-button".to_owned()]
        );
    }

    #[test]
    fn unique_story_ids_report_no_duplicates() {
        let stories = vec![
            story_with_id("atoms-button", "a::button"),
            story_with_id("atoms-card", "b::card"),
        ];

        assert!(duplicate_story_ids(&stories).is_empty());
        assert!(duplicate_story_ids(&[]).is_empty());
    }

    #[test]
    fn discover_components_in_file_extracts_metadata() {
        let dir = temp_dir("dioxus-showcase-discover-file");
        let path = dir.join("button.rs");
        std::fs::write(
            &path,
            r#"
#[showcase(title = "Atoms/Button", tags = ["atoms", "button"])]
#[component]
pub fn Button() -> Element { todo!() }
"#,
        )
        .expect("write file");

        let stories = discover_components_in_file(&dir, &path).expect("discover components");
        assert_eq!(stories.len(), 1);
        assert_eq!(stories[0].id, "atoms-button");
        assert_eq!(stories[0].title, "Atoms/Button");
        assert_eq!(stories[0].tags, vec!["atoms".to_owned(), "button".to_owned()]);
        assert!(stories[0].source_path.contains("button.rs"));
        assert!(Path::new(&stories[0].source_path).is_absolute());
        assert_eq!(stories[0].module_path, "button::Button");
        assert_eq!(stories[0].renderer_symbol, "__dioxus_showcase_render__Button");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn discover_components_in_file_supports_multiline_attribute() {
        let dir = temp_dir("dioxus-showcase-discover-multiline");
        let path = dir.join("card.rs");
        std::fs::write(
            &path,
            r#"
#[showcase(
  title = "Molecules/Card",
  tags = ["molecules", "card"]
)]
#[component]
pub fn Card() -> Element { todo!() }
"#,
        )
        .expect("write file");

        let stories = discover_components_in_file(&dir, &path).expect("discover components");
        assert_eq!(stories.len(), 1);
        assert_eq!(stories[0].id, "molecules-card");
        assert_eq!(stories[0].title, "Molecules/Card");
        assert_eq!(stories[0].tags, vec!["molecules".to_owned(), "card".to_owned()]);
        assert_eq!(stories[0].renderer_symbol, "__dioxus_showcase_render__Card");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn discover_components_defaults_title_to_fn_name() {
        let dir = temp_dir("dioxus-showcase-discover-default-title");
        let path = dir.join("badge.rs");
        std::fs::write(
            &path,
            r#"
#[showcase(tags = ["atoms"])]
#[component]
fn Badge() -> Element { todo!() }
"#,
        )
        .expect("write file");

        let stories = discover_components_in_file(&dir, &path).expect("discover components");
        assert_eq!(stories.len(), 1);
        assert_eq!(stories[0].title, "Badge");
        assert_eq!(stories[0].id, "badge");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn discover_components_includes_story_functions() {
        let dir = temp_dir("dioxus-showcase-discover-story");
        let path = dir.join("button.rs");
        std::fs::write(
            &path,
            r#"
#[story(title = "Button/Primary", tags = ["atoms", "button"])]
fn button_primary() -> Element { todo!() }
"#,
        )
        .expect("write file");

        let stories = discover_components_in_file(&dir, &path).expect("discover components");
        assert_eq!(stories.len(), 1);
        assert_eq!(stories[0].id, "button-primary");
        assert_eq!(stories[0].title, "Button/Primary");
        assert_eq!(stories[0].tags, vec!["atoms".to_owned(), "button".to_owned()]);
        assert_eq!(stories[0].module_path, "button::button_primary");
        assert_eq!(stories[0].renderer_symbol, "__dioxus_showcase_render__button_primary");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn discover_providers_in_file_extracts_ordering() {
        let dir = temp_dir("dioxus-showcase-discover-provider");
        let path = dir.join("provider.rs");
        std::fs::write(
            &path,
            r#"
#[provider(order = -10)]
#[component]
fn Shell(children: Element) -> Element { children }

#[provider]
#[component]
fn Defaulted(children: Element) -> Element { children }
"#,
        )
        .expect("write file");

        let providers = discover_providers_in_file(&dir, &path).expect("discover providers");
        assert_eq!(providers.len(), 2);
        assert_eq!(providers[0].module_path, "provider::Shell");
        assert_eq!(providers[0].wrap_symbol, "__dioxus_showcase_wrap__Shell");
        assert_eq!(providers[0].order, -10);
        // An omitted `order` means 0, matching the runtime default.
        assert_eq!(providers[1].order, 0);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_pre_release_provider_index_spelling_is_rejected() {
        let dir = temp_dir("dioxus-showcase-discover-provider-index");
        let path = dir.join("provider.rs");
        std::fs::write(
            &path,
            r#"
#[provider(index = 2)]
#[component]
fn Shell(children: Element) -> Element { children }
"#,
        )
        .expect("write file");

        let err = discover_providers_in_file(&dir, &path).expect_err("index should be rejected");
        assert!(err.contains("order = <integer>"), "{err}");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn discover_components_skips_target_and_showcase_directories() {
        let dir = temp_dir("dioxus-showcase-discover-root");
        let include_dir = dir.join("examples").join("src");
        let showcase_dir = dir.join("showcase");
        let skip_dir = dir.join("examples").join("target").join("debug");
        std::fs::create_dir_all(&include_dir).expect("create include dir");
        std::fs::create_dir_all(&showcase_dir).expect("create showcase dir");
        std::fs::create_dir_all(&skip_dir).expect("create target dir");
        std::fs::write(
            include_dir.join("lib.rs"),
            r#"
mod ok;
"#,
        )
        .expect("write lib.rs");

        std::fs::write(
            include_dir.join("ok.rs"),
            r#"
#[showcase(title = "Included/Component", tags = ["ok"])]
fn included_component() -> Element { todo!() }
"#,
        )
        .expect("write include component");
        std::fs::write(
            skip_dir.join("skip.rs"),
            r#"
#[showcase(title = "Skipped/Component", tags = ["skip"])]
fn skipped_component() -> Element { todo!() }
"#,
        )
        .expect("write skip component");
        std::fs::write(
            showcase_dir.join("ignored.rs"),
            r#"
#[showcase(title = "Showcase/Internal", tags = ["skip"])]
fn ignored_component() -> Element { todo!() }
"#,
        )
        .expect("write ignored component");

        let mut config = ShowcaseConfig::default();
        config.project.entry_crate = "examples".to_owned();
        let stories = discover_components(&dir, &config).expect("discover components");
        assert_eq!(stories.len(), 1);
        assert_eq!(stories[0].title, "Included/Component");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn discover_components_follows_external_module_graph_only() {
        let dir = temp_dir("dioxus-showcase-discover-module-graph");
        let src_dir = dir.join("crate").join("src");
        std::fs::create_dir_all(src_dir.join("components")).expect("create src tree");

        std::fs::write(
            src_dir.join("lib.rs"),
            r#"
mod components;
"#,
        )
        .expect("write lib.rs");
        std::fs::write(
            src_dir.join("components.rs"),
            r#"
pub mod button;
"#,
        )
        .expect("write components.rs");
        std::fs::write(
            src_dir.join("components").join("button.rs"),
            r#"
#[showcase(title = "Atoms/Button")]
fn Button() -> Element { todo!() }
"#,
        )
        .expect("write reachable module");
        std::fs::write(
            src_dir.join("orphan.rs"),
            r#"
#[showcase(title = "Ignored/Orphan")]
fn Orphan() -> Element { todo!() }
"#,
        )
        .expect("write orphan module");

        let mut config = ShowcaseConfig::default();
        config.project.entry_crate = "crate".to_owned();
        let stories = discover_components(&dir, &config).expect("discover components");

        assert_eq!(stories.len(), 1);
        assert_eq!(stories[0].title, "Atoms/Button");
        assert_eq!(stories[0].module_path, "components::button::Button");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn discover_component_source_files_supports_path_attribute() {
        let dir = temp_dir("dioxus-showcase-discover-path-attr");
        let src_dir = dir.join("crate").join("src");
        std::fs::create_dir_all(src_dir.join("alt")).expect("create alt dir");

        std::fs::write(
            src_dir.join("lib.rs"),
            r#"
#[path = "alt/custom_button.rs"]
mod custom_button;
"#,
        )
        .expect("write lib.rs");
        std::fs::write(
            src_dir.join("alt").join("custom_button.rs"),
            r#"
#[showcase(title = "Atoms/Custom Button")]
fn CustomButton() -> Element { todo!() }
"#,
        )
        .expect("write custom module");

        let mut config = ShowcaseConfig::default();
        config.project.entry_crate = "crate".to_owned();
        let files = discover_component_source_files(&dir, &config).expect("discover files");

        assert_eq!(files.len(), 2);
        assert!(files.iter().any(|path| path.ends_with("lib.rs")));
        assert!(files.iter().any(|path| path.ends_with("alt/custom_button.rs")));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
