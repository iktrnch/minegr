pub mod repository;

use std::fs::{self, File};
use std::io::{Read, Seek};
use std::path::{Component, Path};
use std::time::Duration;

use anyhow::Context;
use indicatif::{ProgressBar, ProgressFinish, ProgressStyle};
use sha2::{Digest, Sha512};

use crate::config::{Addon, Config};
use crate::platform::Platform;
use crate::utils::{http::ApiClient, progress::progress_bar};
use repository::{ReleaseChannel, Version, VersionFile};

/// Installs the newest compatible Modrinth addon and returns its pinned config record.
/// The channel is a ceiling: beta permits releases, and alpha permits every channel.
/// Existing files are never replaced; configuration is saved by the caller.
pub fn get_addon(
    addons_dir: &Path,
    config: &Config,
    slug: &str,
    channel: Option<ReleaseChannel>,
) -> anyhow::Result<Addon> {
    let channel = channel.unwrap_or(ReleaseChannel::Alpha);
    let loaders: &[&str] = match &config.minecraft.platform {
        Platform::Fabric => &["fabric"],
        Platform::Paper => &["paper", "spigot", "bukkit"],
        Platform::Vanilla => anyhow::bail!("Vanilla does not support addons; use Paper or Fabric"),
    };
    anyhow::ensure!(
        !slug.trim().is_empty(),
        "The Modrinth project slug cannot be empty"
    );

    let client = ApiClient::new().context("Failed to prepare the Modrinth HTTP client")?;
    let versions = fetch_versions(&client, slug)?;
    let version = versions
        .into_iter()
        .filter(|version| {
            version.game_versions.contains(&config.minecraft.version)
                && version
                    .loaders
                    .iter()
                    .any(|loader| loaders.contains(&loader.as_str()))
                && channel.allows(version.version_type)
        })
        // Sort by date published, then by ID
        .max_by(|left, right| {
            left.date_published
                .cmp(&right.date_published)
                .then_with(|| left.id.cmp(&right.id))
        })
        .with_context(|| {
            format!(
                "No compatible version of {slug} for Minecraft {} on {} with channel {channel}",
                config.minecraft.version, config.minecraft.platform
            )
        })?;

    let mut files = version.files;
    anyhow::ensure!(
        !files.is_empty(),
        "Modrinth version {} has no downloadable files",
        version.id
    );
    let selected = files.iter().position(|file| file.primary).unwrap_or(0);
    let file = files.swap_remove(selected);
    validate_filename(&file.filename)?;
    anyhow::ensure!(
        file.hashes.sha512.len() == 128
            && file
                .hashes
                .sha512
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit()),
        "Modrinth returned an invalid SHA-512 for {}",
        file.filename
    );
    anyhow::ensure!(
        !config
            .addons
            .iter()
            .any(|addon| addon.version_id == version.id || addon.name == file.filename),
        "{} is already recorded in minegr.toml; refusing to add it twice",
        file.filename
    );

    fs::create_dir_all(addons_dir).with_context(|| {
        format!(
            "Failed to create the addon directory {}",
            addons_dir.display()
        )
    })?;
    let destination = addons_dir.join(&file.filename);
    match fs::symlink_metadata(&destination) {
        Ok(_) => anyhow::bail!(
            "{} already exists; refusing to overwrite it",
            destination.display()
        ),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(error)
                .with_context(|| format!("Failed to inspect {}", destination.display()));
        }
    }

    eprintln!(
        "  Selected {} ({}, {})",
        file.filename, version.id, version.version_type
    );
    let mut download = download_file(&client, &file)?;
    verify_file(&mut download, &file)?;
    install_file(&mut download, &file, addons_dir)?;

    Ok(Addon {
        name: file.filename,
        version_id: version.id,
        hash: format!("sha512:{}", file.hashes.sha512.to_ascii_lowercase()),
        version_type: channel,
    })
}

/// Fetches version metadata without changelogs while displaying resolution progress.
fn fetch_versions(client: &reqwest::blocking::Client, slug: &str) -> anyhow::Result<Vec<Version>> {
    let progress = ProgressBar::new_spinner()
        .with_style(
            ProgressStyle::with_template("{spinner:.green} {msg}")
                .context("Failed to configure Modrinth metadata progress")?,
        )
        .with_message(format!("Finding versions of {slug}"))
        .with_finish(ProgressFinish::AbandonWithMessage(
            "Version lookup failed".into(),
        ));
    progress.enable_steady_tick(Duration::from_millis(100));

    let mut url = reqwest::Url::parse("https://api.modrinth.com/v2/project/")
        .context("Failed to construct the Modrinth project URL")?;
    url.path_segments_mut()
        .map_err(|()| anyhow::anyhow!("The Modrinth URL cannot contain project path segments"))?
        .pop_if_empty()
        .push(slug)
        .push("version");
    url.query_pairs_mut()
        .append_pair("include_changelog", "false");
    let versions: Vec<Version> = client
        .get(url)
        .send()
        .with_context(|| format!("Failed to fetch Modrinth versions for {slug}"))?
        .error_for_status()
        .with_context(|| format!("Modrinth's version request for {slug} failed"))?
        .json()
        .with_context(|| format!("Failed to parse Modrinth versions for {slug}"))?;
    progress.finish_with_message(format!("Found {} versions of {slug}", versions.len()));
    Ok(versions)
}

/// Ensures an upstream filename names one JAR directly inside the addon directory.
fn validate_filename(filename: &str) -> anyhow::Result<()> {
    let path = Path::new(filename);
    let mut components = path.components();
    anyhow::ensure!(
        matches!(components.next(), Some(Component::Normal(_)))
            && components.next().is_none()
            && !filename.contains(['/', '\\', '\0'])
            && !filename.chars().any(char::is_control)
            && path
                .extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| extension.eq_ignore_ascii_case("jar")),
        "Modrinth returned an invalid addon JAR filename: {filename:?}"
    );
    Ok(())
}

/// Streams the addon into private temporary storage and checks its advertised size.
fn download_file(client: &reqwest::blocking::Client, file: &VersionFile) -> anyhow::Result<File> {
    let mut download =
        tempfile::tempfile().context("Failed to create a private addon download file")?;
    let mut response = client
        .get(&file.url)
        .timeout(Duration::from_secs(300))
        .send()
        .with_context(|| format!("Failed to download {}", file.filename))?
        .error_for_status()
        .with_context(|| format!("The download request for {} failed", file.filename))?;
    let progress = progress_bar("Downloading", file.size)?;
    let downloaded = response
        .copy_to(&mut progress.wrap_write(&mut download))
        .with_context(|| format!("Failed to write the download of {}", file.filename))?;
    anyhow::ensure!(
        downloaded == file.size,
        "{} size mismatch: expected {} bytes, downloaded {downloaded}",
        file.filename,
        file.size
    );
    progress.finish_with_message("Downloaded");
    Ok(download)
}

/// Verifies the downloaded bytes against Modrinth's SHA-512 before installation.
fn verify_file(download: &mut File, file: &VersionFile) -> anyhow::Result<()> {
    download
        .rewind()
        .context("Failed to rewind the addon download for verification")?;
    let progress = progress_bar("Hashing", file.size)?;
    let mut hasher = Sha512::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = download
            .read(&mut buffer)
            .context("Failed to read the addon for verification")?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
        progress.inc(count as u64);
    }
    let actual: String = hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    anyhow::ensure!(
        actual.eq_ignore_ascii_case(&file.hashes.sha512),
        "{} SHA-512 mismatch: expected {}, downloaded {actual}",
        file.filename,
        file.hashes.sha512
    );
    progress.finish_with_message("Hash verified");
    Ok(())
}

/// Copies a verified addon beside its destination, then publishes without overwriting files.
fn install_file(download: &mut File, file: &VersionFile, addons_dir: &Path) -> anyhow::Result<()> {
    download
        .rewind()
        .context("Failed to rewind the verified addon for installation")?;
    let mut staged = tempfile::NamedTempFile::new_in(addons_dir)
        .context("Failed to create a temporary file in the addon directory")?;
    let progress = progress_bar("Copying", file.size)?;
    let copied = std::io::copy(download, &mut progress.wrap_write(staged.as_file_mut()))
        .context("Failed to stage the verified addon")?;
    anyhow::ensure!(
        copied == file.size,
        "The staged addon size does not match the verified download"
    );
    staged
        .as_file()
        .sync_all()
        .context("Failed to synchronize the staged addon")?;
    let destination = addons_dir.join(&file.filename);
    staged
        .persist_noclobber(&destination)
        .map_err(|error| error.error)
        .with_context(|| {
            format!(
                "Failed to install {} without overwriting an existing file",
                destination.display()
            )
        })?;
    File::open(addons_dir)
        .context("Failed to open the addon directory for synchronization")?
        .sync_all()
        .context("The addon was installed, but synchronizing its directory failed")?;
    progress.finish_with_message("Copied");
    Ok(())
}
