use std::path::Path;

use crate::{
    commands::load_config,
    discovery::{discover_components, validate_component_ids},
    scaffold::showcase_app_dir,
};

/// Runs a lightweight validation pass without generating new artifacts.
pub fn cmd_check() -> Result<(), String> {
    let config = load_config()?;
    let mut components = discover_components(Path::new("."), &config)?;
    components.sort_by(|a, b| a.title.cmp(&b.title));
    validate_component_ids(&components)?;

    let app_dir = showcase_app_dir(&config);
    if !app_dir.exists() {
        return Err(format!(
            "showcase app crate not found at {}. Run `dioxus-showcase init` first.",
            app_dir.display()
        ));
    }

    println!("Config file found: DioxusShowcase.toml");
    println!("Showcase crate: {}", app_dir.display());
    println!("Discovered {} annotated components.", components.len());
    println!("Checks passed.");
    Ok(())
}
