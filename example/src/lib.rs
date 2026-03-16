//! Example workspace member for `dioxus-showcase`.
//!
//! This crate exists to demonstrate how a regular workspace member can
//! annotate components and have them discovered by `dioxus-showcase`.
//!
//! Run from the repository root:
//! - `cargo run -p dioxus-showcase-cli -- check`
//! - `cargo run -p dioxus-showcase-cli -- dev`

pub mod button_variants;

/// Marker exported so the crate has a concrete API item.
pub const EXAMPLE_CRATE_NAME: &str = env!("CARGO_PKG_NAME");

#[cfg(test)]
mod tests {
    use super::button_variants::{
        __dioxus_showcase_story__PillButtonControllable,
        __dioxus_showcase_story__pill_button_disabled,
        __dioxus_showcase_story__pill_button_primary,
    };

    #[test]
    fn showcase_component_definition_generates_default_story() {
        let generated = __dioxus_showcase_story__PillButtonControllable(
            "src/button_variants.rs",
            "button_variants::PillButtonControllable",
        );

        assert_eq!(generated.len(), 1);
        assert_eq!(generated[0].definition.title, "PillButtonControllable");
        assert_eq!(generated[0].definition.id, "pillbuttoncontrollable");
        assert_eq!(
            generated[0].definition.tags,
            vec!["examples".to_owned(), "workspace".to_owned()]
        );
    }

    #[test]
    fn story_function_definition_generates_primary_story() {
        let generated = __dioxus_showcase_story__pill_button_primary(
            "src/button_variants.rs",
            "button_variants::pill_button_primary",
        );

        assert_eq!(generated.len(), 1);
        assert_eq!(generated[0].definition.title, "PillButtonControllable/Primary");
        assert_eq!(generated[0].definition.id, "pillbuttoncontrollable-primary");
        assert_eq!(
            generated[0].definition.tags,
            vec!["examples".to_owned(), "workspace".to_owned()]
        );
    }

    #[test]
    fn story_function_definition_generates_disabled_story() {
        let generated = __dioxus_showcase_story__pill_button_disabled(
            "src/button_variants.rs",
            "button_variants::pill_button_disabled",
        );

        assert_eq!(generated.len(), 1);
        assert_eq!(generated[0].definition.title, "PillButtonControllable/Disabled");
        assert_eq!(generated[0].definition.id, "pillbuttoncontrollable-disabled");
    }
}
