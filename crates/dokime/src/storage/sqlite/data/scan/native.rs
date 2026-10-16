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
use std::{
    collections::BTreeMap,
    num::{NonZeroU128, NonZeroUsize},
};
// Third Party Imports
use harmos::{
    schema::{
        Described,
        graph::{Schema, View},
    },
    storage::{SchemaKey, frame::*, reduce::*},
};
// Local Imports
use crate::storage::sqlite::{Error, query};
use crate::{
    core::{EvidenceOrigin, RecordScope, Value},
    storage::records,
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub(super) struct Row {
    value: Data,
}

/* === Enums === */
#[derive(Clone, Copy)]
pub(in crate::storage::sqlite::data) enum Kind {
    Record,
}
enum Data {
    Record(crate::records::RecordEntry),
}

/* === Traits === */

/* === Implementations === */
impl Kind {
    pub(super) fn for_schema(key: &SchemaKey) -> Option<Self> {
        if *key == records::schema() {
            Some(Self::Record)
        } else {
            None
        }
    }
    pub(super) fn schema(self) -> Result<Schema, Error> {
        let declaration = match self {
            Self::Record => records::RecordRow::DEFINITION,
        };
        Schema::from_declaration(declaration, &harmos::config::SchemaSettings::default()).map_err(
            |error| {
                harmos::Error::backend(
                    "resolve native scan schema",
                    harmos::BackendFailure::Permanent,
                    error,
                )
                .into()
            },
        )
    }
    pub(super) fn check_schema(self, schema: &Schema) -> Result<(), Error> {
        let expected = self.schema()?;
        if !schema
            .equivalent(
                schema.root,
                &expected,
                expected.root,
                &harmos::config::SchemaSettings::default(),
            )
            .map_err(|error| {
                harmos::Error::backend(
                    "compare native scan schema",
                    harmos::BackendFailure::Corrupt,
                    error,
                )
            })?
        {
            return Err(Error::Corrupt("native scan schema differs from its codec"));
        }
        Ok(())
    }
    pub(super) fn check_field(self, field: &ScalarField, schema: View<'_>) -> Result<(), Error> {
        field.check()?;
        let ScalarPath::Payload(path) = &field.path else {
            return Ok(());
        };
        let first = path[0].as_str();
        let dynamic = match self {
            Self::Record => first == "details",
        };
        if dynamic && path.len() >= 2 {
            return Ok(());
        }
        if path.len() == 1
            && matches!(first, "timestamp" | "received_at")
            && field.kind == ScalarKind::Datetime
        {
            return Ok(());
        }
        if first == "error" {
            return Err(Error::Data(
                "native error details are not scalar projection fields",
            ));
        }
        field.check_schema(schema)?;
        Ok(())
    }
    pub(super) fn decode(self, payload: &[u8], maximum: NonZeroUsize) -> Result<Row, Error> {
        query::checkpoint()?;
        match self {
            Self::Record => {
                let row = records::RecordRow::from_bytes(payload, maximum.get()).map_err(codec)?;
                let bytes = row.projection_memory().map_err(codec)?;
                let record = row.into_entry(bound(bytes)?).map_err(codec)?;
                Ok(Row {
                    value: Data::Record(record),
                })
            }
        }
    }
}
impl Row {
    pub(super) fn scalar(
        &self,
        path: &[harmos::Identifier],
        kind: ScalarKind,
        allowance: NonZeroUsize,
    ) -> Result<ScalarValue, Error> {
        query::checkpoint()?;
        match &self.value {
            Data::Record(entry) => {
                let record = &entry.record;
                if path[0].as_str() == "details" {
                    return cell(&record.details, &path[1..], kind, allowance);
                }
                if path[0].as_str() == "scope" {
                    return scope(record.scope.as_ref(), &path[1..], allowance);
                }
                match path[0].as_str() {
                    "sequence" => Ok(ScalarValue::Unsigned(entry.sequence)),
                    "schema_version" => Ok(ScalarValue::Unsigned(u64::from(record.schema_version))),
                    "timestamp" => timestamp(&record.timestamp, kind, allowance),
                    "received_at" => timestamp(&record.received_at, kind, allowance),
                    "record_id" => text(record.record_id.as_str(), allowance),
                    "record_type" => text(record.record_type.as_str(), allowance),
                    "source" => text(&record.source, allowance),
                    "severity" => text(
                        match record.severity {
                            crate::records::RecordSeverity::Debug => "debug",
                            crate::records::RecordSeverity::Info => "info",
                            crate::records::RecordSeverity::Warning => "warning",
                            crate::records::RecordSeverity::Error => "error",
                        },
                        allowance,
                    ),
                    "message" => optional_text(record.message.as_deref(), allowance),
                    "related_sample_id" => optional_text(
                        record.related_sample_id.as_ref().map(|id| id.as_str()),
                        allowance,
                    ),
                    "related_stream_id" => optional_text(
                        record.related_stream_id.as_ref().map(|id| id.as_str()),
                        allowance,
                    ),
                    "related_artifact_id" => optional_text(
                        record.related_artifact_id.as_ref().map(|id| id.as_str()),
                        allowance,
                    ),
                    _ => Err(Error::Data("unsupported native record field")),
                }
            }
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn cell(
    map: &BTreeMap<String, Value>,
    path: &[harmos::Identifier],
    kind: ScalarKind,
    allowance: NonZeroUsize,
) -> Result<ScalarValue, Error> {
    let Some(mut value) = map.get(path[0].as_str()) else {
        return Ok(ScalarValue::Missing);
    };
    for name in &path[1..] {
        query::checkpoint()?;
        match value {
            Value::Null => return Ok(ScalarValue::Null),
            Value::Object(map) => match map.get(name.as_str()) {
                Some(next) => value = next,
                None => return Ok(ScalarValue::Missing),
            },
            _ => return Err(Error::Data("native scalar path crosses a non-object value")),
        }
    }
    Ok(match value {
        Value::Null => ScalarValue::Null,
        Value::Bool(value) => ScalarValue::Bool(*value),
        Value::Integer(value) if kind == ScalarKind::Exact => ScalarValue::Exact(ExactNumber::new(
            *value < 0,
            u128::from(value.unsigned_abs()),
            NonZeroU128::new(1).unwrap(),
        )),
        Value::Unsigned(value) if kind == ScalarKind::Exact => ScalarValue::Exact(
            ExactNumber::new(false, u128::from(*value), NonZeroU128::new(1).unwrap()),
        ),
        Value::Integer(value) => ScalarValue::Signed(*value),
        Value::Unsigned(value) => ScalarValue::Unsigned(*value),
        Value::Number(value) => ScalarValue::Float(*value),
        Value::Text(value) => text(value, allowance)?,
        Value::Datetime(value) => ScalarValue::Datetime(value.unix_timestamp_nanos()),
        Value::Object(_) | Value::Array(_) => {
            return Err(Error::Data("native projected value is not scalar"));
        }
    })
}
fn scope(
    scope: Option<&RecordScope>,
    path: &[harmos::Identifier],
    allowance: NonZeroUsize,
) -> Result<ScalarValue, Error> {
    let Some(scope) = scope else {
        return Ok(ScalarValue::Null);
    };
    let value = match path[0].as_str() {
        "session_id" => scope.session_id.as_ref().map(|id| id.as_str()),
        "test_cycle_id" => scope.test_cycle_id.as_ref().map(|id| id.as_str()),
        "test_case_id" => scope.test_case_id.as_ref().map(|id| id.as_str()),
        "test_execution_id" => scope.test_execution_id.as_ref().map(|id| id.as_str()),
        "step_id" => scope.step_id.as_ref().map(|id| id.as_str()),
        "criterion_id" => scope.criterion_id.as_ref().map(|id| id.as_str()),
        "template_id" => scope.template_id.as_ref().map(|id| id.as_str()),
        "suite_id" => scope.suite_id.as_ref().map(|id| id.as_str()),
        "origin" => {
            return match path[1].as_str() {
                "phase" => Ok(ScalarValue::Unsigned(match scope.origin {
                    EvidenceOrigin::Default => 0,
                    EvidenceOrigin::Initialization => 1,
                    EvidenceOrigin::Step { .. } => 2,
                    EvidenceOrigin::Cleanup => 3,
                })),
                "attempt_id" => optional_text(
                    match &scope.origin {
                        EvidenceOrigin::Step { attempt_id } => Some(attempt_id.as_str()),
                        _ => None,
                    },
                    allowance,
                ),
                _ => Err(Error::Data("unsupported native origin field")),
            };
        }
        _ => return Err(Error::Data("unsupported native scope field")),
    };
    optional_text(value, allowance)
}
fn timestamp(
    value: &crate::core::Timestamp,
    kind: ScalarKind,
    allowance: NonZeroUsize,
) -> Result<ScalarValue, Error> {
    if kind != ScalarKind::Datetime {
        return text(value.as_str(), allowance);
    }
    let parsed = value
        .to_datetime()
        .map_err(|_| Error::Corrupt("native scan timestamp is invalid"))?;
    Ok(ScalarValue::Datetime(parsed.unix_timestamp_nanos()))
}
fn optional_text(value: Option<&str>, allowance: NonZeroUsize) -> Result<ScalarValue, Error> {
    match value {
        Some(value) => text(value, allowance),
        None => Ok(ScalarValue::Null),
    }
}
fn text(value: &str, allowance: NonZeroUsize) -> Result<ScalarValue, Error> {
    if value.len() > allowance.get() {
        return Err(exhausted());
    }
    let mut text = String::new();
    text.try_reserve_exact(value.len())
        .map_err(|_| exhausted())?;
    if text.capacity() > value.len() {
        return Err(exhausted());
    }
    text.push_str(value);
    Ok(ScalarValue::Text(text))
}
fn bound(bytes: usize) -> Result<NonZeroUsize, Error> {
    NonZeroUsize::new(bytes)
        .filter(|bytes| bytes.get() != usize::MAX)
        .ok_or_else(exhausted)
}
fn exhausted() -> Error {
    harmos::Error::ResourceExhausted {
        resource: "SQLite native scan projection",
    }
    .into()
}
fn codec(error: crate::storage::Error) -> Error {
    match error {
        crate::storage::Error::Runtime(error) => error.into(),
        other => harmos::Error::backend(
            "decode native scan source",
            harmos::BackendFailure::Corrupt,
            other,
        )
        .into(),
    }
}

/* === Tests === */
