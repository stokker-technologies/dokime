/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports

// Third Party Imports
use thiserror::Error as ThisError;
// Local Imports
use dokime_domain::model::diagnostics::ValidationIssue;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */
pub type Result<T> = std::result::Result<T, Error>;

/* === Structs === */

/* === Enums === */
#[derive(Debug, ThisError)]
pub enum Error {
    #[error("{message}")]
    Validation {
        code: String,
        message: String,
        issues: Vec<ValidationIssue>,
    },
    #[error("{message}")]
    Conflict { code: String, message: String },
    #[error("{message}")]
    Overload { code: String, message: String },
    #[error("{message}")]
    Unavailable { code: String, message: String },
    #[error("{message}")]
    Forbidden { code: String, message: String },
    #[error("{message}")]
    NotFound { code: String, message: String },
    #[error("{message}")]
    Timeout { code: String, message: String },
    #[error("{message}")]
    Cancelled { code: String, message: String },
    #[error("{message}")]
    Internal { code: String, message: String },

    #[error(transparent)]
    Dokime(#[from] crate::Error),

    #[error("{}", harmos_message(.0))]
    Harmos(#[from] harmos::Error),

    #[error("sampling capture could not settle: {0}")]
    SamplingSettlement(#[source] Box<dyn std::error::Error + Send + Sync>),

    #[error(transparent)]
    Storage(#[from] crate::storage::Error),

    #[error(transparent)]
    Refused(#[from] harmos::Refusal),

    #[error("export storage failed: {source}")]
    ExportStorage {
        #[from]
        source: crate::exports::storage::StorageError,
    },

    #[error("{0}")]
    LocalState(#[from] crate::storage::sqlite::Error),

    #[error("execution request receipt: {0}")]
    RequestReceipt(#[source] Box<dyn std::error::Error + Send + Sync>),

    #[error("native runtime bootstrap failed: {source}")]
    RuntimeBoot {
        #[source]
        source: Box<harmos::runtime::Incompatible>,
    },

    #[error("runtime task failed")]
    Join {
        #[from]
        source: tokio::task::JoinError,
    },

    #[error("runtime initialization failed: {source}; cleanup failed: {cleanup}")]
    StartupCleanup {
        #[source]
        source: Box<Error>,
        cleanup: Box<Error>,
        rollback: crate::runtime::rollback::Receipt,
    },

    #[error("runtime shutdown could not begin: {source}")]
    ShutdownStart {
        #[source]
        source: harmos::Error<String>,
    },

    #[error("provider request `{request_id}` terminal publication failed: {source}")]
    ProviderCompletion {
        request_id: dokime_domain::model::identity::CommandId,
        #[source]
        source: std::sync::Arc<Error>,
    },

    #[error("runtime is shutting down")]
    ShuttingDown,

    #[error("runtime operation was interrupted")]
    Interrupted,

    #[error("execution output exceeds native content limit")]
    ExecutionFileTooLarge,

    #[error("invalid checkpoint report inputs: {0}")]
    InvalidReportInputs(String),

    #[error("invalid checkpoint query: {0}")]
    InvalidCheckpointQuery(String),

    #[error("Plugin {plugin} could not initialize\n  Cause: {reason}")]
    PluginInitialization { plugin: String, reason: String },

    #[error("{0}")]
    Operation(String),
}

/* === Traits === */

/* === Implementations === */
impl From<dokime_filesystem::Error> for Error {
    fn from(error: dokime_filesystem::Error) -> Self {
        std::io::Error::from(error).into()
    }
}

impl Error {
    pub fn code(&self) -> Option<&str> {
        match self {
            Self::Validation { code, .. }
            | Self::Conflict { code, .. }
            | Self::Overload { code, .. }
            | Self::Unavailable { code, .. }
            | Self::Forbidden { code, .. }
            | Self::NotFound { code, .. }
            | Self::Timeout { code, .. }
            | Self::Cancelled { code, .. }
            | Self::Internal { code, .. } => Some(code),
            _ => None,
        }
    }

    pub fn validation(
        code: impl Into<String>,
        message: impl Into<String>,
        issues: Vec<ValidationIssue>,
    ) -> Self {
        Self::Validation {
            code: code.into(),
            message: message.into(),
            issues,
        }
    }

    pub fn conflict(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Conflict {
            code: code.into(),
            message: message.into(),
        }
    }

    pub fn overload(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Overload {
            code: code.into(),
            message: message.into(),
        }
    }

    pub fn unavailable(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Unavailable {
            code: code.into(),
            message: message.into(),
        }
    }

    pub fn forbidden(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Forbidden {
            code: code.into(),
            message: message.into(),
        }
    }

    pub fn not_found(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::NotFound {
            code: code.into(),
            message: message.into(),
        }
    }

    pub fn timeout(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Timeout {
            code: code.into(),
            message: message.into(),
        }
    }

    pub fn cancelled(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Cancelled {
            code: code.into(),
            message: message.into(),
        }
    }

    pub fn internal(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Internal {
            code: code.into(),
            message: message.into(),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::ExportStorage {
            source: error.into(),
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn harmos_message(error: &harmos::Error) -> String {
    let mut message = error.to_string();
    // Harmos prints an operation failure on its own line; Dokime messages carry the cause.
    if let harmos::Error::OperationFailed { source, .. } = error {
        use std::fmt::Write;
        let _ = write!(message, ": {source}");
        return message;
    }
    if let harmos::Error::PluginLifecycle { source } = error {
        let failures = match source.as_ref() {
            harmos::plugin::PluginFailure::Cleanup(failures)
            | harmos::plugin::PluginFailure::Start {
                cleanup: failures, ..
            } => failures,
            _ => return message,
        };
        for failure in failures {
            use std::fmt::Write;
            let _ = write!(
                message,
                "; plugin `{}`: {}",
                failure.artifact, failure.error
            );
        }
    }
    message
}

/* === Tests === */
