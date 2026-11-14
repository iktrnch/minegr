use anyhow::Context;
use serde::Deserialize;

use crate::platform::{CoreDownload, Fabric, PlatformProvider};
use crate::utils::http::ApiClient;

#[derive(Deserialize)]
struct Version {
    version: String,
    stable: bool,
}

#[derive(Deserialize)]
struct LoaderVersion {
    loader: Version,
}

impl PlatformProvider for Fabric {
    fn select_version() -> anyhow::Result<Box<Vec<String>>> {
        let game_versions: Vec<Version> = ApiClient::new()
            .context("Failed to initialize Fabric version selection")?
            .get("https://meta.fabricmc.net/v2/versions/game")
            .send()
            .context("Failed to fetch Fabric's game versions")?
            .error_for_status()
            .context("Fabric's game versions request failed")?
            .json()
            .context("Failed to parse Fabric's game versions")?;

        // Rayon could parallelize this filter/map, but sequential iterators suffice here.
        // Preserve Fabric's newest-first order and keep all stable versions searchable.
        let versions: Vec<String> = game_versions
            .into_iter()
            .filter(|version| version.stable)
            .map(|version| version.version)
            .collect();

        anyhow::ensure!(
            !versions.is_empty(),
            "Fabric's game metadata contains no stable versions"
        );

        Ok(Box::new(versions))
    }

    /// Downloads the launcher using the first stable loader and installer versions.
    fn get_core(version: &str) -> anyhow::Result<CoreDownload> {
        let client = ApiClient::new().context("Failed to initialize the Fabric server download")?;
        let loaders: Vec<LoaderVersion> = client
            .get(format!(
                "https://meta.fabricmc.net/v2/versions/loader/{version}"
            ))
            .send()
            .with_context(|| format!("Failed to fetch Fabric loaders for Minecraft {version}"))?
            .error_for_status()
            .with_context(|| format!("Fabric loader request for Minecraft {version} failed"))?
            .json()
            .with_context(|| format!("Failed to parse Fabric loaders for Minecraft {version}"))?;
        let loader = loaders
            .into_iter()
            .map(|entry| entry.loader)
            .find(|loader| loader.stable)
            .with_context(|| {
                format!("No stable Fabric loader is available for Minecraft {version}")
            })?;

        let installers: Vec<Version> = client
            .get("https://meta.fabricmc.net/v2/versions/installer")
            .send()
            .context("Failed to fetch Fabric installers")?
            .error_for_status()
            .context("Fabric installer request failed")?
            .json()
            .context("Failed to parse Fabric installers")?;
        let installer = installers
            .into_iter()
            .find(|installer| installer.stable)
            .context("No stable Fabric installer is available")?;

        let url = format!(
            "https://meta.fabricmc.net/v2/versions/loader/{version}/{}/{}/server/jar",
            loader.version, installer.version
        );
        // Fabric generates this launcher without publishing its size or an upstream checksum.
        CoreDownload::download(&client, &url, None, None).with_context(|| {
            format!("Failed to download the Fabric launcher for Minecraft {version}")
        })
    }
}
