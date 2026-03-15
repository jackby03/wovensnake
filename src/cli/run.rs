use std::error::Error;
use std::path::Path;
use std::process::Command;

use crate::core::config;

pub fn execute(args: &[String]) -> Result<(), Box<dyn Error>> {
    if args.is_empty() {
        return Err("No command specified to run.".into());
    }

    let config = config::read_config("wovenpkg.json")?;
    let venv_base = Path::new(&config.virtual_environment);

    // Determine path to python executable in venv
    let python_path = if cfg!(windows) {
        venv_base.join("Scripts").join("python.exe")
    } else {
        let p = venv_base.join("bin").join("python");
        if p.exists() {
            p
        } else {
            venv_base.join("bin").join("python3")
        }
    };

    if !python_path.exists() {
        return Err(format!(
            "Virtual environment not found at {}. Run 'woven install' first.",
            venv_base.display()
        )
        .into());
    }

    let command_name = &args[0];
    let command_args = &args[1..];

    if command_name.contains('/') || command_name.contains('\\') {
        return Err("Command name cannot contain slashes.".into());
    }

    // Check if command is in venv/Scripts (or bin)
    let scripts_dir = if cfg!(windows) {
        venv_base.join("Scripts")
    } else {
        venv_base.join("bin")
    };

    let executable = scripts_dir.join(command_name);
    let executable_with_exe = scripts_dir.join(format!("{command_name}.exe"));

    let final_cmd = if executable.exists() {
        executable
    } else if cfg!(windows) && executable_with_exe.exists() {
        executable_with_exe
    } else if command_name == "python" || command_name == "python3" {
        python_path
    } else {
        return Err(format!("Command '{}' not found in virtual environment.", command_name).into());
    };

    // Construct command environment
    let path_var = std::env::var_os("PATH").unwrap_or_default();
    let mut paths = std::env::split_paths(&path_var).collect::<Vec<_>>();
    paths.insert(0, scripts_dir);
    let new_path = std::env::join_paths(paths)?;

    let status = Command::new(final_cmd)
        .args(command_args)
        .env("PATH", new_path)
        .env("VIRTUAL_ENV", venv_base)
        .status()?;

    if !status.success() {
        if let Some(code) = status.code() {
            std::process::exit(code);
        } else {
            return Err("Command terminated by signal".into());
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_execute_prevents_command_injection_with_slashes() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("wovenpkg.json");
        fs::write(
            &config_path,
            r#"{"name": "test", "version": "0.1.0", "python_version": "3.10", "pythonVersion": "3.10", "virtual_environment": ".venv", "virtualEnvironment": ".venv", "dependencies": {}}"#,
        )
        .unwrap();

        let venv_path = dir.path().join(".venv");
        let bin_path = if cfg!(windows) {
            venv_path.join("Scripts")
        } else {
            venv_path.join("bin")
        };
        fs::create_dir_all(&bin_path).unwrap();

        let python_path = if cfg!(windows) {
            bin_path.join("python.exe")
        } else {
            bin_path.join("python")
        };
        fs::write(&python_path, "").unwrap();

        // Change current directory to temp dir so read_config finds wovenpkg.json
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();

        let args = vec!["../bin/malicious".to_string()];

        // Use a closure or explicit match to ensure we reset the directory even on panic
        let result = std::panic::catch_unwind(|| execute(&args));

        // Reset current directory
        std::env::set_current_dir(&original_dir).unwrap();

        let result = result.unwrap();

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Command name cannot contain slashes.");
    }
}
