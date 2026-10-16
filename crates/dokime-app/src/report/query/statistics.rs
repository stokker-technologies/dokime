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
// Third Party Imports
use harmos::storage::{
    StorageCursor,
    query::transform::{
        FieldPath, Transform,
        aggregate::{Aggregate, Measure},
        histogram::Histogram,
    },
    reduce::ScalarValue,
};
// Local Imports
use super::{invalid, telemetry};
use crate::{
    Result,
    plot::{HistogramBin, HistogramSeries, HistogramStatistics, PlotSeriesSummary, PlotWindow},
};
use dokime::telemetry::TelemetryContract;

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
    FieldPath {
        fields: vec!["payload".into(), "values".into(), name.into()],
    }
}

pub(super) async fn collect(
    read: super::ReadContext<'_, '_>,
    at: &StorageCursor,
    scope: &str,
    contract: &TelemetryContract,
    key: &str,
    x: FieldPath,
) -> Result<Statistics> {
    let super::ReadContext {
        ctx,
        storage,
        settings,
    } = read;
    let codec = dokime::storage::telemetry::codec(contract)?;
    let mut measures = vec![];
    for (name, statistic) in [
        ("count", harmos::Aggregate::Count),
        ("min", harmos::Aggregate::Min),
        ("max", harmos::Aggregate::Max),
        ("mean", harmos::Aggregate::Mean),
        ("median", harmos::Aggregate::Median),
        ("std", harmos::Aggregate::Std),
        ("rms", harmos::Aggregate::Rms),
    ] {
        measures.push(Measure {
            field: field(key),
            statistic,
            name: name.into(),
        });
    }
    measures.extend([
        Measure {
            field: FieldPath {
                fields: vec!["payload".into(), "metadata".into(), "ordinal".into()],
            },
            statistic: harmos::Aggregate::Count,
            name: "rows".into(),
        },
        Measure {
            field: x.clone(),
            statistic: harmos::Aggregate::Min,
            name: "x_min".into(),
        },
        Measure {
            field: x,
            statistic: harmos::Aggregate::Max,
            name: "x_max".into(),
        },
    ]);
    let query = telemetry::plan(
        at,
        scope,
        &codec,
        vec![Transform::Aggregate(Aggregate {
            groups: vec![],
            measures,
        })],
    )?;
    let batches = super::batches(ctx, storage, settings, query).await?;
    let batch = batches
        .first()
        .ok_or_else(|| invalid("aggregate query returned no identity row"))?
        .as_record_batch();
    if batch.num_rows() != 1 || batches.len() != 1 {
        return Err(invalid("ungrouped statistics returned multiple rows"));
    }
    let value = |name: &str| {
        FieldPath {
            fields: vec![name.into()],
        }
        .value(batch, 0)
    };
    let count = count_value(value("count")?)?;
    let rows = count_value(value("rows")?)?;
    let minimum = number(value("x_min")?);
    let maximum = number(value("x_max")?);
    Ok(Statistics {
        count,
        rows,
        min: number(value("min")?),
        max: number(value("max")?),
        mean: number(value("mean")?),
        median: number(value("median")?),
        std: number(value("std")?),
        rms: number(value("rms")?),
        window: minimum
            .zip(maximum)
            .map(|(from, to)| PlotWindow { from, to }),
    })
}

pub(super) async fn histogram(
    read: super::ReadContext<'_, '_>,
    at: &StorageCursor,
    scope: &str,
    contract: &TelemetryContract,
    key: &str,
    bins: usize,
    statistics: &Statistics,
) -> Result<HistogramSeries> {
    let super::ReadContext {
        ctx,
        storage,
        settings,
    } = read;
    let count = bins.max(1);
    if count >= 65_536 {
        return Err(invalid(
            "histogram bin count exceeds storage transform limit",
        ));
    }
    let (minimum, maximum) = statistics.min.zip(statistics.max).unwrap_or((0.0, 0.0));
    let width = if minimum == maximum {
        1.0
    } else {
        (maximum - minimum) / count as f64
    };
    let edges = (0..=count)
        .map(|index| {
            if index == count && minimum != maximum {
                maximum
            } else {
                minimum + index as f64 * width
            }
        })
        .collect();
    let codec = dokime::storage::telemetry::codec(contract)?;
    let query = telemetry::plan(
        at,
        scope,
        &codec,
        vec![Transform::Histogram(Histogram {
            field: field(key),
            edges,
        })],
    )?;
    let batches = super::batches(ctx, storage, settings, query).await?;
    let mut output = Vec::with_capacity(count);
    let mut actual = 0usize;
    for batch in batches.iter() {
        let batch = batch.as_record_batch();
        for row in 0..batch.num_rows() {
            let value = |name: &str| {
                FieldPath {
                    fields: vec![name.into()],
                }
                .value(batch, row)
            };
            let amount = count_value(value("count")?)?;
            let lower = number(value("lower")?);
            let upper = number(value("upper")?);
            let Some((lower, upper)) = lower.zip(upper) else {
                if amount != 0 {
                    return Err(invalid("histogram edges omitted finite selected values"));
                }
                continue;
            };
            actual = actual
                .checked_add(amount)
                .ok_or_else(|| invalid("histogram count overflow"))?;
            output.push(HistogramBin {
                lower,
                upper,
                count: amount,
            });
        }
    }
    if actual != statistics.count || output.len() != count {
        return Err(invalid(
            "histogram does not cover the full selected dataset",
        ));
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
    Ok(HistogramSeries {
        key: key.into(),
        bins: output,
        count: actual,
        statistics,
    })
}

pub(super) fn number(value: ScalarValue) -> Option<f64> {
    let number = match value {
        ScalarValue::Float(value) => value,
        ScalarValue::Signed(value) => value as f64,
        ScalarValue::Unsigned(value) => value as f64,
        ScalarValue::Datetime(value) => value as f64 / 1_000_000_000.0,
        ScalarValue::Exact(value) => {
            (if value.negative { -1.0 } else { 1.0 }) * value.numerator as f64
                / value.denominator.get() as f64
        }
        _ => return None,
    };
    number.is_finite().then_some(number)
}

/* === Private Functions === */
fn count_value(value: ScalarValue) -> Result<usize> {
    if let ScalarValue::Unsigned(value) = value {
        usize::try_from(value).map_err(|_| invalid("aggregate count exceeds address space"))
    } else {
        Err(invalid("aggregate count has the wrong type"))
    }
}

/* === Tests === */
