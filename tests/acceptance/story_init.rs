use std::process::Command;
use tempfile::tempdir;

fn woven_exe() -> std::path::PathBuf {
    let exe_name = if cfg!(windows) { "woven.exe" } else { "woven" };
    std::env::current_dir().unwrap().join("target/debug").join(exe_name)
}

/// Smoke test: `woven --version` must print the current version and exit 0.
#[test]
fn smoke_version() {
    let exe = woven_exe();
    assert!(exe.exists(), "binary not found at {exe:?} — run `cargo build` first");
    let out = Command::new(&exe).arg("--version").output().unwrap();
    assert!(out.status.success(), "woven --version exited non-zero");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains(env!("CARGO_PKG_VERSION")),
        "version string not found in output: {stdout}"
    );
}

/// Smoke test: `woven --help` must exit 0.
#[test]
fn smoke_help() {
    let exe = woven_exe();
    assert!(exe.exists(), "binary not found at {exe:?} — run `cargo build` first");
    let out = Command::new(&exe).arg("--help").output().unwrap();
    assert!(out.status.success(), "woven --help exited non-zero");
}

/// User Story: "As a user, I want to initialize a project and see default config"
///
/// Uses --yes so the command runs non-interactively (no TTY required in CI).
#[test]
fn uat_user_story_create_project() {
    let exe = woven_exe();
    assert!(exe.exists(), "binary not found at {exe:?} — run `cargo build` first");

    let dir = tempdir().unwrap();
    let root = dir.path();

    // ACCEPTANCE CRITERIA 1: Run init non-interactively
    let out = Command::new(&exe)
        .args(["init", "--yes"])
        .current_dir(root)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "woven init --yes failed:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );

    // ACCEPTANCE CRITERIA 2: Config file exists
    let config_file = root.join("wovenpkg.json");
    assert!(config_file.exists(), "wovenpkg.json was not created");

    // ACCEPTANCE CRITERIA 3: Default dependencies are empty
    let content = std::fs::read_to_string(&config_file).unwrap();
    assert!(
        content.contains("\"dependencies\": {}"),
        "dependencies should be empty on init"
    );
}
