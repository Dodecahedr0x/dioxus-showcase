//! The shell's routes and the page each one renders.
//!
//! The route table is deliberately identical to the one the Handlebars template
//! generated, so existing links and the exported `404.html` fallback keep
//! working: `/`, `/component/:id`, and a catch-all.
//!
//! Note that the routes are **not** prefixed with the base path. Under `dx`,
//! `WebHistory` reads the base path from the CLI config and prefixes every
//! rendered link itself; adding it here as well would produce `/repo/repo/...`.
use dioxus::prelude::*;

use crate::canvas::StoryCanvas;
use crate::diagnostics::NoStoriesRegistered;
use crate::filters::{SelectedTag, StoryTags};
use crate::sidebar::Sidebar;
use crate::state::Shell;

#[derive(Routable, Clone, PartialEq, Debug)]
pub(crate) enum Route {
    #[route("/")]
    Home {},
    #[route("/component/:id")]
    Component { id: String },
    #[route("/:..route")]
    NotFound { route: Vec<String> },
}

/// Landing page shown before any story has been selected.
#[component]
pub(crate) fn Home() -> Element {
    let shell = use_context::<Shell>();

    rsx! {
        div { class: "shell",
            Sidebar { active_id: None::<String> }
            main { class: "content",
                if shell.is_empty() {
                    NoStoriesRegistered {}
                } else {
                    h2 { "Select a component" }
                    p { class: "muted", "Browse the title tree on the left and open any story route." }
                }
            }
        }
    }
}

/// Story page for one route id.
#[component]
pub(crate) fn Component(id: String) -> Element {
    let shell = use_context::<Shell>();
    // Consumed so the page fails loudly here rather than deep inside `StoryTags`
    // if the shell root ever stops providing it.
    let _selected_tag = use_context::<SelectedTag>();

    let selected = shell.story(&id).map(|story| story.definition.clone());
    // The public URL of this page, which is the one place the shell has to apply
    // the base path itself — nothing prefixes displayed text for us.
    let public_route = shell.base_path().join(&format!("/component/{id}"));

    rsx! {
        div { class: "shell",
            Sidebar { active_id: Some(id.clone()) }
            main { class: "content",
                if let Some(definition) = selected {
                    h2 { "{definition.title}" }
                    p { class: "muted", "Route: {public_route}" }
                    StoryTags { tags: definition.tags.clone() }
                    StoryCanvas { id }
                } else if shell.is_empty() {
                    NoStoriesRegistered {}
                } else {
                    h2 { "Component not found" }
                    p { class: "muted", "No annotated component matched route id '{id}'." }
                }
            }
        }
    }
}

/// Catch-all page for paths that match no route.
#[component]
pub(crate) fn NotFound(route: Vec<String>) -> Element {
    let shell = use_context::<Shell>();
    let attempted = shell.base_path().join(&route.join("/"));

    rsx! {
        div { class: "shell",
            main { class: "content",
                h2 { "Page not found" }
                p { class: "muted", "Route '{attempted}' does not exist." }
                Link { to: Route::Home {}, class: "back-link", "Back to home" }
            }
        }
    }
}
