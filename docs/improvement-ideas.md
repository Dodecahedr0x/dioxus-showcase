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

The second wave shipped with `v0.1.0`:

- Stories, providers, and components register themselves at link time through `inventory`,
  so the CLI no longer emits or re-derives `__dioxus_showcase_*` symbol names. Those names
  still exist inside the crate that defines a story, but nothing across a crate boundary
  depends on them.
- AST discovery is advisory. It powers `check` diagnostics and the manifest only; the
  runtime reads the registry instead, so discovery drifting from macro behaviour degrades
  diagnostics rather than breaking the app.
- `showcase/src/main.rs` is written once, when it is absent, and is never overwritten. The
  generated app is the user's to edit from that point on.
- The shell moved out of the CLI's inline template into `dioxus-showcase-ui`, a published
  crate of real Dioxus components with its own CSS asset and component tests rendered
  through `dioxus-ssr`.
- Template upgrades reach new projects only. Because `main.rs` is written once and never
  migrated, a newer CLI changes what `init` scaffolds and deliberately leaves existing
  projects alone — there is no migration mechanism, and that is the answer rather than a
  gap waiting to be filled.
- The macros have `trybuild` compile-fail coverage for their error paths, running ungated
  on stable alongside the unit tests over `expand()`.
- `init`, `check`, `build`, and `export` are covered end to end by on-disk tests against
  fixture workspaces, not only by golden files.
- The workspace declares a measured MSRV of `1.85` and complete crates.io metadata
  (`repository`, `readme`, `keywords`, `categories`, `rust-version`) across all five
  published crates, and CI enforces the MSRV on every pull request.

## Backlog

| Area | Improvement idea | Priority | Impact | Difficulty | Global score | Why it matters |
| --- | --- | --- | --- | --- | --- | --- |
| Workspace architecture | Remove duplicated helpers such as `slugify_title` and metadata parsing by centralizing them in `core` or a dedicated shared crate module | P1 | 4 | 2 | 8 | The same logic exists in the facade, macros, and CLI, increasing the chance of inconsistent IDs and discovery behavior. |
| Workspace architecture | Introduce benchmark targets for discovery time, manifest generation, and showcase startup | P2 | 3 | 2 | 5 | The RFC has performance targets, but nothing currently measures them. |
| `dioxus-showcase-core` config | Add semantic validation for paths, host/base path normalization, and invalid combinations | P1 | 4 | 2 | 8 | Parsing currently succeeds even when the config is semantically unusable. |
| `dioxus-showcase-core` config | Support layered config sources: file, env vars, and CLI overrides | P2 | 3 | 3 | 4 | This becomes important once the tool is used in CI and multiple environments. |
| `dioxus-showcase-core` manifest | Add manifest schema evolution support with versioned structs and compatibility tests | P1 | 4 | 3 | 7 | The RFC calls for a stable schema, but the version is a hard-coded integer — bumped by hand from `1` to `2` in `v0.1.0` — with no versioned structs and no compatibility tests behind it. |
| `dioxus-showcase-core` runtime | Add deterministic sorting for navigation tree children, not just top-level story order | P1 | 4 | 2 | 8 | Stories are sorted before generation, but nested navigation ordering still follows discovery order. |
| `dioxus-showcase-core` runtime | Add duplicate-title and malformed-title validation helpers, not just duplicate IDs | P2 | 3 | 2 | 5 | Navigation and UX degrade when titles are empty, inconsistent, or collide in confusing ways. |
| `dioxus-showcase-core` runtime | Add richer story metadata in shared types: description, docs URL, decorators, viewport/background settings, arg schema | P1 | 5 | 4 | 7 | Most roadmap features need shared data structures before they can be built cleanly. |
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
| CLI discovery | Stop AST rescanning the source tree separately from macro expansion, now that registration is authoritative and nothing at runtime reads discovery | P1 | 4 | 4 | 6 | Discovery still duplicates the macros' attribute-parsing rules in a second parser, so the two will keep drifting. `v0.1.0` settled which side wins but left both in the tree. |
| CLI discovery | Detect and report invalid combinations such as duplicate titles, broken module paths, missing story symbols, and non-component functions | P1 | 4 | 3 | 7 | `check` should be the authoritative validation pass before generation. |
| CLI discovery | Respect `.gitignore`/workspace exclusions and make ignore rules configurable | P2 | 3 | 2 | 5 | Recursive scans will become noisy and slow in larger workspaces. |
| CLI build | Prune stale copied assets during `build`, the way `export` already prunes its site output | P2 | 3 | 2 | 5 | Asset syncing into the generated app is still additive and can leave behind dead files. |
| CLI build | Support incremental generation so a single changed file does not rewrite the entire showcase app | P2 | 3 | 4 | 3 | This matters once story counts grow and the dev loop slows down. |
| CLI dev loop | Replace polling-based watches with filesystem notifications using `notify` or equivalent | P1 | 4 | 2 | 8 | Polling every 700ms is simple but wasteful and less responsive. |
| CLI dev loop | Manage child process lifecycle and signal forwarding more carefully, especially for Ctrl+C and failure states | P1 | 4 | 3 | 7 | The current thread/process model is workable but not very robust. |
| CLI scaffold | Stop unconditionally rewriting `showcase/Dioxus.toml`, and preserve user-authored files elsewhere in the generated app | P1 | 4 | 3 | 7 | `main.rs` and `Cargo.toml` became write-once in `v0.1.0`, but `Dioxus.toml` is still overwritten on every build, so any hand edit to it is silently lost — and no other user-authored file in the generated app directory is protected. |
| Showcase shell | Memoize the derived tag list and navigation tree in `dioxus-showcase-ui`'s `Sidebar` instead of rebuilding them on every render | P1 | 4 | 2 | 8 | The registry itself is now read once behind a `use_hook`, but `all_tags` and `navigation` still rebuild a `Vec` and a whole tree on every sidebar render, which will scale poorly. |
| Showcase shell | Add search, sort, and keyboard navigation for larger story sets | P2 | 3 | 2 | 5 | The current tree-only navigation will become clumsy quickly. |
| Showcase shell | Add empty/loading states for missing assets and invalid generated metadata | P2 | 3 | 2 | 5 | Story render failures are handled, but missing assets and malformed metadata are not. |
| Example crate | Expand the example to cover props structs, tags, multi-file modules, decorators, slots, and failure cases | P1 | 4 | 2 | 8 | The current example proves the happy path but not the tricky patterns users will copy. |
| Testing | Add UI or browser smoke tests for the generated showcase shell | P1 | 4 | 3 | 7 | Route rendering, tag filtering, and asset loading are currently verified only by hand. |
| Docs and RFC | Sync the RFC, README, and actual API names/features | P1 | 4 | 2 | 8 | The RFC still references `StoryArgs` derive and capabilities that differ from the implemented surface. |
| Docs and RFC | Add architecture diagrams and a "current limitations" page separate from aspirational roadmap text | P2 | 3 | 1 | 6 | This helps contributors understand what is deliberately incomplete versus accidentally missing. |
| CI | Split CI into fast checks and slower integration/browser jobs | P2 | 3 | 2 | 5 | The current workflow is simple but will not scale well as tests grow. |
| CI | Run doctests and package verification per crate rather than only whole-workspace commands | P2 | 3 | 2 | 5 | This gives better signal for crate-level publish health. |
| Publish/release | Add release dry-run checks for generated scaffold contents and example workflows | P2 | 3 | 2 | 5 | Publishing the libraries without validating the generated app path leaves a gap in release confidence. |
| Observability | Add verbose/debug logging modes for discovery, generation, and watch events | P2 | 3 | 1 | 6 | This will matter once users hit path/module issues in nontrivial workspaces. |
| Security/reliability | Harden path handling around recursive copies and generated dependency paths | P2 | 3 | 2 | 5 | Most inputs are local, but path normalization and surprising relative paths are still easy footguns. |

## Suggested Next Wave

The previous wave shipped with `v0.1.0` and has moved to *Already Shipped*. With every `P0`
row now closed, nothing scores above 8 and thirteen rows tie there, so the ranking below
breaks the tie by judgement rather than by arithmetic:

1. Remove duplicated helpers such as `slugify_title` and metadata parsing (score 8) — the
   same logic still exists in the facade, the macros, and the CLI, which is the remaining
   source of ID and discovery drift after the registration rework.
2. Sync the RFC, README, and the actual API names and features (score 8) — the RFC now
   describes a surface that diverges from what shipped, and it is the document a new
   contributor reads first.
3. Add non-interactive `init` flags (score 8) — interactive-only `init` blocks automation
   and scaffolding tools, and it is the first thing a new user runs.
4. Replace polling-based watches with filesystem notifications (score 8) — the dev loop got
   measurably slower when the generated app took on thin LTO, so the 700ms poll is now a
   larger share of a worse round trip.
5. Stop AST rescanning the source tree separately from macro expansion (score 6) — ranked
   above nine other rows that score higher because it is the only one that removes a class
   of drift rather than a symptom. `v0.1.0` decided which side wins, but left both parsers
   in the tree; deciding whether the second one earns its keep is the last piece of that
   question.

The first four break the tie toward whatever unblocks adoption or reduces known drift. Two
rows were re-scored this wave and no others were touched: the AST-rescanning row dropped
from `P0`/9 to `P1`/6 because the registration contract it was half about has shipped, and
the scaffold-ownership row was rewritten around `Dioxus.toml`, which is the part of it that
survives write-once.
