# dioxus-showcase-core

The stable data layer for the workspace.

This crate keeps the types that every other member shares: parsed config, manifest models, provider definitions, registry state, and the title-to-tree navigation builder. If you need to change what the CLI writes, what the runtime reads, or what metadata the macros target, start here.

## What Lives Here

- `src/config.rs`: `DioxusShowcase.toml` parsing, defaults, and serialization.
- `src/manifest.rs`: `StoryDefinition` and `StoryManifest`.
- `src/runtime.rs`: provider metadata, registry helpers, and sidebar tree construction.

## Why It Exists

The CLI, facade crate, generated app, and macro output all need the same vocabulary. Keeping those types in one crate reduces drift and makes the generated/runtime boundary explicit.

For the full file-by-file reference, see [`../../docs/code-reference.md`](../../docs/code-reference.md).
