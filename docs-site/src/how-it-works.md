# How It Works

Four steps, and one idea that explains all of them: **the runtime source of truth is a
link-time registry, not generated code.**

## 1. Macros register at your call site

`#[showcase]`, `#[story]`, and `#[provider]` each expand into an
[`inventory`](https://docs.rs/inventory) registration carrying the item's source path, its
module path, and a factory function pointer:

```rust
inventory::submit! {
    ShowcaseRegistration {
        source_path: file!(),
        module_path: concat!(module_path!(), "::", stringify!(PillButton)),
        factory: __dioxus_showcase_story__PillButton,
    }
}
```

Because the expansion happens in *your* crate, `file!()` and `module_path!()` capture your
locations. Nothing outside the defining crate names a generated symbol — that coupling is
what the `0.1.0` architecture removed.

## 2. `build` writes two small things

`dioxus-showcase build` produces `target/showcase/showcase.manifest.json` and the showcase
app's `src/generated.rs`. The latter is one constant. It scaffolds the entry point only
when absent.

## 3. The shell reads the registry

`dioxus_showcase_ui::ShowcaseApp` — a compiled component from a published crate, not
generated code — reads the registry at startup, sorts it deterministically, builds routes
at `/component/:id`, and renders each story in a shell with tag filters, tree navigation,
and theme switching.

Sorting matters more than it looks. Link order is not a stable contract, so the registry is
ordered by story id (with further tie-breakers) before anything renders. Without that, story
order could shift between builds of identical source.

Failures stay contained:

- A story that fails to render is caught by an `ErrorBoundary`; the rest of the showcase
  keeps working.
- Duplicate ids are reported in a banner rather than panicking, and both colliding stories
  stay navigable.

## 4. Discovery is advisory

The CLI also scans your entry crate's AST, following `mod foo;` through the module graph.
That scan powers exactly two things: `check` diagnostics and the manifest.

Nothing at runtime depends on it. If the scan and the macros ever disagree, the macros win
and the site still renders correctly — drift degrades diagnostics, it does not break your
build. That is why `check` reports collisions as errors while `build` only warns.

The payoff is speed: `check` validates without compiling anything.

## Why link-time registration

The alternative designs were considered and rejected on evidence:

- **Generated glue naming macro symbols** (the `0.0.x` design) required the CLI to
  re-derive `__dioxus_showcase_*` names by string convention that the macros independently
  emitted. Two implementations of one rule, guaranteed to drift.
- **Macros writing metadata files** is explicitly unsupported by the Cargo team, and breaks
  under incremental compilation and cached CI builds — a fully-cached build emits zero
  metadata files while reporting success. Dioxus itself moved *away* from this for
  `manganis` assets in 0.6.
- **`linkme`** does not build for `wasm32` at all; it emits a hard `compile_error!`.

## The one sharp edge

Link-time registration has a cost, and it is the reason for the
[two load-bearing lines](./generated-app.md#two-lines-you-must-not-delete): on `wasm32` the
linker will happily drop an archive member that nothing references, taking every
registration in it with it. The `use <crate> as _;` import and thin LTO together are what
prevent that.

Both fail silently when removed. `check` warns about the `lto` lines, and the showcase's
empty state names both causes — but this is the part of the design worth knowing about
before you go editing generated files.
