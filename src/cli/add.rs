//! Command used to add a plugin/mod to the server configuration.
use std::path::Path;

use anyhow::Context;

use crate::{
    config::Config,
    modrinth::{get_addon, repository::ReleaseChannel},
};

/// Installs a compatible Modrinth addon and records it in minegr.toml.
#[derive(Debug, clap::Args)]
#[command(allow_missing_positional = true)]
pub struct AddArgs {
    /// Modrinth project slug or ID.
    #[arg(index = 2, value_name = "SLUG")]
    pub slug: String,
    /// Most permissive channel to allow; defaults to alpha, which also permits beta and release.
    #[arg(long, short, value_name = "CHANNEL", value_enum)]
    pub channel: Option<ReleaseChannel>,
}

/// Downloads an addon, verifies it, and saves its pinned record in the selected configuration.
pub fn execute(args: &AddArgs, config_dir: &Path) -> anyhow::Result<()> {
    let mut config = Config::try_load(config_dir)?;
    let addons_dir = config.minecraft.platform.get_addons_dir(config_dir)?;
    let addon = get_addon(&addons_dir, &config, &args.slug, args.channel)?;
    let name = addon.name.clone();
    config.addons.push(addon);
    config.try_save(config_dir).with_context(|| {
        format!(
            "Installed {name}, but saving its record in {}/minegr.toml failed",
            config_dir.display()
        )
    })?;

    eprintln!("  Added {name}");
    eprintln!(
        "  Configuration: {}",
        config_dir.join("minegr.toml").display()
    );
    Ok(())
}
