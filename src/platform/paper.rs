use anyhow::Context;
use dialoguer::{FuzzySelect, theme::ColorfulTheme};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
struct Project {
    versions: HashMap<String, Vec<String>>,
}

/// Fetches Paper release versions and selects one with a five-row fuzzy picker.
pub fn select_paper_version() -> anyhow::Result<String> {
    let project: Project = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .user_agent(concat!(
            env!("CARGO_PKG_NAME"),
            "/",
            env!("CARGO_PKG_VERSION"),
            " (https://github.com/iktrnch/minegr)"
        ))
        .build()?
        .get("https://fill.papermc.io/v3/projects/paper")
        .send()
        .context("Failed to fetch Paper's project metadata")?
        .error_for_status()
        .context("Paper's project metadata request failed")?
        .json()
        .context("Failed to parse Paper's project metadata")?;

    // Rayon could parallelize this filter/map, but sequential iterators suffice here.
    // Numeric components exclude prereleases and sort 1.21.10 ahead of 1.21.9.
    let mut releases: Vec<(Vec<u32>, String)> = project
        .versions
        .into_values()
        .flatten()
        .filter_map(|version| {
            let components = version
                .split('.')
                .map(str::parse::<u32>)
                .collect::<Result<Vec<_>, _>>()
                .ok()?;
            Some((components, version))
        })
        .collect();
    releases.sort_unstable_by(|(left, _), (right, _)| right.cmp(left));
    releases.dedup_by(|left, right| left.1 == right.1);
    let mut versions: Vec<String> = releases.into_iter().map(|(_, version)| version).collect();
    anyhow::ensure!(
        !versions.is_empty(),
        "Paper's project metadata contains no releases"
    );

    let selected = FuzzySelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Select Paper Minecraft version (type to search)")
        .default(0)
        .items(&versions)
        .max_length(5)
        .interact()
        .context("Failed to select a Paper Minecraft version")?;

    Ok(versions.swap_remove(selected))
}
