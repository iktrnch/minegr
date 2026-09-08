//! Owner-only atomic configuration publication.

use std::fs::{self, File, Permissions};
use std::io::{self, Write};
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};

use rustix::fs::{
    AtFlags, FileType, Mode, OFlags, RenameFlags, openat, renameat_with, statat, unlinkat,
};
use rustix::process::geteuid;
use thiserror::Error;
use uuid::Uuid;

use crate::config::Config;
use crate::config_path::ConfigPath;

/// Whether publication must create a new target or replace a safe existing one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WriteMode {
    /// Publish only if the target does not exist.
    Create,
    /// Atomically replace an existing regular current-user-owned target.
    Replace,
}

/// File types relevant to safe replacement.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TargetKind {
    /// A regular file.
    Regular,
    /// A symlink, directory, device, socket, or other non-regular entry.
    NonRegular,
}

/// Metadata needed to authorize a target replacement.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TargetMetadata {
    /// Observed target type without following a final symlink.
    pub kind: TargetKind,
    /// Observed Unix owner ID.
    pub owner_uid: u32,
    /// Filesystem device identity.
    pub device_id: u64,
    /// Filesystem inode identity.
    pub inode: u64,
}

/// Whether a publication failure happened before or after an atomic exchange.
#[derive(Debug)]
pub enum PublishError {
    /// The target was not changed and the temporary file is safe to remove.
    BeforePublication(io::Error),
    /// The exchange was rolled back; the temporary path holds the unpublished replacement.
    RolledBack(io::Error),
    /// An exchange happened; the displaced entry must be retained for recovery.
    AfterExchange(io::Error),
}

/// Filesystem operations required by atomic configuration publication.
pub trait AtomicWriteOps {
    /// Writable temporary-file handle.
    type File: Write;

    /// Inspects a final target without following a final symlink.
    fn inspect_target(&mut self, path: &Path) -> io::Result<Option<TargetMetadata>>;
    /// Creates one unique adjacent owner-only temporary file.
    fn create_temp(&mut self, path: &Path) -> io::Result<Self::File>;
    /// Applies the exact owner-only permissions required before writing.
    fn set_temp_permissions(&mut self, file: &mut Self::File) -> io::Result<()>;
    /// Flushes a temporary file's contents and metadata to storage.
    fn sync_file(&mut self, file: &mut Self::File) -> io::Result<()>;
    /// Atomically publishes a temporary file according to the write mode.
    fn publish(
        &mut self,
        from: &Path,
        to: &Path,
        mode: WriteMode,
        expected: Option<TargetMetadata>,
    ) -> Result<(), PublishError>;
    /// Removes a temporary file after a pre-publication failure.
    fn remove_temp(&mut self, path: &Path) -> io::Result<()>;
    /// Flushes a publication directory after its entry changes.
    fn sync_parent(&mut self, path: &Path) -> io::Result<()>;
    /// Returns the effective user ID allowed to replace the target.
    fn current_uid(&self) -> u32;
}

/// A failure to serialize or atomically publish configuration.
#[derive(Debug, Error)]
pub enum WriteConfigError {
    /// The typed configuration could not be represented as TOML.
    #[error("failed to serialize configuration: {0}")]
    Serialize(#[from] toml::ser::Error),
    /// The target could not be inspected safely.
    #[error("failed to inspect configuration target: {0}")]
    Inspect(#[source] io::Error),
    /// Creation would replace an existing path.
    #[error("configuration already exists")]
    AlreadyExists,
    /// Replacement requires an existing target.
    #[error("configuration does not exist")]
    MissingTarget,
    /// Replacement would operate on a symlink or other non-regular path.
    #[error("configuration target is not a regular file")]
    NonRegularTarget,
    /// Replacement would overwrite another user's file.
    #[error("configuration target is owned by user {actual}, expected user {expected}")]
    ForeignOwner {
        /// Actual target owner.
        actual: u32,
        /// Required target owner.
        expected: u32,
    },
    /// The target no longer matches the entry authorized for replacement.
    #[error("configuration target changed before atomic publication")]
    ChangedTarget,
    /// An adjacent temporary file could not be created.
    #[error("failed to create adjacent temporary file: {0}")]
    CreateTemp(#[source] io::Error),
    /// Exact owner-only permissions could not be applied to the temporary file.
    #[error("failed to set temporary configuration permissions: {0}")]
    Permissions(#[source] io::Error),
    /// Temporary content could not be written or flushed.
    #[error("failed to write temporary configuration: {0}")]
    Write(#[source] io::Error),
    /// Temporary content could not be synchronized before publication.
    #[error("failed to synchronize temporary configuration: {0}")]
    SyncFile(#[source] io::Error),
    /// Atomic publication failed.
    #[error("failed to atomically publish configuration: {0}")]
    Rename(#[source] io::Error),
    /// An exchange happened, so the displaced entry was retained instead of cleaned up.
    #[error(
        "configuration publication could not be verified after exchange: {operation}; displaced entry retained at {}",
        path.display()
    )]
    IndeterminatePublication {
        /// Verification or rollback error after the exchange.
        operation: io::Error,
        /// Path retaining the displaced entry.
        path: PathBuf,
    },
    /// An exchange happened and even its directory durability could not be confirmed.
    #[error(
        "configuration publication could not be verified after exchange: {operation}; displaced entry retained at {}; directory synchronization also failed: {sync}",
        path.display()
    )]
    IndeterminatePublicationAndSync {
        /// Verification or rollback error after the exchange.
        operation: io::Error,
        /// Path retaining the displaced entry.
        path: PathBuf,
        /// Directory synchronization error.
        sync: io::Error,
    },
    /// The original target was restored and the unpublished replacement was retained.
    #[error(
        "configuration target changed during publication: {operation}; original target restored and unpublished replacement retained at {}",
        path.display()
    )]
    PublicationRolledBack {
        /// Reason the exchanged target did not match the authorized target.
        operation: io::Error,
        /// Path retaining the unpublished replacement.
        path: PathBuf,
    },
    /// Rollback succeeded, but its directory durability could not be confirmed.
    #[error(
        "configuration target changed during publication: {operation}; original target restored and unpublished replacement retained at {}; directory synchronization also failed: {sync}",
        path.display()
    )]
    PublicationRolledBackAndSync {
        /// Reason the exchanged target did not match the authorized target.
        operation: io::Error,
        /// Path retaining the unpublished replacement.
        path: PathBuf,
        /// Directory synchronization error.
        sync: io::Error,
    },
    /// Replacement was published, but the displaced old target could not be removed.
    #[error(
        "configuration was published but the replaced file at {} could not be removed: {operation}",
        path.display()
    )]
    RemoveReplaced {
        /// Removal error.
        operation: io::Error,
        /// Path retaining the displaced target.
        path: PathBuf,
    },
    /// Replacement and cleanup durability both failed.
    #[error(
        "configuration was published but the replaced file at {} could not be removed: {operation}; directory synchronization also failed: {sync}",
        path.display()
    )]
    RemoveReplacedAndSync {
        /// Removal error.
        operation: io::Error,
        /// Path retaining the displaced target.
        path: PathBuf,
        /// Directory synchronization error.
        sync: io::Error,
    },
    /// Publication succeeded, but its parent directory could not be synchronized.
    #[error("configuration was published but its directory could not be synchronized: {0}")]
    SyncParent(#[source] io::Error),
    /// A pre-publication failure was followed by a cleanup failure.
    #[error(
        "{stage} failed: {operation}; cleanup of temporary file {} also failed: {cleanup}",
        path.display()
    )]
    Cleanup {
        /// Operation that failed before publication.
        stage: &'static str,
        /// Original operation error.
        operation: io::Error,
        /// Temporary path that remains available for an explicit retry.
        path: PathBuf,
        /// Temporary-file removal error.
        cleanup: io::Error,
    },
}

/// Serializes and publishes a configuration using the system filesystem.
pub fn write_config(
    path: &ConfigPath,
    config: &Config,
    mode: WriteMode,
) -> Result<(), WriteConfigError> {
    let mut operations = SystemAtomicWriteOps::open(path)?;
    let baseline = match mode {
        WriteMode::Create => None,
        WriteMode::Replace => Some(verify_selected_target(&mut operations, path)?),
    };
    write_config_with_baseline(&mut operations, path, config, mode, baseline)
}

/// Serializes and publishes a configuration through an injected filesystem boundary.
pub fn write_config_with<O: AtomicWriteOps>(
    operations: &mut O,
    path: &ConfigPath,
    config: &Config,
    mode: WriteMode,
) -> Result<(), WriteConfigError> {
    write_config_with_baseline(operations, path, config, mode, None)
}

fn write_config_with_baseline<O: AtomicWriteOps>(
    operations: &mut O,
    path: &ConfigPath,
    config: &Config,
    mode: WriteMode,
    baseline: Option<TargetMetadata>,
) -> Result<(), WriteConfigError> {
    let expected = authorize_target(operations, path.as_path(), mode)?;
    if baseline.is_some() && baseline != expected {
        return Err(WriteConfigError::ChangedTarget);
    }
    let source = toml::to_string_pretty(config)?;
    let target = path.as_path();
    let parent = target
        .parent()
        .expect("a canonical configuration path always has a parent");
    let temp = temporary_path(target);
    let mut file = operations
        .create_temp(&temp)
        .map_err(WriteConfigError::CreateTemp)?;

    if let Err(error) = operations.set_temp_permissions(&mut file) {
        drop(file);
        return Err(cleanup_error(
            operations,
            &temp,
            "setting temporary configuration permissions",
            error,
            WriteConfigError::Permissions,
        ));
    }

    if let Err(error) = file
        .write_all(source.as_bytes())
        .and_then(|()| file.flush())
    {
        drop(file);
        return Err(cleanup_error(
            operations,
            &temp,
            "writing the temporary configuration",
            error,
            WriteConfigError::Write,
        ));
    }
    if let Err(error) = operations.sync_file(&mut file) {
        drop(file);
        return Err(cleanup_error(
            operations,
            &temp,
            "synchronizing the temporary configuration",
            error,
            WriteConfigError::SyncFile,
        ));
    }
    drop(file);

    if let Err(error) = operations.publish(&temp, target, mode, expected) {
        return match error {
            PublishError::BeforePublication(error) => Err(cleanup_error(
                operations,
                &temp,
                "publishing the configuration",
                error,
                WriteConfigError::Rename,
            )),
            PublishError::AfterExchange(operation) => match operations.sync_parent(parent) {
                Ok(()) => Err(WriteConfigError::IndeterminatePublication {
                    operation,
                    path: temp,
                }),
                Err(sync) => Err(WriteConfigError::IndeterminatePublicationAndSync {
                    operation,
                    path: temp,
                    sync,
                }),
            },
            PublishError::RolledBack(operation) => match operations.sync_parent(parent) {
                Ok(()) => Err(WriteConfigError::PublicationRolledBack {
                    operation,
                    path: temp,
                }),
                Err(sync) => Err(WriteConfigError::PublicationRolledBackAndSync {
                    operation,
                    path: temp,
                    sync,
                }),
            },
        };
    }
    if mode == WriteMode::Replace
        && let Err(operation) = operations.remove_temp(&temp)
    {
        return match operations.sync_parent(parent) {
            Ok(()) => Err(WriteConfigError::RemoveReplaced {
                operation,
                path: temp,
            }),
            Err(sync) => Err(WriteConfigError::RemoveReplacedAndSync {
                operation,
                path: temp,
                sync,
            }),
        };
    }
    operations
        .sync_parent(parent)
        .map_err(WriteConfigError::SyncParent)
}

fn authorize_target<O: AtomicWriteOps>(
    operations: &mut O,
    path: &Path,
    mode: WriteMode,
) -> Result<Option<TargetMetadata>, WriteConfigError> {
    let metadata = operations
        .inspect_target(path)
        .map_err(WriteConfigError::Inspect)?;
    match (mode, metadata) {
        (WriteMode::Create, None) => Ok(None),
        (WriteMode::Create, Some(_)) => Err(WriteConfigError::AlreadyExists),
        (WriteMode::Replace, None) => Err(WriteConfigError::MissingTarget),
        (WriteMode::Replace, Some(metadata)) if metadata.kind != TargetKind::Regular => {
            Err(WriteConfigError::NonRegularTarget)
        }
        (WriteMode::Replace, Some(metadata)) if metadata.owner_uid != operations.current_uid() => {
            Err(WriteConfigError::ForeignOwner {
                actual: metadata.owner_uid,
                expected: operations.current_uid(),
            })
        }
        (WriteMode::Replace, Some(metadata)) => Ok(Some(metadata)),
    }
}

fn verify_selected_target(
    operations: &mut SystemAtomicWriteOps,
    path: &ConfigPath,
) -> Result<TargetMetadata, WriteConfigError> {
    let selected = fs::symlink_metadata(path.selected_path()).map_err(WriteConfigError::Inspect)?;
    if !selected.file_type().is_file() {
        return Err(WriteConfigError::NonRegularTarget);
    }
    let selected = metadata_from_std(selected);
    let canonical = operations
        .inspect_target(path.as_path())
        .map_err(WriteConfigError::Inspect)?
        .ok_or(WriteConfigError::MissingTarget)?;
    if selected.device_id != canonical.device_id || selected.inode != canonical.inode {
        return Err(WriteConfigError::ChangedTarget);
    }
    Ok(canonical)
}

fn cleanup_error<O: AtomicWriteOps>(
    operations: &mut O,
    temp: &Path,
    stage: &'static str,
    operation: io::Error,
    original: impl FnOnce(io::Error) -> WriteConfigError,
) -> WriteConfigError {
    match operations.remove_temp(temp) {
        Ok(()) => original(operation),
        Err(cleanup) => WriteConfigError::Cleanup {
            stage,
            operation,
            path: temp.to_owned(),
            cleanup,
        },
    }
}

fn temporary_path(target: &Path) -> PathBuf {
    let name = target
        .file_name()
        .expect("a canonical configuration path always names a file")
        .to_string_lossy();
    target.with_file_name(format!(".{name}.{}.tmp", Uuid::now_v7()))
}

#[derive(Debug)]
struct SystemAtomicWriteOps {
    parent: File,
}

impl SystemAtomicWriteOps {
    fn open(path: &ConfigPath) -> Result<Self, WriteConfigError> {
        let parent = path
            .as_path()
            .parent()
            .expect("a canonical configuration path always has a parent");
        let descriptor = openat(
            rustix::fs::CWD,
            parent,
            OFlags::RDONLY | OFlags::DIRECTORY | OFlags::CLOEXEC | OFlags::NOFOLLOW,
            Mode::empty(),
        )
        .map_err(|error| WriteConfigError::Inspect(error.into()))?;
        Ok(Self {
            parent: File::from(descriptor),
        })
    }

    fn name<'a>(&self, path: &'a Path) -> &'a std::ffi::OsStr {
        path.file_name()
            .expect("a canonical configuration path always names a file")
    }
}

impl AtomicWriteOps for SystemAtomicWriteOps {
    type File = File;

    fn inspect_target(&mut self, path: &Path) -> io::Result<Option<TargetMetadata>> {
        match statat(&self.parent, self.name(path), AtFlags::SYMLINK_NOFOLLOW) {
            Ok(metadata) => Ok(Some(TargetMetadata {
                kind: if FileType::from_raw_mode(metadata.st_mode).is_file() {
                    TargetKind::Regular
                } else {
                    TargetKind::NonRegular
                },
                owner_uid: metadata.st_uid,
                device_id: metadata.st_dev,
                inode: metadata.st_ino,
            })),
            Err(error) if error == rustix::io::Errno::NOENT => Ok(None),
            Err(error) => Err(error.into()),
        }
    }

    fn create_temp(&mut self, path: &Path) -> io::Result<Self::File> {
        let descriptor = openat(
            &self.parent,
            self.name(path),
            OFlags::WRONLY | OFlags::CREATE | OFlags::EXCL | OFlags::CLOEXEC | OFlags::NOFOLLOW,
            Mode::RUSR | Mode::WUSR,
        )?;
        Ok(File::from(descriptor))
    }

    fn set_temp_permissions(&mut self, file: &mut Self::File) -> io::Result<()> {
        file.set_permissions(Permissions::from_mode(0o600))
    }

    fn sync_file(&mut self, file: &mut Self::File) -> io::Result<()> {
        file.sync_all()
    }

    fn publish(
        &mut self,
        from: &Path,
        to: &Path,
        mode: WriteMode,
        expected: Option<TargetMetadata>,
    ) -> Result<(), PublishError> {
        match mode {
            WriteMode::Create => renameat_with(
                &self.parent,
                self.name(from),
                &self.parent,
                self.name(to),
                RenameFlags::NOREPLACE,
            )
            .map_err(|error| PublishError::BeforePublication(error.into())),
            WriteMode::Replace => {
                let expected = expected.expect("replace publication has authorized metadata");
                renameat_with(
                    &self.parent,
                    self.name(from),
                    &self.parent,
                    self.name(to),
                    RenameFlags::EXCHANGE,
                )
                .map_err(|error| PublishError::BeforePublication(error.into()))?;
                let displaced = self
                    .inspect_target(from)
                    .map_err(PublishError::AfterExchange)?;
                if displaced == Some(expected) {
                    return Ok(());
                }

                renameat_with(
                    &self.parent,
                    self.name(from),
                    &self.parent,
                    self.name(to),
                    RenameFlags::EXCHANGE,
                )
                .map_err(|error| PublishError::AfterExchange(error.into()))?;
                Err(PublishError::RolledBack(io::Error::other(
                    "configuration target changed during atomic publication; exchange was rolled back",
                )))
            }
        }
    }

    fn remove_temp(&mut self, path: &Path) -> io::Result<()> {
        match unlinkat(&self.parent, self.name(path), AtFlags::empty()) {
            Ok(()) => Ok(()),
            Err(error) if error == rustix::io::Errno::NOENT => Ok(()),
            Err(error) => Err(error.into()),
        }
    }

    fn sync_parent(&mut self, _path: &Path) -> io::Result<()> {
        self.parent.sync_all()
    }

    fn current_uid(&self) -> u32 {
        geteuid().as_raw()
    }
}

fn metadata_from_std(metadata: fs::Metadata) -> TargetMetadata {
    TargetMetadata {
        kind: if metadata.file_type().is_file() {
            TargetKind::Regular
        } else {
            TargetKind::NonRegular
        },
        owner_uid: metadata.uid(),
        device_id: metadata.dev(),
        inode: metadata.ino(),
    }
}
