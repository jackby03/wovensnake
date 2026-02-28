use std::process::Command;
use tempfile::tempdir;

/// End-to-end system test: verifies woven init creates wovenpkg.json on disk.
///
/// Marked #[ignore] because it duplicates acceptance/story_init.rs and previously
/// contained `cargo build` inside the test body (an anti-pattern that inflated
/// test time and hid real failures). The acceptance smoke tests cover this path.
/// Run explicitly with: cargo test -- --ignored
#[test]
#[ignore]
fn test_cli_system_flow_init() {
    let exe_name = if cfg!(windows) { "woven.exe" } else { "woven" };
    let exe_path = std::env::current_dir()
        .unwrap()
        .join("target")
        .join("debug")
        .join(exe_name);

    assert!(
        exe_path.exists(),
        "binary not found — run `cargo build` before executing ignored system tests"
    );

    let dir = tempdir().unwrap();
    let root = dir.path();

    let output = Command::new(&exe_path)
        .args(["init", "--yes"])
        .current_dir(root)
        .output()
        .expect("Failed to execute binary in temp dir");

    assert!(
        output.status.success(),
        "woven init --yes failed:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(root.join("wovenpkg.json").exists(), "wovenpkg.json was not created");
}
