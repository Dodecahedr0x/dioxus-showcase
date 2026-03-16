# Example Workspace Member

This crate is the smallest end-to-end demo of the current `dioxus-showcase` API.

It exists to prove that the pipeline works against an ordinary workspace member, not a special-case demo app. The CLI discovers the annotated code here, the macros generate helper symbols next to it, and the generated showcase shell imports those helpers back out of the crate.

## What It Shows

- `#[provider(index = 0)]` for a shared story wrapper.
- `#[showcase(...)]` on a multi-arg Dioxus component.
- `#[story(...)]` on named story functions.
- test coverage that calls the generated constructor functions directly.

## Key File

- `src/button_variants.rs`

```rust
#[provider(index = 0)]
#[component]
pub fn ExampleStoryShell(children: Element) -> Element {
    rsx! {
        div {
            style: "padding: 24px; background: #f8fafc; border-radius: 18px;",
            {children}
        }
    }
}

#[showcase(tags = ["examples", "workspace"])]
#[component]
pub fn PillButtonControllable(label: String, disabled: bool) -> Element {
    rsx! { button { disabled: disabled, "{label}" } }
}

#[story(title = "PillButtonControllable/Primary", tags = ["examples", "workspace"])]
pub fn pill_button_primary(label: String) -> Element {
    rsx! { PillButtonControllable { label, disabled: false } }
}
```

## Run It From Repo Root

```bash
cargo run -p dioxus-showcase-cli -- check
cargo run -p dioxus-showcase-cli -- build
cargo run -p dioxus-showcase-cli -- dev
```

## Read More

- Full reference: [`../docs/code-reference.md`](../docs/code-reference.md)
- Top-level overview: [`../README.md`](../README.md)
