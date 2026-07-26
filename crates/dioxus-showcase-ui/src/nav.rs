//! The sidebar story tree.
//!
//! The tree itself is built by [`dioxus_showcase::core::build_story_navigation`],
//! which already knows how to fold slash-delimited titles like `Atoms/Button/Ghost`
//! into nested nodes. This module only adapts registered stories to that
//! function's input trait and renders the result.
use dioxus::prelude::*;
use dioxus_showcase::core::{build_story_navigation, StoryDefinition, StoryNavigationNode};

use crate::routes::Route;

/// Adapts a registered story to [`dioxus_showcase::core::StoryTreeEntry`].
///
/// The trait and `StoryDefinition` both live in `dioxus-showcase-core`, so the
/// orphan rule forbids implementing one for the other from this crate. This
/// borrowing wrapper is the local type that closes that gap; it copies nothing.
pub(crate) struct NavEntry<'a> {
    id: &'a str,
    title: &'a str,
}

impl dioxus_showcase::core::StoryTreeEntry for NavEntry<'_> {
    /// Returns the route id this navigation entry links to.
    fn story_id(&self) -> &str {
        self.id
    }

    /// Returns the slash-delimited title the tree is folded on.
    fn story_title(&self) -> &str {
        self.title
    }
}

/// Builds the navigation tree for an already-filtered set of stories.
pub(crate) fn navigation<'a>(
    definitions: impl IntoIterator<Item = &'a StoryDefinition>,
) -> Vec<StoryNavigationNode> {
    let entries = definitions
        .into_iter()
        .map(|definition| NavEntry { id: &definition.id, title: &definition.title })
        .collect::<Vec<_>>();

    build_story_navigation(&entries)
}

/// Returns `true` when a navigation node contains the active story below it.
///
/// Drives which `<details>` branches start open, so opening a deep story leaves
/// its whole ancestor chain expanded.
pub(crate) fn contains_active_story(node: &StoryNavigationNode, active_id: Option<&str>) -> bool {
    active_id.is_some()
        && (node.story_id.as_deref() == active_id
            || node.children.iter().any(|child| contains_active_story(child, active_id)))
}

/// Renders one level of the story navigation tree.
#[component]
pub(crate) fn StoryTree(nodes: Vec<StoryNavigationNode>, active_id: Option<String>) -> Element {
    rsx! {
        ul { class: "component-tree",
            for node in nodes {
                StoryTreeNode { key: "{node.title_path}", node, active_id: active_id.clone() }
            }
        }
    }
}

/// Renders a single navigation node as either a leaf link or a collapsible branch.
#[component]
pub(crate) fn StoryTreeNode(node: StoryNavigationNode, active_id: Option<String>) -> Element {
    let is_active = node.story_id.is_some() && node.story_id.as_deref() == active_id.as_deref();
    let is_open = contains_active_story(&node, active_id.as_deref());

    rsx! {
        li { class: "tree-node",
            if node.children.is_empty() {
                if let Some(story_id) = node.story_id.as_ref() {
                    Link {
                        to: Route::Component { id: story_id.clone() },
                        class: if is_active { "tree-leaf-link active" } else { "tree-leaf-link" },
                        "{node.segment}"
                    }
                }
            } else {
                details { class: "tree-branch", open: is_open,
                    summary { class: "tree-summary",
                        span { class: "tree-summary-label", "{node.segment}" }
                    }
                    div { class: "tree-branch-body",
                        if let Some(story_id) = node.story_id.as_ref() {
                            Link {
                                to: Route::Component { id: story_id.clone() },
                                class: if is_active { "tree-branch-link active" } else { "tree-branch-link" },
                                "Overview"
                            }
                        }
                        StoryTree { nodes: node.children.clone(), active_id: active_id.clone() }
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
    fn navigation_folds_slash_delimited_titles_into_a_tree() {
        let stories = [
            definition("atoms-button", "Atoms/Button", &[]),
            definition("atoms-input", "Atoms/Input", &[]),
            definition("layout-grid", "Layout/Grid", &[]),
        ];

        let tree = navigation(&stories);

        assert_eq!(tree.len(), 2);
        assert_eq!(tree[0].segment, "Atoms");
        assert_eq!(tree[0].children.len(), 2);
        assert_eq!(tree[1].segment, "Layout");
        assert_eq!(tree[0].children[0].story_id.as_deref(), Some("atoms-button"));
    }

    #[test]
    fn navigation_of_no_stories_is_an_empty_tree() {
        assert!(navigation(&[]).is_empty());
    }

    #[test]
    fn a_branch_is_open_only_when_the_active_story_is_below_it() {
        let stories =
            [definition("atoms-button", "Atoms/Button", &[]), definition("layout", "Layout", &[])];
        let tree = navigation(&stories);

        assert!(contains_active_story(&tree[0], Some("atoms-button")));
        assert!(!contains_active_story(&tree[1], Some("atoms-button")));
    }

    #[test]
    fn no_branch_is_open_when_no_story_is_active() {
        // A node with no story id has `story_id == None`, which must not be read
        // as "matches the absent active id" and spring every branch open.
        let tree = navigation(&[definition("atoms-button", "Atoms/Button", &[])]);

        assert!(!contains_active_story(&tree[0], None));
    }
}
