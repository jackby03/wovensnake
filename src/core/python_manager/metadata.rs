use once_cell::sync::OnceCell;
use serde::Deserialize;
use std::env;
use std::error::Error;
use std::fs;

#[derive(Deserialize, Debug)]
pub struct MetadataAsset {
    #[serde(rename = "name")]
    _name: String,
    pub version: String,
    pub platform: String,
    pub url: String,
    pub flavor: Option<String>,
    pub shared: Option<bool>,
}

const EMBEDDED_PYTHON_METADATA: &str = include_str!("../../../scripts/data/python_downloads.json");
const PYTHON_METADATA_ENV: &str = "WOVENSNAKE_PYTHON_ASSETS_JSON";
static METADATA_CACHE: OnceCell<Vec<MetadataAsset>> = OnceCell::new();

pub fn get_metadata_entries() -> Result<&'static Vec<MetadataAsset>, Box<dyn Error>> {
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

pub fn resolve_from_metadata(version: &str, platform_key: &str) -> Result<Option<Vec<String>>, Box<dyn Error>> {
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
