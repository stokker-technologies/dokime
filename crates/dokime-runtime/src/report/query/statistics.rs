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
use std::sync::Arc;

// Third Party Imports
use arrow_array::{Float64Array, RecordBatch};
use arrow_schema::{DataType, Field, Schema};
use harmos::storage::{
    query::scalar::ScalarValue,
    query::transform::{
        AggregateExecution,
        aggregate::{Aggregate as Reduction, Measure},
    },
    query::transform::{
        FieldPath, FieldPathExecution,
        histogram::{Histogram, HistogramExecution},
    },
    query::{Accumulator, Scratch},
};
use harmos::{
    Aggregate,
    schema::{Cardinality, Validator, graph::Kind},
    storage::query::scalar::Agg,
};

// Local Imports
use super::{ReadContext, invalid};
use crate::telemetry::{TelemetryContract, TelemetrySample};
use crate::{
    plot::{HistogramBin, HistogramSeries, HistogramStatistics, PlotSeriesSummary, PlotWindow},
    runtime::Result,
};
use dokime_domain::model::values::Value;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(super) struct Statistics {
    pub count: usize,
    pub rows: usize,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub mean: Option<f64>,
    pub median: Option<f64>,
    pub std: Option<f64>,
    pub rms: Option<f64>,
    pub window: Option<PlotWindow>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Statistics {
    pub fn summary(&self, key: String) -> PlotSeriesSummary {
        PlotSeriesSummary {
            series_id: key,
            count: self.count,
            min: self.min,
            max: self.max,
            mean: self.mean,
            median: self.median,
            std: self.std,
            rms: self.rms,
            window: self.window,
        }
    }
}

/* === Public Functions === */

pub(super) fn field(name: &str) -> FieldPath {
    FieldPath { fields: vec!["payload".into(), "values".into(), name.into()] }
}

pub(super) fn collect(
    read: ReadContext<'_, '_>, samples: &[TelemetrySample], contract: &TelemetryContract, keys: &[String], x: &FieldPath,
) -> Result<Vec<Statistics>> {
    let validator = validator(contract)?;
    for key in keys {
        validate_field(contract, key, true)?;
    }
    let source_time = x.fields == ["_storage", "source_time"];
    let x_key = x.fields.last().ok_or_else(|| invalid("report x field is absent"))?;
    if !source_time {
        validate_field(contract, x_key, false)?;
    }

    // Application-Local Reduction: One Series Owns Its Temporary Memory.
    let bytes = samples
        .len()
        .checked_mul(std::mem::size_of::<ScalarValue>() * 2 + std::mem::size_of::<usize>() * 2)
        .and_then(|bytes| bytes.checked_add(std::mem::size_of::<Accumulator>() * 9 + 65536))
        .ok_or_else(|| invalid("report reduction memory overflow"))?;
    let mut output = Vec::with_capacity(keys.len());
    for key in keys {
        let _memory = reserve(read, bytes)?;
        let mut accumulators =
            [Aggregate::Count, Aggregate::Min, Aggregate::Max, Aggregate::Mean, Aggregate::Median, Aggregate::Std, Aggregate::Rms]
                .map(|statistic| Accumulator::new(Agg::Of(statistic)));
        let mut minimum = Accumulator::new(Agg::Of(Aggregate::Min));
        let mut maximum = Accumulator::new(Agg::Of(Aggregate::Max));
        let mut scratch = Scratch::new(samples.len());
        for (capture, sample) in samples.iter().enumerate() {
            let payload = sample.storage_payload.as_deref().ok_or_else(|| invalid("report sample has no original typed payload"))?;
            let row = crate::storage::telemetry::decode_values(&validator, payload)?;
            let x_value = if source_time {
                ScalarValue::Datetime(
                    sample.timestamp.to_datetime().map_err(|_| invalid("captured report timestamp is invalid"))?.unix_timestamp_nanos(),
                )
            } else {
                scalar(row.get(x_key))?
            };
            minimum.push(capture as u64, &x_value, &mut scratch)?;
            maximum.push(capture as u64, &x_value, &mut scratch)?;
            let value = scalar(row.get(key))?;
            for accumulator in &mut accumulators {
                accumulator.push(capture as u64, &value, &mut scratch)?;
            }
        }
        // Like full(), This Synchronous Application Work Has No Cancellation Token.
        let mut check = || Ok(());
        let window = number(minimum.finish(&mut check)?).zip(number(maximum.finish(&mut check)?)).map(|(from, to)| PlotWindow { from, to });
        let [count, min, max, mean, median, std, rms] = accumulators;
        output.push(Statistics {
            count: count_value(count.finish(&mut check)?)?,
            rows: samples.len(),
            min: number(min.finish(&mut check)?),
            max: number(max.finish(&mut check)?),
            mean: number(mean.finish(&mut check)?),
            median: number(median.finish(&mut check)?),
            std: number(std.finish(&mut check)?),
            rms: number(rms.finish(&mut check)?),
            window,
        });
    }
    Ok(output)
}

pub(super) fn histogram(
    read: ReadContext<'_, '_>, samples: &[TelemetrySample], contract: &TelemetryContract, key: &str, bins: usize, statistics: &Statistics,
) -> Result<HistogramSeries> {
    let count = bins.max(1);
    if count >= 65_536 {
        return Err(invalid("histogram bin count exceeds storage transform limit"));
    }
    validate_field(contract, key, true)?;
    let bytes = samples
        .len()
        .checked_mul(std::mem::size_of::<Option<f64>>() * 2)
        .and_then(|bytes| count.checked_add(2)?.checked_mul(128)?.checked_add(bytes))
        .and_then(|bytes| bytes.checked_add(65536))
        .ok_or_else(|| invalid("report histogram memory overflow"))?;
    let _memory = reserve(read, bytes)?;
    let (minimum, maximum) = statistics.min.zip(statistics.max).unwrap_or((0.0, 0.0));
    let width = if minimum == maximum { 1.0 } else { (maximum - minimum) / count as f64 };
    let edges =
        (0..=count).map(|index| if index == count && minimum != maximum { maximum } else { minimum + index as f64 * width }).collect();
    let validator = validator(contract)?;
    let mut values = Vec::with_capacity(samples.len());
    for sample in samples {
        let payload = sample.storage_payload.as_deref().ok_or_else(|| invalid("report sample has no original typed payload"))?;
        let row = crate::storage::telemetry::decode_values(&validator, payload)?;
        values.push(number(scalar(row.get(key))?));
    }
    let schema = Arc::new(Schema::new(vec![Field::new("value", DataType::Float64, true)]));
    let input = RecordBatch::try_new(schema, vec![Arc::new(Float64Array::from(values))])
        .map_err(|_| invalid("captured histogram input is invalid"))?;
    let plan = Histogram { field: FieldPath { fields: vec!["value".into()] }, edges };
    let batch = plan.apply(&input, &mut || Ok(()))?;
    let mut output = Vec::with_capacity(count);
    let mut actual = 0usize;
    for row in 0..batch.num_rows() {
        let value = |name: &str| FieldPath { fields: vec![name.into()] }.value(&batch, row);
        let amount = count_value(value("count")?)?;
        let lower = number(value("lower")?);
        let upper = number(value("upper")?);
        let Some((lower, upper)) = lower.zip(upper) else {
            if amount != 0 {
                return Err(invalid("histogram edges omitted finite selected values"));
            }
            continue;
        };
        actual = actual.checked_add(amount).ok_or_else(|| invalid("histogram count overflow"))?;
        output.push(HistogramBin { lower, upper, count: amount });
    }
    if actual != statistics.count || output.len() != count {
        return Err(invalid("histogram does not cover the full selected dataset"));
    }
    let statistics = (|| {
        Some(HistogramStatistics {
            min: statistics.min?,
            max: statistics.max?,
            mean: statistics.mean?,
            median: statistics.median?,
            std: statistics.std?,
            rms: statistics.rms?,
        })
    })();
    Ok(HistogramSeries { key: key.into(), bins: output, count: actual, statistics })
}
pub(super) fn number(value: ScalarValue) -> Option<f64> {
    let number = match value {
        ScalarValue::Float(value) => value,
        ScalarValue::Signed(value) => value as f64,
        ScalarValue::Unsigned(value) => value as f64,
        ScalarValue::Datetime(value) => value as f64 / 1_000_000_000.0,
        ScalarValue::Exact(value) => (if value.negative { -1.0 } else { 1.0 }) * value.numerator as f64 / value.denominator.get() as f64,
        _ => return None,
    };
    number.is_finite().then_some(number)
}

/* === Private Functions === */

fn validator(contract: &TelemetryContract) -> Result<Validator> {
    let schema = contract.storage_schema.as_ref().ok_or_else(|| invalid("captured telemetry has no typed storage schema"))?;
    Validator::from_schema(schema, &harmos::config::SchemaSettings::default()).map_err(|_| invalid("captured telemetry graph is invalid"))
}

fn validate_field(contract: &TelemetryContract, key: &str, numeric: bool) -> Result<()> {
    let schema = contract.storage_schema.as_ref().ok_or_else(|| invalid("captured telemetry has no typed storage schema"))?;
    let field = schema
        .definitions
        .get(schema.root)
        .and_then(|fields| fields.iter().find(|field| field.name == key))
        .ok_or_else(|| invalid("report field is absent from the captured schema"))?;
    // Same Direct-Field Mapping As Harmos Query Source; Eligibility Stays In Its Kernel.
    let kind = match field.kind {
        Kind::Bool => DataType::Boolean,
        Kind::U32 => DataType::UInt32,
        Kind::U64 => DataType::UInt64,
        Kind::I32 => DataType::Int32,
        Kind::I64 => DataType::Int64,
        Kind::F32 => DataType::Float32,
        Kind::F64 => DataType::Float64,
        Kind::String => DataType::Utf8,
        Kind::Bytes => DataType::Binary,
        Kind::Message { .. } => DataType::Struct(Vec::<Field>::new().into()),
    };
    let kind = if field.cardinality == Cardinality::Repeated { DataType::List(Arc::new(Field::new("item", kind, false))) } else { kind };
    let input = Schema::new(vec![Field::new("value", kind, matches!(field.cardinality, Cardinality::Optional | Cardinality::Variant))]);
    let plan = Reduction {
        groups: vec![],
        measures: vec![Measure {
            name: "value".into(),
            field: FieldPath { fields: vec!["value".into()] },
            statistic: if numeric { Aggregate::Mean } else { Aggregate::Min },
        }],
    };
    plan.output_schema(&input)?;
    Ok(())
}

fn reserve(read: ReadContext<'_, '_>, bytes: usize) -> Result<harmos::storage::arrow::RetainedMemory> {
    if bytes > read.settings.budget.read_memory_bytes.get() {
        return Err(invalid("report reduction exceeds configured read memory"));
    }
    let storage = read.ctx.scope().storage::<crate::app::Dokime>().ok_or_else(|| invalid("report storage is unavailable"))?;
    Ok(storage.reserve_arrow_memory(bytes)?)
}

fn scalar(value: Option<&Value>) -> Result<ScalarValue> {
    Ok(match value {
        None => ScalarValue::Missing,
        Some(Value::Null) => ScalarValue::Null,
        Some(Value::Bool(value)) => ScalarValue::Bool(*value),
        Some(Value::Integer(value)) => ScalarValue::Signed(*value),
        Some(Value::Unsigned(value)) => ScalarValue::Unsigned(*value),
        Some(Value::Number(value)) => ScalarValue::Float(*value),
        Some(Value::Text(value)) => ScalarValue::Text(value.clone()),
        _ => return Err(invalid("report statistics require original scalar values")),
    })
}

fn count_value(value: ScalarValue) -> Result<usize> {
    if let ScalarValue::Unsigned(value) = value {
        usize::try_from(value).map_err(|_| invalid("aggregate count exceeds address space"))
    } else {
        Err(invalid("aggregate count has the wrong type"))
    }
}

/* === Tests === */
