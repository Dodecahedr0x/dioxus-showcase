# `trybuild` compile-fail fixtures

Each `.rs` file here is a program that must **fail** to compile, paired with a
`.stderr` snapshot of exactly what the user sees when it does. The harness that
runs them is `../ui.rs`.

Cargo only auto-discovers test targets at `tests/*.rs`, so nothing in this
directory is a target of its own; `ui.rs` globs them in.

## Running and regenerating

```sh
cargo test -p dioxus-showcase-macros --test ui                      # verify
TRYBUILD=overwrite cargo test -p dioxus-showcase-macros --test ui   # re-snapshot
```

`TRYBUILD=overwrite` rewrites every `.stderr` in place. **Read the diff** before
committing one: the whole point of a snapshot is that a change in wording becomes
visible, so blessing it unread throws the test away.

## Why these run on stable

serde and thiserror gate their compile-fail suites behind
`rustversion::attr(not(nightly), ignore)`. Do **not** copy that here. Those
projects have a nightly CI leg; this repo does not, so the gate would make the
suite silently never run.

Running ungated is safe because every case asserts this project's own
`compile_error!` string rather than a rustc diagnostic, and those do not move
when rustc's wording changes. The only rustc-owned text in each snapshot is the
trailing `= note: this error originates in the attribute macro ...` line. See
decisions.md A15 and V8.

## Scope

These cases cover what a **user** sees, and that is all they are for. Broad
coverage of the macro error branches lives in `#[cfg(test)] mod tests` inside
`src/`, because this crate is `proc-macro = true` with private modules and most
branches are unreachable from an integration test. Add a case here only when the
rendered diagnostic is itself the thing worth pinning; add it to `src/`
otherwise, where it costs no compile.
