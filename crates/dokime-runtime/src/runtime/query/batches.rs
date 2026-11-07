/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Borrowed Projected Arrow Columns for Deterministic Query Folds

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use arrow_array::{Array, ArrayRef, Float64Array, Int64Array, RecordBatch, StringArray, StructArray, UInt64Array};

// Local Imports
use super::selection::{Envelope, FieldRef, Selection};
use crate::storage::{Error, ipc, sqlite::storage_error};
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
    pub fn new(batch: &'a RecordBatch, contract: &'a TelemetryContract, operation: &'static str) -> Result<Self, Box<Failure>> {
        let structure = |index: usize| {
            batch
                .columns()
                .get(index)
                .and_then(|column| column.as_any().downcast_ref::<StructArray>())
                .ok_or_else(|| storage_error(Error::Corrupt("query batch column is not a struct"), operation))
        };
        let metadata = structure(0)?;
        let values = structure(1)?;
        if metadata.null_count() != 0 || values.null_count() != 0 {
            return Err(storage_error(Error::Corrupt("query root struct is null"), operation));
        }
        Ok(Self { metadata, values, contract, operation })
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

    pub fn source(&self, row: usize) -> Result<&str, Box<Failure>> {
        self.text("source", row)
    }

    pub fn time(&self, name: &str, row: usize) -> Result<time::OffsetDateTime, Box<Failure>> {
        dokime_domain::model::time::Timestamp::new(self.text(name, row)?)
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
                Envelope::Id => Value::Text(self.text("sample_id", row)?.to_owned()),
                Envelope::Source => Value::Text(self.source(row)?.to_owned()),
                Envelope::SchemaVersion => Value::Unsigned(self.contract.schema_version.into()),
                Envelope::Timestamp => Value::Datetime(timestamp),
                Envelope::ReceivedAt => Value::Datetime(self.time("received_at", row)?),
                Envelope::Severity => return Ok(None),
            },
        };
        Ok(Some(value))
    }

    fn text(&self, name: &str, row: usize) -> Result<&str, Box<Failure>> {
        let column = self
            .metadata
            .column_by_name(name)
            .and_then(|column| column.as_any().downcast_ref::<StringArray>())
            .filter(|column| !column.is_null(row))
            .ok_or_else(|| storage_error(Error::Corrupt("query text metadata is absent or null"), self.operation))?;
        Ok(column.value(row))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_column_quality() {
        let numbers = Float64Array::from(vec![None, Some(f64::NAN), Some(f64::INFINITY), Some(-0.0), Some(f64::from_bits(1))]);
        let column = Column::Number(&numbers);
        assert!(matches!(column.cell(0), Cell::Null));
        assert!(matches!(column.cell(1), Cell::NonFinite));
        assert!(matches!(column.cell(2), Cell::NonFinite));
        for index in 3..5 {
            let Cell::Number(value) = column.cell(index) else { panic!("finite number") };
            assert_eq!(value.to_bits(), numbers.value(index).to_bits());
        }
        let unsigned = UInt64Array::from(vec![Some(u64::MAX), None]);
        let signed = Int64Array::from(vec![Some(i64::MIN), None]);
        for (column, expected) in [(Column::Unsigned(&unsigned), u64::MAX as f64), (Column::Signed(&signed), i64::MIN as f64)] {
            let Cell::Number(value) = column.cell(0) else { panic!("projected number") };
            assert_eq!(value.to_bits(), expected.to_bits());
            assert!(matches!(column.cell(1), Cell::Null));
        }
        assert!(matches!(Column::Missing.cell(0), Cell::Missing));
    }

    #[test]
    fn query_batch_corruption() {
        use std::sync::Arc;
        let contract = TelemetryContract::declared("query.readings", "Readings");
        let invalid = RecordBatch::try_from_iter([("metadata", Arc::new(UInt64Array::from(vec![1])) as ArrayRef)]).unwrap();
        assert!(Batch::new(&invalid, &contract, "test.query").is_err());
        let metadata = StructArray::from(vec![(
            Arc::new(arrow_schema::Field::new("source", arrow_schema::DataType::Utf8, true)),
            Arc::new(StringArray::from(vec![None::<&str>])) as ArrayRef,
        )]);
        let input = RecordBatch::try_from_iter([
            ("metadata", Arc::new(metadata) as ArrayRef),
            ("values", Arc::new(StructArray::new_empty_fields(1, None)) as ArrayRef),
        ])
        .unwrap();
        let batch = Batch::new(&input, &contract, "test.query").unwrap();
        assert!(batch.source(0).is_err());
        assert!(matches!(batch.column("absent").cell(0), Cell::Missing));
        let invalid = RecordBatch::try_from_iter([
            ("metadata", input.column(0).clone()),
            ("values", Arc::new(StructArray::new_empty_fields(1, Some(arrow_buffer::NullBuffer::new_null(1)))) as ArrayRef),
        ])
        .unwrap();
        assert!(Batch::new(&invalid, &contract, "test.query").is_err());
    }
}
