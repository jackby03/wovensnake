use std::error::Error;
use std::fs;
use std::path::Path;

use crate::cli::install;
use crate::core::config;

pub async fn execute(package_name: &str) -> Result<(), Box<dyn Error>> {
    println!(
        "\x1b[1m\x1b[36m🐍 WovenSnake\x1b[0m \x1b[90m| Removing package {package_name}\x1b[0m\n"
    );

    let config_path = "wovenpkg.json";
    let mut config = config::read_config(config_path)?;

    if config.dependencies.remove(package_name).is_some() {
        // Write config back
        let new_json = serde_json::to_string_pretty(&config)?;
        fs::write(config_path, new_json)?;
        println!("Removed {package_name} from {config_path}");

        // Strategy: Delete lockfile, run install.
        let lock_path = Path::new("wovenpkg.lock");
        if lock_path.exists() {
            fs::remove_file(lock_path)?;
        }

        println!("Updating environment...");
        install::execute().await?;
    } else {
        println!("Package {package_name} not found in dependencies.");
    }

    Ok(())
}
