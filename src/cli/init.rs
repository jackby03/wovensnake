use crate::cli::ux;
use crate::core::config::Config;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn execute() -> Result<(), Box<dyn std::error::Error>> {
    ux::print_header("Initializing Project");
    let path = Path::new("wovenpkg.json");

    if path.exists() {
        ux::print_warning("wovenpkg.json already exists in this directory.");
        return Ok(());
    }

    let default_config = Config {
        name: "my-python-project".to_string(),
        version: "0.1.0".to_string(),
        python_version: "3.10".to_string(),
        dependencies: HashMap::new(),
        virtual_environment: ".venv".to_string(),
    };

    let json = serde_json::to_string_pretty(&default_config)?;
    fs::write(path, json)?;

    ux::print_success("Successfully initialized WovenSnake project: wovenpkg.json created.");
    Ok(())
}
