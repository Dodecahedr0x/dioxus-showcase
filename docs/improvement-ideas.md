# Dioxus Showcase Improvement Inventory

This is a **backlog of ideas**, not a description of how the project works today. For
current behavior read [`../README.md`](../README.md) and
[`code-reference.md`](code-reference.md); for where the project is heading read
[`rfcs/dioxus-showcase.md`](rfcs/dioxus-showcase.md).

Scoring model:

- `Priority`: `P0` critical foundation, `P1` high leverage, `P2` worthwhile, `P3` nice-to-have
- `Impact`: `1-5`, where `5` materially changes correctness, adoption, or maintainability
- `Difficulty`: `1-5`, where `5` is a large cross-cutting effort
- `Global score = priority weight + impact - difficulty`
- Priority weights: `P0 = 8`, `P1 = 6`, `P2 = 4`, `P3 = 2`

Higher score means better candidate for near-term planning. When an idea ships, delete its
row and add a line to *Already shipped* so the table only ever describes remaining work.

## Already Shipped

These were the original first wave. They are done, and the table below no longer lists them:

- Config and manifest parsing use `serde`, `toml`, and `serde_json` instead of hand-rolled
  parsers, and unknown config keys are rejected rather than ignored.
- Discovery follows `mod foo;` declarations through the module graph instead of only
  handling inline modules.
- The generated shell wraps story rendering in an `ErrorBoundary`, so one broken story no
  longer takes down the whole app.
- `build.base_path` is honored end to end, including asset URLs in exported sites.
- Generated artifacts are deterministic; the generation token is content-derived rather
  than timestamp-derived.
- Golden-file tests cover the generated `main.rs`, `generated.rs`, and manifest JSON.
- `doctor` reports whether the `dx` binary is installed and which version it is.
- The publish workflow releases crates in dependency order, and
  `scripts/set-workspace-version.sh` keeps every internal dependency version in sync.
- `export` produces a deployable static site, clearing its output directory and pruning
  assets left behind by earlier builds.
- A contributor guide exists at [`../CONTRIBUTING.md`](../CONTRIBUTING.md).

## Backlog

| Area | Improvement idea | Priority | Impact | Difficulty | Global score | Why it matters |
| --- | --- | --- | --- | --- | --- | --- |
| Workspace architecture | Define a stable registration contract so the CLI/runtime stops depending on generated symbol naming conventions | P0 | 5 | 4 | 9 | Current discovery and runtime assembly are tightly coupled to macro-generated `__dioxus_showcase_*` symbols and include-style glue. |
| Workspace architecture | Remove duplicated helpers such as `slugify_title` and metadata parsing by centralizing them in `core` or a dedicated shared crate module | P1 | 4 | 2 | 8 | The same logic exists in the facade, macros, and CLI, increasing the chance of inconsistent IDs and discovery behavior. |
| Workspace architecture | Add explicit MSRV, compatibility matrix, and release policy in crate metadata and docs | P2 | 3 | 1 | 6 | The RFC promises compatibility management, but the workspace does not yet declare it. |
| Workspace architecture | Introduce benchmark targets for discovery time, manifest generation, and showcase startup | P2 | 3 | 2 | 5 | The RFC has performance targets, but nothing currently measures them. |
| `dioxus-showcase-core` config | Add semantic validation for paths, host/base path normalization, and invalid combinations | P1 | 4 | 2 | 8 | Parsing currently succeeds even when the config is semantically unusable. |
| `dioxus-showcase-core` config | Support layered config sources: file, env vars, and CLI overrides | P2 | 3 | 3 | 4 | This becomes important once the tool is used in CI and multiple environments. |
| `dioxus-showcase-core` manifest | Add manifest schema evolution support with versioned structs and compatibility tests | P1 | 4 | 3 | 7 | The RFC calls for a stable schema, but the code has only a hard-coded `schema_version = 1`. |
| `dioxus-showcase-core` runtime | Add deterministic sorting for navigation tree children, not just top-level story order | P1 | 4 | 2 | 8 | Stories are sorted before generation, but nested navigation ordering still follows discovery order. |
| `dioxus-showcase-core` runtime | Add duplicate-title and malformed-title validation helpers, not just duplicate IDs | P2 | 3 | 2 | 5 | Navigation and UX degrade when titles are empty, inconsistent, or collide in confusing ways. |
| `dioxus-showcase-core` runtime | Add richer story metadata in shared types: description, docs URL, decorators, viewport/background settings, arg schema | P1 | 5 | 4 | 7 | Most roadmap features need shared data structures before they can be built cleanly. |
| `dioxus-showcase-macros` overall | Add `trybuild`-style compile-fail tests for invalid attribute arguments and unsupported signatures | P0 | 5 | 2 | 11 | The macro test suite mostly checks happy paths; the most important proc-macro regressions are error-path regressions. |
| `dioxus-showcase-macros` overall | Replace `compile_error!(format!("{:?}", err))` style diagnostics with proper `syn::Error` spans | P1 | 4 | 2 | 8 | Error reporting quality is one of the biggest adoption multipliers for proc-macro crates. |
| `dioxus-showcase-macros` overall | Split metadata extraction from UI/control rendering so proc macros do less runtime UI work | P1 | 4 | 4 | 6 | The macros currently own authoring API, runtime wiring, and part of the preview UI, which is a high-coupling design. |
| `#[showcase]` macro | Support explicit story IDs and collision-resistant defaults based on module path plus title | P1 | 4 | 3 | 7 | Title-only slugging can collide easily in real component libraries. |
| `#[showcase]` macro | Stop assuming the first argument named `props` is the only aggregate-props form worth special-casing | P1 | 4 | 2 | 8 | The current rule is narrow and will surprise users with differently named props bindings. |
| `#[showcase]` macro | Support decorators, docs text, and non-default named variants without requiring custom props impl boilerplate | P2 | 4 | 3 | 5 | This would move the API closer to the RFC's addon and docs ambitions. |
| `#[story]` macro | Validate return types and story signatures more explicitly, including better messages for unsupported patterns | P1 | 4 | 2 | 8 | The current API accepts a narrow set of shapes but does not make those constraints obvious. |
| `#[story]` macro | Add arg-schema metadata generation rather than only live control widgets | P1 | 5 | 4 | 7 | Without a serializable arg schema, docs, static export, testing, and addons remain limited. |
| `StoryProps` derive | Generate field-level defaults or custom variant hooks instead of only wrapping `Default::default()` | P2 | 3 | 3 | 4 | Today the derive is convenient but too shallow for realistic component props. |
| Macro utilities | Expand control inference beyond `String`, `bool`, and numeric primitives | P1 | 4 | 3 | 7 | Control generation is one of the main value propositions, and it currently covers only the easiest scalar cases. |
| Macro utilities | Add escape hatches for custom controls and hidden args | P2 | 4 | 3 | 5 | Real component APIs often contain callbacks, slots, IDs, or advanced types that should not become default controls. |
| `dioxus-showcase` facade | Add trait docs and examples for `StoryArg`, `StoryArgs`, `StoryProps`, and `ShowcaseStoryFactory` | P2 | 3 | 1 | 6 | The public API surface is small but under-documented relative to its importance. |
| `dioxus-showcase` facade | Reduce dependency surface and re-export only what is required by generated code and user ergonomics | P2 | 3 | 2 | 5 | The facade currently pulls in multiple Dioxus crates and mixes runtime, macros, and authoring concerns. |
| `dioxus-showcase` facade | Add built-in impls for common standard library/container types that users will hit quickly | P2 | 3 | 2 | 5 | Current defaults are enough for demos but not for realistic props trees. |
| CLI UX | Add non-interactive `init` flags and avoid always prompting | P1 | 4 | 2 | 8 | Interactive-only init is awkward for automation and scaffolding tools. |
| CLI UX | Add machine-readable output modes such as JSON for `check`, `build`, and `doctor` | P1 | 4 | 2 | 8 | CI and editor integrations become much easier once commands can emit structured diagnostics. |
| CLI UX | Add clearer error categorization with actionable hints per failure mode | P1 | 4 | 2 | 8 | Most command errors are currently plain strings with little recovery guidance. |
| CLI discovery | Stop AST rescanning the source tree separately from macro expansion and move toward explicit registration artifacts | P0 | 5 | 4 | 9 | Discovery currently duplicates macro parsing rules and will continue to drift from the proc-macro behavior. |
| CLI discovery | Detect and report invalid combinations such as duplicate titles, broken module paths, missing story symbols, and non-component functions | P1 | 4 | 3 | 7 | `check` should be the authoritative validation pass before generation. |
| CLI discovery | Respect `.gitignore`/workspace exclusions and make ignore rules configurable | P2 | 3 | 2 | 5 | Recursive scans will become noisy and slow in larger workspaces. |
| CLI build | Prune stale copied assets during `build`, the way `export` already prunes its site output | P2 | 3 | 2 | 5 | Asset syncing into the generated app is still additive and can leave behind dead files. |
| CLI build | Support incremental generation so a single changed file does not rewrite the entire showcase app | P2 | 3 | 4 | 3 | This matters once story counts grow and the dev loop slows down. |
| CLI dev loop | Replace polling-based watches with filesystem notifications using `notify` or equivalent | P1 | 4 | 2 | 8 | Polling every 700ms is simple but wasteful and less responsive. |
| CLI dev loop | Manage child process lifecycle and signal forwarding more carefully, especially for Ctrl+C and failure states | P1 | 4 | 3 | 7 | The current thread/process model is workable but not very robust. |
| CLI scaffold | Stop overwriting `showcase/src/main.rs` on every build, or split generated shell from user-editable shell extensions | P0 | 5 | 3 | 10 | Current regeneration makes the scaffold hard to customize safely. |
| CLI scaffold | Preserve user-authored assets/custom files and support partial template upgrades | P1 | 4 | 3 | 7 | The scaffold is currently "owned" by the generator, which is acceptable for a prototype but poor for adoption. |
| CLI scaffold | Add template versioning and migration support | P2 | 3 | 3 | 4 | Once generated apps exist in the wild, upgrades need a supported story. |
| Showcase app template | Cache or memoize `showcase_components()`/derived tag data instead of rebuilding on every render path | P1 | 4 | 2 | 8 | The current template repeatedly rebuilds vectors and trees, which will scale poorly. |
| Showcase app template | Move the large inline CSS and shell UI into template assets/components with tests | P2 | 3 | 2 | 5 | The generated `main.rs` is doing too much and will get harder to evolve. |
| Showcase app template | Add search, sort, and keyboard navigation for larger story sets | P2 | 3 | 2 | 5 | The current tree-only navigation will become clumsy quickly. |
| Showcase app template | Add empty/loading states for missing assets and invalid generated metadata | P2 | 3 | 2 | 5 | Story render failures are handled, but missing assets and malformed metadata are not. |
| Example crate | Expand the example to cover props structs, tags, multi-file modules, decorators, slots, and failure cases | P1 | 4 | 2 | 8 | The current example proves the happy path but not the tricky patterns users will copy. |
| Testing | Add integration tests that run `init`, `check`, `build`, and `export` end to end against fixture workspaces on disk | P0 | 5 | 3 | 10 | Golden tests cover artifact writing, but the command layer that orchestrates it is untested. |
| Testing | Add UI or browser smoke tests for the generated showcase shell | P1 | 4 | 3 | 7 | Route rendering, tag filtering, and asset loading are currently verified only by hand. |
| Docs and RFC | Sync the RFC, README, and actual API names/features | P1 | 4 | 2 | 8 | The RFC still references `StoryArgs` derive and capabilities that differ from the implemented surface. |
| Docs and RFC | Add architecture diagrams and a "current limitations" page separate from aspirational roadmap text | P2 | 3 | 1 | 6 | This helps contributors understand what is deliberately incomplete versus accidentally missing. |
| CI | Split CI into fast checks and slower integration/browser jobs | P2 | 3 | 2 | 5 | The current workflow is simple but will not scale well as tests grow. |
| CI | Run doctests and package verification per crate rather than only whole-workspace commands | P2 | 3 | 2 | 5 | This gives better signal for crate-level publish health. |
| Publish/release | Add release dry-run checks for generated scaffold contents and example workflows | P2 | 3 | 2 | 5 | Publishing the libraries without validating the generated app path leaves a gap in release confidence. |
| Observability | Add verbose/debug logging modes for discovery, generation, and watch events | P2 | 3 | 1 | 6 | This will matter once users hit path/module issues in nontrivial workspaces. |
| Security/reliability | Harden path handling around recursive copies and generated dependency paths | P2 | 3 | 2 | 5 | Most inputs are local, but path normalization and surprising relative paths are still easy footguns. |
| Product direction | Decide whether the long-term source of truth is AST discovery, macro registration, or generated manifests, and align every crate around that choice | P0 | 5 | 4 | 9 | This is the highest-leverage strategic decision in the repo because several current rough edges come from mixed models. |

## Suggested Next Wave

Ranked by score, the strongest remaining candidates are:

1. Add `trybuild` compile-fail tests for the macros (score 11) — error paths are where
   proc-macro regressions actually happen.
2. Stop overwriting `showcase/src/main.rs` on every build (score 10) — the single biggest
   obstacle to customizing a generated showcase.
3. Add end-to-end tests for the command layer (score 10) — the orchestration between
   discovery, generation, and scaffolding is the least covered code in the repo.
4. Decide the source of truth between AST discovery and macro registration (score 9) — a
   strategic call that several other items depend on.
5. Define a stable registration contract so generated symbol names stop being load-bearing
   (score 9).
