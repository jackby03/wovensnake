use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Lockfile {
    pub name: String,
    pub version: String,
    pub packages: HashMap<String, LockedPackage>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LockedPackage {
    pub version: String,
    pub url: String,
    pub filename: String,
    pub sha256: String,
    pub dependencies: Vec<String>,
}

impl Lockfile {
    pub fn new(name: &str, version: &str) -> Self {
        Self {
            name: name.to_string(),
            version: version.to_string(),
            packages: HashMap::new(),
        }
    }

    pub fn write(&self, path: &Path) -> Result<(), Box<dyn Error>> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn read(path: &Path) -> Result<Self, Box<dyn Error>> {
        let content = fs::read_to_string(path)?;
        let lockfile: Lockfile = serde_json::from_str(&content)?;
        Ok(lockfile)
    }
}
