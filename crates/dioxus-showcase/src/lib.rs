//! Public facade crate for dioxus-showcase.
use dioxus::prelude::Element;
use dioxus_core::{Callback, EventHandler};
use dioxus_showcase_core::StoryDefinition;

pub mod prelude {
    pub use crate::{GeneratedStory, ShowcaseStoryFactory};
    pub use crate::{StoryArg, StoryArgs, StoryProps, StoryVariant};
    pub use dioxus_showcase_core::{ShowcaseRegistry, StoryDefinition, StoryEntry};
    pub use dioxus_showcase_macros::{showcase, story, StoryProps};
}

pub use dioxus_showcase_core as core;
pub use dioxus_showcase_macros as macros;

pub fn slugify_title(title: &str) -> String {
    let mut out = String::with_capacity(title.len());
    let mut prev_dash = false;

    for ch in title.chars() {
        let normalized = ch.to_ascii_lowercase();
        if normalized.is_ascii_alphanumeric() {
            out.push(normalized);
            prev_dash = false;
            continue;
        }

        if !prev_dash {
            out.push('-');
            prev_dash = true;
        }
    }

    out.trim_matches('-').to_owned()
}

/// Produces a single default story value for one component parameter.
pub trait StoryArg: Sized {
    fn story_arg() -> Self;
}

/// Produces one or more story values for aggregate props types.
pub trait StoryArgs: Sized {
    fn stories() -> Vec<Self>;
}

impl<T: StoryArg> StoryArgs for T {
    fn stories() -> Vec<Self> {
        vec![T::story_arg()]
    }
}

/// Produces one or more named prop sets for a showcase component.
pub trait StoryProps: Sized {
    fn stories() -> Vec<StoryVariant<Self>>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoryVariant<T> {
    pub name: Option<String>,
    pub value: T,
}

impl<T> StoryVariant<T> {
    pub fn unnamed(value: T) -> Self {
        Self { name: None, value }
    }

    pub fn named(name: impl Into<String>, value: T) -> Self {
        Self { name: Some(name.into()), value }
    }
}

macro_rules! impl_story_arg_with_default {
    ($($ty:ty),* $(,)?) => {
        $(
            impl StoryArg for $ty {
                fn story_arg() -> Self {
                    Self::default()
                }
            }
        )*
    };
}

impl_story_arg_with_default!(
    bool, char, i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize, f32, f64
);

impl StoryArg for &'static str {
    fn story_arg() -> Self {
        "Lorem Ipsum"
    }
}

impl StoryArg for String {
    fn story_arg() -> Self {
        "Lorem Ipsum".to_string()
    }
}

impl<T> StoryArg for Option<T> {
    fn story_arg() -> Self {
        None
    }
}

impl<T> StoryArg for Vec<T> {
    fn story_arg() -> Self {
        Vec::new()
    }
}

impl StoryArg for Element {
    fn story_arg() -> Self {
        ::dioxus::prelude::rsx! {
            div { "Story content" }
        }
    }
}

impl<T: Sized + 'static> StoryArg for EventHandler<T> {
    fn story_arg() -> Self {
        Callback::new(|_| {})
    }
}

pub struct GeneratedStory {
    pub definition: StoryDefinition,
    pub render: Box<dyn Fn() -> Element>,
}

pub trait ShowcaseStoryFactory {
    fn create(source_path: &str, module_path: &str) -> Vec<GeneratedStory>;
}

#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Default)]
    struct DemoArgs;

    impl StoryArg for DemoArgs {
        fn story_arg() -> Self {
            Self::default()
        }
    }

    #[test]
    fn story_arg_trait_is_implementable() {
        fn assert_story_arg<T: StoryArg>() {}
        assert_story_arg::<DemoArgs>();
    }

    #[test]
    fn story_args_trait_is_derived_from_story_arg() {
        fn assert_story_args<T: StoryArgs>() {}
        assert_story_args::<DemoArgs>();
    }

    #[derive(Default)]
    struct DemoProps;

    impl StoryProps for DemoProps {
        fn stories() -> Vec<StoryVariant<Self>> {
            vec![StoryVariant::unnamed(Self::default())]
        }
    }

    #[test]
    fn story_props_trait_supports_named_variants() {
        let stories = DemoProps::stories();
        assert_eq!(stories.len(), 1);
        assert_eq!(stories[0].name, None);
    }

    #[test]
    fn prelude_reexports_core_types() {
        let definition = prelude::StoryDefinition {
            id: "id".to_owned(),
            title: "title".to_owned(),
            source_path: "showcase/button.stories.rs".to_owned(),
            module_path: "module::story".to_owned(),
            renderer_symbol: "story_renderer".to_owned(),
            tags: vec!["tag".to_owned()],
        };

        let entry = prelude::StoryEntry { definition, renderer_symbol: "story_renderer" };

        let mut registry = prelude::ShowcaseRegistry::default();
        registry.register(entry);
        assert_eq!(registry.story_count(), 1);
    }

    #[test]
    fn element_story_arg_defaults_to_placeholder_markup() {
        let element: Element = StoryArg::story_arg();
        assert!(element.is_ok());
    }
}
