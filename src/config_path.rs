//! Canonical configuration-path resolution.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use thiserror::Error;

/// A canonical, absolute, UTF-8 configuration path.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfigPath {
    path: PathBuf,
    selected: PathBuf,
    utf8: String,
}

impl ConfigPath {
    /// Returns the canonical filesystem path.
    pub fn as_path(&self) -> &Path {
        &self.path
    }

    /// Returns the canonical path as UTF-8 for protocol and output use.
    pub fn as_str(&self) -> &str {
        &self.utf8
    }

    /// Returns the absolute path as selected before existing-file canonicalization.
    pub fn selected_path(&self) -> &Path {
        &self.selected
    }

    fn new(path: PathBuf, selected: PathBuf) -> Result<Self, ConfigPathError> {
        let utf8 = path
            .to_str()
            .ok_or_else(|| ConfigPathError::InvalidUtf8 { path: path.clone() })?
            .to_owned();
        Ok(Self {
            path,
            selected,
            utf8,
        })
    }
}

/// A failure to resolve a selected configuration path.
#[derive(Debug, Error)]
pub enum ConfigPathError {
    /// A filesystem component could not be resolved.
    #[error("failed to resolve {path}: {source}")]
    Resolve {
        /// The path whose resolution failed.
        path: PathBuf,
        /// The operating-system error.
        #[source]
        source: io::Error,
    },
    /// A new configuration path does not name a file.
    #[error("configuration path must name a file: {path}")]
    MissingFileName {
        /// The invalid selected path.
        path: PathBuf,
    },
    /// The selected parent exists but is not a directory.
    #[error("configuration parent is not a directory: {path}")]
    ParentNotDirectory {
        /// The invalid parent path.
        path: PathBuf,
    },
    /// The canonical path cannot be represented in the protocol's UTF-8 form.
    #[error("Configuration path is not valid UTF-8: {}", path.display())]
    InvalidUtf8 {
        /// The canonical non-UTF-8 path.
        path: PathBuf,
    },
}

/// Resolves an existing configuration directly, without searching ancestors.
pub fn resolve_existing_path(
    selected: &Path,
    current_directory: &Path,
) -> Result<ConfigPath, ConfigPathError> {
    let candidate = absolute_candidate(selected, current_directory);
    let canonical = fs::canonicalize(&candidate).map_err(|source| ConfigPathError::Resolve {
        path: candidate.clone(),
        source,
    })?;
    ConfigPath::new(canonical, candidate)
}

/// Resolves a path that `init` may create by canonicalizing its existing parent.
pub fn resolve_creation_path(
    selected: &Path,
    current_directory: &Path,
) -> Result<ConfigPath, ConfigPathError> {
    let candidate = absolute_candidate(selected, current_directory);
    if candidate.exists() {
        return resolve_existing_path(&candidate, current_directory);
    }

    let file_name = candidate
        .file_name()
        .ok_or_else(|| ConfigPathError::MissingFileName {
            path: candidate.clone(),
        })?;
    let parent = candidate
        .parent()
        .ok_or_else(|| ConfigPathError::MissingFileName {
            path: candidate.clone(),
        })?;
    let canonical_parent = fs::canonicalize(parent).map_err(|source| ConfigPathError::Resolve {
        path: parent.to_owned(),
        source,
    })?;
    if !canonical_parent.is_dir() {
        return Err(ConfigPathError::ParentNotDirectory {
            path: canonical_parent,
        });
    }

    ConfigPath::new(canonical_parent.join(file_name), candidate)
}

fn absolute_candidate(selected: &Path, current_directory: &Path) -> PathBuf {
    if selected.is_absolute() {
        selected.to_owned()
    } else {
        current_directory.join(selected)
    }
}
