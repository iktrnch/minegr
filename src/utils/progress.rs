use std::time::Duration;

use anyhow::Context;
use indicatif::{ProgressBar, ProgressFinish, ProgressState, ProgressStyle};

/// Creates a byte progress display with a spinner and right-aligned transfer statistics.
/// Unknown totals show transferred bytes and speed without an ETA, bar, or percentage.
pub fn progress_bar(
    step: &'static str,
    total_bytes: impl Into<Option<u64>>,
) -> anyhow::Result<ProgressBar> {
    let (progress, template) = match total_bytes.into() {
        Some(total) => (
            ProgressBar::new(total),
            "{spinner:.blue} {msg:24} {binary_bytes:>10.bold} {binary_bytes_per_sec:>12} {eta:>5.dim} [{wide_bar:.cyan/white.dim}] {percent:>3.cyan}%",
        ),
        None => (
            ProgressBar::new_spinner(),
            "{spinner:.blue} {msg:24} {binary_bytes:>10.bold} {binary_bytes_per_sec:>12}",
        ),
    };
    let style = ProgressStyle::with_template(template)
        .context("Failed to configure the progress display")?
        .progress_chars("##-")
        .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏ ")
        .with_key(
            "eta",
            |state: &ProgressState, output: &mut dyn std::fmt::Write| {
                let seconds = state.eta().as_secs();
                let _ = write!(output, "{:02}:{:02}", seconds / 60, seconds % 60);
            },
        );
    let progress = progress.with_style(style).with_message(step).with_finish(
        ProgressFinish::AbandonWithMessage(format!("{step} failed").into()),
    );
    progress.tick();
    progress.enable_steady_tick(Duration::from_millis(100));
    Ok(progress)
}
