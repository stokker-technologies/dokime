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

pub(crate) fn batch(schema: SchemaRef, rows: &[(u64, TelemetryEntry)]) -> Result<RecordBatch> {
    let mut metadata = Vec::with_capacity(rows.len());
    let mut values = Vec::with_capacity(rows.len());
    for (position, entry) in rows {
        let sample = &entry.sample;
        metadata.push(Value::Object(BTreeMap::from([
            ("position".into(), Value::Unsigned(*position)),
            ("epoch".into(), Value::Text(entry.epoch.to_string())),
            ("ordinal".into(), Value::Unsigned(entry.ordinal)),
            ("sample_id".into(), Value::Text(sample.sample_id.to_string())),
            ("sequence".into(), Value::Unsigned(sample.sequence)),
            ("timestamp".into(), Value::Text(sample.timestamp.to_string())),
            ("received_at".into(), Value::Text(sample.received_at.to_string())),
            ("source".into(), Value::Text(sample.source.clone())),
            ("scope".into(), Value::Text(serde_json::to_string(&sample.scope)?)),
            ("summary".into(), Value::Text(serde_json::to_string(&sample.summary)?)),
            ("metadata".into(), Value::Text(serde_json::to_string(&sample.metadata)?)),
        ])));
        values.push(Value::Object(sample.values.clone()));
    }
    let columns = vec![
        array(schema.field(0).data_type(), &metadata.iter().collect::<Vec<_>>())?,
        array(schema.field(1).data_type(), &values.iter().collect::<Vec<_>>())?,
    ];
    Ok(RecordBatch::try_new(schema, columns)?)
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
                    Value::Object(_) => Ok(Some(serde_json::to_string(value)?)),
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
        DataType::List(_) => {
            let items = array.as_any().downcast_ref::<ListArray>().ok_or(Error::Corrupt("Arrow list type differs"))?.value(row);
            Value::Array((0..items.len()).map(|row| cell(&items, row)).collect::<Result<Vec<_>>>()?)
        }
        DataType::Struct(fields) => {
            let structure = array.as_any().downcast_ref::<StructArray>().ok_or(Error::Corrupt("Arrow struct type differs"))?;
            let fields = fields
                .iter()
                .zip(structure.columns())
                .map(|(field, column)| Ok((field.name().clone(), cell(column, row)?)))
                .collect::<Result<BTreeMap<_, _>>>()?;
            Value::Object(fields)
        }
        _ => return Err(Error::Corrupt("unsupported Arrow telemetry type")),
    })
}

/* === Private Functions === */

/* === Tests === */
