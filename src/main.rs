use clap::{Parser, Subcommand};
use anyhow::Result;

mod config;
mod commands;
mod process;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new taskflow-runner configuration
    Init,
    /// Dispatch tasks to a provider
    Dispatch {
        /// Run once and exit
        #[arg(long)]
        once: bool,
    },
    /// Configuration management
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
}

#[derive(Subcommand)]
enum ConfigCommands {
    /// Validate the current configuration
    Validate,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init => {
            commands::init::execute()?;
        }
        Commands::Dispatch { once } => {
            println!("Dispatching tasks (once: {})...", once);
            // TODO: Implementation will follow in TF-11
        }
        Commands::Config { command } => match command {
            ConfigCommands::Validate => {
                match config::load_config() {
                    Ok(_) => println!("Configuration is valid."),
                    Err(e) => println!("Configuration error: {}", e),
                }
            }
        },
    }

    Ok(())
}
