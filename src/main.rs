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
    Init,
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
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => {
            if let Err(e) = cli::init::execute() {
                eprintln!("Error during init: {e}");
            }
        }
        Commands::Install => {
            if let Err(e) = cli::install::execute().await {
                eprintln!("Error during install: {e}");
            }
        }
        Commands::Update => {
            if let Err(e) = cli::update::execute().await {
                eprintln!("Error during update: {e}");
            }
        }
        Commands::Run { args } => {
            if let Err(e) = cli::run::execute(&args) {
                eprintln!("Error during run: {e}");
                std::process::exit(1);
            }
        }
        Commands::Remove { name } => {
            if let Err(e) = cli::remove::execute(&name).await {
                eprintln!("Error during remove: {e}");
            }
        }
        Commands::List => {
            if let Err(e) = cli::list::execute() {
                eprintln!("Error during list: {e}");
            }
        }
    }
}
