use crate::manifest::{StoryDefinition, StoryManifest};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoryEntry {
    pub definition: StoryDefinition,
    pub renderer_symbol: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoryNavigationNode {
    pub segment: String,
    pub title_path: String,
    pub story_id: Option<String>,
    pub children: Vec<Self>,
}

pub trait StoryTreeEntry {
    fn story_id(&self) -> &str;
    fn story_title(&self) -> &str;
}

#[derive(Debug, Default)]
pub struct ShowcaseRegistry {
    stories: Vec<StoryEntry>,
}

impl ShowcaseRegistry {
    pub fn register(&mut self, entry: StoryEntry) {
        self.stories.push(entry);
    }

    pub fn manifest(&self) -> StoryManifest {
        let mut manifest = StoryManifest::new(1);
        for story in &self.stories {
            manifest.add_story(story.definition.clone());
        }
        manifest
    }

    pub fn story_count(&self) -> usize {
        self.stories.len()
    }
}

pub fn build_story_navigation<T: StoryTreeEntry>(stories: &[T]) -> Vec<StoryNavigationNode> {
    let mut nodes = Vec::new();

    for story in stories {
        let title = story.story_title().trim();
        let segments = split_story_title(title);
        if segments.is_empty() {
            continue;
        }

        insert_story_node(&mut nodes, &segments, story.story_id());
    }

    nodes
}

fn split_story_title(title: &str) -> Vec<&str> {
    title.split('/').map(str::trim).filter(|segment| !segment.is_empty()).collect()
}

fn insert_story_node(nodes: &mut Vec<StoryNavigationNode>, segments: &[&str], story_id: &str) {
    let segment = segments[0];
    let node_index = match nodes.iter().position(|node| node.segment == segment) {
        Some(index) => index,
        None => {
            let title_path = segment.to_owned();
            nodes.push(StoryNavigationNode {
                segment: segment.to_owned(),
                title_path,
                story_id: None,
                children: Vec::new(),
            });
            nodes.len() - 1
        }
    };

    let node = &mut nodes[node_index];
    if segments.len() == 1 {
        node.story_id = Some(story_id.to_owned());
        return;
    }

    insert_story_child(&mut node.children, &node.title_path, &segments[1..], story_id);
}

fn insert_story_child(
    children: &mut Vec<StoryNavigationNode>,
    parent_path: &str,
    segments: &[&str],
    story_id: &str,
) {
    let segment = segments[0];
    let node_index = match children.iter().position(|node| node.segment == segment) {
        Some(index) => index,
        None => {
            let title_path = format!("{parent_path}/{segment}");
            children.push(StoryNavigationNode {
                segment: segment.to_owned(),
                title_path,
                story_id: None,
                children: Vec::new(),
            });
            children.len() - 1
        }
    };

    let node = &mut children[node_index];
    if segments.len() == 1 {
        node.story_id = Some(story_id.to_owned());
        return;
    }

    insert_story_child(&mut node.children, &node.title_path, &segments[1..], story_id);
}

impl StoryTreeEntry for StoryEntry {
    fn story_id(&self) -> &str {
        &self.definition.id
    }

    fn story_title(&self) -> &str {
        &self.definition.title
    }
}

impl<T> StoryTreeEntry for &T
where
    T: StoryTreeEntry + ?Sized,
{
    fn story_id(&self) -> &str {
        (*self).story_id()
    }

    fn story_title(&self) -> &str {
        (*self).story_title()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_updates_count_and_manifest() {
        let mut registry = ShowcaseRegistry::default();
        registry.register(StoryEntry {
            definition: StoryDefinition {
                id: "atoms-button-default".to_owned(),
                title: "Atoms/Button/Default".to_owned(),
                source_path: "showcase/button.stories.rs".to_owned(),
                module_path: "showcase::button_default".to_owned(),
                renderer_symbol: "button_default".to_owned(),
                tags: vec!["atoms".to_owned()],
            },
            renderer_symbol: "button_default",
        });

        assert_eq!(registry.story_count(), 1);
        let manifest = registry.manifest();
        assert_eq!(manifest.schema_version, 1);
        assert_eq!(manifest.stories.len(), 1);
        assert_eq!(manifest.stories[0].id, "atoms-button-default");
    }

    #[test]
    fn build_story_navigation_groups_titles_into_tree() {
        let stories = vec![
            StoryEntry {
                definition: StoryDefinition {
                    id: "atoms-dropdown-link".to_owned(),
                    title: "Atoms/Dropdown Link".to_owned(),
                    source_path: "showcase/button.stories.rs".to_owned(),
                    module_path: "showcase::dropdown_link".to_owned(),
                    renderer_symbol: "dropdown_link".to_owned(),
                    tags: vec![],
                },
                renderer_symbol: "dropdown_link",
            },
            StoryEntry {
                definition: StoryDefinition {
                    id: "atoms-button".to_owned(),
                    title: "Atoms/Button".to_owned(),
                    source_path: "showcase/button.stories.rs".to_owned(),
                    module_path: "showcase::button".to_owned(),
                    renderer_symbol: "button".to_owned(),
                    tags: vec![],
                },
                renderer_symbol: "button",
            },
        ];

        let navigation = build_story_navigation(&stories);
        assert_eq!(navigation.len(), 1);
        assert_eq!(navigation[0].segment, "Atoms");
        assert_eq!(navigation[0].title_path, "Atoms");
        assert_eq!(navigation[0].children.len(), 2);
        assert_eq!(navigation[0].children[0].segment, "Dropdown Link");
        assert_eq!(navigation[0].children[0].story_id.as_deref(), Some("atoms-dropdown-link"));
        assert_eq!(navigation[0].children[1].segment, "Button");
        assert_eq!(navigation[0].children[1].story_id.as_deref(), Some("atoms-button"));
    }

    #[test]
    fn build_story_navigation_allows_branch_to_be_story_and_parent() {
        let stories = vec![
            StoryEntry {
                definition: StoryDefinition {
                    id: "atoms".to_owned(),
                    title: "Atoms".to_owned(),
                    source_path: "showcase/atoms.rs".to_owned(),
                    module_path: "showcase::atoms".to_owned(),
                    renderer_symbol: "atoms".to_owned(),
                    tags: vec![],
                },
                renderer_symbol: "atoms",
            },
            StoryEntry {
                definition: StoryDefinition {
                    id: "atoms-button".to_owned(),
                    title: "Atoms/Button".to_owned(),
                    source_path: "showcase/button.rs".to_owned(),
                    module_path: "showcase::button".to_owned(),
                    renderer_symbol: "button".to_owned(),
                    tags: vec![],
                },
                renderer_symbol: "button",
            },
        ];

        let navigation = build_story_navigation(&stories);
        assert_eq!(navigation.len(), 1);
        assert_eq!(navigation[0].story_id.as_deref(), Some("atoms"));
        assert_eq!(navigation[0].children.len(), 1);
        assert_eq!(navigation[0].children[0].segment, "Button");
    }

    #[test]
    fn build_story_navigation_accepts_slices_of_references() {
        let stories = [StoryEntry {
            definition: StoryDefinition {
                id: "atoms-button".to_owned(),
                title: "Atoms/Button".to_owned(),
                source_path: "showcase/button.rs".to_owned(),
                module_path: "showcase::button".to_owned(),
                renderer_symbol: "button".to_owned(),
                tags: vec![],
            },
            renderer_symbol: "button",
        }];
        let filtered = stories.iter().collect::<Vec<_>>();

        let navigation = build_story_navigation(&filtered);

        assert_eq!(navigation.len(), 1);
        assert_eq!(navigation[0].segment, "Atoms");
        assert_eq!(navigation[0].children.len(), 1);
        assert_eq!(navigation[0].children[0].story_id.as_deref(), Some("atoms-button"));
    }
}
