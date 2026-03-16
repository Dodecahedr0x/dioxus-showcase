# Dioxus Showcase

Storybook-style tooling for Dioxus.

## Current state

This repository contains a scaffold:

1. RFC spec in `docs/rfcs/dioxus-showcase.md`
2. Core data model crate
3. Macro crate with placeholder macros
4. Facade crate for end users
5. CLI prototype (`dioxus-showcase`)

## Quickstart

```bash
cargo run -p dioxus-showcase-cli -- init
cargo run -p dioxus-showcase-cli -- check
cargo run -p dioxus-showcase-cli -- build
cargo run -p dioxus-showcase-cli -- dev
```

## Prototype commands

1. `init`: interactive prompt to write `DioxusShowcase.toml`, then creates the runnable `showcase/` Dioxus app crate
2. `check`: validates config + discovers `#[showcase_component]` annotations + checks duplicate IDs
3. `build`: writes `target/showcase/showcase.manifest.json` and updates `showcase/src/generated.rs`
4. `dev`: launches via `dx serve` and keeps regenerating showcase artifacts when source files change
5. `doctor`: prints host diagnostics

## Next implementation steps

1. Replace include-based runtime generation with stable crate/module registration.
2. Add args/controls integration to generated runtime app.
3. Add per-story decorators on top of global provider hooks.
4. Add visual regression and a11y check addons.

## Runtime bridge (prototype)

`build` updates `showcase/src/generated.rs` with generated component render dispatch stubs:

```rust
mod generated;
```

The app crate uses Dioxus Router to provide a dedicated route per discovered component:

```rust
// /showcase/src/main.rs
#[derive(Routable)]
enum Route {
    #[route("/")]
    Home {},
    #[route("/component/:id")]
    Component { id: String },
}
```

Current constraints:

1. Annotated showcase components must be zero-prop components.
2. Source files are included via `include!`, so imports in annotated files must resolve in the showcase app crate.
3. `#[showcase_component(...)]` supports both single-line and multi-line forms.
4. The generated app layout is responsive and route-based for web and mobile-friendly usage.

## Showcase app crate

`init` creates a runnable Dioxus app crate at `showcase/`:

```bash
cd showcase
dx serve --web --port 6111 --addr 127.0.0.1
```

## Example workspace member

An end-to-end example workspace member is available at:

- `examples/basic`

It includes `#[showcase_component]` annotations and can be discovered by `dioxus-showcase` when run from the repository root.

## Release flow

Release tags must point at a commit that already contains the target workspace version in `Cargo.toml`.

```bash
./scripts/set-workspace-version.sh X.Y.Z
cargo test --workspace --all-targets --all-features
git add Cargo.toml
git commit -m "chore(release): bump workspace version to X.Y.Z"
git tag vX.Y.Z
git push origin main --follow-tags
```

The publish workflow verifies that the pushed tag version matches the committed workspace manifest before running `cargo publish`.
