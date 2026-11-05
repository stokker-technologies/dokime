/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Spike: DataFusion Sample Plans Behind Existing Query Contracts

/* === Modules === */

mod reduction;
mod source;

/* === Re-Exports === */

pub(super) use reduction::Reduced;
pub(super) use source::predicate;

/* === Imports === */

// Standard Library Imports
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{Arc, Mutex},
};

// Third Party Imports
use arrow_array::{Array, Decimal128Array, StringArray};
use arrow_schema::DataType;
use harmos_storage::query::datafusion::engine::{
    dataframe::DataFrame,
    error::DataFusionError,
    logical_expr::{ColumnarValue, Expr, Volatility, create_udf},
    prelude::{col, lit},
};

// Local Imports
use super::{samples::Plan, selection::FieldRef, tokens};
use crate::{
    config::QueryConfig,
    storage::sqlite::{QueryWork, State},
};
use dokime_domain::model::query::{
    error::{Failure, Reason},
    reduction::Output,
};
use reduction::{Admission, Factory};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(super) struct Population {
    pub rows: u64,
    pub fields: BTreeMap<String, Reduced>,
}

pub(super) struct Bucket {
    pub start: i128,
    pub partition: String,
    pub source: String,
    pub samples: u64,
    pub versions: BTreeSet<u32>,
    pub fields: Vec<Reduced>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) async fn statistics(
    local: &State, plan: &Plan, outputs: &[Output], config: &QueryConfig,
) -> std::result::Result<Population, Box<Failure>> {
    let operation = "dokime.query.aggregate_samples";
    let run = async {
        let (frame, meter) = source::frame(local, plan, config, operation).await?;
        let admission = Arc::new(Mutex::new(Admission::default()));
        let factory = Factory::new(config, operation, false, admission);
        let mut expressions = vec![
            factory.expression("spike_meta", vec![lit(0.0f64), lit(3u8), col("version"), col("source")], Vec::new(), true, 0).alias("meta"),
        ];
        let selected = outputs.iter().filter(|output| !output.field.is_empty()).map(|output| output.field.clone()).collect::<BTreeSet<_>>();
        let memory = config.limits().working_bytes as usize / 8 / selected.len().max(1);
        for (index, field) in selected.iter().enumerate() {
            let column = plan
                .selection
                .required_fields()
                .iter()
                .position(|candidate| *candidate == FieldRef::Declared(field.clone()))
                .expect("required output");
            let percentiles = outputs
                .iter()
                .filter(|output| output.field == *field)
                .filter_map(|output| match output.aggregate {
                    harmos::Aggregate::Median => Some(50),
                    harmos::Aggregate::Percentile(rank) if rank != 0 && rank != 100 => Some(rank),
                    _ => None,
                })
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect();
            let name = format!("spike_field_{index}");
            expressions.push(
                factory
                    .expression(
                        &name,
                        vec![source::numeric(column), source::quality(column), col("version"), col("source")],
                        percentiles,
                        false,
                        memory,
                    )
                    .alias(format!("field{index}")),
            );
        }
        let batches = frame.aggregate(Vec::<Expr>::new(), expressions)?.collect().await?;
        let mut result = Population { rows: 0, fields: BTreeMap::new() };
        for batch in batches {
            for row in 0..batch.num_rows() {
                result.rows = decode(&batch, "meta", row)?.samples;
                for (index, field) in selected.iter().enumerate() {
                    result.fields.insert(field.clone(), decode(&batch, &format!("field{index}"), row)?);
                }
            }
        }
        // Preserve The Existing Public Scan-Budget Accounting For Exact Interior Percentiles.
        let rescans = outputs.iter().any(|output| {
            matches!(output.aggregate, harmos::Aggregate::Median | harmos::Aggregate::Percentile(1..=99))
                && result.fields.get(&output.field).is_some_and(|field| field.count != 0)
        });
        let meter = meter.lock().expect("scan meter");
        let multiplier = if rescans { 9 } else { 1 };
        QueryWork::new(config.clone(), operation)
            .charge(meter.rows.saturating_mul(multiplier), meter.bytes.saturating_mul(multiplier))
            .map_err(|error| DataFusionError::External(error))?;
        Ok::<_, DataFusionError>(result)
    };
    run.await.map_err(|error| failure(error, operation))
}

pub(super) async fn buckets(
    local: &State, plan: &Plan, config: &QueryConfig, every: u64, execution: &str, stream: &str,
) -> std::result::Result<Vec<Bucket>, Box<Failure>> {
    let operation = "dokime.query.list_samples";
    let run = async {
        let (frame, _) = source::frame(local, plan, config, operation).await?;
        let frame = bucket_keys(frame, every, execution.to_owned(), stream.to_owned(), config.clone())?;
        let admission = Arc::new(Mutex::new(Admission::default()));
        let factory = Factory::new(config, operation, true, admission);
        let mut expressions = vec![
            factory.expression("spike_meta", vec![lit(0.0f64), lit(3u8), col("version"), col("source")], Vec::new(), true, 0).alias("meta"),
        ];
        for (index, field) in plan.selection.schema().fields.iter().enumerate() {
            let column = plan
                .selection
                .required_fields()
                .iter()
                .position(|candidate| *candidate == FieldRef::Declared(field.id.clone()))
                .expect("required bucket field");
            expressions.push(
                factory
                    .expression(
                        &format!("spike_field_{index}"),
                        vec![source::numeric(column), source::quality(column), col("version"), col("source")],
                        Vec::new(),
                        false,
                        0,
                    )
                    .alias(format!("field{index}")),
            );
        }
        let batches = frame
            .aggregate(vec![col("start"), col("partition"), col("source")], expressions)?
            .sort(vec![col("start").sort(true, false), col("partition").sort(true, false)])?
            .collect()
            .await?;
        let mut buckets = Vec::new();
        for batch in batches {
            let starts = batch.column_by_name("start").unwrap().as_any().downcast_ref::<Decimal128Array>().unwrap();
            let partitions = batch.column_by_name("partition").unwrap().as_any().downcast_ref::<StringArray>().unwrap();
            let sources = batch.column_by_name("source").unwrap().as_any().downcast_ref::<StringArray>().unwrap();
            for row in 0..batch.num_rows() {
                let meta = decode(&batch, "meta", row)?;
                let fields = (0..plan.selection.schema().fields.len())
                    .map(|index| decode(&batch, &format!("field{index}"), row))
                    .collect::<harmos_storage::query::datafusion::engine::error::Result<Vec<_>>>()?;
                buckets.push(Bucket {
                    start: starts.value(row),
                    partition: partitions.value(row).to_owned(),
                    source: sources.value(row).to_owned(),
                    samples: meta.samples,
                    versions: meta.versions,
                    fields,
                });
            }
        }
        Ok::<_, DataFusionError>(buckets)
    };
    run.await.map_err(|error| failure(error, operation))
}

pub(super) fn failure(error: DataFusionError, operation: &str) -> Box<Failure> {
    let mut current: &(dyn std::error::Error + 'static) = &error;
    loop {
        if let Some(failure) = current.downcast_ref::<Failure>() {
            return Box::new(failure.clone());
        }
        let Some(source) = current.source() else {
            break;
        };
        current = source;
    }
    tracing::warn!(%error, operation, "DataFusion query failed");
    Failure::new(Reason::ResourceLimit, operation, "DataFusion could not satisfy the query within its configured budget")
}

/* === Private Functions === */

fn decode(batch: &arrow_array::RecordBatch, column: &str, row: usize) -> harmos_storage::query::datafusion::engine::error::Result<Reduced> {
    let values = batch.column_by_name(column).expect("reduction output").as_any().downcast_ref::<StringArray>().expect("encoded reduction");
    serde_json::from_str(values.value(row)).map_err(|error| DataFusionError::External(Box::new(error)))
}

fn bucket_keys(
    frame: DataFrame, every: u64, execution: String, stream: String, config: QueryConfig,
) -> harmos_storage::query::datafusion::engine::error::Result<DataFrame> {
    let align = create_udf(
        "spike_epoch_bucket",
        vec![DataType::Decimal128(38, 0)],
        DataType::Decimal128(38, 0),
        Volatility::Immutable,
        Arc::new(move |arguments| {
            let arrays = ColumnarValue::values_to_arrays(arguments)?;
            let times = arrays[0].as_any().downcast_ref::<Decimal128Array>().expect("nanoseconds");
            let starts = times.values().iter().map(|time| super::buckets::aligned(*time, every)).collect::<Vec<_>>();
            Ok(ColumnarValue::Array(Arc::new(Decimal128Array::from(starts).with_precision_and_scale(38, 0)?)))
        }),
    );
    let partition = create_udf(
        "spike_partition",
        vec![DataType::Utf8],
        DataType::Utf8,
        Volatility::Immutable,
        Arc::new(move |arguments| {
            let arrays = ColumnarValue::values_to_arrays(arguments)?;
            let sources = arrays[0].as_any().downcast_ref::<StringArray>().expect("sources");
            let strings = sources
                .iter()
                .map(|source| {
                    tokens::bounded_json(
                        &[execution.as_str(), stream.as_str(), source.expect("source")],
                        config.limits().token_bytes,
                        "dokime.query.list_samples",
                    )
                    .map(|bytes| String::from_utf8(bytes).expect("partition UTF-8"))
                    .map_err(|error| DataFusionError::External(error))
                })
                .collect::<harmos_storage::query::datafusion::engine::error::Result<Vec<_>>>()?;
            Ok(ColumnarValue::Array(Arc::new(StringArray::from(strings))))
        }),
    );
    frame.with_column("start", align.call(vec![col("time")]))?.with_column("partition", partition.call(vec![col("source")]))
}

/* === Tests === */
