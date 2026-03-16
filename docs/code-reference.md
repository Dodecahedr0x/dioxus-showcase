# Dioxus Showcase Code Reference

This document is the exhaustive reference for the current repository state. It explains what every code-bearing file does, how the crates fit together, and how the generated showcase runtime is assembled.

## Workspace At A Glance

`dioxus-preview` is a Cargo workspace for a Storybook-style prototype aimed at Dioxus applications.

The workspace is split into five members:

1. `crates/dioxus-showcase-core`
   Shared data types, config parsing, manifests, and navigation helpers.
2. `crates/dioxus-showcase-macros`
   Procedural macros that annotate components, stories, and providers.
3. `crates/dioxus-showcase`
   Facade crate re-exporting the public API end users consume.
4. `crates/dioxus-showcase-cli`
   Discovery, code generation, scaffolding, asset syncing, and local dev orchestration.
5. `example`
   A small workspace member demonstrating `#[showcase]`, `#[story]`, and `#[provider]`.

The prototype flow is:

1. Authors annotate Dioxus functions with `#[showcase]`, `#[story]`, and `#[provider]`.
2. The CLI scans the configured entry crate source tree with `syn`.
3. Discovery builds `StoryDefinition` and `ProviderDefinition` values.
4. Build/scaffold code writes:
   - `target/showcase/showcase.manifest.json`
   - `showcase/src/generated.rs`
   - `showcase/src/main.rs`
   - `showcase/assets/showcase.css`
5. The generated showcase app imports the entry crate, creates routes, assembles providers, and renders stories.

## Top-Level Files

### `Cargo.toml`

Defines the workspace members, shared package metadata, and shared dependencies.

Key points:

- Versioning is centralized through `workspace.package.version`.
- The workspace forbids `unsafe_code`.
- The CLI, macro crate, facade crate, and example all share dependency versions from here.

### `Cargo.lock`

Lockfile for the workspace dependency graph.

### `rust-toolchain.toml`

Pins the Rust toolchain used for local development and CI consistency.

### `rustfmt.toml`

Repository formatting rules.

### `README.md`

Top-level project summary and user-facing quickstart. This was rewritten to act as a concise index into the richer docs in this file.

## Scripts

### `scripts/set-workspace-version.sh`

Release helper that updates the workspace version before tagging and publishing.

### `scripts/verify-workspace-version.sh`

Release helper that verifies the tag/version alignment expected by the publish workflow.

## Existing Design Docs

These are not executable code, but they define intent and future direction:

- `docs/rfcs/dioxus-showcase.md`
- `docs/improvement-ideas.md`
- `docs/worktree-tracks.md`

They should be read as planning material, while this file documents current implementation.

## Crate: `dioxus-showcase-core`

Purpose: hold all data types that need to be stable across macro expansion, discovery, generation, and runtime rendering.

### `crates/dioxus-showcase-core/src/lib.rs`

The crate entry point.

Responsibilities:

- Exposes `config`, `manifest`, and `runtime` modules.
- Re-exports all public structs and helpers so downstream crates can depend on a compact API surface.

### `crates/dioxus-showcase-core/src/config.rs`

Owns the `DioxusShowcase.toml` model.

Types:

- `ShowcaseProjectConfig`
  - `name`: logical project name.
  - `entry_crate`: crate path scanned for annotated code.
  - `showcase_crate`: generated app crate path.
- `ShowcaseDevConfig`
  - `port`: preferred dev server port.
  - `host`: preferred bind host.
- `ShowcaseBuildConfig`
  - `out_dir`: artifact output directory.
  - `base_path`: reserved route base path setting.
- `ShowcaseConfig`
  - top-level container for all sections.

Behavior:

- Implements defaults for all config sections.
- Serializes to TOML with `as_toml_string`.
- Writes a default file via `write_default_if_missing`.
- Parses config from a file or string.
- Uses `deny_unknown_fields`, which makes typo detection strict.

Tests cover:

- TOML round-tripping.
- custom values.
- malformed syntax.
- invalid ports.
- unknown fields.
- partial configs filling defaults.
- write-once behavior.

### `crates/dioxus-showcase-core/src/manifest.rs`

Defines the serialized manifest format the CLI writes out.

Types:

- `StoryDefinition`
  - `id`: slug used for routing and duplicate detection.
  - `title`: human-readable hierarchical story path.
  - `source_path`: absolute path to the Rust file where discovery found the story.
  - `module_path`: Rust module path used to call generated helpers.
  - `renderer_symbol`: generated renderer symbol name.
  - `tags`: arbitrary string tags for filtering.
- `StoryManifest`
  - `schema_version`
  - `stories`

Behavior:

- `new(schema_version)` creates an empty manifest.
- `add_story` appends an entry.
- `to_json` serializes with `serde_json`.

Tests validate JSON escaping and round-tripping.

### `crates/dioxus-showcase-core/src/runtime.rs`

Contains in-memory runtime structures used by the showcase shell.

Types:

- `StoryEntry`
  Wraps a `StoryDefinition` plus a static renderer symbol string.
- `ProviderDefinition`
  Stores provider discovery output: source, module path, wrapper symbol, and wrap order index.
- `StoryNavigationNode`
  Tree node used to render the sidebar hierarchy.
- `StoryTreeEntry`
  Trait abstraction used to build navigation from multiple story-like types.
- `ShowcaseRegistry`
  Minimal story collection helper that can emit a manifest.

Functions:

- `build_story_navigation`
  Converts slash-delimited titles into a nested tree.
- `split_story_title`
  Splits on `/`, trims whitespace, and removes empty segments.
- `insert_story_node` and `insert_story_child`
  Build the navigation tree recursively.

Important runtime behavior:

- A title can be both a leaf and a branch. `Atoms` and `Atoms/Button` can coexist.
- Navigation ordering is insertion-based, not explicitly sorted inside the helper.

Tests verify:

- registry count and manifest output.
- grouped navigation.
- branch-and-leaf coexistence.

## Crate: `dioxus-showcase`

Purpose: public facade crate for end users.

### `crates/dioxus-showcase/src/lib.rs`

This is the main user-facing API.

Exports:

- `prelude` for common imports.
- `core` re-export.
- `macros` re-export.

Public concepts:

- `StoryProvider`
  Function pointer type for global wrappers applied around every story.
- `StoryArg`
  Produces a single default value for one parameter type.
- `StoryArgs`
  Produces one or more values for an aggregate type. There is a blanket impl from `StoryArg`.
- `StoryProps`
  Produces one or more named variants for a props type.
- `StoryVariant<T>`
  Holds an optional display name plus a value.
- `GeneratedStory`
  The unit the macro-generated constructors return: metadata plus a render closure.
- `ShowcaseStoryFactory`
  Trait implemented by macro-generated factory structs.
- `StoryPreviewContent`
  Dioxus component that reads the registered provider chain from context and wraps the story preview in reverse order, so lower-index providers become outer wrappers.

Default `StoryArg` implementations:

- primitive numerics and `bool`
- `char`
- `&'static str`
- `String`
- `Option<T>`
- `Vec<T>`
- `Element`
- `EventHandler<T>`

Notable helper:

- `slugify_title`
  Slug rules are intentionally simple: lowercase ASCII alphanumerics are preserved, all other runs collapse to one `-`, and leading/trailing dashes are trimmed.

Tests assert:

- the traits are implementable.
- prelude re-exports are wired correctly.
- `Element` defaults render valid placeholder markup.

## Crate: `dioxus-showcase-macros`

Purpose: procedural macros that generate runtime registration code next to user-authored story functions and components.

### `crates/dioxus-showcase-macros/src/lib.rs`

Registers the macro entry points:

- `#[story]`
- `#[showcase]`
- `#[provider]`
- `#[derive(StoryProps)]`

This file is mostly API declaration and macro-level docs.

### `crates/dioxus-showcase-macros/src/derive_story_props.rs`

Implements `#[derive(StoryProps)]`.

Behavior:

- Parses a derive input.
- Generates `StoryArg` and `StoryProps` impls requiring `Default`.
- Emits a single unnamed variant using `Default::default()`.

This is intentionally minimal. It exists to make aggregate props and enums easy to expose as stories without hand-written trait impls.

### `crates/dioxus-showcase-macros/src/provider.rs`

Expands `#[provider(index = ...)]`.

What it accepts:

- a free function component
- one explicit `children` parameter
- any additional named parameters whose types implement `StoryArg`

Generated output:

- preserves the original function.
- emits a wrapper function named `__dioxus_showcase_wrap__<ComponentName>`.
- binds non-`children` props from `StoryArg::story_arg()`.
- returns `rsx!` that mounts the provider with both synthesized props and the provided child element.

Guardrails:

- rejects receivers.
- rejects non-identifier patterns.
- requires exactly one `children` parameter.
- only supports `index = <integer>` in macro metadata.

### `crates/dioxus-showcase-macros/src/showcase.rs`

Expands `#[showcase(...)]` for Dioxus components.

Accepted shapes:

1. zero-arg component.
2. single `props` argument implementing `StoryProps`.
3. multiple named parameters implementing `StoryArg`.

Generated items:

- original component function.
- optional hidden controls component for interactive arguments.
- hidden factory struct `__dioxus_showcase_factory__<Name>`.
- hidden renderer function `__dioxus_showcase_render__<Name>`.
- hidden story constructor function `__dioxus_showcase_story__<Name>`.

Branch behavior:

- zero args:
  one story, direct render.
- single `props` argument:
  one or more stories generated from `StoryProps::stories()`.
  named variants append `/<VariantName>` to the base title.
- multiple simple args:
  one generated story plus a controls panel built with signals.

Metadata behavior:

- title defaults to the component function name.
- tags are copied onto each generated `StoryDefinition`.
- story id is derived from the title slug.

### `crates/dioxus-showcase-macros/src/story.rs`

Expands `#[story(...)]` for hand-authored named story functions.

Accepted shapes:

- zero-arg function returning something renderable in `rsx!`.
- parameterized function where each parameter type implements `StoryArg`.

Generated items mirror `#[showcase]`:

- original function.
- optional hidden controls component for interactive args.
- hidden factory struct.
- hidden renderer.
- hidden story constructor.

Behavior:

- zero-arg stories are wrapped inside a standard frame.
- parameterized stories get the same controls infrastructure as multi-arg `#[showcase]`.
- `component = ...` and `name = ...` are rejected. The current API requires a full `title = "..."`.

### `crates/dioxus-showcase-macros/src/utils.rs`

Macro support functions.

Core responsibilities:

- detect whether a component uses a single `props` parameter.
- convert function parameters into state bindings, render args, props syntax, and optional form controls.
- render a standard story frame and controls layout.
- parse macro metadata.
- slugify titles.

Important helpers:

- `story_arg_bindings`
  Decides whether each argument becomes:
  - a live Dioxus control, or
  - a static default value sourced from `StoryArg`.
- `render_story_control`
  Generates controls only for:
  - `String`
  - `bool`
  - numeric primitives
- `parse_showcase_meta`
  Shared parser for `title`, `component`, `name`, `tags`, and `index`.

Current limitation:

- Only a narrow set of argument types become interactive controls. All others still work if they implement `StoryArg`, but they render with default values and no UI control.

### `crates/dioxus-showcase-macros/tests/macros.rs`

Integration-style tests that validate macro expansion behavior in compiled Rust rather than by inspecting only token streams.

Coverage includes:

- `#[story]` preserving original functions.
- title/id generation.
- controlled stories.
- `#[showcase]` on zero-arg, aggregate-props, named-props, multi-arg, and `Element`-arg components.
- `#[provider]` wrapper generation.
- `#[derive(StoryProps)]`.

## Crate: `dioxus-showcase-cli`

Purpose: command-line tooling that ties configuration, discovery, generation, scaffolding, and dev server orchestration together.

### `crates/dioxus-showcase-cli/src/main.rs`

CLI process entry point.

Behavior:

- parses CLI args with `clap`.
- prints help when no subcommand is supplied.
- delegates execution to `commands::run`.
- exits non-zero on error.

### `crates/dioxus-showcase-cli/src/cli.rs`

Pure argument definitions.

Commands:

- `init`
- `dev`
- `build`
- `check`
- `doctor`

Additional args:

- `BuildArgs { watch: bool }`

### `crates/dioxus-showcase-cli/src/commands.rs`

Top-level command dispatch plus interactive init flow.

Key functions:

- `run`
  Routes parsed commands to concrete handlers.
- `load_config`
  Reads `DioxusShowcase.toml` from the repo root.
- `cmd_init`
  Prompts for config values, writes the TOML file, and scaffolds the generated showcase app.
- `prompt_for_config`
  Builds defaults from current workspace context or prior config values.
- `default_entry_crate`
  Heuristically prefers:
  1. current directory if it looks like a crate,
  2. `examples/basic`,
  3. fallback `web`.
- `prompt`
  Small stdin/stdout helper.

The init command is intentionally interactive. The other commands are non-interactive.

### `crates/dioxus-showcase-cli/src/check.rs`

Fast validation command.

What it does:

- loads config.
- discovers stories.
- sorts by title.
- rejects duplicate ids.
- verifies the generated showcase app crate exists.

It does not write artifacts.

### `crates/dioxus-showcase-cli/src/build.rs`

Artifact generation command and file watching logic.

Key functions:

- `cmd_build`
  Performs one build and optionally enters watch mode.
- `rebuild_showcase_artifacts`
  The core pipeline:
  1. discover stories
  2. discover providers
  3. sort stories by title
  4. validate duplicate ids
  5. write generated artifacts
- `watch_and_rebuild`
  Polling loop using modification timestamps.
- `latest_source_stamp`
  Tracks the newest timestamp across discovered source files and `DioxusShowcase.toml`.

Notable design choice:

- File watching is implemented with timestamp polling rather than filesystem notifications, which keeps dependencies small but is less responsive and less precise than a native watcher.

### `crates/dioxus-showcase-cli/src/dev.rs`

Local development workflow.

What `cmd_dev` does:

1. loads config.
2. finds an available port near the preferred port.
3. rebuilds showcase artifacts once.
4. spawns the rebuild watcher thread.
5. runs `dx serve --web` in the generated showcase app directory.
6. stops the watcher when the subprocess exits.

Helper:

- `find_available_port`
  Probes a bounded host/port range using `TcpListener::bind`.

Important assumption:

- The Dioxus CLI binary `dx` must already be installed and reachable in `PATH`.

### `crates/dioxus-showcase-cli/src/discovery.rs`

Repository scanning and AST analysis.

This is the most important CLI module because it turns arbitrary Rust modules into structured showcase metadata.

Public functions:

- `discover_components`
  Walks all reachable Rust files and extracts `StoryDefinition` values for both `#[showcase]` and `#[story]`.
- `discover_providers`
  Same traversal for `#[provider]`.
- `discover_component_source_files`
  Resolves the entry crate, follows `mod` declarations, and returns all reachable Rust sources.
- `validate_component_ids`
  Rejects duplicate route ids.
- `showcase_story_symbol`
  Converts renderer symbol names into generated story constructor names.
- `slugify_title`
  Shared slug algorithm.

Traversal logic:

- starts from `src/lib.rs` and/or `src/main.rs`.
- parses each file with `syn`.
- follows external modules:
  - `mod foo;` -> `foo.rs`
  - `mod foo;` -> `foo/mod.rs`
  - respects `#[path = "..."]`
- deduplicates visited files by canonical path.

Extraction logic:

- a function is a story source when it has `#[showcase]` or `#[story]`.
- a function is a provider when it has `#[provider]`.
- nested inline modules are traversed recursively.
- module paths are reconstructed relative to the configured entry crate `src/` directory.

Metadata parsing:

- `title`
- `tags`
- `index` for providers

Compatibility note:

- the discovery parser still recognizes old fields like `component` and `name` only to produce a helpful error for story attributes. The actual supported story API is title-based.

Tests cover:

- source file detection.
- symbol naming.
- slug normalization.
- duplicate ids.
- single-file extraction.
- multiline attributes.
- default titles.
- story function discovery.
- provider ordering.
- recursive module traversal.
- `#[path = "..."]` modules.

### `crates/dioxus-showcase-cli/src/scaffold.rs`

Owns output file creation and asset syncing.

Key functions:

- `showcase_app_dir`
  Resolves the generated app directory from config.
- `write_artifacts`
  Writes the manifest, refreshes scaffold files, syncs stylesheets, rewrites generated runtime, and rewrites the showcase shell `main.rs`.
- `ensure_showcase_app_scaffold`
  Ensures the generated crate directory structure and seed files exist.
- `sync_entry_assets_and_collect_stylesheets`
  Copies source assets from the entry crate into the showcase app and returns the stylesheet list used by the generated shell.
- `stable_generation_token`
  Computes a deterministic hash from stories and providers.
- `copy_dir_recursive`
  Recursive filesystem copy helper.
- `collect_stylesheets`
  Collects all `.css` files into `/assets/...` URL paths.

Important behavior:

- `Cargo.toml` for the generated app is only written if missing.
- `Dioxus.toml`, `src/main.rs`, `src/generated.rs`, and `assets/showcase.css` are overwritten on each scaffold/update pass.
- Generated runtime stability is tied to metadata hashing, not file mtimes.

Tests validate:

- main regeneration replacing stale content.
- asset stylesheet inclusion.
- stable generation tokens.
- golden manifest and generated runtime output.

### `crates/dioxus-showcase-cli/src/templates.rs`

Turns structured discovery output into concrete source files.

Embedded templates:

- `generated_runtime.rs.hbs`
- `showcase_main.rs.hbs`
- `showcase_cargo.toml.hbs`
- `showcase_dioxus.toml.hbs`
- `showcase_app.css`

Functions:

- `render_generated_runtime_rs`
  Builds the runtime source that imports entry-crate story constructor helpers and provider wrappers.
- `render_showcase_app_main_rs`
  Renders the route shell and sidebar UI.
- `render_showcase_app_cargo_toml`
  Creates the generated showcase app manifest and points it back to the entry crate with a relative path dependency.
- `render_showcase_app_dioxus_toml`
  Creates Dioxus app metadata.
- `render_showcase_app_css`
  Returns the static stylesheet template.

Internal helpers:

- `render_template`
  Handlebars rendering with escaping disabled because the inputs are already escaped for their target format.
- `render_story_path`
  Converts a discovered `module_path` plus renderer symbol into a call path for the generated story constructor.
- `render_provider_paths` and `render_provider_path`
  Build invocation paths for provider wrappers.
- `discover_entry_crate_package_name`
  Reads `[package].name` from the configured entry crate `Cargo.toml`.
- `relative_dependency_path`
  Computes the showcase app dependency path to the entry crate.
- escaping helpers for TOML and Rust string literals.

### `crates/dioxus-showcase-cli/src/templates/generated_runtime.rs.hbs`

Template for `showcase/src/generated.rs`.

Generated contents:

- `SHOWCASE_GENERATION` constant.
- `ShowcaseComponentDefinition`.
- `story_providers()` returning ordered provider wrapper function pointers.
- `showcase_components()` that:
  - calls every generated story constructor in the entry crate,
  - asserts no duplicate ids exist at runtime,
  - returns a vector of renderable showcase component definitions.

### `crates/dioxus-showcase-cli/src/templates/showcase_main.rs.hbs`

Template for the generated showcase shell application.

Main responsibilities:

- defines routes for home, story pages, and not-found pages.
- provides theme and tag-filter context.
- imports all synced stylesheets with `document::Stylesheet`.
- builds sidebar navigation from discovered stories.
- renders tag chips and route-aware tree navigation.
- renders selected stories inside an `ErrorBoundary`.

Key UI behaviors:

- theme toggles between light and dark modes.
- tags can filter the navigation tree.
- parent tree branches auto-open when they contain the active story.
- route `"/component/:id"` maps directly to generated story ids.

### `crates/dioxus-showcase-cli/src/templates/showcase_cargo.toml.hbs`

Template for the generated showcase app package manifest.

Important details:

- marks the crate `publish = false`.
- creates an empty `[workspace]` table so the generated crate can live inside the repo without inheriting the parent workspace unexpectedly.
- depends on:
  - `dioxus`
  - `dioxus-showcase`
  - `showcase_entry`, an alias pointing at the scanned entry crate

### `crates/dioxus-showcase-cli/src/templates/showcase_dioxus.toml.hbs`

Small Dioxus config template setting app name, default platform, and output directory.

### `crates/dioxus-showcase-cli/src/templates/showcase_app.css`

Static CSS for the generated showcase shell.

Style areas:

- app shell colors and theme tokens.
- sidebar layout.
- theme toggle.
- tag filters.
- navigation tree.
- story canvas and controls panel.
- error boundary view.
- mobile layout collapse below `860px`.

### `crates/dioxus-showcase-cli/src/testdata/build_golden_manifest.json`

Golden output fixture used by scaffold/build tests.

### `crates/dioxus-showcase-cli/src/testdata/build_golden_generated.rs`

Golden runtime fixture used to detect generation regressions.

## Crate: `example`

Purpose: end-to-end demonstration of the current macro API and discovery expectations.

### `example/src/lib.rs`

Exports the example module and provides crate-level docs.

Also contains tests that call the macro-generated story constructor functions directly to verify:

- `#[showcase]` generates a default story for a component.
- `#[story]` generates metadata for named story functions.
- tags and ids are derived as expected.

### `example/src/button_variants.rs`

The actual demo surface.

Items:

- `ExampleStoryShell`
  A provider wrapper with `index = 0`.
- `PillButtonControllable`
  A `#[showcase]` component using two controlled arguments: `String` and `bool`.
- `pill_button_primary`
  A `#[story]` function for a named primary variant.
- `pill_button_disabled`
  A `#[story]` function for a disabled variant.

This file demonstrates all three public macro types together in one place.

### `example/README.md`

Human-oriented quickstart for the example crate. It was updated to match the current title-based `#[story]` API.

## End-To-End Runtime Path

The shortest accurate mental model of the whole project is:

1. Author code lives in a normal Dioxus crate.
2. Procedural macros generate hidden renderer/factory/helper symbols next to user functions.
3. CLI discovery scans source files and reconstructs the module path to those generated helpers.
4. The build command renders a generated app that imports the entry crate as `showcase_entry`.
5. Runtime code calls generated story constructor functions to obtain:
   - `StoryDefinition`
   - `render` closure
6. The generated app builds a sidebar tree and route table from story titles and ids.
7. Providers wrap rendered story content through `StoryPreviewContent`.

## Current Constraints

These are implementation constraints visible in the current code, not aspirational ones:

- discovery is source-based and requires reachable Rust modules from `lib.rs` or `main.rs`.
- the generated showcase app depends on being able to import the entry crate normally.
- simple control widgets exist only for `String`, `bool`, and numeric argument types.
- provider arguments besides `children` must be synthesizable through `StoryArg`.
- build watching is timestamp polling.
- `base_path` is stored in config but not yet wired into route generation.

## Suggested Reading Order

If you need to change behavior in this repo:

1. Start with `crates/dioxus-showcase/src/lib.rs` to understand the public API.
2. Read `crates/dioxus-showcase-macros/src/showcase.rs`, `story.rs`, and `utils.rs`.
3. Read `crates/dioxus-showcase-cli/src/discovery.rs`.
4. Read `crates/dioxus-showcase-cli/src/scaffold.rs` and `templates.rs`.
5. Inspect the template files to understand the final generated application shape.
