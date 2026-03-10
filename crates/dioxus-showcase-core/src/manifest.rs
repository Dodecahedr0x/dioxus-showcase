#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoryDefinition {
    pub id: String,
    pub title: String,
    pub source_path: String,
    pub module_path: String,
    pub renderer_symbol: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StoryManifest {
    pub schema_version: u32,
    pub stories: Vec<StoryDefinition>,
}

impl StoryManifest {
    pub fn new(schema_version: u32) -> Self {
        Self { schema_version, stories: Vec::new() }
    }

    pub fn add_story(&mut self, story: StoryDefinition) {
        self.stories.push(story);
    }

    pub fn to_json(&self) -> String {
        let stories_json = self
            .stories
            .iter()
            .map(|story| {
                format!(
                    "{{\"id\":\"{}\",\"title\":\"{}\",\"source_path\":\"{}\",\"module_path\":\"{}\",\"renderer_symbol\":\"{}\",\"tags\":[{}]}}",
                    escape_json(&story.id),
                    escape_json(&story.title),
                    escape_json(&story.source_path),
                    escape_json(&story.module_path),
                    escape_json(&story.renderer_symbol),
                    story
                        .tags
                        .iter()
                        .map(|tag| format!("\"{}\"", escape_json(tag)))
                        .collect::<Vec<_>>()
                        .join(",")
                )
            })
            .collect::<Vec<_>>()
            .join(",");

        format!("{{\"schema_version\":{},\"stories\":[{}]}}", self.schema_version, stories_json)
    }
}

fn escape_json(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n")
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
        assert_eq!(
            json,
            "{\"schema_version\":1,\"stories\":[{\"id\":\"atoms-button\",\"title\":\"Atoms \\\"Button\\\"\",\"source_path\":\"showcase\\\\button.stories.rs\",\"module_path\":\"showcase\\\\button\\nstories.rs::button_default\",\"renderer_symbol\":\"button_default\",\"tags\":[\"atoms\",\"primary\\\"cta\"]}]}"
        );
    }

    #[test]
    fn new_manifest_starts_empty() {
        let manifest = StoryManifest::new(3);
        assert_eq!(manifest.schema_version, 3);
        assert!(manifest.stories.is_empty());
    }
}
