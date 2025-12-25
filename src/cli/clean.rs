use std::error::Error;
use std::fs;
use std::path::Path;

use crate::cli::ux;
use crate::core::cache::Cache;
use crate::core::config;
use crate::core::python_manager;

pub fn execute(all: bool, python: bool) -> Result<(), Box<dyn Error>> {
    ux::print_header("Cleaning project environment...");

    // 1. Read config to find venv
    let config = config::read_config("wovenpkg.json")?;
    let venv_path = Path::new(&config.virtual_environment);

    // 2. Remove venv
    if venv_path.exists() {
        ux::print_info(format!("Removing virtual environment: {}", venv_path.display()));
        fs::remove_dir_all(venv_path)?;
    }

    // 3. Remove packages directory
    let packages_dir = Path::new("packages");
    if packages_dir.exists() {
        ux::print_info("Removing local packages directory...");
        fs::remove_dir_all(packages_dir)?;
    }

    // 4. Remove lockfile
    let lock_path = Path::new("wovenpkg.lock");
    if lock_path.exists() {
        ux::print_info("Removing lockfile...");
        fs::remove_file(lock_path)?;
    }

    // 5. Optional: Clear global cache
    if all {
        ux::print_info("Clearing global cache...");
        let cache = Cache::init()?;
        cache.clear()?;
    }

    // 6. Optional: Clear managed Python versions
    if python {
        ux::print_info("Clearing managed Python versions...");
        python_manager::clear_managed_versions()?;
    }

    ux::print_success("Project cleaned successfully.");
    Ok(())
}
