use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShowcaseProjectConfig {
    pub name: String,
    pub entry_crate: String,
    pub showcase_crate: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShowcaseDevConfig {
    pub port: u16,
    pub host: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShowcaseBuildConfig {
    pub out_dir: String,
    pub base_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShowcaseConfig {
    pub project: ShowcaseProjectConfig,
    pub dev: ShowcaseDevConfig,
    pub build: ShowcaseBuildConfig,
}

impl Default for ShowcaseConfig {
    fn default() -> Self {
        Self {
            project: ShowcaseProjectConfig {
                name: "my-ui".to_owned(),
                entry_crate: "web".to_owned(),
                showcase_crate: "showcase".to_owned(),
            },
            dev: ShowcaseDevConfig { port: 6111, host: "127.0.0.1".to_owned() },
            build: ShowcaseBuildConfig {
                out_dir: "target/showcase".to_owned(),
                base_path: "/".to_owned(),
            },
        }
    }
}

impl ShowcaseConfig {
    pub fn as_toml_string(&self) -> String {
        format!(
            "[project]\nname = \"{}\"\nentry_crate = \"{}\"\nshowcase_crate = \"{}\"\n\n[dev]\nport = {}\nhost = \"{}\"\n\n[build]\nout_dir = \"{}\"\nbase_path = \"{}\"\n",
            self.project.name,
            self.project.entry_crate,
            self.project.showcase_crate,
            self.dev.port,
            self.dev.host,
            self.build.out_dir,
            self.build.base_path,
        )
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
        let mut config = Self::default();
        let mut section: Option<&str> = None;

        for (index, raw_line) in content.lines().enumerate() {
            let line_no = index + 1;
            let line = raw_line.trim();

            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if line.starts_with('[') && line.ends_with(']') {
                section = Some(&line[1..line.len() - 1]);
                continue;
            }

            let Some((key, value_raw)) = line.split_once('=') else {
                return Err(format!("invalid TOML assignment at line {line_no}"));
            };

            let key = key.trim();
            let value = value_raw.trim();
            match section {
                Some("project") => match key {
                    "name" => config.project.name = parse_string(value, line_no)?,
                    "entry_crate" => config.project.entry_crate = parse_string(value, line_no)?,
                    "showcase_crate" => {
                        config.project.showcase_crate = parse_string(value, line_no)?
                    }
                    _ => {}
                },
                Some("dev") => match key {
                    "host" => config.dev.host = parse_string(value, line_no)?,
                    "port" => {
                        config.dev.port = value
                            .parse::<u16>()
                            .map_err(|_| format!("invalid dev.port at line {line_no}"))?
                    }
                    _ => {}
                },
                Some("build") => match key {
                    "out_dir" => config.build.out_dir = parse_string(value, line_no)?,
                    "base_path" => config.build.base_path = parse_string(value, line_no)?,
                    _ => {}
                },
                Some(_) | None => {}
            }
        }

        Ok(config)
    }
}

fn parse_string(value: &str, line_no: usize) -> Result<String, String> {
    if !(value.starts_with('"') && value.ends_with('"')) {
        return Err(format!("expected quoted string at line {line_no}"));
    }

    Ok(value[1..value.len() - 1].to_owned())
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
        assert!(err.contains("invalid TOML assignment"));
    }

    #[test]
    fn parse_rejects_unquoted_string() {
        let err =
            ShowcaseConfig::from_toml_str("[project]\nname = demo").expect_err("must be quoted");
        assert!(err.contains("expected quoted string"));
    }

    #[test]
    fn parse_rejects_invalid_port() {
        let err = ShowcaseConfig::from_toml_str("[dev]\nport = 99999").expect_err("invalid port");
        assert!(err.contains("invalid dev.port"));
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
