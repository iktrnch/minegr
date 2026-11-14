use anyhow::Context;
use dialoguer::{FuzzySelect, theme::ColorfulTheme};
use serde::Deserialize;
use std::collections::HashMap;

use crate::platform::{CoreChecksum, CoreDownload, Paper, PlatformProvider};
use crate::utils::http::ApiClient;

#[derive(Deserialize)]
struct Project {
    versions: HashMap<String, Vec<String>>,
}

#[derive(Deserialize)]
struct Build {
    downloads: Downloads,
}

#[derive(Deserialize)]
struct Downloads {
    #[serde(rename = "server:default")]
    server: Option<ServerDownload>,
}

#[derive(Deserialize)]
struct ServerDownload {
    url: String,
    size: u64,
    checksums: Checksums,
}

#[derive(Deserialize)]
struct Checksums {
    sha256: String,
}

impl PlatformProvider for Paper {
    fn select_version() -> anyhow::Result<String> {
        let project: Project = ApiClient::new()
            .context("Failed to initialize Paper version selection")?
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

    /// Downloads the first returned Paper build into a temporary file with its SHA-256.
    fn get_core(version: &str) -> anyhow::Result<CoreDownload> {
        let client = ApiClient::new().context("Failed to initialize the Paper server download")?;
        let builds: Vec<Build> = client
            .get(format!(
                "https://fill.papermc.io/v3/projects/paper/versions/{version}/builds"
            ))
            .send()
            .with_context(|| format!("Failed to fetch builds for Paper {version}"))?
            .error_for_status()
            .with_context(|| format!("Paper {version} builds request failed"))?
            .json()
            .with_context(|| format!("Failed to parse builds for Paper {version}"))?;
        let build = builds
            .into_iter()
            .next()
            .with_context(|| format!("No Paper builds are available for {version}"))?;
        let server = build.downloads.server.with_context(|| {
            format!("The first Paper {version} build has no server:default download")
        })?;

        CoreDownload::download(
            &client,
            &server.url,
            Some(server.size),
            Some(CoreChecksum::Sha256(server.checksums.sha256)),
        )
        .with_context(|| format!("Failed to download the Paper {version} server JAR"))
    }
}
