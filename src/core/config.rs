use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub name: String,
    pub version: String,
    pub python_version: String,
    pub dependencies: HashMap<String, String>,
    #[serde(rename = "virtualEnvironment")]
    pub virtual_environment: String,
}

pub fn read_config<P: AsRef<Path>>(path: P) -> Result<Config, crate::core::error::WovenError> {
    let content = fs::read_to_string(path)?;
    let config: Config = serde_json::from_str(&content)?;
    Ok(config)
}

pub fn write_config<P: AsRef<Path>>(config: &Config, path: P) -> Result<(), crate::core::error::WovenError> {
    let content = serde_json::to_string_pretty(config)?;
    fs::write(path, content)?;
    Ok(())
}
