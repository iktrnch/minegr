//! Deterministic codecs and safe materialization for Minegr-managed files.

use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{self, Read};
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

use rustix::fs::{CWD, Mode, OFlags, openat};
use rustix::process::geteuid;
use thiserror::Error;

use crate::config::PropertyValue;
use crate::config_path::resolve_creation_path;
use crate::config_write::{WriteConfigError, WriteMode, write_bytes};

/// A failure while parsing `server.properties`.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum PropertiesError {
    /// A decoded property key occurs more than once.
    #[error("duplicate property `{key}` on line {line}")]
    Duplicate {
        /// Duplicate key.
        key: String,
        /// Physical line on which the duplicate completes.
        line: usize,
    },
    /// A property contains an incomplete or invalid escape.
    #[error("malformed property on line {line}: {reason}")]
    Malformed {
        /// Physical line on which the logical property completes.
        line: usize,
        /// Human-readable parse failure.
        reason: String,
    },
}

/// A failure while checking or creating a managed file.
#[derive(Debug, Error)]
pub enum ManagedFileError {
    /// The canonical server root could not be resolved.
    #[error("failed to resolve server root {}: {source}", path.display())]
    ResolveRoot {
        /// Selected root path.
        path: PathBuf,
        /// Resolution error.
        source: io::Error,
    },
    /// An existing managed path could not be inspected.
    #[error("failed to inspect managed file `{name}`: {source}")]
    Inspect {
        /// Stable managed filename.
        name: &'static str,
        /// Inspection error.
        source: io::Error,
    },
    /// A managed path required as an input does not exist.
    #[error("managed file `{name}` does not exist")]
    Missing {
        /// Stable managed filename.
        name: &'static str,
    },
    /// An existing managed path is not a regular file.
    #[error("managed path `{name}` is not a regular file; remove it before retrying")]
    NotRegular {
        /// Stable managed filename.
        name: &'static str,
    },
    /// An existing managed file is owned by another operating-system user.
    #[error("managed file `{name}` is owned by user {actual}, expected user {expected}")]
    ForeignOwner {
        /// Stable managed filename.
        name: &'static str,
        /// Observed owner UID.
        actual: u32,
        /// Current effective UID.
        expected: u32,
    },
    /// A managed path changed between inspection and opening.
    #[error("managed file `{name}` changed while it was being read")]
    ChangedDuringRead {
        /// Stable managed filename.
        name: &'static str,
    },
    /// An existing managed file differs from declarative configuration.
    #[error("managed file `{name}` differs from minegr.toml; remove it before retrying")]
    Differs {
        /// Stable managed filename.
        name: &'static str,
    },
    /// A regular managed file could not be read safely.
    #[error("failed to read managed file `{name}`: {source}")]
    Read {
        /// Stable managed filename.
        name: &'static str,
        /// Read error.
        source: io::Error,
    },
    /// A missing managed file could not be published atomically.
    #[error("failed to materialize managed file `{name}`: {source}")]
    Publish {
        /// Stable managed filename.
        name: &'static str,
        /// Atomic publication failure.
        source: WriteConfigError,
    },
}

/// Serializes configured values into deterministic Java-properties text.
pub fn render_server_properties(properties: &BTreeMap<String, PropertyValue>) -> String {
    let mut output = String::new();
    for (key, value) in properties {
        output.push_str(&escape_key(key));
        output.push('=');
        let value = match value {
            PropertyValue::Boolean(value) => value.to_string(),
            PropertyValue::Integer(value) => value.to_string(),
            PropertyValue::Float(value) => value.to_string(),
            PropertyValue::String(value) => escape_value(value),
        };
        output.push_str(&value);
        output.push('\n');
    }
    output
}

/// Parses one complete Java-properties document without returning partial data.
pub fn parse_server_properties(source: &str) -> Result<BTreeMap<String, String>, PropertiesError> {
    let mut properties = BTreeMap::new();
    let mut logical = String::new();
    let mut logical_start = 1;
    let mut continuing = false;

    for (index, physical) in source.lines().enumerate() {
        let line_number = index + 1;
        let line = physical.strip_suffix('\r').unwrap_or(physical);
        if !continuing {
            logical.clear();
            logical_start = line_number;
        }
        let segment = if continuing {
            line.trim_start_matches([' ', '\t', '\u{000c}'])
        } else {
            line
        };
        logical.push_str(segment);
        if has_odd_trailing_backslashes(&logical) {
            logical.pop();
            continuing = true;
            continue;
        }
        continuing = false;
        parse_logical_line(&logical, logical_start, &mut properties)?;
    }

    if continuing {
        return Err(PropertiesError::Malformed {
            line: logical_start,
            reason: "unterminated line continuation".to_owned(),
        });
    }
    Ok(properties)
}

/// Checks both managed text paths before atomically creating either missing file.
pub fn materialize_text_files(
    server_root: &Path,
    properties: &BTreeMap<String, PropertyValue>,
    eula: bool,
) -> Result<(), ManagedFileError> {
    preflight_text_files(server_root, properties, eula)?;
    materialize_managed_bytes(
        server_root,
        "server.properties",
        render_server_properties(properties).as_bytes(),
    )?;
    materialize_managed_bytes(server_root, "eula.txt", format!("eula={eula}\n").as_bytes())?;
    Ok(())
}

/// Checks both managed text paths without creating or replacing either one.
pub fn preflight_text_files(
    server_root: &Path,
    properties: &BTreeMap<String, PropertyValue>,
    eula: bool,
) -> Result<(), ManagedFileError> {
    let root = fs::canonicalize(server_root).map_err(|source| ManagedFileError::ResolveRoot {
        path: server_root.to_owned(),
        source,
    })?;
    let contents = [
        (
            "server.properties",
            render_server_properties(properties).into_bytes(),
        ),
        ("eula.txt", format!("eula={eula}\n").into_bytes()),
    ];

    for (name, expected) in &contents {
        inspect_managed(&root, name, expected)?;
    }
    Ok(())
}

/// Result of safely comparing a managed path with expected bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ManagedFileState {
    /// The managed path does not exist.
    Missing,
    /// The managed path is a safe regular file with identical content.
    Identical,
}

/// Checks and, when absent, atomically creates one managed file from exact bytes.
pub fn materialize_managed_bytes(
    server_root: &Path,
    name: &'static str,
    expected: &[u8],
) -> Result<ManagedFileState, ManagedFileError> {
    let root = fs::canonicalize(server_root).map_err(|source| ManagedFileError::ResolveRoot {
        path: server_root.to_owned(),
        source,
    })?;
    let state = inspect_managed(&root, name, expected)?;
    if state == ManagedFileState::Missing {
        let path = resolve_creation_path(Path::new(name), &root).map_err(|source| {
            ManagedFileError::Inspect {
                name,
                source: io::Error::other(source),
            }
        })?;
        write_bytes(&path, expected, WriteMode::Create)
            .map_err(|source| ManagedFileError::Publish { name, source })?;
    }
    Ok(state)
}

/// Reads one required regular current-user-owned managed file without following a symlink.
pub fn read_managed_bytes(
    server_root: &Path,
    name: &'static str,
) -> Result<Vec<u8>, ManagedFileError> {
    let root = fs::canonicalize(server_root).map_err(|source| ManagedFileError::ResolveRoot {
        path: server_root.to_owned(),
        source,
    })?;
    let path = root.join(name);
    let metadata = match fs::symlink_metadata(&path) {
        Ok(metadata) => metadata,
        Err(source) if source.kind() == io::ErrorKind::NotFound => {
            return Err(ManagedFileError::Missing { name });
        }
        Err(source) => return Err(ManagedFileError::Inspect { name, source }),
    };
    if !metadata.file_type().is_file() {
        return Err(ManagedFileError::NotRegular { name });
    }
    let expected_uid = geteuid().as_raw();
    if metadata.uid() != expected_uid {
        return Err(ManagedFileError::ForeignOwner {
            name,
            actual: metadata.uid(),
            expected: expected_uid,
        });
    }
    let descriptor = openat(
        CWD,
        &path,
        OFlags::RDONLY | OFlags::CLOEXEC | OFlags::NOFOLLOW,
        Mode::empty(),
    )
    .map_err(|source| ManagedFileError::Read {
        name,
        source: source.into(),
    })?;
    let mut file = File::from(descriptor);
    let opened = file
        .metadata()
        .map_err(|source| ManagedFileError::Read { name, source })?;
    let final_entry =
        fs::symlink_metadata(&path).map_err(|_| ManagedFileError::ChangedDuringRead { name })?;
    if !stable_regular_identity(&metadata, &opened, &final_entry) {
        return Err(ManagedFileError::ChangedDuringRead { name });
    }
    let mut contents = Vec::new();
    file.read_to_end(&mut contents)
        .map_err(|source| ManagedFileError::Read { name, source })?;
    let final_entry =
        fs::symlink_metadata(&path).map_err(|_| ManagedFileError::ChangedDuringRead { name })?;
    if !stable_regular_identity(&metadata, &opened, &final_entry) {
        return Err(ManagedFileError::ChangedDuringRead { name });
    }
    Ok(contents)
}

/// Reads an existing managed path without following a final symlink and compares its bytes.
fn inspect_managed(
    root: &Path,
    name: &'static str,
    expected: &[u8],
) -> Result<ManagedFileState, ManagedFileError> {
    let path = root.join(name);
    let metadata = match fs::symlink_metadata(&path) {
        Ok(metadata) => metadata,
        Err(source) if source.kind() == io::ErrorKind::NotFound => {
            return Ok(ManagedFileState::Missing);
        }
        Err(source) => return Err(ManagedFileError::Inspect { name, source }),
    };
    if !metadata.file_type().is_file() {
        return Err(ManagedFileError::NotRegular { name });
    }
    let expected_uid = geteuid().as_raw();
    if metadata.uid() != expected_uid {
        return Err(ManagedFileError::ForeignOwner {
            name,
            actual: metadata.uid(),
            expected: expected_uid,
        });
    }
    let descriptor = openat(
        CWD,
        &path,
        OFlags::RDONLY | OFlags::CLOEXEC | OFlags::NOFOLLOW,
        Mode::empty(),
    )
    .map_err(|source| ManagedFileError::Read {
        name,
        source: source.into(),
    })?;
    let mut file = File::from(descriptor);
    let opened = file
        .metadata()
        .map_err(|source| ManagedFileError::Read { name, source })?;
    let final_entry =
        fs::symlink_metadata(&path).map_err(|_| ManagedFileError::ChangedDuringRead { name })?;
    if !stable_regular_identity(&metadata, &opened, &final_entry) {
        return Err(ManagedFileError::ChangedDuringRead { name });
    }
    let mut actual = Vec::new();
    file.read_to_end(&mut actual)
        .map_err(|source| ManagedFileError::Read { name, source })?;
    let final_entry =
        fs::symlink_metadata(&path).map_err(|_| ManagedFileError::ChangedDuringRead { name })?;
    if !stable_regular_identity(&metadata, &opened, &final_entry) {
        return Err(ManagedFileError::ChangedDuringRead { name });
    }
    if actual == expected {
        Ok(ManagedFileState::Identical)
    } else {
        Err(ManagedFileError::Differs { name })
    }
}

/// Requires every observation to identify one regular file with unchanged ownership.
fn stable_regular_identity(
    inspected: &fs::Metadata,
    opened: &fs::Metadata,
    final_entry: &fs::Metadata,
) -> bool {
    [inspected, opened, final_entry]
        .iter()
        .all(|metadata| metadata.file_type().is_file())
        && [opened, final_entry].iter().all(|metadata| {
            metadata.uid() == inspected.uid()
                && metadata.dev() == inspected.dev()
                && metadata.ino() == inspected.ino()
        })
}

/// Parses one completed logical property line into the result map.
fn parse_logical_line(
    line: &str,
    line_number: usize,
    properties: &mut BTreeMap<String, String>,
) -> Result<(), PropertiesError> {
    let line = line.trim_start_matches([' ', '\t', '\u{000c}']);
    if line.is_empty() || line.starts_with('#') || line.starts_with('!') {
        return Ok(());
    }
    let mut escaped = false;
    let mut separator = None;
    for (index, character) in line.char_indices() {
        if escaped {
            escaped = false;
        } else if character == '\\' {
            escaped = true;
        } else if character == '=' || character == ':' || character.is_ascii_whitespace() {
            separator = Some(index);
            break;
        }
    }
    let split = separator.unwrap_or(line.len());
    let raw_key = &line[..split];
    let mut value_start = split;
    while let Some(character) = line[value_start..].chars().next() {
        if character.is_ascii_whitespace() {
            value_start += character.len_utf8();
        } else {
            break;
        }
    }
    if matches!(line[value_start..].chars().next(), Some('=' | ':')) {
        value_start += 1;
    }
    while let Some(character) = line[value_start..].chars().next() {
        if character.is_ascii_whitespace() {
            value_start += character.len_utf8();
        } else {
            break;
        }
    }
    let key = decode_escapes(raw_key, line_number)?;
    if key.is_empty() {
        return Err(PropertiesError::Malformed {
            line: line_number,
            reason: "property key is empty".to_owned(),
        });
    }
    let value = decode_escapes(&line[value_start..], line_number)?;
    if properties.insert(key.clone(), value).is_some() {
        return Err(PropertiesError::Duplicate {
            key,
            line: line_number,
        });
    }
    Ok(())
}

/// Decodes Java-properties escape sequences in a key or value.
fn decode_escapes(source: &str, line: usize) -> Result<String, PropertiesError> {
    let mut output = String::new();
    let mut characters = source.chars();
    while let Some(character) = characters.next() {
        if character != '\\' {
            output.push(character);
            continue;
        }
        let escaped = characters
            .next()
            .ok_or_else(|| PropertiesError::Malformed {
                line,
                reason: "trailing escape".to_owned(),
            })?;
        match escaped {
            't' => output.push('\t'),
            'n' => output.push('\n'),
            'r' => output.push('\r'),
            'f' => output.push('\u{000c}'),
            'u' => {
                let digits = characters.by_ref().take(4).collect::<String>();
                if digits.len() != 4 || !digits.chars().all(|digit| digit.is_ascii_hexdigit()) {
                    return Err(PropertiesError::Malformed {
                        line,
                        reason: "invalid Unicode escape".to_owned(),
                    });
                }
                let value = u32::from_str_radix(&digits, 16).expect("validated hexadecimal");
                let decoded = char::from_u32(value).ok_or_else(|| PropertiesError::Malformed {
                    line,
                    reason: "invalid Unicode scalar".to_owned(),
                })?;
                output.push(decoded);
            }
            other => output.push(other),
        }
    }
    Ok(output)
}

/// Escapes a property key for deterministic serialization.
fn escape_key(value: &str) -> String {
    escape(value, true)
}

/// Escapes a property value for deterministic serialization.
fn escape_value(value: &str) -> String {
    escape(value, false)
}

/// Escapes the Java-properties characters that are significant in the selected position.
fn escape(value: &str, key: bool) -> String {
    let mut output = String::new();
    for (index, character) in value.chars().enumerate() {
        match character {
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            '\u{000c}' => output.push_str("\\f"),
            ' ' if key || index == 0 => output.push_str("\\ "),
            '=' | ':' | '#' | '!' if key => {
                output.push('\\');
                output.push(character);
            }
            other => output.push(other),
        }
    }
    output
}

/// Reports whether a physical line ends with an unescaped continuation marker.
fn has_odd_trailing_backslashes(value: &str) -> bool {
    value
        .chars()
        .rev()
        .take_while(|character| *character == '\\')
        .count()
        % 2
        == 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn managed_identity_rejects_a_different_opened_or_final_path_entry() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let first = directory.path().join("first");
        let second = directory.path().join("second");
        fs::write(&first, b"same bytes").expect("first fixture");
        fs::write(&second, b"same bytes").expect("second fixture");
        let first = fs::metadata(first).expect("first metadata");
        let second = fs::metadata(second).expect("second metadata");

        assert!(stable_regular_identity(&first, &first, &first));
        assert!(!stable_regular_identity(&first, &second, &first));
        assert!(!stable_regular_identity(&first, &first, &second));
    }
}
