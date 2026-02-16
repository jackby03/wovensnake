use std::error::Error;
use std::process::Command;

/// Validates that the system Python version matches the expected version.
/// Expected version should be in format "3.10", "3.11", etc.
/// If not found in system, it tries to use the managed Python versions.
pub async fn validate_python_version(expected: &str) -> Result<(), Box<dyn Error>> {
    find_python_executable(expected).await?;
    Ok(())
}

/// Finds a Python executable that matches the expected version.
/// Tries system PATH first, then managed versions.
pub async fn find_python_executable(expected: &str) -> Result<String, Box<dyn Error>> {
    // Try specific version first, then 'python', then 'python3'
    let commands = [format!("python{}", expected), "python".to_string(), "python3".to_string()];

    for cmd in commands {
        if let Ok(output) = Command::new(&cmd).arg("--version").output() {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

                // Python 2.x and some 3.x versions print to stderr, others to stdout
                let version_str = if stdout.is_empty() { stderr } else { stdout };

                if version_str.contains(expected) || version_str.starts_with(&format!("Python {expected}")) {
                    return Ok(cmd);
                }
            }
        }
    }

    // Fallback to managed Python versions
    println!("Python {} not found in PATH. Checking managed versions...", expected);
    let managed_exe = crate::core::python_manager::ensure_python_version(expected).await?;
    Ok(managed_exe.to_string_lossy().to_string())
}

/// Attempts to detect the system's default Python version (major.minor).
pub fn get_system_python_version() -> Option<String> {
    let commands = ["python", "python3"];
    for cmd in commands {
        if let Ok(output) = Command::new(cmd).arg("--version").output() {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                let version_str = if stdout.is_empty() { stderr } else { stdout };

                // version_str is usually "Python 3.10.12"
                let parts: Vec<&str> = version_str.split_whitespace().collect();
                if parts.len() >= 2 {
                    let version_num = parts[1];
                    let version_parts: Vec<&str> = version_num.split('.').collect();
                    if version_parts.len() >= 2 {
                        return Some(format!("{}.{}", version_parts[0], version_parts[1]));
                    }
                }
            }
        }
    }
    None
}
