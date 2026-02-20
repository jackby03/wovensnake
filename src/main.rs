#![warn(clippy::all, clippy::pedantic, clippy::nursery, rust_2018_idioms)]
#![allow(
    clippy::must_use_candidate,
    clippy::module_name_repetitions,
    clippy::missing_errors_doc,
    clippy::too_many_arguments,
    clippy::missing_panics_doc,
    clippy::uninlined_format_args
)]

use clap::{Parser, Subcommand};
use wovensnake::cli;
use wovensnake::cli::ux;

#[derive(Parser)]
#[command(name = "woven")]
#[command(about = "A Python package manager built with Rust", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new `WovenSnake` project
    Init {
        /// Skip interactive prompts and use defaults
        #[arg(short, long)]
        yes: bool,
    },
    /// Add a new package to the project
    Add {
        /// Name of the package to add
        name: String,
        /// Optional version of the package
        version: Option<String>,
    },
    /// Install dependencies from wovenpkg.json
    Install,
    /// Update dependencies to their latest versions
    Update,
    /// Run a command within the virtual environment
    Run {
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Remove a package
    Remove { name: String },
    /// List installed packages
    List,
    /// Clean project dependencies and virtual environment
    Clean {
        /// Also clear the global cache
        #[arg(long)]
        all: bool,
        /// Also clear managed Python versions
        #[arg(long)]
        python: bool,
    },
    /// Uninstall WovenSnake from this machine
    #[command(name = "self-uninstall")]
    SelfUninstall {
        /// Skip confirmation prompt
        #[arg(short, long)]
        yes: bool,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { yes } => {
            if !yes {
                ux::print_welcome();
            }
            if let Err(e) = cli::init::execute(yes) {
                ux::print_error(format!("Failed to initialize project: {e}"));
            }
        }
        Commands::Add { name, version } => {
            if let Err(e) = cli::add::execute(&name, version).await {
                ux::print_error(format!("Failed to add package '{name}': {e}"));
            }
        }
        Commands::Install => {
            if let Err(e) = cli::install::execute(false).await {
                ux::print_error(format!("Installation failed: {e}"));
            }
        }
        Commands::Update => {
            if let Err(e) = cli::update::execute().await {
                ux::print_error(format!("Update failed: {e}"));
            }
        }
        Commands::Run { args } => {
            if let Err(e) = cli::run::execute(&args) {
                ux::print_error(format!("Execution failed: {e}"));
                std::process::exit(1);
            }
        }
        Commands::Remove { name } => {
            if let Err(e) = cli::remove::execute(&name).await {
                ux::print_error(format!("Failed to remove package '{name}': {e}"));
            }
        }
        Commands::List => {
            if let Err(e) = cli::list::execute() {
                ux::print_error(format!("Failed to list packages: {e}"));
            }
        }
        Commands::Clean { all, python } => {
            if let Err(e) = cli::clean::execute(all, python) {
                ux::print_error(format!("Clean failed: {e}"));
            }
        }
        Commands::SelfUninstall { yes } => {
            if let Err(e) = cli::self_uninstall::execute(yes) {
                ux::print_error(format!("Uninstall failed: {e}"));
            }
        }
    }
}
