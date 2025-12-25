use std::error::Error;
use std::process::Command;

/// Validates that the system Python version matches the expected version.
/// Expected version should be in format "3.10", "3.11", etc.
pub fn validate_python_version(expected: &str) -> Result<(), Box<dyn Error>> {
    find_python_executable(expected)?;
    Ok(())
}

/// Finds a Python executable that matches the expected version.
pub fn find_python_executable(expected: &str) -> Result<String, Box<dyn Error>> {
    // Try 'python' first, then 'python3'
    let commands = ["python", "python3"];
    
    for cmd in commands {
        if let Ok(output) = Command::new(cmd).arg("--version").output() {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                
                // Python 2.x and some 3.x versions print to stderr, others to stdout
                let version_str = if !stdout.is_empty() { stdout } else { stderr };
                
                if version_str.contains(expected) {
                    return Ok(cmd.to_string());
                }
            }
        }
    }
    
    Err(format!(
        "Python version mismatch or not found. Expected {}, but could not find a matching executable. Please ensure Python {} is installed and in your PATH.",
        expected, expected
    ).into())
}
