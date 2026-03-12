# Dioxus Showcase Improvement Inventory

This document surveys the current workspace and lists improvement ideas across every major part of the project.

Scoring model:

- `Priority`: `P0` critical foundation, `P1` high leverage, `P2` worthwhile, `P3` nice-to-have
- `Impact`: `1-5`, where `5` materially changes correctness, adoption, or maintainability
- `Difficulty`: `1-5`, where `5` is a large cross-cutting effort
- `Global score = priority weight + impact - difficulty`
- Priority weights: `P0 = 8`, `P1 = 6`, `P2 = 4`, `P3 = 2`

Higher score means better candidate for near-term planning.

| Area | Improvement idea | Priority | Impact | Difficulty | Global score | Why it matters |
| --- | --- | --- | --- | --- | --- | --- |
| Workspace architecture | Replace stringly-typed config, manifest, and template data with `serde`-based shared schemas used by all crates | P0 | 5 | 3 | 10 | The codebase hand-rolls TOML and JSON in several places, which is already duplicated and will drift as the schema expands. |
| Workspace architecture | Remove duplicated helpers such as `slugify_title` and metadata parsing by centralizing them in `core` or a dedicated shared crate module | P1 | 4 | 2 | 8 | The same logic exists in the facade, macros, and CLI, increasing the chance of inconsistent IDs and discovery behavior. |
| Workspace architecture | Define a stable registration contract so the CLI/runtime stops depending on generated symbol naming conventions | P0 | 5 | 4 | 9 | Current discovery and runtime assembly are tightly coupled to macro-generated `__dioxus_showcase_*` symbols and include-style glue. |
| Workspace architecture | Add explicit MSRV, compatibility matrix, and release policy in crate metadata and docs | P2 | 3 | 1 | 6 | The RFC promises compatibility management, but the workspace does not yet declare it. |
| Workspace architecture | Introduce benchmark targets for discovery time, manifest generation, and showcase startup | P2 | 3 | 2 | 5 | The RFC has performance targets, but nothing currently measures them. |
| `dioxus-showcase-core` config | Replace the custom TOML parser/serializer with `toml` + `serde` and validate unknown keys instead of silently ignoring them | P0 | 5 | 2 | 11 | The current parser is intentionally minimal but fragile; it will break on escaped strings, inline comments, richer TOML, and future schema growth. |
| `dioxus-showcase-core` config | Add semantic validation for paths, host/base path normalization, and invalid combinations | P1 | 4 | 2 | 8 | Parsing currently succeeds even when the config is semantically unusable. |
| `dioxus-showcase-core` config | Support layered config sources: file, env vars, and CLI overrides | P2 | 3 | 3 | 4 | This becomes important once the tool is used in CI and multiple environments. |
| `dioxus-showcase-core` manifest | Replace manual JSON generation with `serde_json` and add round-trip tests | P0 | 5 | 1 | 12 | Manual escaping is easy to get subtly wrong and becomes brittle the moment the schema gains nested or optional fields. |
| `dioxus-showcase-core` manifest | Add manifest schema evolution support with versioned structs and compatibility tests | P1 | 4 | 3 | 7 | The RFC calls for a stable schema, but the code has only a hard-coded `schema_version = 1`. |
| `dioxus-showcase-core` runtime | Add deterministic sorting for registry output and navigation tree children | P1 | 4 | 2 | 8 | Current ordering depends on discovery order, which weakens CI determinism and makes UI ordering less predictable. |
| `dioxus-showcase-core` runtime | Add duplicate-title and malformed-title validation helpers, not just duplicate IDs | P2 | 3 | 2 | 5 | Navigation and UX degrade when titles are empty, inconsistent, or collide in confusing ways. |
| `dioxus-showcase-core` runtime | Add richer story metadata in shared types: description, docs URL, decorators, viewport/background settings, arg schema | P1 | 5 | 4 | 7 | Most roadmap features need shared data structures before they can be built cleanly. |
| `dioxus-showcase-macros` overall | Split metadata extraction from UI/control rendering so proc macros do less runtime UI work | P1 | 4 | 4 | 6 | The macros currently own authoring API, runtime wiring, and part of the preview UI, which is a high-coupling design. |
| `dioxus-showcase-macros` overall | Replace `compile_error!(format!("{:?}", err))` style diagnostics with proper `syn::Error` spans | P1 | 4 | 2 | 8 | Error reporting quality is one of the biggest adoption multipliers for proc-macro crates. |
| `dioxus-showcase-macros` overall | Add `trybuild`-style compile-fail tests for invalid attribute arguments and unsupported signatures | P0 | 5 | 2 | 11 | The macro test suite mostly checks happy paths; the most important proc-macro regressions are error-path regressions. |
| `#[showcase]` macro | Support explicit story IDs and collision-resistant defaults based on module path plus title | P1 | 4 | 3 | 7 | Title-only slugging can collide easily in real component libraries. |
| `#[showcase]` macro | Support decorators, docs text, and non-default named variants without requiring custom props impl boilerplate | P2 | 4 | 3 | 5 | This would move the API closer to the RFC’s addon and docs ambitions. |
| `#[showcase]` macro | Stop assuming the first argument named `props` is the only aggregate-props form worth special-casing | P1 | 4 | 2 | 8 | The current rule is narrow and will surprise users with differently named props bindings. |
| `#[story]` macro | Validate return types and story signatures more explicitly, including better messages for unsupported patterns | P1 | 4 | 2 | 8 | The current API accepts a narrow set of shapes but does not make those constraints obvious. |
| `#[story]` macro | Add arg-schema metadata generation rather than only live control widgets | P1 | 5 | 4 | 7 | Without a serializable arg schema, docs, static export, testing, and addons remain limited. |
| `StoryProps` derive | Generate field-level defaults or custom variant hooks instead of only wrapping `Default::default()` | P2 | 3 | 3 | 4 | Today the derive is convenient but too shallow for realistic component props. |
| Macro utilities | Expand control inference beyond `String`, `bool`, and numeric primitives | P1 | 4 | 3 | 7 | Control generation is one of the main value propositions, and it currently covers only the easiest scalar cases. |
| Macro utilities | Add escape hatches for custom controls and hidden args | P2 | 4 | 3 | 5 | Real component APIs often contain callbacks, slots, IDs, or advanced types that should not become default controls. |
| `dioxus-showcase` facade | Reduce dependency surface and re-export only what is required by generated code and user ergonomics | P2 | 3 | 2 | 5 | The facade currently pulls in multiple Dioxus crates and mixes runtime, macros, and authoring concerns. |
| `dioxus-showcase` facade | Add trait docs and examples for `StoryArg`, `StoryArgs`, `StoryProps`, and `ShowcaseStoryFactory` | P2 | 3 | 1 | 6 | The public API surface is small but under-documented relative to its importance. |
| `dioxus-showcase` facade | Add built-in impls for common standard library/container types that users will hit quickly | P2 | 3 | 2 | 5 | Current defaults are enough for demos but not for realistic props trees. |
| CLI UX | Add machine-readable output modes such as JSON for `check`, `build`, and `doctor` | P1 | 4 | 2 | 8 | CI and editor integrations become much easier once commands can emit structured diagnostics. |
| CLI UX | Add non-interactive `init` flags and avoid always prompting | P1 | 4 | 2 | 8 | Interactive-only init is awkward for automation and scaffolding tools. |
| CLI UX | Add clearer error categorization with actionable hints per failure mode | P1 | 4 | 2 | 8 | Most command errors are currently plain strings with little recovery guidance. |
| CLI discovery | Stop AST rescanning the source tree separately from macro expansion and move toward explicit registration artifacts | P0 | 5 | 4 | 9 | Discovery currently duplicates macro parsing rules and will continue to drift from the proc-macro behavior. |
| CLI discovery | Handle external modules declared with `mod foo;` by following the module graph instead of only inline modules and raw file recursion | P0 | 5 | 3 | 10 | The current implementation misses a common Rust module pattern, which directly limits correctness. |
| CLI discovery | Respect `.gitignore`/workspace exclusions and make ignore rules configurable | P2 | 3 | 2 | 5 | Recursive scans will become noisy and slow in larger workspaces. |
| CLI discovery | Detect and report invalid combinations such as duplicate titles, broken module paths, missing story symbols, and non-component functions | P1 | 4 | 3 | 7 | `check` should be the authoritative validation pass before generation. |
| CLI build | Make generated artifacts deterministic by removing timestamp-based generation values from output | P1 | 4 | 1 | 9 | The current generation nonce bakes wall-clock time into generated code, which harms stable diffs and caching. |
| CLI build | Support incremental generation so a single changed file does not rewrite the entire showcase app | P2 | 3 | 4 | 3 | This matters once story counts grow and the dev loop slows down. |
| CLI build | Add a strict `--clean` or output-pruning mode for stale generated files and copied assets | P2 | 3 | 2 | 5 | Asset syncing is additive and can leave behind dead files. |
| CLI dev loop | Replace polling-based watches with filesystem notifications using `notify` or equivalent | P1 | 4 | 2 | 8 | Polling every 700ms is simple but wasteful and less responsive. |
| CLI dev loop | Manage child process lifecycle and signal forwarding more carefully, especially for Ctrl+C and failure states | P1 | 4 | 3 | 7 | The current thread/process model is workable but not very robust. |
| CLI dev loop | Detect whether `dx` is installed before launch and expose richer doctor checks | P2 | 3 | 1 | 6 | This is low effort and reduces first-run confusion. |
| CLI scaffold | Stop overwriting `showcase/src/main.rs` on every build, or split generated shell from user-editable shell extensions | P0 | 5 | 3 | 10 | Current regeneration makes the scaffold hard to customize safely. |
| CLI scaffold | Preserve user-authored assets/custom files and support partial template upgrades | P1 | 4 | 3 | 7 | The scaffold is currently “owned” by the generator, which is acceptable for a prototype but poor for adoption. |
| CLI scaffold | Add template versioning and migration support | P2 | 3 | 3 | 4 | Once generated apps exist in the wild, upgrades need a supported story. |
| Showcase app template | Move the large inline CSS and shell UI into template assets/components with tests | P2 | 3 | 2 | 5 | The generated `main.rs` is doing too much and will get harder to evolve. |
| Showcase app template | Cache or memoize `showcase_components()`/derived tag data instead of rebuilding on every render path | P1 | 4 | 2 | 8 | The current template repeatedly rebuilds vectors and trees, which will scale poorly. |
| Showcase app template | Add error boundaries around story rendering so one broken story does not break the whole shell | P0 | 5 | 3 | 10 | The RFC explicitly calls this out as a reliability target, and the template does not provide it yet. |
| Showcase app template | Honor `build.base_path` in routes and asset URLs | P0 | 5 | 2 | 11 | The config exposes `base_path`, but the generated web app and asset paths are still hard-coded to root. |
| Showcase app template | Add search, sort, and keyboard navigation for larger story sets | P2 | 3 | 2 | 5 | The current tree-only navigation will become clumsy quickly. |
| Showcase app template | Add empty/loading/error states for missing assets and invalid generated metadata | P2 | 3 | 2 | 5 | The shell assumes everything was generated correctly. |
| Example crate | Expand the example to cover props structs, tags, multi-file modules, decorators, slots, and failure cases | P1 | 4 | 2 | 8 | The current example proves the happy path but not the tricky patterns users will copy. |
| Example crate | Add a real example showcase app snapshot or smoke test | P2 | 3 | 2 | 5 | This would verify the generated app end to end, not just the macros. |
| Docs and RFC | Sync the RFC, README, and actual API names/features | P1 | 4 | 2 | 8 | The RFC still references `StoryArgs` derive and capabilities that differ from the implemented surface. |
| Docs and RFC | Add architecture diagrams and a “current limitations” page separate from aspirational roadmap text | P2 | 3 | 1 | 6 | This helps contributors understand what is deliberately incomplete versus accidentally missing. |
| Docs and RFC | Add a contributor guide covering crate boundaries, code generation flow, and test strategy | P2 | 3 | 1 | 6 | The workspace is small now, which is the best time to codify its design. |
| Testing | Add integration tests that run `init`, `check`, and `build` against fixture workspaces on disk | P0 | 5 | 3 | 10 | Most risk sits in cross-crate orchestration, not in isolated units. |
| Testing | Add golden-file tests for generated `main.rs`, `generated.rs`, manifest JSON, and scaffold output | P1 | 4 | 2 | 8 | This would catch accidental generation diffs immediately. |
| Testing | Add UI or browser smoke tests for the generated showcase shell | P1 | 4 | 3 | 7 | Route rendering, tag filtering, and asset loading are currently unverified at the browser level. |
| CI | Split CI into fast checks and slower integration/browser jobs | P2 | 3 | 2 | 5 | The current single-job workflow is simple but will not scale well as tests grow. |
| CI | Run doctests and package verification per crate rather than only whole-workspace commands | P2 | 3 | 2 | 5 | This gives better signal for crate-level publish health. |
| Publish/release | Publish crates in dependency order instead of `cargo publish --workspace` | P1 | 4 | 2 | 8 | Workspace publish can fail on crates.io due to propagation timing of newly published internal dependencies. |
| Publish/release | Version and release all internal crates consistently, including the CLI crate, in the sync script | P1 | 4 | 2 | 8 | The current version sync script only updates some internal workspace dependencies. |
| Publish/release | Add release dry-run checks for generated scaffold contents and example workflows | P2 | 3 | 2 | 5 | Publishing the libraries without validating the generated app path leaves a gap in release confidence. |
| Observability | Add verbose/debug logging modes for discovery, generation, and watch events | P2 | 3 | 1 | 6 | This will matter once users hit path/module issues in nontrivial workspaces. |
| Security/reliability | Harden path handling around recursive copies and generated dependency paths | P2 | 3 | 2 | 5 | Most inputs are local, but path normalization and surprising relative paths are still easy footguns. |
| Product direction | Decide whether the long-term source of truth is AST discovery, macro registration, or generated manifests, and align every crate around that choice | P0 | 5 | 4 | 9 | This is the highest-leverage strategic decision in the repo because several current rough edges come from mixed models. |

## Suggested first wave

If this were prioritized into a first implementation wave, the strongest candidates are:

1. Replace custom TOML/JSON handling with `serde` + ecosystem parsers.
2. Fix discovery correctness for external module layouts and unify discovery rules with macro behavior.
3. Honor `base_path`, add error boundaries, and stop overwriting user-editable `showcase/src/main.rs`.
4. Add integration and golden tests for the end-to-end CLI generation flow.
5. Make publish/versioning more robust and deterministic.
