# dioxus-showcase-cli

The runtime assembly line.

This crate owns the operational workflow: initialize config, scan the entry crate, validate discovered stories, generate the showcase shell, sync assets, write manifests, run `dx serve` with a rebuild watcher, and export a deployable static site.

## Commands

- `init`: create `DioxusShowcase.toml` and scaffold the generated app.
- `check`: validate config, discovery, ids, and scaffold presence.
- `build`: write manifest and generated runtime files.
- `build --watch`: rebuild when source timestamps change.
- `dev`: rebuild in the background and launch the generated Dioxus app.
- `export`: build a deployable static website of the showcased components.
- `doctor`: print host diagnostics, including the detected `dx` version.

## Key Modules

- `src/discovery.rs`: parse Rust files and extract story/provider metadata.
- `src/scaffold.rs`: write files and sync assets.
- `src/templates.rs`: render the generated source and manifest templates.
- `src/build.rs`: artifact generation and watch loop.
- `src/dev.rs`: `dx serve` orchestration.
- `src/export.rs`: `dx bundle` orchestration and static site assembly.

For the detailed execution path, see [`../../docs/code-reference.md`](../../docs/code-reference.md).
For deployment recipes, see [`../../docs/static-site.md`](../../docs/static-site.md).
