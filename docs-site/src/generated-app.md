# The Generated App Is Yours

`showcase/src/main.rs` is written **once**, when it does not exist, and is then **never
regenerated**. After that first write it is your file: edit it, restructure it, add
providers and routes to it. `build` will not overwrite it, diff it, or migrate it, and
there is no `--force`.

`showcase/Cargo.toml` is written once on exactly the same terms, because you add your own
dependencies to it.

| Generated file | Rewritten? |
| --- | --- |
| `showcase/src/main.rs` | Written once, when absent. Yours afterwards. |
| `showcase/Cargo.toml` | Written once, when absent. Yours afterwards. |
| `showcase/src/generated.rs` | Every build. |
| `showcase/Dioxus.toml` | Every build. |
| `showcase/assets/` | Re-synced from your entry crate's `assets/` every build. |

`src/generated.rs` is a build marker holding a single `SHOWCASE_GENERATION` constant — the
app does not even compile it. It exists so CI can prove generation is deterministic.

## The flip side

Improvements to the entry point never reach an existing project on their own. If you add a
stylesheet to your entry crate's `assets/` *after* scaffolding, `build` will not add the
`document::Stylesheet` line for it — you add that line by hand. The same applies to
anything a newer release starts emitting, **including the two `lto` lines below**: a
project scaffolded before they existed does not have them, whether or not you ever touched
the file.

To adopt a new entry point wholesale, delete `showcase/src/main.rs` and run
`dioxus-showcase build` — after bringing `showcase/Cargo.toml` up to date by hand, since
the regenerated entry point depends on things an older manifest does not declare. The
[changelog](./changelog.md) upgrade notes carry the exact lines for each release.

## Two lines you must not delete

Both look like clutter. Both fail **silently** when removed — no error, no warning, just an
empty showcase. That is exactly why they are documented here.

### 1. `use <your_crate> as _;` in `showcase/src/main.rs`

```rust
use showcase_entry as _; // LOAD-BEARING: forces rlib linkage so registrations survive
```

Nothing calls into your crate, so every linter and every instinct says this import is dead.
It is the entire reason any story appears: it is what links your crate into the binary, and
your crate is what carries the registrations.

### 2. The `lto` lines in `showcase/Cargo.toml`

```toml
[profile.dev]
lto = "thin"

[profile.release]
lto = true
```

These are **not** a speed knob. On `wasm32`, the import above does not on its own pull your
crate's object out of its rlib archive — nothing in the binary references a symbol inside
it, so the linker never selects the archive member and every registration in it is dropped.
LTO merges the crate graph before that selection happens, which is what keeps them.

Remove these lines and both `dx build` and `dx bundle --release` produce a showcase with
zero stories and no error anywhere.

The honest cost: thin LTO makes `dx serve` rebuilds slower and disables incremental
compilation for the showcase app. `dioxus-showcase check` warns if the lines go missing.

## What a generated entry point looks like

```rust
// Generated once by dioxus-showcase. Safe to edit; never regenerated.
use dioxus::prelude::*;
use showcase_entry as _; // LOAD-BEARING: forces rlib linkage so registrations survive

fn main() {
    launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        dioxus_showcase_ui::ShowcaseApp { base_path: "/", title: "acme-ui" }
    }
}
```

`ShowcaseApp` is a compiled component from the `dioxus-showcase-ui` crate. It reads the
registry itself — it takes no story list, which is why the generated code stays this small.
Shell improvements reach you through a normal crate version bump rather than a re-scaffold.

`title` is your `project.name`, so the sidebar names the package being showcased rather
than reading a generic "Showcase". Change it to whatever you like — it is your file. A
blank title falls back to `"Showcase"`.

Because this file is write-once, a project scaffolded before `title` existed keeps its old
entry point and still shows the default. Add the argument by hand:

```diff
-        dioxus_showcase_ui::ShowcaseApp { base_path: "/" }
+        dioxus_showcase_ui::ShowcaseApp { base_path: "/", title: "acme-ui" }
```
