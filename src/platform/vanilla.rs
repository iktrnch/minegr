use anyhow::Context;
use dialoguer::{FuzzySelect, theme::ColorfulTheme};
use serde::Deserialize;

#[derive(Deserialize)]
struct Manifest {
    versions: Vec<Version>,
}

#[derive(Deserialize)]
struct Version {
    id: String,
    #[serde(rename = "type")]
    kind: String,
}

pub fn select_vanilla_version() -> anyhow::Result<String> {
    let manifest: Manifest = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?
        .get("https://piston-meta.mojang.com/mc/game/version_manifest_v2.json")
        .send()
        .context("Failed to fetch Mojang's version manifest")?
        .error_for_status()
        .context("Mojang's version manifest request failed")?
        .json()
        .context("Failed to parse Mojang's version manifest")?;

    // Rayon could parallelize this filter/map, but sequential iterators suffice here.
    // Preserve Mojang's newest-first order and keep all releases searchable.
    let versions: Vec<String> = manifest
        .versions
        .into_iter()
        .filter(|version| version.kind == "release")
        .map(|version| version.id)
        .collect();
    anyhow::ensure!(
        !versions.is_empty(),
        "Mojang's manifest contains no releases"
    );

    let selected = FuzzySelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Select Vanilla Minecraft version (type to search)")
        .default(0)
        .items(&versions)
        .max_length(5)
        .interact()
        .context("Failed to select a Vanilla Minecraft version")?;

    Ok(versions[selected].clone())
}
