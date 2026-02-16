use std::error::Error;

use crate::cli::ux;
use crate::core::config;

pub async fn execute() -> Result<(), Box<dyn Error>> {
    let config = config::read_config("wovenpkg.json")?;
    ux::print_header(&format!("Updating dependencies for {}", config.name));

    // Use forced resolution for update
    crate::cli::install::execute(true).await?;

    Ok(())
}
