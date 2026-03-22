use std::collections::HashMap;
use wovensnake::core::config::Config;
use wovensnake::core::lock::Artifact;
use wovensnake::core::selection::select_artifact;
use wovensnake::dependencies::package::{select_best_candidate, Digests, PackageUrl};

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
fn test_select_artifact_linux_aarch64_prefers_any_over_x86_64() {
    // x86_64 manylinux wheels are not compatible with ARM64 Linux.
    // When only an x86_64 wheel and a universal wheel exist, the universal
    // wheel must be selected — NOT the incompatible x86_64 binary.
    let artifacts = vec![
        make_artifact("manylinux", "pkg-1.0-cp311-cp311-manylinux_2_17_x86_64.whl"),
        make_artifact("any", "pkg-1.0-py3-none-any.whl"),
    ];
    let result = select_artifact(&artifacts, "manylinux_aarch64");
    assert!(result.is_some());
    assert_eq!(result.unwrap().platform, "any");
}

#[test]
fn test_select_artifact_linux_aarch64_no_fallback_to_x86_64_when_no_any() {
    // When only an incompatible x86_64 wheel exists and no universal wheel,
    // select_artifact must return None rather than silently install a broken package.
    let artifacts = vec![make_artifact(
        "manylinux",
        "pkg-1.0-cp311-cp311-manylinux_2_17_x86_64.whl",
    )];
    let result = select_artifact(&artifacts, "manylinux_aarch64");
    assert!(
        result.is_none(),
        "should not select an incompatible x86_64 wheel on aarch64"
    );
}

// ── PEP440 candidate-selection unit tests ────────────────────────────────────

/// Build a synthetic releases map from a list of version strings.
/// URLs are empty because select_best_candidate only examines the map keys.
fn make_releases(versions: &[&str]) -> HashMap<String, Vec<wovensnake::dependencies::package::PackageUrl>> {
    use wovensnake::dependencies::package::{Digests, PackageUrl};
    versions
        .iter()
        .map(|v| {
            let url = PackageUrl {
                url: format!("https://files.example.com/{v}.whl"),
                filename: format!("{v}.whl"),
                packagetype: "bdist_wheel".to_string(),
                digests: Digests {
                    sha256: "deadbeef".to_string(),
                },
            };
            (v.to_string(), vec![url])
        })
        .collect()
}

#[test]
fn test_pep440_exact() {
    use std::str::FromStr;
    let releases = make_releases(&["1.2.2", "1.2.3", "1.3.0"]);
    let specs = pep508_rs::pep440_rs::VersionSpecifiers::from_str("==1.2.3").unwrap();
    assert_eq!(select_best_candidate(&releases, &specs).as_deref(), Some("1.2.3"));
}

#[test]
fn test_pep440_range() {
    use std::str::FromStr;
    let releases = make_releases(&["0.9", "1.0", "1.5", "2.0"]);
    let specs = pep508_rs::pep440_rs::VersionSpecifiers::from_str(">=1.0,<2.0").unwrap();
    assert_eq!(select_best_candidate(&releases, &specs).as_deref(), Some("1.5"));
}

#[test]
fn test_pep440_tilde_equal() {
    use std::str::FromStr;
    // ~=1.4 means >=1.4, <2.0
    let releases = make_releases(&["1.3", "1.4", "1.5", "2.0"]);
    let specs = pep508_rs::pep440_rs::VersionSpecifiers::from_str("~=1.4").unwrap();
    assert_eq!(select_best_candidate(&releases, &specs).as_deref(), Some("1.5"));
}

#[test]
fn test_pep440_not_equal() {
    use std::str::FromStr;
    let releases = make_releases(&["1.0", "1.4", "1.5", "1.6"]);
    let specs = pep508_rs::pep440_rs::VersionSpecifiers::from_str("!=1.5,>=1.0").unwrap();
    assert_eq!(select_best_candidate(&releases, &specs).as_deref(), Some("1.6"));
}

#[test]
fn test_pep440_no_match_returns_none() {
    use std::str::FromStr;
    let releases = make_releases(&["1.0", "2.0"]);
    let specs = pep508_rs::pep440_rs::VersionSpecifiers::from_str(">=99.0").unwrap();
    assert!(select_best_candidate(&releases, &specs).is_none());
}

#[test]
fn test_pep440_prerelease_skipped_when_stable_exists() {
    use std::str::FromStr;
    let releases = make_releases(&["1.0", "2.0a1"]);
    let specs = pep508_rs::pep440_rs::VersionSpecifiers::from_str(">=1.0").unwrap();
    // 2.0a1 satisfies >=1.0 but is pre-release; 1.0 is stable and should win.
    assert_eq!(select_best_candidate(&releases, &specs).as_deref(), Some("1.0"));
}
