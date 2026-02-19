use flate2::read::GzDecoder;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use std::io;
use std::path::Path;
use tar::Archive;
use zip::ZipArchive;

#[derive(Serialize, Deserialize, Debug)]
pub struct PypiPackageInfo {
    pub info: Info,
    pub urls: Vec<PackageUrl>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Info {
    pub name: String,
    pub version: String,
    pub summary: Option<String>,
    pub requires_dist: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PackageUrl {
    pub url: String,
    pub filename: String,
    pub packagetype: String,
    pub digests: Digests,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Digests {
    pub sha256: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PypiFullInfo {
    pub info: Info,
    pub releases: std::collections::HashMap<String, Vec<PackageUrl>>,
}

pub async fn fetch_package_info(name: &str, version: Option<&str>) -> Result<PypiPackageInfo, Box<dyn Error>> {
    let url = version.map_or_else(
        || format!("https://pypi.org/pypi/{name}/json"),
        |v| format!("https://pypi.org/pypi/{name}/{v}/json"),
    );

    let response = reqwest::get(url).await?;

    if response.status().is_success() {
        let info: PypiPackageInfo = response.json().await?;
        Ok(info)
    } else {
        Err(format!("Could not find package {name} on PyPI").into())
    }
}

pub async fn fetch_full_package_info(name: &str) -> Result<PypiFullInfo, Box<dyn Error>> {
    let url = format!("https://pypi.org/pypi/{name}/json");
    let response = reqwest::get(url).await?;
    if response.status().is_success() {
        let info: PypiFullInfo = response.json().await?;
        Ok(info)
    } else {
        Err(format!("Could not find package {name} on PyPI").into())
    }
}

pub async fn download_package(url: &str, dest_path: &Path) -> Result<(), Box<dyn Error>> {
    let response = reqwest::get(url).await?;
    let content = response.bytes().await?;
    fs::write(dest_path, content)?;
    Ok(())
}

pub fn extract_wheel(wheel_path: &Path, dest_path: &Path) -> Result<(), Box<dyn Error>> {
    let file = fs::File::open(wheel_path)?;
    let mut archive = ZipArchive::new(file)?;

    if !dest_path.exists() {
        fs::create_dir_all(dest_path)?;
    }

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = match file.enclosed_name() {
            Some(path) => dest_path.join(path),
            None => continue,
        };

        if (*file.name()).ends_with('/') {
            fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(p)?;
                }
            }
            let mut outfile = fs::File::create(&outpath)?;
            io::copy(&mut file, &mut outfile)?;

            // Preserve Unix permissions stored in the wheel (zip) file
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Some(mode) = file.unix_mode() {
                    let _ = fs::set_permissions(&outpath, fs::Permissions::from_mode(mode));
                }
            }
        }
    }

    Ok(())
}

pub fn generate_scripts(dist_info_path: &Path, scripts_dir: &Path, python_version: &str) -> Result<(), Box<dyn Error>> {
    let entry_points_path = dist_info_path.join("entry_points.txt");
    if !entry_points_path.exists() {
        return Ok(());
    }

    let content = fs::read_to_string(entry_points_path)?;
    let mut in_console_scripts = false;

    if !scripts_dir.exists() {
        fs::create_dir_all(scripts_dir)?;
    }

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if line == "[console_scripts]" {
            in_console_scripts = true;
            continue;
        } else if line.starts_with('[') {
            in_console_scripts = false;
            continue;
        }

        if in_console_scripts {
            if let Some((name, target)) = line.split_once('=') {
                let name = name.trim();
                let target = target.trim().split(' ').next().unwrap_or_else(|| target.trim()); // Ignore [extras]

                if let Some((module, function)) = target.split_once(':') {
                    let script_content = format!(
                        "import sys
from {module} import {function}
if __name__ == '__main__':
    sys.exit({function}())"
                    );

                    // Create .py script
                    let script_path = scripts_dir.join(format!("{name}-script.py"));
                    fs::write(&script_path, script_content)?;

                    // Create .bat for Windows
                    if cfg!(windows) {
                        let bat_content = format!(
                            "@echo off
set PYTHONPATH=%~dp0\\..\\Lib\\site-packages;%PYTHONPATH%
python \"%~dp0\\{name}-script.py\" %*"
                        );
                        let bat_path = scripts_dir.join(format!("{name}.bat"));
                        fs::write(bat_path, bat_content)?;
                    } else {
                        // For Unix
                        let sh_content = format!(
                            "#!/bin/sh
export PYTHONPATH=\"$(dirname \"$0\")/../lib/python{python_version}/site-packages:$PYTHONPATH\"
\"$(dirname \"$0\")/python\" \"$(dirname \"$0\")/{name}-script.py\" \"$@\""
                        );
                        let sh_path = scripts_dir.join(name);
                        fs::write(&sh_path, sh_content)?;

                        #[cfg(unix)]
                        {
                            use std::os::unix::fs::PermissionsExt;
                            if let Ok(metadata) = fs::metadata(&sh_path) {
                                let mut perms = metadata.permissions();
                                perms.set_mode(0o755);
                                let _ = fs::set_permissions(&sh_path, perms);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

pub fn extract_targz(path: &Path, dest_path: &Path) -> Result<(), Box<dyn Error>> {
    let tar_gz = fs::File::open(path)?;
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);

    // Most sdists have a single top-level directory. We want to extract its contents directly if possible,
    // but the 'tar' crate doesn't make it super easy to strip levels without manual iteration.
    // For now, we extract everything.
    archive.unpack(dest_path)?;

    // Post-extraction: Find if there's a nested folder that should be moved up.
    // E.g. site-packages/package-1.0.0/package -> site-packages/package
    if let Ok(entries) = fs::read_dir(dest_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = entry.file_name().into_string().unwrap_or_default();
                if name.contains('-') {
                    // Potential sdist root like atomicwrites-1.4.1
                    // We could move its children up, but that's complex to do during installation safely.
                    // The user can always manually fix or we can implement a .pth generator.
                }
            }
        }
    }

    Ok(())
}
