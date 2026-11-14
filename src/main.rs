use clap::{Parser, Subcommand};
use minegr::cli;
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long, value_name = "CONFIG")]
    config: Option<PathBuf>,

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
    Init,
    Logs,
    Restart,
    Start,
    Status,
    Stop,
    Sync,
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Init) => {
            cli::init::execute();
        }
        _ => {
            println!("No command provided. Use --help for more information.");
        }
    }
}
