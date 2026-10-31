/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Captured Record Projection

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::super::selection::{Envelope, FieldRef, Selection};
use dokime_domain::model::{
    evidence::wire::{CapturedErrorCause, CapturedRecord, CapturedRecordError},
    execution::{EvidenceOrigin, RecordScope},
    identity::wire::{EvidenceOrigin as WireOrigin, EvidenceOriginKind, EvidenceScope, Severity, Timestamp},
    query::error::{Failure, Reason},
    records::{RecordSeverity, evidence::RecordEntry},
    values::{Value, wire},
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) fn matches(selection: &Selection, record: &RecordEntry) -> Result<bool, Box<Failure>> {
    let timestamp = record.timestamp.to_datetime().map_err(|_| corrupt())?;
    let received = record.received_at.to_datetime().map_err(|_| corrupt())?;
    let id = Value::Text(record.record_id.to_string());
    let at = Value::Datetime(timestamp);
    let received = Value::Datetime(received);
    let source = Value::Text(record.source.clone());
    let version = Value::Unsigned(record.schema_version.into());
    let severity = Value::Text(format!("{:?}", severity(record.severity)));
    Ok(selection.matches(Some(Timestamp::from(timestamp)), |field| match field {
        FieldRef::Declared(id) => record.details.get(id),
        FieldRef::Envelope(envelope) => Some(match envelope {
            Envelope::Id => &id,
            Envelope::Timestamp => &at,
            Envelope::ReceivedAt => &received,
            Envelope::Source => &source,
            Envelope::SchemaVersion => &version,
            Envelope::Severity => &severity,
        }),
    }))
}

pub(super) fn record(entry: &RecordEntry) -> Result<CapturedRecord, Box<Failure>> {
    let details = wire::Value::try_from(&entry.details).map_err(|_| {
        Failure::new(Reason::ResourceLimit, "dokime.query.list_records", "Record details exceed supported wire capacity or depth")
    })?;
    let error = entry.error.as_ref().map(|error| CapturedRecordError {
        message: error.message.clone(),
        code: error.code.clone(),
        source: error.source.clone(),
        cause_chain: error
            .cause_chain
            .iter()
            .map(|cause| CapturedErrorCause {
                message: cause.message.clone(),
                code: cause.code.clone(),
                source: cause.source.clone(),
            })
            .collect(),
        stacktrace_attachment: error.stacktrace_attachment.as_ref().map(ToString::to_string),
        trace_id: error.trace_id.as_ref().map(ToString::to_string),
        correlation_id: error.correlation_id.as_ref().map(ToString::to_string),
        diagnostics: None,
    });
    Ok(CapturedRecord {
        record_id: entry.record_id.to_string(),
        sequence: entry.sequence,
        timestamp: Some(Timestamp::from(entry.timestamp.to_datetime().map_err(|_| corrupt())?)),
        received_at: Some(Timestamp::from(entry.received_at.to_datetime().map_err(|_| corrupt())?)),
        record_type: entry.record_type.to_string(),
        schema_version: entry.schema_version,
        source: entry.source.clone(),
        severity: severity(entry.severity),
        message: entry.message.clone(),
        error,
        details: Some(details),
        scope: entry.scope.as_ref().map(scope),
        related_sample_id: entry.related_sample_id.as_ref().map(ToString::to_string),
        related_stream_id: entry.related_stream_id.as_ref().map(ToString::to_string),
        related_artifact_id: entry.related_artifact_id.as_ref().map(ToString::to_string),
    })
}

/* === Private Functions === */

fn scope(scope: &RecordScope) -> EvidenceScope {
    let kind = match &scope.origin {
        EvidenceOrigin::Default => EvidenceOriginKind::Default(true),
        EvidenceOrigin::Initialization => EvidenceOriginKind::Initialization(true),
        EvidenceOrigin::Step { attempt_id } => EvidenceOriginKind::StepAttemptId(attempt_id.to_string()),
        EvidenceOrigin::Cleanup => EvidenceOriginKind::Cleanup(true),
    };
    EvidenceScope {
        session_id: scope.session_id.as_ref().map(ToString::to_string),
        cycle_id: scope.test_cycle_id.as_ref().map(ToString::to_string),
        case_id: scope.test_case_id.as_ref().map(ToString::to_string),
        execution_id: scope.test_execution_id.as_ref().map(ToString::to_string),
        step_id: scope.step_id.as_ref().map(ToString::to_string),
        criterion_id: scope.criterion_id.as_ref().map(ToString::to_string),
        template_id: scope.template_id.as_ref().map(ToString::to_string),
        suite_id: scope.suite_id.as_ref().map(ToString::to_string),
        origin: Some(WireOrigin { kind: Some(kind) }),
    }
}

fn severity(value: RecordSeverity) -> Severity {
    match value {
        RecordSeverity::Debug => Severity::Debug,
        RecordSeverity::Info => Severity::Info,
        RecordSeverity::Warning => Severity::Warn,
        RecordSeverity::Error => Severity::Error,
    }
}

fn corrupt() -> Box<Failure> {
    Failure::new(Reason::Corruption, "dokime.query.list_records", "Stored record timestamp is invalid")
}

/* === Tests === */
