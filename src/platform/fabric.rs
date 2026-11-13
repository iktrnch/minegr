use anyhow::Context;
use dialoguer::{FuzzySelect, theme::ColorfulTheme};
use serde::Deserialize;

#[derive(Deserialize)]
struct GameVersion {
    version: String,
    stable: bool,
}

/// Fetches stable Fabric game versions and selects one with a five-row fuzzy picker.
pub fn select_fabric_version() -> anyhow::Result<String> {
    let game_versions: Vec<GameVersion> = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?
        .get("https://meta.fabricmc.net/v2/versions/game")
        .send()
        .context("Failed to fetch Fabric's game versions")?
        .error_for_status()
        .context("Fabric's game versions request failed")?
        .json()
        .context("Failed to parse Fabric's game versions")?;

    // Rayon could parallelize this filter/map, but sequential iterators suffice here.
    // Preserve Fabric's newest-first order and keep all stable versions searchable.
    let mut versions: Vec<String> = game_versions
        .into_iter()
        .filter(|version| version.stable)
        .map(|version| version.version)
        .collect();
    anyhow::ensure!(
        !versions.is_empty(),
        "Fabric's game metadata contains no stable versions"
    );

    let selected = FuzzySelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Select Fabric Minecraft version (type to search)")
        .default(0)
        .items(&versions)
        .max_length(5)
        .interact()
        .context("Failed to select a Fabric Minecraft version")?;

    Ok(versions.swap_remove(selected))
}
