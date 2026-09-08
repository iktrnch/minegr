//! One-way capture of stopped-server properties into `minegr.toml`.

use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::io;
use std::os::unix::fs::{FileTypeExt, MetadataExt};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

use rustix::net::sockopt::socket_peercred;
use rustix::process::geteuid;
use thiserror::Error;
use toml_edit::{DocumentMut, InlineTable, Item, Table, Value};

use crate::config::{Config, LoadedConfig, PropertyValue};
use crate::config_path::ConfigPath;
use crate::config_write::{WriteConfigError, write_bytes_if_unchanged};
use crate::init::{active_session_lock, configured_port_occupied};
use crate::managed_files::{
    ManagedFileError, PropertiesError, parse_server_properties, read_managed_bytes,
};

/// Host observations required to prove that synchronization is safe.
pub trait StoppedServerProbe {
    /// Reports whether a matching daemon or possibly unmanaged server is running.
    fn server_running(
        &self,
        path: &ConfigPath,
        config: &Config,
        properties: &BTreeMap<String, PropertyValue>,
    ) -> Result<bool, String>;
}

/// Linux-backed stopped-server probe for daemon sockets, world locks, and the configured port.
#[derive(Clone, Copy, Debug, Default)]
pub struct SystemStoppedServerProbe;

impl StoppedServerProbe for SystemStoppedServerProbe {
    fn server_running(
        &self,
        path: &ConfigPath,
        config: &Config,
        properties: &BTreeMap<String, PropertyValue>,
    ) -> Result<bool, String> {
        if matching_daemon_running(config)? {
            return Ok(true);
        }
        let server_root = path
            .as_path()
            .parent()
            .ok_or_else(|| "configuration path has no server root".to_owned())?;
        if active_session_lock(server_root, properties)? {
            return Ok(true);
        }
        configured_port_occupied(properties)
    }
}

/// Successful output from one synchronization.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SyncSuccess {
    config_path: String,
}

impl SyncSuccess {
    /// Returns the exact documented stdout payload.
    pub fn stdout(&self) -> String {
        format!("Configuration synchronized: {}\n", self.config_path)
    }
}

/// A failure while checking, parsing, or publishing synchronization state.
#[derive(Debug, Error)]
pub enum SyncError {
    /// The selected server cannot be synchronized while it may be running.
    #[error(
        "The server appears to be running; stop it and inspect its daemon, session.lock, and configured port before synchronizing"
    )]
    Running,
    /// A host probe could not determine a safe stopped state.
    #[error("Failed to verify that the server is stopped: {0}")]
    StoppedCheck(String),
    /// The managed properties source could not be read safely.
    #[error("Failed to read server.properties: {0}")]
    ManagedFile(#[from] ManagedFileError),
    /// The Java-properties source was incomplete, duplicate, or malformed.
    #[error("Invalid server.properties: {0}")]
    Properties(#[from] PropertiesError),
    /// The properties source is not valid UTF-8.
    #[error("Invalid server.properties: file is not UTF-8: {0}")]
    PropertiesEncoding(#[from] std::string::FromUtf8Error),
    /// A recognized property could not be converted to its documented type.
    #[error("Invalid server.properties value for `{name}`: {reason}")]
    RecognizedValue {
        /// Recognized property name.
        name: &'static str,
        /// Expected range, type, or content.
        reason: &'static str,
    },
    /// The existing TOML document could not be preserved and edited.
    #[error("Failed to edit minegr.toml: {0}")]
    Document(String),
    /// The synchronized configuration could not be atomically published.
    #[error("Failed to write synchronized configuration: {0}")]
    Write(#[from] WriteConfigError),
}

impl SyncError {
    /// Reports whether this failure uses the unavailable-server exit category.
    pub fn is_unavailable(&self) -> bool {
        matches!(self, Self::Running)
    }

    /// Reports whether this failure is invalid input or configuration.
    pub fn is_usage(&self) -> bool {
        matches!(
            self,
            Self::StoppedCheck(_)
                | Self::ManagedFile(_)
                | Self::Properties(_)
                | Self::PropertiesEncoding(_)
                | Self::RecognizedValue { .. }
                | Self::Document(_)
        )
    }
}

/// Captures all persisted properties into the selected configuration atomically.
pub fn run_sync(
    path: &ConfigPath,
    loaded: &LoadedConfig,
    stopped: &dyn StoppedServerProbe,
) -> Result<SyncSuccess, SyncError> {
    let config = &loaded.config;
    if stopped
        .server_running(path, config, &config.minecraft.properties)
        .map_err(SyncError::StoppedCheck)?
    {
        return Err(SyncError::Running);
    }
    let server_root = path
        .as_path()
        .parent()
        .expect("a resolved configuration path always has a parent");
    let bytes = read_managed_bytes(server_root, "server.properties")?;
    let source = String::from_utf8(bytes)?;
    let raw = parse_server_properties(&source)?;
    let properties = typed_properties(raw)?;
    if stopped
        .server_running(path, config, &properties)
        .map_err(SyncError::StoppedCheck)?
    {
        return Err(SyncError::Running);
    }

    let mut document = loaded
        .source
        .parse::<DocumentMut>()
        .map_err(|error| SyncError::Document(error.to_string()))?;
    replace_properties_table(&mut document, properties)?;
    write_bytes_if_unchanged(path, document.to_string().as_bytes(), loaded.identity)?;

    Ok(SyncSuccess {
        config_path: path.as_str().to_owned(),
    })
}

/// Converts the three recognized properties and keeps all unknown values as strings.
fn typed_properties(
    raw: BTreeMap<String, String>,
) -> Result<BTreeMap<String, PropertyValue>, SyncError> {
    raw.into_iter()
        .map(|(name, value)| {
            let typed = match name.as_str() {
                "server-port" => {
                    let port = value.parse::<u16>().ok().filter(|port| *port > 0).ok_or(
                        SyncError::RecognizedValue {
                            name: "server-port",
                            reason: "expected an integer in 1..=65535",
                        },
                    )?;
                    PropertyValue::Integer(i64::from(port))
                }
                "white-list" => match value.as_str() {
                    "true" => PropertyValue::Boolean(true),
                    "false" => PropertyValue::Boolean(false),
                    _ => {
                        return Err(SyncError::RecognizedValue {
                            name: "white-list",
                            reason: "expected `true` or `false`",
                        });
                    }
                },
                "level-name" if !value.trim().is_empty() => PropertyValue::String(value),
                "level-name" => {
                    return Err(SyncError::RecognizedValue {
                        name: "level-name",
                        reason: "expected a non-empty string",
                    });
                }
                _ => PropertyValue::String(value),
            };
            Ok((name, typed))
        })
        .collect()
}

/// Replaces the complete document table while preserving its surrounding decoration.
fn replace_properties_table(
    document: &mut DocumentMut,
    properties: BTreeMap<String, PropertyValue>,
) -> Result<(), SyncError> {
    let minecraft = document
        .get_mut("minecraft")
        .ok_or_else(|| SyncError::Document("minecraft table is missing".to_owned()))?;
    if let Some(table) = minecraft.as_table_mut() {
        let decor = match table.get("properties") {
            Some(Item::Table(existing)) => Some(existing.decor().clone()),
            Some(_) => {
                return Err(SyncError::Document(
                    "minecraft.properties is not a table".to_owned(),
                ));
            }
            None => None,
        };
        let mut replacement = Table::new();
        if let Some(decor) = decor {
            *replacement.decor_mut() = decor;
        }
        for (name, value) in properties {
            replacement.insert(&name, Item::Value(toml_value(value)));
        }
        table.insert("properties", Item::Table(replacement));
        return Ok(());
    }
    if let Some(table) = minecraft.as_inline_table_mut() {
        let mut replacement = InlineTable::new();
        for (name, value) in properties {
            replacement.insert(&name, toml_value(value));
        }
        table.insert("properties", Value::InlineTable(replacement));
        return Ok(());
    }
    Err(SyncError::Document("minecraft is not a table".to_owned()))
}

/// Converts one typed configuration scalar into its editable TOML representation.
fn toml_value(value: PropertyValue) -> Value {
    match value {
        PropertyValue::Boolean(value) => Value::from(value),
        PropertyValue::Integer(value) => Value::from(value),
        PropertyValue::Float(value) => Value::from(value),
        PropertyValue::String(value) => Value::from(value),
    }
}

/// Reports an owner-matched responsive daemon socket for this UUID.
fn matching_daemon_running(config: &Config) -> Result<bool, String> {
    let Some(uuid) = config.minegr.uuid.as_deref() else {
        return Ok(false);
    };
    let runtime_root = env::var_os("XDG_RUNTIME_DIR")
        .ok_or_else(|| "XDG_RUNTIME_DIR is unavailable".to_owned())?;
    let socket_path = PathBuf::from(runtime_root)
        .join("minegr")
        .join(format!("{uuid}.sock"));
    let metadata = match fs::symlink_metadata(&socket_path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(error) => {
            return Err(format!(
                "could not inspect daemon socket {}: {error}",
                socket_path.display()
            ));
        }
    };
    if !metadata.file_type().is_socket() || metadata.uid() != geteuid().as_raw() {
        return Err(format!(
            "daemon socket {} is not a safe owner-matched Unix socket",
            socket_path.display()
        ));
    }
    let stream = match UnixStream::connect(&socket_path) {
        Ok(stream) => stream,
        Err(error) if error.kind() == io::ErrorKind::ConnectionRefused => return Ok(false),
        Err(error) => {
            return Err(format!(
                "could not connect to daemon socket {}: {error}",
                socket_path.display()
            ));
        }
    };
    let credentials = socket_peercred(&stream)
        .map_err(|error| format!("could not identify daemon peer: {error}"))?;
    if credentials.uid.as_raw() != geteuid().as_raw() {
        return Err("daemon peer belongs to another operating-system user".to_owned());
    }
    Ok(true)
}
