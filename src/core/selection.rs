use crate::core::lock::Artifact;

pub fn select_artifact<'a>(artifacts: &'a [Artifact], platform: &str) -> Option<&'a Artifact> {
    // 1. Try exact platform match (e.g. win_amd64)
    if let Some(art) = artifacts.iter().find(|a| a.platform == platform) {
        return Some(art);
    }

    // 2. Try "any" (universal wheel)
    if let Some(art) = artifacts.iter().find(|a| a.platform == "any") {
        return Some(art);
    }

    // 3. Fallback to sdist (usually platform="source" or similar, here we assume empty or specific tag)
    // For now, let's look for "sdist" or just take the first source-like thing.
    artifacts
        .iter()
        .find(|a| a.platform == "source" || a.filename.ends_with(".tar.gz"))
}
