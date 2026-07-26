//! Whole-shell tests.
//!
//! Each module's own behaviour is unit-tested beside it; this module covers the
//! shell as one assembled thing, which is where routing, contexts and the error
//! boundary only exist together. Assertions are on rendered HTML, which is what
//! the Dioxus testing guide recommends — there is no matcher crate.
use std::rc::Rc;

use dioxus::history::{History, MemoryHistory};
use dioxus::prelude::*;
use dioxus_showcase::{GeneratedStory, StoryPreviewContent};

use crate::testing::{
    definition, dom_at, failing_story, render_at, render_at_with_prefix, story, update,
    ShellBuilder,
};
use crate::theme::ThemeMode;
use crate::{ShowcaseApp, ShowcaseAppProps};

// --- Empty registry (A22: the linkage footgun explains itself) ---

#[test]
fn an_empty_registry_renders_the_empty_state_instead_of_a_blank_page() {
    let html = render_at(ShellBuilder::new().build(), "/");

    assert!(html.contains("No stories are registered"), "{html}");
    assert!(html.contains("as _;"), "{html}");
    assert!(html.contains("src/main.rs"), "{html}");
}

#[test]
fn an_empty_registry_still_renders_the_empty_state_on_a_story_route() {
    let html = render_at(ShellBuilder::new().build(), "/component/anything");

    assert!(html.contains("No stories are registered"), "{html}");
}

#[test]
fn an_empty_registry_still_renders_the_sidebar_and_its_controls() {
    let html = render_at(ShellBuilder::new().title("Design System").build(), "/");

    assert!(html.contains("Design System"), "{html}");
    assert!(html.contains("theme-toggle"), "{html}");
    assert!(html.contains("No tags available"), "{html}");
}

/// Exercises the real public entry point, registry and all.
///
/// Nothing submits a registration in this crate's test binary, so the global
/// registry is genuinely empty here — which makes this both the empty-state case
/// and the proof that [`ShowcaseApp`] wires its contexts correctly on its own.
#[test]
fn the_public_app_renders_against_the_real_registry_without_panicking() {
    let props = ShowcaseAppProps { base_path: "/".to_owned(), title: None };
    let mut dom = VirtualDom::new_with_props(ShowcaseApp, props)
        .with_root_context(Rc::new(MemoryHistory::default()) as Rc<dyn History>);
    dom.rebuild_in_place();

    let html = dioxus_ssr::render(&dom);

    assert!(html.contains("app-shell"), "{html}");
    // The default title, applied because `title: None` was passed.
    assert!(html.contains("Showcase"), "{html}");
    assert!(html.contains("No stories are registered"), "{html}");
}

/// The generated `main.rs` passes `project.name` straight through, and nothing in the
/// config layer forces that to be non-empty. A blank or whitespace-only name must fall
/// back to the default rather than render an empty heading.
#[test]
fn a_blank_title_falls_back_to_the_default_heading() {
    for blank in ["", "   "] {
        let props = ShowcaseAppProps { base_path: "/".to_owned(), title: Some(blank.to_owned()) };
        let mut dom = VirtualDom::new_with_props(ShowcaseApp, props)
            .with_root_context(Rc::new(MemoryHistory::default()) as Rc<dyn History>);
        dom.rebuild_in_place();

        let html = dioxus_ssr::render(&dom);

        assert!(html.contains("Showcase"), "blank title {blank:?} lost the heading: {html}");
    }
}

/// A package name reaches the heading through the public prop, which is the path the
/// generated entry point uses.
#[test]
fn the_public_app_titles_itself_after_the_package() {
    let props = ShowcaseAppProps { base_path: "/".to_owned(), title: Some("acme-ui".to_owned()) };
    let mut dom = VirtualDom::new_with_props(ShowcaseApp, props)
        .with_root_context(Rc::new(MemoryHistory::default()) as Rc<dyn History>);
    dom.rebuild_in_place();

    let html = dioxus_ssr::render(&dom);

    assert!(html.contains("acme-ui"), "{html}");
}

// --- Duplicate ids (A19: reported, never panicked on) ---

#[test]
fn duplicate_ids_render_a_visible_error_state_rather_than_panicking() {
    let shell = ShellBuilder::new()
        .story(story("dup", "First/Copy", &[]))
        .story(story("dup", "Second/Copy", &[]))
        .duplicate_id("dup")
        .build();

    let html = render_at(shell, "/");

    assert!(html.contains("Duplicate story ids"), "{html}");
    assert!(html.contains(">dup</li>"), "{html}");
}

#[test]
fn duplicate_ids_do_not_stop_the_colliding_stories_being_navigable() {
    let shell = ShellBuilder::new()
        .story(story("dup", "First/Copy", &[]))
        .story(story("dup", "Second/Copy", &[]))
        .duplicate_id("dup")
        .build();

    let html = render_at(shell, "/component/dup");

    // Both titles are still in the tree, and the route resolved to the first.
    assert!(html.contains("First"), "{html}");
    assert!(html.contains("Second"), "{html}");
    assert!(html.contains("story-body-dup"), "{html}");
}

#[test]
fn a_clean_registry_shows_no_duplicate_banner() {
    let shell = ShellBuilder::new().story(story("a", "Atoms/A", &[])).build();

    assert!(!render_at(shell, "/").contains("Duplicate story ids"));
}

// --- Story failures stay local ---

#[test]
fn a_failing_story_is_contained_by_the_error_boundary() {
    let shell = ShellBuilder::new().story(failing_story("boom", "Broken/Boom")).build();

    let html = render_at(shell, "/component/boom");

    assert!(html.contains("Story render failed"), "{html}");
    assert!(html.contains("story exploded on purpose"), "{html}");
}

#[test]
fn a_failing_story_leaves_the_rest_of_the_shell_running() {
    let shell = ShellBuilder::new()
        .story(failing_story("boom", "Broken/Boom"))
        .story(story("fine", "Working/Fine", &[]))
        .build();

    let html = render_at(shell, "/component/boom");

    // The sidebar, the theme control and the sibling story's link all survive.
    assert!(html.contains("Story render failed"), "{html}");
    assert!(html.contains("theme-toggle"), "{html}");
    assert!(html.contains("/component/fine"), "{html}");
}

#[test]
fn a_healthy_sibling_route_is_unaffected_by_a_broken_story() {
    let shell = ShellBuilder::new()
        .story(failing_story("boom", "Broken/Boom"))
        .story(story("fine", "Working/Fine", &[]))
        .build();

    let html = render_at(shell, "/component/fine");

    assert!(html.contains("story-body-fine"), "{html}");
    assert!(!html.contains("Story render failed"), "{html}");
}

// --- Routing ---

#[test]
fn a_story_route_renders_that_story_and_its_title() {
    let shell = ShellBuilder::new().story(story("atoms-button", "Atoms/Button", &[])).build();

    let html = render_at(shell, "/component/atoms-button");

    assert!(html.contains("story-body-atoms-button"), "{html}");
    assert!(html.contains("Atoms/Button"), "{html}");
}

#[test]
fn an_unknown_story_route_reports_a_miss_without_failing() {
    let shell = ShellBuilder::new().story(story("atoms-button", "Atoms/Button", &[])).build();

    let html = render_at(shell, "/component/nope");

    assert!(html.contains("Component not found"), "{html}");
    assert!(html.contains("route id &#39;nope&#39;"), "{html}");
}

#[test]
fn an_unmatched_path_falls_through_to_the_catch_all() {
    let shell = ShellBuilder::new().story(story("a", "Atoms/A", &[])).build();

    let html = render_at(shell, "/definitely/not/a/route");

    assert!(html.contains("Page not found"), "{html}");
    assert!(html.contains("Back to home"), "{html}");
}

#[test]
fn the_tree_links_to_every_registered_story() {
    let shell = ShellBuilder::new()
        .story(story("atoms-button", "Atoms/Button", &[]))
        .story(story("layout-grid", "Layout/Grid", &[]))
        .build();

    let html = render_at(shell, "/");

    assert!(html.contains("/component/atoms-button"), "{html}");
    assert!(html.contains("/component/layout-grid"), "{html}");
}

// --- base_path, at the root and under a sub-path ---

#[test]
fn at_the_root_the_displayed_route_carries_no_prefix() {
    let shell = ShellBuilder::new()
        .base_path("/")
        .story(story("atoms-button", "Atoms/Button", &[]))
        .build();

    let html = render_at(shell, "/component/atoms-button");

    assert!(html.contains("Route: /component/atoms-button"), "{html}");
    assert!(!html.contains("//component"), "{html}");
}

#[test]
fn under_a_sub_path_the_displayed_route_is_prefixed_exactly_once() {
    let shell = ShellBuilder::new()
        .base_path("/my-repo")
        .story(story("atoms-button", "Atoms/Button", &[]))
        .build();

    let html = render_at(shell, "/component/atoms-button");

    assert!(html.contains("Route: /my-repo/component/atoms-button"), "{html}");
    assert!(!html.contains("/my-repo/my-repo"), "{html}");
}

#[test]
fn the_not_found_page_reports_the_attempted_path_under_the_base_path() {
    let root = render_at(ShellBuilder::new().build(), "/missing/page");
    assert!(root.contains("Route &#39;/missing/page&#39;"), "{root}");

    let nested = render_at(ShellBuilder::new().base_path("/my-repo").build(), "/missing/page");
    assert!(nested.contains("Route &#39;/my-repo/missing/page&#39;"), "{nested}");
}

/// Navigation links are prefixed by the history, not by the shell.
///
/// Under `dx` the prefix comes from the CLI config via `WebHistory`; here it is
/// supplied by a `MemoryHistory` prefix. Either way the shell must not add it a
/// second time, and this pins that: the href gains exactly one `/my-repo`.
#[test]
fn navigation_links_are_prefixed_once_by_the_history() {
    let shell = ShellBuilder::new()
        .base_path("/my-repo")
        .story(story("atoms-button", "Atoms/Button", &[]))
        .build();

    let html = render_at_with_prefix(shell, "/component/atoms-button", "/my-repo");

    assert!(html.contains("href=\"/my-repo/component/atoms-button\""), "{html}");
    assert!(!html.contains("/my-repo/my-repo"), "{html}");
}

#[test]
fn without_a_history_prefix_navigation_links_stay_unprefixed() {
    let shell = ShellBuilder::new().story(story("atoms-button", "Atoms/Button", &[])).build();

    let html = render_at(shell, "/");

    assert!(html.contains("href=\"/component/atoms-button\""), "{html}");
}

// --- Tag filtering ---

#[test]
fn every_tag_across_every_story_becomes_a_filter_chip() {
    let shell = ShellBuilder::new()
        .story(story("a", "Atoms/A", &["atoms", "forms"]))
        .story(story("b", "Layout/B", &["layout"]))
        .build();

    let html = render_at(shell, "/");

    for tag in ["atoms", "forms", "layout"] {
        assert!(html.contains(&format!(">{tag}</button>")), "missing chip {tag}: {html}");
    }
}

#[test]
fn selecting_a_tag_narrows_the_navigation_tree_to_matching_stories() {
    let shell = ShellBuilder::new()
        .story(story("atoms-button", "Atoms/Button", &["atoms"]))
        .story(story("layout-grid", "Layout/Grid", &["layout"]))
        .build();
    let mut dom = dom_at(shell, "/", None);

    update(&mut dom, || {
        consume_context::<Signal<Option<String>>>().set(Some("atoms".to_owned()));
    });
    let html = dioxus_ssr::render(&dom);

    assert!(html.contains("/component/atoms-button"), "{html}");
    assert!(!html.contains("/component/layout-grid"), "{html}");
    // The chip needed to undo the filter is still offered.
    assert!(html.contains("Clear atoms"), "{html}");
}

#[test]
fn an_untagged_story_disappears_under_any_tag_filter() {
    let shell = ShellBuilder::new()
        .story(story("plain", "Plain/Story", &[]))
        .story(story("tagged", "Tagged/Story", &["atoms"]))
        .build();
    let mut dom = dom_at(shell, "/", None);

    update(&mut dom, || {
        consume_context::<Signal<Option<String>>>().set(Some("atoms".to_owned()));
    });
    let html = dioxus_ssr::render(&dom);

    assert!(!html.contains("/component/plain"), "{html}");
    assert!(html.contains("/component/tagged"), "{html}");
}

#[test]
fn a_stories_own_tags_are_listed_on_its_page() {
    let shell = ShellBuilder::new().story(story("a", "Atoms/A", &["atoms", "forms"])).build();

    let html = render_at(shell, "/component/a");

    assert!(html.contains("tag-button"), "{html}");
    assert!(html.contains(">atoms</button>"), "{html}");
}

#[test]
fn an_untagged_story_page_says_so_rather_than_rendering_an_empty_row() {
    let shell = ShellBuilder::new().story(story("a", "Atoms/A", &[])).build();

    let html = render_at(shell, "/component/a");

    assert!(html.contains("No tags"), "{html}");
}

// --- Theme toggle ---

#[test]
fn the_shell_starts_in_light_mode() {
    let html = render_at(ShellBuilder::new().build(), "/");

    assert!(html.contains("data-theme=\"light\""), "{html}");
    assert!(html.contains(">Light</span>"), "{html}");
}

#[test]
fn toggling_the_theme_switches_the_shell_to_dark_and_back() {
    let mut dom = dom_at(ShellBuilder::new().build(), "/", None);

    let flip = |dom: &mut VirtualDom| {
        update(dom, || {
            let mut theme = consume_context::<Signal<ThemeMode>>();
            let next = theme().toggle();
            theme.set(next);
        });
    };

    flip(&mut dom);
    let dark = dioxus_ssr::render(&dom);
    assert!(dark.contains("data-theme=\"dark\""), "{dark}");
    assert!(dark.contains(">Dark</span>"), "{dark}");

    flip(&mut dom);
    let light = dioxus_ssr::render(&dom);
    assert!(light.contains("data-theme=\"light\""), "{light}");
}

// --- Providers ---

/// Stories are wrapped in `StoryPreviewContent`, which reads the provider chain
/// out of context. This is the only place the shell has to hand it over.
#[test]
fn registered_providers_wrap_story_content() {
    fn outer(child: Element) -> Element {
        rsx! {
            div { class: "provider-outer", {child} }
        }
    }

    let wrapped = GeneratedStory {
        definition: definition("wrapped", "Wrapped/Story", &[]),
        render: Box::new(|| {
            rsx! {
                StoryPreviewContent {
                    div { class: "story-inner", "inner" }
                }
            }
        }),
    };
    let shell = ShellBuilder::new().story(wrapped).provider(outer).build();

    let html = render_at(shell, "/component/wrapped");

    assert!(html.contains("provider-outer"), "{html}");
    assert!(html.contains("story-inner"), "{html}");
}

// --- Title ---

#[test]
fn an_explicit_title_replaces_the_default_heading() {
    let html = render_at(ShellBuilder::new().title("Design System").build(), "/");

    assert!(html.contains("<h1>Design System</h1>"), "{html}");
}
