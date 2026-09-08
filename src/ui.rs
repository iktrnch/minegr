//! Shared terminal presentation policy.

use std::env;

use console::Style;
use dialoguer::{Confirm, FuzzySelect, Input, Select, theme::ColorfulTheme};
use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use thiserror::Error;

/// Terminal presentation choices derived once for a command invocation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Presentation {
    /// Whether progress may use terminal animation and persistent styling.
    pub pretty: bool,
    /// Whether semantic output may use color.
    pub color: bool,
    /// Whether an interactive prompt is permitted.
    pub interactive: bool,
}

impl Presentation {
    /// Derives presentation from terminal capabilities and process variables.
    pub fn detect(
        stdin_is_terminal: bool,
        stderr_is_terminal: bool,
        ci: Option<&str>,
        no_color: bool,
    ) -> Self {
        let in_ci = ci.is_some_and(is_ci_value);
        Self {
            pretty: stderr_is_terminal && !in_ci,
            color: stderr_is_terminal && !in_ci && !no_color,
            interactive: stdin_is_terminal && stderr_is_terminal && !in_ci,
        }
    }
}

/// Errors produced by shared prompt handling.
#[derive(Debug, Error)]
pub enum UiError {
    /// The command needs input but prompting is not allowed in this environment.
    #[error("Interactive input is unavailable; provide the required command flags")]
    InteractiveUnavailable,
    /// Dialoguer could not complete a terminal prompt.
    #[error("Failed to read interactive input: {0}")]
    Prompt(#[from] dialoguer::Error),
}

/// Shared prompt, progress, warning, success, and error presentation.
#[derive(Debug)]
pub struct Ui {
    presentation: Presentation,
}

impl Ui {
    /// Detects presentation settings for the current process.
    pub fn from_process(stdin_is_terminal: bool, stderr_is_terminal: bool) -> Self {
        let ci = env::var("CI").ok();
        Self {
            presentation: Presentation::detect(
                stdin_is_terminal,
                stderr_is_terminal,
                ci.as_deref(),
                env::var_os("NO_COLOR").is_some(),
            ),
        }
    }

    /// Returns the presentation settings selected for this invocation.
    pub fn presentation(&self) -> Presentation {
        self.presentation
    }

    /// Writes one warning to stderr.
    pub fn warning(&self, message: &str) {
        write_semantic(message, Style::new().yellow(), self.presentation.color);
    }

    /// Writes one error to stderr.
    pub fn error(&self, message: &str) {
        write_semantic(message, Style::new().red(), self.presentation.color);
    }

    /// Writes one success message to stderr.
    pub fn success(&self, message: &str) {
        write_semantic(message, Style::new().green(), self.presentation.color);
    }

    /// Starts a progress operation using a spinner only when stderr supports it.
    pub fn progress(&self, message: &str) -> Progress {
        if self.presentation.pretty {
            let bar = ProgressBar::new_spinner();
            bar.set_draw_target(ProgressDrawTarget::stderr());
            bar.set_style(
                ProgressStyle::with_template("{spinner} {msg}")
                    .expect("the static progress template is valid"),
            );
            bar.set_message(message.to_owned());
            bar.enable_steady_tick(std::time::Duration::from_millis(100));
            Progress::Pretty(bar)
        } else {
            eprintln!("{message}");
            Progress::Plain
        }
    }

    /// Asks for confirmation through the repository-approved prompt library.
    pub fn confirm(&self, prompt: &str, default: bool) -> Result<bool, UiError> {
        if !self.presentation.interactive {
            return Err(UiError::InteractiveUnavailable);
        }

        Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(prompt)
            .default(default)
            .interact()
            .map_err(UiError::from)
    }

    /// Asks for one non-empty text value with a displayed default.
    pub fn input(&self, prompt: &str, default: &str) -> Result<String, UiError> {
        if !self.presentation.interactive {
            return Err(UiError::InteractiveUnavailable);
        }
        Input::with_theme(&ColorfulTheme::default())
            .with_prompt(prompt)
            .default(default.to_owned())
            .interact_text()
            .map_err(UiError::from)
    }

    /// Asks for one searchable item while showing at most twelve rows.
    pub fn fuzzy_select(&self, prompt: &str, items: &[String]) -> Result<usize, UiError> {
        if !self.presentation.interactive {
            return Err(UiError::InteractiveUnavailable);
        }
        FuzzySelect::with_theme(&ColorfulTheme::default())
            .with_prompt(prompt)
            .items(items)
            .max_length(12)
            .interact()
            .map_err(UiError::from)
    }

    /// Asks for one item from a short fixed list.
    pub fn select(&self, prompt: &str, items: &[String]) -> Result<usize, UiError> {
        if !self.presentation.interactive {
            return Err(UiError::InteractiveUnavailable);
        }
        Select::with_theme(&ColorfulTheme::default())
            .with_prompt(prompt)
            .items(items)
            .interact()
            .map_err(UiError::from)
    }

    /// Writes an unstyled informational block to stderr.
    pub fn message(&self, message: &str) {
        eprintln!("{message}");
    }
}

/// A shared progress operation that renders exactly once in plain mode.
#[derive(Debug)]
pub enum Progress {
    /// An interactive Indicatif spinner.
    Pretty(ProgressBar),
    /// A message already written once to plain stderr.
    Plain,
}

impl Progress {
    /// Completes progress and leaves one final message visible.
    pub fn finish(self, message: &str) {
        match self {
            Self::Pretty(bar) => bar.finish_with_message(message.to_owned()),
            Self::Plain => eprintln!("{message}"),
        }
    }
}

fn write_semantic(message: &str, style: Style, color: bool) {
    if color {
        eprintln!("{}", style.apply_to(message));
    } else {
        eprintln!("{message}");
    }
}

fn is_ci_value(value: &str) -> bool {
    ["1", "true", "yes"]
        .iter()
        .any(|candidate| value.eq_ignore_ascii_case(candidate))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ci_values_disable_pretty_and_interactive_output_case_insensitively() {
        for value in ["1", "true", "TRUE", "yes", "YeS"] {
            let presentation = Presentation::detect(true, true, Some(value), false);
            assert_eq!(
                presentation,
                Presentation {
                    pretty: false,
                    color: false,
                    interactive: false,
                }
            );
        }
    }

    #[test]
    fn no_color_preserves_other_terminal_presentation() {
        let presentation = Presentation::detect(true, true, None, true);

        assert!(presentation.pretty);
        assert!(!presentation.color);
        assert!(presentation.interactive);
    }

    #[test]
    fn prompts_require_both_terminal_streams() {
        assert!(!Presentation::detect(false, true, None, false).interactive);
        assert!(!Presentation::detect(true, false, None, false).interactive);
    }
}
