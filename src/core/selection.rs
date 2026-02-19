use crate::core::lock::Artifact;

pub fn select_artifact<'a>(artifacts: &'a [Artifact], platform: &str) -> Option<&'a Artifact> {
    // 1. Exact platform match
    if let Some(art) = artifacts.iter().find(|a| a.platform == platform) {
        return Some(art);
    }

    // 2. Platform-family fallback
    match platform {
        "macosx_arm64" => {
            // Apple Silicon can run x86_64 wheels via Rosetta 2
            if let Some(art) = artifacts.iter().find(|a| a.platform == "macosx_x86_64") {
                return Some(art);
            }
        }
        "manylinux_aarch64" => {
            // Fall back to x86_64 manylinux (won't run natively, but handles the generic case)
            if let Some(art) = artifacts.iter().find(|a| a.platform == "manylinux") {
                return Some(art);
            }
        }
        _ => {}
    }

    // 3. Universal wheel (py3-none-any, py2.py3-none-any, etc.)
    if let Some(art) = artifacts.iter().find(|a| a.platform == "any") {
        return Some(art);
    }

    // 4. Source distribution
    artifacts
        .iter()
        .find(|a| a.platform == "source" || a.filename.ends_with(".tar.gz"))
}
