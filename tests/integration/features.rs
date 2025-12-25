use std::collections::HashMap;
use std::fs;
use tempfile::tempdir;
use wovensnake::core::config;
use wovensnake::core::lock::{LockedPackage, Lockfile};

#[tokio::test]
async fn test_remove_logic_simulation() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("wovenpkg.json");
    let _lock_path = dir.path().join("wovenpkg.lock");

    // 1. Setup Initial State
    let mut config = config::Config {
        name: "test".into(),
        version: "0.1.0".into(),
        python_version: "3.10".into(),
        virtual_environment: "venv".into(),
        dependencies: HashMap::from([
            ("flask".to_string(), "2.0.0".to_string()),
            ("requests".to_string(), "2.25.0".to_string()),
        ]),
    };

    // emulate saving
    let json = serde_json::to_string(&config).unwrap();
    fs::write(&config_path, json).unwrap();

    // 2. Perform Removal Logic
    let pkg_to_remove = "flask";
    assert!(config.dependencies.contains_key(pkg_to_remove));

    config.dependencies.remove(pkg_to_remove);
    assert!(!config.dependencies.contains_key(pkg_to_remove));
    assert!(config.dependencies.contains_key("requests")); // Should stay

    // 3. Verify Config Update
    let new_json = serde_json::to_string(&config).unwrap();
    fs::write(&config_path, new_json).unwrap();

    let re_read = config::read_config(&config_path).unwrap();
    assert_eq!(re_read.dependencies.len(), 1);
    assert_eq!(re_read.dependencies.get("requests").unwrap(), "2.25.0");
}

#[tokio::test]
async fn test_add_logic_simulation() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("wovenpkg.json");

    // 1. Setup Initial State
    let mut config = config::Config {
        name: "test".into(),
        version: "0.1.0".into(),
        python_version: "3.10".into(),
        virtual_environment: "venv".into(),
        dependencies: HashMap::from([("requests".to_string(), "2.25.0".to_string())]),
    };

    let json = serde_json::to_string(&config).unwrap();
    fs::write(&config_path, json).unwrap();

    // 2. Perform Add Logic (Simulated)
    let pkg_to_add = "flask";
    let version_to_add = "2.0.0";

    config
        .dependencies
        .insert(pkg_to_add.to_string(), version_to_add.to_string());

    // 3. Verify Config Update
    let new_json = serde_json::to_string(&config).unwrap();
    fs::write(&config_path, new_json).unwrap();

    let re_read = config::read_config(&config_path).unwrap();
    assert_eq!(re_read.dependencies.len(), 2);
    assert_eq!(re_read.dependencies.get("flask").unwrap(), "2.0.0");
    assert_eq!(re_read.dependencies.get("requests").unwrap(), "2.25.0");
}

#[test]
fn test_lockfile_pruning_logic() {
    // Determine if we can identify packages to remove
    let mut lockfile = Lockfile::new("test", "0.1.0");
    lockfile.packages.insert(
        "flask".into(),
        LockedPackage {
            version: "2.0".into(),
            artifacts: vec![],
            dependencies: vec![],
        },
    );
    lockfile.packages.insert(
        "requests".into(),
        LockedPackage {
            version: "2.25".into(),
            artifacts: vec![],
            dependencies: vec![],
        },
    );

    // Simulating "List" command logic
    assert_eq!(lockfile.packages.len(), 2);

    // Simulate "Prune" logic (conceptually used in install/remove)
    // If we removed flask from config, and re-resolved, the new lockfile would only have requests.
    lockfile.packages.remove("flask");

    assert_eq!(lockfile.packages.len(), 1);
    assert!(lockfile.packages.contains_key("requests"));
}
