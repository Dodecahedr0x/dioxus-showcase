//! The sidebar: title, theme control, tag filters and the story tree.
use dioxus::prelude::*;

use crate::filters::{all_tags, matching, SelectedTag, TagFilterPanel};
use crate::nav::{navigation, StoryTree};
use crate::state::Shell;
use crate::theme::ThemePanel;

/// Renders the showcase sidebar for the currently active story, if any.
#[component]
pub(crate) fn Sidebar(active_id: Option<String>) -> Element {
    let shell = use_context::<Shell>();
    let selected_tag = use_context::<SelectedTag>();
    let active_tag = selected_tag.read().clone();

    // The tag list is drawn from every story, not from the filtered set — a
    // filter that erased the chips needed to clear it would be a trap.
    let tags = all_tags(shell.definitions());
    let nodes = navigation(matching(shell.definitions(), active_tag.as_deref()));

    rsx! {
        aside { class: "sidebar",
            h1 { "{shell.title()}" }
            ThemePanel {}
            TagFilterPanel { tags }
            StoryTree { nodes, active_id }
        }
    }
}
