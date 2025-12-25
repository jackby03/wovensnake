use std::error::Error;
use std::path::Path;
use std::process::Command;

use indicatif::{ProgressBar, ProgressStyle};

use crate::core::python;

pub fn create_venv(path: &Path, python_version: &str) -> Result<(), Box<dyn Error>> {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")?
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈"),
    );
    pb.set_message(format!("Creating virtual environment in {}...", path.display()));

    // Find the correct python executable
    let python_exe = python::find_python_executable(python_version)?;
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
