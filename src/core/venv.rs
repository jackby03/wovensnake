use std::error::Error;
use std::path::Path;
use std::process::Command;

use indicatif::{ProgressBar, ProgressStyle};

use crate::core::python;

pub async fn create_venv(path: &Path, python_version: &str) -> Result<(), Box<dyn Error>> {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")?
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈"),
    );
    pb.set_message(format!("Creating virtual environment in {}...", path.display()));

    // Find the correct python executable
    let python_exe = python::find_python_executable(python_version).await?;
    let version_output = Command::new(&python_exe).arg("-V").output()?;
    let version_str = String::from_utf8_lossy(&version_output.stdout);
    pb.set_message(format!(
        "Using {} ({}) to create venv...",
        python_exe,
        version_str.trim()
    ));

    let output = Command::new(python_exe).arg("-m").arg("venv").arg(path).output();

    match output {
        Ok(out) if out.status.success() => {
            pb.finish_with_message("\x1b[32m✓\x1b[0m Virtual environment created.".to_string());
            Ok(())
        }
        Ok(out) => {
            pb.finish_and_clear();
            let err = String::from_utf8_lossy(&out.stderr);
            Err(format!("Failed to create venv: {err}").into())
        }
        Err(e) => {
            pb.finish_and_clear();
            Err(format!("Could not find python to create venv: {e}").into())
        }
    }
}

/// Gets the Python version of an existing virtual environment.
pub fn get_venv_python_version(path: &Path) -> Result<String, Box<dyn Error>> {
    let python_exe = if cfg!(windows) {
        path.join("Scripts").join("python.exe")
    } else {
        let p = path.join("bin").join("python");
        if p.exists() {
            p
        } else {
            path.join("bin").join("python3")
        }
    };

    if !python_exe.exists() {
        return Err("Python executable not found in virtual environment".into());
    }

    let output = Command::new(python_exe).arg("--version").output()?;
    if !output.status.success() {
        return Err("Failed to get Python version from virtual environment".into());
    }

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let version_str = if stdout.is_empty() { stderr } else { stdout };

    // version_str is usually "Python 3.10.12"
    let parts: Vec<&str> = version_str.split_whitespace().collect();
    if parts.len() >= 2 {
        let version_num = parts[1];
        let version_parts: Vec<&str> = version_num.split('.').collect();
        if version_parts.len() >= 2 {
            return Ok(format!("{}.{}", version_parts[0], version_parts[1]));
        }
    }

    Err("Could not parse Python version from virtual environment".into())
}
