/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Borrowed Projected Arrow Columns For Deterministic Query Folds

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use arrow_array::{Array, ArrayRef, Float64Array, Int64Array, RecordBatch, StringArray, StructArray, UInt64Array};

// Local Imports
use super::selection::{Envelope, FieldRef, Selection};
use crate::storage::{ipc, sqlite::storage_error};
use dokime_domain::model::{identity::wire::Timestamp, query::error::Failure, telemetry::TelemetryContract, values::Value};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(super) struct Batch<'a> {
    metadata: &'a StructArray,
    values: &'a StructArray,
    contract: &'a TelemetryContract,
    operation: &'static str,
}

/* === Enums === */

pub(super) enum Column<'a> {
    Missing,
    Signed(&'a Int64Array),
    Unsigned(&'a UInt64Array),
    Number(&'a Float64Array),
    Other(&'a ArrayRef),
}

pub(super) enum Cell {
    Missing,
    Null,
    NonFinite,
    Number(f64),
}

/* === Traits === */

/* === Implementations === */

impl<'a> Batch<'a> {
    pub fn new(batch: &'a RecordBatch, contract: &'a TelemetryContract, operation: &'static str) -> Self {
        Self {
            metadata: batch.column(0).as_any().downcast_ref::<StructArray>().expect("IPC metadata"),
            values: batch.column(1).as_any().downcast_ref::<StructArray>().expect("IPC values"),
            contract,
            operation,
        }
    }

    pub fn column(&self, id: &str) -> Column<'a> {
        if !self.contract.columns.iter().any(|column| column.metadata.identifier.as_str() == id) {
            return Column::Missing;
        }
        let Some(column) = self.values.column_by_name(id) else {
            return Column::Missing;
        };
        if let Some(array) = column.as_any().downcast_ref::<Float64Array>() {
            return Column::Number(array);
        }
        if let Some(array) = column.as_any().downcast_ref::<Int64Array>() {
            return Column::Signed(array);
        }
        if let Some(array) = column.as_any().downcast_ref::<UInt64Array>() {
            return Column::Unsigned(array);
        }
        Column::Other(column)
    }

    pub fn source(&self, row: usize) -> &str {
        self.text("source", row)
    }

    pub fn time(&self, name: &str, row: usize) -> Result<time::OffsetDateTime, Box<Failure>> {
        dokime_domain::model::time::Timestamp::new(self.text(name, row))
            .to_datetime()
            .map_err(|_| Failure::new(dokime_domain::model::query::error::Reason::Corruption, self.operation, "Invalid query timestamp"))
    }

    pub fn matches(&self, selection: &Selection, row: usize) -> Result<bool, Box<Failure>> {
        let timestamp = self.time("timestamp", row)?;
        let mut failure = None;
        let selected = selection.matches_owned(Some(Timestamp::from(timestamp)), |field| match self.value(field, timestamp, row) {
            Ok(value) => value,
            Err(error) => {
                failure = Some(error);
                None
            }
        });
        if let Some(error) = failure {
            return Err(error);
        }
        Ok(selected)
    }

    fn value(&self, field: &FieldRef, timestamp: time::OffsetDateTime, row: usize) -> Result<Option<Value>, Box<Failure>> {
        let value = match field {
            FieldRef::Declared(id) => {
                if !self.contract.columns.iter().any(|column| column.metadata.identifier.as_str() == id) {
                    return Ok(None);
                }
                let Some(array) = self.values.column_by_name(id) else {
                    return Ok(None);
                };
                ipc::cell(array, row).map_err(|error| storage_error(error, self.operation))?
            }
            FieldRef::Envelope(envelope) => match envelope {
                Envelope::Id => Value::Text(self.text("sample_id", row).to_owned()),
                Envelope::Source => Value::Text(self.source(row).to_owned()),
                Envelope::SchemaVersion => Value::Unsigned(self.contract.schema_version.into()),
                Envelope::Timestamp => Value::Datetime(timestamp),
                Envelope::ReceivedAt => Value::Datetime(self.time("received_at", row)?),
                Envelope::Severity => return Ok(None),
            },
        };
        Ok(Some(value))
    }

    fn text(&self, name: &str, row: usize) -> &str {
        self.metadata
            .column_by_name(name)
            .expect("projected metadata")
            .as_any()
            .downcast_ref::<StringArray>()
            .expect("text metadata")
            .value(row)
    }
}

impl Column<'_> {
    pub fn cell(&self, row: usize) -> Cell {
        let value = match self {
            Self::Missing => return Cell::Missing,
            Self::Signed(array) => {
                if array.is_null(row) {
                    return Cell::Null;
                }
                array.value(row) as f64
            }
            Self::Unsigned(array) => {
                if array.is_null(row) {
                    return Cell::Null;
                }
                array.value(row) as f64
            }
            Self::Number(array) => {
                if array.is_null(row) {
                    return Cell::Null;
                }
                array.value(row)
            }
            Self::Other(array) => return if array.is_null(row) { Cell::Null } else { Cell::NonFinite },
        };
        if value.is_finite() { Cell::Number(value) } else { Cell::NonFinite }
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
