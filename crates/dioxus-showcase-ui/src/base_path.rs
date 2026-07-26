//! Normalisation of the public sub-path the showcase site is served under.
//!
//! Two different mechanisms already prefix URLs with the base path at runtime,
//! and neither of them is the shell:
//!
//! - **Navigation.** `dioxus_history::WebHistory` reads
//!   `dioxus_cli_config::web_base_path()`, so every `Link` the router renders is
//!   prefixed for free.
//! - **Assets.** `manganis`' `Asset::resolve()` reads `dioxus_cli_config::base_path()`
//!   and joins it in front of `/assets/`.
//!
//! So the shell must **not** prefix routes or asset hrefs itself — doing so would
//! apply the prefix twice. What the shell does own is the URLs it *displays* to
//! the user, which have no runtime machinery behind them. [`BasePath`] is what
//! makes those come out right.

/// A normalised base path: either empty (served from the domain root) or a
/// leading-slash path with no trailing slash, e.g. `/my-repo`.
///
/// The normalisation deliberately matches `normalize_base_path` in the CLI's
/// `export.rs`, so a value that round-trips through `DioxusShowcase.toml`,
/// `Dioxus.toml` and `dx bundle --base-path` means the same thing here.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct BasePath(String);

impl BasePath {
    /// Normalises a raw configured base path such as `"/"`, `""` or `"/my-repo/"`.
    pub(crate) fn new(raw: &str) -> Self {
        let trimmed = raw.trim().trim_matches('/');
        if trimmed.is_empty() {
            Self(String::new())
        } else {
            Self(format!("/{trimmed}"))
        }
    }

    /// Returns `true` when the site is served straight from the domain root.
    pub(crate) fn is_root(&self) -> bool {
        self.0.is_empty()
    }

    /// Joins an in-app absolute path onto the base path.
    ///
    /// `join("/component/x")` is `/component/x` at the root and
    /// `/my-repo/component/x` under `/my-repo`. Joining nothing yields the site
    /// root itself, which is `/` rather than the empty string.
    pub(crate) fn join(&self, path: &str) -> String {
        let path = path.trim_start_matches('/');
        if path.is_empty() {
            if self.is_root() {
                "/".to_owned()
            } else {
                self.0.clone()
            }
        } else {
            format!("{}/{path}", self.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn root_spellings_all_normalise_to_the_empty_base() {
        for raw in ["/", "", "   ", "//", " / "] {
            assert!(BasePath::new(raw).is_root(), "{raw:?} should be the root base path");
            assert_eq!(BasePath::new(raw).join("/"), "/");
        }
    }

    #[test]
    fn a_sub_path_gains_a_leading_slash_and_loses_its_trailing_one() {
        assert_eq!(BasePath::new("/my-repo").join(""), "/my-repo");
        assert_eq!(BasePath::new("my-repo/").join(""), "/my-repo");
        assert_eq!(BasePath::new("/my-repo/").join(""), "/my-repo");
        assert_eq!(BasePath::new("  /my-repo/  ").join(""), "/my-repo");
    }

    #[test]
    fn nested_sub_paths_keep_their_inner_separators() {
        assert_eq!(BasePath::new("/nested/path/").join("/component/x"), "/nested/path/component/x");
    }

    #[test]
    fn join_produces_single_slash_boundaries_at_the_root() {
        let root = BasePath::new("/");
        assert_eq!(root.join("/component/x"), "/component/x");
        assert_eq!(root.join("component/x"), "/component/x");
    }

    #[test]
    fn join_prefixes_every_in_app_path_under_a_sub_path() {
        let repo = BasePath::new("/repo");
        assert_eq!(repo.join("/component/x"), "/repo/component/x");
        assert_eq!(repo.join("component/x"), "/repo/component/x");
        assert!(!repo.is_root());
    }
}
