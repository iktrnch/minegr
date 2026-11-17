use std::path::Path;

use anyhow::Context;
use serde::{Deserialize, Serialize};

pub use crate::platform::Platform;
use crate::platform::properties::Properties;

#[derive(Debug, strum::Display, strum::EnumString, strum::VariantNames, Deserialize, Serialize)]
#[strum(serialize_all = "lowercase")]
pub enum ApplicationType {
    Server,
    Instance,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Moxum {
    pub application_type: ApplicationType,
    pub version: u8,
    pub uuid: String,
    pub name: String,
}

impl Moxum {
    pub fn new(name: String) -> Self {
        Moxum {
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
    pub moxum: Moxum,
    pub minecraft: Minecraft,
    pub properties: Properties,
    pub java: Java,
}

impl Config {
    pub fn try_load(path: &Path) -> anyhow::Result<Self> {
        let config_str = std::fs::read_to_string(path).context("Failed to read config file")?;
        toml::from_str(&config_str).context("Failed to parse config file")
    }

    pub fn try_save(&self, path: &Path) -> anyhow::Result<()> {
        let text = toml::to_string_pretty(&self).context("Failed to parse config into a file")?;
        std::fs::write(path.join("moxum.toml"), text).context("Failed to write config file")?;
        Ok(())
    }
}
