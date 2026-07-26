//! The story canvas, and the error boundary that keeps a broken story local.
use dioxus::prelude::*;

use crate::state::Shell;

/// Renders one story inside an error boundary.
///
/// A story is arbitrary user code and is fully entitled to fail. When it does,
/// only this subtree is replaced — the sidebar, the router and every other story
/// keep working, and the failure is retryable without reloading the page.
#[component]
pub(crate) fn StoryCanvas(id: String) -> Element {
    let shell = use_context::<Shell>();

    rsx! {
        section { class: "canvas",
            ErrorBoundary {
                handle_error: |errors: ErrorContext| {
                    rsx! {
                        div { class: "story-surface story-surface-error",
                            h3 { class: "story-error-title", "Story render failed" }
                            p { class: "muted", "The showcase shell is still running. Fix the story and try again." }
                            pre { class: "story-error-details", "{errors:?}" }
                            button {
                                class: "story-error-retry",
                                onclick: move |_| errors.clear_errors(),
                                "Retry story"
                            }
                        }
                    }
                },
                div { class: "story-surface",
                    StorySurface { shell, id }
                }
            }
        }
    }
}

/// Invokes one story's render function.
///
/// This is a component rather than an inline expression on purpose: the render
/// call has to happen *inside* a scope below [`StoryCanvas`]'s `ErrorBoundary`,
/// so that a story returning `Err` is caught by that boundary instead of
/// propagating out of the scope that built the boundary's children.
#[component]
fn StorySurface(shell: Shell, id: String) -> Element {
    match shell.story(&id) {
        Some(story) => (story.render)(),
        // Unreachable through the router, which only renders this for an id it
        // already resolved. Rendering a note beats unwrapping.
        None => rsx! {
            p { class: "muted", "Story '{id}' is no longer registered." }
        },
    }
}
