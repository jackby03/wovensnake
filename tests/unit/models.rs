use std::collections::HashMap;
use wovensnake::core::config::Config;
use wovensnake::core::lock::Artifact;
use wovensnake::core::selection::select_artifact;

#[test]
fn test_config_model() {
    let conf = Config {
        name: "test".into(),
        version: "1.0".into(),
        python_version: "3.8".into(),
        virtual_environment: "env".into(),
        dependencies: HashMap::from([("pip".into(), "20.0".into())]),
    };

    // Pure logic assertions
    assert_eq!(conf.name, "test");
    assert!(conf.dependencies.contains_key("pip"));
}

fn make_artifact(platform: &str, filename: &str) -> Artifact {
    Artifact {
        url: format!("https://files.pythonhosted.org/{filename}"),
        filename: filename.to_string(),
        sha256: "abc123".to_string(),
        platform: platform.to_string(),
    }
}

#[test]
fn test_select_artifact_exact_match() {
    let artifacts = vec![
        make_artifact("win_amd64", "numpy-1.24.0-cp311-cp311-win_amd64.whl"),
        make_artifact("macosx_arm64", "numpy-1.24.0-cp311-cp311-macosx_11_0_arm64.whl"),
        make_artifact("any", "numpy-1.24.0-py3-none-any.whl"),
    ];
    let result = select_artifact(&artifacts, "macosx_arm64");
    assert!(result.is_some());
    assert_eq!(result.unwrap().platform, "macosx_arm64");
}

#[test]
fn test_select_artifact_macos_arm64_falls_back_to_x86_64() {
    let artifacts = vec![
        make_artifact("win_amd64", "pkg-1.0-cp311-cp311-win_amd64.whl"),
        make_artifact("macosx_x86_64", "pkg-1.0-cp311-cp311-macosx_10_9_x86_64.whl"),
    ];
    let result = select_artifact(&artifacts, "macosx_arm64");
    assert!(result.is_some());
    assert_eq!(result.unwrap().platform, "macosx_x86_64");
}

#[test]
fn test_select_artifact_falls_back_to_universal() {
    let artifacts = vec![
        make_artifact("win_amd64", "pkg-1.0-cp311-cp311-win_amd64.whl"),
        make_artifact("any", "pkg-1.0-py3-none-any.whl"),
    ];
    let result = select_artifact(&artifacts, "macosx_arm64");
    assert!(result.is_some());
    assert_eq!(result.unwrap().platform, "any");
}

#[test]
fn test_select_artifact_falls_back_to_source() {
    let artifacts = vec![
        make_artifact("win_amd64", "pkg-1.0-cp311-cp311-win_amd64.whl"),
        make_artifact("source", "pkg-1.0.tar.gz"),
    ];
    let result = select_artifact(&artifacts, "macosx_arm64");
    assert!(result.is_some());
    assert_eq!(result.unwrap().platform, "source");
}

#[test]
fn test_select_artifact_linux_aarch64_falls_back_to_manylinux() {
    let artifacts = vec![
        make_artifact("manylinux", "pkg-1.0-cp311-cp311-manylinux_2_17_x86_64.whl"),
        make_artifact("any", "pkg-1.0-py3-none-any.whl"),
    ];
    let result = select_artifact(&artifacts, "manylinux_aarch64");
    assert!(result.is_some());
    assert_eq!(result.unwrap().platform, "manylinux");
}
