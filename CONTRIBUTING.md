# Contributing

## Prerequisites

- **Rust** — the toolchain is pinned in [`rust-toolchain.toml`](rust-toolchain.toml), so
  `rustup` installs the right version automatically the first time you build.
- **Dioxus CLI** — only needed for `dev` and `export`, which shell out to `dx`:

  ```bash
  cargo install dioxus-cli --locked
  rustup target add wasm32-unknown-unknown
  ```

Run `cargo run -p dioxus-showcase-cli -- doctor` at any point to see whether `dx` was found.

## First Run

Everything works from a fresh clone with no setup step. The repo's own
`DioxusShowcase.toml` is checked in and points at the `example/` crate, so you never need
to run the interactive `init` to work on this project.

```bash
git clone https://github.com/Dodecahedr0x/dioxus-preview
cd dioxus-preview

cargo test --workspace --all-targets --all-features   # unit + golden tests
cargo run -p dioxus-showcase-cli -- check             # validate discovery against example/
cargo run -p dioxus-showcase-cli -- build             # regenerate example/showcase sources
cargo run -p dioxus-showcase-cli -- dev               # serve the example showcase (needs dx)
cargo run -p dioxus-showcase-cli -- export            # build a static site (needs dx)
```

`init` is for *consumers* setting up their own project. You only need it when testing that
flow itself, and it will overwrite `DioxusShowcase.toml`, so revert it afterwards.

## Before Opening A Pull Request

These are exactly what CI runs, so a green local run means a green CI run:

```bash
cargo fmt --all
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
```

If you touched discovery, generation, or the templates, also run
`cargo run -p dioxus-showcase-cli -- build` and check that the regenerated files under
`example/showcase/` still look right.

## Which Files Are Generated

Almost everything under `example/showcase/` is written by `dioxus-showcase build` and is
gitignored. Do not edit it by hand — change the templates in
`crates/dioxus-showcase-cli/src/templates/` and rebuild.

| Path | Status |
| --- | --- |
| `example/showcase/Cargo.toml` | Checked in. The one hand-maintained file in that directory. |
| `example/showcase/src/main.rs` | Generated from `templates/showcase_main.rs.hbs`, overwritten on every build. |
| `example/showcase/src/generated.rs` | Generated from `templates/generated_runtime.rs.hbs`, overwritten on every build. |
| `example/showcase/Dioxus.toml` | Generated from `templates/showcase_dioxus.toml.hbs`, overwritten on every build. |
| `example/showcase/assets/` | Synced from `example/assets/` plus the shell stylesheet. |
| `target/showcase/` | Manifest and exported static site output. |

`example/showcase/Cargo.toml` is checked in for one reason: `init` normally writes a
manifest depending on `dioxus-showcase` from crates.io, and that would make the example
build against the last published release instead of your working copy. The checked-in
version uses workspace path dependencies so your changes actually take effect.

## Golden Tests

`crates/dioxus-showcase-cli/src/testdata/` holds expected generation output. If you
intentionally change what the CLI generates, those files must be updated in the same
commit — the diff in the pull request is how a reviewer sees the change in generated code.

## Where Things Live

| Crate | Responsibility |
| --- | --- |
| `dioxus-showcase-core` | Config, manifest, and runtime types shared by everything else. |
| `dioxus-showcase-macros` | `#[showcase]`, `#[story]`, `#[provider]`, `#[derive(StoryProps)]`. |
| `dioxus-showcase` | The facade crate that user code depends on. |
| `dioxus-showcase-cli` | Discovery, scaffolding, generation, `dx serve`, and `dx bundle`. |
| `example` | The annotated fixture crate the CLI is exercised against. |

A file-by-file walkthrough is in [`docs/code-reference.md`](docs/code-reference.md), and
`AGENTS.md` holds the same orientation in a form aimed at coding agents.

## Releasing

Releases are tag-driven. Pushing a `vX.Y.Z` tag runs
[`.github/workflows/publish.yml`](.github/workflows/publish.yml), which verifies the tag
matches the committed workspace version and then publishes the crates to crates.io in
dependency order.

```bash
./scripts/set-workspace-version.sh X.Y.Z     # rewrites [workspace.package] + internal deps
./scripts/verify-workspace-version.sh X.Y.Z  # what CI will check
cargo test --workspace --all-targets --all-features

git add Cargo.toml Cargo.lock
git commit -m "release: vX.Y.Z"
git tag vX.Y.Z
git push origin main --follow-tags
```

The publish job needs a `CARGO_REGISTRY_TOKEN` repository secret.
