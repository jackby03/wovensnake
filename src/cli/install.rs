use crate::core::config;
use crate::core::lock::{Lockfile, LockedPackage};
use crate::dependencies::package;
use indicatif::{ProgressBar, ProgressStyle, MultiProgress};
use std::collections::{HashSet, VecDeque};

use std::error::Error;
use std::path::{Path, PathBuf};
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

    let mut total_count = 0;
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
    let mut count = 0;
    for (name, pkg) in &lockfile.packages {
        let pkg_normalized = name.to_lowercase().replace('-', "_");
        if installed.contains(&pkg_normalized) {
            continue;
        }

        let pb = multi.add(ProgressBar::new_spinner());
        pb.set_style(ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")?
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"));
        pb.set_message(format!("Installing: {} v{}", name, pkg.version));

        let dest_path = packages_dir.join(&pkg.filename);
        if !dest_path.exists() {
            package::download_package(&pkg.url, &dest_path).await?;
        }

        let hash = calculate_sha256(&dest_path)?;
        if hash != pkg.sha256 {
            return Err(format!("Hash mismatch for package: {}", name).into());
        }

        if pkg.filename.ends_with(".whl") {
            package::extract_wheel(&dest_path, site_packages)?;
            let dist_info_name = format!("{}-{}.dist-info", pkg_normalized, pkg.version);
            let mut dist_info_path = site_packages.join(&dist_info_name);
            if !dist_info_path.exists() {
                dist_info_path = site_packages.join(format!("{}-{}.dist-info", name, pkg.version));
            }
            if dist_info_path.exists() {
                package::generate_scripts(&dist_info_path, scripts_dir)?;
            }
        } else {
            package::extract_targz(&dest_path, site_packages)?;
        }

        pb.finish_with_message(format!("\x1b[32m✓\x1b[0m {}", name));
        count += 1;
    }
    Ok(count)
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
    let mut local_installed = installed.clone();
    let mut lockfile = Lockfile::new(&config.name, &config.version);
    let mut queue = VecDeque::new();
    let mut processed_deps = HashSet::new();
    let mut count = 0;

    let blacklist = vec!["dataclasses", "typing", "typing-extensions", "enum34", "pathlib"];

    for (name, version) in &config.dependencies {
        queue.push_back((name.clone(), Some(version.clone()), true));
    }

    while let Some((name, version, is_top_level)) = queue.pop_front() {
        let normalized_name = name.to_lowercase().replace('-', "_");
        if blacklist.contains(&normalized_name.as_str()) { continue; }
        if !is_top_level && processed_deps.contains(&normalized_name) { continue; }
        processed_deps.insert(normalized_name.clone());

        let pb = multi.add(ProgressBar::new_spinner());
        pb.set_style(ProgressStyle::default_spinner().template("{spinner:.cyan} {msg}")?.tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"));
        pb.set_message(format!("Resolving: {}", name));

        match package::fetch_package_info(&name, version.as_deref()).await {
            Ok(info) => {
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
                            if !processed_deps.contains(&dep_norm) {
                                queue.push_back((base_dep.to_string(), None, false));
                            }
                        }
                    }
                }

                let download = info.urls.iter()
                    .find(|u| u.packagetype == "bdist_wheel" && (u.filename.contains("none-any") || u.filename.contains("win_amd64")))
                    .or_else(|| info.urls.iter().find(|u| u.packagetype == "bdist_wheel"))
                    .or_else(|| info.urls.iter().find(|u| u.packagetype == "sdist"));

                if let Some(pkg_url) = download {
                    lockfile.packages.insert(info.info.name.clone(), LockedPackage {
                        version: info.info.version.clone(),
                        url: pkg_url.url.clone(),
                        filename: pkg_url.filename.clone(),
                        sha256: pkg_url.digests.sha256.clone(),
                        dependencies: current_deps,
                    });

                    if !local_installed.contains(&pkg_normalized) {
                        pb.set_message(format!("Installing: {}", info.info.name));
                        let dest_path = packages_dir.join(&pkg_url.filename);
                        if !dest_path.exists() {
                            package::download_package(&pkg_url.url, &dest_path).await?;
                        }

                        if pkg_url.packagetype == "bdist_wheel" {
                            package::extract_wheel(&dest_path, site_packages)?;
                            let dist_info_name = format!("{}-{}.dist-info", pkg_normalized, info.info.version);
                            let mut dist_info_path = site_packages.join(&dist_info_name);
                            if !dist_info_path.exists() {
                                dist_info_path = site_packages.join(format!("{}-{}.dist-info", info.info.name, info.info.version));
                            }
                            if dist_info_path.exists() {
                                package::generate_scripts(&dist_info_path, scripts_dir)?;
                            }
                        } else {
                            package::extract_targz(&dest_path, site_packages)?;
                        }
                        pb.finish_with_message(format!("\x1b[32m✓\x1b[0m {}", info.info.name));
                        local_installed.insert(pkg_normalized);
                        count += 1;
                    } else {
                        pb.finish_and_clear();
                    }
                } else {
                    pb.finish_with_message(format!("\x1b[33m!\x1b[0m No wheel for {}", name));
                }
            }
            Err(e) => {
                pb.finish_with_message(format!("\x1b[31m✗\x1b[0m Failed {}: {}", name, e));
            }
        }
    }
    lockfile.write(lock_path)?;
    println!("\x1b[90mGenerated wovenpkg.lock\x1b[0m");
    Ok(count)
}

fn calculate_sha256(path: &Path) -> Result<String, Box<dyn Error>> {
    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    io::copy(&mut file, &mut hasher)?;
    Ok(format!("{:x}", hasher.finalize()))
}
