use std::error::Error;
use std::fs;
use std::path::Path;

use crate::cli::ux;
use crate::core::config;

pub async fn execute() -> Result<(), Box<dyn Error>> {
    let config = config::read_config("wovenpkg.json")?;
    ux::print_header(&format!("Updating dependencies for {}", config.name));

    let lock_path = Path::new("wovenpkg.lock");
    if lock_path.exists() {
        ux::print_info("Removing old lockfile...");
        fs::remove_file(lock_path)?;
    }

    // Now just run the install command logic which will trigger 'install_and_resolve'
    // since the lockfile is gone.
    crate::cli::install::execute().await?;

    Ok(())
}
