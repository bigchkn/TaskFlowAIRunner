use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{Shell, generate};

mod commands;
mod config;
mod driver;
mod git;
mod process;
mod taskflow;

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
    /// Generate shell completions
    Completions {
        /// Shell to generate completions for
        shell: Shell,
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
            commands::dispatch::execute(*once)?;
        }
        Commands::Config { command } => match command {
            ConfigCommands::Validate => match config::load_config() {
                Ok(_) => println!("Configuration is valid."),
                Err(e) => println!("Configuration error: {}", e),
            },
        },
        Commands::Completions { shell } => {
            let mut cmd = Cli::command();
            let bin_name = cmd.get_name().to_string();
            generate(*shell, &mut cmd, bin_name, &mut std::io::stdout());
        }
    }

    Ok(())
}
