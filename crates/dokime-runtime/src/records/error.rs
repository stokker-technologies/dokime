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
use serde::{Deserialize, Serialize};
// Local Imports
use dokime_domain::model::diagnostics::StructuredErrorDetail;
use crate::records::RecordSeverity;

/* === Constants === */

/* === Macros === */
product_error!(RecordFailure, formatter {
    Self::UnknownRecordType { record_type } => write!(formatter, "record type {record_type:?} is not registered"),
    Self::SchemaVersionMismatch { record_type, expected, actual } => write!(formatter, "record {record_type:?} expects schema version {expected}, got {actual}"),
    Self::MissingField { record_type, field } => write!(formatter, "record {record_type:?} requires field {field:?}"),
    Self::InvalidField { record_type, field } => write!(formatter, "record {record_type:?} field {field:?} does not match its declared schema"),
    Self::InvalidChoice { record_type, field } => write!(formatter, "record {record_type:?} field {field:?} must be one of the declared choices"),
    Self::UnexpectedFields { record_type, fields } => write!(formatter, "record {record_type:?} does not declare fields: {fields:?}"),
    Self::SeverityTooLow { record_type, minimum } => write!(formatter, "record {record_type:?} cannot lower severity below {minimum:?}"),
    Self::InvalidWindow => formatter.write_str("record window requires records in sequence order"),
    Self::IncompatibleContract(message) => write!(formatter, "record contract compatibility failed: {message}"),
    Self::InvalidProductState { field, value } => write!(formatter, "product record field {field:?} has invalid state {value:?}"),
});

/* === Type Aliases === */
pub type Result<T> = crate::Result<T>;

/* === Structs === */
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordErrorDetail {
    pub message: String,
    pub code: Option<String>,
    pub source: Option<String>,
    pub cause_chain: Vec<dokime_domain::model::diagnostics::ErrorCause>,
    pub stacktrace_attachment: Option<dokime_domain::model::identity::ArtifactId>,
    pub trace_id: Option<dokime_domain::model::identity::TraceId>,
    pub correlation_id: Option<dokime_domain::model::identity::CorrelationId>,
}

/* === Enums === */

#[derive(Debug, PartialEq)]
pub enum RecordFailure {
    UnknownRecordType {
        record_type: String,
    },
    SchemaVersionMismatch {
        record_type: String,
        expected: u32,
        actual: u32,
    },
    MissingField {
        record_type: String,
        field: String,
    },
    InvalidField {
        record_type: String,
        field: String,
    },
    InvalidChoice {
        record_type: String,
        field: String,
    },
    UnexpectedFields {
        record_type: String,
        fields: Vec<String>,
    },
    SeverityTooLow {
        record_type: String,
        minimum: RecordSeverity,
    },
    InvalidWindow,
    IncompatibleContract(String),
    InvalidProductState {
        field: String,
        value: String,
    },
}

/* === Traits === */

/* === Implementations === */
impl harmos::MemorySize for RecordErrorDetail {
    fn memory_size(&self) -> usize {
        let fields = [
            (
                harmos::MemorySize::memory_size(&self.message),
                std::mem::size_of_val(&self.message),
            ),
            (
                harmos::MemorySize::memory_size(&self.code),
                std::mem::size_of_val(&self.code),
            ),
            (
                harmos::MemorySize::memory_size(&self.source),
                std::mem::size_of_val(&self.source),
            ),
            (
                harmos::MemorySize::memory_size(&self.cause_chain),
                std::mem::size_of_val(&self.cause_chain),
            ),
            (
                harmos::MemorySize::memory_size(&self.stacktrace_attachment),
                std::mem::size_of_val(&self.stacktrace_attachment),
            ),
            (
                harmos::MemorySize::memory_size(&self.trace_id),
                std::mem::size_of_val(&self.trace_id),
            ),
            (
                harmos::MemorySize::memory_size(&self.correlation_id),
                std::mem::size_of_val(&self.correlation_id),
            ),
        ];
        fields
            .into_iter()
            .fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
                if owned == usize::MAX {
                    usize::MAX
                } else {
                    bytes.saturating_add(owned.saturating_sub(inline))
                }
            })
    }
}

impl RecordErrorDetail {
    pub fn new(message: impl Into<String>) -> Self {
        Self::from(StructuredErrorDetail::new(message))
    }
}

impl From<StructuredErrorDetail> for RecordErrorDetail {
    fn from(detail: StructuredErrorDetail) -> Self {
        Self {
            message: detail.message,
            code: detail.code,
            source: detail.source,
            cause_chain: detail.cause_chain,
            stacktrace_attachment: detail.stacktrace_attachment,
            trace_id: detail.trace_id,
            correlation_id: detail.correlation_id,
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
