//! Visible states for the two ways a showcase can be misconfigured.
//!
//! Neither of these is an error the shell can recover from, and neither is a
//! reason to stop: a panic here would replace a diagnosable problem with a blank
//! page and a console trace.
use dioxus::prelude::*;

/// Banner shown when more than one story claims the same route id.
///
/// Registration reports collisions instead of asserting on them, precisely so
/// this can exist. Every colliding story is still registered and still reachable
/// through the tree; what is ambiguous is only which one `/component/<id>`
/// resolves to.
#[component]
pub(crate) fn DuplicateStoryIds(ids: Vec<String>) -> Element {
    if ids.is_empty() {
        return rsx! {};
    }

    rsx! {
        div { class: "duplicate-ids-banner",
            h2 { class: "duplicate-ids-title", "Duplicate story ids" }
            p { class: "muted",
                "More than one story claims each of these route ids, so "
                "/component/<id> resolves to whichever one sorts first. Rename or "
                "re-title the others to make each id unique."
            }
            ul { class: "duplicate-ids-list",
                for id in ids {
                    li { key: "{id}", class: "duplicate-id", "{id}" }
                }
            }
        }
    }
}

/// Empty state shown when the registry contains no stories at all.
///
/// Stories register themselves at link time from the crate that defines them,
/// and there are **two independent ways** the linker discards those
/// registrations while the build still succeeds — leaving an empty registry with
/// no error anywhere:
///
/// 1. The defining crate is never referenced, so it is dropped wholesale. The
///    generated `main.rs` prevents this with a `use <crate> as _;` line that
///    looks exactly like a stray unused import.
/// 2. LTO is off. Without it the wasm32 linker never selects the component
///    crate's archive member and every `inventory` registration goes with it
///    (V13). New projects are pinned to `[profile.dev] lto = "thin"` and
///    `[profile.release] lto = true`, but the generated `Cargo.toml` is
///    write-once, so a showcase upgraded from 0.0.7 never gains those lines.
///
/// Both are silent and neither is distinguishable from the benign third case —
/// a new project with nothing annotated yet. This is the last line of defense
/// against a blank page, so it names all three.
#[component]
pub(crate) fn NoStoriesRegistered() -> Element {
    rsx! {
        div { class: "empty-state",
            h2 { "No stories are registered" }
            p { class: "muted",
                "The showcase started, but nothing registered itself. Stories "
                "register at link time, so this is almost always one of three "
                "things — the first two fail silently, with no build error."
            }
            ol { class: "empty-state-causes",
                li { class: "muted",
                    strong { "The entry crate is not linked." }
                    " The showcase's "
                    code { "src/main.rs" }
                    " has to keep the line "
                    code { class: "empty-state-code", "use <your crate> as _;" }
                    ". It looks like an unused import, but without it the linker "
                    "drops the crate and every registration inside it."
                }
                li { class: "muted",
                    strong { "LTO is disabled." }
                    " On wasm32 the linker only keeps those registrations when "
                    "link-time optimization is on, so "
                    code { "showcase/Cargo.toml" }
                    " needs both of these:"
                    pre { class: "empty-state-block",
                        code { "[profile.dev]\nlto = \"thin\"\n\n[profile.release]\nlto = true" }
                    }
                    "This is the usual cause after upgrading from 0.0.7 — that "
                    "file is written once, when the showcase is created, so an "
                    "existing one never gains the lines on its own."
                }
                li { class: "muted",
                    strong { "Nothing is annotated yet." }
                    " Expected on a new project: mark a component with "
                    code { "#[showcase]" }
                    " or "
                    code { "#[story]" }
                    " and rebuild."
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_duplicate_ids_renders_nothing_at_all() {
        let html = dioxus_ssr::render_element(rsx! { DuplicateStoryIds { ids: Vec::new() } });

        assert_eq!(html, "");
    }

    #[test]
    fn duplicate_ids_are_each_listed_by_name() {
        let ids = vec!["atoms-button".to_owned(), "layout-grid".to_owned()];

        let html = dioxus_ssr::render_element(rsx! { DuplicateStoryIds { ids } });

        assert!(html.contains("Duplicate story ids"), "{html}");
        assert!(html.contains("atoms-button"), "{html}");
        assert!(html.contains("layout-grid"), "{html}");
    }

    #[test]
    fn the_empty_state_names_the_linkage_line_that_is_usually_missing() {
        let html = dioxus_ssr::render_element(rsx! { NoStoriesRegistered {} });

        assert!(html.contains("No stories are registered"), "{html}");
        // The angle brackets in the placeholder are HTML-escaped on the way out.
        assert!(html.contains("use &#60;your crate&#62; as _;"), "{html}");
        assert!(html.contains("<code>src/main.rs</code>"), "{html}");
    }

    /// The second silent cause (V13), and the one upgraders hit: without LTO the
    /// wasm32 linker drops every registration and the page is blank with no error.
    #[test]
    fn the_empty_state_names_the_lto_settings_that_upgraders_are_missing() {
        let html = dioxus_ssr::render_element(rsx! { NoStoriesRegistered {} });

        assert!(html.contains("showcase/Cargo.toml"), "{html}");
        assert!(html.contains("[profile.dev]"), "{html}");
        // Quotes are escaped numerically on the way out, like the angle brackets.
        assert!(html.contains("lto = &#34;thin&#34;"), "{html}");
        assert!(html.contains("[profile.release]"), "{html}");
        assert!(html.contains("lto = true"), "{html}");
    }

    /// A showcase upgraded from 0.0.7 has a write-once `Cargo.toml` that never
    /// gained the profile section, so the empty state has to say so by name.
    #[test]
    fn the_empty_state_calls_out_upgrades_from_the_previous_release() {
        let html = dioxus_ssr::render_element(rsx! { NoStoriesRegistered {} });

        assert!(html.contains("0.0.7"), "{html}");
    }

    /// The benign case stays reachable: a brand new user who has annotated
    /// nothing yet must not be told their build is broken.
    #[test]
    fn the_empty_state_keeps_the_benign_no_components_yet_case_reachable() {
        let html = dioxus_ssr::render_element(rsx! { NoStoriesRegistered {} });

        assert!(html.contains("#[showcase]"), "{html}");
        assert!(html.contains("#[story]"), "{html}");
    }
}
