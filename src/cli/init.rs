use crate::config::Config;
use std::fs;

/// Runs the interactive initialization prompts.
pub fn execute() {
    // Create the config
    let config = Config::init();

    let text = toml::to_string_pretty(&config).expect("Failed to parse config into a file");
    fs::write("config.toml", text).expect("Failed to write config file");

    // Get the server core as server.jar
    config
        .minecraft
        .platform
        .get_core(config.minecraft.version)
        .expect("Failed to get core");
}
