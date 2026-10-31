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
use dokime_domain::model::diagnostics::admission::AdmissionFailure;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

pub type Result<T> = std::result::Result<T, Error>;

/* === Structs === */

/* === Enums === */

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("interface lifecycle failed: {source}")]
    Hosting {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error(transparent)]
    Admission(#[from] AdmissionFailure),

    #[error(transparent)]
    Dokime(#[from] crate::Error),

    #[error("{}", harmos_message(.0))]
    Harmos(#[from] harmos::Error),

    #[error(transparent)]
    HarmosStorage(#[from] harmos::storage::Error),

    #[error(transparent)]
    HarmosDomain(#[from] harmos::domain::Error),

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

    #[error("execution request receipt: {0}")]
    RequestReceipt(#[source] Box<dyn std::error::Error + Send + Sync>),

    #[error("native runtime bootstrap failed: {source}")]
    RuntimeBoot {
        #[source]
        source: Box<harmos::StartupError>,
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

    #[error("invalid checkpoint report inputs: {0}")]
    InvalidReportInputs(String),

    #[error("The report could not be created from saved evidence. Try again; if the problem continues, collect diagnostics.")]
    ReportUnavailable(#[source] Box<Error>),

    #[error("invalid checkpoint query: {0}")]
    InvalidCheckpointQuery(String),

    #[error("Plugin {plugin} could not initialize\n  Cause: {reason}")]
    PluginInitialization { plugin: String, reason: String },

    #[error("{}", super::plugins::start_error(plugin, plugin, source))]
    PluginStart {
        plugin: String,

        #[source]
        source: Box<harmos::Error>,
    },

    #[error("{0}")]
    Operation(String),
}

/* === Traits === */

/* === Implementations === */

// A Missing Or Duplicated Registration Resource Is A Harmos Registration Failure.
impl From<harmos::ResourceError> for Error {
    fn from(error: harmos::ResourceError) -> Self {
        Self::Harmos(error.into())
    }
}

impl From<harmos::filesystem::Error> for Error {
    fn from(error: harmos::filesystem::Error) -> Self {
        std::io::Error::from(error).into()
    }
}

impl Error {
    pub fn code(&self) -> Option<&str> {
        match self {
            Self::Admission(failure) => Some(failure.code()),
            _ => None,
        }
    }

    pub fn validation(code: impl Into<String>, message: impl Into<String>, issues: Vec<ValidationIssue>) -> Self {
        AdmissionFailure::validation(code, message, issues).into()
    }

    pub fn conflict(code: impl Into<String>, message: impl Into<String>) -> Self {
        AdmissionFailure::conflict(code, message).into()
    }

    pub fn overload(code: impl Into<String>, message: impl Into<String>) -> Self {
        AdmissionFailure::overload(code, message).into()
    }

    pub fn unavailable(code: impl Into<String>, message: impl Into<String>) -> Self {
        AdmissionFailure::unavailable(code, message).into()
    }

    pub fn forbidden(code: impl Into<String>, message: impl Into<String>) -> Self {
        AdmissionFailure::forbidden(code, message).into()
    }

    pub fn not_found(code: impl Into<String>, message: impl Into<String>) -> Self {
        AdmissionFailure::not_found(code, message).into()
    }

    pub fn timeout(code: impl Into<String>, message: impl Into<String>) -> Self {
        AdmissionFailure::timeout(code, message).into()
    }

    pub fn cancelled(code: impl Into<String>, message: impl Into<String>) -> Self {
        AdmissionFailure::cancelled(code, message).into()
    }

    pub fn internal(code: impl Into<String>, message: impl Into<String>) -> Self {
        AdmissionFailure::internal(code, message).into()
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::ExportStorage { source: error.into() }
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn harmos_message(error: &harmos::Error) -> String {
    use std::fmt::Write as _;

    let mut message = error.to_string();
    // Harmos prints an operation failure on its own line; Dokime messages carry the cause.
    if let harmos::Error::OperationFailed { source, .. } = error {
        use std::fmt::Write;
        let _ = write!(message, ": {source}");
        return message;
    }
    if let harmos::Error::PluginLifecycle { source } = error {
        let failures = match source.as_ref() {
            harmos::plugin::PluginFailure::Cleanup(failures) => failures,
            harmos::plugin::PluginFailure::Start { artifact, source, cleanup } => {
                let _ = write!(message, "; plugin `{artifact}`: {}", harmos::domain::failure::Report(source));
                cleanup
            }
            _ => return message,
        };
        for failure in failures {
            let _ = write!(message, "; plugin `{}`: {}", failure.artifact, failure.error);
        }
    }
    message
}

/* === Tests === */
