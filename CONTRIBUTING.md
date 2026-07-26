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
git clone https://github.com/Dodecahedr0x/dioxus-showcase
cd dioxus-showcase

cargo test --workspace --all-targets --all-features   # unit + golden tests
cargo run -p dioxus-showcase-cli -- check             # validate discovery against example/
cargo run -p dioxus-showcase-cli -- build             # regenerate example/showcase sources
cargo run -p dioxus-showcase-cli -- dev               # serve the example showcase (needs dx)
cargo run -p dioxus-showcase-cli -- export            # build a static site (needs dx)
```

`init` is for *consumers* setting up their own project. You only need it when testing that
flow itself, and it will overwrite `DioxusShowcase.toml`, so revert it afterwards.

## Before Opening A Pull Request

Run these three locally. They are the gate CI applies to every pull request:

```bash
cargo fmt --all
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
```

CI runs more than that, so a green local run is necessary but not sufficient. The extra
work, described by purpose rather than by quoting the workflow:

- **Publish dry run** — `cargo publish --workspace --dry-run`, so a crate-metadata or
  packaging mistake surfaces on the pull request instead of at tag time, when the first
  crate has already gone out and cannot be taken back.
- **MSRV** — a `cargo check` on the declared minimum toolchain, 1.85. See the comment
  above `rust-version` in [`Cargo.toml`](Cargo.toml) for why that number is a hard floor
  and not a preference.
- **Example pipeline** — runs `check` and `build` against the checked-in `example/`,
  asserts the generated artifacts exist, runs `build` a second time and asserts
  `generated.rs` came out byte-identical, then asserts the working tree is clean.
- **Static site export** — the only place `export` runs against a real Dioxus toolchain.
  It asserts `index.html`, `404.html`, `.nojekyll` and a non-empty `assets/` were
  produced, and then greps the shipped wasm for the example's own story symbols, because
  a showcase whose registrations were dropped still produces a complete-looking site.
  Nothing is deployed and there is no hosted demo.

If you touched discovery, generation, or the templates, also run
`cargo run -p dioxus-showcase-cli -- check` and `-- build`, and check that the regenerated
files under `example/showcase/` still look right. Run both, not just `build`: discovery is
advisory now, so `build` downgrades every discovery problem — including duplicate story
ids — to a warning, and only `check` reports them as errors.

## Which Files Are Generated

Most of `example/showcase/` is written by `dioxus-showcase build` and is gitignored. For
the regenerated files, do not edit them by hand — change the templates in
`crates/dioxus-showcase-cli/src/templates/` (or the inline entry-point template in
`crates/dioxus-showcase-cli/src/templates.rs`) and rebuild.

| Path | Status |
| --- | --- |
| `example/showcase/Cargo.toml` | Checked in. The one hand-maintained file in that directory. |
| `example/showcase/src/main.rs` | **Written once**, only when it is absent. `build` never overwrites, diffs, or migrates it, and there is no `--force`. It is the user's file after that. |
| `example/showcase/src/generated.rs` | Generated from `templates/generated_runtime.rs.hbs`, overwritten on every build. |
| `example/showcase/Dioxus.toml` | Generated from `templates/showcase_dioxus.toml.hbs`, overwritten on every build. |
| `example/showcase/assets/` | Synced from `example/assets/`. The shell's own CSS ships inside `dioxus-showcase-ui` and is no longer copied here. |
| `target/showcase/` | Advisory manifest and exported static site output. |

`example/showcase/Cargo.toml` is checked in for one reason: `init` normally writes a
manifest depending on `dioxus-showcase` from crates.io, and that would make the example
build against the last published release instead of your working copy. The checked-in
version uses workspace path dependencies so your changes actually take effect.

`example/showcase/src/main.rs` deliberately stays gitignored rather than being checked in
alongside it. A write-once file under version control would go permanently stale: the
moment the entry-point template changed, or a stylesheet was added to `example/assets/`,
`build` would refuse to update a file that already exists, and the checked-in copy would
silently diverge. A fresh clone regenerates it correctly every time. The
"file already exists, leave it alone" branch is covered by
`crates/dioxus-showcase-cli/tests/commands.rs` instead of by the example.

`generated.rs` is now a build artifact rather than source: it holds a single
`SHOWCASE_GENERATION` constant, the generated `main.rs` does not declare `mod generated;`,
and nothing compiles it into the app. It survives as a regeneration marker and as what CI
compares byte-for-byte to prove generation is deterministic.

## Tests

### Golden tests

`crates/dioxus-showcase-cli/src/testdata/` holds expected generation output. If you
intentionally change what the CLI generates, those files must be updated in the same
commit — the diff in the pull request is how a reviewer sees the change in generated code.

### On-disk command tests

`crates/dioxus-showcase-cli/tests/commands.rs` drives `init`, `check`, `build`, and
`export` end to end against fixture workspaces in temporary directories. This is where the
write-once guarantee is actually pinned (`build` must leave a user-edited `main.rs`
byte-identical). `export` is exercised against a fake `dx` placed on `PATH`, so the tests
stay hermetic; the two tests that do this are `#[cfg(unix)]` and the CI export job is the
only coverage on Windows.

### Compile-fail tests

`crates/dioxus-showcase-macros/tests/ui.rs` runs `trybuild` over the fixtures in
`crates/dioxus-showcase-macros/tests/ui/`, each a `.rs` case paired with its expected
`.stderr`. They run ungated on stable and are picked up by the normal
`cargo test --workspace --all-targets --all-features`. If you deliberately change a macro
diagnostic, regenerate the fixtures with:

```bash
TRYBUILD=overwrite cargo test -p dioxus-showcase-macros --test ui
```

then read the resulting `.stderr` diff before committing it — an overwritten fixture that
nobody read is a test that no longer tests anything.

## Where Things Live

| Crate | Responsibility |
| --- | --- |
| `dioxus-showcase-core` | Config, manifest, and shared data types. No dependency on `dioxus`. |
| `dioxus-showcase-macros` | `#[showcase]`, `#[story]`, `#[provider]`, `#[derive(StoryProps)]`. Each also emits an `inventory::submit!` registration at your call site. |
| `dioxus-showcase` | The facade crate that user code depends on. Owns the registration types and the `registered_stories()` / `registered_providers()` readers. |
| `dioxus-showcase-ui` | The showcase shell, as compiled components: routing, tree navigation, tag filters, theme toggle, the error/empty/duplicate-id states, and `showcase_app.css` as an asset. Exports `ShowcaseApp`. |
| `dioxus-showcase-cli` | Discovery, scaffolding, generation, `dx serve`, and `dx bundle`. It does **not** own the shell any more — that moved to `dioxus-showcase-ui`, and the CLI now generates a ~15-line entry point that mounts it. |
| `example` | The annotated fixture crate the CLI is exercised against. |

A file-by-file walkthrough is in [`docs/code-reference.md`](docs/code-reference.md), and
`AGENTS.md` holds the same orientation in a form aimed at coding agents.

## Releasing

Releases are tag-driven. Pushing a `vX.Y.Z` tag runs
[`.github/workflows/publish.yml`](.github/workflows/publish.yml), which verifies the tag
matches the committed workspace version and then runs a single `cargo publish --workspace`.
Cargo works out the publish order across the workspace itself, so there is no per-crate
call chain and no `sleep` between crates any more.

**Five crates are published**: `dioxus-showcase-core`, `dioxus-showcase-macros`,
`dioxus-showcase`, `dioxus-showcase-ui`, and `dioxus-showcase-cli`. `example` is
`publish = false` and is never released.

```bash
./scripts/set-workspace-version.sh X.Y.Z     # rewrites [workspace.package] + internal deps
./scripts/verify-workspace-version.sh X.Y.Z  # what CI will check
cargo publish --workspace --dry-run          # packages all five, publishes nothing
cargo test --workspace --all-targets --all-features

git add Cargo.toml Cargo.lock
git commit -m "release: vX.Y.Z"
git tag vX.Y.Z
git push origin main --follow-tags
```

`cargo publish --workspace --dry-run` is worth running every time. Because it treats the
workspace as one unit it succeeds with nothing published yet, which the old per-crate chain
could not do — you could not dry-run `dioxus-showcase-cli` until `dioxus-showcase` was
already on crates.io. It is also what proves the packaged `.crate` for the CLI still
contains `src/templates/`, which the CLI needs at runtime.

Known gap, worth checking by eye until it is fixed:
`scripts/verify-workspace-version.sh` validates the `[workspace.package]` version plus
three internal dependency versions by name — `dioxus-showcase-core`,
`dioxus-showcase-macros`, and `dioxus-showcase`. It does not check `dioxus-showcase-ui`,
which is also in `[workspace.dependencies]`, so a stale version there would pass
verification and fail at publish time.

Record every user-visible change in [`CHANGELOG.md`](CHANGELOG.md) in the release commit.

The publish job needs a `CARGO_REGISTRY_TOKEN` repository secret.
