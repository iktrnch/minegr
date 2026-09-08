//! Shared Minegr CLI foundations.

pub mod cli;
pub mod config;
pub mod config_path;
pub mod config_write;
pub mod error;
pub mod ui;
pub mod validation;

use std::env;

use cli::Cli;
use cli::Command;
use config::load_config;
use config_path::{ConfigPathError, resolve_creation_path, resolve_existing_path};
use error::AppError;
use ui::Ui;
use validation::{
    ConfigFileObservation, FileKind, HostValidationInput, Observation, validate_config,
    validate_host,
};

/// Dispatches one parsed CLI invocation.
pub fn dispatch(cli: Cli, ui: &Ui) -> Result<(), AppError> {
    let current_directory = env::current_dir().map_err(|error| {
        AppError::Usage(format!("Failed to resolve current directory: {error}"))
    })?;
    let resolved = match &cli.command {
        Command::Init(arguments) if !arguments.uuid => {
            resolve_creation_path(&cli.config, &current_directory)
        }
        _ => resolve_existing_path(&cli.config, &current_directory),
    }
    .map_err(|error| path_error(&cli, error))?;

    if !matches!(cli.command, Command::Init(ref arguments) if !arguments.uuid)
        || resolved.as_path().exists()
    {
        let loaded = load_config(&resolved).map_err(|error| {
            AppError::Usage(format!(
                "Failed to load configuration {}: {error}",
                resolved.as_str()
            ))
        })?;
        let mut report = validate_config(&loaded.config);
        let host_report = validate_host(
            &HostValidationInput {
                config_file: ConfigFileObservation {
                    kind: Observation::Known(FileKind::Regular),
                    owner_uid: Observation::Known(loaded.owner_uid),
                    expected_uid: loaded.owner_uid,
                    mode: Observation::Known(loaded.mode),
                },
                checks: Vec::new(),
            },
            &report,
        );
        for warning in host_report.warnings() {
            ui.warning(&warning.message);
        }
        report.append(host_report);
        if report.has_errors() {
            let errors = report
                .errors()
                .map(|finding| finding.message.as_str())
                .collect::<Vec<_>>()
                .join("; ");
            return Err(AppError::Usage(format!(
                "Invalid configuration {}: {errors}",
                resolved.as_str()
            )));
        }
    }

    Err(AppError::unimplemented(cli.command.name()))
}

fn path_error(cli: &Cli, error: ConfigPathError) -> AppError {
    match error {
        ConfigPathError::InvalidUtf8 { path } => AppError::Usage(format!(
            "Configuration path is not valid UTF-8: {}",
            path.display()
        )),
        other => AppError::Usage(format!(
            "Failed to load configuration {}: {other}",
            cli.config.display()
        )),
    }
}
