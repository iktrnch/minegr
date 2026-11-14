use serde::{Deserialize, Serialize};

/// A Modrinth release channel, encoded using its lowercase API name.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum::Display, clap::ValueEnum,
)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum ReleaseChannel {
    Release,
    Beta,
    Alpha,
}

impl ReleaseChannel {
    /// Checks whether this channel policy permits a version's release channel.
    pub fn allows(self, version_type: Self) -> bool {
        match self {
            Self::Release => version_type == Self::Release,
            Self::Beta => version_type != Self::Alpha,
            Self::Alpha => true,
        }
    }
}

/// Metadata used to select one compatible Modrinth version.
#[derive(Debug, Deserialize)]
pub(super) struct Version {
    pub game_versions: Vec<String>,
    pub id: String,
    pub loaders: Vec<String>,
    pub files: Vec<VersionFile>,
    pub version_type: ReleaseChannel,
    pub date_published: chrono::DateTime<chrono::Utc>,
}

/// One downloadable file belonging to a Modrinth version.
#[derive(Debug, Deserialize)]
pub(super) struct VersionFile {
    pub hashes: Hashes,
    pub url: String,
    pub filename: String,
    pub primary: bool,
    pub size: u64,
}

/// Upstream content hash used to verify an addon.
#[derive(Debug, Deserialize)]
pub(super) struct Hashes {
    pub sha512: String,
}
