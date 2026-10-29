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
use std::fmt;

// Third Party Imports
#[cfg(feature = "native")]
use tokio_stream::Elapsed;

// Local Imports
use crate::document::DocumentFailure;
use crate::values::FieldError;
use dokime_domain::model::identity::Identifier;
use dokime_domain::model::records::error::RecordFailure;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

pub type Result<T, E = Error> = std::result::Result<T, E>;

/* === Structs === */

/* === Enums === */

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Api(#[source] dokime_domain::Error),

    #[error("setting {0:?} is not available in this context")]
    UnknownSetting(String),

    #[error("setting {0:?} requires secret() access")]
    SecretSetting(String),

    #[error("setting {0:?} does not have the requested type")]
    SettingType(String),

    #[error("secret {0:?} is not declared in this context")]
    UnknownSecret(String),

    #[error("secret {0:?} has no available binding")]
    UnavailableSecret(String),

    #[error("parameter {name:?}: {reason}")]
    Parameter { name: String, reason: String },

    #[error("the run was cancelled while the question was outstanding")]
    PromptCancelled,

    #[error("the operator question was refused: {0}")]
    PromptRefused(String),

    #[error("the stream could not be leased: {0}")]
    LeaseRefused(String),

    #[error("the execution ended before the stream lease was granted")]
    LeaseCancelled,

    #[error(transparent)]
    RecordDetail(#[from] dokime_domain::model::records::error::RecordDetailError),

    #[error("record {record:?}: {reason}")]
    RecordDecode { record: String, reason: String },

    #[error("{0}")]
    Publish(PublishFailure),

    #[error("{0}")]
    Telemetry(TelemetryFailure),

    #[error("{0}")]
    Document(#[source] DocumentFailure),

    #[error("{0}")]
    Record(#[source] RecordFailure),

    #[cfg(feature = "native")]
    #[error("the deadline elapsed before the stream produced an item")]
    Deadline(#[source] Elapsed),

    #[error("evidence file: {0}")]
    Artifact(#[source] std::io::Error),

    #[error("operation {operation:?} did not answer: {reason}")]
    Call { operation: &'static str, reason: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PublishFailure {
    Undeclared { declared_as: String },
    Oversized { declared_as: String, encoded: usize, limit: usize },
    Closed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TelemetryFailure {
    UnknownStream { stream: String },
    SchemaVersionMismatch { stream: String, expected: u32, actual: u32 },
    MissingColumn { stream: String, column: String },
    InvalidColumn { stream: String, column: String },
    UnexpectedColumns { stream: String, columns: Vec<String> },
    IncompatibleContract(String),
}

/* === Traits === */

/* === Implementations === */

impl Error {
    pub fn record_mismatched(wanted: &'static str, found: &Identifier) -> Self {
        Self::RecordDecode {
            record: found.to_string(),
            reason: format!("is not a {wanted:?}"),
        }
    }

    pub fn record_field(record_type: &'static str, field: FieldError) -> Self {
        Self::RecordDecode {
            record: record_type.to_owned(),
            reason: field.to_string(),
        }
    }
}

impl fmt::Display for PublishFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Undeclared { declared_as } => write!(
                f,
                "this plugin declared no {declared_as:?}: include it in the sidecar records or streams catalog, or in a declared observer catalog"
            ),
            Self::Oversized { declared_as, encoded, limit } => {
                write!(f, "one {declared_as:?} publication needs {encoded} bytes; the transport frame limit is {limit}")
            }
            Self::Closed => f.write_str("the host is no longer listening to this plugin"),
        }
    }
}

impl fmt::Display for TelemetryFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownStream { stream } => {
                write!(f, "telemetry stream {stream:?} is not registered")
            }
            Self::SchemaVersionMismatch { stream, expected, actual } => {
                write!(f, "telemetry stream {stream:?} expects schema version {expected}, got {actual}")
            }
            Self::MissingColumn { stream, column } => {
                write!(f, "telemetry stream {stream:?} requires column {column:?}")
            }
            Self::InvalidColumn { stream, column } => {
                write!(f, "telemetry stream {stream:?} column {column:?} does not match its declared schema")
            }
            Self::UnexpectedColumns { stream, columns } => write!(f, "telemetry stream {stream:?} does not declare columns: {columns:?}"),
            Self::IncompatibleContract(message) => {
                write!(f, "telemetry contract compatibility failed: {message}")
            }
        }
    }
}

impl From<DocumentFailure> for Error {
    fn from(source: DocumentFailure) -> Self {
        Self::Document(source)
    }
}

impl From<RecordFailure> for Error {
    fn from(source: RecordFailure) -> Self {
        Self::Record(source)
    }
}

impl From<harmos::IdentifierError> for Error {
    fn from(source: harmos::IdentifierError) -> Self {
        Self::Api(source.into())
    }
}

impl From<dokime_domain::Error> for Error {
    fn from(source: dokime_domain::Error) -> Self {
        Self::Api(source)
    }
}

#[cfg(feature = "native")]
impl From<Elapsed> for Error {
    fn from(source: Elapsed) -> Self {
        Self::Deadline(source)
    }
}

impl From<std::io::Error> for Error {
    fn from(source: std::io::Error) -> Self {
        Self::Artifact(source)
    }
}

impl From<dokime_domain::model::execution::snapshot::SnapshotError> for Error {
    fn from(source: dokime_domain::model::execution::snapshot::SnapshotError) -> Self {
        Self::Parameter { name: source.name, reason: source.reason }
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
