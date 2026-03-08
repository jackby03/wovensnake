use std::error::Error;
use std::fs;
use std::str::FromStr;

use pep508_rs::pep440_rs::{Version, VersionSpecifiers};
use pep508_rs::{Requirement, VerbatimUrl, VersionOrUrl};

use crate::cli::install;
use crate::cli::ux;
use crate::core::config;
use crate::dependencies::package;

#[derive(Debug, Clone, PartialEq, Eq)]
struct AddRequest {
    package_name: String,
    requested_specifier: Option<String>,
    fetch_version: Option<String>,
}

fn build_requirement_input(name: &str, version: Option<&str>) -> String {
    match version {
        Some(v)
            if v.starts_with('=')
                || v.starts_with('>')
                || v.starts_with('<')
                || v.starts_with('~')
                || v.starts_with('!') =>
        {
            format!("{name}{v}")
        }
        Some(v) => format!("{name}=={v}"),
        None => name.to_string(),
    }
}

fn fetch_version_for_spec(specifier: Option<&str>) -> Option<String> {
    let spec = specifier?;
    if !spec.starts_with("==") {
        return None;
    }

    let version = spec.trim_start_matches("==");
    if version.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        Some(version.to_string())
    } else {
        None
    }
}

fn parse_add_request(input: &str) -> Result<AddRequest, Box<dyn Error>> {
    let req = Requirement::<VerbatimUrl>::from_str(input)
        .map_err(|e| format!("Invalid dependency expression '{input}': {e}"))?;

    let requested_specifier = match req.version_or_url {
        Some(VersionOrUrl::VersionSpecifier(spec)) => Some(spec.to_string()),
        Some(VersionOrUrl::Url(_)) => {
            return Err("URL requirements are not yet supported by `woven add`.".into());
        }
        None => None,
    };

    Ok(AddRequest {
        package_name: req.name.to_string(),
        fetch_version: fetch_version_for_spec(requested_specifier.as_deref()),
        requested_specifier,
    })
}

fn ensure_resolved_matches_constraint(
    package_name: &str,
    constraint: &str,
    resolved_version: &str,
) -> Result<(), Box<dyn Error>> {
    let specifiers = VersionSpecifiers::from_str(constraint)?;
    let resolved = Version::from_str(resolved_version)?;

    if specifiers.contains(&resolved) {
        Ok(())
    } else {
        Err(
            format!("Resolved version {resolved_version} for {package_name} does not satisfy constraint {constraint}")
                .into(),
        )
    }
}

pub async fn execute(name: &str, version: Option<String>) -> Result<(), Box<dyn Error>> {
    let requirement_input = build_requirement_input(name, version.as_deref());
    let request = parse_add_request(&requirement_input)?;

    ux::print_header(&format!("Adding package {}", request.package_name));

    let config_path = "wovenpkg.json";
    let mut config = config::read_config(config_path)?;

    if config.dependencies.contains_key(&request.package_name) {
        ux::print_warning(format!("Package {} is already in dependencies.", request.package_name));
        return Ok(());
    }

    let info = package::fetch_package_info(&request.package_name, request.fetch_version.as_deref()).await?;
    let resolved_version = info.info.version;

    if let Some(spec) = request.requested_specifier.as_deref() {
        ensure_resolved_matches_constraint(&request.package_name, spec, &resolved_version)?;
    }

    let saved_specifier = request
        .requested_specifier
        .unwrap_or_else(|| format!(">={resolved_version}"));

    ux::print_info(format!(
        "Resolved {} to version {resolved_version} (saved constraint: {saved_specifier})",
        request.package_name
    ));

    config.dependencies.insert(request.package_name, saved_specifier);
    let new_json = serde_json::to_string_pretty(&config)?;
    fs::write(config_path, new_json)?;

    ux::print_success(format!("Updated {config_path}"));

    ux::print_info("Updating environment...");
    install::execute(true).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_requirement_input_without_version() {
        assert_eq!(build_requirement_input("requests", None), "requests");
    }

    #[test]
    fn build_requirement_input_with_plain_version() {
        assert_eq!(build_requirement_input("requests", Some("2.31.0")), "requests==2.31.0");
    }

    #[test]
    fn build_requirement_input_with_specifier_fragment() {
        assert_eq!(build_requirement_input("requests", Some(">=2.25")), "requests>=2.25");
    }

    #[test]
    fn parse_add_request_with_range_specifier() {
        let parsed = parse_add_request("requests>=2.25").expect("parse should succeed");
        assert_eq!(parsed.package_name, "requests");
        assert_eq!(parsed.requested_specifier.as_deref(), Some(">=2.25"));
        assert_eq!(parsed.fetch_version, None);
    }

    #[test]
    fn parse_add_request_with_exact_specifier() {
        let parsed = parse_add_request("requests==2.28.0").expect("parse should succeed");
        assert_eq!(parsed.package_name, "requests");
        assert_eq!(parsed.requested_specifier.as_deref(), Some("==2.28.0"));
        assert_eq!(parsed.fetch_version.as_deref(), Some("2.28.0"));
    }

    #[test]
    fn parse_add_request_without_specifier() {
        let parsed = parse_add_request("requests").expect("parse should succeed");
        assert_eq!(parsed.package_name, "requests");
        assert_eq!(parsed.requested_specifier, None);
        assert_eq!(parsed.fetch_version, None);
    }
}
