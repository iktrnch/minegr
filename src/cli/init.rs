use crate::config::Config;
use std::fs;

/// Runs the interactive initialization prompts.
pub fn execute() -> anyhow::Result<()> {
    // Create the config
    let config = Config::init();

    // Agree to the EULA
    if (&config).minecraft.eula {
        fs::write("eula.txt", "eula=true").expect("Failed to write eula.txt");
    } else {
        anyhow::bail!("Agree to EULA to continue")
    }

    // Get the server core as server.jar
    config
        .minecraft
        .platform
        .get_core(&config.minecraft.version)?;

    config.properties.try_save()?;

    config.try_save()?;
    Ok(())
}
