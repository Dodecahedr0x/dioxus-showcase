# Getting Started

## 1. Install the tooling

```bash
cargo install dioxus-showcase-cli --locked   # provides the `dioxus-showcase` binary
cargo install dioxus-cli --locked            # provides `dx`, used by dev and export
rustup target add wasm32-unknown-unknown
```

`dx` is only needed for `dev` and `export`, which shell out to it. Everything else —
`check`, `build`, `doctor` — works without it.

Then add the library to the crate that holds your components:

```bash
cargo add dioxus-showcase
```

Run `dioxus-showcase doctor` at any point to see what the tool can find on your machine,
including whether `dx` is installed and which version.

## 2. Annotate a component

In the crate you just added the dependency to:

```rust
use dioxus::prelude::*;
use dioxus_showcase::prelude::*;

#[showcase(tags = ["atoms"])]
#[component]
pub fn PillButton(label: String, disabled: bool) -> Element {
    rsx! { button { disabled, "{label}" } }
}
```

`#[showcase]` goes **above** `#[component]`. That ordering matters: the showcase macro
needs to see the original function signature to generate controls from it.

## 3. Scaffold

```bash
dioxus-showcase init
```

`init` asks two things — which crate holds your components, and where the generated
showcase app should live — and writes the answers to `DioxusShowcase.toml`:

```toml
[project]
name = "my-design-system"
entry_crate = "ui"
showcase_crate = "ui/showcase"

[dev]
port = 6111
host = "127.0.0.1"

[build]
out_dir = "target/showcase"
base_path = "/"
```

Every other command reads those answers back, so you only run `init` once. It also
scaffolds the generated app crate at `showcase_crate`.

<div class="warning">

`init` overwrites `DioxusShowcase.toml`. If you already have one you have customised,
edit it by hand instead of re-running `init`.

</div>

## 4. Run it

```bash
dioxus-showcase dev
```

This rebuilds the manifest in the background and launches `dx serve`. Open
<http://127.0.0.1:6111>.

## 5. Ship it

```bash
dioxus-showcase export
```

You get a folder of plain static files in `target/showcase/site` that any host will serve.
See [Publishing A Static Site](./static-site.md) for per-host recipes.

## If the showcase comes up empty

An empty showcase is almost always one of three things, and the first two fail *silently*
— no build error, no warning:

1. **The entry crate is not linked.** `showcase/src/main.rs` must keep its
   `use <your_crate> as _;` line.
2. **LTO is disabled.** `showcase/Cargo.toml` needs its `[profile.dev]` and
   `[profile.release]` `lto` lines.
3. **Nothing is annotated yet** — the benign case.

Both silent causes are explained in [The Generated App Is Yours](./generated-app.md).
`dioxus-showcase check` warns about the second one, and the showcase's own empty state
names all three.
