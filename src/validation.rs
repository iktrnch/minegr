//! Deterministic pure and host-dependent validation.

use std::fmt;

use uuid::Uuid;

use crate::config::{CONFIG_VERSION, Config, Platform, PropertyValue};

/// Stable identifiers for validation findings and dependencies.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ValidationCode {
    /// The configuration format version is unsupported.
    ConfigVersion,
    /// The instance UUID is absent or invalid.
    Uuid,
    /// Platform coordinates are missing or incompatible.
    PlatformCoordinates,
    /// A checksum does not use `<algorithm>:<hex>`.
    Checksum,
    /// Java launch arguments conflict with Minegr-owned arguments.
    JavaArguments,
    /// The configured server port is invalid.
    ServerPort,
    /// The white-list property has the wrong type.
    WhiteList,
    /// The level-name property is invalid.
    LevelName,
    /// Minecraft's EULA has not been accepted.
    Eula,
    /// Configuration file type could not be verified as regular.
    ConfigFileType,
    /// Configuration ownership is unavailable or unsafe.
    ConfigOwner,
    /// Configuration permissions are broader than owner-only.
    ConfigPermissions,
    /// The configured Java runtime is missing or incompatible.
    JavaCompatibility,
    /// A managed path is missing, conflicting, or unsafe.
    ManagedArtifact,
    /// A managed artifact does not match its checksum.
    ArtifactChecksum,
    /// Available memory cannot satisfy the launch configuration.
    Memory,
    /// Available disk space cannot satisfy the operation.
    DiskSpace,
    /// The configured port cannot be used safely.
    Port,
    /// An active process or session lock conflicts with the operation.
    ActiveProcess,
    /// The server root conflicts with an owned path or operation.
    ServerRoot,
}

/// Minecraft properties whose semantics are defined by this schema version.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecognizedProperty {
    /// TCP listening port.
    ServerPort,
    /// Whether the whitelist is enforced.
    WhiteList,
    /// Primary world directory name.
    LevelName,
}

impl RecognizedProperty {
    /// Complete recognized-property registry for configuration version 1.
    pub const ALL: [Self; 3] = [Self::ServerPort, Self::WhiteList, Self::LevelName];

    /// Returns the `server.properties` spelling.
    pub const fn name(self) -> &'static str {
        match self {
            Self::ServerPort => "server-port",
            Self::WhiteList => "white-list",
            Self::LevelName => "level-name",
        }
    }
}

impl fmt::Display for ValidationCode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::ConfigVersion => "config-version",
            Self::Uuid => "uuid",
            Self::PlatformCoordinates => "platform-coordinates",
            Self::Checksum => "checksum",
            Self::JavaArguments => "java-arguments",
            Self::ServerPort => "server-port",
            Self::WhiteList => "white-list",
            Self::LevelName => "level-name",
            Self::Eula => "eula",
            Self::ConfigFileType => "config-file-type",
            Self::ConfigOwner => "config-owner",
            Self::ConfigPermissions => "config-permissions",
            Self::JavaCompatibility => "java-compatibility",
            Self::ManagedArtifact => "managed-artifact",
            Self::ArtifactChecksum => "artifact-checksum",
            Self::Memory => "memory",
            Self::DiskSpace => "disk-space",
            Self::Port => "port",
            Self::ActiveProcess => "active-process",
            Self::ServerRoot => "server-root",
        };
        formatter.write_str(name)
    }
}

/// Whether a finding blocks an operation, warns, or records a skipped check.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValidationLevel {
    /// The operation must not proceed.
    Error,
    /// The operation may proceed after presenting the finding.
    Warning,
    /// A prerequisite prevented this check from running.
    Skipped,
}

/// One stable, user-presentable validation finding.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidationFinding {
    /// Stable finding identifier.
    pub code: ValidationCode,
    /// Finding severity.
    pub level: ValidationLevel,
    /// Safe user-facing explanation.
    pub message: String,
}

/// Ordered findings produced by a validation pass.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ValidationReport {
    findings: Vec<ValidationFinding>,
}

impl ValidationReport {
    /// Returns findings in deterministic validation order.
    pub fn findings(&self) -> &[ValidationFinding] {
        &self.findings
    }

    /// Returns only error findings without changing their order.
    pub fn errors(&self) -> impl Iterator<Item = &ValidationFinding> {
        self.findings
            .iter()
            .filter(|finding| finding.level == ValidationLevel::Error)
    }

    /// Returns only warning findings without changing their order.
    pub fn warnings(&self) -> impl Iterator<Item = &ValidationFinding> {
        self.findings
            .iter()
            .filter(|finding| finding.level == ValidationLevel::Warning)
    }

    /// Reports whether any finding blocks the operation.
    pub fn has_errors(&self) -> bool {
        self.errors().next().is_some()
    }

    /// Reports whether an error with a particular code occurred.
    pub fn has_error(&self, code: ValidationCode) -> bool {
        self.errors().any(|finding| finding.code == code)
    }

    /// Returns finding codes in their observable order.
    pub fn codes(&self) -> Vec<ValidationCode> {
        self.findings.iter().map(|finding| finding.code).collect()
    }

    /// Appends another validation pass while preserving both reports' order.
    pub fn append(&mut self, other: Self) {
        self.findings.extend(other.findings);
    }

    fn push(&mut self, code: ValidationCode, level: ValidationLevel, message: String) {
        self.findings.push(ValidationFinding {
            code,
            level,
            message,
        });
    }

    fn error(&mut self, code: ValidationCode, message: impl Into<String>) {
        self.push(code, ValidationLevel::Error, message.into());
    }

    fn warning(&mut self, code: ValidationCode, message: impl Into<String>) {
        self.push(code, ValidationLevel::Warning, message.into());
    }

    fn skipped(&mut self, code: ValidationCode, message: impl Into<String>) {
        self.push(code, ValidationLevel::Skipped, message.into());
    }
}

/// A stateless validation rule over one input type.
pub trait Validator<T> {
    /// Appends this rule's independent findings to the report.
    fn validate(&self, input: &T, report: &mut ValidationReport);
}

/// Runs every pure configuration check in stable documented order.
pub fn validate_config(config: &Config) -> ValidationReport {
    let validators: [&dyn Validator<Config>; 7] = [
        &VersionValidator,
        &UuidValidator,
        &PlatformValidator,
        &ChecksumValidator,
        &JavaArgumentsValidator,
        &MinecraftPropertiesValidator,
        &CrossFieldValidator,
    ];
    let mut report = ValidationReport::default();
    for validator in validators {
        validator.validate(config, &mut report);
    }
    report
}

struct VersionValidator;

impl Validator<Config> for VersionValidator {
    fn validate(&self, config: &Config, report: &mut ValidationReport) {
        if config.minegr.config_version != CONFIG_VERSION {
            report.error(
                ValidationCode::ConfigVersion,
                format!(
                    "unsupported config_version {}; expected {CONFIG_VERSION}",
                    config.minegr.config_version
                ),
            );
        }
    }
}

struct UuidValidator;

impl Validator<Config> for UuidValidator {
    fn validate(&self, config: &Config, report: &mut ValidationReport) {
        match config.minegr.uuid.as_deref() {
            Some(value) if Uuid::parse_str(value).is_ok() => {}
            Some(_) => report.error(ValidationCode::Uuid, "minegr.uuid is not a valid UUID"),
            None => report.error(
                ValidationCode::Uuid,
                "minegr.uuid is required; generate it with `minegr init --uuid`",
            ),
        }
    }
}

struct PlatformValidator;

impl Validator<Config> for PlatformValidator {
    fn validate(&self, config: &Config, report: &mut ValidationReport) {
        let minecraft = &config.minecraft;
        if minecraft.version.trim().is_empty() {
            report.error(
                ValidationCode::PlatformCoordinates,
                "minecraft.version must not be empty",
            );
        }

        match minecraft.platform {
            Platform::Vanilla => {
                reject_coordinate(minecraft.build.is_some(), "build", "vanilla", report);
                reject_coordinate(minecraft.loader.is_some(), "loader", "vanilla", report);
                reject_coordinate(
                    minecraft.installer.is_some(),
                    "installer",
                    "vanilla",
                    report,
                );
            }
            Platform::Paper => {
                if minecraft.build.is_none() {
                    report.error(
                        ValidationCode::PlatformCoordinates,
                        "minecraft.build is required for paper",
                    );
                }
                reject_coordinate(minecraft.loader.is_some(), "loader", "paper", report);
                reject_coordinate(minecraft.installer.is_some(), "installer", "paper", report);
            }
            Platform::Fabric => {
                reject_coordinate(minecraft.build.is_some(), "build", "fabric", report);
                require_nonempty_coordinate(
                    minecraft.loader.as_deref(),
                    "loader",
                    "fabric",
                    report,
                );
                require_nonempty_coordinate(
                    minecraft.installer.as_deref(),
                    "installer",
                    "fabric",
                    report,
                );
            }
        }
    }
}

fn reject_coordinate(present: bool, field: &str, platform: &str, report: &mut ValidationReport) {
    if present {
        report.error(
            ValidationCode::PlatformCoordinates,
            format!("minecraft.{field} is not allowed for {platform}"),
        );
    }
}

fn require_nonempty_coordinate(
    value: Option<&str>,
    field: &str,
    platform: &str,
    report: &mut ValidationReport,
) {
    if value.is_none_or(|value| value.trim().is_empty()) {
        report.error(
            ValidationCode::PlatformCoordinates,
            format!("minecraft.{field} is required for {platform}"),
        );
    }
}

struct ChecksumValidator;

impl Validator<Config> for ChecksumValidator {
    fn validate(&self, config: &Config, report: &mut ValidationReport) {
        let Some(checksum) = config.minecraft.checksum.as_deref() else {
            return;
        };
        let valid = checksum.split_once(':').is_some_and(|(algorithm, digest)| {
            !algorithm.is_empty()
                && algorithm
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
                && !digest.is_empty()
                && digest.bytes().all(|byte| byte.is_ascii_hexdigit())
        });
        if !valid {
            report.error(
                ValidationCode::Checksum,
                "minecraft.checksum must use <algorithm>:<hex>",
            );
        }
    }
}

struct JavaArgumentsValidator;

impl Validator<Config> for JavaArgumentsValidator {
    fn validate(&self, config: &Config, report: &mut ValidationReport) {
        if config.java.executable.as_deref().is_some_and(has_nul) {
            report.error(
                ValidationCode::JavaArguments,
                "java.executable contains a NUL byte",
            );
        }
        for argument in &config.java.jvm_args {
            if argument == "-jar" || argument == "./server.jar" {
                report.error(
                    ValidationCode::JavaArguments,
                    format!("java.jvm_args must not contain Minegr-owned argument `{argument}`"),
                );
            } else if has_nul(argument) {
                report.error(
                    ValidationCode::JavaArguments,
                    "java.jvm_args contains a NUL byte",
                );
            }
        }
        for argument in &config.java.server_args {
            if has_nul(argument) {
                report.error(
                    ValidationCode::JavaArguments,
                    "java.server_args contains a NUL byte",
                );
            }
        }
    }
}

fn has_nul(value: &str) -> bool {
    value.as_bytes().contains(&0)
}

struct MinecraftPropertiesValidator;

impl Validator<Config> for MinecraftPropertiesValidator {
    fn validate(&self, config: &Config, report: &mut ValidationReport) {
        let properties = &config.minecraft.properties;
        for property in RecognizedProperty::ALL {
            let Some(value) = properties.get(property.name()) else {
                continue;
            };
            match (property, value) {
                (RecognizedProperty::ServerPort, PropertyValue::Integer(1..=65_535)) => {}
                (RecognizedProperty::WhiteList, PropertyValue::Boolean(_)) => {}
                (RecognizedProperty::LevelName, PropertyValue::String(name))
                    if !name.trim().is_empty() => {}
                (RecognizedProperty::ServerPort, _) => report.error(
                    ValidationCode::ServerPort,
                    "minecraft.properties.server-port must be an integer in 1..=65535",
                ),
                (RecognizedProperty::WhiteList, _) => report.error(
                    ValidationCode::WhiteList,
                    "minecraft.properties.white-list must be a boolean",
                ),
                (RecognizedProperty::LevelName, _) => report.error(
                    ValidationCode::LevelName,
                    "minecraft.properties.level-name must be a non-empty string",
                ),
            }
        }
    }
}

struct CrossFieldValidator;

impl Validator<Config> for CrossFieldValidator {
    fn validate(&self, config: &Config, report: &mut ValidationReport) {
        if !config.minecraft.eula {
            report.error(
                ValidationCode::Eula,
                "minecraft.eula must record explicit acceptance before materialization or launch",
            );
        }
    }
}

/// A host observation that may be unavailable without being treated as success.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Observation<T> {
    /// A host value was observed.
    Known(T),
    /// The host value could not be determined.
    Unavailable(String),
}

/// The observed type of the selected configuration path.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileKind {
    /// A regular file.
    Regular,
    /// Any directory, symlink, device, socket, or other non-regular entry.
    NonRegular,
}

/// Host facts about the selected configuration file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfigFileObservation {
    /// Observed filesystem type.
    pub kind: Observation<FileKind>,
    /// Observed owner UID.
    pub owner_uid: Observation<u32>,
    /// UID that is allowed to access the file.
    pub expected_uid: u32,
    /// Observed Unix mode bits.
    pub mode: Observation<u32>,
}

/// Stable ordering groups for host-dependent checks.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum HostCheckKind {
    /// Java availability and version compatibility.
    JavaCompatibility,
    /// Managed file presence, content, and path safety.
    ManagedArtifact,
    /// Artifact checksum agreement.
    ArtifactChecksum,
    /// Memory capacity.
    Memory,
    /// Disk-space capacity.
    DiskSpace,
    /// Configured port availability.
    Port,
    /// Active process and `session.lock` conflicts.
    ActiveProcess,
    /// Server-root conflicts.
    ServerRoot,
}

impl HostCheckKind {
    fn code(self) -> ValidationCode {
        match self {
            Self::JavaCompatibility => ValidationCode::JavaCompatibility,
            Self::ManagedArtifact => ValidationCode::ManagedArtifact,
            Self::ArtifactChecksum => ValidationCode::ArtifactChecksum,
            Self::Memory => ValidationCode::Memory,
            Self::DiskSpace => ValidationCode::DiskSpace,
            Self::Port => ValidationCode::Port,
            Self::ActiveProcess => ValidationCode::ActiveProcess,
            Self::ServerRoot => ValidationCode::ServerRoot,
        }
    }
}

/// Result supplied by a read-only host probe.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HostCheckStatus {
    /// The requirement was verified.
    Passed,
    /// The requirement was checked and failed.
    Failed(String),
    /// The requirement could not be checked and must not be assumed satisfied.
    Unavailable(String),
}

/// One exact prerequisite for a host check.
#[derive(Clone, Debug, PartialEq, Eq)]
enum HostPrerequisite {
    Pure(ValidationCode),
    Host {
        kind: HostCheckKind,
        subject: Option<String>,
    },
}

/// One host check and the pure checks it depends on.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostCheck {
    kind: HostCheckKind,
    subject: Option<String>,
    status: HostCheckStatus,
    prerequisites: Vec<HostPrerequisite>,
}

impl HostCheck {
    /// Creates a host check from a supplied read-only observation.
    pub fn new(kind: HostCheckKind, subject: Option<String>, status: HostCheckStatus) -> Self {
        Self {
            kind,
            subject,
            status,
            prerequisites: Vec::new(),
        }
    }

    /// Declares one pure finding that prevents this check from running.
    pub fn requiring(mut self, prerequisite: ValidationCode) -> Self {
        self.prerequisites
            .push(HostPrerequisite::Pure(prerequisite));
        self
    }

    /// Declares one exact host check that prevents this check from running.
    pub fn requiring_host(mut self, kind: HostCheckKind, subject: Option<String>) -> Self {
        self.prerequisites
            .push(HostPrerequisite::Host { kind, subject });
        self
    }
}

/// Read-only observations consumed by host validation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostValidationInput {
    /// Selected configuration-file facts.
    pub config_file: ConfigFileObservation,
    /// Java, managed-path, checksum, memory, disk, port, process, and root checks.
    pub checks: Vec<HostCheck>,
}

/// Validates supplied host observations in stable order without host side effects.
pub fn validate_host(input: &HostValidationInput, pure: &ValidationReport) -> ValidationReport {
    let mut report = ValidationReport::default();
    validate_file_observation(&input.config_file, &mut report);

    let mut checks: Vec<&HostCheck> = input.checks.iter().collect();
    checks.sort_by(|left, right| {
        left.kind
            .cmp(&right.kind)
            .then_with(|| left.subject.cmp(&right.subject))
    });
    for check in &checks {
        let code = check.kind.code();
        if let Some(prerequisite) = failed_prerequisite(check, &checks, pure, &mut Vec::new()) {
            report.skipped(
                code,
                format!("skipped {code} because {prerequisite} validation failed"),
            );
            continue;
        }

        let subject = check
            .subject
            .as_deref()
            .map(|value| format!(" `{value}`"))
            .unwrap_or_default();
        match &check.status {
            HostCheckStatus::Passed => {}
            HostCheckStatus::Failed(reason) => {
                report.error(code, format!("{code}{subject}: {reason}"));
            }
            HostCheckStatus::Unavailable(reason) => {
                report.error(
                    code,
                    format!("could not validate {code}{subject}: {reason}"),
                );
            }
        }
    }
    report
}

fn failed_prerequisite(
    check: &HostCheck,
    checks: &[&HostCheck],
    pure: &ValidationReport,
    visiting: &mut Vec<(HostCheckKind, Option<String>)>,
) -> Option<String> {
    let identity = (check.kind, check.subject.clone());
    if visiting.contains(&identity) {
        return None;
    }
    visiting.push(identity);

    let failed = check
        .prerequisites
        .iter()
        .find_map(|prerequisite| match prerequisite {
            HostPrerequisite::Pure(code) if pure.has_error(*code) => Some(code.to_string()),
            HostPrerequisite::Pure(_) => None,
            HostPrerequisite::Host { kind, subject } => checks
                .iter()
                .find(|candidate| candidate.kind == *kind && candidate.subject == *subject)
                .and_then(|candidate| {
                    let directly_failed = !matches!(candidate.status, HostCheckStatus::Passed);
                    if directly_failed
                        || failed_prerequisite(candidate, checks, pure, visiting).is_some()
                    {
                        Some(format_host_identity(*kind, subject.as_deref()))
                    } else {
                        None
                    }
                }),
        });
    visiting.pop();
    failed
}

fn format_host_identity(kind: HostCheckKind, subject: Option<&str>) -> String {
    let code = kind.code();
    match subject {
        Some(subject) => format!("{code} `{subject}`"),
        None => code.to_string(),
    }
}

fn validate_file_observation(observation: &ConfigFileObservation, report: &mut ValidationReport) {
    match &observation.kind {
        Observation::Known(FileKind::Regular) => {}
        Observation::Known(FileKind::NonRegular) => report.error(
            ValidationCode::ConfigFileType,
            "configuration path is not a regular file",
        ),
        Observation::Unavailable(reason) => report.error(
            ValidationCode::ConfigFileType,
            format!("could not validate configuration file type: {reason}"),
        ),
    }
    match &observation.owner_uid {
        Observation::Known(owner) if *owner == observation.expected_uid => {}
        Observation::Known(owner) => report.error(
            ValidationCode::ConfigOwner,
            format!(
                "configuration is owned by user {owner}, expected user {}",
                observation.expected_uid
            ),
        ),
        Observation::Unavailable(reason) => report.error(
            ValidationCode::ConfigOwner,
            format!("could not validate configuration ownership: {reason}"),
        ),
    }
    match &observation.mode {
        Observation::Known(mode) if mode & 0o077 == 0 => {}
        Observation::Known(mode) => report.warning(
            ValidationCode::ConfigPermissions,
            format!(
                "configuration permissions {:04o} are broader than owner-only",
                mode & 0o777
            ),
        ),
        Observation::Unavailable(reason) => report.error(
            ValidationCode::ConfigPermissions,
            format!("could not validate configuration permissions: {reason}"),
        ),
    }
}
