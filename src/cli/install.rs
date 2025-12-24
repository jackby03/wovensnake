use crate::core::config;
use crate::core::lock::{Lockfile, LockedPackage};
use crate::core::cache::Cache;
use crate::dependencies::package;
use indicatif::{ProgressBar, ProgressStyle, MultiProgress};
use std::collections::{HashSet, HashMap, VecDeque};
use futures::stream::{self, StreamExt};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use std::error::Error;
use std::path::Path;
use sha2::{Sha256, Digest};
use std::io;

pub async fn execute() -> Result<(), Box<dyn Error>> {
    let config = config::read_config("wovenpkg.json")?;
    let lock_path = Path::new("wovenpkg.lock");
    let cache = Cache::init()?;
    
    let packages_dir = Path::new("packages");
    if !packages_dir.exists() {
        std::fs::create_dir_all(packages_dir)?;
    }

    let venv_base = Path::new(&config.virtual_environment);
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

    let multi = MultiProgress::new();

    if lock_path.exists() {
        println!("\x1b[1m\x1b[36m🐍 WovenSnake\x1b[0m \x1b[90m| Synchronizing from lockfile...\x1b[0m\n");
        let lockfile = Lockfile::read(lock_path)?;
        let count = install_from_lock(&lockfile, &installed, &cache, packages_dir, &site_packages, &scripts_dir, &multi).await?;
        if count > 0 {
             println!("\n\x1b[1m\x1b[32m✨ Done!\x1b[0m {} packages ready.", count);
        } else {
             println!("\n\x1b[1m\x1b[32m✨ All dependencies are already satisfied.\x1b[0m");
        }
    } else {
        println!("\x1b[1m\x1b[36m🐍 WovenSnake\x1b[0m \x1b[90m| Weaving dependency tree for {}\x1b[0m\n", config.name);
        resolve_and_install_final(&config, &installed, &cache, packages_dir, &site_packages, &scripts_dir, &multi, lock_path).await?;
        println!("\n\x1b[1m\x1b[32m✨ Done!\x1b[0m Resolution complete.");
    }

    Ok(())
}

async fn install_from_lock(
    lockfile: &Lockfile,
    installed: &HashSet<String>,
    cache: &Cache,
    packages_dir: &Path,
    site_packages: &Path,
    scripts_dir: &Path,
    multi: &MultiProgress,
) -> Result<usize, Box<dyn Error>> {
    let packages_to_install: Vec<_> = lockfile.packages.iter()
        .filter(|(name, _)| !installed.contains(&name.to_lowercase().replace('-', "_")))
        .collect();

    let count = Arc::new(AtomicUsize::new(0));
    let cache_arc = Arc::new(cache.clone());

    stream::iter(packages_to_install)
        .for_each_concurrent(8, |(name, pkg)| {
            let count = Arc::clone(&count);
            let site_packages = site_packages.to_path_buf();
            let scripts_dir = scripts_dir.to_path_buf();
            let packages_dir = packages_dir.to_path_buf();
            let name = name.clone();
            let pkg = pkg.clone();
            let multi = multi.clone();
            let cache = Arc::clone(&cache_arc);

            async move {
                let pb = multi.add(ProgressBar::new_spinner());
                pb.set_style(ProgressStyle::with_template("{spinner:.cyan} {msg}").unwrap().tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"));
                pb.set_message(format!("Syncing: {}", name));

                let dest_path = packages_dir.join(&pkg.filename);
                if cache.contains(&pkg.filename, &pkg.sha256) {
                    let _ = cache.link_to_project(&pkg.filename, &pkg.sha256, &packages_dir);
                } else if !dest_path.exists() {
                    if let Ok(res) = reqwest::get(&pkg.url).await {
                        if let Ok(data) = res.bytes().await {
                            let hash = format!("{:x}", Sha256::digest(&data));
                            if hash == pkg.sha256 {
                                let _ = cache.save(&pkg.filename, &pkg.sha256, &data);
                                let _ = std::fs::write(&dest_path, &data);
                            }
                        }
                    }
                }

                if pkg.filename.ends_with(".whl") {
                    let _ = package::extract_wheel(&dest_path, &site_packages);
                } else {
                    let _ = package::extract_targz(&dest_path, &site_packages);
                }
                pb.finish_with_message(format!("\x1b[32m✓\x1b[0m {}", name));
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
    scripts_dir: &Path,
    multi: &MultiProgress,
    lock_path: &Path,
) -> Result<usize, Box<dyn Error>> {
    let mut lockfile = Lockfile::new(&config.name, &config.version);
    let mut resolved = HashMap::<String, String>::new();
    let mut queue = VecDeque::<String>::new();
    let mut local_installed = installed_project.clone();
    
    for (name, _) in &config.dependencies { queue.push_back(name.clone()); }

    let pb = multi.add(ProgressBar::new_spinner());
    pb.set_style(ProgressStyle::with_template("{spinner:.magenta} {msg}").unwrap().tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈"));
    pb.set_message("Weaving Giga dependency tree...");

    while let Some(name) = queue.pop_front() {
        let name_lower = name.to_lowercase().replace('-', "_");
        if resolved.contains_key(&name_lower) { continue; }

        pb.set_message(format!("Resolving: {}", name));
        let info = package::fetch_package_info(&name, None).await?;
        resolved.insert(name_lower.clone(), info.info.version.clone());

        if let Some(subs) = info.info.requires_dist {
            for sub_str in subs {
                if sub_str.contains("extra ==") || (cfg!(windows) && sub_str.contains("sys_platform") && !sub_str.contains("win32")) { continue; }
                let sub_name = sub_str.split(|c:char| c == ';' || c == '(' || c == '<' || c == '>' || c == '=' || c == '!').next().unwrap().trim().to_string();
                if !sub_name.is_empty() { queue.push_back(sub_name); }
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
                dependencies: vec![],
            });

            if !local_installed.contains(&name_lower) {
                let dest_path = packages_dir.join(&pkg_url.filename);
                if !cache.contains(&pkg_url.filename, &pkg_url.digests.sha256) && !dest_path.exists() {
                    package::download_package(&pkg_url.url, &dest_path).await?;
                    let data = std::fs::read(&dest_path)?;
                    let _ = cache.save(&pkg_url.filename, &pkg_url.digests.sha256, &data);
                } else if cache.contains(&pkg_url.filename, &pkg_url.digests.sha256) {
                    let _ = cache.link_to_project(&pkg_url.filename, &pkg_url.digests.sha256, packages_dir);
                }

                if pkg_url.filename.ends_with(".whl") {
                    let _ = package::extract_wheel(&dest_path, site_packages);
                } else {
                    let _ = package::extract_targz(&dest_path, site_packages);
                }
                local_installed.insert(name_lower);
            }
        }
    }
    
    lockfile.write(lock_path)?;
    pb.finish_with_message("\x1b[32m✓\x1b[0m Tree woven.");
    Ok(resolved.len())
}

fn prune_unused_packages(site_packages: &Path, lockfile: &Lockfile, multi: &MultiProgress) -> Result<(), Box<dyn Error>> {
    let pb = multi.add(ProgressBar::new_spinner());
    pb.set_style(ProgressStyle::with_template("{spinner:.red} {msg}").unwrap().tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈"));
    pb.set_message("Pruning environment...");
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
            if !lockfile.packages.keys().any(|k| k.to_lowercase().replace('-', "_") == pkg_base_name) {
                if path.is_dir() { let _ = std::fs::remove_dir_all(&path); } else { let _ = std::fs::remove_file(&path); }
            }
        }
    }
    pb.finish_and_clear();
    Ok(())
}
