//! Tag collection and tag filtering.
//!
//! The selected tag lives in a `Signal<Option<String>>` provided by the shell
//! root; `None` means "show everything". Everything in this module that decides
//! *what* is shown is a plain function over story definitions, so it is testable
//! without a VirtualDom.
use std::collections::BTreeSet;

use dioxus::prelude::*;
use dioxus_showcase::core::StoryDefinition;

/// The currently selected tag filter. `None` shows every story.
pub(crate) type SelectedTag = Signal<Option<String>>;

/// Collects every distinct tag across the given stories, sorted.
///
/// Sorted because the registry's own ordering is by story id, which says nothing
/// useful about tags, and the filter chips should not move between builds.
pub(crate) fn all_tags<'a>(
    definitions: impl IntoIterator<Item = &'a StoryDefinition>,
) -> Vec<String> {
    definitions
        .into_iter()
        .flat_map(|definition| definition.tags.iter().cloned())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

/// Returns `true` when a story survives the given tag filter.
pub(crate) fn matches_tag(definition: &StoryDefinition, tag: Option<&str>) -> bool {
    match tag {
        None => true,
        Some(tag) => definition.tags.iter().any(|candidate| candidate == tag),
    }
}

/// Keeps only the stories that survive the given tag filter.
pub(crate) fn matching<'a>(
    definitions: impl IntoIterator<Item = &'a StoryDefinition>,
    tag: Option<&str>,
) -> Vec<&'a StoryDefinition> {
    definitions.into_iter().filter(|definition| matches_tag(definition, tag)).collect()
}

/// The sidebar panel of tag filter chips.
#[component]
pub(crate) fn TagFilterPanel(tags: Vec<String>) -> Element {
    let mut selected = use_context::<SelectedTag>();
    let active = selected.read().clone();

    rsx! {
        div { class: "tag-filter-panel",
            div { class: "tag-filter-header",
                h2 { class: "tag-filter-title", "Tags" }
                if let Some(tag) = active.as_ref() {
                    button {
                        class: "tag-filter-clear",
                        onclick: move |_| selected.set(None),
                        "Clear {tag}"
                    }
                }
            }
            if tags.is_empty() {
                p { class: "muted", "No tags available" }
            } else {
                div { class: "tag-filter-list",
                    button {
                        class: if active.is_none() { "tag-filter-chip active" } else { "tag-filter-chip" },
                        onclick: move |_| selected.set(None),
                        "All"
                    }
                    for tag in tags {
                        {
                            let is_active = active.as_deref() == Some(tag.as_str());
                            rsx! {
                                button {
                                    key: "{tag}",
                                    class: if is_active { "tag-filter-chip active" } else { "tag-filter-chip" },
                                    onclick: move |_| selected.set(Some(tag.clone())),
                                    "{tag}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// A row of clickable tags shown beside a story, which also set the filter.
#[component]
pub(crate) fn StoryTags(tags: Vec<String>) -> Element {
    let mut selected = use_context::<SelectedTag>();
    let active = selected.read().clone();

    if tags.is_empty() {
        return rsx! {
            p { class: "muted", "No tags" }
        };
    }

    rsx! {
        div { class: "tags",
            for tag in tags {
                {
                    let is_active = active.as_deref() == Some(tag.as_str());
                    let value = tag.clone();
                    rsx! {
                        button {
                            key: "{tag}",
                            class: if is_active { "tag tag-button active" } else { "tag tag-button" },
                            onclick: move |_| selected.set(Some(value.clone())),
                            "{tag}"
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::definition;

    #[test]
    fn all_tags_is_sorted_deduplicated_and_independent_of_story_order() {
        let forwards = [
            definition("z", "Z", &["forms", "atoms"]),
            definition("a", "A", &["atoms"]),
            definition("m", "M", &["layout"]),
        ];
        let backwards = [
            definition("m", "M", &["layout"]),
            definition("a", "A", &["atoms"]),
            definition("z", "Z", &["forms", "atoms"]),
        ];

        assert_eq!(all_tags(&forwards), vec!["atoms", "forms", "layout"]);
        assert_eq!(all_tags(&forwards), all_tags(&backwards));
    }

    #[test]
    fn all_tags_is_empty_when_no_story_is_tagged() {
        assert!(all_tags(&[definition("a", "A", &[])]).is_empty());
        assert!(all_tags(&[]).is_empty());
    }

    #[test]
    fn no_selected_tag_matches_every_story_including_untagged_ones() {
        assert!(matches_tag(&definition("a", "A", &[]), None));
        assert!(matches_tag(&definition("a", "A", &["atoms"]), None));
    }

    #[test]
    fn a_selected_tag_keeps_only_stories_carrying_it() {
        let stories = [
            definition("a", "A", &["atoms"]),
            definition("b", "B", &["layout", "atoms"]),
            definition("c", "C", &["layout"]),
            definition("d", "D", &[]),
        ];

        let kept =
            matching(&stories, Some("atoms")).iter().map(|d| d.id.as_str()).collect::<Vec<_>>();

        assert_eq!(kept, vec!["a", "b"]);
    }

    #[test]
    fn a_tag_nothing_carries_filters_everything_out() {
        let stories = [definition("a", "A", &["atoms"])];

        assert!(matching(&stories, Some("nope")).is_empty());
    }
}
