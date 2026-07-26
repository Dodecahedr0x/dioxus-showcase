//! Light/dark theme mode and the control that flips it.
//!
//! The mode lives in a `Signal<ThemeMode>` provided by the shell root, and is
//! projected into the DOM as `data-theme` on `.app-shell`. The stylesheet keys
//! every dark override off that attribute, so switching themes is one attribute
//! write and no re-styling work in Rust.
use dioxus::prelude::*;

/// Which of the two shipped themes the shell is currently rendering.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum ThemeMode {
    #[default]
    Light,
    Dark,
}

impl ThemeMode {
    /// Returns the serialized theme token used in the shell DOM.
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Light => "light",
            Self::Dark => "dark",
        }
    }

    /// Switches between light and dark theme modes.
    pub(crate) fn toggle(self) -> Self {
        match self {
            Self::Light => Self::Dark,
            Self::Dark => Self::Light,
        }
    }

    /// Returns the label shown by the theme toggle control.
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Light => "Light",
            Self::Dark => "Dark",
        }
    }
}

/// The sidebar control that flips the shell between light and dark.
#[component]
pub(crate) fn ThemePanel() -> Element {
    let mut theme = use_context::<Signal<ThemeMode>>();
    let active = *theme.read();

    rsx! {
        div { class: "theme-panel",
            h2 { class: "theme-panel-title", "Theme" }
            button {
                class: "theme-toggle",
                onclick: move |_| theme.set(active.toggle()),
                span { class: "theme-toggle-label", "{active.label()}" }
                span { class: "theme-toggle-track",
                    span { class: "theme-toggle-thumb" }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toggling_twice_returns_to_the_starting_mode() {
        assert_eq!(ThemeMode::Light.toggle(), ThemeMode::Dark);
        assert_eq!(ThemeMode::Dark.toggle(), ThemeMode::Light);
        assert_eq!(ThemeMode::default().toggle().toggle(), ThemeMode::default());
    }

    #[test]
    fn each_mode_has_a_distinct_dom_token_and_label() {
        assert_eq!(ThemeMode::Light.as_str(), "light");
        assert_eq!(ThemeMode::Dark.as_str(), "dark");
        assert_eq!(ThemeMode::Light.label(), "Light");
        assert_eq!(ThemeMode::Dark.label(), "Dark");
    }

    /// Renders the panel with the theme signal seeded to `mode`.
    fn panel_html(mode: ThemeMode) -> String {
        #[component]
        fn Harness(mode: ThemeMode) -> Element {
            use_context_provider(|| Signal::new(mode));
            rsx! { ThemePanel {} }
        }

        let mut dom = VirtualDom::new_with_props(Harness, HarnessProps { mode });
        dom.rebuild_in_place();
        dioxus_ssr::render(&dom)
    }

    #[test]
    fn the_panel_shows_the_label_of_the_active_mode() {
        assert!(panel_html(ThemeMode::Light).contains("Light"));
        assert!(panel_html(ThemeMode::Dark).contains("Dark"));
    }
}
