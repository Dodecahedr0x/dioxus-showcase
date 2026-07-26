//! The public entry point: [`ShowcaseApp`].
use dioxus::prelude::*;

use crate::base_path::BasePath;
use crate::diagnostics::DuplicateStoryIds;
use crate::routes::Route;
use crate::state::Shell;
use crate::theme::ThemeMode;

/// The shell's own stylesheet.
///
/// `asset!()` resolves under a bare `cargo test` as well as under `dx` — the
/// unbundled path is the file on disk, the bundled path is
/// `<base-path>/assets/<hashed name>`. Because `manganis` joins the base path in
/// `Asset::resolve()`, this href is correct under `/` and under `/my-repo`
/// without the shell doing anything.
const SHOWCASE_CSS: Asset = asset!("/assets/showcase_app.css");

#[derive(Props, Clone, PartialEq)]
pub struct ShowcaseAppProps {
    /// Base path the site is served under, e.g. `"/"` or `"/my-repo"`.
    /// No trailing slash unless it is exactly `"/"`.
    pub base_path: String,
    /// Sidebar heading, normally the name of the package being showcased.
    /// Defaults to `"Showcase"` when omitted or blank.
    ///
    /// `into` so a generated `main.rs` can pass a plain string literal rather than
    /// `Some("…".to_owned())` — this is a file users read and edit by hand.
    #[props(default, into)]
    pub title: Option<String>,
}

/// The whole showcase application.
///
/// Takes no story list: stories and providers register themselves at link time
/// and are read straight from the registry here. That is what lets a generated
/// showcase `main.rs` be a fixed ten lines that never has to be regenerated.
#[component]
pub fn ShowcaseApp(props: ShowcaseAppProps) -> Element {
    // Read once. The registry is fixed at link time, so re-reading it per render
    // would be wasted work and would hand `ShellRoot` a fresh `Rc` every time.
    let shell = use_hook(|| {
        // A blank title falls back rather than rendering an empty heading: the value
        // comes from `project.name` in the user's config, which nothing forces to be
        // non-empty.
        let title = props
            .title
            .clone()
            .map(|title| title.trim().to_owned())
            .filter(|title| !title.is_empty())
            .unwrap_or_else(|| "Showcase".to_owned());

        Shell::from_registry(BasePath::new(&props.base_path), title)
    });

    rsx! {
        ShellRoot { shell }
    }
}

/// The shell rendered against an explicit state, rather than the global registry.
///
/// [`ShowcaseApp`] is a thin wrapper over this. Splitting them is what makes the
/// shell testable: a test drives `ShellRoot` with any story set it likes, while
/// the link-time registry — which is global, and identical for every test in a
/// binary — stays out of the way.
#[component]
pub(crate) fn ShellRoot(shell: Shell) -> Element {
    let theme = use_context_provider(|| Signal::new(ThemeMode::default()));
    use_context_provider(|| Signal::new(None::<String>));
    // The provider chain, consumed by `StoryPreviewContent` inside each story.
    use_context_provider(|| shell.providers());
    use_context_provider(|| shell.clone());

    let duplicate_ids = shell.duplicate_ids().to_vec();

    rsx! {
        document::Stylesheet { href: SHOWCASE_CSS }
        div { class: "app-shell", "data-theme": theme.read().as_str(),
            DuplicateStoryIds { ids: duplicate_ids }
            Router::<Route> {}
        }
    }
}
