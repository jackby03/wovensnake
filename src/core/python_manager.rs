use flate2::read::GzDecoder;
use futures::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest;
use std::env;
use std::error::Error;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use tar::Archive;
use zip::ZipArchive;

const PYTHON_BUILD_STANDALONE_RELEASE: &str = "20251217";

/// Ensures that a specific Python version is available.
/// If not found locally in the managed directory, it downloads it.
pub async fn ensure_python_version(version: &str) -> Result<PathBuf, Box<dyn Error>> {
    let managed_path = get_managed_python_path(version)?;

    // The structure inside the extracted folder from python-build-standalone
    // usually has a 'python' directory.
    let python_exe = if cfg!(windows) {
        managed_path.join("python").join("install").join("python.exe")
    } else {
        managed_path.join("python").join("install").join("bin").join("python3")
    };

    if python_exe.exists() {
        return Ok(python_exe);
    }

    println!("Python {} not found in managed storage. Downloading...", version);
    download_and_extract_python(version, &managed_path).await?;

    if python_exe.exists() {
        Ok(python_exe)
    } else {
        // Try a fallback path if the structure is different
        let fallback_exe = if cfg!(windows) {
            managed_path.join("python.exe")
        } else {
            managed_path.join("bin").join("python3")
        };

        if fallback_exe.exists() {
            Ok(fallback_exe)
        } else {
            Err(format!(
                "Failed to find python executable after extraction in {}",
                managed_path.display()
            )
            .into())
        }
    }
}

fn get_managed_python_path(version: &str) -> Result<PathBuf, Box<dyn Error>> {
    let home = dirs::home_dir().ok_or("Could not find home directory")?;
    let managed_base = home.join(".woven").join("python").join(version);
    if !managed_base.exists() {
        fs::create_dir_all(&managed_base)?;
    }
    Ok(managed_base)
}

async fn download_and_extract_python(version: &str, dest: &Path) -> Result<(), Box<dyn Error>> {
    let url = get_download_url(version)?;
    let response = reqwest::get(url.clone()).await?;

    if !response.status().is_success() {
        return Err(format!("Failed to download Python from {}: {}", url, response.status()).into());
    }

    let total_size = response.content_length().unwrap_or(0);
    let pb = ProgressBar::new(total_size);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")?
            .progress_chars("#>-"),
    );

    let mut temp_file = tempfile::NamedTempFile::new()?;
    let mut stream = response.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item.map_err(|e| format!("Error downloading: {}", e))?;
        temp_file.write_all(&chunk)?;
        pb.inc(chunk.len() as u64);
    }
    pb.finish_with_message("Download complete");

    println!("Extracting Python...");
    extract_archive(temp_file.path(), dest)?;

    Ok(())
}

fn get_download_url(version: &str) -> Result<String, Box<dyn Error>> {
    let os = env::consts::OS;
    let arch = env::consts::ARCH;

    // Mapping for python-build-standalone assets
    let platform = match (os, arch) {
        ("windows", "x86_64") => "x86_64-pc-windows-msvc-shared-install_only",
        ("linux", "x86_64") => "x86_64-unknown-linux-gnu-install_only",
        ("macos", "x86_64") => "x86_64-apple-darwin-install_only",
        ("macos", "aarch64") => "aarch64-apple-darwin-install_only",
        _ => return Err(format!("Unsupported platform: {} {}", os, arch).into()),
    };

    let extension = if os == "windows" { "zip" } else { "tar.gz" };

    // Map major.minor to a specific patch version available in the release
    let full_version = match version {
        "3.12" => "3.12.8",
        "3.11" => "3.11.11",
        "3.10" => "3.10.16",
        "3.9" => "3.9.21",
        _ => version,
    };

    Ok(format!(
        "https://github.com/astral-sh/python-build-standalone/releases/download/{}/cpython-{}+{}-{}.{}",
        PYTHON_BUILD_STANDALONE_RELEASE, full_version, PYTHON_BUILD_STANDALONE_RELEASE, platform, extension
    ))
}

fn extract_archive(archive_path: &Path, dest: &Path) -> Result<(), Box<dyn Error>> {
    let file = fs::File::open(archive_path)?;

    if archive_path.extension().and_then(|s| s.to_str()) == Some("zip") {
        let mut archive = ZipArchive::new(file)?;
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let outpath = match file.enclosed_name() {
                Some(path) => dest.join(path),
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
            }
        }
    } else {
        let tar = GzDecoder::new(file);
        let mut archive = Archive::new(tar);
        archive.unpack(dest)?;
    }

    Ok(())
}
