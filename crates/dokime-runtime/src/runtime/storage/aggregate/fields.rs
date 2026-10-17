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

// Third Party Imports

// Local Imports
use crate::{runtime::Error, runtime::Result, runtime::query::aggregate::AggregateSource};
use dokime_domain::model::identity::Metadata;
use dokime_domain::model::time::Timestamp;
use dokime_domain::model::values::Value;
use crate::definition::TestCase;
use crate::execution::CycleStatus;
use crate::execution::ExecutionStatus;
use crate::execution::StepRecord;
use crate::execution::StepStatus;
use crate::execution::TestCycle;
use crate::execution::TestExecution;
use dokime_domain::model::records::evidence::RecordEntry;
use crate::records::RecordSeverity;

/* === Constants === */
const COMMON: &[&str] = &[
    "label",
    "description",
    "tags",
    "attributes",
    "definition_identifier",
    "definition_created_at",
    "definition_updated_at",
];
const CASE: &[&str] = &[
    "identifier",
    "template_identifier",
    "suite_identifier",
    "generation_index",
];
const CYCLE: &[&str] = &[
    "identifier",
    "session_identifier",
    "status",
    "created_at",
    "updated_at",
    "started_at",
    "continue_on_failure",
    "execution_count",
];
const EXECUTION: &[&str] = &[
    "identifier",
    "case_identifier",
    "cycle_identifier",
    "template_identifier",
    "status",
    "mode",
    "started_at",
    "finished_at",
    "sealed_at",
];
const STEP: &[&str] = &[
    "identifier",
    "execution_identifier",
    "case_identifier",
    "cycle_identifier",
    "template_identifier",
    "execution_status",
    "status",
    "ordinal",
    "started_at",
    "finished_at",
    "duration_ms",
];
const RECORD: &[&str] = &[
    "identifier",
    "sequence",
    "timestamp",
    "received_at",
    "record_type",
    "schema_version",
    "source",
    "severity",
    "message",
    "details",
    "session_identifier",
    "execution_identifier",
    "cycle_identifier",
    "case_identifier",
    "step_identifier",
    "related_sample_identifier",
    "related_stream_identifier",
    "related_artifact_identifier",
];

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */
pub(super) enum Row<'a> {
    Case(&'a TestCase),
    Cycle(&'a TestCycle),
    Execution(&'a TestExecution),
    Step {
        execution: &'a TestExecution,
        step: &'a StepRecord,
        ordinal: usize,
    },
    Record(&'a RecordEntry),
}

pub(super) enum Field<'a> {
    Null,
    Bool(bool),
    Unsigned(u64),
    Number(f64),
    Datetime(time::OffsetDateTime),
    Text(&'a str),
    Strings(&'a [String]),
    Object(&'a BTreeMap<String, Value>),
}

/* === Traits === */

/* === Implementations === */
impl Row<'_> {
    pub(super) fn source(&self) -> AggregateSource {
        match self {
            Self::Case(_) => AggregateSource::Cases,
            Self::Cycle(_) => AggregateSource::Cycles,
            Self::Execution(_) => AggregateSource::Executions,
            Self::Step { .. } => AggregateSource::Steps,
            Self::Record(_) => AggregateSource::Records,
        }
    }

    pub(super) fn field(&self, name: &str) -> Result<Field<'_>> {
        if !known(self.source(), name) {
            return Err(invalid("unknown native aggregate column"));
        }
        let metadata = match self {
            Self::Case(case) => Some(&case.metadata),
            Self::Cycle(cycle) => Some(&cycle.metadata),
            Self::Execution(execution) => Some(&execution.snapshot.case().metadata),
            Self::Step { step, .. } => Some(&step.metadata),
            Self::Record(_) => None,
        };
        if let Some(field) = metadata.and_then(|metadata| metadata_field(metadata, name)) {
            return Ok(field);
        }
        let field = match self {
            Self::Case(case) => match name {
                "identifier" => Field::Text(case.test_case_id.as_str()),
                "template_identifier" => Field::Text(case.test_case_template_id.as_str()),
                "suite_identifier" => Field::Text(&case.suite_id),
                "generation_index" => Field::Unsigned(case.generation_index),
                _ => return Err(invalid("native case column has no projection")),
            },
            Self::Cycle(cycle) => match name {
                "identifier" => Field::Text(cycle.test_cycle_id.as_str()),
                "session_identifier" => text(cycle.session_id.as_ref().map(|id| id.as_str())),
                "status" => Field::Text(cycle_status(cycle.lifecycle)),
                "created_at" => timestamp(Some(&cycle.created_at))?,
                "updated_at" => timestamp(Some(&cycle.updated_at))?,
                "started_at" => timestamp(cycle.started_at.as_ref())?,
                "continue_on_failure" => Field::Bool(cycle.continue_on_failure),
                "execution_count" => Field::Unsigned(
                    u64::try_from(cycle.execution_ids.len())
                        .map_err(|_| invalid("cycle membership count overflows"))?,
                ),
                _ => return Err(invalid("native cycle column has no projection")),
            },
            Self::Execution(execution) => execution_field(execution, name)?,
            Self::Step {
                execution,
                step,
                ordinal,
            } => match name {
                "identifier" => Field::Text(step.step_id.as_str()),
                "execution_identifier" => Field::Text(execution.test_execution_id.as_str()),
                "execution_status" => Field::Text(execution_status(&execution.status)),
                "status" => Field::Text(step_status(&step.status)),
                "ordinal" => Field::Unsigned(
                    u64::try_from(*ordinal).map_err(|_| invalid("step ordinal overflows"))?,
                ),
                "started_at" => timestamp(step.started_at.as_ref())?,
                "finished_at" => timestamp(step.finished_at.as_ref())?,
                "duration_ms" => duration(step.started_at.as_ref(), step.finished_at.as_ref())?,
                _ => execution_field(execution, name)?,
            },
            Self::Record(record) => match name {
                "identifier" => Field::Text(record.record_id.as_str()),
                "sequence" => Field::Unsigned(record.sequence),
                "timestamp" => timestamp(Some(&record.timestamp))?,
                "received_at" => timestamp(Some(&record.received_at))?,
                "record_type" => Field::Text(record.record_type.as_str()),
                "schema_version" => Field::Unsigned(u64::from(record.schema_version)),
                "source" => Field::Text(&record.source),
                "severity" => Field::Text(match record.severity {
                    RecordSeverity::Debug => "debug",
                    RecordSeverity::Info => "info",
                    RecordSeverity::Warning => "warning",
                    RecordSeverity::Error => "error",
                }),
                "message" => text(record.message.as_deref()),
                "details" => Field::Object(&record.details),
                "session_identifier" => text(
                    record
                        .scope
                        .as_ref()
                        .and_then(|scope| scope.session_id.as_ref())
                        .map(|id| id.as_str()),
                ),
                "execution_identifier" => text(
                    record
                        .scope
                        .as_ref()
                        .and_then(|scope| scope.test_execution_id.as_ref())
                        .map(|id| id.as_str()),
                ),
                "cycle_identifier" => text(
                    record
                        .scope
                        .as_ref()
                        .and_then(|scope| scope.test_cycle_id.as_ref())
                        .map(|id| id.as_str()),
                ),
                "case_identifier" => text(
                    record
                        .scope
                        .as_ref()
                        .and_then(|scope| scope.test_case_id.as_ref())
                        .map(|id| id.as_str()),
                ),
                "step_identifier" => text(
                    record
                        .scope
                        .as_ref()
                        .and_then(|scope| scope.step_id.as_ref())
                        .map(|id| id.as_str()),
                ),
                "related_sample_identifier" => {
                    text(record.related_sample_id.as_ref().map(|id| id.as_str()))
                }
                "related_stream_identifier" => {
                    text(record.related_stream_id.as_ref().map(|id| id.as_str()))
                }
                "related_artifact_identifier" => {
                    text(record.related_artifact_id.as_ref().map(|id| id.as_str()))
                }
                _ => return Err(invalid("native record column has no projection")),
            },
        };
        Ok(field)
    }
}

impl Field<'_> {
    pub(super) fn project(&self) -> Result<Value> {
        Ok(match self {
            Self::Null => Value::Null,
            Self::Bool(value) => Value::Bool(*value),
            Self::Unsigned(value) => Value::Unsigned(*value),
            Self::Number(value) if value.is_finite() => Value::Number(*value),
            Self::Number(_) => return Err(invalid("aggregate number must be finite")),
            Self::Datetime(value) => Value::Datetime(*value),
            Self::Text(value) => Value::Text((*value).to_owned()),
            Self::Strings(values) => Value::Array(
                values
                    .iter()
                    .map(|value| Value::Text(value.clone()))
                    .collect(),
            ),
            Self::Object(values) => Value::Object((*values).clone()),
        })
    }
}

/* === Public Functions === */
pub(crate) fn columns(source: AggregateSource) -> impl Iterator<Item = &'static str> {
    let specific = match source {
        AggregateSource::Cases => CASE,
        AggregateSource::Cycles => CYCLE,
        AggregateSource::Executions => EXECUTION,
        AggregateSource::Steps => STEP,
        AggregateSource::Records => RECORD,
    };
    let common = if source == AggregateSource::Records {
        &[]
    } else {
        COMMON
    };
    common.iter().chain(specific.iter()).copied()
}

pub(super) fn known(source: AggregateSource, name: &str) -> bool {
    columns(source).any(|column| column == name)
}

/* === Private Functions === */
fn metadata_field<'a>(metadata: &'a Metadata, name: &str) -> Option<Field<'a>> {
    Some(match name {
        "label" => Field::Text(&metadata.label),
        "description" => text(metadata.description.as_deref()),
        "tags" => Field::Strings(&metadata.tags),
        "attributes" => Field::Object(&metadata.attributes),
        "definition_identifier" => Field::Text(metadata.identifier.as_str()),
        "definition_created_at" => metadata.created_at.map_or(Field::Null, Field::Datetime),
        "definition_updated_at" => metadata.updated_at.map_or(Field::Null, Field::Datetime),
        _ => return None,
    })
}
fn execution_field<'a>(execution: &'a TestExecution, name: &str) -> Result<Field<'a>> {
    Ok(match name {
        "identifier" => Field::Text(execution.test_execution_id.as_str()),
        "case_identifier" => Field::Text(execution.test_case_id.as_str()),
        "cycle_identifier" => Field::Text(execution.test_cycle_id.as_str()),
        "template_identifier" => {
            Field::Text(execution.snapshot.case().test_case_template_id.as_str())
        }
        "status" => Field::Text(execution_status(&execution.status)),
        "mode" => Field::Text(execution.mode.as_str()),
        "started_at" => timestamp(execution.started_at.as_ref())?,
        "finished_at" => timestamp(execution.finished_at.as_ref())?,
        "sealed_at" => timestamp(execution.saved.as_ref().map(|seal| &seal.sealed_at))?,
        _ => return Err(invalid("native execution column has no projection")),
    })
}
fn text(value: Option<&str>) -> Field<'_> {
    value.map_or(Field::Null, Field::Text)
}
fn timestamp(value: Option<&Timestamp>) -> Result<Field<'_>> {
    value
        .map(|value| {
            value
                .to_datetime()
                .map(Field::Datetime)
                .map_err(|_| invalid("invalid aggregate timestamp"))
        })
        .transpose()
        .map(|value| value.unwrap_or(Field::Null))
}
fn duration(start: Option<&Timestamp>, finish: Option<&Timestamp>) -> Result<Field<'static>> {
    let (Some(start), Some(finish)) = (start, finish) else {
        return Ok(Field::Null);
    };
    let start = start
        .to_datetime()
        .map_err(|_| invalid("invalid step start timestamp"))?;
    let finish = finish
        .to_datetime()
        .map_err(|_| invalid("invalid step finish timestamp"))?;
    let nanos = finish
        .unix_timestamp_nanos()
        .checked_sub(start.unix_timestamp_nanos())
        .ok_or_else(|| invalid("step duration overflows"))?;
    if nanos < 0 {
        return Err(invalid("step finished before it started"));
    }
    Ok(Field::Number(nanos as f64 / 1_000_000.0))
}
fn execution_status(status: &ExecutionStatus) -> &'static str {
    match status {
        ExecutionStatus::Skipped => "skipped",
        ExecutionStatus::Invalidated => "invalidated",
        ExecutionStatus::Pending => "pending",
        ExecutionStatus::Running => "running",
        ExecutionStatus::Passed => "passed",
        ExecutionStatus::Failed => "failed",
        ExecutionStatus::Blocked => "blocked",
        ExecutionStatus::Cancelled => "cancelled",
        ExecutionStatus::Interrupted => "interrupted",
    }
}
fn step_status(status: &StepStatus) -> &'static str {
    match status {
        StepStatus::Pending => "pending",
        StepStatus::Running => "running",
        StepStatus::Passed => "passed",
        StepStatus::Failed => "failed",
        StepStatus::Skipped => "skipped",
        StepStatus::Cancelled => "cancelled",
    }
}
fn cycle_status(status: CycleStatus) -> &'static str {
    match status {
        CycleStatus::Ready => "ready",
        CycleStatus::Running => "running",
        CycleStatus::Paused => "paused",
        CycleStatus::Stopping => "stopping",
        CycleStatus::Completed => "completed",
        CycleStatus::Failed => "failed",
        CycleStatus::Blocked => "blocked",
        CycleStatus::Stopped => "stopped",
    }
}
fn invalid(message: &'static str) -> Error {
    Error::validation("aggregate_query_invalid", message, Vec::new())
}

/* === Tests === */
