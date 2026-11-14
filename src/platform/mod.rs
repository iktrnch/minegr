pub mod properties;
mod vendor;

use anyhow::Context;
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use sha2::Sha256;
use std::fs::File;
use std::io::{Read, Seek};
use std::path::Path;

use crate::utils::progress::progress_bar;

#[derive(Debug, strum::Display, strum::EnumString, strum::VariantNames, Deserialize, Serialize)]
#[strum(serialize_all = "lowercase")]
pub enum Platform {
    Paper,
    Fabric,
    Vanilla,
}

/// implements [PlatformProvider] for each platform variant.
impl Platform {
    /// Selects a version using this platform's provider.
    pub fn select_version(&self) -> anyhow::Result<Box<Vec<String>>> {
        match self {
            Platform::Paper => Paper::select_version(),
            Platform::Fabric => Fabric::select_version(),
            Platform::Vanilla => Vanilla::select_version(),
        }
    }

    /// Downloads a core, verifies any upstream checksum, and installs `config_dir/server.jar`
    /// without replacing an existing file.
    /// Returns the checksum if the provider supplies one.
    pub fn get_core(&self, version: &str, config_dir: &Path) -> anyhow::Result<String> {
        let destination = config_dir.join("server.jar");
        match std::fs::symlink_metadata(&destination) {
            Ok(_) => anyhow::bail!("server.jar already exists; refusing to overwrite it"),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error).context("Failed to inspect server.jar"),
        }

        let mut core = match self {
            Platform::Paper => Paper::get_core(version),
            Platform::Fabric => Fabric::get_core(version),
            Platform::Vanilla => Vanilla::get_core(version),
        }
        .with_context(|| format!("Failed to download the {self} server core"))?;

        if let Some(checksum) = &core.checksum {
            core.file
                .rewind()
                .context("Failed to rewind the downloaded server JAR for verification")?;

            checksum
                .compute(
                    self.to_string(),
                    core.size,
                    core.file
                        .try_clone()
                        .context("Failed to clone the server JAR handle for verification")?,
                )
                .with_context(|| format!("Failed to verify the {self} server JAR"))?;
        }

        core.file
            .rewind()
            .context("Failed to rewind the verified server JAR for installation")?;

        let mut staged = tempfile::NamedTempFile::new_in(config_dir)
            .context("Failed to create a temporary file beside server.jar")?;
        let copy_progress = progress_bar("Copying", core.size)?;
        std::io::copy(
            &mut core.file,
            &mut copy_progress.wrap_write(staged.as_file_mut()),
        )
        .context("Failed to stage the verified server JAR beside server.jar")?;
        staged
            .as_file()
            .sync_all()
            .context("Failed to synchronize the staged server JAR")?;
        staged
            .persist_noclobber(&destination)
            .map_err(|error| error.error)
            .context("Failed to publish server.jar without overwriting an existing file")?;
        File::open(config_dir)
            .context("Failed to open the server directory for synchronization")?
            .sync_all()
            .context("server.jar was installed, but synchronizing its directory failed")?;
        copy_progress.finish_with_message("Copied");

        let checksum_string = match core.checksum {
            Some(CoreChecksum::Sha1(checksum)) => checksum,
            Some(CoreChecksum::Sha256(checksum)) => checksum,
            None => String::new(),
        };
        Ok(checksum_string)
    }
}

/// A downloaded server JAR and the upstream metadata needed to verify it.
pub struct CoreDownload {
    /// An owned private temporary file; closing it discards the download.
    pub file: File,
    /// Downloaded byte count, checked against the upstream size when available.
    pub size: u64,
    /// Expected upstream checksum and its algorithm, when the provider supplies one.
    pub checksum: Option<CoreChecksum>,
}

impl CoreDownload {
    /// Streams a server JAR into private storage, checking an advertised byte count if available.
    fn download(
        client: &reqwest::blocking::Client,
        url: &str,
        size: Option<u64>,
        checksum: Option<CoreChecksum>,
    ) -> anyhow::Result<Self> {
        let mut file = tempfile::tempfile()
            .context("Failed to create a private temporary file for the server download")?;
        let mut response = client
            .get(url)
            .timeout(std::time::Duration::from_secs(300))
            .send()
            .context("Failed to download the server JAR")?
            .error_for_status()
            .context("Server JAR download request failed")?;
        let expected_size = size.or(response.content_length());
        let download_progress = progress_bar("Downloading", expected_size)?;
        let downloaded_bytes = response
            .copy_to(&mut download_progress.wrap_write(&mut file))
            .context("Failed to write the server JAR download")?;
        if let Some(expected_size) = expected_size {
            anyhow::ensure!(
                downloaded_bytes == expected_size,
                "Server JAR size mismatch: expected {expected_size}, downloaded {downloaded_bytes}"
            );
        }
        download_progress.finish_with_message("Downloaded");

        Ok(Self {
            file,
            size: downloaded_bytes,
            checksum,
        })
    }
}

/// An upstream server JAR checksum encoded as hexadecimal text.
pub enum CoreChecksum {
    /// SHA-1 checksum supplied by Mojang.
    Sha1(String),
    /// SHA-256 checksum supplied by Paper.
    Sha256(String),
}

impl CoreChecksum {
    fn compute(&self, platform: String, size: u64, mut file: File) -> anyhow::Result<()> {
        let hash_progress = progress_bar("Hashing", size)?;
        let (algorithm, expected, checksum) = match self {
            CoreChecksum::Sha1(expected) => (
                "SHA-1",
                expected,
                hash_core::<Sha1>(&mut file, &hash_progress)
                    .context("Failed to calculate the server JAR SHA-1")?,
            ),
            CoreChecksum::Sha256(expected) => (
                "SHA-256",
                expected,
                hash_core::<Sha256>(&mut file, &hash_progress)
                    .context("Failed to calculate the server JAR SHA-256")?,
            ),
        };
        anyhow::ensure!(
            checksum.eq_ignore_ascii_case(expected),
            "{platform} server JAR {algorithm} mismatch: expected {expected}, downloaded {checksum}"
        );
        hash_progress.finish_with_message("Hash verified");

        Ok(())
    }
}

/// Hashes a downloaded JAR with the selected algorithm and updates byte progress.
fn hash_core<D: Digest>(
    file: &mut File,
    progress: &indicatif::ProgressBar,
) -> anyhow::Result<String> {
    let mut hasher = D::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = file
            .read(&mut buffer)
            .context("Failed to read the downloaded server JAR for verification")?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
        progress.inc(count as u64);
    }
    Ok(hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}

pub trait PlatformProvider {
    /// Fetches platform versions and selects one with a five-row fuzzy picker.
    fn select_version() -> anyhow::Result<Box<Vec<String>>>;
    /// Downloads the requested core into a temporary file with any available upstream checksum.
    /// `Platform::get_core` verifies a supplied checksum and installs the file as `server.jar`.
    fn get_core(version: &str) -> anyhow::Result<CoreDownload>;
}

pub struct Paper;
pub struct Fabric;
pub struct Vanilla;
