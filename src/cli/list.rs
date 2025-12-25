use std::error::Error;
use std::path::Path;

use crate::cli::ux;
use crate::core::config;
use crate::core::lock::Lockfile;
use crate::core::python_manager;

pub fn execute() -> Result<(), Box<dyn Error>> {
    let config = config::read_config("wovenpkg.json")?;
    ux::print_header(&format!("Project: {} v{}", config.name, config.version));
    println!(" \x1b[36mPython Version:\x1b[0m {}", config.python_version);

    let lock_path = Path::new("wovenpkg.lock");
    if lock_path.exists() {
        let lockfile = Lockfile::read(lock_path)?;
        ux::print_info(format!("Installed Packages ({})", lockfile.packages.len()));

        // Sort for nice output
        let mut pkgs: Vec<_> = lockfile.packages.iter().collect();
        pkgs.sort_by_key(|k| k.0);

        for (name, pkg) in pkgs {
            println!(" \x1b[32m•\x1b[0m {} \x1b[90mv{}\x1b[0m", name, pkg.version);
        }
    } else {
        ux::print_warning("No lockfile found. Run 'woven install' to weave dependencies.");
        ux::print_info("Declared Dependencies:");
        for (name, ver) in config.dependencies {
            println!(" - {name} {ver}");
        }
    }

    // Show managed Python versions
    if let Ok(managed) = python_manager::list_managed_versions() {
        if !managed.is_empty() {
            ux::print_info("Managed Python Versions:");
            for ver in managed {
                println!(" \x1b[34m•\x1b[0m {ver}");
            }
        }
    }

    Ok(())
}
