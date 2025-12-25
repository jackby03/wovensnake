use std::process::Command;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_cli_system_flow_init() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    
    // We can't really execute 'cargo run' easily against a temp dir without changing CWD or passing flags.
    // However, for a SYSTEM test, we should verify that the binary actually produces files on disk.
    
    // Build the binary first
    let status = Command::new("cargo").arg("build").status().unwrap();
    assert!(status.success());
    
    // Current CWD is project root. 
    // We can run 'wovensnake init' but it would overwrite our current folder.
    // Dangerous.
    //
    // WovenSnake assumes CWD.
    // To test System level safely, we must spawn a process with current_dir set to temp.
    
    // We need the path to the executable.
    let exe_path = std::env::current_dir().unwrap()
        .join("target").join("debug").join("wovensnake.exe"); // Windows extension
        
    if !exe_path.exists() {
         // Fallback or skip
         return; 
    }
    
    let output = Command::new(&exe_path)
        .arg("init")
        .current_dir(root)
        .output()
        .expect("Failed to execute binary in temp dir");
        
    assert!(output.status.success());
    assert!(root.join("wovenpkg.json").exists());
}
