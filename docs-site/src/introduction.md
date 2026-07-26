# Dioxus Showcase

Storybook-style tooling for [Dioxus](https://dioxuslabs.com). Annotate your components,
run one command, and get a browsable showcase app — or a static website you can deploy
anywhere.

<div class="warning">

**Status: alpha (`0.x`).** It works and it is used, but the API and the generated output
still break between minor releases. Concretely: breaking changes ship in `0.x` releases,
there is no deprecation cycle and there are no compatibility shims, and the
[changelog](./changelog.md) is the only notice you get. Read the section for a version
before upgrading to it, and pin an exact version if you need stability.

</div>

## Requires Rust 1.85 or newer

This is a hard floor, not a formality. Story registration rides on
[`inventory`](https://docs.rs/inventory), whose `wasm32-unknown-unknown` support is
version-gated on 1.85. Below that the registry comes back *empty with no error* — your
showcase would build, launch, and show nothing. It cannot be lowered.

## What it looks like

You annotate ordinary Dioxus functions:

```rust
use dioxus::prelude::*;
use dioxus_showcase::prelude::*;

#[showcase(tags = ["atoms"])]
#[component]
pub fn PillButton(label: String, disabled: bool) -> Element {
    rsx! { button { disabled, "{label}" } }
}

#[story(title = "PillButton/Primary", tags = ["atoms"])]
pub fn pill_button_primary(label: String) -> Element {
    rsx! { PillButton { label, disabled: false } }
}
```

Then run one command:

```bash
dioxus-showcase dev      # live showcase at http://127.0.0.1:6111
```

You get a browsable app with tag filters, tree navigation, theme switching, and generated
controls for your component's arguments.

## Where to go next

| If you want to… | Read |
| --- | --- |
| Install it and see something on screen | [Getting Started](./getting-started.md) |
| Know what the macros accept | [Authoring Stories](./authoring.md) |
| Understand what `build` will and will not overwrite | [The Generated App Is Yours](./generated-app.md) |
| Deploy a static showcase | [Publishing A Static Site](./static-site.md) |
| Understand the registration architecture | [How It Works](./how-it-works.md) |

## The crates

Five crates are published to crates.io. Most projects only ever name two of them —
`dioxus-showcase` as a dependency and `dioxus-showcase-cli` as an installed binary.

| Crate | Responsibility |
| --- | --- |
| [`dioxus-showcase`](https://crates.io/crates/dioxus-showcase) | Public facade: registration types and the `registered_stories()` / `registered_providers()` readers. **This is what you depend on.** |
| [`dioxus-showcase-cli`](https://crates.io/crates/dioxus-showcase-cli) | The `dioxus-showcase` binary: discovery, scaffolding, generation, `dx serve`, and `dx bundle`. |
| [`dioxus-showcase-macros`](https://crates.io/crates/dioxus-showcase-macros) | `#[showcase]`, `#[story]`, `#[provider]`, `#[derive(StoryProps)]`, and the registrations they emit. Re-exported through the facade. |
| [`dioxus-showcase-ui`](https://crates.io/crates/dioxus-showcase-ui) | The showcase shell: routing, navigation, tag filters, theme toggle, error states. A compiled component, not generated code. |
| [`dioxus-showcase-core`](https://crates.io/crates/dioxus-showcase-core) | Config, manifest, and shared data types. No dependency on `dioxus`. |

## License

MIT.
