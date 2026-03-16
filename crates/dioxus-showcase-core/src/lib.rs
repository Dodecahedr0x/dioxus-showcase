//! Core types and helpers for dioxus-showcase.

pub mod config;
pub mod manifest;
pub mod runtime;

pub use config::{ShowcaseBuildConfig, ShowcaseConfig, ShowcaseDevConfig, ShowcaseProjectConfig};
pub use manifest::{StoryDefinition, StoryManifest};
pub use runtime::{
    build_story_navigation, ProviderDefinition, ShowcaseRegistry, StoryEntry, StoryNavigationNode,
    StoryTreeEntry,
};
