//! End-to-end tests for the command layer.
//!
//! These drive the real `dioxus-showcase` binary against throwaway workspaces on
//! disk, rather than calling library functions. That is deliberate: the P0 these
//! tests exist for — a user's `showcase/src/main.rs` surviving a build — is a
//! property of the *orchestration* between discovery, scaffolding and generation,
//! and every previous test covered only artifact writing in isolation.
//!
//! Every test gets its own temp directory and passes it as the child process's
//! working directory, so nothing here mutates global state and the suite stays
//! parallel-safe.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

/// The binary under test, built by cargo for this integration test.
const BIN: &str = env!("CARGO_BIN_EXE_dioxus-showcase");

/// A throwaway project on disk, removed when the test finishes.
struct Fixture {
    root: PathBuf,
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

impl Fixture {
    /// Creates a project with a `ui` entry crate and a `DioxusShowcase.toml`.
    fn new(prefix: &str) -> Self {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be monotonic")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("dioxus-showcase-e2e-{prefix}-{unique}"));
        let fixture = Self { root };

        fixture.write(
            "DioxusShowcase.toml",
            "[project]\n\
             name = \"Fixture\"\n\
             entry_crate = \"ui\"\n\
             showcase_crate = \"showcase\"\n\
             \n\
             [dev]\n\
             port = 6111\n\
             host = \"127.0.0.1\"\n\
             \n\
             [build]\n\
             out_dir = \"target/showcase\"\n\
             base_path = \"/\"\n",
        );
        fixture.write(
            "ui/Cargo.toml",
            "[package]\nname = \"fixture-ui\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        );
        fixture.write(
            "ui/src/lib.rs",
            "use dioxus::prelude::*;\n\
             use dioxus_showcase::prelude::*;\n\
             \n\
             #[showcase(title = \"Atoms/Button\", tags = [\"atoms\"])]\n\
             #[component]\n\
             pub fn Button() -> Element { rsx! { button { \"go\" } } }\n",
        );

        fixture
    }

    /// Writes a file, creating any missing parent directories.
    fn write(&self, relative: &str, contents: &str) {
        let path = self.path(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create fixture parent dir");
        }
        fs::write(&path, contents).expect("write fixture file");
    }

    /// Reads a file that a command was expected to produce.
    fn read(&self, relative: &str) -> String {
        fs::read_to_string(self.path(relative))
            .unwrap_or_else(|err| panic!("read {relative}: {err}"))
    }

    /// Resolves a path inside the fixture.
    fn path(&self, relative: &str) -> PathBuf {
        self.root.join(relative)
    }

    /// Runs the CLI in the fixture, feeding it the given stdin.
    fn run_with_stdin(&self, args: &[&str], stdin: &str) -> Run {
        use std::io::Write;

        let mut child = Command::new(BIN)
            .args(args)
            .current_dir(&self.root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn dioxus-showcase");
        child
            .stdin
            .as_mut()
            .expect("stdin is piped")
            .write_all(stdin.as_bytes())
            .expect("write stdin");

        Run::from(args, child.wait_with_output().expect("wait for dioxus-showcase"))
    }

    /// Runs the CLI in the fixture with no stdin.
    fn run(&self, args: &[&str]) -> Run {
        self.run_with_stdin(args, "")
    }

    /// Runs the CLI with an extra directory prepended to `PATH`.
    fn run_with_path_prefix(&self, args: &[&str], prefix: &Path) -> Run {
        let existing = std::env::var("PATH").unwrap_or_default();
        let path = format!("{}:{}", prefix.display(), existing);
        let output = Command::new(BIN)
            .args(args)
            .current_dir(&self.root)
            .env("PATH", path)
            .output()
            .expect("spawn dioxus-showcase");

        Run::from(args, output)
    }
}

/// One finished CLI invocation.
struct Run {
    args: String,
    success: bool,
    stdout: String,
    stderr: String,
}

impl Run {
    fn from(args: &[&str], output: Output) -> Self {
        Self {
            args: args.join(" "),
            success: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        }
    }

    /// Asserts the command exited zero, showing both streams when it did not.
    fn ok(self) -> Self {
        assert!(
            self.success,
            "`dioxus-showcase {}` should succeed\n--- stdout ---\n{}\n--- stderr ---\n{}",
            self.args, self.stdout, self.stderr
        );
        self
    }

    /// Asserts the command exited non-zero and returns its stderr.
    fn failed(self) -> String {
        assert!(
            !self.success,
            "`dioxus-showcase {}` should fail\n--- stdout ---\n{}",
            self.args, self.stdout
        );
        self.stderr
    }
}

// --- init -------------------------------------------------------------------

#[test]
fn init_writes_the_config_and_scaffolds_a_buildable_app() {
    let fixture = Fixture::new("init");
    fs::remove_file(fixture.path("DioxusShowcase.toml")).expect("start without a config");

    fixture
        .run_with_stdin(&["init"], "Fixture\nui\nshowcase\n127.0.0.1\n6111\ntarget/showcase\n")
        .ok();

    let config = fixture.read("DioxusShowcase.toml");
    assert!(config.contains("entry_crate = \"ui\""), "config was:\n{config}");
    assert!(fixture.path("showcase/Cargo.toml").exists());
    assert!(fixture.path("showcase/Dioxus.toml").exists());
    assert!(fixture.path("showcase/src/main.rs").exists());
    assert!(fixture.path("showcase/src/generated.rs").exists());
}

#[test]
fn init_check_build_run_in_sequence_on_a_fresh_project() {
    let fixture = Fixture::new("sequence");
    fs::remove_file(fixture.path("DioxusShowcase.toml")).expect("start without a config");

    fixture
        .run_with_stdin(&["init"], "Fixture\nui\nshowcase\n127.0.0.1\n6111\ntarget/showcase\n")
        .ok();
    let check = fixture.run(&["check"]).ok();
    assert!(check.stdout.contains("Discovered 1 annotated component"), "{}", check.stdout);
    fixture.run(&["build"]).ok();

    assert!(fixture.path("target/showcase/showcase.manifest.json").exists());
    assert!(fixture.path("showcase/src/generated.rs").exists());
}

// --- the P0: main.rs is written once ----------------------------------------

#[test]
fn build_leaves_a_user_edited_main_rs_byte_identical() {
    let fixture = Fixture::new("write-once");
    fixture.run(&["build"]).ok();

    let generated_main = fixture.read("showcase/src/main.rs");
    assert!(generated_main.contains("use showcase_entry as _;"));

    // Exactly what a user does: keep the entry point, add their own code.
    let edited = format!("{generated_main}\n// hand-written by the user\nconst KEEP: u8 = 7;\n");
    fixture.write("showcase/src/main.rs", &edited);

    // A new story changes discovery, so this is not a no-op build.
    fixture.write(
        "ui/src/lib.rs",
        "use dioxus::prelude::*;\n\
         use dioxus_showcase::prelude::*;\n\
         \n\
         #[showcase(title = \"Atoms/Button\")]\n\
         #[component]\n\
         pub fn Button() -> Element { rsx! { button { \"go\" } } }\n\
         \n\
         #[showcase(title = \"Atoms/Badge\")]\n\
         #[component]\n\
         pub fn Badge() -> Element { rsx! { span { \"new\" } } }\n",
    );

    fixture.run(&["build"]).ok();

    assert_eq!(
        fixture.read("showcase/src/main.rs"),
        edited,
        "build must never rewrite an existing showcase main.rs"
    );
    // The rest of the build did happen.
    assert!(fixture.read("target/showcase/showcase.manifest.json").contains("atoms-badge"));
}

#[test]
fn build_writes_main_rs_only_when_it_is_absent() {
    let fixture = Fixture::new("recreate");
    fixture.run(&["build"]).ok();

    fixture.write("showcase/src/main.rs", "// deleted by hand in a moment");
    fs::remove_file(fixture.path("showcase/src/main.rs")).expect("remove main.rs");

    fixture.run(&["build"]).ok();
    assert!(fixture.read("showcase/src/main.rs").contains("dioxus_showcase_ui::ShowcaseApp"));
}

// --- generated.rs no longer names generated symbols -------------------------

#[test]
fn generated_rs_names_no_generated_symbol() {
    let fixture = Fixture::new("generated");
    fixture.run(&["build"]).ok();

    let generated = fixture.read("showcase/src/generated.rs");
    assert!(
        !generated.contains("__dioxus_showcase_"),
        "generated.rs must not name macro-generated symbols, got:\n{generated}"
    );
    assert!(!generated.contains("ShowcaseComponentDefinition"), "{generated}");
    assert!(!generated.contains("showcase_components"), "{generated}");
    assert!(!generated.contains("story_providers"), "{generated}");
    assert!(generated.contains("pub const SHOWCASE_GENERATION"), "{generated}");
}

// --- determinism ------------------------------------------------------------

#[test]
fn a_second_build_reproduces_every_artifact_byte_for_byte() {
    let fixture = Fixture::new("determinism");
    fixture.run(&["build"]).ok();

    let first_generated = fixture.read("showcase/src/generated.rs");
    let first_manifest = fixture.read("target/showcase/showcase.manifest.json");
    let first_main = fixture.read("showcase/src/main.rs");
    let first_dioxus_toml = fixture.read("showcase/Dioxus.toml");

    fixture.run(&["build"]).ok();

    assert_eq!(fixture.read("showcase/src/generated.rs"), first_generated);
    assert_eq!(fixture.read("target/showcase/showcase.manifest.json"), first_manifest);
    assert_eq!(fixture.read("showcase/src/main.rs"), first_main);
    assert_eq!(fixture.read("showcase/Dioxus.toml"), first_dioxus_toml);
}

// --- the manifest is advisory, and versioned ---------------------------------

#[test]
fn the_manifest_declares_schema_version_two() {
    let fixture = Fixture::new("schema");
    fixture.run(&["build"]).ok();

    let manifest = fixture.read("target/showcase/showcase.manifest.json");
    assert!(manifest.contains("\"schema_version\":2"), "{manifest}");
    // `renderer_symbol` is retained, documented as advisory.
    assert!(manifest.contains("\"renderer_symbol\":\"__dioxus_showcase_render__Button\""));
}

// --- user stylesheets survive into the generated entry point -----------------

#[test]
fn main_rs_links_every_stylesheet_found_in_the_entry_crate() {
    let fixture = Fixture::new("stylesheets");
    fixture.write("ui/assets/app.css", "body { color: red; }");
    fixture.write("ui/assets/styles/tailwind.css", ".btn { display: flex; }");

    fixture.run(&["build"]).ok();

    let main_rs = fixture.read("showcase/src/main.rs");
    assert!(
        main_rs.contains("document::Stylesheet { href: asset!(\"/assets/app.css\") }"),
        "{main_rs}"
    );
    assert!(
        main_rs.contains("document::Stylesheet { href: asset!(\"/assets/styles/tailwind.css\") }"),
        "{main_rs}"
    );
    assert!(fixture.path("showcase/assets/app.css").exists());
    assert!(fixture.path("showcase/assets/styles/tailwind.css").exists());
    // The shell's own stylesheet ships inside dioxus-showcase-ui now, so the CLI
    // must not drop a second copy of it into the user's assets directory.
    assert!(!fixture.path("showcase/assets/showcase.css").exists());
}

#[test]
fn a_stale_generated_showcase_css_is_cleaned_up_on_build() {
    let fixture = Fixture::new("stale-css");
    // What an upgrading user has on disk: the stylesheet older CLI versions wrote.
    fixture.write("showcase/assets/showcase.css", ":root { --old: 1; }");

    fixture.run(&["build"]).ok();

    assert!(!fixture.path("showcase/assets/showcase.css").exists());
    assert!(!fixture.read("showcase/src/main.rs").contains("/assets/showcase.css"));
}

#[test]
fn a_user_owned_showcase_css_in_the_entry_crate_is_kept() {
    let fixture = Fixture::new("owned-css");
    fixture.write("ui/assets/showcase.css", ".mine { color: blue; }");

    fixture.run(&["build"]).ok();

    assert_eq!(fixture.read("showcase/assets/showcase.css"), ".mine { color: blue; }");
    assert!(fixture.read("showcase/src/main.rs").contains("asset!(\"/assets/showcase.css\")"));
}

// --- the scaffolded manifest ------------------------------------------------

#[test]
fn the_scaffolded_cargo_toml_wires_the_ui_crate_and_keeps_registrations_linked() {
    let fixture = Fixture::new("cargo-toml");
    fixture.run(&["build"]).ok();

    let cargo_toml = fixture.read("showcase/Cargo.toml");
    assert!(cargo_toml.contains("dioxus-showcase-ui"), "{cargo_toml}");
    // Without LTO the component crate's rlib member is never pulled into the wasm
    // link, and every story silently vanishes. Both profiles need it: `dx serve`
    // builds through `dev` and `dx bundle --release` through `release`.
    assert!(cargo_toml.contains("[profile.dev]"), "{cargo_toml}");
    assert!(cargo_toml.contains("lto = \"thin\""), "{cargo_toml}");
    assert!(cargo_toml.contains("[profile.release]"), "{cargo_toml}");
    assert!(cargo_toml.contains("lto = true"), "{cargo_toml}");
    // dx's vendored wasm-opt aborts on a release profile carrying DWARF and then
    // silently ships unoptimised wasm, so the profile has to strip.
    assert!(cargo_toml.contains("strip = true"), "{cargo_toml}");
}

// --- check ------------------------------------------------------------------

#[test]
fn check_reports_every_duplicate_story_id() {
    let fixture = Fixture::new("duplicates");
    fixture.write(
        "ui/src/lib.rs",
        "use dioxus::prelude::*;\n\
         use dioxus_showcase::prelude::*;\n\
         \n\
         #[showcase(title = \"Atoms/Button\")]\n\
         #[component]\n\
         pub fn Button() -> Element { rsx! { button {} } }\n\
         \n\
         #[story(title = \"Atoms/Button\")]\n\
         pub fn also_button() -> Element { rsx! { button {} } }\n",
    );
    // Scaffold first, so the failure below can only be about the colliding ids.
    fixture.run(&["build"]).ok();

    let stderr = fixture.run(&["check"]).failed();
    assert!(stderr.contains("duplicate showcase id(s) found"), "{stderr}");
    assert!(stderr.contains("duplicate id 'atoms-button'"), "{stderr}");
    // Both claimants are named, because the id alone is not actionable.
    assert!(stderr.contains("ui::Button") || stderr.contains("Button"), "{stderr}");
    assert!(stderr.contains("also_button"), "{stderr}");
}

#[test]
fn check_reports_an_unknown_config_key() {
    let fixture = Fixture::new("unknown-key");
    fixture.write(
        "DioxusShowcase.toml",
        "[project]\nname = \"Fixture\"\nentry_crate = \"ui\"\nshocase_crate = \"showcase\"\n",
    );

    let stderr = fixture.run(&["check"]).failed();
    assert!(stderr.contains("shocase_crate"), "{stderr}");
}

#[test]
fn check_does_not_compile_the_entry_crate() {
    let fixture = Fixture::new("no-compile");
    // Valid Rust *syntax* that could never compile: unresolved types and calls.
    fixture.write(
        "ui/src/lib.rs",
        "use dioxus::prelude::*;\n\
         use dioxus_showcase::prelude::*;\n\
         \n\
         #[showcase(title = \"Atoms/Button\")]\n\
         #[component]\n\
         pub fn Button() -> Element { totally_undefined_function(NotARealType) }\n",
    );
    // There is not even a lockfile or a src/main.rs to compile against.
    fixture.run(&["build"]).ok();
    let check = fixture.run(&["check"]).ok();

    assert!(check.stdout.contains("Discovered 1 annotated component"), "{}", check.stdout);
    // Compiling anything would have created a target directory for the entry crate.
    assert!(!fixture.path("ui/target").exists());
    assert!(!fixture.path("showcase/target").exists());
}

// --- discovery is advisory: drift degrades diagnostics, never fails a build ---

#[test]
fn build_survives_a_source_file_discovery_cannot_parse() {
    let fixture = Fixture::new("drift");
    fixture.write(
        "ui/src/lib.rs",
        "use dioxus::prelude::*;\n\
         use dioxus_showcase::prelude::*;\n\
         \n\
         pub mod broken;\n\
         \n\
         #[showcase(title = \"Atoms/Button\")]\n\
         #[component]\n\
         pub fn Button() -> Element { rsx! { button {} } }\n",
    );
    fixture.write("ui/src/broken.rs", "fn ( this is not rust {{{\n");

    let build = fixture.run(&["build"]).ok();
    assert!(build.stderr.contains("warning"), "stderr was:\n{}", build.stderr);
    assert!(fixture.path("showcase/src/generated.rs").exists());
    assert!(fixture.path("showcase/src/main.rs").exists());

    // `check` is the diagnostic surface, so there the same drift is an error.
    let stderr = fixture.run(&["check"]).failed();
    assert!(stderr.contains("broken.rs"), "{stderr}");
}

#[test]
fn build_does_not_fail_on_duplicate_story_ids() {
    let fixture = Fixture::new("duplicate-build");
    fixture.write(
        "ui/src/lib.rs",
        "use dioxus::prelude::*;\n\
         use dioxus_showcase::prelude::*;\n\
         \n\
         #[showcase(title = \"Atoms/Button\")]\n\
         #[component]\n\
         pub fn Button() -> Element { rsx! { button {} } }\n\
         \n\
         #[story(title = \"Atoms/Button\")]\n\
         pub fn also_button() -> Element { rsx! { button {} } }\n",
    );

    // The shell renders a banner for colliding ids; it is not a build failure.
    fixture.run(&["build"]).ok();
    assert!(fixture.path("showcase/src/generated.rs").exists());
}

// --- provider ordering ------------------------------------------------------

#[test]
fn providers_are_ordered_by_the_order_key() {
    let fixture = Fixture::new("provider-order");
    fixture.write(
        "ui/src/lib.rs",
        "use dioxus::prelude::*;\n\
         use dioxus_showcase::prelude::*;\n\
         \n\
         #[provider(order = -10)]\n\
         #[component]\n\
         pub fn Outer(children: Element) -> Element { children }\n",
    );

    fixture.run(&["build"]).ok();
    let check = fixture.run(&["check"]).ok();
    assert!(check.success);
}

#[test]
fn the_retired_provider_index_key_is_rejected() {
    let fixture = Fixture::new("provider-index");
    fixture.write(
        "ui/src/lib.rs",
        "use dioxus::prelude::*;\n\
         use dioxus_showcase::prelude::*;\n\
         \n\
         #[provider(index = 0)]\n\
         #[component]\n\
         pub fn Outer(children: Element) -> Element { children }\n",
    );

    let stderr = fixture.run(&["check"]).failed();
    assert!(stderr.contains("order = <integer>"), "{stderr}");
}

// --- export -----------------------------------------------------------------

/// `export` shells out to `dx bundle`, which takes minutes and needs a wasm
/// toolchain. Standing in a fake `dx` on `PATH` keeps the test hermetic while
/// still exercising everything `export` itself owns: regenerating artifacts,
/// staging, flattening the bundle, the SPA fallback and asset pruning. The real
/// `dx` is covered by the CI export job.
#[cfg(unix)]
#[test]
fn export_flattens_a_bundle_into_a_static_site() {
    use std::os::unix::fs::PermissionsExt;

    let fixture = Fixture::new("export");
    let bin_dir = fixture.path("fake-bin");
    fs::create_dir_all(&bin_dir).expect("create fake bin dir");

    let dx = bin_dir.join("dx");
    fs::write(
        &dx,
        "#!/bin/sh\n\
         out=\"\"\n\
         while [ $# -gt 0 ]; do\n\
         \x20 case \"$1\" in\n\
         \x20   --out-dir) out=\"$2\"; shift 2 ;;\n\
         \x20   *) shift ;;\n\
         \x20 esac\n\
         done\n\
         mkdir -p \"$out/public/assets\"\n\
         printf '<html><script src=\"/assets/app-dxh1.js\"></script></html>' > \"$out/public/index.html\"\n\
         printf 'wasm-glue' > \"$out/public/assets/app-dxh1.js\"\n\
         printf 'left-over' > \"$out/public/assets/app-dxh9.js\"\n",
    )
    .expect("write fake dx");
    fs::set_permissions(&dx, fs::Permissions::from_mode(0o755)).expect("chmod fake dx");

    fixture.run_with_path_prefix(&["export"], &bin_dir).ok();

    let site = fixture.path("target/showcase/site");
    assert!(site.join("index.html").exists());
    assert_eq!(
        fs::read_to_string(site.join("404.html")).expect("read fallback"),
        fs::read_to_string(site.join("index.html")).expect("read index"),
    );
    assert!(site.join(".nojekyll").exists());
    assert!(site.join("assets/app-dxh1.js").exists());
    assert!(!site.join("assets/app-dxh9.js").exists(), "stale assets should be pruned");
    assert!(!site.join(".dx-bundle").exists(), "the staging dir should be removed");
}

#[cfg(unix)]
#[test]
fn export_regenerates_artifacts_before_bundling() {
    use std::os::unix::fs::PermissionsExt;

    let fixture = Fixture::new("export-regen");
    let bin_dir = fixture.path("fake-bin");
    fs::create_dir_all(&bin_dir).expect("create fake bin dir");

    let dx = bin_dir.join("dx");
    fs::write(
        &dx,
        "#!/bin/sh\n\
         out=\"\"\n\
         while [ $# -gt 0 ]; do\n\
         \x20 case \"$1\" in\n\
         \x20   --out-dir) out=\"$2\"; shift 2 ;;\n\
         \x20   *) shift ;;\n\
         \x20 esac\n\
         done\n\
         mkdir -p \"$out/public\"\n\
         printf '<html></html>' > \"$out/public/index.html\"\n",
    )
    .expect("write fake dx");
    fs::set_permissions(&dx, fs::Permissions::from_mode(0o755)).expect("chmod fake dx");

    fixture.run_with_path_prefix(&["export"], &bin_dir).ok();

    assert!(fixture.read("target/showcase/showcase.manifest.json").contains("atoms-button"));
    assert!(fixture.path("showcase/src/main.rs").exists());
}

#[test]
fn export_reports_a_missing_dioxus_cli_clearly() {
    let fixture = Fixture::new("export-no-dx");
    let empty_bin = fixture.path("empty-bin");
    fs::create_dir_all(&empty_bin).expect("create empty bin dir");

    let stderr = {
        let existing_path = std::env::var("PATH").unwrap_or_default();
        let _ = existing_path;
        let output = Command::new(BIN)
            .arg("export")
            .current_dir(&fixture.root)
            .env("PATH", empty_bin.display().to_string())
            .output()
            .expect("spawn dioxus-showcase");
        assert!(!output.status.success(), "export should fail without dx on PATH");
        String::from_utf8_lossy(&output.stderr).into_owned()
    };

    assert!(stderr.contains("Dioxus CLI"), "{stderr}");
}

#[test]
fn check_warns_when_the_showcase_manifest_drops_lto() {
    let fixture = Fixture::new("lto-guard");
    fixture.run(&["build"]).ok();

    // Clean scaffolds carry the profiles, so nothing is reported.
    let clean = fixture.run(&["check"]).ok();
    assert!(!clean.stderr.contains("does not set `lto`"), "{}", clean.stderr);

    // A user editing their own Cargo.toml is the realistic way this breaks. Without
    // LTO the component crate's registrations never reach the wasm binary and the
    // showcase renders empty with no error at all, so `check` has to say so.
    let manifest = fixture.read("showcase/Cargo.toml");
    let stripped: String = manifest
        .lines()
        .filter(|line| !line.trim_start().starts_with("lto"))
        .collect::<Vec<_>>()
        .join("\n");
    fixture.write("showcase/Cargo.toml", &stripped);

    let warned = fixture.run(&["check"]).ok();
    assert!(warned.stderr.contains("[profile.dev]"), "{}", warned.stderr);
    assert!(warned.stderr.contains("[profile.release]"), "{}", warned.stderr);
    assert!(warned.stderr.contains("renders EMPTY"), "{}", warned.stderr);
}
