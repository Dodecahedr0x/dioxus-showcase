//! Shell UI for `dioxus-showcase`.
//!
//! This crate owns the showcase application shell — routing, story tree
//! navigation, tag filtering, the theme toggle and the story error boundary — as
//! compiled Dioxus components rather than as source generated into the user's
//! crate.
//!
//! The whole public surface is [`ShowcaseApp`]. It takes no story list: stories
//! and providers register themselves at link time and are read from the registry
//! by the shell itself, so a generated showcase entry point is a fixed ten lines
//! that never needs regenerating:
//!
//! ```rust,ignore
//! use dioxus::prelude::*;
//! use my_component_crate as _; // LOAD-BEARING: keeps the registrations linked
//!
//! fn main() {
//!     launch(App);
//! }
//!
//! #[component]
//! fn App() -> Element {
//!     rsx! { dioxus_showcase_ui::ShowcaseApp { base_path: "/" } }
//! }
//! ```
//!
//! ## Failure states
//!
//! The shell renders, without panicking, for every way a showcase can be
//! misconfigured. An empty registry gets an empty state naming the linkage line
//! that is usually missing; colliding story ids get a banner rather than an
//! `assert!`; and a story whose render fails is contained by an error boundary
//! that leaves the rest of the application running.

mod base_path;
mod canvas;
mod diagnostics;
mod filters;
mod nav;
mod routes;
mod shell;
mod sidebar;
mod state;
mod theme;

#[cfg(test)]
mod testing;

#[cfg(test)]
mod tests;

pub use shell::{ShowcaseApp, ShowcaseAppProps};
