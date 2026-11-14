use anyhow::Context;
use dialoguer::{Confirm, Input, Select, theme::ColorfulTheme};
use serde::{Deserialize, Serialize};
use strum::VariantNames;

pub use crate::platform::Platform;
use crate::platform::properties::Properties;

#[derive(Debug, strum::Display, strum::EnumString, strum::VariantNames, Deserialize, Serialize)]
#[strum(serialize_all = "lowercase")]
pub enum ApplicationType {
    Server,
    Instance,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Minegr {
    pub application_type: ApplicationType,
    pub version: u8,
    pub uuid: String,
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Minecraft {
    pub platform: Platform,
    pub version: String,
    pub checksum: String,
    pub eula: bool,
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
}

impl Config {
    pub fn init() -> Self {
        let minegr = minegr();
        let minecraft = minecraft();

        Config {
            minegr,
            minecraft,
            properties: Properties::default(),
            java: Java::default(),
        }
    }

    pub fn try_load() -> anyhow::Result<Self> {
        let config_str =
            std::fs::read_to_string("config.toml").context("Failed to read config file")?;
        toml::from_str(&config_str).context("Failed to parse config file")
    }

    pub fn try_save(&self) -> anyhow::Result<()> {
        let text = toml::to_string_pretty(&self).context("Failed to parse config into a file")?;
        std::fs::write("config.toml", text).context("Failed to write config file")?;
        Ok(())
    }
}

fn minegr() -> Minegr {
    let name: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Name your instance: ")
        .interact_text()
        .unwrap();

    Minegr {
        application_type: ApplicationType::Server,
        version: 1,
        uuid: uuid::Uuid::new_v4().to_string(),
        name,
    }
}

/// Collects the Minecraft platform and EULA acceptance.
fn minecraft() -> Minecraft {
    let platform_index = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select platform")
        .default(0)
        .items(Platform::VARIANTS)
        .interact()
        .unwrap();
    let platform: Platform = Platform::VARIANTS[platform_index]
        .parse()
        .expect("platform variant names must parse as Platform");

    let version = platform
        .select_version()
        .expect("Failed to select a Minecraft version");
    let checksum = String::new(); // TODO: Implement checksum calculation logic
    let eula = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Do you accept the EULA?")
        .interact()
        .unwrap();

    Minecraft {
        platform,
        version,
        checksum,
        eula,
    }
}
