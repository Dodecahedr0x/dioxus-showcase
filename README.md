# Dioxus Showcase

Storybook-style tooling for [Dioxus](https://dioxuslabs.com). Annotate your components,
run one command, and get a browsable showcase app — or a static website you can deploy
anywhere.

**[Documentation](https://dodecahedr0x.github.io/dioxus-showcase/)** ·
**[Live showcase demo](https://dodecahedr0x.github.io/dioxus-showcase/showcase/)**

> **Status: alpha (`0.x`).** It works and it is used, but the API and the generated
> output still break between minor releases. Concretely, what alpha means here:
> breaking changes ship in `0.x` releases, there is no deprecation cycle and there are
> no compatibility shims, and [`CHANGELOG.md`](CHANGELOG.md) is the only notice you get.
> Read the section for a version before upgrading to it. Pin an exact version if you
> need stability.

**Requires Rust 1.85 or newer.** This is a hard floor, not a formality: story
registration rides on `inventory`, whose `wasm32-unknown-unknown` support is
version-gated on 1.85, and below that the registry comes back *empty with no error* —
your showcase would build, launch, and show nothing. It cannot be lowered.

## Using It In Your Project

**1. Install the CLI and add the library.**

```bash
cargo install dioxus-showcase-cli --locked   # provides the `dioxus-showcase` binary
cargo install dioxus-cli --locked            # provides `dx`, used by dev and export
rustup target add wasm32-unknown-unknown

cargo add dioxus-showcase
```

**2. Annotate components in your UI crate.**

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

**3. Set up and run.**

```bash
dioxus-showcase init     # writes DioxusShowcase.toml and scaffolds the showcase app
dioxus-showcase dev      # live showcase at http://127.0.0.1:6111
dioxus-showcase export   # static site in target/showcase/site
```

`init` asks which crate holds your components and where the generated app should live. Every
other command reads those answers back from `DioxusShowcase.toml`, so you only run it once.

## The Generated App Is Yours

`showcase/src/main.rs` is written **once**, when it does not exist, and is then
**never regenerated**. After that first write it is your file: edit it, restructure it,
add providers and routes to it. `build` will not overwrite it, diff it, or migrate it,
and there is no `--force`.

`showcase/Cargo.toml` is written once on exactly the same terms, because you add your own
dependencies to it. What `build` *does* rewrite on every run is `src/generated.rs` (a build
marker the app does not even compile) and `Dioxus.toml`, and it re-syncs your entry crate's
`assets/` into the generated app.

| Generated file | Rewritten? |
| --- | --- |
| `showcase/src/main.rs` | Written once, when absent. Yours afterwards. |
| `showcase/Cargo.toml` | Written once, when absent. Yours afterwards. |
| `showcase/src/generated.rs` | Every build. |
| `showcase/Dioxus.toml` | Every build. |
| `showcase/assets/` | Re-synced from your entry crate's `assets/` every build. |

The flip side of write-once is that improvements to the entry point and the manifest never
reach an existing project on their own. If you add a stylesheet to your entry crate's
`assets/` directory *after* scaffolding, `build` will not add the `document::Stylesheet`
line for it — you add that line by hand. Same for anything else a newer release starts
emitting, **including the two `lto` lines below**: a project scaffolded before they existed
does not have them, whether or not you ever touched the file. To adopt a new entry point
wholesale, delete `showcase/src/main.rs` and run `dioxus-showcase build` — after bringing
`showcase/Cargo.toml` up to date by hand, since the regenerated entry point depends on
things an older manifest does not declare. See the upgrade notes in
[`CHANGELOG.md`](CHANGELOG.md) for the exact lines.

### Two lines you must not delete

Both of these look like clutter and both fail **silently** when removed — no error, no
warning, just an empty showcase. That is exactly why they are documented here.

**1. `use <your_crate> as _;` in `showcase/src/main.rs`.**

```rust
use showcase_entry as _; // LOAD-BEARING: forces rlib linkage so registrations survive
```

Nothing calls into your crate, so every linter and every instinct says this import is dead.
It is the entire reason any story appears: it is what links your crate into the binary, and
your crate is what carries the registrations.

**2. The `lto` lines in the generated `showcase/Cargo.toml`.**

```toml
[profile.dev]
lto = "thin"

[profile.release]
lto = true
```

These are **not** a speed knob. On `wasm32`, the import above does not on its own pull your
crate's object out of its rlib archive — nothing in the binary references a symbol inside
it, so the linker never selects the archive member and every registration in it is dropped.
LTO merges the crate graph before that selection happens, which is what keeps them. Remove
these lines and both `dx build` and `dx bundle --release` produce a showcase with zero
stories and no error anywhere.

The honest cost: thin LTO makes `dx serve` rebuilds slower and disables incremental
compilation for the showcase app. `dioxus-showcase check` warns if the lines go missing.

## Commands

| Command | What it does |
| --- | --- |
| `init` | Prompt for `DioxusShowcase.toml` values and scaffold the generated app crate. |
| `check` | Validate config, discovery, duplicate ids, and scaffold presence. |
| `build` | Write the manifest and generated runtime files. |
| `build --watch` | Rebuild when annotated source files change. |
| `dev` | Rebuild in the background and launch `dx serve`. |
| `export` | Build a deployable static website of the showcased components. |
| `doctor` | Print host diagnostics, including whether `dx` is installed. |

## Publishing A Static Showcase Site

`export` turns the showcase into a folder of plain files that any static host can serve:

```bash
# Served from a domain root (Netlify, Vercel, Cloudflare Pages, user/org GitHub Pages)
dioxus-showcase export

# Served from a sub-path (project GitHub Pages at https://<user>.github.io/<repo>/)
dioxus-showcase export --out-dir dist/showcase --base-path /<repo>
```

The site lands in `<build.out_dir>/site` by default and contains `index.html`, a hashed
`assets/` directory, a `404.html` fallback so deep story routes survive a refresh, and a
`.nojekyll` marker for GitHub Pages. Preview it with any static file server:

```bash
python3 -m http.server --directory target/showcase/site
```

Copy-paste deployment recipes for GitHub Pages, Netlify, Vercel, and Cloudflare Pages are
in [`docs/static-site.md`](docs/static-site.md).

## How It Works

1. You annotate Dioxus functions with `#[showcase]`, `#[story]`, and `#[provider]`. Each
   macro expands, at your own call site, into an `inventory` registration carrying the
   item's source path, module path, and a factory function pointer. **This is the runtime
   source of truth.** Nothing outside your crate names a generated symbol.
2. `dioxus-showcase build` writes `target/showcase/showcase.manifest.json` and the showcase
   app's `src/generated.rs`. It scaffolds `showcase/src/main.rs` only if that file is
   absent.
3. `dioxus_showcase_ui::ShowcaseApp` — a compiled component from the `dioxus-showcase-ui`
   crate, not generated code — reads the registry at startup, sorts it deterministically,
   builds routes at `/component/:id`, and renders each story in a shell with tag filters,
   tree navigation, and theme switching. Duplicate story ids are reported in a banner
   rather than panicking, and both colliding stories stay navigable.
4. The CLI's AST scan of your entry crate is **advisory**. It powers exactly two things:
   `check` diagnostics and the manifest. `check` reports drift as an error; `build`
   downgrades it to a warning, because nothing at runtime depends on it. Run `check` in CI
   if you want id collisions to fail your pipeline.

## Working On This Repo

Everything runs from a fresh clone with no setup step — the repo's own
`DioxusShowcase.toml` is checked in and points at `example/`:

```bash
cargo test --workspace --all-targets --all-features
cargo run -p dioxus-showcase-cli -- check
cargo run -p dioxus-showcase-cli -- dev
```

See [`CONTRIBUTING.md`](CONTRIBUTING.md) for prerequisites, the pre-PR checklist, which
files are generated, and the release process.

### The Shape Of The Repo

| Path | Responsibility |
| --- | --- |
| `crates/dioxus-showcase-core` | Config, manifest, provider metadata, navigation helpers. |
| `crates/dioxus-showcase-macros` | `#[showcase]`, `#[story]`, `#[provider]`, `#[derive(StoryProps)]`, and the `inventory` registrations they emit. |
| `crates/dioxus-showcase` | Public facade: the registration types and the `registered_stories()` / `registered_providers()` readers. |
| `crates/dioxus-showcase-ui` | The showcase shell: routing, navigation, tag filters, theme toggle, error states. |
| `crates/dioxus-showcase-cli` | Advisory discovery, scaffolding, generation, `dx serve`, and `dx bundle`. No longer owns the shell. |
| `example` | A working annotated crate that exercises the end-to-end pipeline. |

The first five are published to crates.io; `example` is not.

## Documentation

The full documentation site is at
<https://dodecahedr0x.github.io/dioxus-showcase/>, with the example crate's showcase
deployed live at
[`/showcase/`](https://dodecahedr0x.github.io/dioxus-showcase/showcase/) — built by this
tool's own `export` command on every push to `main`. Its source is the mdBook in
[`docs-site/`](docs-site/).

- Release history and breaking changes: [`CHANGELOG.md`](CHANGELOG.md)
- Static site publishing: [`docs/static-site.md`](docs/static-site.md)
- Contributor guide: [`CONTRIBUTING.md`](CONTRIBUTING.md)
- File-by-file code reference: [`docs/code-reference.md`](docs/code-reference.md)
- Example walkthrough: [`example/README.md`](example/README.md)
- Backlog: [`docs/improvement-ideas.md`](docs/improvement-ideas.md)
- Design intent: [`docs/rfcs/dioxus-showcase.md`](docs/rfcs/dioxus-showcase.md)

## License

MIT
