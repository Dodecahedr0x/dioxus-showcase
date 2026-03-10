use dioxus::prelude::*;
use dioxus_showcase::{StoryProps as StoryPropsTrait, StoryVariant};
use dioxus_showcase_macros::{showcase, story, StoryProps};

#[derive(Default, StoryProps)]
struct ButtonArgs;

#[derive(Default, StoryProps)]
enum Variant {
    #[default]
    A,
}

#[derive(Clone, PartialEq, Default, Props, StoryProps)]
struct ButtonProps {
    label: String,
}

#[derive(Clone, PartialEq, Default, Props)]
struct ButtonNamedProps {
    label: String,
}

impl StoryPropsTrait for ButtonNamedProps {
    fn stories() -> Vec<StoryVariant<Self>> {
        vec![
            StoryVariant::unnamed(Self::default()),
            StoryVariant::named("Filled", Self { label: "filled".to_owned() }),
        ]
    }
}

#[story(component = button_component, name = "Default")]
fn button_default() -> &'static str {
    "ok"
}

#[story(title = "Atoms/Button/Explicit Title")]
fn button_explicit_title() -> &'static str {
    "explicit"
}

#[story(component = button_component, name = "Controlled")]
fn button_controlled_story(label: String, disabled: bool) -> Element {
    rsx! {
        button { disabled, "{label}" }
    }
}

#[showcase(title = "Atoms/Button")]
#[component]
fn button_component() -> Element {
    rsx! { "component" }
}

#[showcase]
#[component]
fn button_component_with_props(props: ButtonProps) -> Element {
    rsx! { "{props.label}" }
}

#[showcase(title = "Atoms/Button Named")]
#[component]
fn button_component_with_named_props(props: ButtonNamedProps) -> Element {
    rsx! { "{props.label}" }
}

#[showcase(title = "Atoms/Button/Args")]
#[component]
fn button_component_with_args(label: String, disabled: bool) -> Element {
    rsx! {
        button { disabled, "{label}" }
    }
}

#[showcase(title = "Atoms/Slot")]
#[component]
fn slot_component(content: Element) -> Element {
    rsx! {
        section {
            {content}
        }
    }
}

#[test]
fn story_attribute_preserves_function_item() {
    assert_eq!(button_default(), "ok");
}

#[test]
fn story_attribute_generates_story_metadata() {
    let generated =
        __dioxus_showcase_story__button_default("src/macros.rs", "macros::button_default");
    assert_eq!(generated.len(), 1);
    assert_eq!(generated[0].definition.id, "button-component-default");
    assert_eq!(generated[0].definition.title, "button_component/Default");
    assert_eq!(generated[0].definition.renderer_symbol, "__dioxus_showcase_render__button_default");
}

#[test]
fn story_attribute_still_supports_explicit_title() {
    let generated = __dioxus_showcase_story__button_explicit_title(
        "src/macros.rs",
        "macros::button_explicit_title",
    );
    assert_eq!(generated.len(), 1);
    assert_eq!(generated[0].definition.id, "atoms-button-explicit-title");
    assert_eq!(generated[0].definition.title, "Atoms/Button/Explicit Title");
}

#[test]
fn story_attribute_supports_controlled_parameters() {
    let _ = __dioxus_showcase_render__button_controlled_story();
    let generated = __dioxus_showcase_story__button_controlled_story(
        "src/macros.rs",
        "macros::button_controlled_story",
    );

    assert_eq!(generated.len(), 1);
    assert_eq!(generated[0].definition.id, "button-component-controlled");
    assert_eq!(generated[0].definition.title, "button_component/Controlled");
}

#[test]
fn showcase_attribute_preserves_component_item() {
    let _ = button_component();
}

#[test]
fn showcase_generates_renderer_for_props_component() {
    let _ = __dioxus_showcase_render__button_component();
    let _ = __dioxus_showcase_render__button_component_with_props();
}

#[test]
fn showcase_generates_renderer_for_multi_arg_component() {
    let _ = __dioxus_showcase_render__button_component_with_args();
}

#[test]
fn showcase_supports_element_arguments() {
    let _ = __dioxus_showcase_render__slot_component();
    let generated =
        __dioxus_showcase_story__slot_component("src/macros.rs", "macros::slot_component");
    assert_eq!(generated.len(), 1);

    let content: Element = dioxus_showcase::StoryArg::story_arg();
    assert!(content.is_ok());
}

#[test]
fn showcase_generates_story_metadata() {
    let generated =
        __dioxus_showcase_story__button_component("src/macros.rs", "macros::button_component");
    assert_eq!(generated.len(), 1);
    assert_eq!(generated[0].definition.id, "atoms-button");
    assert_eq!(generated[0].definition.title, "Atoms/Button");
    assert_eq!(generated[0].definition.tags, Vec::<String>::new());
    assert_eq!(
        generated[0].definition.renderer_symbol,
        "__dioxus_showcase_render__button_component"
    );
}

#[test]
fn showcase_generates_multiple_named_prop_stories() {
    let generated = __dioxus_showcase_story__button_component_with_named_props(
        "src/macros.rs",
        "macros::button_component_with_named_props",
    );

    assert_eq!(generated.len(), 2);
    assert_eq!(generated[0].definition.title, "Atoms/Button Named");
    assert_eq!(generated[0].definition.id, "atoms-button-named");
    assert_eq!(generated[1].definition.title, "Atoms/Button Named/Filled");
    assert_eq!(generated[1].definition.id, "atoms-button-named-filled");
}

#[test]
fn story_props_derive_supports_default_types() {
    let _: ButtonArgs = dioxus_showcase::StoryArg::story_arg();
    let _: Variant = dioxus_showcase::StoryArg::story_arg();
    let _: ButtonProps = dioxus_showcase::StoryArg::story_arg();
}
