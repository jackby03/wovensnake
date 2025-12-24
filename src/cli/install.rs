use crate::core::config;
use crate::dependencies::package;
use indicatif::{ProgressBar, ProgressStyle, MultiProgress};
use std::collections::{HashSet, VecDeque};
use std::error::Error;
use std::path::Path;

pub async fn execute() -> Result<(), Box<dyn Error>> {
    let config = config::read_config("wovenpkg.json")?;
    println!("\x1b[1m\x1b[36m🐍 WovenSnake\x1b[0m \x1b[90m| Synchronizing dependencies for {}\x1b[0m\n", config.name);
    
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
    let mut processed_deps = HashSet::new();
    let mut queue = VecDeque::new();
    let multi = MultiProgress::new();
    let mut total_count = 0;

    // Blacklist of packages that should NOT be installed on Python 3.10+
    let blacklist = vec!["dataclasses", "typing", "typing-extensions", "enum34", "pathlib"];

    // Scan site-packages
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

    // Initial dependencies from config
    for (name, version) in &config.dependencies {
        queue.push_back((name.clone(), Some(version.clone()), true));
    }

    while let Some((name, version, is_top_level)) = queue.pop_front() {
        let normalized_name = name.to_lowercase().replace('-', "_");
        
        if blacklist.contains(&normalized_name.as_str()) {
            continue;
        }

        if !is_top_level && (installed.contains(&normalized_name) || processed_deps.contains(&normalized_name)) {
            continue;
        }
        processed_deps.insert(normalized_name.clone());

        let pb = multi.add(ProgressBar::new_spinner());
        pb.set_style(ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")?
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"));
        pb.set_message(format!("Processing: {}", name));

        match package::fetch_package_info(&name, version.as_deref()).await {
            Ok(info) => {
                let pkg_name = info.info.name.to_lowercase().replace('-', "_");
                
                if !installed.contains(&pkg_name) {
                    pb.set_message(format!("Fetching: {} v{}", info.info.name, info.info.version));
                    
                    let download = info.urls.iter()
                        .find(|u| u.packagetype == "bdist_wheel" && (u.filename.contains("none-any") || u.filename.contains("win_amd64")))
                        .or_else(|| info.urls.iter().find(|u| u.packagetype == "bdist_wheel"))
                        .or_else(|| info.urls.iter().find(|u| u.packagetype == "sdist"));

                    if let Some(pkg_url) = download {
                        let dest_path = packages_dir.join(&pkg_url.filename);
                        if !dest_path.exists() {
                            pb.set_message(format!("Downloading: {}", info.info.name));
                            package::download_package(&pkg_url.url, &dest_path).await?;
                        }

                        pb.set_message(format!("Extracting: {}", info.info.name));
                        if pkg_url.packagetype == "bdist_wheel" {
                            package::extract_wheel(&dest_path, &site_packages)?;
                            let dist_info_name = format!("{}-{}.dist-info", pkg_name, info.info.version);
                            let mut dist_info_path = site_packages.join(&dist_info_name);
                            if !dist_info_path.exists() {
                                dist_info_path = site_packages.join(format!("{}-{}.dist-info", info.info.name, info.info.version));
                            }
                            if dist_info_path.exists() {
                                package::generate_scripts(&dist_info_path, &scripts_dir)?;
                            }
                        } else {
                            package::extract_targz(&dest_path, &site_packages)?;
                        }

                        pb.finish_with_message(format!("\x1b[32m✓\x1b[0m {}", info.info.name));
                        installed.insert(pkg_name.clone());
                        total_count += 1;
                    } else {
                        pb.finish_with_message(format!("\x1b[33m!\x1b[0m No wheel for {}", name));
                        installed.insert(pkg_name.clone());
                    }
                } else {
                    pb.finish_and_clear();
                }

                // Sub-dependencies
                if let Some(deps) = info.info.requires_dist {
                    for dep_str in deps {
                        let dep_str_lower = dep_str.to_lowercase();
                        
                        // Heuristic filtering for markers
                        if dep_str_lower.contains("extra ==") { continue; }
                        if cfg!(windows) && dep_str_lower.contains("sys_platform") && !dep_str_lower.contains("win32") { continue; }
                        if cfg!(windows) && (dep_str_lower.contains("linux") || dep_str_lower.contains("unix")) { continue; }
                        if dep_str_lower.contains("python_version < '3") { continue; } // Very basic check

                        let base_dep = dep_str.split(|c: char| c == ';' || c == '(' || c == '<' || c == '>' || c == '=' || c == '!').next().unwrap_or("").trim();
                        if !base_dep.is_empty() {
                            let dep_name = base_dep.to_string();
                            let norm_dep = dep_name.to_lowercase().replace('-', "_");
                            if !installed.contains(&norm_dep) && !processed_deps.contains(&norm_dep) {
                                queue.push_back((dep_name, None, false));
                            }
                        }
                    }
                }
            }
            Err(e) => {
                pb.finish_with_message(format!("\x1b[31m✗\x1b[0m Failed {}: {}", name, e));
            }
        }
    }
    
    println!("\n\x1b[1m\x1b[32m✨ Done!\x1b[0m {} packages ready.", total_count);
    Ok(())
}
