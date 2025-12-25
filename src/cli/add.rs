use std::error::Error;
use std::fs;
use std::path::Path;

use crate::cli::install;
use crate::cli::ux;
use crate::core::config;
use crate::dependencies::package;

pub async fn execute(name: &str, version: Option<String>) -> Result<(), Box<dyn Error>> {
    ux::print_header(&format!("Adding package {name}"));

    let config_path = "wovenpkg.json";
    let mut config = config::read_config(config_path)?;

    // Check if already exists
    if config.dependencies.contains_key(name) {
        ux::print_warning(format!("Package {name} is already in dependencies."));
        return Ok(());
    }

    // Fetch package info to validate and get version
    let info = package::fetch_package_info(name, version.as_deref()).await?;
    let resolved_version = info.info.version;

    ux::print_info(format!("Resolved {name} to version {resolved_version}"));

    // Update config
    config.dependencies.insert(name.to_string(), resolved_version);
    let new_json = serde_json::to_string_pretty(&config)?;
    fs::write(config_path, new_json)?;

    ux::print_success(format!("Updated {config_path}"));

    // Strategy: Delete lockfile, run install.
    let lock_path = Path::new("wovenpkg.lock");
    if lock_path.exists() {
        ux::print_info("Removing old lockfile for re-resolution...");
        fs::remove_file(lock_path)?;
    }

    ux::print_info("Updating environment...");
    install::execute().await?;

    Ok(())
}
