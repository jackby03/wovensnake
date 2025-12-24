use crate::core::config;
use crate::core::lock::{Lockfile, LockedPackage};
use crate::dependencies::package;
use indicatif::{ProgressBar, ProgressStyle, MultiProgress};
use std::collections::HashSet;
use futures::stream::{self, StreamExt};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::Mutex;

use std::error::Error;
use std::path::Path;
use sha2::{Sha256, Digest};
use std::io;

pub async fn execute() -> Result<(), Box<dyn Error>> {
    let config = config::read_config("wovenpkg.json")?;
    let lock_path = Path::new("wovenpkg.lock");
    
    let packages_dir = Path::new("packages");
    if !packages_dir.exists() {
        std::fs::create_dir_all(packages_dir)?;
    }

    let venv_base = Path::new(&config.virtual_environment);
    
    // Automatic venv creation
    if !venv_base.exists() {
        crate::core::venv::create_venv(venv_base)?;
    }

    let site_packages = if cfg!(windows) {
        venv_base.join("Lib").join("site-packages")
    } else {
        venv_base.join("lib").join("python3.10").join("site-packages")
    };

    if !site_packages.exists() {
        std::fs::create_dir_all(&site_packages)?;
    }

    let scripts_dir = if cfg!(windows) { venv_base.join("Scripts") } else { venv_base.join("bin") };

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

    let total_count;
    let multi = MultiProgress::new();

    if lock_path.exists() {
        println!("\x1b[1m\x1b[36m🐍 WovenSnake\x1b[0m \x1b[90m| Synchronizing from lockfile...\x1b[0m\n");
        let lockfile = Lockfile::read(lock_path)?;
        total_count = install_from_lock(&lockfile, &installed, packages_dir, &site_packages, &scripts_dir, &multi).await?;
    } else {
        println!("\x1b[1m\x1b[36m🐍 WovenSnake\x1b[0m \x1b[90m| Resolving dependencies for {}\x1b[0m\n", config.name);
        total_count = install_and_resolve(&config, &installed, packages_dir, &site_packages, &scripts_dir, &multi, lock_path).await?;
    }

    if total_count > 0 {
        println!("\n\x1b[1m\x1b[32m✨ Done!\x1b[0m {} packages ready.", total_count);
    } else {
        println!("\n\x1b[1m\x1b[32m✨ All dependencies are already satisfied.\x1b[0m");
    }
    Ok(())
}

async fn install_from_lock(
    lockfile: &Lockfile,
    installed: &HashSet<String>,
    packages_dir: &Path,
    site_packages: &Path,
    scripts_dir: &Path,
    multi: &MultiProgress,
) -> Result<usize, Box<dyn Error>> {
    let packages_to_install: Vec<_> = lockfile.packages.iter()
        .filter(|(name, _)| !installed.contains(&name.to_lowercase().replace('-', "_")))
        .collect();

    if packages_to_install.is_empty() {
        return Ok(0);
    }

    let count = Arc::new(AtomicUsize::new(0));
    let concurrency_limit = 8;

    stream::iter(packages_to_install)
        .for_each_concurrent(concurrency_limit, |(name, pkg)| {
            let count = Arc::clone(&count);
            let site_packages = site_packages.to_path_buf();
            let scripts_dir = scripts_dir.to_path_buf();
            let packages_dir = packages_dir.to_path_buf();
            let name = name.clone();
            let pkg = pkg.clone();
            let multi = multi.clone();

            async move {
                let pkg_normalized = name.to_lowercase().replace('-', "_");
                let pb = multi.add(ProgressBar::new_spinner());
                pb.set_style(ProgressStyle::default_spinner()
                    .template("{spinner:.cyan} {msg}")
                    .unwrap_or_else(|_| ProgressStyle::with_template("{spinner:.cyan} {msg}").unwrap())
                    .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"));
                pb.set_message(format!("Syncing: {} v{}", name, pkg.version));

                let dest_path = packages_dir.join(&pkg.filename);
                if !dest_path.exists() {
                    if let Err(_) = package::download_package(&pkg.url, &dest_path).await { return; }
                }

                if let Ok(hash) = calculate_sha256(&dest_path) {
                    if hash != pkg.sha256 { return; }
                } else { return; }

                if pkg.filename.ends_with(".whl") {
                    if let Ok(_) = package::extract_wheel(&dest_path, &site_packages) {
                        let dist_info_name = format!("{}-{}.dist-info", pkg_normalized, pkg.version);
                        let mut dist_info_path = site_packages.join(&dist_info_name);
                        if !dist_info_path.exists() {
                            dist_info_path = site_packages.join(format!("{}-{}.dist-info", name, pkg.version));
                        }
                        if dist_info_path.exists() {
                            let _ = package::generate_scripts(&dist_info_path, &scripts_dir);
                        }
                    }
                } else {
                    let _ = package::extract_targz(&dest_path, &site_packages);
                }

                pb.finish_with_message(format!("\x1b[32m✓\x1b[0m {}", name));
                count.fetch_add(1, Ordering::SeqCst);
            }
        })
        .await;

    prune_unused_packages(site_packages, lockfile, multi)?;
    Ok(count.load(Ordering::SeqCst))
}

async fn install_and_resolve(
    config: &config::Config,
    installed: &HashSet<String>,
    packages_dir: &Path,
    site_packages: &Path,
    scripts_dir: &Path,
    multi: &MultiProgress,
    lock_path: &Path,
) -> Result<usize, Box<dyn Error>> {
    let local_installed = Arc::new(Mutex::new(installed.clone()));
    let lockfile = Arc::new(Mutex::new(Lockfile::new(&config.name, &config.version)));
    let processed_deps = Arc::new(Mutex::new(HashSet::new()));
    let count = Arc::new(AtomicUsize::new(0));
    
    let mut current_level_deps: Vec<(String, Option<String>, bool)> = config.dependencies.iter()
        .map(|(n, v)| (n.clone(), Some(v.clone()), true))
        .collect();

    let blacklist = vec!["dataclasses", "typing", "typing-extensions", "enum34", "pathlib"];

    while !current_level_deps.is_empty() {
        let next_level_mutex = Arc::new(Mutex::new(Vec::new()));

        stream::iter(current_level_deps)
            .for_each_concurrent(5, |(name, version, is_top_level)| {
                let processed_deps = Arc::clone(&processed_deps);
                let lockfile = Arc::clone(&lockfile);
                let local_installed = Arc::clone(&local_installed);
                let next_level_mutex = Arc::clone(&next_level_mutex);
                let count = Arc::clone(&count);
                let multi = multi.clone();
                let packages_dir = packages_dir.to_path_buf();
                let site_packages = site_packages.to_path_buf();
                let scripts_dir = scripts_dir.to_path_buf();
                let blacklist = blacklist.clone();

                async move {
                    let normalized_name = name.to_lowercase().replace('-', "_");
                    if blacklist.contains(&normalized_name.as_str()) { return; }
                    
                    {
                        let mut processed = processed_deps.lock().await;
                        if !is_top_level && processed.contains(&normalized_name) { return; }
                        processed.insert(normalized_name.clone());
                    }

                    let pb = multi.add(ProgressBar::new_spinner());
                    pb.set_style(ProgressStyle::default_spinner()
                        .template("{spinner:.cyan} {msg}")
                        .unwrap_or_else(|_| ProgressStyle::with_template("{spinner:.cyan} {msg}").unwrap())
                        .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"));
                    pb.set_message(format!("Resolving: {}", name));

                    if let Ok(info) = package::fetch_package_info(&name, version.as_deref()).await {
                        let pkg_normalized = info.info.name.to_lowercase().replace('-', "_");
                        let mut current_deps = Vec::new();

                        if let Some(deps) = info.info.requires_dist {
                            for dep_str in deps {
                                let dep_str_lower = dep_str.to_lowercase();
                                if dep_str_lower.contains("extra ==") { continue; }
                                if cfg!(windows) && dep_str_lower.contains("sys_platform") && !dep_str_lower.contains("win32") { continue; }
                                if cfg!(windows) && (dep_str_lower.contains("linux") || dep_str_lower.contains("unix")) { continue; }
                                if dep_str_lower.contains("python_version < '3") { continue; }

                                let base_dep = dep_str.split(|c: char| c == ';' || c == '(' || c == '<' || c == '>' || c == '=' || c == '!').next().unwrap_or("").trim();
                                if !base_dep.is_empty() {
                                    current_deps.push(base_dep.to_string());
                                    let dep_norm = base_dep.to_lowercase().replace('-', "_");
                                    {
                                        let processed = processed_deps.lock().await;
                                        if !processed.contains(&dep_norm) {
                                            next_level_mutex.lock().await.push((base_dep.to_string(), None, false));
                                        }
                                    }
                                }
                            }
                        }

                        let download = info.urls.iter()
                            .find(|u| u.packagetype == "bdist_wheel" && (u.filename.contains("none-any") || u.filename.contains("win_amd64")))
                            .or_else(|| info.urls.iter().find(|u| u.packagetype == "bdist_wheel"))
                            .or_else(|| info.urls.iter().find(|u| u.packagetype == "sdist"));

                        if let Some(pkg_url) = download {
                            {
                                let mut lock = lockfile.lock().await;
                                lock.packages.insert(info.info.name.clone(), LockedPackage {
                                    version: info.info.version.clone(),
                                    url: pkg_url.url.clone(),
                                    filename: pkg_url.filename.clone(),
                                    sha256: pkg_url.digests.sha256.clone(),
                                    dependencies: current_deps,
                                });
                            }

                            let is_installed = { local_installed.lock().await.contains(&pkg_normalized) };
                            if !is_installed {
                                pb.set_message(format!("Installing: {}", info.info.name));
                                let dest_path = packages_dir.join(&pkg_url.filename);
                                if !dest_path.exists() {
                                    let _ = package::download_package(&pkg_url.url, &dest_path).await;
                                }

                                if pkg_url.packagetype == "bdist_wheel" {
                                    if let Ok(_) = package::extract_wheel(&dest_path, &site_packages) {
                                        let dist_info_name = format!("{}-{}.dist-info", pkg_normalized, info.info.version);
                                        let mut dist_info_path = site_packages.join(&dist_info_name);
                                        if !dist_info_path.exists() {
                                            dist_info_path = site_packages.join(format!("{}-{}.dist-info", info.info.name, info.info.version));
                                        }
                                        if dist_info_path.exists() {
                                            let _ = package::generate_scripts(&dist_info_path, &scripts_dir);
                                        }
                                    }
                                } else {
                                    let _ = package::extract_targz(&dest_path, &site_packages);
                                }
                                pb.finish_with_message(format!("\x1b[32m✓\x1b[0m {}", info.info.name));
                                { local_installed.lock().await.insert(pkg_normalized); }
                                count.fetch_add(1, Ordering::SeqCst);
                            } else {
                                pb.finish_and_clear();
                            }
                        }
                    } else {
                        pb.finish_with_message(format!("\x1b[31m✗\x1b[0m Failed: {}", name));
                    }
                }
            })
            .await;

        current_level_deps = Arc::try_unwrap(next_level_mutex).unwrap().into_inner();
    }

    let final_lockfile = Arc::try_unwrap(lockfile).unwrap().into_inner();
    final_lockfile.write(lock_path)?;
    println!("\x1b[90mGenerated wovenpkg.lock\x1b[0m");
    Ok(count.load(Ordering::SeqCst))
}

fn prune_unused_packages(site_packages: &Path, lockfile: &Lockfile, multi: &MultiProgress) -> Result<(), Box<dyn Error>> {
    let pb = multi.add(ProgressBar::new_spinner());
    pb.set_style(ProgressStyle::default_spinner()
        .template("{spinner:.red} {msg}")
        .unwrap_or_else(|_| ProgressStyle::with_template("{spinner:.red} {msg}").unwrap())
        .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈"));
    pb.set_message("Pruning unused packages...");
    let protected = vec!["pip", "setuptools", "pkg_resources", "_distutils_hack", "wheel"];
    if let Ok(entries) = std::fs::read_dir(site_packages) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            let pkg_base_name = if name.ends_with(".dist-info") {
                name.split('-').next().unwrap_or("").to_lowercase().replace('-', "_")
            } else if !name.contains('.') && path.is_dir() {
                name.to_lowercase().replace('-', "_")
            } else { continue; };
            if protected.contains(&pkg_base_name.as_str()) { continue; }
            let in_lock = lockfile.packages.keys().any(|k| k.to_lowercase().replace('-', "_") == pkg_base_name);
            if !in_lock {
                if path.is_dir() { let _ = std::fs::remove_dir_all(&path); } else { let _ = std::fs::remove_file(&path); }
            }
        }
    }
    pb.finish_and_clear();
    Ok(())
}

fn calculate_sha256(path: &Path) -> Result<String, Box<dyn Error>> {
    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    io::copy(&mut file, &mut hasher)?;
    Ok(format!("{:x}", hasher.finalize()))
}
