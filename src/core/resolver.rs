use pep508_rs::pep440_rs::{Version, VersionSpecifiers};
use pep508_rs::{Requirement, VerbatimUrl, VersionOrUrl};
use std::collections::{HashMap, VecDeque};
use std::str::FromStr;

use crate::core::marker;
use crate::dependencies::package::{self, PackageUrl};

#[derive(Debug, Clone)]
pub struct ResolutionNode {
    pub name: String,
    pub version: String,
    pub dependencies: Vec<String>,
    /// Download URLs already fetched during resolution — avoids a second `PyPI` request.
    pub urls: Vec<PackageUrl>,
}

pub struct DependencyGraph {
    pub packages: HashMap<String, ResolutionNode>,
}

/// Resolves dependencies for the project.
#[allow(clippy::implicit_hasher)]
pub async fn resolve(
    root_deps: &HashMap<String, String>,
    python_version: &str,
) -> Result<DependencyGraph, crate::core::error::WovenError> {
    let mut resolved = HashMap::<String, ResolutionNode>::new();
    let mut queue = VecDeque::<(String, Option<String>)>::new();
    let marker_env = marker::build_marker_environment(python_version)?;

    // Start with root dependencies
    for (name, version_req) in root_deps {
        queue.push_back((name.clone(), Some(version_req.clone())));
    }

    while let Some((name, version_constraint)) = queue.pop_front() {
        let name_lower = name.to_lowercase().replace('-', "_");

        if let Some(existing) = resolved.get(&name_lower) {
            // Basic conflict detection: if we have a new constraint, check if existing version satisfies it
            if let Some(constraint_str) = version_constraint {
                if let Ok(specifiers) = VersionSpecifiers::from_str(&constraint_str) {
                    if let Ok(version) = Version::from_str(&existing.version) {
                        if !specifiers.contains(&version) {
                            return Err(format!(
                                "Conflict detected for package {}: existing version {} does not satisfy new constraint {}",
                                name, existing.version, constraint_str
                            ).into());
                        }
                    }
                }
            }
            continue;
        }

        // Determine which version to fetch from PyPI:
        //   1. Exact specifier (==x.y.z)  → fetch the versioned endpoint directly.
        //   2. Range/inequality specifier  → enumerate all releases, select the
        //      highest version that satisfies all constraints (PEP440), then fetch
        //      that specific version.
        //   3. No constraint               → fetch the latest release endpoint.
        let fetch_version: Option<String> = match version_constraint.as_deref() {
            None | Some("") => None,
            Some(constraint_str) => {
                // Try to interpret as a bare exact version like "1.2.3"
                let trimmed = constraint_str.trim_start_matches("==");
                if trimmed.chars().next().is_some_and(|c| c.is_ascii_digit()) {
                    Some(trimmed.to_string())
                } else {
                    // Range / inequality specifier: find the best satisfying version.
                    match VersionSpecifiers::from_str(constraint_str) {
                        Ok(specifiers) => {
                            let full = package::fetch_full_package_info(&name).await?;
                            match package::select_best_candidate(&full.releases, &specifiers) {
                                Some(best) => Some(best),
                                None => {
                                    return Err(format!(
                                        "No version of '{name}' satisfies constraint '{constraint_str}'"
                                    )
                                    .into())
                                }
                            }
                        }
                        Err(_) => None, // malformed specifier → fall back to latest
                    }
                }
            }
        };

        let info = package::fetch_package_info(&name, fetch_version.as_deref()).await?;
        // The info object is consumed directly below
        let mut sub_deps = Vec::new();

        if let Some(requires_dist) = info.info.requires_dist {
            for req_str in requires_dist {
                // Filter by markers
                if marker::should_include_requirement(&req_str, &marker_env) {
                    let req = Requirement::<VerbatimUrl>::from_str(&req_str)?;
                    let sub_name = req.name.to_string();
                    let sub_constraint = match req.version_or_url {
                        Some(VersionOrUrl::VersionSpecifier(spec)) => Some(spec.to_string()),
                        _ => None,
                    };

                    sub_deps.push(sub_name.clone());
                    queue.push_back((sub_name, sub_constraint));
                }
            }
        }

        resolved.insert(
            name_lower,
            ResolutionNode {
                name: info.info.name,
                version: info.info.version,
                dependencies: sub_deps,
                urls: info.urls,
            },
        );
    }

    Ok(DependencyGraph { packages: resolved })
}
