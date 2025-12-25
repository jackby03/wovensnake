use std::error::Error;
use std::path::Path;

use crate::core::config;
use crate::core::lock::Lockfile;

pub fn execute() -> Result<(), Box<dyn Error>> {
    let config = config::read_config("wovenpkg.json")?;
    println!("\x1b[1m\x1b[36m🐍 Project: {} v{}\x1b[0m", config.name, config.version);

    let lock_path = Path::new("wovenpkg.lock");
    if lock_path.exists() {
        let lockfile = Lockfile::read(lock_path)?;
        println!("\x1b[90mInstalled Packages ({})\x1b[0m", lockfile.packages.len());

        // Sort for nice output
        let mut pkgs: Vec<_> = lockfile.packages.iter().collect();
        pkgs.sort_by_key(|k| k.0);

        for (name, pkg) in pkgs {
            println!(" \x1b[32m•\x1b[0m {} \x1b[90mv{}\x1b[0m", name, pkg.version);
        }
    } else {
        println!("No lockfile found. Run 'woven install' to weave dependencies.");
        println!("\x1b[90mDeclared Dependencies:\x1b[0m");
        for (name, ver) in config.dependencies {
            println!(" - {name} {ver}");
        }
    }

    Ok(())
}
