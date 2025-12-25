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
        venv_base.join("bin").join("python")
    };

    if !python_path.exists() {
        return Err(format!(
            "Virtual environment not found at {}. Run 'wovensnake install' first.",
            venv_base.display()
        )
        .into());
    }

    let command_name = &args[0];
    let command_args = &args[1..];

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
        Path::new(command_name).to_path_buf()
    };

    // Construct command environment
    let path_var = std::env::var("PATH").unwrap_or_default();
    let new_path = format!("{};{}", scripts_dir.display(), path_var); // Windows separator ;

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
