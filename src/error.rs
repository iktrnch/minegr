//! Stable application error categories.

use thiserror::Error;

/// An error returned by command dispatch.
#[derive(Debug, Error)]
pub enum AppError {
    /// A command has parsed successfully but belongs to a later milestone.
    #[error("Command `{command}` is not implemented yet")]
    Unimplemented { command: &'static str },
    /// A usage or configuration error.
    #[error("{0}")]
    Usage(String),
    /// A requested server or daemon is unavailable.
    #[error("{0}")]
    Unavailable(String),
    /// An accepted operation failed.
    #[error("{0}")]
    Operation(String),
    /// Foreground work was interrupted and cancelled.
    #[error("{0}")]
    Interrupted(String),
}

impl AppError {
    /// Creates the temporary explicit error used for a later-milestone command.
    pub fn unimplemented(command: &'static str) -> Self {
        Self::Unimplemented { command }
    }

    /// Returns the documented stable process exit code.
    pub fn exit_code(&self) -> u8 {
        match self {
            Self::Usage(_) => 2,
            Self::Unavailable(_) => 3,
            Self::Unimplemented { .. } | Self::Operation(_) => 4,
            Self::Interrupted(_) => 130,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn application_errors_use_stable_exit_categories() {
        assert_eq!(AppError::Usage(String::new()).exit_code(), 2);
        assert_eq!(AppError::Unavailable(String::new()).exit_code(), 3);
        assert_eq!(AppError::Operation(String::new()).exit_code(), 4);
        assert_eq!(AppError::Interrupted(String::new()).exit_code(), 130);
    }
}
