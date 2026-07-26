# Contributing

The full contributor guide lives in the repository, next to the code it describes:
[`CONTRIBUTING.md`](https://github.com/Dodecahedr0x/dioxus-showcase/blob/main/CONTRIBUTING.md).

It covers prerequisites, the pre-PR checklist, which files are generated, the test layout,
and the release process. The short version:

```bash
git clone https://github.com/Dodecahedr0x/dioxus-showcase
cd dioxus-showcase

cargo test --workspace --all-targets --all-features
cargo run -p dioxus-showcase-cli -- check
cargo run -p dioxus-showcase-cli -- dev
```

Everything runs from a fresh clone with no setup step — the repo's own
`DioxusShowcase.toml` is checked in and points at `example/`.

Before opening a pull request, run the three commands CI gates on:

```bash
cargo fmt --all
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
```

CI runs more than that — a publish dry run, an MSRV check on 1.85, the example pipeline,
and a real static-site export whose shipped wasm is grepped for the example's story
symbols. A green local run is necessary but not sufficient.

## Editing these docs

This site is an [mdBook](https://rust-lang.github.io/mdBook/) in `docs-site/`. Chapters that
mirror a file in the repository — the changelog, the code reference, the static-site guide,
the RFC — use `\{{#include}}` so there is exactly one copy of that text. Edit the source
file, not the chapter.

```bash
cargo install mdbook --locked
mdbook serve docs-site --open
```

It deploys to GitHub Pages automatically on every push to `main`.
