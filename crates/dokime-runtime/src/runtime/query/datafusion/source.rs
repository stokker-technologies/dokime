/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Catalog And IPC Scan Adapter With Lossless Predicate Columns

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::{Arc, Mutex};

// Third Party Imports
use arrow_array::{
    Array, ArrayRef, BooleanArray, Decimal128Array, Float64Array, Int64Array, RecordBatch, StringArray, StructArray,
    TimestampMicrosecondArray, UInt8Array, UInt32Array, UInt64Array, new_null_array,
};
use arrow_schema::{DataType, Field, Fields, Schema};
use harmos_storage::query::datafusion::{
    IpcTable,
    engine::{
        dataframe::DataFrame,
        error::{DataFusionError, Result},
        logical_expr::{ColumnarValue, Expr, Volatility, create_udf},
        physical_plan::stream::RecordBatchStreamAdapter,
        prelude::col,
    },
};

// Local Imports
use super::super::{
    samples::Plan,
    selection::{Envelope, FieldRef},
};
use crate::{
    config::QueryConfig,
    storage::sqlite::{QueryWork, State},
};
use dokime_domain::model::{
    query::error::{Failure, Reason},
    telemetry::TelemetryContract,
    values::{Value, numeric_value},
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Default)]
pub(super) struct Meter {
    pub rows: u64,
    pub bytes: u64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) async fn frame(
    local: &State, plan: &Plan, config: &QueryConfig, operation: &'static str,
) -> Result<(DataFrame, Arc<Mutex<Meter>>)> {
    let mut fields = vec![
        Field::new("position", DataType::UInt64, false),
        Field::new("time", DataType::Decimal128(38, 0), false),
        Field::new("source", DataType::Utf8, false),
        Field::new("version", DataType::UInt32, false),
    ];
    let required = plan.selection.required_fields().to_vec();
    fields.extend((0..required.len()).map(|index| Field::new(format!("v{index}"), tagged_type(), false)));
    let schema = Arc::new(Schema::new(fields));
    let read = plan.read.clone();
    let local = local.clone();
    let config = config.clone();
    let meter = Arc::new(Mutex::new(Meter::default()));
    let accounting = meter.clone();
    let input_schema = schema.clone();
    let context = harmos_storage::query::datafusion::context(config.limits().working_bytes as usize / 2)?;
    let open: harmos_storage::query::datafusion::Open = Arc::new(move || {
        let (sender, receiver) = tokio::sync::mpsc::channel(1);
        let (local, read, config, required, schema, accounting) =
            (local.clone(), read.clone(), config.clone(), required.clone(), input_schema.clone(), accounting.clone());
        tokio::spawn(async move {
            let sending = sender.clone();
            let result = local
                .query_native_batches(read, QueryWork::new(config, operation), move |input, contract| {
                    for offset in (0..input.num_rows()).step_by(1024) {
                        let input = input.slice(offset, (input.num_rows() - offset).min(1024));
                        let batch =
                            batch(&input, &contract, schema.clone(), &required).map_err(|error| super::failure(error, operation))?;
                        sending.blocking_send(Ok(batch)).map_err(|_| Failure::new(Reason::Deadline, operation, "Query consumer closed"))?;
                    }
                    Ok(())
                })
                .await;
            match result {
                Ok(work) => {
                    let mut meter = accounting.lock().expect("query meter");
                    meter.rows += work.rows;
                    meter.bytes += work.bytes;
                }
                Err(error) => {
                    let _ = sender.send(Err(DataFusionError::External(error))).await;
                }
            }
        });
        let stream =
            futures_util::stream::unfold(receiver, |mut receiver| async move { receiver.recv().await.map(|item| (item, receiver)) });
        Box::pin(RecordBatchStreamAdapter::new(input_schema.clone(), stream)) as _
    });
    let mut frame = context.read_table(Arc::new(IpcTable::new(schema, open)))?;
    for predicate in plan.selection.datafusion_filters() {
        frame = frame.filter(predicate)?;
    }
    Ok((frame, meter))
}

pub(crate) fn predicate(index: usize, name: &str, matches: impl Fn(Option<&Value>) -> bool + Send + Sync + 'static) -> Expr {
    let function = create_udf(
        name,
        vec![tagged_type()],
        DataType::Boolean,
        Volatility::Immutable,
        Arc::new(move |arguments| {
            let arrays = ColumnarValue::values_to_arrays(arguments)?;
            let array = arrays[0].as_any().downcast_ref::<StructArray>().expect("tagged predicate");
            let values = (0..array.len()).map(|row| matches(value(array, row).as_ref())).collect::<Vec<_>>();
            Ok(ColumnarValue::Array(Arc::new(BooleanArray::from(values))))
        }),
    );
    function.call(vec![col(format!("v{index}"))])
}

pub(super) fn numeric(index: usize) -> Expr {
    let function = create_udf(
        "spike_numeric",
        vec![tagged_type()],
        DataType::Float64,
        Volatility::Immutable,
        Arc::new(|arguments| {
            let arrays = ColumnarValue::values_to_arrays(arguments)?;
            let array = arrays[0].as_any().downcast_ref::<StructArray>().expect("tagged numeric");
            let values = (0..array.len()).map(|row| value(array, row).as_ref().and_then(numeric_value)).collect::<Vec<_>>();
            Ok(ColumnarValue::Array(Arc::new(Float64Array::from(values))))
        }),
    );
    function.call(vec![col(format!("v{index}"))])
}

pub(super) fn quality(index: usize) -> Expr {
    let function = create_udf(
        "spike_quality",
        vec![tagged_type()],
        DataType::UInt8,
        Volatility::Immutable,
        Arc::new(|arguments| {
            let arrays = ColumnarValue::values_to_arrays(arguments)?;
            let array = arrays[0].as_any().downcast_ref::<StructArray>().expect("tagged quality");
            let values = (0..array.len())
                .map(|row| match value(array, row) {
                    None => 0,
                    Some(Value::Null) => 1,
                    Some(value) if numeric_value(&value).is_some() => 3,
                    _ => 2,
                })
                .collect::<Vec<u8>>();
            Ok(ColumnarValue::Array(Arc::new(UInt8Array::from(values))))
        }),
    );
    function.call(vec![col(format!("v{index}"))])
}

/* === Private Functions === */

fn tagged_fields() -> Fields {
    vec![
        Field::new("kind", DataType::UInt8, false),
        Field::new("signed", DataType::Int64, true),
        Field::new("unsigned", DataType::UInt64, true),
        Field::new("number", DataType::Float64, true),
        Field::new("text", DataType::Utf8, true),
        Field::new("boolean", DataType::Boolean, true),
        Field::new("time", DataType::Decimal128(38, 0), true),
    ]
    .into()
}
fn tagged_type() -> DataType {
    DataType::Struct(tagged_fields())
}

fn batch(input: &RecordBatch, contract: &TelemetryContract, schema: Arc<Schema>, required: &[FieldRef]) -> Result<RecordBatch> {
    use crate::storage::ipc;
    use dokime_domain::model::time::Timestamp;
    let metadata = input.column(0).as_any().downcast_ref::<StructArray>().expect("IPC metadata");
    let values = input.column(1).as_any().downcast_ref::<StructArray>().expect("IPC values");
    let times = (0..input.num_rows())
        .map(|row| {
            let text = ipc::metadata_text(input, row, "timestamp").map_err(|error| DataFusionError::External(Box::new(error)))?;
            Timestamp::new(text)
                .to_datetime()
                .map(|time| time.unix_timestamp_nanos())
                .map_err(|error| DataFusionError::External(Box::new(error)))
        })
        .collect::<Result<Vec<_>>>()?;
    let time: ArrayRef = Arc::new(Decimal128Array::from(times).with_precision_and_scale(38, 0)?);
    let version: ArrayRef = Arc::new(UInt64Array::from(vec![u64::from(contract.schema_version); input.num_rows()]));
    let mut arrays: Vec<ArrayRef> = vec![
        metadata.column_by_name("position").expect("position").clone(),
        time.clone(),
        metadata.column_by_name("source").expect("source").clone(),
        Arc::new(UInt32Array::from(vec![contract.schema_version; input.num_rows()])),
    ];
    for field in required {
        let received;
        let column = match field {
            FieldRef::Declared(id) => {
                if contract.columns.iter().any(|column| column.metadata.identifier.as_str() == id) {
                    values.column_by_name(id)
                } else {
                    None
                }
            }
            FieldRef::Envelope(envelope) => match envelope {
                Envelope::Id => metadata.column_by_name("sample_id"),
                Envelope::Timestamp => Some(&time),
                Envelope::ReceivedAt => {
                    let times = (0..input.num_rows())
                        .map(|row| {
                            let text = ipc::metadata_text(input, row, "received_at")
                                .map_err(|error| DataFusionError::External(Box::new(error)))?;
                            Timestamp::new(text)
                                .to_datetime()
                                .map(|time| time.unix_timestamp_nanos())
                                .map_err(|error| DataFusionError::External(Box::new(error)))
                        })
                        .collect::<Result<Vec<_>>>()?;
                    received = Arc::new(Decimal128Array::from(times).with_precision_and_scale(38, 0)?) as ArrayRef;
                    Some(&received)
                }
                Envelope::Source => metadata.column_by_name("source"),
                Envelope::SchemaVersion => Some(&version),
                Envelope::Severity => None,
            },
        };
        arrays.push(tagged(column, input.num_rows())?);
    }
    Ok(RecordBatch::try_new(schema, arrays)?)
}

fn tagged(column: Option<&ArrayRef>, rows: usize) -> Result<ArrayRef> {
    use harmos_storage::query::datafusion::engine::arrow::compute::cast;
    let mut columns = tagged_fields().iter().map(|field| new_null_array(field.data_type(), rows)).collect::<Vec<_>>();
    let kind = if let Some(column) = column {
        let (kind, slot, target) = match column.data_type() {
            DataType::Int8 | DataType::Int16 | DataType::Int32 | DataType::Int64 => (2, 1, DataType::Int64),
            DataType::UInt8 | DataType::UInt16 | DataType::UInt32 | DataType::UInt64 => (3, 2, DataType::UInt64),
            DataType::Float32 | DataType::Float64 => (4, 3, DataType::Float64),
            DataType::Utf8 => (5, 4, DataType::Utf8),
            DataType::Boolean => (6, 5, DataType::Boolean),
            DataType::Decimal128(38, 0) => (7, 6, DataType::Decimal128(38, 0)),
            DataType::Timestamp(_, _) => (7, 6, DataType::Decimal128(38, 0)),
            _ => (8, 0, DataType::UInt8),
        };
        if kind != 8 {
            columns[slot] = if let Some(times) = column.as_any().downcast_ref::<TimestampMicrosecondArray>() {
                Arc::new(
                    Decimal128Array::from(times.iter().map(|value| value.map(|value| i128::from(value) * 1000)).collect::<Vec<_>>())
                        .with_precision_and_scale(38, 0)?,
                )
            } else {
                cast(column, &target)?
            };
        }
        (0..rows).map(|row| if column.is_null(row) { 1 } else { kind }).collect::<Vec<u8>>()
    } else {
        vec![0u8; rows]
    };
    columns[0] = Arc::new(UInt8Array::from(kind));
    Ok(Arc::new(StructArray::new(tagged_fields(), columns, None)))
}

fn value(array: &StructArray, row: usize) -> Option<Value> {
    let kind = array.column(0).as_any().downcast_ref::<UInt8Array>().expect("kind").value(row);
    Some(match kind {
        0 => return None,
        1 => Value::Null,
        2 => Value::Integer(array.column(1).as_any().downcast_ref::<Int64Array>().expect("signed").value(row)),
        3 => Value::Unsigned(array.column(2).as_any().downcast_ref::<UInt64Array>().expect("unsigned").value(row)),
        4 => Value::Number(array.column(3).as_any().downcast_ref::<Float64Array>().expect("number").value(row)),
        5 => Value::Text(array.column(4).as_any().downcast_ref::<StringArray>().expect("text").value(row).to_owned()),
        6 => Value::Bool(array.column(5).as_any().downcast_ref::<BooleanArray>().expect("boolean").value(row)),
        7 => Value::Datetime(
            time::OffsetDateTime::from_unix_timestamp_nanos(
                array.column(6).as_any().downcast_ref::<Decimal128Array>().expect("time").value(row),
            )
            .expect("validated timestamp"),
        ),
        _ => Value::Array(Vec::new()),
    })
}

/* === Tests === */
