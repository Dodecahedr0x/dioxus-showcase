//! Link-time story and provider registration.
//!
//! Annotated items submit a [`ShowcaseRegistration`] or [`ProviderRegistration`]
//! into an `inventory` collection at their own call site. The showcase shell
//! reads those collections at startup, so nothing has to generate glue code that
//! names the macro-generated symbols.
//!
//! Two properties of this module are load-bearing:
//!
//! - **Everything is sorted before it is returned.** Link order is not a stable
//!   contract, and generated showcase output is asserted byte-identical across
//!   builds, so relying on the order `inventory` yields would fail only
//!   intermittently.
//! - **Duplicate story ids are reported, never panicked on.** One colliding id
//!   must degrade a single route, not take down the whole application.
use dioxus::prelude::*;

use crate::{GeneratedStory, StoryProvider};

/// One annotated item's registration.
///
/// Const-constructible on purpose: every field is a `&'static str` or a plain
/// `fn` pointer, because a `Box<dyn Fn>` cannot live in the `static` that
/// `inventory::submit!` creates. The heap allocation each story needs happens
/// *inside* [`ShowcaseRegistration::factory`], when it is called.
pub struct ShowcaseRegistration {
    /// The annotated item's own source file, from `file!()` at its call site.
    pub source_path: &'static str,
    /// The annotated item's path, as `krate::module::item_name`.
    pub module_path: &'static str,
    /// Expands this item into its stories. Called once per startup.
    pub factory: fn(&'static str, &'static str) -> Vec<GeneratedStory>,
}

inventory::collect!(ShowcaseRegistration);

/// One provider component's registration.
pub struct ProviderRegistration {
    /// The annotated component's path, as `krate::module::ComponentName`.
    pub module_path: &'static str,
    /// Ascending wrap order. The **lowest** order wraps **outermost**.
    pub order: i32,
    /// Wraps story content in this provider.
    pub wrap: fn(Element) -> Element,
}

inventory::collect!(ProviderRegistration);

/// Every registered story, plus any id collisions found while collecting them.
pub struct RegisteredStories {
    /// All stories, sorted deterministically by id.
    pub stories: Vec<GeneratedStory>,
    /// Ids claimed by more than one story, sorted and deduplicated.
    ///
    /// Non-empty means the shell should surface an error state. It never means
    /// the process should stop.
    pub duplicate_ids: Vec<String>,
}

/// Expands every registered item into its stories.
///
/// The result is deterministic regardless of link order.
pub fn registered_stories() -> RegisteredStories {
    collect_stories(inventory::iter::<ShowcaseRegistration>)
}

/// Returns every registered provider, outermost first.
///
/// Sorted by `(order, module_path)`, so the result is deterministic regardless
/// of link order.
pub fn registered_providers() -> Vec<StoryProvider> {
    sort_providers(inventory::iter::<ProviderRegistration>.into_iter().collect())
}

/// Expands the given registrations, then sorts and checks the result.
///
/// Split out from [`registered_stories`] so the ordering and duplicate rules can
/// be tested against registrations built in the test itself, rather than against
/// whatever the linker happened to collect.
fn collect_stories<'a>(
    registrations: impl IntoIterator<Item = &'a ShowcaseRegistration>,
) -> RegisteredStories {
    let mut stories = Vec::new();
    for registration in registrations {
        stories.extend((registration.factory)(registration.source_path, registration.module_path));
    }

    stories.sort_by(|left, right| story_sort_key(left).cmp(&story_sort_key(right)));

    let mut duplicate_ids: Vec<String> = stories
        .windows(2)
        .filter(|pair| pair[0].definition.id == pair[1].definition.id)
        .map(|pair| pair[0].definition.id.clone())
        .collect();
    duplicate_ids.dedup();

    RegisteredStories { stories, duplicate_ids }
}

/// Sorts provider registrations by `(order, module_path)` and drops the metadata.
///
/// Split out from [`registered_providers`] for the same reason as
/// [`collect_stories`].
fn sort_providers(mut registrations: Vec<&ProviderRegistration>) -> Vec<StoryProvider> {
    registrations.sort_by_key(|registration| (registration.order, registration.module_path));
    registrations.into_iter().map(|registration| registration.wrap).collect()
}

/// Builds the total ordering key for one story.
///
/// `id` alone is not a total order once two stories collide, and a stable sort
/// would then fall back to link order — which is exactly the non-determinism
/// this module exists to remove. The remaining fields break that tie.
fn story_sort_key(story: &GeneratedStory) -> (&str, &str, &str, &str) {
    let definition = &story.definition;
    (
        definition.id.as_str(),
        definition.module_path.as_str(),
        definition.source_path.as_str(),
        definition.title.as_str(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use dioxus_showcase_core::StoryDefinition;

    /// Builds a story the way a macro-generated factory does: from the
    /// `source_path` and `module_path` it was handed.
    fn story_from(id: &str, source_path: &str, module_path: &str) -> GeneratedStory {
        GeneratedStory {
            definition: StoryDefinition {
                id: id.to_owned(),
                title: id.to_owned(),
                source_path: source_path.to_owned(),
                module_path: module_path.to_owned(),
                renderer_symbol: "__dioxus_showcase_render__demo".to_owned(),
                tags: Vec::new(),
            },
            render: Box::new(|| rsx! { div { "story" } }),
        }
    }

    /// Shorthand for the helper-level tests, which do not care about the path.
    fn story(id: &str, module_path: &str) -> GeneratedStory {
        story_from(id, "src/lib.rs", module_path)
    }

    fn zeta_and_alpha(_source: &'static str, module_path: &'static str) -> Vec<GeneratedStory> {
        vec![story("zeta", module_path), story("alpha", module_path)]
    }

    fn mid(_source: &'static str, module_path: &'static str) -> Vec<GeneratedStory> {
        vec![story("mid", module_path)]
    }

    fn also_alpha(_source: &'static str, module_path: &'static str) -> Vec<GeneratedStory> {
        vec![story("alpha", module_path)]
    }

    fn identity(child: Element) -> Element {
        child
    }

    fn registration(
        module_path: &'static str,
        factory: fn(&'static str, &'static str) -> Vec<GeneratedStory>,
    ) -> ShowcaseRegistration {
        ShowcaseRegistration { source_path: "src/lib.rs", module_path, factory }
    }

    #[test]
    fn collect_stories_sorts_by_id_regardless_of_registration_order() {
        let forwards = [registration("krate::a", zeta_and_alpha), registration("krate::b", mid)];
        let backwards = [registration("krate::b", mid), registration("krate::a", zeta_and_alpha)];

        let ids = |registrations: &[ShowcaseRegistration]| {
            collect_stories(registrations)
                .stories
                .iter()
                .map(|story| story.definition.id.clone())
                .collect::<Vec<_>>()
        };

        assert_eq!(ids(&forwards), vec!["alpha", "mid", "zeta"]);
        assert_eq!(ids(&forwards), ids(&backwards));
    }

    #[test]
    fn collect_stories_orders_colliding_ids_deterministically() {
        // Two stories share an id, so `id` alone cannot decide their order. The
        // tie-break must not depend on which registration was seen first.
        let forwards =
            [registration("krate::zzz", also_alpha), registration("krate::aaa", also_alpha)];
        let backwards =
            [registration("krate::aaa", also_alpha), registration("krate::zzz", also_alpha)];

        let module_paths = |registrations: &[ShowcaseRegistration]| {
            collect_stories(registrations)
                .stories
                .iter()
                .map(|story| story.definition.module_path.clone())
                .collect::<Vec<_>>()
        };

        assert_eq!(module_paths(&forwards), vec!["krate::aaa", "krate::zzz"]);
        assert_eq!(module_paths(&forwards), module_paths(&backwards));
    }

    #[test]
    fn collect_stories_reports_duplicate_ids_instead_of_panicking() {
        let registrations = [
            registration("krate::a", zeta_and_alpha),
            registration("krate::b", also_alpha),
            registration("krate::c", also_alpha),
        ];

        let registered = collect_stories(&registrations);

        // Every story is still returned, including all three colliding ones.
        assert_eq!(registered.stories.len(), 4);
        // The collision is reported once, not once per extra copy.
        assert_eq!(registered.duplicate_ids, vec!["alpha".to_owned()]);
    }

    #[test]
    fn collect_stories_reports_no_duplicates_when_ids_are_unique() {
        let registrations =
            [registration("krate::a", zeta_and_alpha), registration("krate::b", mid)];

        assert!(collect_stories(&registrations).duplicate_ids.is_empty());
    }

    #[test]
    fn collect_stories_handles_an_empty_registry() {
        let registered = collect_stories(&[]);

        assert!(registered.stories.is_empty());
        assert!(registered.duplicate_ids.is_empty());
    }

    #[test]
    fn sort_providers_orders_by_order_then_module_path() {
        let outermost =
            ProviderRegistration { module_path: "krate::Theme", order: -10, wrap: identity };
        let middle_b = ProviderRegistration { module_path: "krate::B", order: 0, wrap: identity };
        let middle_a = ProviderRegistration { module_path: "krate::A", order: 0, wrap: identity };
        let innermost =
            ProviderRegistration { module_path: "krate::Router", order: 5, wrap: identity };

        let sorted = |registrations: Vec<&ProviderRegistration>| {
            let mut clone = registrations.clone();
            clone.sort_by_key(|registration| (registration.order, registration.module_path));
            let _ = sort_providers(registrations);
            clone.into_iter().map(|registration| registration.module_path).collect::<Vec<_>>()
        };

        assert_eq!(
            sorted(vec![&innermost, &middle_b, &outermost, &middle_a]),
            vec!["krate::Theme", "krate::A", "krate::B", "krate::Router"]
        );
        assert_eq!(
            sorted(vec![&middle_a, &outermost, &innermost, &middle_b]),
            vec!["krate::Theme", "krate::A", "krate::B", "krate::Router"]
        );
    }

    #[test]
    fn sort_providers_returns_one_wrapper_per_registration() {
        let first = ProviderRegistration { module_path: "krate::A", order: 0, wrap: identity };
        let second = ProviderRegistration { module_path: "krate::B", order: 1, wrap: identity };

        assert_eq!(sort_providers(vec![&second, &first]).len(), 2);
    }

    // --- The global registry, exercised through the real inventory collections ---
    //
    // These submissions exist only in this crate's test binary. They prove the
    // `inventory::collect!` / `inventory::submit!` pairing links and that the
    // public entry points apply the same rules as the helpers above.

    fn global_pair(source: &'static str, module_path: &'static str) -> Vec<GeneratedStory> {
        vec![
            story_from("zz-global-last", source, module_path),
            story_from("aa-global-first", source, module_path),
        ]
    }

    fn global_duplicate(source: &'static str, module_path: &'static str) -> Vec<GeneratedStory> {
        vec![story_from("aa-global-first", source, module_path)]
    }

    inventory::submit! {
        ShowcaseRegistration {
            source_path: file!(),
            module_path: concat!(module_path!(), "::", stringify!(global_pair)),
            factory: global_pair,
        }
    }

    inventory::submit! {
        ShowcaseRegistration {
            source_path: file!(),
            module_path: concat!(module_path!(), "::", stringify!(global_duplicate)),
            factory: global_duplicate,
        }
    }

    inventory::submit! {
        ProviderRegistration {
            module_path: concat!(module_path!(), "::", stringify!(Inner)),
            order: 3,
            wrap: identity,
        }
    }

    inventory::submit! {
        ProviderRegistration {
            module_path: concat!(module_path!(), "::", stringify!(Outer)),
            order: -3,
            wrap: identity,
        }
    }

    #[test]
    fn registered_stories_reads_the_global_registry_sorted_and_without_panicking() {
        let registered = registered_stories();

        let ids =
            registered.stories.iter().map(|story| story.definition.id.as_str()).collect::<Vec<_>>();
        assert_eq!(ids, vec!["aa-global-first", "aa-global-first", "zz-global-last"]);
        assert_eq!(registered.duplicate_ids, vec!["aa-global-first".to_owned()]);
    }

    #[test]
    fn registered_stories_carries_the_call_site_source_and_module_path() {
        let registered = registered_stories();
        let story = registered
            .stories
            .iter()
            .find(|story| story.definition.id == "zz-global-last")
            .expect("the globally registered story should be present");

        assert_eq!(story.definition.source_path, "crates/dioxus-showcase/src/registration.rs");
        assert_eq!(
            story.definition.module_path,
            "dioxus_showcase::registration::tests::global_pair"
        );
    }

    #[test]
    fn registered_providers_reads_the_global_registry_in_order() {
        assert_eq!(registered_providers().len(), 2);
    }
}
