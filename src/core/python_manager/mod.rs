pub mod downloader;
pub mod metadata;

use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use downloader::download_and_extract_python;

/// Ensures that a specific Python version is available.
/// If not found locally in the managed directory, it downloads it.
pub async fn ensure_python_version(version: &str) -> Result<PathBuf, Box<dyn Error>> {
    let managed_path = get_managed_python_path(version)?;

    // Try to find the executable in common locations within the managed path
    if let Some(exe) = find_executable_in_managed_path(&managed_path) {
        return Ok(exe);
    }

    println!("Python {} not found in managed storage. Downloading...", version);
    if !managed_path.exists() {
        fs::create_dir_all(&managed_path)?;
    }

    if let Err(e) = download_and_extract_python(version, &managed_path).await {
        // Clean up on failure
        let _ = fs::remove_dir_all(&managed_path);
        return Err(e);
    }

    find_executable_in_managed_path(&managed_path).ok_or_else(|| {
        format!(
            "Failed to find python executable after extraction in {}",
            managed_path.display()
        )
        .into()
    })
}

fn find_executable_in_managed_path(managed_path: &Path) -> Option<PathBuf> {
    let candidates = if cfg!(windows) {
        vec![
            managed_path.join("python").join("install").join("python.exe"),
            managed_path.join("install").join("python.exe"),
            managed_path.join("python.exe"),
        ]
    } else {
        vec![
            managed_path.join("python").join("install").join("bin").join("python3"),
            managed_path.join("install").join("bin").join("python3"),
            managed_path.join("bin").join("python3"),
            managed_path.join("python3"),
        ]
    };

    candidates.into_iter().find(|candidate| candidate.exists())
}

fn get_managed_python_path(version: &str) -> Result<PathBuf, Box<dyn Error>> {
    let home = dirs::home_dir().ok_or("Could not find home directory")?;
    let managed_base = home.join(".woven").join("python");
    Ok(managed_base.join(version))
}

/// Lists all Python versions currently managed by `WovenSnake`.
pub fn list_managed_versions() -> Result<Vec<String>, Box<dyn Error>> {
    let home = dirs::home_dir().ok_or("Could not find home directory")?;
    let managed_base = home.join(".woven").join("python");

    if !managed_base.exists() {
        return Ok(vec![]);
    }

    let mut versions = Vec::new();
    for entry in fs::read_dir(managed_base)? {
        let entry = entry?;
        if entry.path().is_dir() {
            if let Some(name) = entry.file_name().to_str() {
                versions.push(name.to_string());
            }
        }
    }
    Ok(versions)
}

/// Removes a specific managed Python version.
pub fn remove_managed_version(version: &str) -> Result<(), Box<dyn Error>> {
    let managed_path = get_managed_python_path(version)?;
    if managed_path.exists() {
        fs::remove_dir_all(managed_path)?;
    }
    Ok(())
}

/// Removes all managed Python versions.
pub fn clear_managed_versions() -> Result<(), Box<dyn Error>> {
    let home = dirs::home_dir().ok_or("Could not find home directory")?;
    let managed_base = home.join(".woven").join("python");
    if managed_base.exists() {
        fs::remove_dir_all(managed_base)?;
    }
    Ok(())
}
