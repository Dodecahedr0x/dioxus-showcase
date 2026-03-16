# Dioxus Showcase

Storybook-style tooling for Dioxus, split into macros, shared runtime types, a facade crate, a CLI generator, and a live example workspace member.

## The Shape Of The Repo

- `crates/dioxus-showcase-core`: config, manifest, provider metadata, navigation helpers.
- `crates/dioxus-showcase-macros`: `#[showcase]`, `#[story]`, `#[provider]`, and `#[derive(StoryProps)]`.
- `crates/dioxus-showcase`: public facade and trait surface for app code.
- `crates/dioxus-showcase-cli`: discovery, scaffolding, generation, asset sync, and `dx serve` orchestration.
- `example`: a working annotated crate used to exercise the end-to-end pipeline.

## Quickstart

```bash
cargo run -p dioxus-showcase-cli -- init
cargo run -p dioxus-showcase-cli -- check
cargo run -p dioxus-showcase-cli -- build
cargo run -p dioxus-showcase-cli -- dev
```

## Current Runtime Model

1. Annotate Dioxus functions with `#[showcase]`, `#[story]`, and `#[provider]`.
2. Run the CLI against the configured entry crate.
3. The CLI discovers story/provider metadata and writes:
   - `target/showcase/showcase.manifest.json`
   - `showcase/src/generated.rs`
   - `showcase/src/main.rs`
4. The generated showcase app imports the entry crate, builds routes at `/component/:id`, and renders stories inside a responsive shell with tags, tree navigation, and theme switching.

## Commands

- `init`: prompt for `DioxusShowcase.toml` values and scaffold the generated app crate.
- `check`: validate config, discovery, duplicate ids, and scaffold presence.
- `build`: write the manifest and generated runtime files.
- `build --watch`: rebuild when annotated source files change.
- `dev`: rebuild in the background and launch `dx serve`.
- `doctor`: print basic host diagnostics.

## Read The Detailed Docs

- Full code reference: [`docs/code-reference.md`](docs/code-reference.md)
- Example walkthrough: [`example/README.md`](example/README.md)
- Design intent: [`docs/rfcs/dioxus-showcase.md`](docs/rfcs/dioxus-showcase.md)

## Release Flow

```bash
./scripts/set-workspace-version.sh X.Y.Z
cargo test --workspace --all-targets --all-features
git add Cargo.toml
git commit -m "chore(release): bump workspace version to X.Y.Z"
git tag vX.Y.Z
git push origin main --follow-tags
```

The publish workflow verifies that the pushed tag version matches the committed workspace manifest before running `cargo publish`.
