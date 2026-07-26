# Command Reference

All commands read `DioxusShowcase.toml` from the working directory, except `init`, which
writes it.

| Command | What it does | Needs `dx`? |
| --- | --- | --- |
| [`init`](#init) | Prompt for config values and scaffold the generated app crate | no |
| [`check`](#check) | Validate config, discovery, duplicate ids, and scaffold presence | no |
| [`build`](#build) | Write the manifest and generated runtime files | no |
| [`dev`](#dev) | Rebuild in the background and launch `dx serve` | yes |
| [`export`](#export) | Build a deployable static website | yes |
| [`doctor`](#doctor) | Print host diagnostics | no |

## `init`

Interactive. Asks which crate holds your components and where the generated app should
live, writes `DioxusShowcase.toml`, and scaffolds the showcase crate.

Run once per project. It **overwrites** `DioxusShowcase.toml`, so edit that file by hand
rather than re-running `init` on a project you have already configured.

## `check`

The authoritative validation pass. Reports, as **errors**:

- config that parses but is semantically unusable, and unknown config keys
- duplicate story ids
- a missing or malformed scaffold
- a `showcase/Cargo.toml` that has lost its [`lto` lines](./generated-app.md#2-the-lto-lines-in-showcasecargotoml)

`check` never compiles your crate, so it stays fast — it is the right thing to run in CI.
This matters because `build` deliberately downgrades every discovery problem to a warning:

```bash
dioxus-showcase check   # fails the pipeline on id collisions
```

## `build`

Writes `target/showcase/showcase.manifest.json` and the showcase app's
`src/generated.rs`, and re-syncs assets. Scaffolds `showcase/src/main.rs` and
`showcase/Cargo.toml` **only if absent** — see
[The Generated App Is Yours](./generated-app.md).

Generation is deterministic: the same inputs produce byte-identical output, and the
generation token is content-derived rather than timestamped.

| Flag | Effect |
| --- | --- |
| `--watch` | Rebuild when annotated source files change |

Discovery failures are warnings here, not errors. Use `check` when you want them fatal.

## `dev`

Rebuilds the manifest in the background and launches `dx serve`, using the `[dev]` host and
port from your config (default `127.0.0.1:6111`).

## `export`

Builds a static site: `index.html`, a hashed `assets/` directory, a `404.html` fallback so
deep story routes survive a refresh, and a `.nojekyll` marker for GitHub Pages. The output
directory is cleared first, so assets from earlier builds are not left behind.

| Flag | Default | Effect |
| --- | --- | --- |
| `--out-dir <DIR>` | `<build.out_dir>/site` | Where to write the site |
| `--base-path <PATH>` | `build.base_path` | Public sub-path the site is served from |
| `--debug` | off | Build in debug instead of release |

```bash
# Served from a domain root
dioxus-showcase export

# Served from a sub-path, e.g. project GitHub Pages
dioxus-showcase export --out-dir dist/showcase --base-path /my-repo
```

Preview the result with any static file server:

```bash
python3 -m http.server --directory target/showcase/site
```

Per-host recipes are in [Publishing A Static Site](./static-site.md).

## `doctor`

Prints host diagnostics: whether `dx` is on `PATH` and which version, plus what the tool
can see of your project. Start here when something behaves unexpectedly.
