use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ShowcaseProjectConfig {
    pub name: String,
    pub entry_crate: String,
    pub showcase_crate: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ShowcaseDevConfig {
    pub port: u16,
    pub host: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ShowcaseBuildConfig {
    pub out_dir: String,
    pub base_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ShowcaseConfig {
    pub project: ShowcaseProjectConfig,
    pub dev: ShowcaseDevConfig,
    pub build: ShowcaseBuildConfig,
}

impl Default for ShowcaseProjectConfig {
    fn default() -> Self {
        Self {
            name: "my-ui".to_owned(),
            entry_crate: "web".to_owned(),
            showcase_crate: "showcase".to_owned(),
        }
    }
}

impl Default for ShowcaseDevConfig {
    fn default() -> Self {
        Self { port: 6111, host: "127.0.0.1".to_owned() }
    }
}

impl Default for ShowcaseBuildConfig {
    fn default() -> Self {
        Self { out_dir: "target/showcase".to_owned(), base_path: "/".to_owned() }
    }
}

impl Default for ShowcaseConfig {
    fn default() -> Self {
        Self {
            project: ShowcaseProjectConfig::default(),
            dev: ShowcaseDevConfig::default(),
            build: ShowcaseBuildConfig::default(),
        }
    }
}

impl ShowcaseConfig {
    pub fn as_toml_string(&self) -> String {
        toml::to_string_pretty(self).expect("showcase config serialization should not fail")
    }

    pub fn write_default_if_missing(path: impl AsRef<Path>) -> std::io::Result<bool> {
        let path = path.as_ref();
        if path.exists() {
            return Ok(false);
        }

        std::fs::write(path, Self::default().as_toml_string())?;
        Ok(true)
    }

    pub fn from_toml_file(path: impl AsRef<Path>) -> Result<Self, String> {
        let content = std::fs::read_to_string(path.as_ref())
            .map_err(|err| format!("failed to read {}: {err}", path.as_ref().display()))?;
        Self::from_toml_str(&content)
    }

    pub fn from_toml_str(content: &str) -> Result<Self, String> {
        toml::from_str(content).map_err(|err| format!("failed to parse showcase config: {err}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn default_round_trips_through_toml() {
        let config = ShowcaseConfig::default();
        let parsed = ShowcaseConfig::from_toml_str(&config.as_toml_string()).expect("valid toml");
        assert_eq!(parsed, config);
    }

    #[test]
    fn parse_custom_values() {
        let content = r#"
[project]
name = "demo"
entry_crate = "client"
showcase_crate = "ui-showcase"

[dev]
port = 7000
host = "0.0.0.0"

[build]
out_dir = "dist/showcase"
base_path = "/showcase"
"#;
        let parsed = ShowcaseConfig::from_toml_str(content).expect("should parse");
        assert_eq!(parsed.project.name, "demo");
        assert_eq!(parsed.project.entry_crate, "client");
        assert_eq!(parsed.project.showcase_crate, "ui-showcase");
        assert_eq!(parsed.dev.port, 7000);
        assert_eq!(parsed.dev.host, "0.0.0.0");
        assert_eq!(parsed.build.out_dir, "dist/showcase");
        assert_eq!(parsed.build.base_path, "/showcase");
    }

    #[test]
    fn parse_rejects_invalid_assignment() {
        let err = ShowcaseConfig::from_toml_str("[project]\nname \"demo\"")
            .expect_err("missing = should fail");
        assert!(err.contains("failed to parse showcase config"));
    }

    #[test]
    fn parse_rejects_unquoted_string() {
        let err =
            ShowcaseConfig::from_toml_str("[project]\nname = demo").expect_err("must be quoted");
        assert!(err.contains("failed to parse showcase config"));
    }

    #[test]
    fn parse_rejects_invalid_port() {
        let err = ShowcaseConfig::from_toml_str("[dev]\nport = 99999").expect_err("invalid port");
        assert!(err.contains("failed to parse showcase config"));
    }

    #[test]
    fn parse_rejects_unknown_fields() {
        let err = ShowcaseConfig::from_toml_str("[project]\nunknown = \"value\"")
            .expect_err("unknown fields should fail");
        assert!(err.contains("failed to parse showcase config"));
    }

    #[test]
    fn parse_fills_missing_sections_from_defaults() {
        let parsed = ShowcaseConfig::from_toml_str("[project]\nname = \"demo\"")
            .expect("partial config should parse");

        assert_eq!(parsed.project.name, "demo");
        assert_eq!(parsed.project.entry_crate, "web");
        assert_eq!(parsed.project.showcase_crate, "showcase");
        assert_eq!(parsed.dev, ShowcaseDevConfig::default());
        assert_eq!(parsed.build, ShowcaseBuildConfig::default());
    }

    #[test]
    fn write_default_if_missing_only_writes_once() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be monotonic")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("dioxus-showcase-config-test-{unique}"));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        let path = dir.join("DioxusShowcase.toml");

        let first = ShowcaseConfig::write_default_if_missing(&path).expect("first write");
        let second = ShowcaseConfig::write_default_if_missing(&path).expect("second write");
        let written = std::fs::read_to_string(&path).expect("read config");

        assert!(first);
        assert!(!second);
        assert!(written.contains("[project]"));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
