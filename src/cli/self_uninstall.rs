use std::error::Error;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

use crate::cli::ux;

/// Paths that the installer may have modified to add woven to PATH.
#[cfg(unix)]
const RC_FILES: &[&str] = &[".bashrc", ".zshrc", ".profile", ".bash_profile"];

pub fn execute(yes: bool) -> Result<(), Box<dyn Error>> {
    ux::print_header("Uninstalling WovenSnake...");

    // ── 1. Locate the running binary ──────────────────────────────────────────
    let exe_path = std::env::current_exe()?;
    ux::print_info(format!("Binary: {}", exe_path.display()));

    // ── 2. Confirm ────────────────────────────────────────────────────────────
    if !yes && !confirm("Remove the woven binary and global cache?")? {
        println!("Aborted.");
        return Ok(());
    }

    // ── 3. Remove binary ──────────────────────────────────────────────────────
    // On Unix we can unlink ourselves while still running; on Windows we
    // schedule deletion via a rename trick instead.
    remove_binary(&exe_path)?;
    ux::print_success(format!("Removed binary: {}", exe_path.display()));

    // ── 4. Remove global data directory (~/.wovensnake) ───────────────────────
    if let Some(home) = dirs::home_dir() {
        let data_dir = home.join(".wovensnake");
        if data_dir.exists() {
            fs::remove_dir_all(&data_dir)?;
            ux::print_success(format!("Removed data directory: {}", data_dir.display()));
        }
    }

    // ── 5. Clean PATH entries from shell rc files (Unix only) ─────────────────
    #[cfg(unix)]
    clean_path_from_rc_files();

    // ── 6. Done ───────────────────────────────────────────────────────────────
    println!();
    ux::print_success("WovenSnake has been uninstalled.");

    #[cfg(windows)]
    ux::print_info(
        "On Windows, also remove the woven directory from your user PATH in System Properties.",
    );

    Ok(())
}

fn confirm(prompt: &str) -> Result<bool, Box<dyn Error>> {
    print!("  {} [y/N] ", prompt);
    io::stdout().flush()?;
    let mut line = String::new();
    io::stdin().read_line(&mut line)?;
    Ok(matches!(line.trim().to_lowercase().as_str(), "y" | "yes"))
}

fn remove_binary(path: &PathBuf) -> Result<(), Box<dyn Error>> {
    #[cfg(unix)]
    {
        fs::remove_file(path)?;
    }
    #[cfg(windows)]
    {
        // On Windows, rename to a temp name so the OS can delete it after exit.
        let tmp = path.with_extension("exe.delete");
        fs::rename(path, &tmp)?;
        // Schedule deletion on reboot as a best-effort.
        // The renamed file will be left behind on older Windows; inform user.
        ux::print_warning(format!(
            "Windows: binary renamed to {}. You may delete it manually.",
            tmp.display()
        ));
    }
    Ok(())
}

#[cfg(unix)]
fn clean_path_from_rc_files() {
    let Some(home) = dirs::home_dir() else {
        return;
    };

    let patterns = [
        r#"export PATH="$HOME/.local/bin:$PATH""#,
        r#"export PATH="$HOME/.local/bin:$PATH" # added by woven installer"#,
    ];

    for rc_name in RC_FILES {
        let rc = home.join(rc_name);
        if !rc.exists() {
            continue;
        }
        let Ok(content) = fs::read_to_string(&rc) else {
            continue;
        };
        let cleaned: String = content
            .lines()
            .filter(|line| !patterns.iter().any(|p| line.trim() == *p))
            .map(|l| format!("{l}\n"))
            .collect();
        if cleaned != content {
            if fs::write(&rc, cleaned).is_ok() {
                ux::print_success(format!("Cleaned PATH entry from {}", rc.display()));
            }
        }
    }
}
