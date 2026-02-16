use crate::cli::ux;
use crate::core::config::Config;
use crate::core::python;
use console::{style, Emoji};
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn execute(non_interactive: bool) -> Result<(), Box<dyn std::error::Error>> {
    ux::print_header("Initializing Project");
    let path = Path::new("wovenpkg.json");

    if path.exists() {
        ux::print_warning("wovenpkg.json already exists in this directory.");
        if non_interactive {
             return Ok(());
        }
        if !Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Do you want to overwrite it?")
            .default(false)
            .interact()?
        {
            return Ok(());
        }
    }

    // Default Values
    let current_dir = std::env::current_dir()?
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("my-project")
        .to_string();
    
    let system_version = python::get_system_python_version().unwrap_or_else(|| "3.12".to_string());


    // Interactive Flow
    let (name, python_version, virtual_environment) = if non_interactive {
        (current_dir, system_version, ".venv".to_string())
    } else {
        // 1. Project Name
        let name: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Project Name")
            .default(current_dir)
            .interact_text()?;

        // 2. Python Version
        let system_option = format!("System ({})", system_version);
        let versions = vec![
            system_option.as_str(),
            "3.12",
            "3.11",
            "3.10",
            "Manual Path",
        ];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select Python Version")
            .default(0)
            .items(&versions)
            .interact()?;

        let py_ver = match selection {
            0 => system_version,
            4 => Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Enter Python Path")
                .interact_text()?,
            _ => versions[selection].to_string(),
        };

        // 3. Virtual Environment
        let venv_options = vec![
            "In-Project (.venv)",
            "Centralized (~/.wovensnake/venvs/...)",
        ];
        let venv_selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Virtual Environment Location")
            .default(0)
            .items(&venv_options)
            .interact()?;

        let venv = if venv_selection == 0 {
            ".venv".to_string()
        } else {
            println!(
                "{}",
                style("Note: Centralized venvs are partially supported. Defaulting to local config for now.")
                    .yellow()
            );
            ".venv".to_string()
        };
        
        (name, py_ver, venv)
    };

    let config = Config {
        name,
        version: "0.1.0".to_string(),
        python_version,
        dependencies: HashMap::new(),
        virtual_environment,
    };

    let json = serde_json::to_string_pretty(&config)?;

    if !non_interactive {
        println!("\n{}", style("Configuration:").bold());
        println!("{}", json);

        if !Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Ready to create?")
            .default(true)
            .interact()?
        {
            println!("{}", style("Aborted.").red());
            return Ok(());
        }
    }

    fs::write(path, json)?;

    if !non_interactive {
        println!(
            "\n{} {}",
            Emoji("✨", ":-)"),
            style("Project initialized successfully!").green()
        );
        println!(
            "Next step: Run {} to install dependencies.",
            style("woven install").cyan()
        );
    } else {
        println!("Initialized project.");
    }

    Ok(())
}
