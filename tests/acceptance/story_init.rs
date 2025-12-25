use std::process::Command;
use tempfile::tempdir;

#[test]
fn uat_user_story_create_project() {
    // User Story: "As a user, I want to initialize a project and see default config"
    
    let dir = tempdir().unwrap();
    let root = dir.path();
    let exe = std::env::current_dir().unwrap().join("target/debug/wovensnake.exe");
    
    if !exe.exists() { return; } // Skip if not built

    // ACCEPANCE CRITERIA 1: Run init
    let init_cmd = Command::new(&exe)
        .arg("init")
        .current_dir(root)
        .output()
        .unwrap();
    assert!(init_cmd.status.success());
    
    // ACCEPTANCE CRITERIA 2: Config file exists
    let config_file = root.join("wovenpkg.json");
    assert!(config_file.exists());
    
    // ACCEPTANCE CRITERIA 3: Default dependencies are empty
    let content = std::fs::read_to_string(config_file).unwrap();
    assert!(content.contains("\"dependencies\": {}"));
}
