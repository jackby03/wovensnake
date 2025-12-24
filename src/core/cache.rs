use std::path::{Path, PathBuf};
use std::fs;
use std::error::Error;

#[derive(Clone)]
pub struct Cache {
    base_dir: PathBuf,
}


impl Cache {
    pub fn new(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }

    pub fn init() -> Result<Self, Box<dyn Error>> {
        let home = dirs::home_dir().ok_or("Could not find home directory")?;
        let cache_dir = home.join(".wovensnake").join("cache");
        
        if !cache_dir.exists() {
            fs::create_dir_all(&cache_dir)?;
        }
        
        Ok(Self::new(cache_dir))
    }

    pub fn get_pkg_path(&self, filename: &str, sha256: &str) -> PathBuf {
        // We use a content-addressable subfolder to avoid filename collisions
        self.base_dir.join(sha256).join(filename)
    }

    pub fn contains(&self, filename: &str, sha256: &str) -> bool {
        self.get_pkg_path(filename, sha256).exists()
    }

    pub fn link_to_project(&self, filename: &str, sha256: &str, project_packages_dir: &Path) -> Result<(), Box<dyn Error>> {
        let cache_path = self.get_pkg_path(filename, sha256);
        let dest_path = project_packages_dir.join(filename);
        
        if !dest_path.exists() {
            // Try hardlinking first for zero-copy, fallback to copy
            if let Err(_) = fs::hard_link(&cache_path, &dest_path) {
                fs::copy(&cache_path, &dest_path)?;
            }
        }
        
        Ok(())
    }

    pub fn save(&self, filename: &str, sha256: &str, data: &[u8]) -> Result<PathBuf, Box<dyn Error>> {
        let pkg_dir = self.base_dir.join(sha256);
        if !pkg_dir.exists() {
            fs::create_dir_all(&pkg_dir)?;
        }
        
        let pkg_path = pkg_dir.join(filename);
        fs::write(&pkg_path, data)?;
        Ok(pkg_path)
    }
}
