//! Command-line parsing for Minegr.

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum, value_parser};

/// Minegr's public command-line interface.
#[derive(Debug, Parser, PartialEq, Eq)]
#[command(version, about = "Manage Minecraft servers from the terminal")]
pub struct Cli {
    /// Direct path to the instance configuration file.
    #[arg(
        long,
        global = true,
        default_value = "./minegr.toml",
        value_name = "PATH"
    )]
    pub config: PathBuf,

    /// Server-management command to run.
    #[command(subcommand)]
    pub command: Command,
}

/// A user-facing Minegr command.
#[derive(Debug, Subcommand, PartialEq, Eq)]
pub enum Command {
    /// Create or materialize an instance.
    Init(InitArgs),
    /// Capture stopped-server properties into minegr.toml.
    Sync,
    /// Start Java through the per-server daemon.
    Start,
    /// Stop Java and its daemon.
    Stop,
    /// Replace Java while retaining the daemon.
    Restart,
    /// Print lifecycle state and process usage.
    Status,
    /// Read or follow Minecraft's current log through the daemon.
    Logs(LogsArgs),
    /// Open the interactive server console.
    Console,
    /// Archive mutable world files.
    Backup,
}

impl Command {
    /// Returns the stable CLI spelling of this command.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Init(_) => "init",
            Self::Sync => "sync",
            Self::Start => "start",
            Self::Stop => "stop",
            Self::Restart => "restart",
            Self::Status => "status",
            Self::Logs(_) => "logs",
            Self::Console => "console",
            Self::Backup => "backup",
        }
    }
}

/// Arguments accepted while creating a configuration.
#[derive(Debug, Args, PartialEq, Eq)]
pub struct InitArgs {
    /// Server display name.
    #[arg(long)]
    pub name: Option<String>,

    /// Exact Minecraft version.
    #[arg(long)]
    pub minecraft_version: Option<String>,

    /// Server platform.
    #[arg(long, value_enum)]
    pub platform: Option<PlatformArg>,

    /// Heap size used for matching -Xms and -Xmx arguments.
    #[arg(long, default_value = "2G")]
    pub memory: String,

    /// Minecraft server port.
    #[arg(
        long,
        default_value_t = 25_565,
        value_parser = value_parser!(u16).range(1..=65_535)
    )]
    pub port: u16,

    /// Record explicit Minecraft EULA acceptance.
    #[arg(long)]
    pub accept_eula: bool,

    /// Skip the final creation confirmation.
    #[arg(long)]
    pub yes: bool,

    /// Regenerate only the UUID in an existing configuration.
    #[arg(long)]
    pub uuid: bool,
}

/// Platforms available during initial configuration.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum PlatformArg {
    /// Mojang's Vanilla server.
    Vanilla,
    /// PaperMC Paper.
    Paper,
    /// Fabric Loader.
    Fabric,
}

/// Arguments accepted by the log reader.
#[derive(Debug, Args, PartialEq, Eq)]
pub struct LogsArgs {
    /// Initial line count.
    #[arg(long, default_value_t = 1_000, value_parser = value_parser!(u16).range(1..=10_000))]
    pub last: u16,

    /// Continue printing new lines.
    #[arg(long)]
    pub follow: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_the_current_directory_file() {
        let cli = Cli::try_parse_from(["minegr", "status"]).expect("valid command");

        assert_eq!(cli.config, PathBuf::from("./minegr.toml"));
    }

    #[test]
    fn config_is_global_before_or_after_the_command() {
        let before = Cli::try_parse_from(["minegr", "--config", "before.toml", "status"])
            .expect("global option before command");
        let after = Cli::try_parse_from(["minegr", "status", "--config", "after.toml"])
            .expect("global option after command");

        assert_eq!(before.config, PathBuf::from("before.toml"));
        assert_eq!(after.config, PathBuf::from("after.toml"));
    }
}
