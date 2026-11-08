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
    #[error(transparent)]
    Query(#[from] Box<dokime_domain::model::query::error::Failure>),

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

    #[error("{}", report_save_message(.subject, .stage, .source))]
    ReportSave {
        subject: dokime_domain::model::reports::wire::ReportSubject,
        stage: &'static str,

        #[source]
        source: Box<Error>,
    },

    #[error("invalid checkpoint report inputs: {0}")]
    InvalidReportInputs(String),

    #[error("{0}")]
    ReportFinalizationFailed(String),

    #[error(
        "The report could not be created from saved evidence. Open Settings → Diagnostics → Support Bundle and send it to the plugin developer."
    )]
    ReportUnavailable(#[source] Box<Error>),

    #[error("invalid checkpoint query: {0}")]
    InvalidCheckpointQuery(String),

    #[error("Plugin {plugin} could not initialize\n  Cause: {reason}")]
    PluginInitialization { plugin: String, reason: String },

    #[error("embedded plugin `{plugin}` conflicts with an installed plugin")]
    PluginIdConflict { plugin: String },

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
    /// Returns the recovery action supplied by the evidence or finalization owner.
    pub fn report_evidence_message(&self) -> Option<String> {
        match self {
            Self::Storage(
                error @ (crate::storage::Error::TelemetryIntegrity { .. } | crate::storage::Error::TelemetryProjection { .. }),
            ) => Some(error.to_string()),
            Self::ReportFinalizationFailed(message) => Some(message.clone()),
            Self::ReportUnavailable(source) | Self::ReportSave { source, .. } => source.report_evidence_message(),
            _ => None,
        }
    }

    pub fn code(&self) -> Option<&str> {
        match self {
            Self::Admission(failure) => Some(failure.code()),
            Self::ReportSave { .. } => Some("report_not_saved"),
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

/// The error with every cause its own message does not already state, for durable failure records.
pub(crate) fn chain_message(error: &(dyn std::error::Error + 'static)) -> String {
    let mut message = error.to_string();
    for cause in std::iter::successors(error.source(), |cause| cause.source()) {
        let cause = cause.to_string();
        if !message.contains(&cause) {
            message.push_str(": ");
            message.push_str(&cause);
        }
    }
    message
}

/* === Private Functions === */

fn report_save_message(subject: &dokime_domain::model::reports::wire::ReportSubject, stage: &str, source: &Error) -> String {
    if let Some(message) = source.report_evidence_message() {
        return format!("Report for {} could not be saved during {stage}: {message}", report_subject(subject));
    }
    format!("The report could not be saved during {stage} for {}. Retry sealing the cycle to save it again.", report_subject(subject))
}

fn report_subject(subject: &dokime_domain::model::reports::wire::ReportSubject) -> String {
    match subject {
        dokime_domain::model::reports::wire::ReportSubject::Execution(id) => format!("execution `{id}`"),
        dokime_domain::model::reports::wire::ReportSubject::Cycle(id) => format!("cycle `{id}`"),
    }
}

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
