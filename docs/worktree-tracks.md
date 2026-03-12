# Worktree Tracks

Independent worktrees created from `main` at commit `f0d7760`.

| Track | Branch | Worktree path | Improvement focus |
| --- | --- | --- | --- |
| 1 | `improve-serde-config-manifest` | `/tmp/dioxus-preview-serde-config-manifest` | Replace custom TOML/JSON handling with `serde`, `toml`, and `serde_json` |
| 2 | `improve-discovery-module-graph` | `/tmp/dioxus-preview-discovery-module-graph` | Fix discovery correctness for external modules and reduce drift with macro behavior |
| 3 | `improve-basepath-shell` | `/tmp/dioxus-preview-basepath-shell` | Honor `build.base_path`, add shell hardening, and stop destructive scaffold overwrites |
| 4 | `improve-e2e-golden-tests` | `/tmp/dioxus-preview-e2e-golden-tests` | Add end-to-end CLI integration tests and golden output coverage |
| 5 | `improve-release-determinism` | `/tmp/dioxus-preview-release-determinism` | Improve deterministic generation and release/publish robustness |

Notes:

- The primary worktree at `/Users/dode/Documents/rust/dioxus-preview` remains on `main`.
- Existing uncommitted changes in the primary worktree were left untouched.
- Each worktree was branched from the same `main` commit so they can be developed independently and merged later.
