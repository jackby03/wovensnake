use crate::cli::ux;
use crate::core::config;
use crate::dependencies::package;

pub async fn execute() -> anyhow::Result<()> {
    let mut config = config::read_config("wovenpkg.json")?;
    ux::print_header(&format!("Updating dependencies for {}", config.name));

    let deps: Vec<String> = config.dependencies.keys().cloned().collect();
    let mut updated = false;

    for req_name in deps {
        if let Ok(pypi_info) = package::fetch_package_info(&req_name, None).await {
            let latest_version = pypi_info.info.version;
            if let Some(current) = config.dependencies.get(&req_name) {
                if current != &latest_version {
                    ux::print_info(format!("Updating {} ({} -> {})", req_name, current, latest_version));
                    config.dependencies.insert(req_name.clone(), latest_version);
                    updated = true;
                }
            }
        } else {
            ux::print_warning(format!("Could not fetch latest info for {}", req_name));
        }
    }

    if updated {
        config::write_config(&config, "wovenpkg.json")?;
    } else {
        ux::print_success("All tracked dependencies are already at their latest version.");
    }

    // Use forced resolution for update
    crate::cli::install::execute(true).await?;

    Ok(())
}
