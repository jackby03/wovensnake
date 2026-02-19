use flate2::read::GzDecoder;
use futures::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use once_cell::sync::OnceCell;
use reqwest;
use serde::Deserialize;
use std::env;
use std::error::Error;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use tar::Archive;
use zip::ZipArchive;

#[derive(Deserialize, Debug)]
struct GithubRelease {
    #[serde(rename = "tag_name")]
    _tag_name: String,
    assets: Vec<GithubAsset>,
}

#[derive(Deserialize, Debug)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
}

#[derive(Deserialize, Debug)]
struct MetadataAsset {
    #[serde(rename = "name")]
    _name: String,
    version: String,
    platform: String,
    url: String,
    flavor: Option<String>,
    shared: Option<bool>,
}

const PYTHON_BUILD_STANDALONE_REPO: &str = "astral-sh/python-build-standalone";
const EMBEDDED_PYTHON_METADATA: &str = include_str!("../../scripts/data/python_downloads.json");
const PYTHON_METADATA_ENV: &str = "WOVENSNAKE_PYTHON_ASSETS_JSON";
static METADATA_CACHE: OnceCell<Vec<MetadataAsset>> = OnceCell::new();

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
    let managed_base = home.join(".woven").join("python").join(version);
    Ok(managed_base)
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

async fn download_and_extract_python(version: &str, dest: &Path) -> Result<(), Box<dyn Error>> {
    let urls = resolve_python_assets(version).await?;
    let mut response = None;
    let mut last_url = String::new();

    for url in urls {
        last_url.clone_from(&url);
        let client = reqwest::Client::builder().user_agent("wovensnake").build()?;
        match client.get(&url).send().await {
            Ok(res) if res.status().is_success() => {
                response = Some(res);
                break;
            }
            Ok(res) => {
                println!("  Notice: URL {} returned {}", url, res.status());
            }
            Err(e) => {
                println!("  Notice: Failed to connect to {}: {}", url, e);
            }
        }
    }

    let response = response.ok_or_else(|| {
        format!(
            "Failed to download Python for version {}. No matching assets found or all downloads failed.",
            version
        )
    })?;

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
    let is_zip = Path::new(&last_url)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("zip"));
    extract_archive(temp_file.path(), dest, is_zip)?;

    // Post-installation patches (inspired by uv)
    if let Err(e) = apply_post_install_patches(dest, version) {
        println!("  Warning: Failed to apply post-install patches: {}", e);
    }

    Ok(())
}

fn apply_post_install_patches(dest: &Path, version: &str) -> Result<(), Box<dyn Error>> {
    // 1. Create EXTERNALLY-MANAGED file (PEP 668)
    // This prevents users from accidentally using pip to modify this managed python
    let install_dir = find_install_dir(dest).ok_or("Could not find install directory")?;

    let lib_dir = if cfg!(windows) {
        install_dir.join("Lib")
    } else {
        // On Unix, it's usually lib/python3.x
        let major_minor = version.split('.').take(2).collect::<Vec<_>>().join(".");
        install_dir.join("lib").join(format!("python{}", major_minor))
    };

    if lib_dir.exists() {
        let em_path = lib_dir.join("EXTERNALLY-MANAGED");
        if !em_path.exists() {
            let content = "[externally-managed]\nError=This Python installation is managed by WovenSnake.\n";
            fs::write(em_path, content)?;
        }
    }

    // 2. Ensure canonical executables (symlinks/copies)
    if cfg!(windows) {
        // On Windows: ensure python3.x.exe exists alongside python.exe
        let python_exe = install_dir.join("python.exe");
        if python_exe.exists() {
            let major_minor = version.split('.').take(2).collect::<Vec<_>>().join(".");
            let python_ver_exe = install_dir.join(format!("python{}.exe", major_minor));
            if !python_ver_exe.exists() {
                fs::copy(&python_exe, &python_ver_exe)?;
            }
        }
    } else {
        // On Unix: ensure 'python' symlink exists alongside 'python3'
        let bin_dir = install_dir.join("bin");
        let python3 = bin_dir.join("python3");
        let python = bin_dir.join("python");
        if python3.exists() && !python.exists() {
            #[cfg(unix)]
            {
                let _ = std::os::unix::fs::symlink(&python3, &python);
            }
        }
    }

    Ok(())
}

fn find_install_dir(base: &Path) -> Option<PathBuf> {
    // python-build-standalone usually extracts to a 'python' folder, then 'install'
    let candidates = vec![
        base.join("python").join("install"),
        base.join("install"),
        base.to_path_buf(),
    ];

    for c in candidates {
        if c.join(if cfg!(windows) { "python.exe" } else { "bin/python3" })
            .exists()
        {
            return Some(c);
        }
    }
    None
}

fn get_metadata_entries() -> Result<&'static Vec<MetadataAsset>, Box<dyn Error>> {
    METADATA_CACHE.get_or_try_init(load_metadata_entries)
}

fn load_metadata_entries() -> Result<Vec<MetadataAsset>, Box<dyn Error>> {
    if let Ok(path) = env::var(PYTHON_METADATA_ENV) {
        let content = fs::read_to_string(path)?;
        let parsed: Vec<MetadataAsset> = serde_json::from_str(&content)?;
        return Ok(parsed);
    }
    let parsed: Vec<MetadataAsset> = serde_json::from_str(EMBEDDED_PYTHON_METADATA)?;
    Ok(parsed)
}

fn resolve_from_metadata(version: &str, platform_key: &str) -> Result<Option<Vec<String>>, Box<dyn Error>> {
    let entries = get_metadata_entries()?;
    let mut matches = Vec::new();
    for asset in entries {
        if !asset.platform.starts_with(platform_key) {
            continue;
        }

        let score = version_match_score(&asset.version, version);
        if score == 0 {
            continue;
        }

        let flavor_bonus = if asset.flavor.as_deref() == Some("install_only") {
            20
        } else {
            0
        };
        let shared_bonus = if asset.shared.unwrap_or(false) { 15 } else { 0 };
        matches.push((score + flavor_bonus + shared_bonus, asset.url.clone()));
    }

    if matches.is_empty() {
        return Ok(None);
    }

    matches.sort_by(|a, b| b.0.cmp(&a.0));
    let urls: Vec<String> = matches.into_iter().map(|(_, url)| url).collect();
    println!(
        "  INFO: Using embedded metadata catalog ({} entries) for platform {}.",
        urls.len(),
        platform_key
    );
    Ok(Some(urls))
}

fn version_match_score(asset_version: &str, requested_version: &str) -> i32 {
    if asset_version == requested_version {
        return 1000;
    }
    if asset_version.starts_with(&format!("{}.", requested_version)) {
        return 600;
    }
    if requested_version.starts_with(asset_version) {
        return 500;
    }
    let req_parts: Vec<_> = requested_version.split('.').collect();
    let asset_parts: Vec<_> = asset_version.split('.').collect();
    if req_parts.len() >= 2
        && asset_parts.len() >= 2
        && req_parts[0] == asset_parts[0]
        && req_parts[1] == asset_parts[1]
    {
        return 300;
    }
    0
}

async fn resolve_python_assets(version: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let os = env::consts::OS;
    let arch = env::consts::ARCH;

    let platform_key = match (os, arch) {
        ("windows", "x86_64") => "x86_64-pc-windows-msvc",
        ("linux", "x86_64") => "x86_64-unknown-linux",
        ("linux", "aarch64") => "aarch64-unknown-linux",
        ("macos", "x86_64") => "x86_64-apple-darwin",
        ("macos", "aarch64") => "aarch64-apple-darwin",
        _ => return Err(format!("Unsupported platform: {} {}", os, arch).into()),
    };

    // Try to load from local cache first
    if let Ok(cached_urls) = load_cached_assets(version, platform_key) {
        if !cached_urls.is_empty() {
            return Ok(cached_urls);
        }
    }

    if let Some(metadata_urls) = resolve_from_metadata(version, platform_key)? {
        return Ok(metadata_urls);
    }

    println!("Resolving Python {} for {} via GitHub API...", version, platform_key);

    let client = reqwest::Client::builder().user_agent("wovensnake").build()?;

    // We fetch releases page by page until we find at least one matching asset
    let mut all_assets = Vec::new();
    for page in 1..=5 {
        let url = format!(
            "https://api.github.com/repos/{}/releases?page={}",
            PYTHON_BUILD_STANDALONE_REPO, page
        );

        let mut request = client.get(&url);
        if let Ok(token) = env::var("GITHUB_TOKEN") {
            request = request.header("Authorization", format!("token {}", token));
        }

        let releases: Vec<GithubRelease> = match request.send().await {
            Ok(res) => {
                if res.status() == 403 {
                    return Err("GitHub API rate limit exceeded. Please try again later or set a GITHUB_TOKEN.".into());
                }
                if !res.status().is_success() {
                    println!(
                        "  Warning: GitHub API returned {} on page {}. Body will be skipped.",
                        res.status(),
                        page
                    );
                    break;
                }
                res.json().await?
            }
            Err(e) => {
                println!("  Warning: Failed to fetch releases from GitHub (page {}): {}", page, e);
                break;
            }
        };

        if releases.is_empty() {
            break;
        }

        let mut found_in_page = false;
        for release in releases {
            for asset in release.assets {
                let name = asset.name.to_lowercase();
                let version_prefix = format!("cpython-{}", version);

                if name.starts_with(&version_prefix) && name.contains(platform_key) {
                    let score = if name.contains("install_only") { 100 } else { 0 }
                        + if name.contains("shared") { 50 } else { 0 }
                        + if name.contains("gnu") && os == "linux" { 25 } else { 0 };

                    all_assets.push((score, asset.browser_download_url));
                    found_in_page = true;
                }
            }
        }

        // If we found matches in this page, we probably don't need to look further
        // unless we want to be absolutely sure we have the "best" one, but usually
        // the latest releases are in the first pages.
        if found_in_page {
            break;
        }
    }

    // Sort by score descending
    all_assets.sort_by(|a, b| b.0.cmp(&a.0));

    let urls: Vec<String> = all_assets.into_iter().map(|(_, url)| url).collect();

    if urls.is_empty() {
        return Err(format!(
            "Could not find any Python assets for version {} on platform {}",
            version, platform_key
        )
        .into());
    }

    // Save to cache
    let _ = save_cached_assets(version, platform_key, &urls);

    Ok(urls)
}

fn get_cache_path() -> Result<PathBuf, Box<dyn Error>> {
    let home = dirs::home_dir().ok_or("Could not find home directory")?;
    let cache_dir = home.join(".woven").join("cache");
    if !cache_dir.exists() {
        fs::create_dir_all(&cache_dir)?;
    }
    Ok(cache_dir.join("python_assets.json"))
}

fn load_cached_assets(version: &str, platform: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let path = get_cache_path()?;
    if !path.exists() {
        return Ok(vec![]);
    }

    let content = fs::read_to_string(path)?;
    let cache: serde_json::Value = serde_json::from_str(&content)?;

    let key = format!("{}-{}", version, platform);
    if let Some(urls) = cache.get(&key) {
        let urls: Vec<String> = serde_json::from_value(urls.clone())?;
        return Ok(urls);
    }

    Ok(vec![])
}

fn save_cached_assets(version: &str, platform: &str, urls: &[String]) -> Result<(), Box<dyn Error>> {
    let path = get_cache_path()?;
    let mut cache: serde_json::Map<String, serde_json::Value> = if path.exists() {
        let content = fs::read_to_string(&path)?;
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        serde_json::Map::new()
    };

    let key = format!("{}-{}", version, platform);
    cache.insert(key, serde_json::to_value(urls)?);

    let content = serde_json::to_string_pretty(&cache)?;
    fs::write(path, content)?;
    Ok(())
}

fn extract_archive(archive_path: &Path, dest: &Path, is_zip: bool) -> Result<(), Box<dyn Error>> {
    let file = fs::File::open(archive_path)?;

    if is_zip {
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
