use std::error::Error;
use std::fs;
use std::path::Path;

use crate::cli::install;
use crate::core::config;
use crate::dependencies::package;

pub async fn execute(name: &str, version: Option<String>) -> Result<(), Box<dyn Error>> {
    println!("\x1b[1m\x1b[36m🐍 WovenSnake\x1b[0m \x1b[90m| Adding package {name}\x1b[0m\n");

    let config_path = "wovenpkg.json";
    let mut config = config::read_config(config_path)?;

    // Check if already exists
    if config.dependencies.contains_key(name) {
        println!("Package {name} is already in dependencies.");
        return Ok(());
    }

    // Fetch package info to validate and get version
    let info = package::fetch_package_info(name, version.as_deref()).await?;
    let resolved_version = info.info.version;

    println!("Resolved {name} to version {resolved_version}");

    // Update config
    config.dependencies.insert(name.to_string(), resolved_version);
    let new_json = serde_json::to_string_pretty(&config)?;
    fs::write(config_path, new_json)?;

    println!("Updated {config_path}");

    // Strategy: Delete lockfile, run install.
    let lock_path = Path::new("wovenpkg.lock");
    if lock_path.exists() {
        fs::remove_file(lock_path)?;
    }

    println!("Updating environment...");
    install::execute().await?;

    Ok(())
}
