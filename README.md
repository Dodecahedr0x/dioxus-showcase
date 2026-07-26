# Dioxus Showcase

Storybook-style tooling for [Dioxus](https://dioxuslabs.com). Annotate your components,
run one command, and get a browsable showcase app — or a static website you can deploy
anywhere.

> Status: early prototype. The API and generated output still change between releases.

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

1. You annotate Dioxus functions with `#[showcase]`, `#[story]`, and `#[provider]`. The
   macros emit helper symbols next to each annotated item.
2. The CLI scans the configured entry crate and collects that metadata.
3. It writes `target/showcase/showcase.manifest.json` plus the showcase app's
   `src/generated.rs` and `src/main.rs`.
4. The generated app imports your crate, builds routes at `/component/:id`, and renders
   each story in a shell with tag filters, tree navigation, and theme switching.

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
| `crates/dioxus-showcase-macros` | `#[showcase]`, `#[story]`, `#[provider]`, `#[derive(StoryProps)]`. |
| `crates/dioxus-showcase` | Public facade and trait surface for app code. |
| `crates/dioxus-showcase-cli` | Discovery, scaffolding, generation, `dx serve`, and `dx bundle`. |
| `example` | A working annotated crate that exercises the end-to-end pipeline. |

## Documentation

- Static site publishing: [`docs/static-site.md`](docs/static-site.md)
- Contributor guide: [`CONTRIBUTING.md`](CONTRIBUTING.md)
- File-by-file code reference: [`docs/code-reference.md`](docs/code-reference.md)
- Example walkthrough: [`example/README.md`](example/README.md)
- Backlog: [`docs/improvement-ideas.md`](docs/improvement-ideas.md)
- Design intent: [`docs/rfcs/dioxus-showcase.md`](docs/rfcs/dioxus-showcase.md)

## License

MIT
