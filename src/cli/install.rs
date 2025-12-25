use std::collections::{HashMap, HashSet, VecDeque};
use std::error::Error;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use futures::stream::{self, StreamExt};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use sha2::{Digest, Sha256};

use crate::cli::ux;
use crate::core::cache::Cache;
use crate::core::config;
use crate::core::lock::{Artifact, LockedPackage, Lockfile};
use crate::core::selection::select_artifact;
use crate::dependencies::package;

pub async fn execute() -> Result<(), Box<dyn Error>> {
    let config = config::read_config("wovenpkg.json")?;

    // Validate Python version before proceeding
    crate::core::python::validate_python_version(&config.python_version)?;

    let lock_path = Path::new("wovenpkg.lock");
    let cache = Cache::init()?;

    let packages_dir = Path::new("packages");
    if !packages_dir.exists() {
        std::fs::create_dir_all(packages_dir)?;
    }

    let venv_base = Path::new(&config.virtual_environment);
    if !venv_base.exists() {
        crate::core::venv::create_venv(venv_base, &config.python_version)?;
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

    let multi = MultiProgress::new();

    if lock_path.exists() {
        ux::print_header("Synchronizing from lockfile...");
        let lockfile = Lockfile::read(lock_path)?;
        let count = install_from_lock(
            &lockfile,
            &installed,
            &cache,
            packages_dir,
            &site_packages,
            &scripts_dir,
            &multi,
        )
        .await?;

        prune_unused_packages(&site_packages, &lockfile, &multi);

        if count > 0 {
            ux::print_success(format!("{count} packages ready."));
        } else {
            ux::print_success("All dependencies are already satisfied.");
        }
    } else {
        ux::print_header(&format!("Weaving dependency tree for {}", config.name));
        resolve_and_install_final(
            &config,
            &installed,
            &cache,
            packages_dir,
            &site_packages,
            &scripts_dir,
            &multi,
            lock_path,
        )
        .await?;

        // Re-read lockfile for pruning
        let lockfile = Lockfile::read(lock_path)?;
        prune_unused_packages(&site_packages, &lockfile, &multi);

        ux::print_success("Resolution complete.");
    }

    Ok(())
}

async fn install_from_lock(
    lockfile: &Lockfile,
    installed: &HashSet<String>,
    cache: &Cache,
    packages_dir: &Path,
    site_packages: &Path,
    _scripts_dir: &Path,
    multi: &MultiProgress,
) -> Result<usize, Box<dyn Error>> {
    let packages_to_install: Vec<_> = lockfile
        .packages
        .iter()
        .filter(|(name, _)| !installed.contains(&name.to_lowercase().replace('-', "_")))
        .collect();

    let count = Arc::new(AtomicUsize::new(0));
    let cache_arc = Arc::new(cache.clone());

    stream::iter(packages_to_install)
        .for_each_concurrent(8, |(name, pkg)| {
            let count = Arc::clone(&count);
            let site_packages = site_packages.to_path_buf();
            let packages_dir = packages_dir.to_path_buf();
            let name = name.clone();
            let pkg = pkg.clone();
            let multi = multi.clone();
            let cache = Arc::clone(&cache_arc);

            async move {
                let pb = multi.add(ProgressBar::new_spinner());
                pb.set_style(
                    ProgressStyle::with_template("{spinner:.cyan} {msg}")
                        .unwrap()
                        .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"),
                );
                pb.set_message(format!("Syncing: {name}"));

                if let Some(artifact) =
                    select_artifact(&pkg.artifacts, if cfg!(windows) { "win_amd64" } else { "manylinux" })
                {
                    let dest_path = packages_dir.join(&artifact.filename);
                    if cache.contains(&artifact.filename, &artifact.sha256) {
                        let _ = cache.link_to_project(&artifact.filename, &artifact.sha256, &packages_dir);
                    } else if !dest_path.exists() {
                        if let Ok(res) = reqwest::get(&artifact.url).await {
                            if let Ok(data) = res.bytes().await {
                                let hash = format!("{:x}", Sha256::digest(&data));
                                if hash == artifact.sha256 {
                                    let _ = cache.save(&artifact.filename, &artifact.sha256, &data);
                                    let _ = std::fs::write(&dest_path, &data);
                                }
                            }
                        }
                    }

                    if artifact.filename.to_lowercase().ends_with(".whl") {
                        let _ = package::extract_wheel(&dest_path, &site_packages);
                    } else {
                        let _ = package::extract_targz(&dest_path, &site_packages);
                    }
                }
                pb.finish_with_message(format!("\x1b[32m✓\x1b[0m {name}"));
                count.fetch_add(1, Ordering::SeqCst);
            }
        })
        .await;

    Ok(count.load(Ordering::SeqCst))
}

async fn resolve_and_install_final(
    config: &config::Config,
    installed_project: &HashSet<String>,
    cache: &Cache,
    packages_dir: &Path,
    site_packages: &Path,
    _scripts_dir: &Path,
    multi: &MultiProgress,
    lock_path: &Path,
) -> Result<usize, Box<dyn Error>> {
    let mut lockfile = Lockfile::new(&config.name, &config.version);
    let mut resolved = HashMap::<String, String>::new();
    let mut queue = VecDeque::<String>::new();
    let mut local_installed = installed_project.clone();

    for name in config.dependencies.keys() {
        queue.push_back(name.clone());
    }

    let pb = multi.add(ProgressBar::new_spinner());
    pb.set_style(
        ProgressStyle::with_template("{spinner:.magenta} {msg}")
            .unwrap()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈"),
    );
    pb.set_message("Weaving Giga dependency tree...");

    while let Some(name) = queue.pop_front() {
        let name_lower = name.to_lowercase().replace('-', "_");
        if resolved.contains_key(&name_lower) {
            continue;
        }

        pb.set_message(format!("Resolving: {name}"));
        let info = package::fetch_package_info(&name, None).await?;
        resolved.insert(name_lower.clone(), info.info.version.clone());

        if let Some(subs) = info.info.requires_dist {
            for sub_str in subs {
                if sub_str.contains("extra ==")
                    || (cfg!(windows) && sub_str.contains("sys_platform") && !sub_str.contains("win32"))
                {
                    continue;
                }
                let sub_name = sub_str
                    .split([';', '(', '<', '>', '=', '!'])
                    .next()
                    .unwrap()
                    .trim()
                    .to_string();
                if !sub_name.is_empty() {
                    queue.push_back(sub_name);
                }
            }
        }

        // Collect ALL artifacts for the Lockfile (Multiplatform support)
        let mut artifacts: Vec<Artifact> = Vec::new();
        for url in &info.urls {
            if url.packagetype == "bdist_wheel" {
                // Simplistic platform detection from filename
                let platform = if url.filename.contains("win_amd64") {
                    "win_amd64".to_string()
                } else if url.filename.contains("manylinux") {
                    "manylinux".to_string()
                } else if url.filename.contains("any") {
                    "any".to_string()
                } else {
                    "other".to_string()
                };

                artifacts.push(Artifact {
                    url: url.url.clone(),
                    filename: url.filename.clone(),
                    sha256: url.digests.sha256.clone(),
                    platform,
                });
            } else if url.packagetype == "sdist" {
                artifacts.push(Artifact {
                    url: url.url.clone(),
                    filename: url.filename.clone(),
                    sha256: url.digests.sha256.clone(),
                    platform: "source".to_string(),
                });
            }
        }

        lockfile.packages.insert(
            info.info.name.clone(),
            LockedPackage {
                version: info.info.version.clone(),
                artifacts: artifacts.clone(),
                dependencies: vec![],
            },
        );

        // Resolve local installation
        let current_platform = if cfg!(windows) { "win_amd64" } else { "manylinux" }; // simplified
        if let Some(pkg_url) = select_artifact(&artifacts, current_platform) {
            if local_installed.insert(name_lower) {
                let dest_path = packages_dir.join(&pkg_url.filename);
                if !cache.contains(&pkg_url.filename, &pkg_url.sha256) && !dest_path.exists() {
                    package::download_package(&pkg_url.url, &dest_path).await?;
                    let data = std::fs::read(&dest_path)?;
                    let _ = cache.save(&pkg_url.filename, &pkg_url.sha256, &data);
                } else if cache.contains(&pkg_url.filename, &pkg_url.sha256) {
                    let _ = cache.link_to_project(&pkg_url.filename, &pkg_url.sha256, packages_dir);
                }

                if pkg_url.filename.to_lowercase().ends_with(".whl") {
                    let _ = package::extract_wheel(&dest_path, site_packages);
                } else {
                    let _ = package::extract_targz(&dest_path, site_packages);
                }
            }
        }
    }

    lockfile.write(lock_path)?;
    pb.finish_with_message("\x1b[32m✓\x1b[0m Tree woven.");
    Ok(resolved.len())
}

fn prune_unused_packages(site_packages: &Path, lockfile: &Lockfile, multi: &MultiProgress) {
    let pb = multi.add(ProgressBar::new_spinner());
    pb.set_style(
        ProgressStyle::with_template("{spinner:.red} {msg}")
            .unwrap()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈"),
    );
    pb.set_message("Pruning environment...");
    let protected = ["pip", "setuptools", "pkg_resources", "_distutils_hack", "wheel"];
    if let Ok(entries) = std::fs::read_dir(site_packages) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            let pkg_base_name = if name.ends_with(".dist-info") {
                name.split('-').next().unwrap_or("").to_lowercase().replace('-', "_")
            } else if !name.contains('.') && path.is_dir() {
                name.to_lowercase().replace('-', "_")
            } else {
                continue;
            };
            if protected.contains(&pkg_base_name.as_str()) {
                continue;
            }
            if !lockfile
                .packages
                .keys()
                .any(|k| k.to_lowercase().replace('-', "_") == pkg_base_name)
            {
                if path.is_dir() {
                    let _ = std::fs::remove_dir_all(&path);
                } else {
                    let _ = std::fs::remove_file(&path);
                }
            }
        }
    }
    pb.finish_and_clear();
}
