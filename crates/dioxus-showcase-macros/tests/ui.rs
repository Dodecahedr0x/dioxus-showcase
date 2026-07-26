//! Compile-fail coverage for the macro error paths.
//!
//! Deliberately **not** nightly-gated. serde and thiserror wrap their suites in
//! `rustversion::attr(not(nightly), ignore)`, but they have a nightly CI leg and
//! this repo does not — copying that gate here would mean the suite never runs
//! at all. It is safe to run on stable because every case below asserts this
//! project's own `compile_error!` text rather than a rustc diagnostic, so
//! nothing here moves when rustc's wording does. See decisions.md A15 and V8.
//!
//! Broader error-path coverage lives in `#[cfg(test)] mod tests` inside `src/`:
//! this crate is `proc-macro = true` with private modules, so most branches are
//! only reachable from within the crate. These cases exist for what the user
//! actually sees.
//!
//! Regenerate the snapshots with:
//!
//! ```text
//! TRYBUILD=overwrite cargo test -p dioxus-showcase-macros --test ui
//! ```

#[test]
fn compile_fail_cases_render_the_expected_errors() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/*.rs");
}
