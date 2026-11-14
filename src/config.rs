use std::{io::Write, path::Path};

use anyhow::Context;
use serde::{Deserialize, Serialize};

use crate::modrinth::repository::ReleaseChannel;
pub use crate::platform::Platform;
use crate::platform::properties::Properties;

#[derive(Debug, strum::Display, strum::EnumString, strum::VariantNames, Deserialize, Serialize)]
#[strum(serialize_all = "lowercase")]
pub enum ApplicationType {
    Server,
    Instance,
}

/// A pinned addon file and the channel policy used for future updates.
#[derive(Debug, Deserialize, Serialize)]
pub struct Addon {
    /// Installed filename supplied by Modrinth.
    pub name: String,
    pub version_id: String,
    /// Algorithm-qualified checksum, such as sha512:<hex>.
    pub hash: String,
    /// Most permissive allowed channel, which may differ from the installed release's channel.
    pub version_type: ReleaseChannel,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Minegr {
    pub application_type: ApplicationType,
    pub version: u8,
    pub uuid: String,
    pub name: String,
}

impl Minegr {
    pub fn new(name: String) -> Self {
        Minegr {
            application_type: ApplicationType::Server,
            version: 1,
            uuid: uuid::Uuid::new_v4().to_string(),
            name,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Minecraft {
    pub platform: Platform,
    pub version: String,
    pub checksum: String,
    pub eula: bool,
}

impl Minecraft {
    pub fn new(platform: Platform, version: String, checksum: String, eula: bool) -> Self {
        Minecraft {
            platform,
            version,
            checksum,
            eula,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Java {
    pub executable: String,
    pub jvm_args: String,
    pub server_args: String,
}

impl Default for Java {
    fn default() -> Self {
        Java {
            executable: "java".to_string(),
            jvm_args: "-Xms2G -Xmx2G".to_string(),
            server_args: "nogui".to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub minegr: Minegr,
    pub minecraft: Minecraft,
    pub properties: Properties,
    pub java: Java,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub addons: Vec<Addon>,
}

impl Config {
    pub fn try_load(config_dir: &Path) -> anyhow::Result<Self> {
        let config_str = std::fs::read_to_string(config_dir.join("minegr.toml"))
            .context("Failed to read config file")?;
        toml::from_str(&config_str).context("Failed to parse config file")
    }

    /// Atomically saves minegr.toml inside the selected directory with owner-only permissions.
    pub fn try_save(&self, path: &Path) -> anyhow::Result<()> {
        let text = toml::to_string_pretty(self).context("Failed to serialize the configuration")?;
        let mut staged = tempfile::NamedTempFile::new_in(path)
            .context("Failed to create a temporary configuration file")?;
        staged
            .write_all(text.as_bytes())
            .context("Failed to write the temporary configuration")?;
        staged
            .as_file()
            .sync_all()
            .context("Failed to synchronize the temporary configuration")?;
        staged
            .persist(path.join("minegr.toml"))
            .map_err(|error| error.error)
            .context("Failed to publish minegr.toml")?;
        std::fs::File::open(path)
            .context("Failed to open the configuration directory for synchronization")?
            .sync_all()
            .context("minegr.toml was saved, but synchronizing its directory failed")?;
        Ok(())
    }
}
