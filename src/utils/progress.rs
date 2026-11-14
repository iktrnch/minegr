use anyhow::Context;
use indicatif::{ProgressBar, ProgressFinish, ProgressStyle};

/// Creates byte progress with a bar for a known total or a counter for an unknown total.
pub fn progress_bar(
    step: &'static str,
    total_bytes: impl Into<Option<u64>>,
) -> anyhow::Result<ProgressBar> {
    let (progress, template) = match total_bytes.into() {
        Some(total) => (
            ProgressBar::new(total),
            "{spinner:.green} {msg:14} {bar} {bytes}/{total_bytes} ({bytes_per_sec}, ETA {eta})",
        ),
        None => (
            ProgressBar::new_spinner(),
            "{spinner:.green} {msg:14} {bytes} ({bytes_per_sec})",
        ),
    };
    let style = ProgressStyle::with_template(template)
        .context("Failed to configure the server JAR progress bar")?;
    let progress = progress
        .with_style(style.progress_chars("█▒░"))
        .with_message(step)
        .with_finish(ProgressFinish::AbandonWithMessage(
            format!("{step} failed").into(),
        ));
    progress.tick();
    Ok(progress)
}
