use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use futures::stream::{self, StreamExt};
use sha2::{Digest, Sha256};

use crate::core::cache::Cache;
use crate::core::config;
use crate::core::error::WovenError;
use crate::core::lock::{Artifact, LockedPackage, Lockfile};
use crate::core::selection::select_artifact;
use crate::dependencies::package;

pub trait InstallReporter: Send + Sync {
    fn create_task(&self, name: &str) -> Box<dyn InstallTaskReporter>;
    fn create_spinner(&self, msg: &str) -> Box<dyn InstallTaskReporter>;
}

pub trait InstallTaskReporter: Send + Sync {
    fn set_message(&self, msg: String);
    fn finish_success(&self, msg: String);
    fn finish_error(&self, msg: String);
    fn warning(&self, msg: String);
    fn print_line(&self, msg: String);
    fn finish_and_clear(&self);
}

pub const fn current_platform() -> &'static str {
    if cfg!(windows) {
        "win_amd64"
    } else if cfg!(target_os = "macos") {
        if cfg!(target_arch = "aarch64") {
            "macosx_arm64"
        } else {
            "macosx_x86_64"
        }
    } else if cfg!(target_arch = "aarch64") {
        "manylinux_aarch64"
    } else {
        "manylinux"
    }
}

pub fn platform_from_filename(filename: &str) -> String {
    if filename.contains("win_amd64") {
        "win_amd64".to_string()
    } else if filename.contains("win32") {
        "win32".to_string()
    } else if filename.contains("macosx") {
        if filename.contains("arm64") || filename.contains("aarch64") {
            "macosx_arm64".to_string()
        } else {
            "macosx_x86_64".to_string()
        }
    } else if filename.contains("manylinux") {
        if filename.contains("aarch64") {
            "manylinux_aarch64".to_string()
        } else {
            "manylinux".to_string()
        }
    } else if filename.contains("none-any") || filename.contains("py3-none") || filename.contains("py2.py3") {
        "any".to_string()
    } else {
        "other".to_string()
    }
}

pub async fn install_from_lock<S: std::hash::BuildHasher + Sync>(
    lockfile: &Lockfile,
    installed: &HashSet<String, S>,
    cache: &Cache,
    packages_dir: &Path,
    site_packages: &Path,
    scripts_dir: &Path,
    reporter: Arc<dyn InstallReporter>,
) -> Result<usize, WovenError> {
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
            let scripts_dir = scripts_dir.to_path_buf();
            let python_version = lockfile.python_version.clone();
            let name = name.clone();
            let pkg = pkg.clone();
            let reporter = Arc::clone(&reporter);
            let cache = Arc::clone(&cache_arc);

            async move {
                let task = reporter.create_task(&name);

                if let Some(artifact) = select_artifact(&pkg.artifacts, current_platform()) {
                    let dest_path = packages_dir.join(&artifact.filename);

                    if cache.contains(&artifact.filename, &artifact.sha256) {
                        if let Err(e) = cache.link_to_project(&artifact.filename, &artifact.sha256, &packages_dir) {
                            task.warning(format!("{name}: cache link failed ({e})"));
                        }
                    } else if !dest_path.exists() {
                        let res = match crate::core::http::CLIENT.get(&artifact.url).send().await {
                            Ok(r) => r,
                            Err(e) => {
                                task.finish_error(format!("{name}: request failed ({e})"));
                                return;
                            }
                        };
                        let data = match res.bytes().await {
                            Ok(d) => d,
                            Err(e) => {
                                task.finish_error(format!("{name}: download failed ({e})"));
                                return;
                            }
                        };
                        let hash = format!("{:x}", Sha256::digest(&data));
                        if hash != artifact.sha256 {
                            task.finish_error(format!("{name}: hash mismatch (corrupt download)"));
                            return;
                        }
                        if let Err(e) = cache.save(&artifact.filename, &artifact.sha256, &data) {
                            task.warning(format!("{name}: cache save failed ({e})"));
                        }
                        if let Err(e) = tokio::fs::write(&dest_path, &data).await {
                            task.finish_error(format!("{name}: write failed ({e})"));
                            return;
                        }
                    }

                    let is_wheel = artifact.filename.to_lowercase().ends_with(".whl");
                    let dest_path_clone = dest_path.clone();
                    let site_packages_clone = site_packages.clone();

                    let extract_result = tokio::task::spawn_blocking(move || {
                        let res = if is_wheel {
                            package::extract_wheel(&dest_path_clone, &site_packages_clone)
                        } else {
                            package::extract_targz(&dest_path_clone, &site_packages_clone)
                        };
                        res.map_err(|e| e.to_string())
                    })
                    .await
                    .unwrap_or_else(|e| Err(format!("Task joined failed: {e}")));

                    if let Err(e) = extract_result {
                        task.finish_error(format!("{name}: extract failed ({e})"));
                        return;
                    }

                    if is_wheel {
                        if let Some(dist_info) = find_dist_info(&site_packages, &name) {
                            if let Err(e) = package::generate_scripts(&dist_info, &scripts_dir, &python_version) {
                                task.warning(format!("{name}: script generation failed ({e})"));
                            }
                        }
                    }
                }
                task.finish_success(name.clone());
                count.fetch_add(1, Ordering::SeqCst);
            }
        })
        .await;

    Ok(count.load(Ordering::SeqCst))
}

pub async fn resolve_and_install_final<S: std::hash::BuildHasher + Sync>(
    config: &config::Config,
    installed_project: &HashSet<String, S>,
    cache: &Cache,
    packages_dir: &Path,
    site_packages: &Path,
    scripts_dir: &Path,
    reporter: Arc<dyn InstallReporter>,
    lock_path: &Path,
) -> Result<usize, WovenError> {
    let mut lockfile = Lockfile::new(&config.name, &config.version, &config.python_version);
    let mut local_installed: HashSet<String> = installed_project.iter().cloned().collect();

    let task = reporter.create_spinner("Solving dependencies...");

    let graph = crate::core::resolver::resolve(&config.dependencies, &config.python_version).await?;
    task.set_message("Dependency tree resolved. Satisfying packages...".to_string());

    let mut installed_count = 0;

    for (name_lower, node) in graph.packages {
        let mut artifacts: Vec<Artifact> = Vec::new();
        for url in node.urls {
            if url.packagetype == "bdist_wheel" {
                let platform = platform_from_filename(&url.filename);
                artifacts.push(Artifact {
                    url: url.url,
                    filename: url.filename,
                    sha256: url.digests.sha256,
                    platform,
                });
            } else if url.packagetype == "sdist" {
                artifacts.push(Artifact {
                    url: url.url,
                    filename: url.filename,
                    sha256: url.digests.sha256,
                    platform: "source".to_string(),
                });
            }
        }

        let node_name = node.name;
        lockfile.packages.insert(
            node_name.clone(),
            LockedPackage {
                version: node.version,
                artifacts: artifacts.clone(),
                dependencies: node.dependencies,
            },
        );

        if let Some(pkg_url) = select_artifact(&artifacts, current_platform()) {
            if local_installed.insert(name_lower) {
                let dest_path = packages_dir.join(&pkg_url.filename);
                if !cache.contains(&pkg_url.filename, &pkg_url.sha256) && !dest_path.exists() {
                    task.set_message(format!("Downloading: {node_name}"));
                    package::download_package(&pkg_url.url, &dest_path).await?;
                    let data = tokio::fs::read(&dest_path).await?;
                    if let Err(e) = cache.save(&pkg_url.filename, &pkg_url.sha256, &data) {
                        task.warning(format!("Cache save failed for {node_name}: {e}"));
                    }
                } else if cache.contains(&pkg_url.filename, &pkg_url.sha256) {
                    if let Err(e) = cache.link_to_project(&pkg_url.filename, &pkg_url.sha256, packages_dir) {
                        task.warning(format!("Cache link failed for {node_name}: {e}"));
                    }
                }

                task.set_message(format!("Installing: {node_name}"));
                let is_wheel = pkg_url.filename.to_lowercase().ends_with(".whl");
                let site_packages_clone = site_packages.to_path_buf();

                let ext_res = tokio::task::spawn_blocking(move || {
                    let res = if is_wheel {
                        package::extract_wheel(&dest_path, &site_packages_clone)
                    } else {
                        package::extract_targz(&dest_path, &site_packages_clone)
                    };
                    res.map_err(|e| e.to_string())
                })
                .await
                .unwrap_or_else(|e| Err(format!("Task joined failed: {e}")));

                if let Err(e) = ext_res {
                    task.warning(format!("Extract failed for {node_name}: {e}"));
                }
                if is_wheel {
                    if let Some(dist_info) = find_dist_info(site_packages, &node_name) {
                        if let Err(e) = package::generate_scripts(&dist_info, scripts_dir, &config.python_version) {
                            task.warning(format!("Script generation failed for {node_name}: {e}"));
                        }
                    }
                }
                installed_count += 1;
            }
        }
    }

    lockfile.write(lock_path)?;
    task.finish_success("Tree woven and environment satisfied.".to_string());
    Ok(installed_count)
}

pub fn find_dist_info(site_packages: &Path, name: &str) -> Option<PathBuf> {
    let normalized = name.to_lowercase().replace('-', "_");
    std::fs::read_dir(site_packages)
        .ok()?
        .filter_map(std::result::Result::ok)
        .find(|e| {
            let fname = e.file_name().to_string_lossy().to_lowercase().replace('-', "_");
            fname.starts_with(&normalized) && fname.ends_with(".dist-info")
        })
        .map(|e| e.path())
}

pub fn prune_unused_packages(site_packages: &Path, lockfile: &Lockfile, reporter: &Arc<dyn InstallReporter>) {
    let task = reporter.create_spinner("Pruning environment...");
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
                let remove_result = if path.is_dir() {
                    std::fs::remove_dir_all(&path)
                } else {
                    std::fs::remove_file(&path)
                };
                if let Err(e) = remove_result {
                    task.warning(format!("Could not prune {name}: {e}"));
                }
            }
        }
    }
    task.finish_and_clear();
}
