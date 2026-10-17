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
use tonic::Status;
// Local Imports
use crate::grpc::value::{self, Budget, canonical};
use dokime_domain::model::records::evidence::RecordEntry;
use dokime_runtime::records::RecordSeverity;
use dokime_domain::model::identity::wire::Severity;
use dokime_domain::operations::evidence as wire;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(in crate::grpc) fn memory(entry: &RecordEntry) -> Result<usize, Status> {
    let record = &entry.record;
    let mut bytes = std::mem::size_of::<dokime_domain::model::evidence::wire::CapturedRecord>()
        .saturating_add(record.record_id.as_str().len())
        .saturating_add(record.record_type.as_str().len())
        .saturating_add(record.source.len())
        .saturating_add(record.message.as_ref().map_or(0, String::len))
        .saturating_add(canonical::object_memory(
            &record.details,
            &mut Budget::new(),
        )?);
    if let Some(scope) = &record.scope {
        bytes = bytes.saturating_add(canonical::scope_memory(scope)?);
    }
    for id in [
        record.related_sample_id.as_ref().map(|id| id.as_str()),
        record.related_stream_id.as_ref().map(|id| id.as_str()),
        record.related_artifact_id.as_ref().map(|id| id.as_str()),
    ]
    .into_iter()
    .flatten()
    {
        bytes = bytes.saturating_add(id.len());
    }
    if let Some(error) = &record.error {
        value::collection(error.cause_chain.len())?;
        bytes = bytes.saturating_add(error.message.len());
        for text in [error.code.as_ref(), error.source.as_ref()]
            .into_iter()
            .flatten()
        {
            bytes = bytes.saturating_add(text.len());
        }
        bytes = bytes.saturating_add(
            error
                .cause_chain
                .len()
                .saturating_mul(4 * std::mem::size_of::<dokime_domain::model::evidence::wire::CapturedErrorCause>()),
        );
        for cause in &error.cause_chain {
            bytes = bytes.saturating_add(cause.message.len());
            for text in [cause.code.as_ref(), cause.source.as_ref()]
                .into_iter()
                .flatten()
            {
                bytes = bytes.saturating_add(text.len());
            }
        }
        for id in [
            error.stacktrace_attachment.as_ref().map(|id| id.as_str()),
            error.trace_id.as_ref().map(|id| id.as_str()),
            error.correlation_id.as_ref().map(|id| id.as_str()),
        ]
        .into_iter()
        .flatten()
        {
            bytes = bytes.saturating_add(id.len());
        }
    }
    if bytes == usize::MAX {
        return Err(Status::resource_exhausted(
            "record construction size overflow",
        ));
    }
    Ok(bytes)
}

pub(crate) fn record(entry: &RecordEntry) -> Result<dokime_domain::model::evidence::wire::CapturedRecord, Status> {
    let record = &entry.record;
    memory(entry)?;
    let details = canonical::object(&record.details, &mut Budget::new())?;
    let error = record
        .error
        .as_ref()
        .map(|error| {
            value::collection(error.cause_chain.len())?;
            Ok::<_, Status>(dokime_domain::model::evidence::wire::CapturedRecordError {
                message: error.message.clone(),
                code: error.code.clone(),
                source: error.source.clone(),
                cause_chain: error
                    .cause_chain
                    .iter()
                    .map(|cause| dokime_domain::model::evidence::wire::CapturedErrorCause {
                        message: cause.message.clone(),
                        code: cause.code.clone(),
                        source: cause.source.clone(),
                    })
                    .collect(),
                stacktrace_attachment: error
                    .stacktrace_attachment
                    .as_ref()
                    .map(ToString::to_string),
                trace_id: error.trace_id.as_ref().map(ToString::to_string),
                correlation_id: error.correlation_id.as_ref().map(ToString::to_string),
            })
        })
        .transpose()?;
    Ok(dokime_domain::model::evidence::wire::CapturedRecord {
        record_id: record.record_id.to_string(),
        sequence: entry.sequence,
        timestamp: Some(canonical::timestamp(&record.timestamp)?),
        received_at: Some(canonical::timestamp(&record.received_at)?),
        record_type: record.record_type.to_string(),
        schema_version: record.schema_version,
        source: record.source.clone(),
        severity: severity(record.severity),
        message: record.message.clone(),
        error,
        details: Some(details),
        scope: record.scope.as_ref().map(canonical::scope).transpose()?,
        related_sample_id: record.related_sample_id.as_ref().map(ToString::to_string),
        related_stream_id: record.related_stream_id.as_ref().map(ToString::to_string),
        related_artifact_id: record.related_artifact_id.as_ref().map(ToString::to_string),
    })
}

/* === Private Functions === */
pub(crate) fn severity(value: RecordSeverity) -> Severity {
    match value {
        RecordSeverity::Debug => Severity::Debug,
        RecordSeverity::Info => Severity::Info,
        RecordSeverity::Warning => Severity::Warn,
        RecordSeverity::Error => Severity::Error,
    }
}

/* === Tests === */
