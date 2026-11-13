use dialoguer::{Confirm, Input, Select, theme::ColorfulTheme};
use serde::{Deserialize, Serialize};
use strum::VariantNames;

use crate::platform::{
    fabric::select_fabric_version, paper::select_paper_version, vanilla::select_vanilla_version,
};

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

#[derive(Debug, strum::Display, strum::EnumString, strum::VariantNames, Deserialize, Serialize)]
#[strum(serialize_all = "lowercase")]
pub enum Platform {
    Paper,
    Fabric,
    Vanilla,
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
    pub jvm_args: Vec<String>,
    pub server_args: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub minegr: Minegr,
    pub minecraft: Minecraft,
    pub java: Java,
}

impl Config {
    pub fn init() -> Self {
        let minegr = minegr();
        let minecraft = minecraft();
        let java = java();

        Config {
            minegr,
            minecraft,
            java,
        }
    }
}

fn minegr() -> Minegr {
    let application_type_index = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select application type")
        .default(0)
        .items(ApplicationType::VARIANTS)
        .interact()
        .unwrap();
    let name: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Name your instance: ")
        .interact_text()
        .unwrap();

    Minegr {
        application_type: ApplicationType::VARIANTS[application_type_index]
            .parse()
            .unwrap(),
        version: 1,
        uuid: uuid::Uuid::new_v4().to_string(),
        name,
    }
}

/// Selects a Minecraft version for the requested platform.
fn get_version_from_platform(platform: &Platform) -> String {
    match platform {
        Platform::Paper => {
            select_paper_version().expect("Failed to select a Paper Minecraft release")
        }
        Platform::Fabric => {
            select_fabric_version().expect("Failed to select a stable Fabric Minecraft version")
        }
        Platform::Vanilla => {
            select_vanilla_version().expect("Failed to select a Vanilla Minecraft release")
        }
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
    let platform = Platform::VARIANTS[platform_index]
        .parse()
        .expect("platform variant names must parse as Platform");

    let version = get_version_from_platform(&platform);
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

/// Collects the Java executable and space-separated JVM and server arguments.
fn java() -> Java {
    let executable = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter the path to the Java executable")
        .default("java".to_string())
        .interact_text()
        .unwrap();
    let jvm_args = Input::<String>::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter JVM arguments (space-separated)")
        .default("-Xms2G -Xmx2G".to_string())
        .interact_text()
        .unwrap()
        .split_whitespace()
        .map(str::to_owned)
        .collect();
    let server_args = Input::<String>::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter server arguments (space-separated)")
        .default("nogui".to_string())
        .interact_text()
        .unwrap()
        .split_whitespace()
        .map(str::to_owned)
        .collect();

    Java {
        executable,
        jvm_args,
        server_args,
    }
}
