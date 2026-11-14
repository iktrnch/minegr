use clap::{Parser, Subcommand};
use minegr::cli;
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(
        short,
        long,
        global = true,
        value_name = "CONFIG",
        default_value = "minegr.toml"
    )]
    config: PathBuf,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Daemon mode
    Daemon,
    /// Client connection
    Backup,
    Console,
    Init(cli::init::InitArgs),
    Logs,
    Restart,
    Start,
    Status,
    Stop,
    Sync,
}

/// Parses command-line arguments and dispatches the selected command.
fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Init(args)) => {
            cli::init::execute(args, &cli.config)?;
        }
        _ => {
            println!("No command provided. Use --help for more information.");
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "tests/cli.rs"]
mod tests;
