use std::error::Error;
use std::fs;

use crate::cli::install;
use crate::cli::ux;
use crate::core::config;

pub async fn execute(package_name: &str) -> Result<(), Box<dyn Error>> {
    ux::print_header(&format!("Removing package {package_name}"));

    let config_path = "wovenpkg.json";
    let mut config = config::read_config(config_path)?;

    if config.dependencies.remove(package_name).is_some() {
        // Write config back
        let new_json = serde_json::to_string_pretty(&config)?;
        fs::write(config_path, new_json)?;
        ux::print_success(format!("Removed {package_name} from {config_path}"));

        // Use forced resolution after removal to ensure lockfile is consistent
        ux::print_info("Updating environment...");
        install::execute(true).await?;
    } else {
        ux::print_error(format!("Package {package_name} not found in dependencies."));
    }

    Ok(())
}
