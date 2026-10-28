/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use tonic::Status;

use crate::grpc::value::{Budget, canonical};
use dokime::domain::model::identity::wire::Severity;
use dokime::domain::model::records::evidence::RecordEntry;
use dokime::records::RecordSeverity;

pub(crate) fn record(entry: &RecordEntry) -> Result<dokime::domain::model::evidence::wire::CapturedRecord, Status> {
    let record = &entry.record;
    let details = canonical::object(&record.details, &mut Budget::new()?)?;
    let error = record
        .error
        .as_ref()
        .map(|error| {
            let mut cause_chain = crate::grpc::value::projection::slots(error.cause_chain.len()).map_err(allocation_error)?;
            for cause in &error.cause_chain {
                cause_chain.push(dokime::domain::model::evidence::wire::CapturedErrorCause {
                    message: text(&cause.message)?,
                    code: optional(cause.code.as_deref())?,
                    source: optional(cause.source.as_deref())?,
                });
            }
            Ok::<_, Status>(dokime::domain::model::evidence::wire::CapturedRecordError {
                message: text(&error.message)?,
                code: optional(error.code.as_deref())?,
                source: optional(error.source.as_deref())?,
                cause_chain,
                stacktrace_attachment: optional(error.stacktrace_attachment.as_ref().map(|id| id.as_str()))?,
                trace_id: optional(error.trace_id.as_ref().map(|id| id.as_str()))?,
                correlation_id: optional(error.correlation_id.as_ref().map(|id| id.as_str()))?,
            })
        })
        .transpose()?;
    Ok(dokime::domain::model::evidence::wire::CapturedRecord {
        record_id: text(record.record_id.as_str())?,
        sequence: entry.sequence,
        timestamp: Some(canonical::timestamp(&record.timestamp)?),
        received_at: Some(canonical::timestamp(&record.received_at)?),
        record_type: text(record.record_type.as_str())?,
        schema_version: record.schema_version,
        source: text(&record.source)?,
        severity: severity(record.severity),
        message: optional(record.message.as_deref())?,
        error,
        details: Some(details),
        scope: record.scope.as_ref().map(canonical::scope).transpose()?,
        related_sample_id: optional(record.related_sample_id.as_ref().map(|id| id.as_str()))?,
        related_stream_id: optional(record.related_stream_id.as_ref().map(|id| id.as_str()))?,
        related_artifact_id: optional(record.related_artifact_id.as_ref().map(|id| id.as_str()))?,
    })
}

pub(crate) fn severity(value: RecordSeverity) -> Severity {
    match value {
        RecordSeverity::Debug => Severity::Debug,
        RecordSeverity::Info => Severity::Info,
        RecordSeverity::Warning => Severity::Warn,
        RecordSeverity::Error => Severity::Error,
    }
}

fn text(value: &str) -> Result<String, Status> {
    crate::grpc::value::projection::text(value).map_err(allocation_error)
}

fn optional(value: Option<&str>) -> Result<Option<String>, Status> {
    value.map(text).transpose()
}

fn allocation_error(source: dokime::harmos::Error) -> Status {
    Status::resource_exhausted(source.to_string())
}
