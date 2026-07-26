use dioxus::prelude::*;
use dioxus_showcase::{StoryProps as StoryPropsTrait, StoryVariant};
use dioxus_showcase_macros::{provider, showcase, story, StoryProps};

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

#[story(title = "Atoms/Button/Default")]
fn button_default() -> &'static str {
    "ok"
}

#[story(title = "Atoms/Button/Explicit Title")]
fn button_explicit_title() -> &'static str {
    "explicit"
}

#[story(title = "Atoms/Button/Controlled")]
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

#[provider(order = 2)]
#[component]
fn story_shell(children: Element) -> Element {
    rsx! {
        div { class: "story-shell", {children} }
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
    assert_eq!(generated[0].definition.id, "atoms-button-default");
    assert_eq!(generated[0].definition.title, "Atoms/Button/Default");
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
    assert_eq!(generated[0].definition.id, "atoms-button-controlled");
    assert_eq!(generated[0].definition.title, "Atoms/Button/Controlled");
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
fn provider_attribute_generates_wrapper_function() {
    let wrapped = __dioxus_showcase_wrap__story_shell(rsx! { span { "inside" } });
    assert!(wrapped.is_ok());
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

// --- Link-time registration (C1/C2) ---
//
// These items are annotated above and referenced by nothing below. They reach
// the registry only through the `inventory::submit!` the macros emit, which is
// the whole point of the contract.

#[story(title = "Atoms/Button/Default")]
fn button_default_collides_on_purpose() -> &'static str {
    "duplicate"
}

#[test]
fn annotated_items_register_themselves_at_link_time() {
    let registered = dioxus_showcase::registered_stories();

    let ids =
        registered.stories.iter().map(|story| story.definition.id.as_str()).collect::<Vec<_>>();

    assert!(ids.contains(&"atoms-button-default"), "got {ids:?}");
    assert!(ids.contains(&"atoms-button-controlled"), "got {ids:?}");
    assert!(ids.contains(&"atoms-slot"), "got {ids:?}");
}

#[test]
fn registered_stories_are_sorted_by_id() {
    let registered = dioxus_showcase::registered_stories();

    let ids =
        registered.stories.iter().map(|story| story.definition.id.clone()).collect::<Vec<_>>();
    let mut expected = ids.clone();
    expected.sort();

    assert_eq!(ids, expected);
}

#[test]
fn colliding_story_ids_are_reported_rather_than_panicking() {
    // Two `#[story]` items in this file claim "Atoms/Button/Default".
    let registered = dioxus_showcase::registered_stories();

    assert!(
        registered.duplicate_ids.contains(&"atoms-button-default".to_owned()),
        "got {:?}",
        registered.duplicate_ids
    );
    // Both colliding stories are still present; neither was dropped.
    let collisions = registered
        .stories
        .iter()
        .filter(|story| story.definition.id == "atoms-button-default")
        .count();
    assert_eq!(collisions, 2);
}

#[test]
fn registration_captures_the_call_site_file_and_module_path() {
    let registered = dioxus_showcase::registered_stories();
    let story = registered
        .stories
        .iter()
        .find(|story| story.definition.id == "atoms-slot")
        .expect("the slot story should be registered");

    assert_eq!(story.definition.source_path, "crates/dioxus-showcase-macros/tests/macros.rs");
    assert_eq!(story.definition.module_path, "macros::slot_component");
}

#[test]
fn providers_register_themselves_at_link_time() {
    assert_eq!(dioxus_showcase::registered_providers().len(), 1);
}

#[story(title = "Controls/Defaults")]
fn story_with_param_defaults(
    #[default = 32.0] size: f64,
    #[default = "currentColor"] color: String,
    #[default = 6] count: usize,
    #[default = true] filled: bool,
) -> Element {
    rsx! { "{size} {color} {count} {filled}" }
}

#[test]
fn param_defaults_seed_the_controls_and_the_preview() {
    // The rendered markup carries both halves: the preview reflects the
    // defaults, and each control input opens on the same value rather than on
    // `StoryArg`'s placeholder seed.
    let mut dom = VirtualDom::new(__dioxus_showcase_render__story_with_param_defaults);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);

    assert!(html.contains("32 currentColor 6 true"), "preview: {html}");
    assert!(html.contains(r#"type="number" value="32""#), "size control: {html}");
    assert!(html.contains(r#"type="text" value="currentColor""#), "color control: {html}");
    assert!(html.contains(r#"type="number" value="6""#), "count control: {html}");
    // dioxus-ssr emits boolean attributes unquoted.
    assert!(html.contains(r#"type="checkbox" checked=true"#), "bool control: {html}");
}

#[story(title = "Controls/NoDefaults")]
fn story_without_param_defaults(size: f64, color: String) -> Element {
    rsx! { "{size} {color}" }
}

#[test]
fn a_parameter_without_a_default_still_falls_back_to_the_story_arg_seed() {
    let mut dom = VirtualDom::new(__dioxus_showcase_render__story_without_param_defaults);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);

    assert!(html.contains("0 Lorem Ipsum"), "preview: {html}");
    assert!(html.contains(r#"type="number" value="0""#), "size control: {html}");
}
