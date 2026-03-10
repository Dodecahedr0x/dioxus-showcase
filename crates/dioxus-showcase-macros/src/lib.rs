extern crate proc_macro;

mod derive_story_props;
mod showcase;
mod story;
mod utils;

use proc_macro::TokenStream;

/// Marks a function as a named story entry.
///
/// The annotated function is preserved and hidden showcase registration items
/// are generated alongside it, similar to `#[showcase]`.
///
/// Annotated story functions may take either no arguments or any number of
/// named parameters whose types implement `dioxus_showcase::StoryArg`. Basic
/// scalar parameter types become interactive controls automatically.
///
/// # Example
///
/// ```no_run
/// use dioxus::prelude::*;
/// use dioxus_showcase::prelude::*;
///
/// #[story(component = Button, name = "Primary")]
/// fn button_primary() -> Element {
///     rsx! { button { "Save" } }
/// }
/// ```
///
/// Supported arguments:
/// - `title = "..."`
/// - `component = ComponentName`
/// - `name = "..."`
/// - `tags = ["...", "..."]`
#[proc_macro_attribute]
pub fn story(attr: TokenStream, item: TokenStream) -> TokenStream {
    story::expand(attr.into(), item.into()).into()
}

/// Registers a Dioxus component as a showcase story source.
///
/// This attribute preserves the original component function and generates
/// hidden helper items used by the showcase runtime:
/// - a renderer function that mounts the component,
/// - a factory type implementing `dioxus_showcase::ShowcaseStoryFactory`,
/// - a constructor function returning one or more
///   `dioxus_showcase::GeneratedStory` values.
///
/// Annotated components may take either:
/// - no arguments,
/// - a single aggregate `props` argument implementing
///   `dioxus_showcase::StoryProps`,
/// - or any number of named parameters whose types implement
///   `dioxus_showcase::StoryArg`.
///
/// # Example
///
/// ```no_run
/// use dioxus::prelude::*;
/// use dioxus_showcase::prelude::*;
///
/// #[showcase(title = "Atoms/Button", tags = ["atoms", "button"])]
/// #[component]
/// fn Button(label: String, disabled: bool) -> Element {
///     rsx! { button { disabled: disabled, "{label}" } }
/// }
/// ```
///
/// Supported arguments:
/// - `title = "..."`
/// - `tags = ["...", "..."]`
#[proc_macro_attribute]
pub fn showcase(attr: TokenStream, item: TokenStream) -> TokenStream {
    showcase::expand(attr.into(), item.into()).into()
}

/// Derives `dioxus_showcase::StoryArg` and `dioxus_showcase::StoryProps`
/// from `Default`.
///
/// The generated implementation returns a single parameter value created with
/// `Default::default()`. This is useful for individual component parameters or
/// for aggregate props structs used as a single `props` argument.
///
/// # Example
///
/// ```no_run
/// use dioxus_showcase::prelude::*;
///
/// #[derive(Default, StoryProps)]
/// struct ButtonProps {
///     label: String,
/// }
/// ```
///
/// Equivalent expansion:
///
/// ```no_run
/// #[derive(Default)]
/// struct ButtonProps {
///     label: String,
/// }
///
/// impl dioxus_showcase::StoryArg for ButtonProps {
///     fn story_arg() -> Self {
///         Self::default()
///     }
/// }
///
/// impl dioxus_showcase::StoryProps for ButtonProps {
///     fn stories() -> Vec<dioxus_showcase::StoryVariant<Self>> {
///         vec![dioxus_showcase::StoryVariant::unnamed(Self::default())]
///     }
/// }
/// ```
#[proc_macro_derive(StoryProps)]
pub fn derive_story_props(input: TokenStream) -> TokenStream {
    derive_story_props::expand(input.into()).into()
}
