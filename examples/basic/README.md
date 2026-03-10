# Workspace Member Showcase Example

This crate is a fully valid Cargo workspace member that includes Dioxus showcase component annotations.

## Why this exists

It demonstrates a project layout where annotated components live inside the workspace member source tree.

## Files

- `src/button_variants.rs`: annotated `#[showcase]` components and `#[story]` functions discovered by `dioxus-showcase`
- `src/lib.rs`: crate-level docs and exported module wiring

## Defining stories

Use `#[showcase]` for an interactive component surface and `#[story]` for fixed named states:

```rust
#[showcase(tags = ["examples", "workspace"])]
#[component]
pub fn PillButtonControllable(label: String, disabled: bool) -> Element {
    rsx! { button { disabled: disabled, "{label}" } }
}

#[story(component = PillButtonControllable, name = "Primary")]
pub fn pill_button_primary() -> Element {
    rsx! { PillButtonControllable { label: "Save Changes".to_string(), disabled: false } }
}
```

## Run with dioxus-showcase

Run these commands from the repository root (`dioxus-preview/`):

```bash
cargo run -p dioxus-showcase-cli -- check
cargo run -p dioxus-showcase-cli -- build
cargo run -p dioxus-showcase-cli -- dev
```

Notes:

- The CLI scans `.rs` files recursively and discovers `#[showcase(...)]` and `#[story(...)]`.
- Running from root will pick up annotated components from workspace members.
