//! Everything the shell renders from, in one context value.
//!
//! The registry is read exactly once, at the root, and handed down as [`Shell`].
//! It cannot be passed as ordinary props: a [`GeneratedStory`] owns a
//! `Box<dyn Fn() -> Element>`, which is neither `Clone` nor `PartialEq`, and
//! Dioxus props require both. Wrapping the whole state in an `Rc` gives cheap
//! `Clone` and a `PartialEq` that is pointer identity — which is also the
//! correct comparison here, since the registry is fixed at link time and a new
//! `Rc` only ever appears when the shell is genuinely rebuilt.
//!
//! Routing this through a context rather than the global registry is what makes
//! the shell testable: a test constructs whatever story set it wants and injects
//! it, instead of trying to arrange link-time registrations per test case.
use std::rc::Rc;

use dioxus_showcase::core::StoryDefinition;
use dioxus_showcase::{registered_providers, registered_stories, GeneratedStory, StoryProvider};

use crate::base_path::BasePath;

/// The immutable half of the shell's state.
pub(crate) struct ShellState {
    base_path: BasePath,
    title: String,
    stories: Vec<GeneratedStory>,
    duplicate_ids: Vec<String>,
    providers: Vec<StoryProvider>,
}

/// A cheap handle to [`ShellState`], suitable as a prop and as a context value.
#[derive(Clone)]
pub(crate) struct Shell(Rc<ShellState>);

impl PartialEq for Shell {
    /// Compares by identity, not contents.
    ///
    /// The contents cannot be compared — story render closures have no
    /// `PartialEq` — and do not need to be: the registry is link-time constant,
    /// so two handles differ only when the shell was rebuilt from scratch.
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl Shell {
    /// Builds shell state from an explicit story set.
    pub(crate) fn new(
        base_path: BasePath,
        title: String,
        stories: Vec<GeneratedStory>,
        duplicate_ids: Vec<String>,
        providers: Vec<StoryProvider>,
    ) -> Self {
        Self(Rc::new(ShellState { base_path, title, stories, duplicate_ids, providers }))
    }

    /// Builds shell state from the link-time registry.
    ///
    /// Duplicate ids are carried through rather than raised: a collision degrades
    /// one route into an ambiguity, and must not take the application down.
    pub(crate) fn from_registry(base_path: BasePath, title: String) -> Self {
        let registered = registered_stories();
        Self::new(
            base_path,
            title,
            registered.stories,
            registered.duplicate_ids,
            registered_providers(),
        )
    }

    /// Returns the normalised base path the site is served under.
    pub(crate) fn base_path(&self) -> &BasePath {
        &self.0.base_path
    }

    /// Returns the sidebar heading.
    pub(crate) fn title(&self) -> &str {
        &self.0.title
    }

    /// Returns every registered story's definition, in registry order.
    pub(crate) fn definitions(&self) -> impl Iterator<Item = &StoryDefinition> {
        self.0.stories.iter().map(|story| &story.definition)
    }

    /// Returns `true` when nothing at all is registered.
    pub(crate) fn is_empty(&self) -> bool {
        self.0.stories.is_empty()
    }

    /// Returns the ids claimed by more than one story.
    pub(crate) fn duplicate_ids(&self) -> &[String] {
        &self.0.duplicate_ids
    }

    /// Returns the provider chain, outermost first.
    pub(crate) fn providers(&self) -> Vec<StoryProvider> {
        self.0.providers.clone()
    }

    /// Finds the story serving a route id.
    ///
    /// When an id is duplicated this returns the first match under the registry's
    /// deterministic ordering, so the route stays stable across builds even while
    /// the collision is reported separately.
    pub(crate) fn story(&self, id: &str) -> Option<&GeneratedStory> {
        self.0.stories.iter().find(|story| story.definition.id == id)
    }
}

#[cfg(test)]
mod tests {
    use crate::testing::{story, ShellBuilder};

    #[test]
    fn a_shell_handle_equals_only_its_own_clone() {
        let shell = ShellBuilder::new().build();
        let other = ShellBuilder::new().build();

        assert!(shell == shell.clone(), "a clone shares the same state");
        assert!(shell != other, "separately built states are distinct");
    }

    #[test]
    fn story_lookup_finds_a_registered_id_and_misses_an_unknown_one() {
        let shell = ShellBuilder::new().story(story("atoms-button", "Atoms/Button", &[])).build();

        assert!(shell.story("atoms-button").is_some());
        assert!(shell.story("nope").is_none());
        assert!(!shell.is_empty());
    }

    #[test]
    fn story_lookup_on_a_duplicated_id_resolves_to_the_first_registered_match() {
        let shell = ShellBuilder::new()
            .story(story("dup", "First/Copy", &[]))
            .story(story("dup", "Second/Copy", &[]))
            .duplicate_id("dup")
            .build();

        assert_eq!(shell.story("dup").unwrap().definition.title, "First/Copy");
        assert_eq!(shell.duplicate_ids(), ["dup".to_owned()]);
    }

    #[test]
    fn an_empty_registry_yields_an_empty_shell_rather_than_an_error() {
        let shell = ShellBuilder::new().build();

        assert!(shell.is_empty());
        assert_eq!(shell.definitions().count(), 0);
        assert!(shell.duplicate_ids().is_empty());
    }
}
