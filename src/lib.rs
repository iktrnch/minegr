//! Shared Minegr CLI foundations.

pub mod artifact;
pub mod cli;
pub mod config;
pub mod config_path;
pub mod config_write;
pub mod error;
pub mod init;
pub mod managed_files;
pub mod sync;
pub mod ui;
pub mod validation;

use std::env;

use cli::Cli;
use cli::Command;
use config::load_config;
use config_path::{ConfigPathError, resolve_creation_path, resolve_existing_path};
use error::AppError;
use init::{
    SystemInitHostValidator, SystemJavaRuntime, SystemRunningInstanceProbe, run_existing_init,
    run_new_init,
};
use sync::{SystemStoppedServerProbe, run_sync};
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

    if let Command::Init(arguments) = &cli.command
        && !arguments.uuid
        && !resolved.as_path().exists()
    {
        let artifacts = artifact::ArtifactService::official()
            .map_err(|error| AppError::Operation(error.to_string()))?;
        let output = run_new_init(
            arguments,
            &resolved,
            ui,
            &artifacts,
            &SystemJavaRuntime,
            &SystemInitHostValidator,
        )
        .map_err(|error| {
            if error.is_usage() {
                AppError::Usage(error.to_string())
            } else {
                AppError::Operation(error.to_string())
            }
        })?;
        print!("{}", output.stdout());
        return Ok(());
    }

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
        let checks = if matches!(cli.command, Command::Init(ref arguments) if !arguments.uuid) {
            let root = resolved
                .as_path()
                .parent()
                .expect("a resolved configuration path always has a parent");
            SystemInitHostValidator.host_checks(root, &loaded.config)
        } else {
            Vec::new()
        };
        let host_report = validate_host(
            &HostValidationInput {
                config_file: ConfigFileObservation {
                    kind: Observation::Known(FileKind::Regular),
                    owner_uid: Observation::Known(loaded.owner_uid),
                    expected_uid: loaded.owner_uid,
                    mode: Observation::Known(loaded.mode),
                },
                checks,
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

        if let Command::Init(arguments) = &cli.command {
            let artifacts = artifact::ArtifactService::official()
                .map_err(|error| AppError::Operation(error.to_string()))?;
            let output = run_existing_init(
                arguments,
                &resolved,
                &loaded,
                &artifacts,
                &SystemJavaRuntime,
                &SystemRunningInstanceProbe,
                &SystemInitHostValidator,
            )
            .map_err(|error| {
                if error.is_usage() {
                    AppError::Usage(error.to_string())
                } else {
                    AppError::Operation(error.to_string())
                }
            })?;
            print!("{}", output.stdout());
            return Ok(());
        }

        if matches!(cli.command, Command::Sync) {
            let output =
                run_sync(&resolved, &loaded, &SystemStoppedServerProbe).map_err(|error| {
                    if error.is_unavailable() {
                        AppError::Unavailable(error.to_string())
                    } else if error.is_usage() {
                        AppError::Usage(error.to_string())
                    } else {
                        AppError::Operation(error.to_string())
                    }
                })?;
            print!("{}", output.stdout());
            return Ok(());
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
