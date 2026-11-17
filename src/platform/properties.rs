use std::path::Path;

use anyhow::Context;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

/// Default game mode, serialized using its canonical name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum::Display)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum GameMode {
    #[serde(alias = "0")]
    Survival,
    #[serde(alias = "1")]
    Creative,
    #[serde(alias = "2")]
    Adventure,
    #[serde(alias = "3")]
    Spectator,
}

/// Server difficulty, accepting legacy numeric strings and emitting canonical names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum::Display)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum Difficulty {
    #[serde(alias = "0")]
    Peaceful,
    #[serde(alias = "1")]
    Easy,
    #[serde(alias = "2")]
    Normal,
    #[serde(alias = "3")]
    Hard,
}

/// Compression used for chunks stored in region files.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum::Display)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum RegionFileCompression {
    Deflate,
    Lz4,
    None,
}

/// A standard world preset or an extensible datapack/legacy preset name.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, strum::Display)]
#[serde(from = "String", into = "String")]
pub enum LevelType {
    #[strum(serialize = "minecraft:normal")]
    Normal,
    #[strum(serialize = "minecraft:flat")]
    Flat,
    #[strum(serialize = "minecraft:large_biomes")]
    LargeBiomes,
    #[strum(serialize = "minecraft:amplified")]
    Amplified,
    #[strum(serialize = "minecraft:single_biome_surface")]
    SingleBiomeSurface,
    #[strum(transparent)]
    Custom(String),
}

impl From<String> for LevelType {
    /// Recognizes standard presets with or without the vanilla namespace.
    fn from(value: String) -> Self {
        match value.as_str() {
            "minecraft:normal" | "normal" => Self::Normal,
            "minecraft:flat" | "flat" => Self::Flat,
            "minecraft:large_biomes" | "large_biomes" => Self::LargeBiomes,
            "minecraft:amplified" | "amplified" => Self::Amplified,
            "minecraft:single_biome_surface" | "single_biome_surface" => Self::SingleBiomeSurface,
            _ => Self::Custom(value),
        }
    }
}

impl From<LevelType> for String {
    /// Emits namespaced standard presets and preserves custom or legacy values.
    fn from(value: LevelType) -> Self {
        match value {
            LevelType::Normal => "minecraft:normal".into(),
            LevelType::Flat => "minecraft:flat".into(),
            LevelType::LargeBiomes => "minecraft:large_biomes".into(),
            LevelType::Amplified => "minecraft:amplified".into(),
            LevelType::SingleBiomeSurface => "minecraft:single_biome_surface".into(),
            LevelType::Custom(value) => value,
        }
    }
}

/// Operator permission level, serialized as an integer from zero through four.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr, strum::Display)]
#[repr(u8)]
pub enum OperatorPermissionLevel {
    #[strum(serialize = "0")]
    Level0 = 0,
    #[strum(serialize = "1")]
    Level1 = 1,
    #[strum(serialize = "2")]
    Level2 = 2,
    #[strum(serialize = "3")]
    Level3 = 3,
    #[strum(serialize = "4")]
    Level4 = 4,
}

/// Function permission level, serialized as an integer from one through four.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr, strum::Display)]
#[repr(u8)]
pub enum FunctionPermissionLevel {
    #[strum(serialize = "1")]
    Level1 = 1,
    #[strum(serialize = "2")]
    Level2 = 2,
    #[strum(serialize = "3")]
    Level3 = 3,
    #[strum(serialize = "4")]
    Level4 = 4,
}

/// Text-filtering configuration version, serialized as zero or one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr, strum::Display)]
#[repr(u8)]
pub enum TextFilteringVersion {
    #[strum(serialize = "0")]
    V0 = 0,
    #[strum(serialize = "1")]
    V1 = 1,
}

/// Optional Java Edition `server.properties` settings.
///
/// Missing fields become `None` and are omitted from serialized output.
/// `Default` sets only the Moxum MOTD and zero spawn protection.
/// Strings preserve property values such as JSON text and comma-separated lists.
/// Fixed choices use enums with canonical strings or numeric property values.
/// Version-specific and legacy keys are included; the target server determines support.
/// Dotted keys such as `rcon.port` remain literal keys in both output formats.
/// `serde-java-properties` reads empty optional values as `None`; TOML preserves them.
/// See <https://minecraft.wiki/w/Server.properties#Java_Edition> for version details.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Properties {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accepts_transfers: Option<bool>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow_flight: Option<bool>,

    /// Introduced in Java Edition 26.4 snapshots.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allowed_connection_ids: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub broadcast_console_to_ops: Option<bool>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub broadcast_rcon_to_ops: Option<bool>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bug_report_link: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chat_spam_threshold_seconds: Option<u32>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command_spam_threshold_seconds: Option<u32>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub difficulty: Option<Difficulty>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enable_code_of_conduct: Option<bool>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enable_jmx_monitoring: Option<bool>,

    /// Introduced in Java Edition 26.4 snapshots.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enable_legacy_status: Option<bool>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enable_query: Option<bool>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enable_rcon: Option<bool>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enable_status: Option<bool>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enforce_secure_profile: Option<bool>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enforce_whitelist: Option<bool>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entity_broadcast_range_percentage: Option<u32>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub force_gamemode: Option<bool>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub function_permission_level: Option<FunctionPermissionLevel>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gamemode: Option<GameMode>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generate_structures: Option<bool>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generator_settings: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hardcore: Option<bool>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hide_online_players: Option<bool>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initial_disabled_packs: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initial_enabled_packs: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub level_name: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub level_seed: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub level_type: Option<LevelType>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub log_ips: Option<bool>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub management_server_allowed_origins: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub management_server_enabled: Option<bool>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub management_server_host: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub management_server_port: Option<u16>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub management_server_secret: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub management_server_tls_enabled: Option<bool>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub management_server_tls_keystore: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub management_server_tls_keystore_password: Option<String>,

    /// Negative values disable the chained neighbor-update limit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_chained_neighbor_updates: Option<i32>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_players: Option<u32>,

    /// Watchdog timeout in milliseconds; -1 disables the watchdog.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_tick_time: Option<i64>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_world_size: Option<u32>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub motd: Option<String>,

    /// Packet compression threshold in bytes; -1 disables compression.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub network_compression_threshold: Option<i32>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub online_mode: Option<bool>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub op_permission_level: Option<OperatorPermissionLevel>,

    /// Nonpositive values disable pausing an empty server.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pause_when_empty_seconds: Option<i32>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player_idle_timeout: Option<u32>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prevent_proxy_connections: Option<bool>,

    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "query.port"
    )]
    pub query_port: Option<u16>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rate_limit: Option<u32>,

    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "rcon.password"
    )]
    pub rcon_password: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none", rename = "rcon.port")]
    pub rcon_port: Option<u16>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub region_file_compression: Option<RegionFileCompression>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub require_resource_pack: Option<bool>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource_pack: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource_pack_id: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource_pack_prompt: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource_pack_sha1: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub server_ip: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub server_port: Option<u16>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub simulation_distance: Option<u8>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spawn_protection: Option<u32>,

    /// Introduced in Java Edition 26.4 snapshots.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status_contact_details: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status_heartbeat_interval: Option<u32>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sync_chunk_writes: Option<bool>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_filtering_config: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_filtering_version: Option<TextFilteringVersion>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub use_native_transport: Option<bool>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub view_distance: Option<u8>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub white_list: Option<bool>,

    // Legacy Java Edition properties retained for older server releases.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow_nether: Option<bool>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub announce_player_achievements: Option<bool>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enable_command_block: Option<bool>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_build_height: Option<u32>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previews_chat: Option<bool>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pvp: Option<bool>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub snooper_enabled: Option<bool>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spawn_animals: Option<bool>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spawn_monsters: Option<bool>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spawn_npcs: Option<bool>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub texture_pack: Option<String>,
}

impl Default for Properties {
    /// Sets the Moxum MOTD and zero spawn protection, leaving all other settings unspecified.
    fn default() -> Self {
        Self {
            accepts_transfers: None,
            allow_flight: None,
            allowed_connection_ids: None,
            broadcast_console_to_ops: None,
            broadcast_rcon_to_ops: None,
            bug_report_link: None,
            chat_spam_threshold_seconds: None,
            command_spam_threshold_seconds: None,
            difficulty: None,
            enable_code_of_conduct: None,
            enable_jmx_monitoring: None,
            enable_legacy_status: None,
            enable_query: None,
            enable_rcon: None,
            enable_status: None,
            enforce_secure_profile: None,
            enforce_whitelist: None,
            entity_broadcast_range_percentage: None,
            force_gamemode: None,
            function_permission_level: None,
            gamemode: None,
            generate_structures: None,
            generator_settings: None,
            hardcore: None,
            hide_online_players: None,
            initial_disabled_packs: None,
            initial_enabled_packs: None,
            level_name: None,
            level_seed: None,
            level_type: None,
            log_ips: None,
            management_server_allowed_origins: None,
            management_server_enabled: None,
            management_server_host: None,
            management_server_port: None,
            management_server_secret: None,
            management_server_tls_enabled: None,
            management_server_tls_keystore: None,
            management_server_tls_keystore_password: None,
            max_chained_neighbor_updates: None,
            max_players: None,
            max_tick_time: None,
            max_world_size: None,
            motd: Some(
                "              \u{00a7}kg\u{00a7}7 Powered by \u{00a7}2moxum \u{00a7}kg".to_owned(),
            ),
            network_compression_threshold: None,
            online_mode: None,
            op_permission_level: None,
            pause_when_empty_seconds: None,
            player_idle_timeout: None,
            prevent_proxy_connections: None,
            query_port: None,
            rate_limit: None,
            rcon_password: None,
            rcon_port: None,
            region_file_compression: None,
            require_resource_pack: None,
            resource_pack: None,
            resource_pack_id: None,
            resource_pack_prompt: None,
            resource_pack_sha1: None,
            server_ip: None,
            server_port: None,
            simulation_distance: None,
            spawn_protection: Some(0),
            status_contact_details: None,
            status_heartbeat_interval: None,
            sync_chunk_writes: None,
            text_filtering_config: None,
            text_filtering_version: None,
            use_native_transport: None,
            view_distance: None,
            white_list: None,
            allow_nether: None,
            announce_player_achievements: None,
            enable_command_block: None,
            max_build_height: None,
            previews_chat: None,
            pvp: None,
            snooper_enabled: None,
            spawn_animals: None,
            spawn_monsters: None,
            spawn_npcs: None,
            texture_pack: None,
        }
    }
}

impl Properties {
    /// Writes `server.properties` under `config_dir` with a warning that moxum manages it.
    pub fn try_save(&self, config_dir: &Path) -> anyhow::Result<()> {
        let text = serde_java_properties::to_string(self)
            .context("Failed to serialize server.properties")?;
        let text = format!(
            "# IMPORTANT: Managed by moxum; changes are overwritten on the next server start.\n\
             # Use moxum or edit moxum.toml to change server properties.\n\
             {text}"
        );
        std::fs::write(config_dir.join("server.properties"), text)
            .context("Failed to write server.properties")?;
        Ok(())
    }
}
