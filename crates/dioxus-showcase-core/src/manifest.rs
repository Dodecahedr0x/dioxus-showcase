use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoryDefinition {
    pub id: String,
    pub title: String,
    pub source_path: String,
    pub module_path: String,
    pub renderer_symbol: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct StoryManifest {
    pub schema_version: u32,
    pub stories: Vec<StoryDefinition>,
}

impl StoryManifest {
    /// Creates an empty manifest using the provided schema version.
    pub fn new(schema_version: u32) -> Self {
        Self { schema_version, stories: Vec::new() }
    }

    /// Appends a discovered story definition to the manifest.
    pub fn add_story(&mut self, story: StoryDefinition) {
        self.stories.push(story);
    }

    /// Serializes the manifest to compact JSON for artifact output.
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("story manifest serialization should not fail")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_json_serializes_manifest_with_escaped_content() {
        let mut manifest = StoryManifest::new(1);
        manifest.add_story(StoryDefinition {
            id: "atoms-button".to_owned(),
            title: "Atoms \"Button\"".to_owned(),
            source_path: "showcase\\button.stories.rs".to_owned(),
            module_path: "showcase\\button\nstories.rs::button_default".to_owned(),
            renderer_symbol: "button_default".to_owned(),
            tags: vec!["atoms".to_owned(), "primary\"cta".to_owned()],
        });

        let json = manifest.to_json();
        let decoded: StoryManifest = serde_json::from_str(&json).expect("valid json");
        assert_eq!(decoded, manifest);
    }

    #[test]
    fn new_manifest_starts_empty() {
        let manifest = StoryManifest::new(3);
        assert_eq!(manifest.schema_version, 3);
        assert!(manifest.stories.is_empty());
    }

    #[test]
    fn manifest_round_trips_through_json() {
        let mut manifest = StoryManifest::new(2);
        manifest.add_story(StoryDefinition {
            id: "atoms-button".to_owned(),
            title: "Atoms/Button".to_owned(),
            source_path: "src/button.rs".to_owned(),
            module_path: "button::Button".to_owned(),
            renderer_symbol: "__dioxus_showcase_render__Button".to_owned(),
            tags: vec!["atoms".to_owned()],
        });

        let json = manifest.to_json();
        let decoded: StoryManifest = serde_json::from_str(&json).expect("valid json");

        assert_eq!(decoded, manifest);
    }
}
