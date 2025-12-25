use std::error::Error;
use std::fs;
use std::path::Path;

use crate::core::config;

pub async fn execute() -> Result<(), Box<dyn Error>> {
    let config = config::read_config("wovenpkg.json")?;
    println!(
        "\x1b[1m\x1b[36m🐍 WovenSnake\x1b[0m \x1b[90m| Updating dependencies for {}\x1b[0m\n",
        config.name
    );

    let lock_path = Path::new("wovenpkg.lock");
    if lock_path.exists() {
        println!("\x1b[90mRemoving old lockfile...\x1b[0m");
        fs::remove_file(lock_path)?;
    }

    // Now just run the install command logic which will trigger 'install_and_resolve'
    // since the lockfile is gone.
    crate::cli::install::execute().await?;

    Ok(())
}
