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
use std::collections::BTreeMap;
use std::fmt::{self, Write};

// Third Party Imports
use tonic::Status;
// Local Imports
use super::super::value::{self, Budget};
use dokime_app::core::{EvidenceOrigin, RecordScope, Value};
use dokime_app::records::{RecordEntry, RecordSeverity};
use dokime_protocol::v1;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct DisplayBytes(usize);

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Write for DisplayBytes {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        self.0 = self.0.checked_add(text.len()).ok_or(fmt::Error)?;
        Ok(())
    }
}

/* === Public Functions === */
pub(crate) fn allocation_memory(entry: &RecordEntry) -> Result<usize, Status> {
    let record = &entry.record;
    let mut bytes = formatted_memory(&record.record_id)
        .saturating_add(formatted_memory(&record.record_type))
        .saturating_add(copy_memory(record.source.len()));
    if let Some(message) = &record.message {
        bytes = bytes.saturating_add(copy_memory(message.len()));
    }
    for identity in [
        record
            .related_sample_id
            .as_ref()
            .map(|value| value as &dyn fmt::Display),
        record
            .related_stream_id
            .as_ref()
            .map(|value| value as &dyn fmt::Display),
        record
            .related_artifact_id
            .as_ref()
            .map(|value| value as &dyn fmt::Display),
    ]
    .into_iter()
    .flatten()
    {
        bytes = bytes.saturating_add(formatted_memory(identity));
    }
    let mut budget = Budget::new();
    bytes = bytes.saturating_add(object_allocation(&record.details, &mut budget, 0)?);
    if let Some(scope) = &record.scope {
        bytes = bytes.saturating_add(scope_allocation(scope));
    }
    if let Some(error) = &record.error {
        value::collection(error.cause_chain.len())?;
        bytes = bytes.saturating_add(copy_memory(error.message.len()));
        for text in [error.code.as_ref(), error.source.as_ref()]
            .into_iter()
            .flatten()
        {
            bytes = bytes.saturating_add(copy_memory(text.len()));
        }
        bytes = bytes.saturating_add(slots::<v1::CapturedErrorCause>(error.cause_chain.len()));
        for cause in &error.cause_chain {
            bytes = bytes.saturating_add(copy_memory(cause.message.len()));
            for text in [cause.code.as_ref(), cause.source.as_ref()]
                .into_iter()
                .flatten()
            {
                bytes = bytes.saturating_add(copy_memory(text.len()));
            }
        }
        for identity in [
            error
                .stacktrace_attachment
                .as_ref()
                .map(|value| value as &dyn fmt::Display),
            error
                .trace_id
                .as_ref()
                .map(|value| value as &dyn fmt::Display),
            error
                .correlation_id
                .as_ref()
                .map(|value| value as &dyn fmt::Display),
        ]
        .into_iter()
        .flatten()
        {
            bytes = bytes.saturating_add(formatted_memory(identity));
        }
    }
    if bytes == usize::MAX {
        return Err(Status::resource_exhausted(
            "captured record allocation size overflow",
        ));
    }
    Ok(bytes)
}

pub(super) fn owned_memory(record: &v1::CapturedRecord) -> usize {
    let mut bytes = record
        .record_id
        .capacity()
        .saturating_add(record.record_type.capacity())
        .saturating_add(record.source.capacity());
    for text in [
        record.message.as_ref(),
        record.related_sample_id.as_ref(),
        record.related_stream_id.as_ref(),
        record.related_artifact_id.as_ref(),
    ]
    .into_iter()
    .flatten()
    {
        bytes = bytes.saturating_add(text.capacity());
    }
    if let Some(details) = &record.details {
        bytes = bytes.saturating_add(object_owned(&details.fields));
    }
    if let Some(scope) = &record.scope {
        for text in [
            scope.session_id.as_ref(),
            scope.cycle_id.as_ref(),
            scope.case_id.as_ref(),
            scope.execution_id.as_ref(),
            scope.step_id.as_ref(),
            scope.criterion_id.as_ref(),
            scope.template_id.as_ref(),
            scope.suite_id.as_ref(),
        ]
        .into_iter()
        .flatten()
        {
            bytes = bytes.saturating_add(text.capacity());
        }
        if let Some(v1::evidence_origin::Kind::StepAttemptId(identity)) = scope
            .origin
            .as_ref()
            .and_then(|origin| origin.kind.as_ref())
        {
            bytes = bytes.saturating_add(identity.capacity());
        }
    }
    if let Some(error) = &record.error {
        bytes = bytes
            .saturating_add(error.message.capacity())
            .saturating_add(
                error
                    .cause_chain
                    .capacity()
                    .saturating_mul(std::mem::size_of::<v1::CapturedErrorCause>()),
            );
        for text in [
            error.code.as_ref(),
            error.source.as_ref(),
            error.stacktrace_attachment.as_ref(),
            error.trace_id.as_ref(),
            error.correlation_id.as_ref(),
        ]
        .into_iter()
        .flatten()
        {
            bytes = bytes.saturating_add(text.capacity());
        }
        for cause in &error.cause_chain {
            bytes = bytes.saturating_add(cause.message.capacity());
            for text in [cause.code.as_ref(), cause.source.as_ref()]
                .into_iter()
                .flatten()
            {
                bytes = bytes.saturating_add(text.capacity());
            }
        }
    }
    bytes
}

pub(crate) fn copy_memory(bytes: usize) -> usize {
    if bytes == 0 {
        return 0;
    }
    bytes.saturating_mul(2).max(8)
}

pub(super) fn severity(value: RecordSeverity) -> i32 {
    (match value {
        RecordSeverity::Debug => v1::Severity::Debug,
        RecordSeverity::Info => v1::Severity::Info,
        RecordSeverity::Warning => v1::Severity::Warn,
        RecordSeverity::Error => v1::Severity::Error,
    }) as i32
}

pub(crate) fn record(entry: &RecordEntry) -> Result<v1::CapturedRecord, Status> {
    let record = &entry.record;
    // ValueObject uses the shared top-level-map convention: its field values
    // start at depth zero and share one node budget, as in decode_object.
    let mut budget = Budget::new();
    value::collection(record.details.len())?;
    let details = record
        .details
        .iter()
        .map(|(key, item)| {
            value::text(key)?;
            Ok((key.clone(), value::encode(item, &mut budget, 0)?))
        })
        .collect::<Result<_, Status>>()?;
    let error = record
        .error
        .as_ref()
        .map(|error| {
            value::collection(error.cause_chain.len())?;
            Ok::<_, Status>(v1::CapturedRecordError {
                message: error.message.clone(),
                code: error.code.clone(),
                source: error.source.clone(),
                cause_chain: error
                    .cause_chain
                    .iter()
                    .map(|cause| v1::CapturedErrorCause {
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
    Ok(v1::CapturedRecord {
        record_id: record.record_id.to_string(),
        sequence: entry.sequence,
        timestamp: Some(value::timestamp(&record.timestamp)?),
        received_at: Some(value::timestamp(&record.received_at)?),
        record_type: record.record_type.to_string(),
        schema_version: record.schema_version,
        source: record.source.clone(),
        severity: severity(record.severity),
        message: record.message.clone(),
        error,
        details: Some(v1::ValueObject { fields: details }),
        scope: record.scope.as_ref().map(value::scope),
        related_sample_id: record.related_sample_id.as_ref().map(ToString::to_string),
        related_stream_id: record.related_stream_id.as_ref().map(ToString::to_string),
        related_artifact_id: record.related_artifact_id.as_ref().map(ToString::to_string),
    })
}

/* === Private Functions === */
fn slots<T>(len: usize) -> usize {
    if len == 0 {
        return 0;
    }
    len.max(4)
        .saturating_mul(2)
        .saturating_mul(std::mem::size_of::<T>())
}

fn formatted_memory(value: &dyn fmt::Display) -> usize {
    let mut bytes = DisplayBytes(0);
    if write!(&mut bytes, "{value}").is_err() {
        return usize::MAX;
    }
    copy_memory(bytes.0)
}

fn map_nodes(entries: usize) -> usize {
    let memory = harmos::btree_map_memory::<String, v1::Value>(entries);
    if memory == usize::MAX {
        return usize::MAX;
    }
    memory.saturating_sub(std::mem::size_of::<BTreeMap<String, v1::Value>>())
}

pub(crate) fn object_allocation(
    values: &BTreeMap<String, Value>,
    budget: &mut Budget,
    depth: usize,
) -> Result<usize, Status> {
    value::collection(values.len())?;
    let mut bytes = map_nodes(values.len());
    for (key, item) in values {
        value::text(key)?;
        bytes = bytes
            .saturating_add(copy_memory(key.len()))
            .saturating_add(value_allocation(item, budget, depth)?);
    }
    Ok(bytes)
}

fn value_allocation(item: &Value, budget: &mut Budget, depth: usize) -> Result<usize, Status> {
    budget.node(depth)?;
    Ok(match item {
        Value::Text(text) => {
            value::text(text)?;
            copy_memory(text.len())
        }
        // RFC 3339's bounded date/time spelling and String growth fit this scratch allowance.
        Value::Datetime(_) => 128,
        Value::Array(values) => {
            value::collection(values.len())?;
            let mut bytes = slots::<v1::Value>(values.len());
            for item in values {
                bytes = bytes.saturating_add(value_allocation(item, budget, depth + 1)?);
            }
            bytes
        }
        Value::Object(values) => object_allocation(values, budget, depth + 1)?,
        _ => 0,
    })
}

pub(crate) fn scope_allocation(scope: &RecordScope) -> usize {
    let mut bytes: usize = 0;
    for identity in [
        scope
            .session_id
            .as_ref()
            .map(|value| value as &dyn fmt::Display),
        scope
            .test_cycle_id
            .as_ref()
            .map(|value| value as &dyn fmt::Display),
        scope
            .test_case_id
            .as_ref()
            .map(|value| value as &dyn fmt::Display),
        scope
            .test_execution_id
            .as_ref()
            .map(|value| value as &dyn fmt::Display),
        scope
            .step_id
            .as_ref()
            .map(|value| value as &dyn fmt::Display),
        scope
            .criterion_id
            .as_ref()
            .map(|value| value as &dyn fmt::Display),
        scope
            .template_id
            .as_ref()
            .map(|value| value as &dyn fmt::Display),
        scope
            .suite_id
            .as_ref()
            .map(|value| value as &dyn fmt::Display),
    ]
    .into_iter()
    .flatten()
    {
        bytes = bytes.saturating_add(formatted_memory(identity));
    }
    if let EvidenceOrigin::Step { attempt_id } = &scope.origin {
        bytes = bytes.saturating_add(formatted_memory(attempt_id));
    }
    bytes
}

fn object_owned(values: &BTreeMap<String, v1::Value>) -> usize {
    let mut bytes = map_nodes(values.len());
    for (key, item) in values {
        bytes = bytes
            .saturating_add(key.capacity())
            .saturating_add(value_owned(item));
    }
    bytes
}

fn value_owned(item: &v1::Value) -> usize {
    match item.kind.as_ref() {
        Some(v1::value::Kind::TextValue(text) | v1::value::Kind::DatetimeValue(text)) => {
            text.capacity()
        }
        Some(v1::value::Kind::ArrayValue(values)) => values.values.iter().fold(
            values
                .values
                .capacity()
                .saturating_mul(std::mem::size_of::<v1::Value>()),
            |bytes, item| bytes.saturating_add(value_owned(item)),
        ),
        Some(v1::value::Kind::ObjectValue(values)) => object_owned(&values.fields),
        _ => 0,
    }
}

/* === Tests === */
