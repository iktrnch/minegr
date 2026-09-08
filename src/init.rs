//! Creation of new declarative Minegr instances.

use std::collections::BTreeMap;
use std::env;
use std::fs::{self, File};
use std::io;
use std::net::TcpListener;
use std::os::unix::fs::MetadataExt;
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::Command;

use rustix::fs::{FlockOperation, fcntl_lock, statvfs};
use rustix::net::sockopt::socket_peercred;
use rustix::process::geteuid;
use thiserror::Error;
use toml_edit::{DocumentMut, Item, Value};
use uuid::Uuid;

use crate::artifact::{
    ArtifactService, MinecraftVersion, MinecraftVersionKind, PlatformAvailability, ResolvedArtifact,
};
use crate::cli::{InitArgs, PlatformArg};
use crate::config::{
    CONFIG_VERSION, Config, JavaConfig, LoadedConfig, MinecraftConfig, MinegrConfig, Platform,
    PropertyValue,
};
use crate::config_path::ConfigPath;
use crate::config_write::{WriteConfigError, WriteMode, write_bytes_if_unchanged, write_config};
use crate::managed_files::{ManagedFileError, materialize_text_files, preflight_text_files};
use crate::ui::Ui;
use crate::validation::{
    ConfigFileObservation, FileKind, HostCheck, HostCheckKind, HostCheckStatus,
    HostValidationInput, Observation, validate_config, validate_host,
};

/// Artifact operations needed by the initialization coordinator.
pub trait ArtifactProvider {
    /// Returns selectable Minecraft versions in newest-first upstream order.
    fn versions(&self) -> Result<Vec<MinecraftVersion>, String>;
    /// Returns eligible platforms for one selected version.
    fn available_platforms(
        &self,
        version: &MinecraftVersion,
    ) -> Result<PlatformAvailability, String>;
    /// Pins one exact platform artifact.
    fn resolve(
        &self,
        version: &MinecraftVersion,
        platform: Platform,
    ) -> Result<ResolvedArtifact, String>;
    /// Resolves exactly the artifact coordinates already stored in a configuration.
    fn resolve_pinned(&self, config: &MinecraftConfig) -> Result<ResolvedArtifact, String>;
    /// Downloads and atomically publishes the pinned artifact.
    fn download(&self, artifact: &ResolvedArtifact, server_root: &Path) -> Result<(), String>;
}

impl ArtifactProvider for ArtifactService {
    fn versions(&self) -> Result<Vec<MinecraftVersion>, String> {
        self.versions().map_err(|error| error.to_string())
    }

    fn available_platforms(
        &self,
        version: &MinecraftVersion,
    ) -> Result<PlatformAvailability, String> {
        self.available_platforms(version)
            .map_err(|error| error.to_string())
    }

    fn resolve(
        &self,
        version: &MinecraftVersion,
        platform: Platform,
    ) -> Result<ResolvedArtifact, String> {
        self.resolve(version, platform)
            .map_err(|error| error.to_string())
    }

    fn resolve_pinned(&self, config: &MinecraftConfig) -> Result<ResolvedArtifact, String> {
        self.resolve_pinned(config)
            .map_err(|error| error.to_string())
    }

    fn download(&self, artifact: &ResolvedArtifact, server_root: &Path) -> Result<(), String> {
        self.download(artifact, server_root)
            .map_err(|error| error.to_string())
    }
}

/// Java executable observation used before materialization.
pub trait JavaRuntime {
    /// Returns the detected Java major version.
    fn major_version(&self, executable: Option<&str>) -> Result<u32, String>;
}

/// Host-dependent checks required before an init operation mutates an instance.
pub trait InitHostValidator {
    /// Validates root ownership, memory, disk, port, and active-process state.
    fn validate(&self, server_root: &Path, config: &Config) -> Result<(), String>;
}

/// Host observations used to reject UUID changes for the selected running instance.
pub trait RunningInstanceProbe {
    /// Reports whether the selected canonical instance appears to be running.
    fn selected_instance_running(&self, path: &ConfigPath, config: &Config)
    -> Result<bool, String>;
}

/// Terminal interactions used by the initialization questionnaire.
pub trait InitPrompter {
    /// Reports whether this process may ask questions.
    fn is_interactive(&self) -> bool;
    /// Collects the instance display name.
    fn name(&self, default: &str) -> Result<String, String>;
    /// Selects one Minecraft version by index.
    fn version(&self, versions: &[MinecraftVersion]) -> Result<usize, String>;
    /// Selects one eligible platform by index.
    fn platform(&self, platforms: &[Platform]) -> Result<usize, String>;
    /// Collects explicit Minecraft EULA acceptance.
    fn accept_eula(&self) -> Result<bool, String>;
    /// Displays the complete proposed configuration summary.
    fn show_summary(&self, summary: &str);
    /// Collects final creation confirmation.
    fn confirm_creation(&self) -> Result<bool, String>;
}

impl InitPrompter for Ui {
    fn is_interactive(&self) -> bool {
        self.presentation().interactive
    }

    fn name(&self, default: &str) -> Result<String, String> {
        self.input("Server name", default)
            .map_err(|error| error.to_string())
    }

    fn version(&self, versions: &[MinecraftVersion]) -> Result<usize, String> {
        let items = versions
            .iter()
            .map(|version| version.id.clone())
            .collect::<Vec<_>>();
        self.fuzzy_select("Minecraft version", &items)
            .map_err(|error| error.to_string())
    }

    fn platform(&self, platforms: &[Platform]) -> Result<usize, String> {
        let items = platforms
            .iter()
            .copied()
            .map(platform_name)
            .map(str::to_owned)
            .collect::<Vec<_>>();
        self.select("Server platform", &items)
            .map_err(|error| error.to_string())
    }

    fn accept_eula(&self) -> Result<bool, String> {
        self.confirm("Do you accept the Minecraft EULA?", false)
            .map_err(|error| error.to_string())
    }

    fn show_summary(&self, summary: &str) {
        self.message(summary);
    }

    fn confirm_creation(&self) -> Result<bool, String> {
        self.confirm("Create this Minegr instance?", true)
            .map_err(|error| error.to_string())
    }
}

/// Host-backed Java version probe that never invokes a shell.
#[derive(Clone, Copy, Debug, Default)]
pub struct SystemJavaRuntime;

impl JavaRuntime for SystemJavaRuntime {
    fn major_version(&self, executable: Option<&str>) -> Result<u32, String> {
        let executable = executable.unwrap_or("java");
        let output = Command::new(executable)
            .arg("-version")
            .output()
            .map_err(|error| format!("Java `{executable}` is unavailable: {error}"))?;
        let text = format!(
            "{}\n{}",
            String::from_utf8_lossy(&output.stderr),
            String::from_utf8_lossy(&output.stdout)
        );
        if !output.status.success() {
            return Err(format!("`{executable} -version` failed: {}", text.trim()));
        }
        parse_java_major(&text)
            .ok_or_else(|| format!("could not determine Java version from `{executable} -version`"))
    }
}

/// Linux-backed init host validation.
#[derive(Clone, Copy, Debug, Default)]
pub struct SystemInitHostValidator;

impl InitHostValidator for SystemInitHostValidator {
    fn validate(&self, server_root: &Path, config: &Config) -> Result<(), String> {
        let report = validate_host(
            &HostValidationInput {
                config_file: ConfigFileObservation {
                    kind: Observation::Known(FileKind::Regular),
                    owner_uid: Observation::Known(geteuid().as_raw()),
                    expected_uid: geteuid().as_raw(),
                    mode: Observation::Known(0o600),
                },
                checks: self.host_checks(server_root, config),
            },
            &validate_config(config),
        );
        let errors = report
            .errors()
            .map(|finding| finding.message.as_str())
            .collect::<Vec<_>>();
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors.join("; "))
        }
    }
}

impl SystemInitHostValidator {
    /// Collects all independent init host observations for stable validation reporting.
    pub fn host_checks(&self, server_root: &Path, config: &Config) -> Vec<HostCheck> {
        vec![
            HostCheck::new(
                HostCheckKind::ServerRoot,
                None,
                server_root_status(server_root),
            ),
            HostCheck::new(HostCheckKind::Memory, None, memory_status(config)),
            HostCheck::new(HostCheckKind::DiskSpace, None, disk_status(server_root)),
            HostCheck::new(
                HostCheckKind::ActiveProcess,
                None,
                session_lock_status(server_root, &config.minecraft.properties),
            ),
            HostCheck::new(
                HostCheckKind::Port,
                None,
                port_status(&config.minecraft.properties),
            ),
        ]
    }
}

fn server_root_status(server_root: &Path) -> HostCheckStatus {
    let metadata = match fs::symlink_metadata(server_root) {
        Ok(metadata) => metadata,
        Err(error) => {
            return HostCheckStatus::Unavailable(format!("could not inspect server root: {error}"));
        }
    };
    if !metadata.file_type().is_dir() {
        return HostCheckStatus::Failed("server root is not a directory".to_owned());
    }
    let expected_uid = geteuid().as_raw();
    if metadata.uid() != expected_uid {
        return HostCheckStatus::Failed(format!(
            "server root is owned by user {}, expected user {expected_uid}",
            metadata.uid()
        ));
    }
    HostCheckStatus::Passed
}

fn memory_status(config: &Config) -> HostCheckStatus {
    match configured_heap_bytes(&config.java.jvm_args).and_then(|required_memory| {
        let available_memory = available_memory_bytes()?;
        if required_memory > available_memory {
            Err(format!(
                    "configured memory requires {required_memory} bytes but only {available_memory} bytes are available"
            ))
        } else {
            Ok(())
        }
    }) {
        Ok(()) => HostCheckStatus::Passed,
        Err(error) => HostCheckStatus::Failed(error),
    }
}

fn disk_status(server_root: &Path) -> HostCheckStatus {
    match statvfs(server_root) {
        Ok(filesystem) if filesystem.f_bavail.saturating_mul(filesystem.f_frsize) > 0 => {
            HostCheckStatus::Passed
        }
        Ok(_) => {
            HostCheckStatus::Failed("server-root filesystem has no available disk space".to_owned())
        }
        Err(error) => HostCheckStatus::Unavailable(format!(
            "could not inspect server-root disk space: {error}"
        )),
    }
}

fn session_lock_status(
    server_root: &Path,
    properties: &BTreeMap<String, PropertyValue>,
) -> HostCheckStatus {
    match active_session_lock(server_root, properties) {
        Ok(true) => {
            HostCheckStatus::Failed("an active Minecraft session.lock was found".to_owned())
        }
        Ok(false) => HostCheckStatus::Passed,
        Err(error) => HostCheckStatus::Unavailable(error),
    }
}

fn port_status(properties: &BTreeMap<String, PropertyValue>) -> HostCheckStatus {
    match configured_port_occupied(properties) {
        Ok(true) => {
            HostCheckStatus::Failed("the configured server port is already occupied".to_owned())
        }
        Ok(false) => HostCheckStatus::Passed,
        Err(error) => HostCheckStatus::Unavailable(error),
    }
}

/// Linux-backed running-instance probe using the world lock and daemon peer identity.
#[derive(Clone, Copy, Debug, Default)]
pub struct SystemRunningInstanceProbe;

impl RunningInstanceProbe for SystemRunningInstanceProbe {
    fn selected_instance_running(
        &self,
        path: &ConfigPath,
        config: &Config,
    ) -> Result<bool, String> {
        let server_root = path
            .as_path()
            .parent()
            .ok_or_else(|| "configuration path has no server root".to_owned())?;
        if active_session_lock(server_root, &config.minecraft.properties)? {
            return Ok(true);
        }
        daemon_peer_matches_selected(path, config)
    }
}

/// Successful output of a new-instance initialization.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InitSuccess {
    config_path: String,
}

impl InitSuccess {
    /// Returns the exact documented stdout payload.
    pub fn stdout(&self) -> String {
        format!(
            "Configuration: {}\nStart it with: minegr start --config {}\n",
            self.config_path, self.config_path
        )
    }
}

/// A failure while collecting, validating, or materializing a new instance.
#[derive(Debug, Error)]
pub enum InitError {
    /// Required flags were omitted where prompting is unavailable.
    #[error("Interactive input is unavailable; provide the required flags: {0}")]
    MissingFlags(String),
    /// A flag or interactive answer is invalid.
    #[error("Invalid init input: {0}")]
    InvalidInput(String),
    /// The Minecraft EULA was not accepted.
    #[error("Minecraft EULA acceptance is required; pass `--accept-eula`")]
    EulaDenied,
    /// The user declined the final creation confirmation.
    #[error("Instance creation was cancelled")]
    CreationDenied,
    /// An interactive prompt failed.
    #[error("Failed to collect init input: {0}")]
    Prompt(String),
    /// Upstream metadata could not produce the requested exact artifact.
    #[error("Failed to resolve server artifact: {0}")]
    ArtifactResolution(String),
    /// The host Java runtime is unavailable or incompatible.
    #[error("Java validation failed: {0}")]
    Java(String),
    /// The derived typed configuration failed pure validation.
    #[error("Invalid generated configuration: {0}")]
    Validation(String),
    /// The owner-only configuration could not be created atomically.
    #[error("Failed to write configuration: {0}")]
    WriteConfig(#[from] WriteConfigError),
    /// The pinned server artifact could not be materialized.
    #[error("Failed to materialize server.jar: {0}")]
    Download(String),
    /// A generated text file could not be materialized.
    #[error("Failed to materialize managed files: {0}")]
    ManagedFiles(#[from] ManagedFileError),
    /// The selected instance is running and cannot safely change identity.
    #[error("The selected instance is running; stop it before regenerating its UUID")]
    Running,
    /// The existing configuration source could not be read or edited safely.
    #[error("Failed to update existing configuration: {0}")]
    ExistingConfig(String),
    /// A host prerequisite makes initialization unsafe.
    #[error("Init host validation failed: {0}")]
    HostValidation(String),
}

impl InitError {
    /// Reports whether this failure belongs to the stable usage/configuration exit category.
    pub fn is_usage(&self) -> bool {
        matches!(
            self,
            Self::MissingFlags(_)
                | Self::InvalidInput(_)
                | Self::EulaDenied
                | Self::CreationDenied
                | Self::Prompt(_)
                | Self::ArtifactResolution(_)
                | Self::Java(_)
                | Self::Validation(_)
                | Self::Running
                | Self::ExistingConfig(_)
                | Self::HostValidation(_)
        )
    }
}

/// Collects, validates, writes, and materializes one new Minegr instance.
pub fn run_new_init(
    arguments: &InitArgs,
    path: &ConfigPath,
    prompts: &dyn InitPrompter,
    artifacts: &dyn ArtifactProvider,
    java: &dyn JavaRuntime,
    host: &dyn InitHostValidator,
) -> Result<InitSuccess, InitError> {
    reject_missing_non_interactive(arguments, prompts)?;
    validate_memory(&arguments.memory)?;
    let server_root = path
        .as_path()
        .parent()
        .expect("a resolved configuration path always has a parent");
    let default_name = server_root
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            InitError::InvalidInput("server-root directory has no UTF-8 name".to_owned())
        })?;
    let name = match arguments.name.as_deref() {
        Some(name) => name.to_owned(),
        None if prompts.is_interactive() => {
            prompts.name(default_name).map_err(InitError::Prompt)?
        }
        None => default_name.to_owned(),
    };
    if name.trim().is_empty() {
        return Err(InitError::InvalidInput(
            "server name must not be empty".to_owned(),
        ));
    }

    let mut versions = artifacts
        .versions()
        .map_err(InitError::ArtifactResolution)?;
    prioritize_releases(&mut versions);
    let version = match arguments.minecraft_version.as_deref() {
        Some(id) => versions
            .into_iter()
            .find(|version| version.id == id)
            .ok_or_else(|| {
                InitError::ArtifactResolution(format!(
                    "Minecraft version `{id}` is unavailable from Mojang"
                ))
            })?,
        None => {
            if versions.is_empty() {
                return Err(InitError::ArtifactResolution(
                    "Mojang returned no selectable Minecraft versions".to_owned(),
                ));
            }
            let selected = prompts.version(&versions).map_err(InitError::Prompt)?;
            versions.get(selected).cloned().ok_or_else(|| {
                InitError::Prompt("version prompt returned an invalid selection".to_owned())
            })?
        }
    };
    let availability = artifacts
        .available_platforms(&version)
        .map_err(InitError::ArtifactResolution)?;
    let platforms = available_platforms(availability);
    if platforms.is_empty() {
        return Err(InitError::ArtifactResolution(format!(
            "no eligible server platform exists for Minecraft {}",
            version.id
        )));
    }
    let platform = match arguments.platform {
        Some(platform) => {
            let platform = Platform::from(platform);
            if !platforms.contains(&platform) {
                return Err(InitError::ArtifactResolution(format!(
                    "no eligible {} artifact exists for Minecraft {}",
                    platform_name(platform),
                    version.id
                )));
            }
            platform
        }
        None => {
            let selected = prompts.platform(&platforms).map_err(InitError::Prompt)?;
            *platforms.get(selected).ok_or_else(|| {
                InitError::Prompt("platform prompt returned an invalid selection".to_owned())
            })?
        }
    };
    let artifact = artifacts
        .resolve(&version, platform)
        .map_err(InitError::ArtifactResolution)?;
    let eula = arguments.accept_eula || prompts.accept_eula().map_err(InitError::Prompt)?;
    if !eula {
        return Err(InitError::EulaDenied);
    }
    let config = config_from(arguments, name, artifact.clone(), eula);
    let report = validate_config(&config);
    if report.has_errors() {
        return Err(InitError::Validation(
            report
                .errors()
                .map(|finding| finding.message.as_str())
                .collect::<Vec<_>>()
                .join("; "),
        ));
    }
    let java_major = java
        .major_version(config.java.executable.as_deref())
        .map_err(InitError::Java)?;
    if java_major < artifact.required_java_major {
        return Err(InitError::Java(format!(
            "Minecraft {} requires Java {}, but Java {java_major} was found",
            artifact.version, artifact.required_java_major
        )));
    }
    host.validate(server_root, &config)
        .map_err(InitError::HostValidation)?;
    prompts.show_summary(&summary(&config));
    if !arguments.yes && !prompts.confirm_creation().map_err(InitError::Prompt)? {
        return Err(InitError::CreationDenied);
    }

    write_config(path, &config, WriteMode::Create)?;
    artifacts
        .download(&artifact, server_root)
        .map_err(InitError::Download)?;
    materialize_text_files(
        server_root,
        &config.minecraft.properties,
        config.minecraft.eula,
    )?;
    Ok(InitSuccess {
        config_path: path.as_str().to_owned(),
    })
}

/// Materializes an existing instance or changes only its UUID when requested.
pub fn run_existing_init(
    arguments: &InitArgs,
    path: &ConfigPath,
    loaded: &LoadedConfig,
    artifacts: &dyn ArtifactProvider,
    java: &dyn JavaRuntime,
    running: &dyn RunningInstanceProbe,
    host: &dyn InitHostValidator,
) -> Result<InitSuccess, InitError> {
    let config = &loaded.config;
    if arguments.uuid {
        if running
            .selected_instance_running(path, config)
            .map_err(InitError::ExistingConfig)?
        {
            return Err(InitError::Running);
        }
        regenerate_uuid(path, loaded)?;
        return Ok(InitSuccess {
            config_path: path.as_str().to_owned(),
        });
    }

    let server_root = path
        .as_path()
        .parent()
        .expect("a resolved configuration path always has a parent");
    preflight_text_files(
        server_root,
        &config.minecraft.properties,
        config.minecraft.eula,
    )?;
    let artifact = artifacts
        .resolve_pinned(&config.minecraft)
        .map_err(InitError::ArtifactResolution)?;
    let java_major = java
        .major_version(config.java.executable.as_deref())
        .map_err(InitError::Java)?;
    if java_major < artifact.required_java_major {
        return Err(InitError::Java(format!(
            "Minecraft {} requires Java {}, but Java {java_major} was found",
            artifact.version, artifact.required_java_major
        )));
    }
    host.validate(server_root, config)
        .map_err(InitError::HostValidation)?;
    artifacts
        .download(&artifact, server_root)
        .map_err(InitError::Download)?;
    materialize_text_files(
        server_root,
        &config.minecraft.properties,
        config.minecraft.eula,
    )?;
    Ok(InitSuccess {
        config_path: path.as_str().to_owned(),
    })
}

/// Replaces only `minegr.uuid` in a document-preserving atomic rewrite.
fn regenerate_uuid(path: &ConfigPath, loaded: &LoadedConfig) -> Result<(), InitError> {
    let mut document = loaded
        .source
        .parse::<DocumentMut>()
        .map_err(|error| InitError::ExistingConfig(error.to_string()))?;
    let current = document["minegr"]["uuid"]
        .as_value()
        .ok_or_else(|| InitError::ExistingConfig("minegr.uuid is not a value".to_owned()))?;
    let decor = current.decor().clone();
    let mut replacement = Value::from(Uuid::now_v7().to_string());
    *replacement.decor_mut() = decor;
    document["minegr"]["uuid"] = Item::Value(replacement);
    write_bytes_if_unchanged(path, document.to_string().as_bytes(), loaded.identity)?;
    Ok(())
}

/// Tests the configured world directory's advisory session lock without retaining it.
pub(crate) fn active_session_lock(
    server_root: &Path,
    properties: &BTreeMap<String, PropertyValue>,
) -> Result<bool, String> {
    let world = match properties.get("level-name") {
        Some(PropertyValue::String(name)) => name.as_str(),
        _ => "world",
    };
    let lock_path = server_root.join(world).join("session.lock");
    let file = match File::options().read(true).write(true).open(&lock_path) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(error) => {
            return Err(format!(
                "could not inspect {}: {error}",
                lock_path.display()
            ));
        }
    };
    match fcntl_lock(&file, FlockOperation::NonBlockingLockExclusive) {
        Ok(()) => {
            fcntl_lock(&file, FlockOperation::Unlock)
                .map_err(|error| format!("could not release {}: {error}", lock_path.display()))?;
            Ok(false)
        }
        Err(rustix::io::Errno::WOULDBLOCK) => Ok(true),
        Err(error) => Err(format!(
            "could not inspect {}: {error}",
            lock_path.display()
        )),
    }
}

/// Uses the configured TCP port as evidence of a possibly active server.
pub(crate) fn configured_port_occupied(
    properties: &BTreeMap<String, PropertyValue>,
) -> Result<bool, String> {
    let port = match properties.get("server-port") {
        Some(PropertyValue::Integer(port)) => u16::try_from(*port)
            .ok()
            .filter(|port| *port > 0)
            .ok_or_else(|| "configured server-port is outside 1..=65535".to_owned())?,
        None => 25_565,
        Some(_) => return Err("configured server-port is not an integer".to_owned()),
    };
    match TcpListener::bind(("0.0.0.0", port)) {
        Ok(listener) => {
            drop(listener);
            Ok(false)
        }
        Err(error) if error.kind() == io::ErrorKind::AddrInUse => Ok(true),
        Err(error) => Err(format!("could not inspect configured port {port}: {error}")),
    }
}

/// Reads Linux's current available-memory observation.
fn available_memory_bytes() -> Result<u64, String> {
    let source = fs::read_to_string("/proc/meminfo")
        .map_err(|error| format!("could not inspect available memory: {error}"))?;
    let kibibytes = source
        .lines()
        .find_map(|line| line.strip_prefix("MemAvailable:"))
        .and_then(|value| value.split_whitespace().next())
        .and_then(|value| value.parse::<u64>().ok())
        .ok_or_else(|| "could not parse MemAvailable from /proc/meminfo".to_owned())?;
    Ok(kibibytes.saturating_mul(1024))
}

/// Finds and converts the configured maximum heap size, if one is explicit.
fn configured_heap_bytes(arguments: &[String]) -> Result<u64, String> {
    let Some(value) = arguments
        .iter()
        .rev()
        .find_map(|argument| argument.strip_prefix("-Xmx"))
    else {
        return Ok(0);
    };
    parse_memory_bytes(value)
        .ok_or_else(|| format!("could not interpret configured maximum heap `-Xmx{value}`"))
}

/// Converts a positive Java memory size to bytes.
fn parse_memory_bytes(value: &str) -> Option<u64> {
    let (digits, multiplier) = match value.as_bytes().last().copied() {
        Some(b'K' | b'k') => (&value[..value.len() - 1], 1024_u64),
        Some(b'M' | b'm') => (&value[..value.len() - 1], 1024_u64.pow(2)),
        Some(b'G' | b'g') => (&value[..value.len() - 1], 1024_u64.pow(3)),
        Some(b'T' | b't') => (&value[..value.len() - 1], 1024_u64.pow(4)),
        Some(_) => (value, 1),
        None => return None,
    };
    digits
        .parse::<u64>()
        .ok()
        .filter(|amount| *amount > 0)
        .and_then(|amount| amount.checked_mul(multiplier))
}

/// Connects to the UUID socket and identifies whether its peer belongs to this config path.
fn daemon_peer_matches_selected(path: &ConfigPath, config: &Config) -> Result<bool, String> {
    let Some(uuid) = config.minegr.uuid.as_deref() else {
        return Ok(false);
    };
    let runtime_root = env::var_os("XDG_RUNTIME_DIR")
        .ok_or_else(|| "XDG_RUNTIME_DIR is unavailable".to_owned())?;
    let socket_path = PathBuf::from(runtime_root)
        .join("minegr")
        .join(format!("{uuid}.sock"));
    let stream = match UnixStream::connect(&socket_path) {
        Ok(stream) => stream,
        Err(error)
            if matches!(
                error.kind(),
                io::ErrorKind::NotFound | io::ErrorKind::ConnectionRefused
            ) =>
        {
            return Ok(false);
        }
        Err(error) => {
            return Err(format!(
                "could not inspect daemon socket {}: {error}",
                socket_path.display()
            ));
        }
    };
    let credentials = socket_peercred(&stream)
        .map_err(|error| format!("could not identify daemon peer: {error}"))?;
    let peer_cwd = fs::canonicalize(format!("/proc/{}/cwd", credentials.pid.as_raw_nonzero()))
        .map_err(|error| format!("could not inspect daemon working directory: {error}"))?;
    let selected_root = path
        .as_path()
        .parent()
        .ok_or_else(|| "configuration path has no server root".to_owned())?;
    Ok(peer_cwd == selected_root)
}

impl From<PlatformArg> for Platform {
    fn from(value: PlatformArg) -> Self {
        match value {
            PlatformArg::Vanilla => Self::Vanilla,
            PlatformArg::Paper => Self::Paper,
            PlatformArg::Fabric => Self::Fabric,
        }
    }
}

/// Rejects omitted answers before any upstream access when prompts cannot run.
fn reject_missing_non_interactive(
    arguments: &InitArgs,
    prompts: &dyn InitPrompter,
) -> Result<(), InitError> {
    if prompts.is_interactive() {
        return Ok(());
    }
    let mut missing = Vec::new();
    if arguments.minecraft_version.is_none() {
        missing.push("--minecraft-version");
    }
    if arguments.platform.is_none() {
        missing.push("--platform");
    }
    if !arguments.accept_eula {
        missing.push("--accept-eula");
    }
    if !arguments.yes {
        missing.push("--yes");
    }
    if missing.is_empty() {
        Ok(())
    } else {
        Err(InitError::MissingFlags(missing.join(", ")))
    }
}

/// Accepts positive whole-number heap sizes with a binary unit suffix.
fn validate_memory(memory: &str) -> Result<(), InitError> {
    let Some((digits, unit)) = memory.split_at_checked(memory.len().saturating_sub(1)) else {
        return Err(InitError::InvalidInput(
            "--memory must be a positive size such as 2G or 512M".to_owned(),
        ));
    };
    if digits.is_empty()
        || !digits.bytes().all(|byte| byte.is_ascii_digit())
        || digits.parse::<u64>().ok().is_none_or(|value| value == 0)
        || !matches!(unit, "K" | "M" | "G" | "T")
    {
        return Err(InitError::InvalidInput(
            "--memory must be a positive size such as 2G or 512M".to_owned(),
        ));
    }
    Ok(())
}

/// Keeps newest releases first while retaining upstream order within each classification.
fn prioritize_releases(versions: &mut [MinecraftVersion]) {
    versions.sort_by_key(|version| match version.kind {
        MinecraftVersionKind::Release => 0,
        MinecraftVersionKind::Snapshot => 1,
    });
}

/// Converts availability flags to the stable platform prompt order.
fn available_platforms(availability: PlatformAvailability) -> Vec<Platform> {
    let mut platforms = Vec::new();
    if availability.vanilla {
        platforms.push(Platform::Vanilla);
    }
    if availability.paper {
        platforms.push(Platform::Paper);
    }
    if availability.fabric {
        platforms.push(Platform::Fabric);
    }
    platforms
}

/// Derives the complete runtime configuration from validated questionnaire values.
fn config_from(
    arguments: &InitArgs,
    name: String,
    artifact: ResolvedArtifact,
    eula: bool,
) -> Config {
    Config {
        minegr: MinegrConfig {
            config_version: CONFIG_VERSION,
            uuid: Some(Uuid::now_v7().to_string()),
            name,
        },
        minecraft: MinecraftConfig {
            platform: artifact.platform,
            version: artifact.version,
            build: artifact.build,
            loader: artifact.loader,
            installer: artifact.installer,
            checksum: artifact.checksum,
            eula,
            properties: BTreeMap::from([(
                "server-port".to_owned(),
                PropertyValue::Integer(i64::from(arguments.port)),
            )]),
        },
        java: JavaConfig {
            executable: None,
            jvm_args: vec![
                format!("-Xms{}", arguments.memory),
                format!("-Xmx{}", arguments.memory),
            ],
            server_args: vec!["nogui".to_owned()],
        },
    }
}

/// Formats the human-readable pre-creation summary sent to stderr.
fn summary(config: &Config) -> String {
    let mut output = format!(
        "Name: {}\nMinecraft: {} {}\nMemory: {}\nPort: {}\nEULA accepted: {}",
        config.minegr.name,
        platform_name(config.minecraft.platform),
        config.minecraft.version,
        config
            .java
            .jvm_args
            .first()
            .and_then(|argument| argument.strip_prefix("-Xms"))
            .unwrap_or("unknown"),
        match config.minecraft.properties.get("server-port") {
            Some(PropertyValue::Integer(port)) => port.to_string(),
            _ => "unknown".to_owned(),
        },
        config.minecraft.eula
    );
    if let Some(build) = config.minecraft.build {
        output.push_str(&format!("\nPaper build: {build}"));
    }
    if let Some(loader) = config.minecraft.loader.as_deref() {
        output.push_str(&format!("\nFabric Loader: {loader}"));
    }
    if let Some(installer) = config.minecraft.installer.as_deref() {
        output.push_str(&format!("\nFabric Installer: {installer}"));
    }
    if let Some(checksum) = config.minecraft.checksum.as_deref() {
        output.push_str(&format!("\nChecksum: {checksum}"));
    }
    output
}

/// Returns the stable lowercase platform spelling.
fn platform_name(platform: Platform) -> &'static str {
    match platform {
        Platform::Vanilla => "vanilla",
        Platform::Paper => "paper",
        Platform::Fabric => "fabric",
    }
}

/// Parses the conventional quoted `java -version` token into a major version.
fn parse_java_major(output: &str) -> Option<u32> {
    let version = output.split('"').nth(1)?;
    let first = version.split('.').next()?.parse::<u32>().ok()?;
    if first == 1 {
        version.split('.').nth(1)?.parse().ok()
    } else {
        Some(first)
    }
}

#[cfg(test)]
mod tests {
    use super::{configured_heap_bytes, parse_java_major, parse_memory_bytes};

    #[test]
    fn java_major_parser_accepts_legacy_and_modern_version_output() {
        assert_eq!(parse_java_major(r#"java version "1.8.0_442""#), Some(8));
        assert_eq!(
            parse_java_major(r#"openjdk version "21.0.7" 2025-04-15"#),
            Some(21)
        );
        assert_eq!(parse_java_major("unrecognized output"), None);
    }

    #[test]
    fn configured_heap_parser_uses_the_last_maximum_and_binary_units() {
        assert_eq!(parse_memory_bytes("2G"), Some(2 * 1024_u64.pow(3)));
        assert_eq!(parse_memory_bytes("512M"), Some(512 * 1024_u64.pow(2)));
        assert_eq!(parse_memory_bytes("0G"), None);
        assert_eq!(
            configured_heap_bytes(&["-Xmx1G".to_owned(), "-Xmx2G".to_owned()]).unwrap(),
            2 * 1024_u64.pow(3)
        );
    }
}
