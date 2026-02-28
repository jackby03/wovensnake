use std::collections::HashSet;
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

const fn current_platform() -> &'static str {
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

fn platform_from_filename(filename: &str) -> String {
    if filename.contains("win_amd64") || filename.contains("win32") {
        "win_amd64".to_string()
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

pub async fn execute(force_resolve: bool) -> Result<(), Box<dyn Error>> {
    let config = config::read_config("wovenpkg.json")?;

    let lock_path = Path::new("wovenpkg.lock");
    let cache = Cache::init()?;

    let packages_dir = Path::new("packages");
    if !packages_dir.exists() {
        std::fs::create_dir_all(packages_dir)?;
    }

    let venv_base = Path::new(&config.virtual_environment);
    if venv_base.exists() {
        // Check if existing venv matches the required version
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
            _ => {} // Versions match
        }
    }

    // Validate Python version before proceeding (or creating venv)
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

    let multi = MultiProgress::new();

    if lock_path.exists() && !force_resolve {
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
        if force_resolve && lock_path.exists() {
            ux::print_header(&format!("Re-weaving dependency tree for {}", config.name));
        } else {
            ux::print_header(&format!("Weaving dependency tree for {}", config.name));
        }

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
    scripts_dir: &Path,
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
            let scripts_dir = scripts_dir.to_path_buf();
            let python_version = lockfile.python_version.clone();
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

                if let Some(artifact) = select_artifact(&pkg.artifacts, current_platform()) {
                    let dest_path = packages_dir.join(&artifact.filename);

                    if cache.contains(&artifact.filename, &artifact.sha256) {
                        if let Err(e) = cache.link_to_project(&artifact.filename, &artifact.sha256, &packages_dir) {
                            pb.println(format!("\x1b[33m⚠ Warning:\x1b[0m {name}: cache link failed ({e})"));
                        }
                    } else if !dest_path.exists() {
                        let res = match reqwest::get(&artifact.url).await {
                            Ok(r) => r,
                            Err(e) => {
                                pb.finish_with_message(format!("\x1b[31m✗\x1b[0m {name}: request failed ({e})"));
                                return;
                            }
                        };
                        let data = match res.bytes().await {
                            Ok(d) => d,
                            Err(e) => {
                                pb.finish_with_message(format!("\x1b[31m✗\x1b[0m {name}: download failed ({e})"));
                                return;
                            }
                        };
                        let hash = format!("{:x}", Sha256::digest(&data));
                        if hash != artifact.sha256 {
                            pb.finish_with_message(format!(
                                "\x1b[31m✗\x1b[0m {name}: hash mismatch (corrupt download)"
                            ));
                            return;
                        }
                        if let Err(e) = cache.save(&artifact.filename, &artifact.sha256, &data) {
                            pb.println(format!("\x1b[33m⚠ Warning:\x1b[0m {name}: cache save failed ({e})"));
                        }
                        if let Err(e) = std::fs::write(&dest_path, &data) {
                            pb.finish_with_message(format!("\x1b[31m✗\x1b[0m {name}: write failed ({e})"));
                            return;
                        }
                    }

                    let is_wheel = artifact.filename.to_lowercase().ends_with(".whl");
                    let extract_result = if is_wheel {
                        package::extract_wheel(&dest_path, &site_packages)
                    } else {
                        package::extract_targz(&dest_path, &site_packages)
                    };
                    if let Err(e) = extract_result {
                        pb.finish_with_message(format!("\x1b[31m✗\x1b[0m {name}: extract failed ({e})"));
                        return;
                    }

                    if is_wheel {
                        if let Some(dist_info) = find_dist_info(&site_packages, &name) {
                            if let Err(e) = package::generate_scripts(&dist_info, &scripts_dir, &python_version) {
                                pb.println(format!(
                                    "\x1b[33m⚠ Warning:\x1b[0m {name}: script generation failed ({e})"
                                ));
                            }
                        }
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
    scripts_dir: &Path,
    multi: &MultiProgress,
    lock_path: &Path,
) -> Result<usize, Box<dyn Error>> {
    let mut lockfile = Lockfile::new(&config.name, &config.version, &config.python_version);
    let mut local_installed = installed_project.clone();

    let pb = multi.add(ProgressBar::new_spinner());
    pb.set_style(
        ProgressStyle::with_template("{spinner:.magenta} {msg}")
            .unwrap()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈"),
    );
    pb.set_message("Solving giga-brain dependencies...");

    // Use the new centralized resolver
    let graph = crate::core::resolver::resolve(&config.dependencies, &config.python_version).await?;
    pb.set_message("Dependency tree resolved. Satisfying packages...");

    let mut installed_count = 0;

    for (name_lower, node) in graph.packages {
        // URLs were already fetched during resolution — no second PyPI request needed.
        let mut artifacts: Vec<Artifact> = Vec::new();
        for url in &node.urls {
            if url.packagetype == "bdist_wheel" {
                let platform = platform_from_filename(&url.filename);

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
            node.name.clone(),
            LockedPackage {
                version: node.version.clone(),
                artifacts: artifacts.clone(),
                dependencies: node.dependencies.clone(),
            },
        );

        // Resolve local installation
        if let Some(pkg_url) = select_artifact(&artifacts, current_platform()) {
            if local_installed.insert(name_lower) {
                let dest_path = packages_dir.join(&pkg_url.filename);
                if !cache.contains(&pkg_url.filename, &pkg_url.sha256) && !dest_path.exists() {
                    pb.set_message(format!("Downloading: {}", node.name));
                    package::download_package(&pkg_url.url, &dest_path).await?;
                    let data = std::fs::read(&dest_path)?;
                    if let Err(e) = cache.save(&pkg_url.filename, &pkg_url.sha256, &data) {
                        ux::print_warning(format!("Cache save failed for {}: {e}", node.name));
                    }
                } else if cache.contains(&pkg_url.filename, &pkg_url.sha256) {
                    if let Err(e) = cache.link_to_project(&pkg_url.filename, &pkg_url.sha256, packages_dir) {
                        ux::print_warning(format!("Cache link failed for {}: {e}", node.name));
                    }
                }

                pb.set_message(format!("Installing: {}", node.name));
                let is_wheel = pkg_url.filename.to_lowercase().ends_with(".whl");
                if is_wheel {
                    package::extract_wheel(&dest_path, site_packages)?;
                } else {
                    package::extract_targz(&dest_path, site_packages)?;
                }
                if is_wheel {
                    if let Some(dist_info) = find_dist_info(site_packages, &node.name) {
                        if let Err(e) = package::generate_scripts(&dist_info, scripts_dir, &config.python_version) {
                            ux::print_warning(format!("Script generation failed for {}: {e}", node.name));
                        }
                    }
                }
                installed_count += 1;
            }
        }
    }

    lockfile.write(lock_path)?;
    pb.finish_with_message("\x1b[32m✓\x1b[0m Tree woven and environment satisfied.");
    Ok(installed_count)
}

/// Locate the `.dist-info` directory for a package inside `site_packages`.
/// Only wheels produce a `.dist-info` dir; sdists do not.
fn find_dist_info(site_packages: &Path, name: &str) -> Option<std::path::PathBuf> {
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
                let remove_result = if path.is_dir() {
                    std::fs::remove_dir_all(&path)
                } else {
                    std::fs::remove_file(&path)
                };
                if let Err(e) = remove_result {
                    pb.println(format!("\x1b[33m⚠ Warning:\x1b[0m Could not prune {name}: {e}"));
                }
            }
        }
    }
    pb.finish_and_clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_from_filename_windows() {
        assert_eq!(
            platform_from_filename("numpy-1.24.0-cp311-cp311-win_amd64.whl"),
            "win_amd64"
        );
        assert_eq!(
            platform_from_filename("numpy-1.24.0-cp311-cp311-win32.whl"),
            "win_amd64"
        );
    }

    #[test]
    fn test_platform_from_filename_macos() {
        assert_eq!(
            platform_from_filename("numpy-1.24.0-cp311-cp311-macosx_11_0_arm64.whl"),
            "macosx_arm64"
        );
        assert_eq!(
            platform_from_filename("numpy-1.24.0-cp311-cp311-macosx_10_9_x86_64.whl"),
            "macosx_x86_64"
        );
        assert_eq!(
            platform_from_filename("some-pkg-macosx_12_0_aarch64.whl"),
            "macosx_arm64"
        );
    }

    #[test]
    fn test_platform_from_filename_linux() {
        assert_eq!(
            platform_from_filename("numpy-1.24.0-cp311-cp311-manylinux_2_17_x86_64.manylinux2014_x86_64.whl"),
            "manylinux"
        );
        assert_eq!(
            platform_from_filename("numpy-1.24.0-cp311-cp311-manylinux_2_17_aarch64.whl"),
            "manylinux_aarch64"
        );
    }

    #[test]
    fn test_platform_from_filename_universal() {
        assert_eq!(platform_from_filename("requests-2.28.0-py3-none-any.whl"), "any");
        assert_eq!(platform_from_filename("six-1.16.0-py2.py3-none-any.whl"), "any");
    }

    #[test]
    fn test_platform_from_filename_other() {
        assert_eq!(platform_from_filename("unknown-1.0-cp311-cp311-unknown.whl"), "other");
    }

    #[test]
    fn test_current_platform_is_valid() {
        let platform = current_platform();
        let valid = [
            "win_amd64",
            "macosx_arm64",
            "macosx_x86_64",
            "manylinux",
            "manylinux_aarch64",
        ];
        assert!(valid.contains(&platform), "unexpected platform: {platform}");
    }
}
