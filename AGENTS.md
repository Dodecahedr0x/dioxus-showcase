# AGENTS.md

This file gives coding agents the minimum project context needed to make correct changes in `dioxus-preview` without rediscovering the repository each time.

## Purpose

`dioxus-preview` is a Rust workspace for `dioxus-showcase`, a Storybook-style toolchain for Dioxus. The current repository contains:

- shared data types and config parsing
- procedural macros for showcase/stories/providers, which also emit link-time registrations
- a facade crate for end users, which owns the registration types and readers
- a UI crate holding the showcase shell as compiled Dioxus components
- a CLI that scaffolds and generates the showcase app, runs advisory discovery, and drives `dx`
- an `example/` workspace member used as the end-to-end fixture

The top-level `README.md` reflects the current shipped scope more accurately than the RFC in places. Use both, but treat implemented code and tests as the source of truth.

## Read This First

When working in this repository, orient yourself in this order:

1. `README.md`
2. **The architecture section below.** The project was restructured for `v0.1.0` and a
   surprising amount of older-looking code no longer means what it looks like it means.
3. `CONTRIBUTING.md` for the verification commands and which files are generated
4. `Cargo.toml`
5. `crates/dioxus-showcase-cli/src/commands.rs`
6. the crate or module you are changing
7. `example/README.md` if the change affects story authoring or discovery
8. `docs/rfcs/dioxus-showcase.md` only for intended direction, not for assuming already-shipped behavior

## Architecture In One Page

Read this before changing anything. Every bullet is a fact an agent will otherwise get wrong.

1. **Registration is link-time, via the `inventory` crate.** `#[showcase]`, `#[story]` and
   `#[provider]` each emit an `inventory::submit!` block at the *user's* call site,
   capturing `file!()` and `concat!(module_path!(), "::", stringify!(item))`. At runtime
   the shell calls `dioxus_showcase::registered_stories()` and `registered_providers()`,
   which collect and **sort** those registrations. Link order is not a stable contract, so
   the sort is mandatory, not cosmetic.
2. **`__dioxus_showcase_*` symbol names are no longer load-bearing across crate
   boundaries.** They still exist inside the crate that defines a story, but nothing
   outside that crate names them, and they appear nowhere in generated code. Do not reintroduce string conventions that
   let one crate guess another crate's symbol names — removing exactly that coupling is
   what `v0.1.0` was for.
3. **AST discovery is advisory.** `crates/dioxus-showcase-cli/src/discovery.rs` still scans
   the source tree, but nothing at runtime reads its output. It powers exactly two things:
   `check` diagnostics and `target/showcase/showcase.manifest.json`. Drift between
   discovery and the macros degrades diagnostics; it must never fail a build.
4. **`build` warns where `check` errors.** `build` downgrades every discovery failure —
   unparseable sources, unresolvable modules, and duplicate story ids alike — to a warning.
   Config errors still fail. Consequence: **CI must run `check`, not just `build`**, or a
   duplicate story id ships silently.
5. **`showcase/src/main.rs` is written once.** It is generated only when absent, and is
   never overwritten, diffed, or migrated. There is no `--force`. Only `generated.rs` and
   `Dioxus.toml` are regenerated on every build.
6. **`generated.rs` is one constant and is not compiled.** It contains only
   `pub const SHOWCASE_GENERATION`, and the generated `main.rs` deliberately has no
   `mod generated;` — declaring it would put a `dead_code` warning in every user's build.
   It survives as a regeneration marker and as the anchor for CI's byte-identical
   determinism assertion.
7. **Duplicate story ids do not panic.** They come back in
   `RegisteredStories::duplicate_ids` and the shell renders a banner. `check` still reports
   them as an error.
8. **MSRV is 1.85, measured, and is a hard floor.** Below it `inventory`'s
   `wasm32-unknown-unknown` support fails *silently* — empty registry, no error. Never
   lower it.
9. **`#[provider(order = N)]`** is the spelling; `index = N` is retired and rejected.
   Lowest `order` wraps outermost, default `0`. Note the internal
   `dioxus_showcase_core::ProviderDefinition` still spells its *field* `index`; that is
   advisory-only data, is not serialised into the manifest, and is deliberately left alone.

## Load-Bearing Things That Look Deletable

Both of these look like dead code or a slow default. **Deleting either produces a showcase
that builds, launches, and renders completely empty, with no error anywhere** —
indistinguishable from a project where nobody annotated anything. This is the single most
expensive mistake available in this repository.

### `use showcase_entry as _;` in the generated `main.rs`

Nothing calls into the component crate, so this import looks unused. It is the *entire*
reason any story appears: it is what asks the linker for the component crate's rlib, which
is what carries the `inventory` registrations. It ships with a `LOAD-BEARING` comment.
Do not "clean up the unused import".

### `[profile.dev] lto = "thin"` / `[profile.release] lto = true` in the generated `Cargo.toml`

These are **required for correctness on wasm, not an optimisation**. On
`wasm32-unknown-unknown` the `use ... as _;` import alone does not pull the component
crate's object out of its rlib archive: nothing in the binary references a symbol inside
it, so `wasm-ld` never selects the archive member and every registration in it is dropped.
LTO merges the crate graph before member selection happens, which is what keeps them.

Verified negatives, so nobody re-derives them: `-Clink-arg=--no-gc-sections` does not fix
it, `-Clink-dead-code` does not fix it, and `#[used(linker)]` is nightly-only. The cost is
real — thin LTO makes `dx serve` rebuilds slower and disables incremental compilation for
the showcase app — and it is accepted deliberately. `check` warns if a user removes either
profile setting.

## Workspace Layout

Top-level workspace members:

- `crates/dioxus-showcase-core`
  Shared config, manifest, and advisory discovery data structures. Has **no** `dioxus`
  dependency, and must not acquire one.
- `crates/dioxus-showcase-macros`
  Procedural macros such as `#[showcase]`, `#[story]`, `#[provider]`, and derives. Each
  also emits the `inventory::submit!` registration at the user's call site.
- `crates/dioxus-showcase`
  Facade crate that re-exports the user-facing API **and owns the registration contract**:
  `ShowcaseRegistration`, `ProviderRegistration`, `registered_stories()`,
  `registered_providers()`. These live here rather than in `core` because `GeneratedStory`
  holds a `dioxus::Element`.
- `crates/dioxus-showcase-ui`
  The showcase shell, as compiled Dioxus components: routing, tree navigation, tag filters,
  theme toggle, the error/empty/duplicate-id states, and `assets/showcase_app.css`. Exports
  `ShowcaseApp`, which reads the registry itself and takes no story list. Published.
- `crates/dioxus-showcase-cli`
  Main operational crate. Handles init/check/build/dev/export, advisory discovery, and
  scaffold generation. It no longer owns the shell.
- `example`
  Example workspace member used to validate annotation discovery and generated showcase output.

Five of these are published to crates.io — everything except `example`, which is
`publish = false`.

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

It must stay free of a `dioxus` dependency. Anything holding a `dioxus::Element` belongs in
the facade instead.

Current key files:

- `src/config.rs`
- `src/manifest.rs`
  `StoryManifest` is now `schema_version = 2` and is advisory output only. Its
  `renderer_symbol` field is retained but nothing depends on it.
- `src/runtime.rs`

### `dioxus-showcase-macros`

Owns parsing and code generation for procedural macros. Keep parsing behavior deterministic and error messages concrete. Macro changes should usually be accompanied by updates to:

- `crates/dioxus-showcase-macros/tests/macros.rs`
- `crates/dioxus-showcase-macros/tests/ui/` — `trybuild` compile-fail fixtures. Regenerate
  deliberately with `TRYBUILD=overwrite cargo test -p dioxus-showcase-macros --test ui`,
  then read the `.stderr` diff. They run ungated on stable; do not add a nightly gate,
  because this repo has no nightly CI leg and the tests would silently never run.
- any example usage in `example/src/`
- CLI discovery behavior if the *attribute surface* changes — discovery re-parses the same
  attributes and will drift otherwise. Note that drift now degrades diagnostics rather than
  breaking the runtime.

The macros reach `inventory` through `::dioxus_showcase::__private::inventory`, so users
never declare `inventory` in their own manifests. Do not change that to a direct
`inventory::` path in the expansion.

### `dioxus-showcase`

This is the stable facade for end users. Favor re-exports and small API shaping here; avoid embedding discovery or file-generation logic in this crate.

It does own one piece of real logic, by design: `src/registration.rs`, the `inventory`
collection points and the sorted readers. `registered_stories()` and
`registered_providers()` **must sort before returning** — link order is not stable, and CI
asserts deterministic output. Story sorting breaks ties beyond `id`
(`(id, module_path, source_path, title)`) precisely because `id` alone is not a total order
in the duplicate case that matters.

### `dioxus-showcase-ui`

The shell. `ShowcaseApp { base_path, title }` calls the registry itself; it takes no story
list, and adding one would reintroduce generated glue.

It must render without panicking for zero stories, for duplicate ids, and for a story whose
render fails. Those three states have tests; keep them.

Two non-obvious facts about its test harness (`src/testing.rs`), both found the hard way:

- **`ScopeId::ROOT` is not the component you passed to `VirtualDom::new_with_props`.**
  `dioxus-core` wraps the root component in an internal `RootScopeWrapper` which takes
  `ScopeId::ROOT`, so contexts your root component provides are invisible from
  `in_scope(ScopeId::ROOT, ..)` and `consume_context` panics. Hardcoding `ScopeId(1)` also
  fails; the depth is a `dioxus-core` implementation detail. The harness searches for the
  lowest scope that can see the `Shell` context.
- **A story returning `Err` needs a `render_immediate` pass after `rebuild_in_place`.**
  Returning an error marks the `ErrorBoundary` dirty but does not re-run it, so
  `rebuild_in_place()` alone renders an empty surface and a naive test reports a swallowed
  error as a caught one.

Components are rendered through `dioxus-ssr` (a dev-dependency) under bare `cargo test` —
no `dx` wrapper is needed. `asset!()` resolves in that setting, but it resolves to a
**machine-absolute path**, so no test may assert on an asset href. `document::Stylesheet`
emits nothing into `dioxus_ssr` output at all, being a head element.

Do not prefix routes or asset URLs with `base_path` inside the shell. `dioxus-web`'s
`WebHistory` and `manganis` each read the base path from the `dx` CLI config themselves, so
prefixing again yields `/repo/repo/...`. The `base_path` prop drives only the paths the
shell *displays* — the `Route:` line on a story page and the attempted path on the
not-found page.

### `dioxus-showcase-cli`

This is the most behavior-heavy crate. It currently owns:

- config loading
- interactive `init`
- advisory source discovery over `.rs` files
- validation via `check`
- generated artifact writing and scaffold maintenance via `build`
- dev loop and `dx serve` integration
- static site export via `dx bundle`

Key files:

- `src/cli.rs`
  Clap argument surface.
- `src/commands.rs`
  Entry-point dispatch and config bootstrapping.
- `src/discovery.rs`
  Source scanning and metadata extraction. Advisory: feeds `check` and the manifest only.
- `src/check.rs`
  Validation rules. Also warns when the generated app's `Cargo.toml` has lost its `lto`
  profile settings.
- `src/build.rs`
  Generated artifact writing. Downgrades discovery failures to warnings.
- `src/dev.rs`
  Dev workflow and file watching / regeneration path.
- `src/export.rs`
  Static site build, staging, `404.html` fallback, and asset pruning.
- `src/scaffold.rs`
  Showcase app creation and scaffold maintenance, including the write-once `main.rs` guard
  and syncing the entry crate's stylesheets.
- `src/templates.rs`
  Template contents for generated files. The write-once entry point lives here as an
  inline `const`, not as a `.hbs` file; only `generated_runtime.rs.hbs`,
  `showcase_cargo.toml.hbs` and `showcase_dioxus.toml.hbs` remain under `src/templates/`.
- `tests/commands.rs`
  On-disk end-to-end tests for `init`/`check`/`build`/`export`. `export` runs against a
  fake `dx` on `PATH` and those tests are `#[cfg(unix)]`.

## Actual Current Workflow

The implemented command flow is:

1. `init`
   Writes `DioxusShowcase.toml` and creates a runnable showcase app crate.
2. `check`
   Loads config, runs advisory discovery, and reports duplicate IDs, malformed annotations,
   configuration issues, and a missing `lto` profile setting. This is the only command that
   treats discovery problems as errors.
3. `build`
   Regenerates `generated.rs` and `Dioxus.toml`, writes `main.rs` **only if it is absent**,
   syncs the entry crate's assets, and writes the advisory manifest. Discovery problems are
   warnings here.
4. `dev`
   Rebuilds generated artifacts and launches the Dioxus app through `dx serve`.
5. `export`
   Rebuilds generated artifacts and compiles them into a deployable static site through `dx bundle`.
6. `doctor`
   Prints basic host diagnostics, including the detected `dx` version.

Do not assume the RFC's target architecture is fully implemented, and do not assume the
inverse either: the registration rework described above **is** shipped. Include-style glue
and generated symbol naming are gone.

## Generated Artifacts And Fixtures

There are generated files in this repository and in the example app. Before editing, determine whether a file is source-of-truth or generated output.

Regenerated on every build:

- `example/showcase/src/generated.rs` — one `SHOWCASE_GENERATION` constant, not compiled
  into the app (there is no `mod generated;`). CI diffs two consecutive builds of this file
  byte-for-byte, so anything nondeterministic here fails CI.
- `example/showcase/Dioxus.toml`
- `target/showcase/showcase.manifest.json` — advisory output of discovery. Nothing at
  runtime reads it.

Written once and then owned by the user:

- `example/showcase/src/main.rs` — created only when absent. `build` never overwrites,
  diffs, or migrates it, and there is no `--force`. If you are changing what new projects
  get, change the inline `SHOWCASE_MAIN_TEMPLATE` in
  `crates/dioxus-showcase-cli/src/templates.rs`, and understand that the change reaches
  **new** projects only. There is no migration story for existing ones; that is a
  deliberate non-answer, not an oversight.

The one hand-maintained file in `example/showcase/` is its `Cargo.toml`; see
`CONTRIBUTING.md` for why it is checked in while the rest of the directory is gitignored.
The example's `main.rs` stays gitignored on purpose — a checked-in write-once file would go
permanently stale.

Rules:

- Prefer changing the generator rather than hand-editing generated output.
- If a code change legitimately alters generated output, regenerate it and keep the fixture consistent.
- Avoid noisy churn in generated files when the semantic output is unchanged.
- Do not try to make `main.rs` regenerate. Write-once is the contract `v0.1.0` shipped, and
  it is the point of the whole restructuring.

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
- `#[provider(order = ...)]` wrapper ordering — lowest `order` wraps outermost, default `0`.
  The pre-release `index = ...` spelling is retired and is now rejected by discovery.
- a user stylesheet at `example/assets/example.css`, which is what keeps the
  "user stylesheets reach the generated `main.rs`" path exercised by CI
- generated showcase app output under `example/showcase/`

## Edit Guidelines

### When changing discovery or build logic

- Read both `discovery.rs` and `build.rs`.
- Check what `example/src/` contains, because the example is the practical discovery fixture.
- Preserve deterministic ordering. Manifest and generated-source stability matter for CI and reviewability.
- Treat duplicate IDs and malformed annotations as user-facing validation problems, not internal-only failures.
- Keep the asymmetry: `check` errors on discovery problems, `build` warns. Do not "fix"
  `build` by making it fail again — nothing at runtime reads discovery, so nothing it finds
  can justify stopping a build that would otherwise link and run. Config errors are the
  exception and still fail both.
- Do not make discovery authoritative again, and do not have the CLI derive symbol names.

### When changing macros

- Keep macro parsing tolerant enough for multi-line attribute forms if already supported.
  Bare `#[showcase]`, `#[story]` and `#[provider]` with no parentheses are valid and must
  stay valid in both the macros and discovery.
- Prefer precise compile-time diagnostics over silent behavior changes.
- Verify the facade crate still re-exports the necessary user API.
- If you change a user-visible diagnostic, update the `trybuild` fixtures in the same
  commit and read the regenerated `.stderr`.
- The `inventory::submit!` block the macros emit lands in the *user's* crate. It must keep
  using `file!()` and `module_path!()` so those bake in at the user's call site, and it must
  keep reaching `inventory` through the facade's `__private` re-export.

### When changing config or manifest schema

- Update `dioxus-showcase-core` first.
- Then update CLI readers/writers.
- Then update example artifacts or tests that encode the schema.
- Be careful with backwards-compatibility assumptions; this project is early-stage, but generated outputs still need to stay coherent.

### When changing the scaffolded app

- Look in `scaffold.rs` and `templates.rs` together.
- Remember that the generated showcase app is expected to run via `dx serve`.
- Keep the scaffold usable for local development without manual patching after `init`.
- **`main.rs` is write-once.** A change to `SHOWCASE_MAIN_TEMPLATE` reaches new projects
  only. If your change genuinely needs to reach existing ones, that is a design discussion,
  not a code change you make locally.
- Do not remove the `use showcase_entry as _;` line or the `lto` profile settings from
  `showcase_cargo.toml.hbs`. See *Load-Bearing Things That Look Deletable* above; both
  failures are silent and total.
- Shell UI changes belong in `dioxus-showcase-ui`, not in the entry-point template. The
  template exists to do the three things a library cannot: force rlib linkage, name the
  user's own stylesheets at the user's compile time (`asset!()` needs a literal in the crate
  it is compiled in), and mount `ShowcaseApp`.

## Commands

Run commands from the repository root unless a task clearly requires otherwise. No setup
step is needed: `DioxusShowcase.toml` is checked in, so `check`, `build`, `dev`, and
`export` all work directly from a clone. Do not run `init` unless you are specifically
testing that flow, because it overwrites that config.

Useful commands:

```bash
cargo test --workspace --all-targets --all-features
cargo run -p dioxus-showcase-cli -- check
cargo run -p dioxus-showcase-cli -- build
cargo run -p dioxus-showcase-cli -- dev
cargo run -p dioxus-showcase-cli -- export
```

Use `--all-features` on `cargo test`: it is what pulls the `trybuild` compile-fail suite
onto the main path. And run `check`, not only `build` — `build` warns where `check` errors.

Release-oriented commands (these scripts are maintained separately; describe them, do not
rewrite them casually):

```bash
./scripts/set-workspace-version.sh X.Y.Z     # rewrites [workspace.package] + internal deps
./scripts/verify-workspace-version.sh X.Y.Z  # what CI checks at tag time
cargo publish --workspace --dry-run          # packages all five published crates
```

`verify-workspace-version.sh` currently checks three internal dependency versions by name
and does not cover `dioxus-showcase-ui`; see `CONTRIBUTING.md`.

If you changed macros, discovery, manifests, or scaffolding, `cargo test --workspace --all-targets --all-features` is the default verification bar.

## Validation Expectations

Match the verification depth to the change:

- Small refactor with no behavior change:
  Run targeted tests or at least `cargo check`.
- Macro or parsing change:
  Run macro tests and workspace tests, including the `trybuild` suite via `--all-features`.
- Discovery/build/generation change:
  Run workspace tests and the CLI flow against the example when feasible.
- Scaffold/dev workflow change:
  Validate the CLI path, not just unit tests. `crates/dioxus-showcase-cli/tests/commands.rs`
  is where the on-disk behaviour is pinned.
- Registration, linkage, or generated-`Cargo.toml` change:
  Unit tests cannot see this class of failure. A dropped registration produces a wasm that
  builds and runs and shows nothing. The only real check is to build the example for wasm
  and confirm the user's own symbols are in the binary — which is exactly what CI's export
  job does by grepping the shipped `.wasm`. Say so explicitly if you could not run it.

Good practical checks:

```bash
cargo test --workspace --all-targets --all-features
cargo run -p dioxus-showcase-cli -- check
cargo run -p dioxus-showcase-cli -- build
```

If verification is skipped, say exactly what was not run.

## Rust Conventions

- Edition is `2021`.
- **MSRV is `1.85`**, declared in `[workspace.package]` and measured, not guessed. It is a
  hard floor for a correctness reason, not a style preference: below it `inventory`'s
  `wasm32-unknown-unknown` support fails silently. Never lower it. Raising it needs a
  measurement and a CHANGELOG entry.
- Workspace lints are enabled.
- `unsafe_code` is forbidden at the workspace level. Any new crate inherits that.
- Prefer small, explicit data structures and deterministic iteration over clever abstractions.
- Preserve human-readable error messages in CLI code; this tool is directly user-facing.

## What To Avoid

- Do not treat RFC text as proof that a feature already exists.
- Do not hand-edit generated files unless the task is specifically about the generated output itself.
- Do not move logic into the facade crate that belongs in `core`, `macros`, or `cli`. The
  registration types are the one deliberate exception, and the reason is in
  *Crate Responsibilities*.
- Do not introduce unnecessary workspace-wide dependency expansion for one localized feature.
- Do not make discovery output nondeterministic.
- **Do not delete the `use showcase_entry as _;` line or the `lto` profile settings.** See
  *Load-Bearing Things That Look Deletable*. Both look like cleanup and both silently empty
  the showcase.
- Do not add a `dioxus` dependency to `dioxus-showcase-core`.
- Do not have the shell prefix routes or asset URLs with `base_path`; that prefixing is
  already applied by `dioxus-web` and `manganis`, and doing it twice breaks every link.
- Do not make duplicate story ids panic again. They are reported, not fatal.
- Do not reintroduce cross-crate dependence on `__dioxus_showcase_*` symbol names.
- Do not add a hosted-demo deploy step. `export` is built in CI on purpose and deployed
  nowhere; documenting how *users* deploy their own sites is fine and lives in
  `docs/static-site.md`.

## Decision Heuristics

Use these placement rules when deciding where code belongs:

- Shared serializable types or config parsing, with no `dioxus` dependency:
  `dioxus-showcase-core`
- Attribute parsing or compile-time codegen:
  `dioxus-showcase-macros`
- End-user re-exports, and anything holding a `dioxus::Element` that both the macros and the
  shell need — including the registration contract:
  `dioxus-showcase`
- Showcase UI: routing, navigation, filtering, theming, empty/error/duplicate states, shell
  CSS:
  `dioxus-showcase-ui`
- Filesystem, scanning, generation, CLI UX:
  `dioxus-showcase-cli`

When something could live in either the shell or the generated entry point, it goes in the
shell unless it needs a literal from the user's crate at the user's compile time. In
practice that exception is only `asset!()` for user stylesheets, and rlib linkage.

## Documentation Hygiene

If you change behavior that affects end users or contributors, update the nearest relevant doc:

- `README.md` for supported workflows and command behavior
- `CONTRIBUTING.md` for the pre-PR checklist, the generated-file map, and the release process
- `example/README.md` for story-authoring conventions
- `docs/static-site.md` for anything that changes what `export` produces
- **this file** when you change something an agent would otherwise get wrong — especially
  anything in *Architecture In One Page* or *Load-Bearing Things That Look Deletable*
- RFC/docs only when the design direction itself changed

Keep docs aligned with implemented behavior. This repository is small enough that stale docs become misleading quickly.
