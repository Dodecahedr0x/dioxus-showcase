# AGENTS.md

This file gives coding agents the minimum project context needed to make correct changes in `dioxus-preview` without rediscovering the repository each time.

## Purpose

`dioxus-preview` is a Rust workspace for `dioxus-showcase`, a Storybook-style toolchain for Dioxus. The current repository contains:

- shared data types and config parsing
- procedural macros for showcase/stories/providers
- a facade crate for end users
- a CLI that discovers annotated Rust items, generates runtime artifacts, and launches a Dioxus showcase app
- an `example/` workspace member used as the end-to-end fixture

The top-level `README.md` reflects the current prototype scope more accurately than the RFC in places. Use both, but treat implemented code and tests as the source of truth.

## Read This First

When working in this repository, orient yourself in this order:

1. `README.md`
2. `CONTRIBUTING.md` for the verification commands and which files are generated
3. `Cargo.toml`
4. `crates/dioxus-showcase-cli/src/commands.rs`
5. the crate or module you are changing
6. `example/README.md` if the change affects story authoring or discovery
7. `docs/rfcs/dioxus-showcase.md` only for intended direction, not for assuming already-shipped behavior

## Workspace Layout

Top-level workspace members:

- `crates/dioxus-showcase-core`
  Shared config, manifest, and runtime-facing data structures.
- `crates/dioxus-showcase-macros`
  Procedural macros such as `#[showcase]`, `#[story]`, `#[provider]`, and derives.
- `crates/dioxus-showcase`
  Facade crate that re-exports the user-facing API.
- `crates/dioxus-showcase-cli`
  Main operational crate. Handles init/check/build/dev and scaffold generation.
- `example`
  Example workspace member used to validate annotation discovery and generated showcase output.

Other important paths:

- `DioxusShowcase.toml`
  The repo's own CLI config, checked in and pointing at `example/`. Because it is tracked,
  every command works straight from a clone without running the interactive `init`.
- `CONTRIBUTING.md`
  Prerequisites, pre-PR checklist, generated-file map, and release process.
- `docs/static-site.md`
  How `export` works and how to deploy its output.
- `docs/rfcs/dioxus-showcase.md`
  Product direction and intended architecture.
- `docs/improvement-ideas.md`
  Backlog of remaining work, plus a list of what has already shipped.
- `scripts/set-workspace-version.sh`
  Release helper.
- `scripts/verify-workspace-version.sh`
  Release verification helper.

## Crate Responsibilities

### `dioxus-showcase-core`

Owns durable schema and shared types. Put logic here when it needs to be used by multiple crates or persisted in config/manifest artifacts. Changes here tend to ripple into CLI generation and example fixtures.

Current key files:

- `src/config.rs`
- `src/manifest.rs`
- `src/runtime.rs`

### `dioxus-showcase-macros`

Owns parsing and code generation for procedural macros. Keep parsing behavior deterministic and error messages concrete. Macro changes should usually be accompanied by updates to:

- `crates/dioxus-showcase-macros/tests/macros.rs`
- any example usage in `example/src/`
- CLI discovery behavior if generated metadata assumptions change

### `dioxus-showcase`

This is the stable facade for end users. Favor re-exports and small API shaping here; avoid embedding discovery or file-generation logic in this crate.

### `dioxus-showcase-cli`

This is the most behavior-heavy crate. It currently owns:

- config loading
- interactive `init`
- source discovery over `.rs` files
- validation via `check`
- runtime artifact generation via `build`
- dev loop and `dx serve` integration
- scaffold creation for the showcase app crate

Key files:

- `src/cli.rs`
  Clap argument surface.
- `src/commands.rs`
  Entry-point dispatch and config bootstrapping.
- `src/discovery.rs`
  Source scanning and metadata extraction.
- `src/check.rs`
  Validation rules.
- `src/build.rs`
  Generated artifact writing.
- `src/dev.rs`
  Dev workflow and file watching / regeneration path.
- `src/scaffold.rs`
  Showcase app creation and scaffold maintenance.
- `src/templates.rs`
  Template contents for generated files.

## Actual Current Workflow

The repository is in a prototype state. The implemented command flow is:

1. `init`
   Writes `DioxusShowcase.toml` and creates a runnable showcase app crate.
2. `check`
   Loads config, discovers annotations, and validates duplicate IDs / configuration issues.
3. `build`
   Generates runtime artifacts such as the manifest and `generated.rs`.
4. `dev`
   Rebuilds generated artifacts and launches the Dioxus app through `dx serve`.
5. `export`
   Rebuilds generated artifacts and compiles them into a deployable static site through `dx bundle`.
6. `doctor`
   Prints basic host diagnostics, including the detected `dx` version.

Do not assume the RFC’s target architecture is fully implemented. For example, some prototype behavior still relies on generated source and include-like integration patterns.

## Generated Artifacts And Fixtures

There are generated files in this repository and in the example app. Before editing, determine whether a file is source-of-truth or generated output.

Common generated artifacts:

- `example/showcase/src/generated.rs`
- `example/showcase/src/main.rs`
- `example/showcase/Dioxus.toml`
- `target/showcase/showcase.manifest.json`

The one hand-maintained file in `example/showcase/` is its `Cargo.toml`; see
`CONTRIBUTING.md` for why it is checked in while the rest of the directory is gitignored.

Rules:

- Prefer changing the generator rather than hand-editing generated output.
- If a code change legitimately alters generated output, regenerate it and keep the fixture consistent.
- Avoid noisy churn in generated files when the semantic output is unchanged.

## Example Project Conventions

`example/` is not just sample code; it is the easiest end-to-end validation path in the repo.

Use it when changes affect:

- annotation syntax
- macro expansion assumptions
- story discovery
- provider ordering
- manifest shape
- scaffold/runtime integration

The example currently demonstrates:

- `#[showcase]` components
- `#[story]` functions
- `#[provider(index = ...)]` wrapper ordering
- generated showcase app output under `example/showcase/`

## Edit Guidelines

### When changing discovery or build logic

- Read both `discovery.rs` and `build.rs`.
- Check what `example/src/` contains, because the example is the practical discovery fixture.
- Preserve deterministic ordering. Manifest and generated-source stability matter for CI and reviewability.
- Treat duplicate IDs and malformed annotations as user-facing validation problems, not internal-only failures.

### When changing macros

- Keep macro parsing tolerant enough for multi-line attribute forms if already supported.
- Prefer precise compile-time diagnostics over silent behavior changes.
- Verify the facade crate still re-exports the necessary user API.

### When changing config or manifest schema

- Update `dioxus-showcase-core` first.
- Then update CLI readers/writers.
- Then update example artifacts or tests that encode the schema.
- Be careful with backwards-compatibility assumptions; this project is early-stage, but generated outputs still need to stay coherent.

### When changing the scaffolded app

- Look in `scaffold.rs` and `templates.rs` together.
- Remember that the generated showcase app is expected to run via `dx serve`.
- Keep the scaffold usable for local development without manual patching after `init`.

## Commands

Run commands from the repository root unless a task clearly requires otherwise. No setup
step is needed: `DioxusShowcase.toml` is checked in, so `check`, `build`, `dev`, and
`export` all work directly from a clone. Do not run `init` unless you are specifically
testing that flow, because it overwrites that config.

Useful commands:

```bash
cargo test --workspace --all-targets
cargo run -p dioxus-showcase-cli -- check
cargo run -p dioxus-showcase-cli -- build
cargo run -p dioxus-showcase-cli -- dev
cargo run -p dioxus-showcase-cli -- export
```

Release-oriented commands from the existing docs:

```bash
./scripts/set-workspace-version.sh X.Y.Z
./scripts/verify-workspace-version.sh
```

If you changed macros, discovery, manifests, or scaffolding, `cargo test --workspace --all-targets` is the default verification bar.

## Validation Expectations

Match the verification depth to the change:

- Small refactor with no behavior change:
  Run targeted tests or at least `cargo check`.
- Macro or parsing change:
  Run macro tests and workspace tests.
- Discovery/build/generation change:
  Run workspace tests and the CLI flow against the example when feasible.
- Scaffold/dev workflow change:
  Validate the CLI path, not just unit tests.

Good practical checks:

```bash
cargo test --workspace --all-targets
cargo run -p dioxus-showcase-cli -- check
cargo run -p dioxus-showcase-cli -- build
```

If verification is skipped, say exactly what was not run.

## Rust Conventions

- Edition is `2021`.
- Workspace lints are enabled.
- `unsafe_code` is forbidden at the workspace level.
- Prefer small, explicit data structures and deterministic iteration over clever abstractions.
- Preserve human-readable error messages in CLI code; this tool is directly user-facing.

## What To Avoid

- Do not treat RFC text as proof that a feature already exists.
- Do not hand-edit generated files unless the task is specifically about the generated output itself.
- Do not move logic into the facade crate that belongs in `core`, `macros`, or `cli`.
- Do not introduce unnecessary workspace-wide dependency expansion for one localized feature.
- Do not make discovery output nondeterministic.

## Decision Heuristics

Use these placement rules when deciding where code belongs:

- Shared serializable types or config parsing:
  `dioxus-showcase-core`
- Attribute parsing or compile-time codegen:
  `dioxus-showcase-macros`
- End-user re-exports:
  `dioxus-showcase`
- Filesystem, scanning, generation, CLI UX:
  `dioxus-showcase-cli`

## Documentation Hygiene

If you change behavior that affects end users or contributors, update the nearest relevant doc:

- `README.md` for supported workflows and command behavior
- `example/README.md` for story-authoring conventions
- RFC/docs only when the design direction itself changed

Keep docs aligned with implemented behavior. This repository is small enough that stale docs become misleading quickly.
