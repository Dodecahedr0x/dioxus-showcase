# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

`dioxus-showcase` is alpha software on `0.x`. Breaking changes ship in minor releases
with no deprecation cycle and no compatibility shims. **This file is the only notice
you get** — read the whole section for a version before upgrading to it.

## [Unreleased]

### Changed

- **The showcase is titled after the package it belongs to.** The generated
  `showcase/src/main.rs` now passes `project.name` from `DioxusShowcase.toml` as
  `ShowcaseApp`'s `title`, so the sidebar heading reads `acme-ui` rather than a generic
  "Showcase". A blank or whitespace-only name still falls back to `"Showcase"`.

  This matches the browser tab title, which has always been `"<project.name> Showcase"` —
  the two disagreed until now.

- `ShowcaseAppProps::title` gained `#[props(into)]`, so it accepts a string literal
  instead of `Some("…".to_owned())`. Additive: existing callers still compile.

### Upgrade notes

`showcase/src/main.rs` is **write-once**, so an existing project keeps its current entry
point and will still show "Showcase". Add the argument by hand:

```diff
-        dioxus_showcase_ui::ShowcaseApp { base_path: "/" }
+        dioxus_showcase_ui::ShowcaseApp { base_path: "/", title: "acme-ui" }
```

Nothing breaks if you skip it — the heading just keeps its old default.

## [0.1.0] - 2026-07-26

The first alpha. This is the release where the generated showcase app stops being
owned by the generator and becomes yours to edit.

It is a large breaking release. Every break is listed below.

### Added

- **New published crate `dioxus-showcase-ui`.** The showcase shell — routing, tree
  navigation, tag filters, theme toggle, empty and error states — is now a compiled
  library instead of a 12 KB Handlebars template rendered into your project. The
  workspace publishes **five** crates now, up from four: `dioxus-showcase-core`,
  `dioxus-showcase-macros`, `dioxus-showcase`, `dioxus-showcase-ui`, and
  `dioxus-showcase-cli`. Its public surface is `ShowcaseApp` / `ShowcaseAppProps`,
  taking a `base_path` and an optional `title`; it reads the story registry itself
  and takes no story list.
- **`dioxus-showcase export`** builds a deployable static website of your showcase.
  `--out-dir` and `--base-path` override the configured values. The site lands in
  `<build.out_dir>/site` and contains `index.html`, a hashed `assets/` directory, a
  `404.html` fallback so deep story routes survive a refresh, and a `.nojekyll`
  marker for GitHub Pages. Deployment recipes for GitHub Pages, Netlify, Vercel and
  Cloudflare Pages are in `docs/static-site.md`.
- **A declared MSRV: Rust 1.85**, inherited by all five published crates. This is a
  hard floor, not a formality. `inventory`'s `wasm32-unknown-unknown` support landed
  in 0.3.24 and is version-gated on 1.85; below it the registry comes back **empty
  with no error at all**, so the showcase would build, launch and render nothing.
  The measured dependency-tree floor is independently 1.85 (`time-core` needs
  edition 2024). The MSRV may rise in a future release; it will never be lowered.
- **Crate metadata on all five published crates**: `repository`, `readme`,
  `keywords`, `categories` and `rust-version`. Previously the crates.io pages
  carried no source link.
- **`check` now warns if the generated app's `Cargo.toml` has dropped `lto`** from
  `[profile.dev]` or `[profile.release]`, because removing it silently empties the
  showcase (see below).
- **`check` now names every colliding story**, with its module path, for each
  duplicated id, instead of reporting the first collision only.
- **On-disk end-to-end tests for `init`, `check`, `build` and `export`.** Previously
  only artifact *writing* was covered by golden files; the command orchestration was
  untested.
- **`trybuild` compile-fail coverage for the macro error paths**, run ungated on
  stable alongside the rest of the suite.
- Bare `#[showcase]`, `#[story]` and `#[provider]` — written with no parentheses —
  are now accepted by discovery. The macros always accepted them; discovery rejected
  them with `expected attribute arguments in parentheses`. That was a bug.

### Changed

- **BREAKING — stories and providers now register themselves at link time**, via
  `inventory`, from the annotated item's own call site. The CLI no longer generates
  glue code that names the macros' `__dioxus_showcase_*` symbols, and nothing outside
  the defining crate names them any more. `src/generated.rs` contains no
  `__dioxus_showcase_*` symbol at all. This removes the string-convention coupling
  between the CLI's discovery and the macro crate's output, which was the reason
  generated output broke whenever either side moved.
- **BREAKING — `showcase/src/main.rs` is written once and never regenerated.**
  After `init` (or the first `build`) the file is yours: edit it freely, `build`
  will not overwrite it, diff it, or migrate it. Only `src/generated.rs` is
  regenerated on subsequent builds.
  Existing projects keep the `main.rs` they have and are **not migrated** to the new
  entry point — see the upgrade note below.
- **BREAKING — `#[provider(index = N)]` is now `#[provider(order = N)]`.** `order` is
  an `i32`, defaults to `0`, and the **lowest** order wraps **outermost**. `check`
  rejects `index` with `provider attributes only support order = <integer>`. Provider
  order used to come from discovery sequence, which you could not control explicitly.
- **BREAKING — duplicate story ids no longer panic.** The generated runtime used to
  `assert!` on insert, taking down the entire app for one collision. Colliding ids
  are now collected, reported in a banner rendered above the router on every page,
  and **both** colliding stories stay navigable through the tree. `/component/<id>`
  resolves to the first match in registry sort order.
- **BREAKING — the manifest's `schema_version` is now `2`.** `renderer_symbol` is
  retained on every story but is now **advisory**: nothing at runtime reads it.
- **BREAKING — AST discovery is demoted to advisory.** It powers exactly two things:
  `check` diagnostics and `target/showcase/showcase.manifest.json`. Drift between
  discovery and the macros degrades diagnostics; it can no longer produce a broken
  app, because the app does not read discovery output.
- **BREAKING — `build` downgrades *every* discovery failure to a warning**, including
  unparseable sources, unresolvable modules, and duplicate story ids. `check` still
  reports all of them as errors. Config errors — a missing entry crate, an unknown
  config key — are not downgraded and still fail. **Consequence: run `check` in CI,
  not just `build`, or id collisions will not fail your pipeline.**
- **BREAKING — the generated app's `Cargo.toml` now pins `lto = "thin"` under
  `[profile.dev]` and `lto = true` under `[profile.release]`. These lines are
  required, not an optimisation.** On `wasm32`, `use <your_crate> as _;` does not on
  its own pull the component crate's object out of its rlib archive: nothing in the
  binary references a symbol inside it, so the linker never selects the archive
  member and every registration in it is dropped. LTO merges the crate graph before
  member selection happens, which is what keeps the registrations. Without it the app
  builds, launches and renders an **empty** showcase with no error anywhere —
  indistinguishable from having annotated nothing. Verified on Rust 1.94 / Dioxus CLI
  0.7.9: without these two lines both `dx build` and `dx bundle --release` produce
  zero stories. `--no-gc-sections` and `-Clink-dead-code` do not fix it, and
  `#[used(linker)]` is nightly-only. **The honest cost: `dx serve` rebuilds are
  slower, and incremental compilation is disabled for the showcase app.**
  `[profile.release] strip = true` is set too, to keep DWARF out of release wasm,
  which `wasm-opt` aborts on.
- **BREAKING — user stylesheets are now linked from the generated `main.rs`**, one
  `document::Stylesheet` line per file found in your entry crate's `assets/`
  directory, rather than by the shell. A library crate cannot do it: `asset!()`
  resolves relative to the crate it is compiled in, so `dioxus-showcase-ui` can only
  ship its own CSS.
- **BREAKING — `src/generated.rs` collapses to a single constant**,
  `SHOWCASE_GENERATION`, and is no longer compiled into the app: the generated
  `main.rs` does not declare `mod generated;`. `ShowcaseComponentDefinition`,
  `showcase_components()` and `story_providers()` are gone. `generated.rs` is now a
  pure build artifact — a regeneration marker and the anchor for the byte-identical
  determinism assertion — rather than source.
- The generated app now depends on `dioxus-showcase-ui` and enables the `dioxus`
  `router` feature.
- `inventory` is pinned at `>= 0.3.24`, the first release with
  `wasm32-unknown-unknown` support. Earlier versions link but yield an empty registry.
- Publishing switched to a single `cargo publish --workspace`, which makes
  `cargo publish --dry-run` work before anything has been published.

### Removed

- **BREAKING — `showcase_main.rs.hbs` and `showcase_app.css` are gone from the CLI.**
  The shell they rendered lives in `dioxus-showcase-ui` as compiled components and
  ships its own stylesheet. If you were relying on the CLI overwriting `main.rs` on
  every build to pick up shell changes, that no longer happens by design.

### Fixed

- A stale generator-written `showcase/assets/showcase.css` — the shell stylesheet
  older versions wrote there — is now deleted on `build`, unless your entry crate
  ships an `assets/showcase.css` of its own, in which case that file is yours and is
  copied and linked as normal. Without the cleanup the old 12 KB shell stylesheet
  would survive an upgrade, be picked up by the stylesheet scan as if you had written
  it, and fight the new one.
- Discovery no longer rejects attributes written without parentheses.
- The displayed `Route:` line on a story page and the attempted path on the not-found
  page are now prefixed with the configured base path. They previously printed
  `/component/x` even when the site was served from a sub-path.

### Upgrade notes

**Every project upgrading from `0.0.7` must hand-edit its existing
`showcase/Cargo.toml`.** This is not conditional on having customised it: like
`src/main.rs`, **`showcase/Cargo.toml` is written once**, only when it is absent, so
*no* project scaffolded by an earlier release receives any of the manifest changes
this version introduces. `dioxus-showcase build` will not add them for you, and it
will not tell you they are missing except for the `lto` lines, which
`dioxus-showcase check` warns about.

1. **Hand-edit `showcase/Cargo.toml`.** Add the new dependency, the new `dioxus`
   feature, and both profile blocks:

   ```toml
   [dependencies]
   # `router` is new: the shell is a router-based application now.
   dioxus = { version = "0.7", features = ["web", "router"] }
   # New crate. The shell that used to be generated into main.rs lives here.
   dioxus-showcase-ui = "0.1.0"

   # LTO IS LOAD-BEARING ON WASM. DO NOT REMOVE IT.
   [profile.dev]
   lto = "thin"

   [profile.release]
   lto = true
   ```

   Missing `dioxus-showcase-ui`, or the `router` feature, is a compile error — loud,
   and easy to fix. **Missing the two `lto` lines is not.** The app compiles, links,
   launches, and renders an **empty** showcase, with no error anywhere, because on
   `wasm32` nothing in the binary references a symbol inside your component crate's
   rlib, so the linker never selects that archive member and every registration in it
   is dropped. LTO merges the crate graph before that selection happens. This is the
   one upgrade step whose omission fails silently.

2. **Then deal with `showcase/src/main.rs`, which will not be migrated either.**
   `build` writes it only when it is absent and will not rewrite one that exists.
   Deleting it and re-running `dioxus-showcase build` gets you the new entry point —
   but do step 1 **first**: on its own, deleting `main.rs` yields a regenerated file
   that references `dioxus_showcase_ui` against a manifest that does not depend on it,
   which simply fails to compile. If instead you have edits you want to keep, the two
   lines that must be present are `use <your_crate> as _;` (without it you get zero
   stories, silently) and `dioxus_showcase_ui::ShowcaseApp { base_path: "..." }`.

3. **Stylesheets are in the same position.** A project scaffolded before you added a
   stylesheet to your entry crate's `assets/` directory will not gain the new
   `document::Stylesheet` line, because `build` will not edit an existing `main.rs`.
   Add that line by hand.

4. **Rename `#[provider(index = N)]` to `#[provider(order = N)]`.** Check the sign:
   the lowest `order` wraps outermost.

## [0.0.7] - 2026-03-16

### Added

- `#[provider]` — components that wrap every story, for themes and context.
- The generated showcase app is now standalone: it declares its own `[workspace]` so
  it no longer has to be a member of yours.

### Changed

- Documentation pass across the repository and the example crate.
- The example moved from `examples/basic` to `example/`.

## [0.0.6] - 2026-03-12

### Fixed

- Generated component wiring in the showcase app's entry point.
- Workspace dependency declarations across the published crates.

## [0.0.5] - 2026-03-12

### Added

- `scripts/verify-workspace-version.sh`, and a documented release flow in the README.

### Changed

- A release tag must now point at a commit that already carries the target workspace
  version. The publish workflow verifies the tag against the committed manifest
  instead of rewriting `Cargo.toml` and pushing the bump back to the default branch
  after publishing.

## [0.0.4] - 2026-03-12

### Added

- Story failures are isolated in the showcase shell: one story that fails to render
  no longer takes down the whole app.

## [0.0.3] - 2026-03-12

### Added

- Configured base paths are honoured by the generated app, so a showcase can be
  served from a sub-path.
- Golden-file coverage for the build artifacts.

### Changed

- Config and manifest parsing moved to `serde`.
- Generation and publishing are deterministic: a second `build` on unchanged sources
  reproduces byte-identical artifacts.

### Fixed

- Discovery follows the module graph instead of scanning files in isolation, so
  components declared in submodules are found.

## [0.0.2] - 2026-03-10

### Fixed

- The generated showcase crate builds against the published crates.

## [0.0.1] - 2026-03-10

### Added

- Initial release: `#[showcase]`, `#[story]` and `#[derive(StoryProps)]` macros;
  the `dioxus-showcase` facade and `dioxus-showcase-core` data model; and the
  `dioxus-showcase` CLI with `init`, `check`, `build`, `dev` and `doctor`.

[0.1.0]: https://github.com/Dodecahedr0x/dioxus-showcase/compare/v0.0.7...v0.1.0
[0.0.7]: https://github.com/Dodecahedr0x/dioxus-showcase/compare/v0.0.6...v0.0.7
[0.0.6]: https://github.com/Dodecahedr0x/dioxus-showcase/compare/v0.0.5...v0.0.6
[0.0.5]: https://github.com/Dodecahedr0x/dioxus-showcase/compare/v0.0.4...v0.0.5
[0.0.4]: https://github.com/Dodecahedr0x/dioxus-showcase/compare/v0.0.3...v0.0.4
[0.0.3]: https://github.com/Dodecahedr0x/dioxus-showcase/compare/v0.0.2...v0.0.3
[0.0.2]: https://github.com/Dodecahedr0x/dioxus-showcase/compare/v0.0.1...v0.0.2
[0.0.1]: https://github.com/Dodecahedr0x/dioxus-showcase/releases/tag/v0.0.1
