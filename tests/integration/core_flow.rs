use wovensnake::core::config;
use wovensnake::core::lock::{Lockfile, LockedPackage};
use wovensnake::core::cache::Cache;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use tempfile::tempdir;

#[test]
fn test_config_parsing() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("wovenpkg.json");
    
    let content = r#"{
        "name": "test-project",
        "version": "0.1.0",
        "python_version": "3.9",
        "virtualEnvironment": "venv",
        "dependencies": {
            "requests": "2.25.1",
            "numpy": "1.21.0"
        }
    }"#;
    
    let mut file = File::create(&config_path).unwrap();
    file.write_all(content.as_bytes()).unwrap();
    
    let config = config::read_config(&config_path).expect("Failed to read config");
    
    assert_eq!(config.name, "test-project");
    assert_eq!(config.version, "0.1.0");
    assert_eq!(config.python_version, "3.9");
    assert_eq!(config.virtual_environment, "venv");
    assert_eq!(config.dependencies.len(), 2);
    assert_eq!(config.dependencies.get("requests").unwrap(), "2.25.1");
}

#[test]
fn test_lockfile_integrity() {
    let dir = tempdir().unwrap();
    let lock_path = dir.path().join("wovenpkg.lock");
    
    let mut lockfile = Lockfile::new("test-project", "0.1.0");
    lockfile.packages.insert("requests".to_string(), LockedPackage {
        version: "2.25.1".to_string(),
        artifacts: vec![],
        dependencies: vec!["urllib3".to_string(), "chardet".to_string()],
    });
    
    lockfile.write(&lock_path).expect("Failed to write lockfile");
    
    // Verify file exists
    assert!(lock_path.exists());
    
    // Read back
    let loaded = Lockfile::read(&lock_path).expect("Failed to read lockfile");
    assert_eq!(loaded.name, "test-project");
    assert_eq!(loaded.packages.len(), 1);
    
    let pkg = loaded.packages.get("requests").unwrap();
    assert_eq!(pkg.version, "2.25.1");
    assert_eq!(pkg.dependencies.len(), 2);
    assert_eq!(pkg.dependencies[0], "urllib3");
}

#[test]
fn test_cache_mechanism() {
    let dir = tempdir().unwrap();
    let cache_root = dir.path().join("cache");
    let project_pkg_dir = dir.path().join("packages");
    fs::create_dir(&project_pkg_dir).unwrap();
    
    let cache = Cache::new(cache_root);
    
    let filename = "test-pkg.whl";
    let sha256 = "1234567890abcdef";
    let content = b"fake package content";
    
    // 1. Save to cache
    let saved_path = cache.save(filename, sha256, content).expect("Failed to save to cache");
    assert!(saved_path.exists());
    assert!(cache.contains(filename, sha256));
    
    // Verify content in cache
    let read_back = fs::read(saved_path).unwrap();
    assert_eq!(read_back, content);
    
    // 2. Link to project
    cache.link_to_project(filename, sha256, &project_pkg_dir).expect("Failed to link");
    
    let linked_path = project_pkg_dir.join(filename);
    assert!(linked_path.exists());
    
    let linked_content = fs::read(linked_path).unwrap();
    assert_eq!(linked_content, content);
}
