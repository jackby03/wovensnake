use clap::{Parser, Subcommand};
use wovensnake::cli;

#[derive(Parser)]
#[command(name = "wovensnake")]
#[command(about = "A Python package manager built with Rust", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new WovenSnake project
    Init,
    /// Install dependencies from wovenpkg.json
    Install,
    /// Update dependencies to their latest versions
    Update,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => {
            if let Err(e) = cli::init::execute() {
                eprintln!("Error during init: {}", e);
            }
        }
        Commands::Install => {
            if let Err(e) = cli::install::execute().await {
                eprintln!("Error during install: {}", e);
            }
        }
        Commands::Update => {
            if let Err(e) = cli::update::execute().await {
                eprintln!("Error during update: {}", e);
            }
        }
    }
}
