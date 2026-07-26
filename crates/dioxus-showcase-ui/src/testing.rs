//! Test-only harness for driving the shell without the global registry.
//!
//! Two things make shell components awkward to render bare, and both are handled
//! here:
//!
//! - `use_context` panics when the context is absent, so the shell has to be
//!   rendered under [`ShellRoot`], which provides all of it.
//! - `use_route` and `Link` panic outside a `Router`, and the router needs a
//!   `History`. Injecting a [`MemoryHistory`] as a root context is what lets a
//!   test render a specific route — and, with a prefix, assert on base-path
//!   behaviour without a browser.
use std::rc::Rc;

use dioxus::dioxus_core::{consume_context_from_scope, NoOpMutations};
use dioxus::history::{History, MemoryHistory};
use dioxus::prelude::*;
use dioxus::CapturedError;
use dioxus_showcase::core::StoryDefinition;
use dioxus_showcase::{GeneratedStory, StoryProvider};

use crate::base_path::BasePath;
use crate::shell::{ShellRoot, ShellRootProps};
use crate::state::Shell;

/// Builds a story definition with the given id, title and tags.
pub(crate) fn definition(id: &str, title: &str, tags: &[&str]) -> StoryDefinition {
    StoryDefinition {
        id: id.to_owned(),
        title: title.to_owned(),
        source_path: "src/lib.rs".to_owned(),
        module_path: format!("krate::{}", id.replace('-', "_")),
        renderer_symbol: format!("__dioxus_showcase_render__{}", id.replace('-', "_")),
        tags: tags.iter().map(|tag| (*tag).to_owned()).collect(),
    }
}

/// Builds a story that renders a marker element carrying its own id.
pub(crate) fn story(id: &str, title: &str, tags: &[&str]) -> GeneratedStory {
    let marker = format!("story-body-{id}");
    GeneratedStory {
        definition: definition(id, title, tags),
        render: Box::new(move || {
            rsx! {
                div { class: "{marker}", "rendered {marker}" }
            }
        }),
    }
}

/// Builds a story whose render function fails, the way broken user code does.
pub(crate) fn failing_story(id: &str, title: &str) -> GeneratedStory {
    GeneratedStory {
        definition: definition(id, title, &[]),
        render: Box::new(|| {
            Err(RenderError::Error(CapturedError::from_display("story exploded on purpose")))
        }),
    }
}

/// Assembles a [`Shell`] for a test.
pub(crate) struct ShellBuilder {
    base_path: BasePath,
    title: String,
    stories: Vec<GeneratedStory>,
    duplicate_ids: Vec<String>,
    providers: Vec<StoryProvider>,
}

impl ShellBuilder {
    /// Starts from an empty registry served at the domain root.
    pub(crate) fn new() -> Self {
        Self {
            base_path: BasePath::new("/"),
            title: "Showcase".to_owned(),
            stories: Vec::new(),
            duplicate_ids: Vec::new(),
            providers: Vec::new(),
        }
    }

    /// Serves the shell under a sub-path.
    pub(crate) fn base_path(mut self, raw: &str) -> Self {
        self.base_path = BasePath::new(raw);
        self
    }

    /// Sets the sidebar heading.
    pub(crate) fn title(mut self, title: &str) -> Self {
        self.title = title.to_owned();
        self
    }

    /// Adds one story to the registry.
    pub(crate) fn story(mut self, story: GeneratedStory) -> Self {
        self.stories.push(story);
        self
    }

    /// Reports an id as duplicated, the way `registered_stories` does.
    pub(crate) fn duplicate_id(mut self, id: &str) -> Self {
        self.duplicate_ids.push(id.to_owned());
        self
    }

    /// Adds one provider to the wrap chain.
    pub(crate) fn provider(mut self, provider: StoryProvider) -> Self {
        self.providers.push(provider);
        self
    }

    /// Finalises the shell state.
    pub(crate) fn build(self) -> Shell {
        Shell::new(self.base_path, self.title, self.stories, self.duplicate_ids, self.providers)
    }
}

/// Finds the scope [`ShellRoot`] occupies.
///
/// It is **not** `ScopeId::ROOT`: `VirtualDom` reserves scope 0 for its own
/// `RootScopeWrapper` and mounts the component handed to `new_with_props`
/// somewhere below it, so the shell's contexts are invisible from
/// `ScopeId::ROOT` and its signals cannot be driven from there.
///
/// How many scopes sit in between is a `dioxus-core` implementation detail that
/// has already moved once, so this searches rather than hardcoding: the lowest
/// scope that can see the `Shell` context is the one that provided it, because
/// scope ids are handed out in creation order and a provider is created before
/// its children.
fn shell_scope(dom: &VirtualDom) -> ScopeId {
    dom.in_runtime(|| {
        (0..32)
            .map(ScopeId)
            .find(|scope| consume_context_from_scope::<Shell>(*scope).is_some())
            .expect("the mounted shell root should provide its state to some scope")
    })
}

/// Builds a mounted VirtualDom rendering `shell` at `path`.
///
/// `prefix` is the base path the *history* believes it is serving under, which
/// under `dx` comes from the CLI config rather than from the shell.
pub(crate) fn dom_at(shell: Shell, path: &str, prefix: Option<&str>) -> VirtualDom {
    let mut history = MemoryHistory::with_initial_path(path);
    if let Some(prefix) = prefix {
        history = history.with_prefix(prefix);
    }

    let mut dom = VirtualDom::new_with_props(ShellRoot, ShellRootProps { shell })
        .with_root_context(Rc::new(history) as Rc<dyn History>);
    dom.rebuild_in_place();
    // A story that failed during the build marked its error boundary dirty but
    // did not re-run it. Flushing that pass is what a real renderer does next,
    // and without it a caught error renders as an empty surface.
    dom.render_immediate(&mut NoOpMutations);
    dom
}

/// Applies `change` inside the shell's own scope, then re-renders.
pub(crate) fn update(dom: &mut VirtualDom, change: impl FnOnce()) {
    let scope = shell_scope(dom);
    dom.in_scope(scope, change);
    dom.render_immediate(&mut NoOpMutations);
}

/// Renders `shell` at `path` and returns the HTML.
pub(crate) fn render_at(shell: Shell, path: &str) -> String {
    dioxus_ssr::render(&dom_at(shell, path, None))
}

/// Renders `shell` at `path` with the history serving under `prefix`.
pub(crate) fn render_at_with_prefix(shell: Shell, path: &str, prefix: &str) -> String {
    dioxus_ssr::render(&dom_at(shell, path, Some(prefix)))
}
