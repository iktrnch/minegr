//! Resolution and verified acquisition of pinned Minecraft server artifacts.

use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;

use reqwest::StatusCode;
use reqwest::blocking::{Client, Response};
use rustix::fs::statvfs;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use sha1::Sha1;
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::config::{MinecraftConfig, Platform};
use crate::managed_files::{ManagedFileError, materialize_managed_bytes};

const USER_AGENT: &str = concat!(
    "minegr/",
    env!("CARGO_PKG_VERSION"),
    " (https://github.com/iktrnch/minegr)"
);

/// Authoritative service locations used for artifact resolution.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Endpoints {
    /// Mojang version manifest.
    pub mojang_manifest: String,
    /// Paper project metadata.
    pub paper_project: String,
    /// Paper build endpoint containing a `{version}` placeholder.
    pub paper_builds: String,
    /// Fabric-supported game versions.
    pub fabric_games: String,
    /// Fabric Loader endpoint containing a `{version}` placeholder.
    pub fabric_loaders: String,
    /// Fabric Installer versions.
    pub fabric_installers: String,
    /// Fabric server launcher endpoint containing version placeholders.
    pub fabric_server: String,
}

impl Endpoints {
    /// Returns the upstream endpoints fixed by Minegr's architecture contract.
    pub fn official() -> Self {
        Self {
            mojang_manifest:
                "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json".to_owned(),
            paper_project: "https://fill.papermc.io/v3/projects/paper".to_owned(),
            paper_builds:
                "https://fill.papermc.io/v3/projects/paper/versions/{version}/builds".to_owned(),
            fabric_games: "https://meta.fabricmc.net/v2/versions/game".to_owned(),
            fabric_loaders: "https://meta.fabricmc.net/v2/versions/loader/{version}".to_owned(),
            fabric_installers: "https://meta.fabricmc.net/v2/versions/installer".to_owned(),
            fabric_server: "https://meta.fabricmc.net/v2/versions/loader/{version}/{loader}/{installer}/server/jar".to_owned(),
        }
    }
}

/// Mojang's classification of a selectable Minecraft version.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MinecraftVersionKind {
    /// A production release.
    Release,
    /// A snapshot, pre-release, or release candidate.
    Snapshot,
}

/// One exact Minecraft version from Mojang's newest-first manifest.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MinecraftVersion {
    /// Exact version identifier.
    pub id: String,
    /// Release or snapshot classification.
    pub kind: MinecraftVersionKind,
    /// Authoritative per-version metadata URL.
    pub metadata_url: String,
}

impl MinecraftVersion {
    /// Creates a version value from already validated upstream fields.
    pub fn new(
        id: impl Into<String>,
        metadata_url: impl Into<String>,
        kind: MinecraftVersionKind,
    ) -> Self {
        Self {
            id: id.into(),
            kind,
            metadata_url: metadata_url.into(),
        }
    }
}

/// Platforms with an eligible artifact for a selected Minecraft version.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlatformAvailability {
    /// Mojang publishes a dedicated server artifact.
    pub vanilla: bool,
    /// Paper lists the exact Minecraft version.
    pub paper: bool,
    /// Fabric lists the game version and a stable compatible Loader.
    pub fabric: bool,
}

/// Exact coordinates and download information selected once during initialization.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedArtifact {
    /// Selected server platform.
    pub platform: Platform,
    /// Exact Minecraft version.
    pub version: String,
    /// Exact Paper build ID.
    pub build: Option<u64>,
    /// Exact Fabric Loader version.
    pub loader: Option<String>,
    /// Exact Fabric Installer version.
    pub installer: Option<String>,
    /// Published digest in `<algorithm>:<hex>` form.
    pub checksum: Option<String>,
    /// Immutable artifact URL resolved for these coordinates.
    pub download_url: String,
    /// Java major required by the pinned Minecraft version.
    pub required_java_major: u32,
}

/// A failure while resolving, verifying, or publishing a server artifact.
#[derive(Debug, Error)]
pub enum ArtifactError {
    /// The HTTP client could not be constructed.
    #[error("failed to create upstream HTTP client: {0}")]
    Client(#[source] reqwest::Error),
    /// An upstream request could not be completed.
    #[error("upstream request failed for {url}: {source}")]
    Request {
        /// Requested URL.
        url: String,
        /// Transport error.
        source: reqwest::Error,
    },
    /// An artifact response could not be copied into unique temporary storage.
    #[error("failed to download artifact from {url}: {source}")]
    Download {
        /// Requested artifact URL.
        url: String,
        /// Streaming I/O error.
        source: io::Error,
    },
    /// An upstream service returned a non-success status.
    #[error("upstream request returned HTTP {status} for {url}")]
    Status {
        /// Requested URL.
        url: String,
        /// Returned status.
        status: StatusCode,
    },
    /// An upstream JSON response did not match the required schema.
    #[error("invalid upstream metadata from {url}: {source}")]
    Metadata {
        /// Requested URL.
        url: String,
        /// JSON decoding error.
        source: reqwest::Error,
    },
    /// The exact version is absent from a required catalog.
    #[error("Minecraft version `{version}` is unavailable from {service}")]
    VersionUnavailable {
        /// Requested exact version.
        version: String,
        /// Human-facing upstream name.
        service: &'static str,
    },
    /// No eligible stable artifact exists for the requested platform/version pair.
    #[error("no eligible {platform} artifact exists for Minecraft {version}")]
    NoEligibleArtifact {
        /// Lowercase platform name.
        platform: &'static str,
        /// Requested exact Minecraft version.
        version: String,
    },
    /// A required checksum string uses an unsupported or malformed format.
    #[error("unsupported artifact checksum `{0}`")]
    UnsupportedChecksum(String),
    /// Downloaded bytes did not match the pinned upstream digest.
    #[error("artifact checksum mismatch: expected {expected}, got {actual}")]
    ChecksumMismatch {
        /// Pinned checksum.
        expected: String,
        /// Computed checksum.
        actual: String,
    },
    /// Unique owner-only temporary storage could not be created or written.
    #[error("failed to use temporary artifact storage: {0}")]
    Temporary(#[source] std::io::Error),
    /// Available space in the server root could not be inspected.
    #[error("failed to inspect server-root disk space: {0}")]
    DiskInspect(#[source] std::io::Error),
    /// The verified artifact cannot fit on the destination filesystem.
    #[error(
        "server-root disk space is insufficient: {available} bytes available, {required} required"
    )]
    DiskSpace {
        /// Bytes currently available to the invoking user.
        available: u64,
        /// Bytes required for the verified artifact.
        required: u64,
    },
    /// The verified artifact conflicts with or could not be published to `server.jar`.
    #[error("failed to materialize server.jar: {0}")]
    ManagedFile(#[from] ManagedFileError),
}

/// Blocking upstream client used by one CLI invocation.
#[derive(Debug)]
pub struct ArtifactService {
    client: Client,
    endpoints: Endpoints,
}

impl ArtifactService {
    /// Creates a service for the supplied endpoint set.
    pub fn new(endpoints: Endpoints) -> Result<Self, ArtifactError> {
        let client = Client::builder()
            .user_agent(USER_AGENT)
            .build()
            .map_err(ArtifactError::Client)?;
        Ok(Self { client, endpoints })
    }

    /// Creates a service using Minegr's authoritative upstream endpoints.
    pub fn official() -> Result<Self, ArtifactError> {
        Self::new(Endpoints::official())
    }

    /// Returns Mojang's selectable versions in upstream newest-first order.
    pub fn versions(&self) -> Result<Vec<MinecraftVersion>, ArtifactError> {
        let manifest: MojangManifest = self.get_json(&self.endpoints.mojang_manifest)?;
        Ok(manifest
            .versions
            .into_iter()
            .map(|version| MinecraftVersion {
                id: version.id,
                kind: if version.kind == "release" {
                    MinecraftVersionKind::Release
                } else {
                    MinecraftVersionKind::Snapshot
                },
                metadata_url: version.url,
            })
            .collect())
    }

    /// Finds one exact version in Mojang's manifest without substituting another version.
    pub fn find_version(&self, id: &str) -> Result<MinecraftVersion, ArtifactError> {
        self.versions()?
            .into_iter()
            .find(|version| version.id == id)
            .ok_or_else(|| ArtifactError::VersionUnavailable {
                version: id.to_owned(),
                service: "Mojang",
            })
    }

    /// Determines which documented platforms have an eligible artifact for one version.
    pub fn available_platforms(
        &self,
        version: &MinecraftVersion,
    ) -> Result<PlatformAvailability, ArtifactError> {
        let details: MojangVersion = self.get_json(&version.metadata_url)?;
        let paper: PaperProject = self.get_json(&self.endpoints.paper_project)?;
        let paper = if paper
            .versions
            .values()
            .flatten()
            .any(|candidate| candidate == &version.id)
        {
            let url = replace(&self.endpoints.paper_builds, "version", &version.id);
            let builds: Vec<PaperBuild> = self.get_json(&url)?;
            builds.iter().any(|build| {
                build.channel == "STABLE" && build.downloads.contains_key("server:default")
            })
        } else {
            false
        };
        let fabric_games: Vec<FabricGame> = self.get_json(&self.endpoints.fabric_games)?;
        let fabric = if fabric_games.iter().any(|game| game.version == version.id) {
            let url = replace(&self.endpoints.fabric_loaders, "version", &version.id);
            let loaders: Vec<FabricLoaderEntry> = self.get_json(&url)?;
            if loaders.iter().any(|entry| entry.loader.stable) {
                let installers: Vec<FabricInstaller> =
                    self.get_json(&self.endpoints.fabric_installers)?;
                installers.iter().any(|installer| installer.stable)
            } else {
                false
            }
        } else {
            false
        };
        Ok(PlatformAvailability {
            vanilla: details.downloads.server.is_some(),
            paper,
            fabric,
        })
    }

    /// Resolves one exact eligible platform artifact without selecting fallbacks.
    pub fn resolve(
        &self,
        version: &MinecraftVersion,
        platform: Platform,
    ) -> Result<ResolvedArtifact, ArtifactError> {
        let details: MojangVersion = self.get_json(&version.metadata_url)?;
        match platform {
            Platform::Vanilla => self.resolve_vanilla(version, details),
            Platform::Paper => self.resolve_paper(version, details.java_version.major_version),
            Platform::Fabric => self.resolve_fabric(version, details.java_version.major_version),
        }
    }

    /// Resolves the exact coordinates already pinned in an existing configuration.
    pub fn resolve_pinned(
        &self,
        config: &MinecraftConfig,
    ) -> Result<ResolvedArtifact, ArtifactError> {
        let version = self.find_version(&config.version)?;
        let details: MojangVersion = self.get_json(&version.metadata_url)?;
        let required_java_major = details.java_version.major_version;
        match config.platform {
            Platform::Vanilla => {
                let server =
                    details
                        .downloads
                        .server
                        .ok_or_else(|| ArtifactError::NoEligibleArtifact {
                            platform: "vanilla",
                            version: config.version.clone(),
                        })?;
                Ok(ResolvedArtifact {
                    platform: Platform::Vanilla,
                    version: config.version.clone(),
                    build: None,
                    loader: None,
                    installer: None,
                    checksum: config
                        .checksum
                        .clone()
                        .or_else(|| Some(format!("sha1:{}", server.sha1))),
                    download_url: server.url,
                    required_java_major,
                })
            }
            Platform::Paper => {
                let url = replace(&self.endpoints.paper_builds, "version", &config.version);
                let builds: Vec<PaperBuild> = self.get_json(&url)?;
                let expected_build =
                    config
                        .build
                        .ok_or_else(|| ArtifactError::NoEligibleArtifact {
                            platform: "paper",
                            version: config.version.clone(),
                        })?;
                let build = builds
                    .into_iter()
                    .find(|build| build.id == expected_build)
                    .ok_or_else(|| ArtifactError::NoEligibleArtifact {
                        platform: "paper",
                        version: config.version.clone(),
                    })?;
                let download = build.downloads.get("server:default").ok_or_else(|| {
                    ArtifactError::NoEligibleArtifact {
                        platform: "paper",
                        version: config.version.clone(),
                    }
                })?;
                let upstream_checksum = download
                    .checksums
                    .as_ref()
                    .and_then(|checksums| checksums.sha256.as_ref())
                    .map(|digest| format!("sha256:{digest}"));
                Ok(ResolvedArtifact {
                    platform: Platform::Paper,
                    version: config.version.clone(),
                    build: Some(expected_build),
                    loader: None,
                    installer: None,
                    checksum: config.checksum.clone().or(upstream_checksum),
                    download_url: download.url.clone(),
                    required_java_major,
                })
            }
            Platform::Fabric => {
                let loader =
                    config
                        .loader
                        .as_deref()
                        .ok_or_else(|| ArtifactError::NoEligibleArtifact {
                            platform: "fabric",
                            version: config.version.clone(),
                        })?;
                let installer = config.installer.as_deref().ok_or_else(|| {
                    ArtifactError::NoEligibleArtifact {
                        platform: "fabric",
                        version: config.version.clone(),
                    }
                })?;
                Ok(ResolvedArtifact {
                    platform: Platform::Fabric,
                    version: config.version.clone(),
                    build: None,
                    loader: Some(loader.to_owned()),
                    installer: Some(installer.to_owned()),
                    checksum: config.checksum.clone(),
                    download_url: replace_many(
                        &self.endpoints.fabric_server,
                        &[
                            ("version", config.version.as_str()),
                            ("loader", loader),
                            ("installer", installer),
                        ],
                    ),
                    required_java_major,
                })
            }
        }
    }

    /// Downloads, verifies, and atomically publishes a pinned artifact as `server.jar`.
    pub fn download(
        &self,
        artifact: &ResolvedArtifact,
        server_root: &Path,
    ) -> Result<(), ArtifactError> {
        let temporary = tempfile::Builder::new()
            .prefix("minegr-artifact-")
            .tempdir()
            .map_err(ArtifactError::Temporary)?;
        let temporary_path = temporary.path().join("server.jar");
        let mut response = self.get(&artifact.download_url)?;
        let mut file = File::options()
            .create_new(true)
            .write(true)
            .open(&temporary_path)
            .map_err(ArtifactError::Temporary)?;
        io::copy(&mut response, &mut file).map_err(|source| ArtifactError::Download {
            url: artifact.download_url.clone(),
            source,
        })?;
        file.flush().map_err(ArtifactError::Temporary)?;
        file.sync_all().map_err(ArtifactError::Temporary)?;
        drop(file);
        let verified = fs::read(&temporary_path).map_err(ArtifactError::Temporary)?;
        if let Some(expected) = artifact.checksum.as_deref() {
            verify_checksum(expected, &verified)?;
        }
        let filesystem =
            statvfs(server_root).map_err(|error| ArtifactError::DiskInspect(error.into()))?;
        let available = filesystem.f_bavail.saturating_mul(filesystem.f_frsize);
        let required = u64::try_from(verified.len()).unwrap_or(u64::MAX);
        if required > available {
            return Err(ArtifactError::DiskSpace {
                available,
                required,
            });
        }
        materialize_managed_bytes(server_root, "server.jar", &verified)?;
        Ok(())
    }

    /// Resolves the Mojang server artifact from already fetched version metadata.
    fn resolve_vanilla(
        &self,
        version: &MinecraftVersion,
        details: MojangVersion,
    ) -> Result<ResolvedArtifact, ArtifactError> {
        let server = details
            .downloads
            .server
            .ok_or_else(|| ArtifactError::NoEligibleArtifact {
                platform: "vanilla",
                version: version.id.clone(),
            })?;
        Ok(ResolvedArtifact {
            platform: Platform::Vanilla,
            version: version.id.clone(),
            build: None,
            loader: None,
            installer: None,
            checksum: Some(format!("sha1:{}", server.sha1)),
            download_url: server.url,
            required_java_major: details.java_version.major_version,
        })
    }

    /// Resolves the newest stable Paper build returned for the exact version.
    fn resolve_paper(
        &self,
        version: &MinecraftVersion,
        required_java_major: u32,
    ) -> Result<ResolvedArtifact, ArtifactError> {
        let url = replace(&self.endpoints.paper_builds, "version", &version.id);
        let builds: Vec<PaperBuild> = self.get_json(&url)?;
        let build = builds
            .into_iter()
            .find(|build| build.channel == "STABLE")
            .ok_or_else(|| ArtifactError::NoEligibleArtifact {
                platform: "paper",
                version: version.id.clone(),
            })?;
        let download = build.downloads.get("server:default").ok_or_else(|| {
            ArtifactError::NoEligibleArtifact {
                platform: "paper",
                version: version.id.clone(),
            }
        })?;
        let checksum = download
            .checksums
            .as_ref()
            .and_then(|checksums| checksums.sha256.as_ref())
            .map(|digest| format!("sha256:{digest}"));
        Ok(ResolvedArtifact {
            platform: Platform::Paper,
            version: version.id.clone(),
            build: Some(build.id),
            loader: None,
            installer: None,
            checksum,
            download_url: download.url.clone(),
            required_java_major,
        })
    }

    /// Resolves the newest stable Fabric Loader and Installer returned by their catalogs.
    fn resolve_fabric(
        &self,
        version: &MinecraftVersion,
        required_java_major: u32,
    ) -> Result<ResolvedArtifact, ArtifactError> {
        let loader_url = replace(&self.endpoints.fabric_loaders, "version", &version.id);
        let loaders: Vec<FabricLoaderEntry> = self.get_json(&loader_url)?;
        let loader = loaders
            .into_iter()
            .find(|entry| entry.loader.stable)
            .ok_or_else(|| ArtifactError::NoEligibleArtifact {
                platform: "fabric",
                version: version.id.clone(),
            })?
            .loader
            .version;
        let installers: Vec<FabricInstaller> = self.get_json(&self.endpoints.fabric_installers)?;
        let installer = installers
            .into_iter()
            .find(|entry| entry.stable)
            .ok_or_else(|| ArtifactError::NoEligibleArtifact {
                platform: "fabric",
                version: version.id.clone(),
            })?
            .version;
        let download_url = replace_many(
            &self.endpoints.fabric_server,
            &[
                ("version", version.id.as_str()),
                ("loader", loader.as_str()),
                ("installer", installer.as_str()),
            ],
        );
        Ok(ResolvedArtifact {
            platform: Platform::Fabric,
            version: version.id.clone(),
            build: None,
            loader: Some(loader),
            installer: Some(installer),
            checksum: None,
            download_url,
            required_java_major,
        })
    }

    /// Sends one GET request and requires a successful status.
    fn get(&self, url: &str) -> Result<Response, ArtifactError> {
        let response = self
            .client
            .get(url)
            .send()
            .map_err(|source| ArtifactError::Request {
                url: url.to_owned(),
                source,
            })?;
        if !response.status().is_success() {
            return Err(ArtifactError::Status {
                url: url.to_owned(),
                status: response.status(),
            });
        }
        Ok(response)
    }

    /// Fetches and decodes one required upstream JSON response.
    fn get_json<T: DeserializeOwned>(&self, url: &str) -> Result<T, ArtifactError> {
        self.get(url)?
            .json()
            .map_err(|source| ArtifactError::Metadata {
                url: url.to_owned(),
                source,
            })
    }
}

#[derive(Debug, Deserialize)]
struct MojangManifest {
    versions: Vec<MojangManifestVersion>,
}

#[derive(Debug, Deserialize)]
struct MojangManifestVersion {
    id: String,
    #[serde(rename = "type")]
    kind: String,
    url: String,
}

#[derive(Debug, Deserialize)]
struct MojangVersion {
    downloads: MojangDownloads,
    #[serde(rename = "javaVersion")]
    java_version: MojangJavaVersion,
}

#[derive(Debug, Deserialize)]
struct MojangDownloads {
    server: Option<MojangServerDownload>,
}

#[derive(Debug, Deserialize)]
struct MojangServerDownload {
    url: String,
    sha1: String,
}

#[derive(Debug, Deserialize)]
struct MojangJavaVersion {
    #[serde(rename = "majorVersion")]
    major_version: u32,
}

#[derive(Debug, Deserialize)]
struct PaperProject {
    versions: BTreeMap<String, Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct PaperBuild {
    id: u64,
    channel: String,
    downloads: BTreeMap<String, PaperDownload>,
}

#[derive(Debug, Deserialize)]
struct PaperDownload {
    url: String,
    checksums: Option<PaperChecksums>,
}

#[derive(Debug, Deserialize)]
struct PaperChecksums {
    sha256: Option<String>,
}

#[derive(Debug, Deserialize)]
struct FabricGame {
    version: String,
}

#[derive(Debug, Deserialize)]
struct FabricLoaderEntry {
    loader: FabricLoader,
}

#[derive(Debug, Deserialize)]
struct FabricLoader {
    version: String,
    stable: bool,
}

#[derive(Debug, Deserialize)]
struct FabricInstaller {
    version: String,
    stable: bool,
}

/// Computes and compares one supported pinned checksum.
fn verify_checksum(expected: &str, bytes: &[u8]) -> Result<(), ArtifactError> {
    let (algorithm, digest) = expected
        .split_once(':')
        .ok_or_else(|| ArtifactError::UnsupportedChecksum(expected.to_owned()))?;
    let actual = match algorithm {
        "sha1" => hex_digest(&Sha1::digest(bytes)),
        "sha256" => hex_digest(&Sha256::digest(bytes)),
        _ => return Err(ArtifactError::UnsupportedChecksum(expected.to_owned())),
    };
    if actual.eq_ignore_ascii_case(digest) {
        Ok(())
    } else {
        Err(ArtifactError::ChecksumMismatch {
            expected: expected.to_owned(),
            actual: format!("{algorithm}:{actual}"),
        })
    }
}

/// Encodes digest bytes as lowercase hexadecimal without another dependency.
fn hex_digest(bytes: &[u8]) -> String {
    use std::fmt::Write as _;

    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(output, "{byte:02x}").expect("writing to a String cannot fail");
    }
    output
}

/// Replaces one endpoint placeholder with a percent-encoded path segment.
fn replace(template: &str, name: &str, value: &str) -> String {
    template.replace(&format!("{{{name}}}"), &encode_path_segment(value))
}

/// Replaces several endpoint placeholders with percent-encoded path segments.
fn replace_many(template: &str, replacements: &[(&str, &str)]) -> String {
    replacements
        .iter()
        .fold(template.to_owned(), |url, (name, value)| {
            replace(&url, name, value)
        })
}

/// Percent-encodes one URL path segment using UTF-8 bytes.
fn encode_path_segment(value: &str) -> String {
    use std::fmt::Write as _;

    let mut encoded = String::new();
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            encoded.push(char::from(byte));
        } else {
            write!(encoded, "%{byte:02X}").expect("writing to a String cannot fail");
        }
    }
    encoded
}
