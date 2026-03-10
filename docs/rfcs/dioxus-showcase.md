# RFC: dioxus-showcase

- Status: Draft
- Last updated: 2026-03-09
- Owners: UI Platform

## 1. Summary

`dioxus-showcase` is a Storybook-style toolchain for Dioxus that supports isolated component rendering, args/controls, interactive documentation, and static publishing.

## 2. Goals

1. Isolated development environment for Dioxus components.
2. Minimal setup in existing projects.
3. Deterministic story discovery for CI.
4. Static build output suitable for deployment on any static host.
5. Extensible addon system for controls/docs/backgrounds/viewports.

## 3. Non-Goals

1. Replace E2E testing.
2. Execute untrusted code or dynamic plugin binaries at runtime.
3. Provide full design-tool synchronization in v1.

## 4. Scope

### 4.1 Included in v1

1. Story declaration macros.
2. Registry and manifest generation.
3. CLI: `init`, `dev`, `build`, `check`, `doctor`.
4. Static shell generation for showcase UI.
5. Config file with stable schema.

### 4.2 Deferred

1. Visual regression addon.
2. Snapshot approvals.
3. Rich markdown/docgen from Rust AST.

## 5. Terminology

1. Story: a function that renders one component state.
2. Args: serializable input values mapped to story controls.
3. Decorator: wrapper around story rendering for providers/theme/layout.
4. Manifest: build artifact containing normalized story metadata.

## 6. User Experience

### 6.1 Authoring stories

```rust
use dioxus::prelude::*;
use dioxus_showcase::prelude::*;

#[derive(Clone, PartialEq, StoryArgs)]
struct ButtonProps {
    label: String,
    disabled: bool,
}

#[component]
fn Button(props: ButtonProps) -> Element {
    rsx! { button { disabled: props.disabled, "{props.label}" } }
}

#[story(title = "Atoms/Button/Default")]
fn button_default() -> Element {
    rsx! { Button { label: "Click me".to_string(), disabled: false } }
}
```

### 6.2 CLI flow

1. `dioxus-showcase init`
2. `dioxus-showcase dev`
3. `dioxus-showcase build`

## 7. Workspace Layout

Recommended project crates:

1. `dioxus-showcase-core`
2. `dioxus-showcase-macros`
3. `dioxus-showcase` (facade)
4. `dioxus-showcase-cli`

## 8. Architecture

1. Discovery phase: collect story symbols and metadata.
2. Registry phase: normalize IDs and enforce uniqueness.
3. Runtime phase: load selected story in isolated mount root.
4. Build phase: produce `index.html` and `stories.manifest.json`.

## 9. APIs

### 9.1 Macro API

1. `#[story(title = "...")]`
2. `#[derive(StoryArgs)]`

### 9.2 Core API

1. `StoryDefinition`
2. `StoryManifest`
3. `ShowcaseRegistry`

### 9.3 Addon API (target)

```rust
pub trait ShowcaseAddon {
    fn id(&self) -> &'static str;
    fn register(&self, app: &mut ShowcaseApp);
}
```

## 10. Config Schema

`DioxusShowcase.toml`

```toml
[project]
name = "my-ui"
entry_crate = "web"

[dev]
port = 6111
host = "127.0.0.1"

[build]
out_dir = "target/showcase"
base_path = "/"
```

## 11. Story Metadata Schema

Each story record includes:

1. `id`
2. `title`
3. `module_path`
4. `tags`

`stories.manifest.json` includes `schema_version` and `stories[]`.

## 12. Integration Contract

### 12.1 Existing Dioxus projects

1. Add `dioxus-showcase` and `dioxus-showcase-macros` dependencies.
2. Add `DioxusShowcase.toml`.
3. Create at least one `#[story]` function.
4. Run `dioxus-showcase dev` locally.
5. Run `dioxus-showcase build` in CI.

### 12.2 Zero-refactor requirement

Stories can live next to existing components or in dedicated `*.stories.rs` modules.

## 13. Validation Rules (`check`)

1. Config file must exist and parse.
2. Story IDs must be unique.
3. Required metadata fields must be present.
4. Build output path must be writable.

## 14. Reliability and Performance Targets

1. Startup under 3s for 200 stories on typical developer machines.
2. Story switch under 150ms after warm start.
3. Broken story isolated via boundary and does not crash shell.

## 15. Security

1. No remote plugin execution.
2. Validate and escape metadata rendered in UI.
3. Restrict runtime to statically linked Rust artifacts.

## 16. Compatibility

1. Rust MSRV declared in Cargo metadata.
2. Dioxus version compatibility matrix maintained per release.
3. SemVer across crates.

## 17. Rollout Plan

1. Milestone 1: core types + CLI skeleton + init/check/build.
2. Milestone 2: real discovery and manifest generation.
3. Milestone 3: web UI shell and story canvas runtime.
4. Milestone 4: controls/docs addons.
5. Milestone 5: CI templates and static hosting guides.

## 18. Acceptance Criteria

1. Existing project setup in under 15 minutes.
2. 100+ stories discoverable and renderable in one workspace.
3. Deterministic manifest output in CI.
4. Static output deployable without custom server logic.

