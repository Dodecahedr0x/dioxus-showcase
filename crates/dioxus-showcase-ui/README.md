# dioxus-showcase-ui

The shell UI for [`dioxus-showcase`](https://crates.io/crates/dioxus-showcase).

This crate provides the showcase application shell as real, compiled Dioxus
components — routing, the story tree navigation, tag filtering, the theme
toggle, and the error boundary — instead of generating that code into your
project. Generated `main.rs` shrinks to a short entry point that mounts
`ShowcaseApp` and is never regenerated afterwards.

```rust,ignore
use dioxus::prelude::*;
use my_component_crate as _; // forces rlib linkage so story registrations survive

fn main() {
    launch(App);
}

#[component]
fn App() -> Element {
    rsx! { dioxus_showcase_ui::ShowcaseApp { base_path: "/" } }
}
```

`ShowcaseApp` reads the stories and providers registered by the
`#[showcase]`, `#[story]` and `#[provider]` macros at link time. It takes no
story list.

You normally do not depend on this crate directly — `dioxus-showcase-cli`
scaffolds a project that does.

## License

MIT
