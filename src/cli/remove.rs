use std::error::Error;
use std::fs;
use crate::core::config;
use crate::cli::install;

pub async fn execute(package_name: &str) -> Result<(), Box<dyn Error>> {
    println!("\x1b[1m\x1b[36m🐍 WovenSnake\x1b[0m \x1b[90m| Removing package {}\x1b[0m\n", package_name);

    let config_path = "wovenpkg.json";
    let mut config = config::read_config(config_path)?;

    if config.dependencies.remove(package_name).is_some() {
        // Write config back
        let new_json = serde_json::to_string_pretty(&config)?;
        fs::write(config_path, new_json)?;
        println!("Removed {} from {}", package_name, config_path);

        // Remove lockfile to force recalculation? 
        // Or just run install logic which should prune it?
        // Our install logic in 'prune_unused_packages' checks against the Lockfile.
        // But the Lockfile is generated from Config in `resolve_and_install_final`.
        // So valid flow:
        // 1. Update Config (done)
        // 2. Resolve (weave) new tree from Config -> Updates Lockfile -> Prunes packages not in Lockfile.
        
        // However, `install::execute` branches: if lockfile exists, it does install_from_lock.
        // It WON'T re-resolve just because config changed unless we force it or delete lockfile.
        // SO: We must regenerate the lockfile.

        // Strategy: Delete lockfile, run install.
        let lock_path = std::path::Path::new("wovenpkg.lock");
        if lock_path.exists() {
            fs::remove_file(lock_path)?;
        }

        println!("Updating environment...");
        install::execute().await?;
    } else {
        println!("Package {} not found in dependencies.", package_name);
    }

    Ok(())
}
