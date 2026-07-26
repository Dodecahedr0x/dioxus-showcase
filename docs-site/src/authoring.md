# Authoring Stories

Four macros, all re-exported from `dioxus_showcase::prelude`. Each expands **at your own
call site** into an `inventory` registration — nothing outside your crate ever names a
generated symbol.

## `#[showcase]` — put a component in the showcase

Goes on a Dioxus component. Place it **above** `#[component]` so it sees the original
signature.

```rust
#[showcase(tags = ["atoms"])]
#[component]
pub fn PillButton(label: String, disabled: bool) -> Element {
    rsx! { button { disabled, "{label}" } }
}
```

| Argument | Type | Default |
| --- | --- | --- |
| `title` | string literal | the function name |
| `tags` | array of string literals | `[]` |

The route id is derived from the title: `Atoms/Button` becomes `atoms-button`. Ids must be
unique — see [duplicate ids](#duplicate-ids) below.

### Controls are generated from the signature

Each argument becomes a live control in the showcase. Inference covers `String`, `bool`,
and the numeric primitives; `Option<T>` defaults to `None`, `Vec<T>` to empty, `Element` to
placeholder markup, and `EventHandler<T>` to a no-op callback.

A type outside that set needs a `StoryArg` impl, which is a one-method trait:

```rust
impl StoryArg for Size {
    fn story_arg() -> Self { Size::Medium }
}
```

## `#[story]` — a specific state

Where `#[showcase]` renders a component with controls, `#[story]` pins one arrangement.
It goes on a plain function returning `Element` — it does not need to be a component.

```rust
#[story(title = "PillButton/Primary", tags = ["atoms"])]
pub fn pill_button_primary(label: String) -> Element {
    rsx! { PillButton { label, disabled: false } }
}

#[story(title = "PillButton/Disabled", tags = ["atoms"])]
pub fn pill_button_disabled() -> Element {
    rsx! { PillButton { label: "Unavailable".into(), disabled: true } }
}
```

Takes the same `title` and `tags` arguments. A story with parameters still gets controls
for them; a story with none renders a fixed state.

Titles containing `/` nest in the sidebar tree, so the two above group under one
`PillButton` node.

## `#[provider]` — wrap every story

For theme providers, context, routers — anything every story needs around it.

```rust
#[provider(order = 0)]
#[component]
pub fn ExampleStoryShell(children: Element) -> Element {
    rsx! {
        div { style: "padding: 24px; background: #f8fafc; border-radius: 18px;",
            {children}
        }
    }
}
```

| Argument | Type | Default |
| --- | --- | --- |
| `order` | `i32` | `0` |

**Lowest `order` wraps outermost.** A provider at `order = -10` sits outside one at
`order = 5`. Providers are applied in ascending order and sorted deterministically, so
nesting does not depend on link order or file layout.

<div class="warning">

`#[provider(index = N)]` was the `0.0.x` spelling and is **rejected** as of `0.1.0`. The
macro names `order` as its replacement in the error. The meaning is unchanged.

</div>

## `#[derive(StoryProps)]` — one component, many variants

When a component takes an aggregate props struct, deriving `StoryProps` turns each named
variant into its own story.

```rust
#[derive(Props, Clone, PartialEq, StoryProps)]
pub struct CardProps {
    pub title: String,
    pub elevated: bool,
}

#[showcase(title = "Molecules/Card")]
#[component]
pub fn Card(props: CardProps) -> Element { /* … */ }
```

The derive produces a single default variant from each field's `StoryArg`. To control the
set, implement `StoryProps` by hand and return named variants — each becomes a separate
story titled `<base title>/<variant name>`:

```rust
impl StoryProps for CardProps {
    fn stories() -> Vec<StoryVariant<Self>> {
        vec![
            StoryVariant::named("Flat", CardProps { title: "Flat".into(), elevated: false }),
            StoryVariant::named("Elevated", CardProps { title: "Raised".into(), elevated: true }),
        ]
    }
}
```

`StoryVariant::unnamed(value)` keeps the base title unchanged.

## Where annotations are found

Discovery follows `mod foo;` declarations through the module graph from your entry crate's
root, so components in submodules and separate files are picked up without configuration.

Note that discovery is **advisory** — it drives `check` diagnostics and the manifest, not
what actually renders. See [How It Works](./how-it-works.md).

## Duplicate ids

Two stories whose titles slugify to the same id are a configuration error, but not a fatal
one. Both stay in the registry and both remain listed; the showcase renders a banner naming
the collision, and `/component/<id>` resolves to the first in sort order.

`dioxus-showcase check` reports collisions as an **error**; `build` downgrades them to a
warning. Run `check` in CI if you want them to fail your pipeline.
