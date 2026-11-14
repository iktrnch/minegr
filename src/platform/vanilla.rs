use crate::{
    platform::{CoreChecksum, CoreDownload, PlatformProvider, Vanilla},
    utils::http::ApiClient,
};
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
    url: String,
}

#[derive(Deserialize)]
struct VersionMetadata {
    downloads: Downloads,
}

#[derive(Deserialize)]
struct Downloads {
    server: Option<ServerDownload>,
}

#[derive(Deserialize)]
struct ServerDownload {
    url: String,
    sha1: String,
    size: u64,
}

impl Manifest {
    /// Fetches Mojang's version manifest using the supplied HTTP client.
    fn fetch(client: &reqwest::blocking::Client) -> anyhow::Result<Self> {
        client
            .get("https://piston-meta.mojang.com/mc/game/version_manifest_v2.json")
            .send()
            .context("Failed to fetch Mojang's version manifest")?
            .error_for_status()
            .context("Mojang's version manifest request failed")?
            .json()
            .context("Failed to parse Mojang's version manifest")
    }
}

impl PlatformProvider for Vanilla {
    fn select_version() -> anyhow::Result<String> {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .context("Failed to build the Vanilla HTTP client")?;
        let manifest = Manifest::fetch(&client).context("Failed to load Vanilla versions")?;

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

    /// Downloads the exact Vanilla version into a temporary file with its SHA-1 checksum.
    fn get_core(version: String) -> anyhow::Result<CoreDownload> {
        let client = ApiClient::new().context("Failed to build the Vanilla HTTP client")?;
        let manifest = Manifest::fetch(&client).context("Failed to load Vanilla versions")?;
        let entry = manifest
            .versions
            .into_iter()
            .find(|entry| entry.id == version)
            .with_context(|| format!("Vanilla Minecraft version {version} was not found"))?;

        let metadata: VersionMetadata = client
            .get(&entry.url)
            .send()
            .with_context(|| format!("Failed to fetch metadata for Vanilla {version}"))?
            .error_for_status()
            .with_context(|| format!("Vanilla {version} metadata request failed"))?
            .json()
            .with_context(|| format!("Failed to parse metadata for Vanilla {version}"))?;
        let server = metadata
            .downloads
            .server
            .with_context(|| format!("Vanilla {version} has no server download"))?;

        CoreDownload::download(
            &client,
            &server.url,
            Some(server.size),
            Some(CoreChecksum::Sha1(server.sha1)),
        )
        .with_context(|| format!("Failed to download the Vanilla {version} server JAR"))
    }
}
