use crate::core::lock::Artifact;

pub fn select_artifact<'a>(artifacts: &'a [Artifact], platform: &str) -> Option<&'a Artifact> {
    // 1. Exact platform match
    if let Some(art) = artifacts.iter().find(|a| a.platform == platform) {
        return Some(art);
    }

    // 2. Platform-family fallback (only where binary compatibility is guaranteed)
    if platform == "macosx_arm64" {
        // Apple Silicon can run x86_64 wheels transparently via Rosetta 2
        if let Some(art) = artifacts.iter().find(|a| a.platform == "macosx_x86_64") {
            return Some(art);
        }
    }
    // NOTE: manylinux_aarch64 does NOT fall back to manylinux (x86_64).
    // An x86_64 wheel will not run on ARM64 Linux without emulation — installing
    // it would silently produce a broken environment. Fall through to "any" or source.

    // 3. Universal wheel (py3-none-any, py2.py3-none-any, etc.)
    if let Some(art) = artifacts.iter().find(|a| a.platform == "any") {
        return Some(art);
    }

    // 4. Source distribution
    artifacts
        .iter()
        .find(|a| a.platform == "source" || a.filename.ends_with(".tar.gz"))
}
