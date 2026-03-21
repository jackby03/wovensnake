use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;

use crate::cli::progress::CliProgressReporter;
use crate::cli::ux;
use crate::core::cache::Cache;
use crate::core::config;
use crate::core::installer;
use crate::core::lock::Lockfile;

pub async fn execute(force_resolve: bool) -> anyhow::Result<()> {
    let config = config::read_config("wovenpkg.json")?;
    let lock_path = Path::new("wovenpkg.lock");
    let cache = Cache::init()?;

    let packages_dir = Path::new("packages");
    if !packages_dir.exists() {
        std::fs::create_dir_all(packages_dir)?;
    }

    let venv_base = Path::new(&config.virtual_environment);
    if venv_base.exists() {
        match crate::core::venv::get_venv_python_version(venv_base) {
            Ok(version) if version != config.python_version => {
                ux::print_warning(format!(
                    "Existing virtual environment uses Python {}, but {} is required by wovenpkg.json.",
                    version, config.python_version
                ));
                ux::print_info("Consider running 'woven clean' and then 'woven install' to recreate the environment.");
            }
            Err(e) => {
                ux::print_warning(format!("Could not verify virtual environment Python version: {e}"));
            }
            _ => {}
        }
    }

    crate::core::python::validate_python_version(&config.python_version).await?;

    if !venv_base.exists() {
        crate::core::venv::create_venv(venv_base, &config.python_version).await?;
    }

    let python_dir = format!("python{}", config.python_version);
    let site_packages = if cfg!(windows) {
        venv_base.join("Lib").join("site-packages")
    } else {
        venv_base.join("lib").join(python_dir).join("site-packages")
    };

    if !site_packages.exists() {
        std::fs::create_dir_all(&site_packages)?;
    }

    let scripts_dir = if cfg!(windows) {
        venv_base.join("Scripts")
    } else {
        venv_base.join("bin")
    };

    let mut installed = HashSet::new();
    if let Ok(entries) = std::fs::read_dir(&site_packages) {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                if name.ends_with(".dist-info") {
                    let pkg_name = name.split('-').next().unwrap_or("").to_lowercase().replace('-', "_");
                    installed.insert(pkg_name);
                }
            }
        }
    }

    let reporter: Arc<dyn installer::InstallReporter> = CliProgressReporter::new();

    if lock_path.exists() && !force_resolve {
        ux::print_header("Synchronizing from lockfile...");
        let lockfile = Lockfile::read(lock_path)?;
        let count = installer::install_from_lock(
            &lockfile,
            &installed,
            &cache,
            packages_dir,
            &site_packages,
            &scripts_dir,
            reporter.clone(),
        )
        .await?;

        installer::prune_unused_packages(&site_packages, &lockfile, &reporter);

        if count > 0 {
            ux::print_success(format!("{count} packages ready."));
        } else {
            ux::print_success("All dependencies are already satisfied.");
        }
    } else {
        if force_resolve && lock_path.exists() {
            ux::print_header(&format!("Re-weaving dependency tree for {}", config.name));
        } else {
            ux::print_header(&format!("Weaving dependency tree for {}", config.name));
        }

        installer::resolve_and_install_final(
            &config,
            &installed,
            &cache,
            packages_dir,
            &site_packages,
            &scripts_dir,
            reporter.clone(),
            lock_path,
        )
        .await?;

        let lockfile = Lockfile::read(lock_path)?;
        installer::prune_unused_packages(&site_packages, &lockfile, &reporter);

        ux::print_success("Resolution complete.");
    }

    Ok(())
}
