use crate::config::Minegr;
use crate::config::{Config, Java, Minecraft};
use crate::platform::Platform;
use crate::platform::properties::Properties;
use anyhow::Context;
use dialoguer::{Confirm, FuzzySelect, Input, Select, theme::ColorfulTheme};
use std::{fs, path::Path};
use strum::VariantNames;

/// Optional command-line inputs for server initialization.
#[derive(Debug, clap::Args)]
pub struct InitArgs {
    /// Display name; defaults to the root directory name when applied.
    #[arg(long, value_name = "NAME")]
    pub name: Option<String>,

    /// Exact Minecraft version.
    #[arg(long, value_name = "VERSION")]
    pub minecraft_version: Option<String>,

    /// Minecraft server platform.
    #[arg(long, value_name = "PLATFORM", value_parser = ["vanilla", "paper", "fabric"])]
    pub platform: Option<String>,

    /// Explicitly accept the Minecraft EULA.
    #[arg(long, num_args = 0..=1, default_missing_value = "true")]
    pub accept_eula: Option<bool>,
}

/// Initializes the selected directory, writing minegr.toml and managed server files there.
pub fn execute(args: &InitArgs, config_dir: &Path) -> anyhow::Result<()> {
    // Check if the directory exists; if not, create it
    if !config_dir.exists() {
        fs::create_dir_all(config_dir).context("Failed to create the configuration directory")?;
    }
    // Check if directory is empty; if not, refuse to initialize
    let is_empty = std::fs::read_dir(config_dir)?.next().transpose()?.is_none();

    if !is_empty {
        anyhow::bail!("Directory is not empty; refusing to initialize");
    }

    // Collect the required information from the user
    let name = args.name.clone().map_or_else(prompt_name, Ok)?;

    let platform = match &args.platform {
        Some(p) => p
            .parse::<Platform>()
            .context("Invalid platform specified")?,
        None => prompt_platform()?,
    };

    let version = match &args.minecraft_version {
        Some(v) => {
            // Validate the version by attempting to select it
            let versions = platform
                .select_version()
                .context("Failed to fetch Minecraft versions")?;
            if !versions.contains(v) {
                anyhow::bail!("Invalid Minecraft version specified for the selected platform");
            }
            v.clone()
        }
        None => prompt_version(&platform)?,
    };

    let eula = args.accept_eula.map_or_else(prompt_eula, Ok)?;

    // Check if the user has accepted the EULA; if not, exit with an error
    if eula {
        fs::write(config_dir.join("eula.txt"), "eula=true").context("Failed to write eula.txt")?;
    } else {
        anyhow::bail!("Agree to EULA to continue")
    }

    let checksum = platform
        .get_core(&version, config_dir)
        .context("Failed to download the server core")?;

    // Create the config
    let config = Config {
        minegr: Minegr::new(name),
        minecraft: Minecraft::new(platform, version, checksum, eula),
        properties: Properties::default(),
        java: Java::default(),
        addons: Vec::new(),
    };

    // Save the config and properties
    config.properties.try_save(config_dir)?;
    config.try_save(&config_dir)?;

    Ok(())
}

fn prompt_name() -> anyhow::Result<String> {
    let name: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Name your instance: ")
        .interact_text()
        .context("Failed to read instance name")?;
    Ok(name)
}

/// Prompts the user to select a platform from the available variants.
fn prompt_platform() -> anyhow::Result<Platform> {
    let platform_index = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select platform")
        .default(0)
        .items(Platform::VARIANTS)
        .interact()?;
    let platform: Platform = Platform::VARIANTS[platform_index]
        .parse()
        .expect("platform variant names must parse as Platform");

    Ok(platform)
}

/// Prompts the user to select a version from the available versions for the given platform.
fn prompt_version(platform: &Platform) -> anyhow::Result<String> {
    let versions = platform
        .select_version()
        .context("Failed to fetch Minecraft versions")?;

    let version_index = FuzzySelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Select Vanilla Minecraft version (type to search)")
        .default(0)
        .items((*versions).clone())
        .max_length(5)
        .interact()
        .context("Failed to select a Vanilla Minecraft version")?;

    Ok(versions[version_index].clone())
}

fn prompt_eula() -> anyhow::Result<bool> {
    let eula = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Do you accept the EULA?")
        .interact()
        .context("Failed to read EULA acceptance")?;
    Ok(eula)
}
