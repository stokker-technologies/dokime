/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Contract-Declared Arrow Telemetry Values

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{collections::BTreeMap, sync::Arc};

// Third Party Imports
use arrow_array::{
    Array, ArrayRef, BooleanArray, Float64Array, Int64Array, ListArray, RecordBatch, StringArray, StructArray, TimestampMicrosecondArray,
    UInt64Array,
};
use arrow_buffer::{NullBuffer, OffsetBuffer};
use arrow_schema::{DataType, SchemaRef, TimeUnit};

// Local Imports
use crate::storage::{Error, Result, telemetry::TelemetryEntry};
use dokime_domain::model::{
    identity::{Identifier, SessionId, TelemetrySampleId},
    time::Timestamp,
    values::Value,
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) fn normalize_values(field: &arrow_schema::Field, values: &mut BTreeMap<String, Value>) -> Result<()> {
    fn normalize(field: &arrow_schema::Field, value: &mut Value) -> Result<()> {
        if matches!(value, Value::Null) {
            return Ok(());
        }
        match field.data_type() {
            DataType::Timestamp(TimeUnit::Microsecond, _) => {
                let time = match value {
                    Value::Datetime(time) => *time,
                    Value::Text(text) => Timestamp::new(text.as_str()).to_datetime()?,
                    _ => return Err(Error::Invalid("telemetry timestamp differs from Arrow contract")),
                };
                *value = Value::Datetime(
                    time::OffsetDateTime::from_unix_timestamp_nanos(time.unix_timestamp_nanos().div_euclid(1000) * 1000)
                        .map_err(|_| Error::Invalid("telemetry timestamp exceeds Arrow range"))?,
                );
            }
            DataType::Utf8 if field.metadata().get("encoding").is_some_and(|encoding| encoding == "json") => {
                if let Value::Text(text) = value {
                    *value =
                        Value::from_plain_json(serde_json::from_str(text)?).map_err(|_| Error::Invalid("invalid JSON telemetry object"))?;
                }
                if !matches!(value, Value::Object(_)) {
                    return Err(Error::Invalid("telemetry JSON field is not an object"));
                }
            }
            DataType::Utf8 => {
                // Native Timestamp Projections Can Be Datetimes Even When The Contract Declares String.
                if let Value::Datetime(time) = value {
                    *value = Value::Text(Timestamp::from_datetime(*time).to_string());
                }
            }
            DataType::Struct(fields) => {
                if let Value::Object(values) = value {
                    for field in fields {
                        if let Some(value) = values.get_mut(field.name()) {
                            normalize(field, value)?;
                        }
                    }
                }
            }
            DataType::List(field) => {
                if let Value::Array(values) = value {
                    for value in values {
                        normalize(field, value)?;
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
    let DataType::Struct(fields) = field.data_type() else {
        return Err(Error::Invalid("telemetry values are not a struct"));
    };
    for field in fields {
        if let Some(value) = values.get_mut(field.name()) {
            normalize(field, value)?;
        }
    }
    Ok(())
}

pub(crate) fn batch(schema: SchemaRef, rows: &[(u64, crate::storage::telemetry::Admitted)]) -> Result<RecordBatch> {
    let times = rows
        .iter()
        .map(|(_, entry)| {
            i64::try_from(entry.sample.timestamp.to_datetime()?.unix_timestamp_nanos().div_euclid(1000))
                .map_err(|_| Error::Invalid("telemetry timestamp exceeds Arrow range"))
        })
        .collect::<Result<Vec<_>>>()?;
    batch_at(schema, rows, &times)
}

// Construct Columns Directly From Borrowed Admissions, Without Per-Row Metadata Or Value Maps.
pub(crate) fn batch_at(schema: SchemaRef, rows: &[(u64, crate::storage::telemetry::Admitted)], times: &[i64]) -> Result<RecordBatch> {
    if rows.len() != times.len() {
        return Err(Error::Invalid("telemetry timestamp count differs"));
    }
    let DataType::Struct(metadata_fields) = schema.field(0).data_type() else {
        return Err(Error::Invalid("telemetry metadata is not a struct"));
    };
    let DataType::Struct(value_fields) = schema.field(1).data_type() else {
        return Err(Error::Invalid("telemetry values are not a struct"));
    };
    macro_rules! text {
        ($value:expr) => {
            Arc::new(StringArray::from_iter_values(rows.iter().map($value))) as ArrayRef
        };
    }
    let mut metadata = Vec::<ArrayRef>::with_capacity(metadata_fields.len());
    for field in metadata_fields {
        let column: ArrayRef = match field.name().as_str() {
            "position" => Arc::new(UInt64Array::from_iter_values(rows.iter().map(|(position, _)| *position))),
            "at" => Arc::new(TimestampMicrosecondArray::from_iter_values(times.iter().copied()).with_timezone("UTC")),
            "ordinal" => Arc::new(UInt64Array::from_iter_values(rows.iter().map(|(_, entry)| entry.ordinal))),
            "sequence" => Arc::new(UInt64Array::from_iter_values(rows.iter().map(|(_, entry)| entry.sample.sequence))),
            "epoch" => text!(|(_, entry)| entry.epoch.as_str()),
            "sample_id" => text!(|(_, entry)| entry.sample.sample_id.as_str()),
            "timestamp" => text!(|(_, entry)| entry.sample.timestamp.as_str()),
            "received_at" => text!(|(_, entry)| entry.sample.received_at.as_str()),
            "source" => text!(|(_, entry)| entry.sample.source.as_str()),
            "native_metadata" => text!(|(_, entry)| entry.metadata.as_str()),
            "scope" => json_column(rows.iter().map(|(_, entry)| &entry.sample.scope))?,
            "summary" => json_column(rows.iter().map(|(_, entry)| &entry.sample.summary))?,
            "metadata" => json_column(rows.iter().map(|(_, entry)| &entry.sample.metadata))?,
            "original_strings" => {
                let strings = rows
                    .iter()
                    .map(|(_, entry)| {
                        serde_json::to_string(
                            &entry
                                .strings
                                .iter()
                                .filter(|(path, _)| path.as_slice() == ["timestamp"] || changes_text(schema.field(1).data_type(), path))
                                .collect::<Vec<_>>(),
                        )
                    })
                    .collect::<std::result::Result<Vec<_>, _>>()?;
                Arc::new(StringArray::from(strings))
            }
            _ => return Err(Error::Invalid("unknown telemetry metadata column")),
        };
        metadata.push(column);
    }
    let values = value_fields
        .iter()
        .map(|field| {
            let values: Vec<_> = rows.iter().map(|(_, entry)| entry.sample.values.get(field.name()).unwrap_or(&Value::Null)).collect();
            array(field.data_type(), &values)
        })
        .collect::<Result<Vec<_>>>()?;
    let values = if value_fields.is_empty() {
        StructArray::new_empty_fields(rows.len(), None)
    } else {
        StructArray::new(value_fields.clone(), values, None)
    };
    Ok(RecordBatch::try_new(schema.clone(), vec![Arc::new(StructArray::new(metadata_fields.clone(), metadata, None)), Arc::new(values)])?)
}

pub(crate) fn entry(
    batch: &RecordBatch, row: usize, contract: &dokime_domain::model::telemetry::TelemetryContract,
) -> Result<(u64, TelemetryEntry)> {
    let storage = batch.column(0).as_any().downcast_ref::<StructArray>().ok_or(Error::Corrupt("telemetry metadata is not a struct"))?;
    let number = |name| -> Result<u64> {
        let column = storage
            .column_by_name(name)
            .and_then(|column| column.as_any().downcast_ref::<UInt64Array>())
            .ok_or(Error::Corrupt("telemetry numeric metadata is absent"))?;
        if column.is_null(row) {
            return Err(Error::Corrupt("telemetry numeric metadata is null"));
        }
        Ok(column.value(row))
    };
    let text = |name| -> Result<&str> {
        let column = storage
            .column_by_name(name)
            .and_then(|column| column.as_any().downcast_ref::<StringArray>())
            .ok_or(Error::Corrupt("telemetry text metadata is absent"))?;
        if column.is_null(row) {
            return Err(Error::Corrupt("telemetry text metadata is null"));
        }
        Ok(column.value(row))
    };
    let Value::Object(values) = cell(batch.column(1), row)? else {
        return Err(Error::Corrupt("telemetry values are not a struct"));
    };
    let sample = crate::telemetry::TelemetrySample {
        captured_scopes: Vec::new(),
        sample_id: TelemetrySampleId::from_uuid(text("sample_id")?)?,
        stream: Identifier::new(contract.stream.as_str())?,
        schema_version: contract.schema_version,
        sequence: number("sequence")?,
        timestamp: Timestamp::new(text("timestamp")?),
        received_at: Timestamp::new(text("received_at")?),
        source: text("source")?.to_owned(),
        scope: serde_json::from_str(text("scope")?)?,
        values,
        summary: serde_json::from_str(text("summary")?)?,
        metadata: serde_json::from_str(text("metadata")?)?,
        storage_payload: None,
    };
    Ok((
        number("position")?,
        TelemetryEntry {
            epoch: SessionId::from_uuid(text("epoch")?)?,
            ordinal: number("ordinal")?,
            sample,
        },
    ))
}

pub(crate) fn row_key(batch: &RecordBatch, row: usize) -> Result<(u64, i64)> {
    let (positions, times) = row_keys(batch)?;
    Ok((positions.value(row), times.value(row)))
}

pub(crate) fn row_keys(batch: &RecordBatch) -> Result<(&UInt64Array, &TimestampMicrosecondArray)> {
    let storage = batch.column(0).as_any().downcast_ref::<StructArray>().ok_or(Error::Corrupt("telemetry metadata is not a struct"))?;
    let positions = storage
        .column_by_name("position")
        .and_then(|array| array.as_any().downcast_ref::<UInt64Array>())
        .ok_or(Error::Corrupt("telemetry position is absent"))?;
    let times = storage
        .column_by_name("at")
        .and_then(|array| array.as_any().downcast_ref::<TimestampMicrosecondArray>())
        .ok_or(Error::Corrupt("telemetry timestamp is absent"))?;
    Ok((positions, times))
}

pub(crate) fn native_encoding(batch: &RecordBatch, row: usize) -> Result<(&str, &str)> {
    Ok((metadata_text(batch, row, "native_metadata")?, metadata_text(batch, row, "original_strings")?))
}

pub(crate) fn metadata_text<'a>(batch: &'a RecordBatch, row: usize, name: &str) -> Result<&'a str> {
    let storage = batch.column(0).as_any().downcast_ref::<StructArray>().ok_or(Error::Corrupt("telemetry metadata is not a struct"))?;
    let array = storage
        .column_by_name(name)
        .and_then(|column| column.as_any().downcast_ref::<StringArray>())
        .ok_or(Error::Corrupt("telemetry native metadata is absent"))?;
    if array.is_null(row) {
        return Err(Error::Corrupt("telemetry native metadata is null"));
    }
    Ok(array.value(row))
}

pub(crate) fn array(kind: &DataType, values: &[&Value]) -> Result<ArrayRef> {
    let validity = || Some(NullBuffer::from(values.iter().map(|value| !matches!(value, Value::Null)).collect::<Vec<_>>()));
    let mismatch = || Error::Invalid("telemetry value differs from Arrow contract");
    macro_rules! primitive {
        ($array:ty, $pattern:pat => $value:expr) => {{
            let items = values
                .iter()
                .map(|value| match value {
                    Value::Null => Ok(None),
                    $pattern => Ok(Some($value)),
                    _ => Err(mismatch()),
                })
                .collect::<Result<Vec<_>>>()?;
            Arc::new(<$array>::from(items)) as ArrayRef
        }};
    }
    Ok(match kind {
        DataType::Boolean => primitive!(BooleanArray, Value::Bool(value) => *value),
        DataType::Int64 => primitive!(Int64Array, Value::Integer(value) => *value),
        DataType::UInt64 => {
            let items = values
                .iter()
                .map(|value| match value {
                    Value::Null => Ok(None),
                    Value::Unsigned(value) => Ok(Some(*value)),
                    Value::Integer(value) if *value >= 0 => Ok(Some(*value as u64)),
                    _ => Err(mismatch()),
                })
                .collect::<Result<Vec<_>>>()?;
            Arc::new(UInt64Array::from(items))
        }
        DataType::Float64 => {
            let items = values
                .iter()
                .map(|value| match value {
                    Value::Null => Ok(None),
                    Value::Number(value) => Ok(Some(*value)),
                    Value::Integer(value) => Ok(Some(*value as f64)),
                    Value::Unsigned(value) => Ok(Some(*value as f64)),
                    _ => Err(mismatch()),
                })
                .collect::<Result<Vec<_>>>()?;
            Arc::new(Float64Array::from(items))
        }
        DataType::Utf8 => {
            let items = values
                .iter()
                .map(|value| match value {
                    Value::Null => Ok(None),
                    Value::Text(value) => Ok(Some(value.clone())),
                    Value::Object(_) => Ok(Some(serde_json::to_string(&value.to_plain_json().map_err(|_| mismatch())?)?)),
                    _ => Err(mismatch()),
                })
                .collect::<Result<Vec<_>>>()?;
            Arc::new(StringArray::from(items))
        }
        DataType::Timestamp(TimeUnit::Microsecond, _) => {
            let items = values
                .iter()
                .map(|value| {
                    let timestamp = match value {
                        Value::Null => return Ok(None),
                        Value::Datetime(value) => *value,
                        Value::Text(value) => Timestamp::new(value).to_datetime()?,
                        _ => return Err(mismatch()),
                    };
                    let micros = i64::try_from(timestamp.unix_timestamp_nanos().div_euclid(1000)).map_err(|_| mismatch())?;
                    Ok(Some(micros))
                })
                .collect::<Result<Vec<_>>>()?;
            Arc::new(TimestampMicrosecondArray::from(items).with_timezone("UTC"))
        }
        DataType::List(field) => {
            let mut lengths = Vec::with_capacity(values.len());
            let mut children = Vec::new();
            for value in values {
                match value {
                    Value::Array(items) => {
                        lengths.push(items.len());
                        children.extend(items.iter());
                    }
                    Value::Null => lengths.push(0),
                    _ => return Err(mismatch()),
                }
            }
            let children = array(field.data_type(), &children)?;
            Arc::new(ListArray::new(field.clone(), OffsetBuffer::from_lengths(lengths), children, validity()))
        }
        DataType::Struct(fields) => {
            let children = fields
                .iter()
                .map(|field| {
                    let items = values
                        .iter()
                        .map(|value| match value {
                            Value::Object(fields) => Ok(fields.get(field.name()).unwrap_or(&Value::Null)),
                            Value::Null => Ok(&Value::Null),
                            _ => Err(mismatch()),
                        })
                        .collect::<Result<Vec<_>>>()?;
                    array(field.data_type(), &items)
                })
                .collect::<Result<Vec<_>>>()?;
            Arc::new(StructArray::new(fields.clone(), children, validity()))
        }
        _ => return Err(Error::Invalid("unsupported Arrow telemetry type")),
    })
}

pub(crate) fn cell(array: &ArrayRef, row: usize) -> Result<Value> {
    if array.is_null(row) {
        return Ok(Value::Null);
    }
    macro_rules! scalar {
        ($array:ty, $variant:ident) => {
            Value::$variant(array.as_any().downcast_ref::<$array>().ok_or(Error::Corrupt("Arrow scalar type differs"))?.value(row))
        };
    }
    Ok(match array.data_type() {
        DataType::Boolean => scalar!(BooleanArray, Bool),
        DataType::Int64 => scalar!(Int64Array, Integer),
        DataType::UInt64 => scalar!(UInt64Array, Unsigned),
        DataType::Float64 => scalar!(Float64Array, Number),
        DataType::Utf8 => Value::Text(
            array.as_any().downcast_ref::<StringArray>().ok_or(Error::Corrupt("Arrow text type differs"))?.value(row).to_owned(),
        ),
        DataType::Timestamp(TimeUnit::Microsecond, _) => {
            let micros = array
                .as_any()
                .downcast_ref::<TimestampMicrosecondArray>()
                .ok_or(Error::Corrupt("Arrow timestamp type differs"))?
                .value(row);
            Value::Datetime(
                time::OffsetDateTime::from_unix_timestamp_nanos(i128::from(micros) * 1000)
                    .map_err(|_| Error::Corrupt("Arrow timestamp is outside range"))?,
            )
        }
        DataType::List(field) => {
            let items = array.as_any().downcast_ref::<ListArray>().ok_or(Error::Corrupt("Arrow list type differs"))?.value(row);
            Value::Array((0..items.len()).map(|row| field_cell(field, &items, row)).collect::<Result<Vec<_>>>()?)
        }
        DataType::Struct(fields) => {
            let structure = array.as_any().downcast_ref::<StructArray>().ok_or(Error::Corrupt("Arrow struct type differs"))?;
            let fields = fields
                .iter()
                .zip(structure.columns())
                .map(|(field, column)| Ok((field.name().clone(), field_cell(field, column, row)?)))
                .collect::<Result<BTreeMap<_, _>>>()?;
            Value::Object(fields)
        }
        _ => return Err(Error::Corrupt("unsupported Arrow telemetry type")),
    })
}

/* === Private Functions === */

fn json_column<'a, T: serde::Serialize + PartialEq + 'a>(values: impl IntoIterator<Item = &'a T>) -> Result<ArrayRef> {
    let mut column = arrow_array::builder::StringBuilder::new();
    let mut previous = None;
    let mut encoded = String::new();
    for value in values {
        if previous != Some(value) {
            encoded = serde_json::to_string(value)?;
            previous = Some(value);
        }
        column.append_value(&encoded);
    }
    Ok(Arc::new(column.finish()))
}

fn field_cell(field: &arrow_schema::Field, array: &ArrayRef, row: usize) -> Result<Value> {
    let mut value = cell(array, row)?;
    if field.metadata().get("encoding").is_some_and(|encoding| encoding == "json")
        && let Value::Text(text) = value
    {
        value = Value::from_plain_json(serde_json::from_str(&text)?).map_err(|_| Error::Corrupt("invalid JSON telemetry object"))?;
    }
    Ok(value)
}

fn changes_text(kind: &DataType, path: &[String]) -> bool {
    let Some((name, rest)) = path.split_first() else {
        return matches!(kind, DataType::Timestamp(_, _));
    };
    match kind {
        DataType::Struct(fields) => fields.iter().find(|field| field.name() == name).is_some_and(|field| {
            rest.is_empty() && field.metadata().get("encoding").is_some_and(|encoding| encoding == "json")
                || changes_text(field.data_type(), rest)
        }),
        DataType::List(field) => changes_text(field.data_type(), rest),
        _ => false,
    }
}

/* === Tests === */
