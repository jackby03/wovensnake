use pep508_rs::marker::{MarkerEnvironment, MarkerEnvironmentBuilder};
use pep508_rs::{Requirement, VerbatimUrl};
use std::env;
use std::error::Error;
use std::str::FromStr;

/// Build the current environment context for marker evaluation.
///
/// Constructs a `MarkerEnvironment` that represents the current platform,
/// Python version, and implementation details. This is used to evaluate
/// PEP 508 environment markers on dependency strings.
pub fn build_marker_environment(python_version: &str) -> Result<MarkerEnvironment, Box<dyn Error>> {
    let os_name = if cfg!(windows) { "nt" } else { "posix" };

    let sys_platform = if cfg!(windows) {
        "win32"
    } else if cfg!(target_os = "macos") {
        "darwin"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        env::consts::OS
    };

    let platform_machine = env::consts::ARCH;

    let platform_system = if cfg!(windows) {
        "Windows"
    } else if cfg!(target_os = "macos") {
        "Darwin"
    } else if cfg!(target_os = "linux") {
        "Linux"
    } else {
        env::consts::OS
    };

    // Parse Python version into components
    let version_parts: Vec<&str> = python_version.split('.').collect();
    let python_full_version = if version_parts.len() >= 3 {
        python_version.to_string()
    } else if version_parts.len() == 2 {
        format!("{}.0", python_version)
    } else {
        format!("{}.0.0", python_version)
    };

    let python_version_short = if version_parts.len() >= 2 {
        format!("{}.{}", version_parts[0], version_parts[1])
    } else {
        python_version.to_string()
    };

    let builder = MarkerEnvironmentBuilder {
        implementation_name: "cpython",
        implementation_version: &python_full_version,
        os_name,
        platform_machine,
        platform_python_implementation: "CPython",
        platform_release: "",
        platform_system,
        platform_version: "",
        python_full_version: &python_full_version,
        python_version: &python_version_short,
        sys_platform,
    };

    let env = MarkerEnvironment::try_from(builder)?;
    Ok(env)
}

/// Parse a PEP 508 requirement string and check if it should be included
/// for the current environment.
///
/// Uses `Requirement::evaluate_markers()` to properly evaluate all marker
/// types: `sys_platform`, `os_name`, `python_version`, `platform_machine`,
/// `implementation_name`, and combined expressions with `and`/`or`.
///
/// Returns `true` if the requirement should be installed, `false` if its
/// markers exclude it from the current environment.
pub fn should_include_requirement(
    requirement_str: &str,
    marker_env: &MarkerEnvironment,
) -> Result<bool, Box<dyn Error>> {
    let requirement = match Requirement::<VerbatimUrl>::from_str(requirement_str) {
        Ok(req) => req,
        Err(e) => {
            eprintln!("Warning: Failed to parse requirement '{}': {}", requirement_str, e);
            return Ok(true);
        }
    };

    // evaluate_markers returns true if the requirement should be installed
    // (either no markers, or markers match the environment)
    Ok(requirement.evaluate_markers(marker_env, &[]))
}

/// Extract the package name from a PEP 508 requirement string.
///
/// Attempts proper parsing first via `pep508_rs`, falling back to
/// simple string splitting if parsing fails.
pub fn extract_package_name(requirement_str: &str) -> String {
    if let Ok(req) = Requirement::<VerbatimUrl>::from_str(requirement_str) {
        return req.name.to_string();
    }

    // Fallback: split on common delimiters
    requirement_str
        .split([';', '(', '<', '>', '=', '!', ' ', '['])
        .next()
        .unwrap_or(requirement_str)
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_marker_environment() {
        let env = build_marker_environment("3.12");
        assert!(env.is_ok());
    }

    #[test]
    fn test_extract_package_name_simple() {
        assert_eq!(extract_package_name("requests"), "requests");
    }

    #[test]
    fn test_extract_package_name_with_version() {
        assert_eq!(extract_package_name("requests>=2.0.0"), "requests");
    }

    #[test]
    fn test_extract_package_name_with_marker() {
        assert_eq!(
            extract_package_name("requests>=2.0.0; python_version>='3.8'"),
            "requests"
        );
    }

    #[test]
    fn test_should_include_no_marker() {
        let env = build_marker_environment("3.12").unwrap();
        assert!(should_include_requirement("requests>=2.0.0", &env).unwrap());
    }

    #[test]
    #[cfg(windows)]
    fn test_should_include_win32_on_windows() {
        let env = build_marker_environment("3.12").unwrap();
        assert!(should_include_requirement("pywin32; sys_platform=='win32'", &env).unwrap());
    }

    #[test]
    #[cfg(windows)]
    fn test_should_exclude_linux_on_windows() {
        let env = build_marker_environment("3.12").unwrap();
        assert!(!should_include_requirement("uvloop; sys_platform=='linux'", &env).unwrap());
    }

    #[test]
    fn test_should_include_python_version_match() {
        let env = build_marker_environment("3.12").unwrap();
        assert!(should_include_requirement("package; python_version>='3.8'", &env).unwrap());
    }

    #[test]
    fn test_should_exclude_old_python_version() {
        let env = build_marker_environment("3.12").unwrap();
        assert!(!should_include_requirement("package; python_version<'3.10'", &env).unwrap());
    }

    #[test]
    fn test_should_handle_combined_markers() {
        let env = build_marker_environment("3.12").unwrap();
        // Python 3.12 >= 3.8 should be true regardless of the AND with os_name
        let result = should_include_requirement("package; python_version>='3.8' and os_name=='nt'", &env).unwrap();
        // On Windows this should be true, on other platforms false
        if cfg!(windows) {
            assert!(result);
        } else {
            assert!(!result);
        }
    }
}
