/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Saved Plot Queries With Bounded Exact Reductions.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

use super::{DownsampleTechnique, PlotPayload, PlotPayloadContext, PlotPayloadRequest, PlotSeriesSummary, PlotWindow};
use crate::{
    runtime::{Error, Result},
    storage::{Manager, telemetry::TypedCodec},
};
use harmos::{
    Aggregate,
    storage::{
        ReadBatch,
        operations::Query,
        query::{
            QueryArrow,
            scalar::ScalarValue,
            transform::{
                FieldPath, FieldPathExecution, Transform,
                aggregate::{Aggregate as Fold, Measure},
                downsample::{Algorithm, Downsample},
                histogram::Histogram,
                sort::{Sort, SortKey},
            },
        },
    },
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(crate) struct Selection<'a> {
    pub query: Query,
    pub codec: &'a TypedCodec,
    pub producer_order: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) fn supports(request: &PlotPayloadRequest) -> bool {
    request.analysis.is_none()
        && request.max_points != Some(0)
        && request.series_by.is_none()
        && request.series_by_expression.is_none()
        && request.x_expression.is_none()
        && request.y_expressions.is_empty()
        && request.z_expression.is_none()
        && request.z_stream.is_none()
        && request.z_field.is_none()
        && request.filters.is_empty()
        && !request.y_keys.is_empty()
        && matches!(
            request.plot_type,
            None | Some(
                crate::document::PlotType::TimeSeries
                    | crate::document::PlotType::XyScatter
                    | crate::document::PlotType::XyLine
                    | crate::document::PlotType::Histogram
            )
        )
}

pub(crate) fn supports_codec(request: &PlotPayloadRequest, codec: &TypedCodec) -> bool {
    use harmos::schema::{Cardinality, graph::Kind};
    let graph = codec.graph();
    let Some(values) = graph.definitions.get(graph.root).and_then(|fields| fields.iter().find(|field| field.name == "values")) else {
        return false;
    };
    let Kind::Message { definition: index } = values.kind else {
        return false;
    };
    let numeric = |name: &str| {
        graph.definitions.get(index).and_then(|fields| fields.iter().find(|field| field.name == name)).is_some_and(|field| {
            field.cardinality != Cardinality::Repeated
                && matches!(field.kind, Kind::U32 | Kind::U64 | Kind::I32 | Kind::I64 | Kind::F32 | Kind::F64)
        })
    };
    request.y_keys.iter().all(|key| numeric(key)) && request.x_key.as_deref().is_none_or(|key| key == "timestamp" || numeric(key))
}

// Each Series Reduces The Original Selection Independently; No Re-Reduction Of A Union.
pub(crate) async fn materialize(
    session: &Manager, selection: Selection<'_>, request: &PlotPayloadRequest, generated_at: &str, context: &PlotPayloadContext<'_>,
    records: crate::records::view::View<'_>,
) -> Result<ReadBatch<PlotPayload>> {
    let Selection { query, codec, producer_order } = selection;
    let maximum = request.max_points.unwrap_or(context.default_max_points);
    if maximum == 0 {
        return Err(invalid("plot output requires a finite point bound"));
    }
    let mut completed = None::<PlotPayload>;
    let mut histograms = Vec::new();
    for key in &request.y_keys {
        let x = request.x_key.as_deref().unwrap_or("timestamp");
        let axis = if x == "timestamp" { path("_storage.source_time") } else { value(x) };
        let (rows, summary) = summary(session, &query, key, axis.clone(), request.summary_window).await?;
        let eligible = if request.summary_window.is_some() {
            let mut complete = query.clone();
            complete.transforms.push(Transform::Filter(harmos::storage::query::transform::filter::Filter {
                clauses: vec![harmos::storage::query::transform::filter::Clause {
                    terms: vec![harmos::storage::query::transform::filter::Term {
                        field: value(key),
                        condition: harmos::storage::query::transform::filter::Condition::IsNull,
                        negate: true,
                    }],
                }],
            }));
            row_count(session, &complete).await?
        } else {
            summary.count
        };
        let technique = match request.technique {
            DownsampleTechnique::Auto | DownsampleTechnique::Unspecified if rows <= maximum.saturating_mul(4) => DownsampleTechnique::Lttb,
            DownsampleTechnique::Auto | DownsampleTechnique::Unspecified => DownsampleTechnique::MinMax,
            technique => technique,
        };
        let algorithm = if eligible != rows
            || maximum == 1
            || matches!(technique, DownsampleTechnique::Raw)
            || (maximum < 3 && matches!(technique, DownsampleTechnique::Lttb))
        {
            Algorithm::Decimate
        } else if matches!(technique, DownsampleTechnique::MinMax) {
            Algorithm::Extrema
        } else {
            Algorithm::Lttb
        };
        let mut selected = query.clone();
        selected.transforms.push(Transform::Sort(Sort {
            keys: vec![
                SortKey { field: axis.clone(), descending: false, nulls_first: false },
                SortKey {
                    field: path(if producer_order { "payload.metadata.sequence" } else { "_storage.position" }),
                    descending: false,
                    nulls_first: false,
                },
            ],
        }));
        selected.transforms.push(Transform::Downsample(Downsample {
            max_rows: maximum,
            algorithm,
            value: Some(value(key)),
            axis: Some(axis),
        }));
        let mut samples = Vec::new();
        let mut reader = session.query(selected).await?;
        while let Some(output) = reader.next().await? {
            if let QueryArrow::Batch(batch) = output {
                let entries = crate::storage::telemetry::decode_positioned(codec, batch)?;
                samples.try_reserve(entries.len()).map_err(|_| invalid("plot output workspace allocation failed"))?;
                for (_, entry) in entries.iter() {
                    let mut sample = entry.sample.clone();
                    sample.storage_payload = None;
                    sample.values.retain(|field, _| field == key || field == x);
                    samples.push(sample);
                }
            }
        }
        reader.completion()?;
        let mut single = request.clone();
        single.y_keys = vec![key.clone()];
        let payload = super::project_retained_plot_payload(&samples, records, &single, generated_at, context)?;
        let payload = payload.try_map(|mut payload| {
            for series in &mut payload.series {
                series.raw_points = rows;
                series.downsampled = series.points.len() < rows;
            }
            if matches!(request.plot_type, Some(crate::document::PlotType::TimeSeries | crate::document::PlotType::XyScatter)) {
                payload.summaries = vec![summary.clone()];
            }
            Ok::<_, Error>(payload)
        })?;
        if request.plot_type == Some(crate::document::PlotType::Histogram) {
            histograms.push(histogram(session, &query, key, request.histogram.bins, &summary).await?);
        }
        if let Some(completed) = &mut completed {
            completed.series.extend(payload.series.iter().cloned());
            completed.summaries.extend(payload.summaries.iter().cloned());
        } else {
            completed = Some((*payload).clone());
        }
    }
    let mut payload = completed.ok_or_else(|| invalid("plot has no selected series"))?;
    if !histograms.is_empty() {
        if let Some(unit) = &request.histogram.unit {
            for series in &mut payload.series {
                series.unit = Some(unit.clone());
            }
        }
        payload.analysis = Some(super::PlotAnalysis::Histogram { series: histograms });
    }
    if request.summary_window.is_none() {
        let window = payload.summaries.iter().filter_map(|summary| summary.window).fold(None::<PlotWindow>, |current, window| {
            Some(current.map_or(window, |current| PlotWindow { from: current.from.min(window.from), to: current.to.max(window.to) }))
        });
        for summary in &mut payload.summaries {
            summary.window = window;
        }
    }
    let payload = super::finish_plot_frame(payload, request)?;
    Ok(ReadBatch::new(payload))
}

pub(crate) fn path(field: &str) -> FieldPath {
    FieldPath { fields: field.split('.').map(str::to_owned).collect() }
}
fn value(field: &str) -> FieldPath {
    FieldPath { fields: vec!["payload".into(), "values".into(), field.into()] }
}

/* === Private Functions === */

async fn summary(
    session: &Manager, base: &Query, key: &str, axis: FieldPath, window: Option<PlotWindow>,
) -> Result<(usize, PlotSeriesSummary)> {
    let raw_rows = row_count(session, base).await?;
    let mut measures = [
        ("count", Aggregate::Count),
        ("min", Aggregate::Min),
        ("max", Aggregate::Max),
        ("mean", Aggregate::Mean),
        ("median", Aggregate::Median),
        ("std", Aggregate::Std),
        ("rms", Aggregate::Rms),
    ]
    .into_iter()
    .map(|(name, statistic)| Measure { field: value(key), statistic, name: name.into() })
    .collect::<Vec<_>>();
    measures.extend([
        Measure { field: path("_storage.position"), statistic: Aggregate::Count, name: "rows".into() },
        Measure { field: axis.clone(), statistic: Aggregate::Min, name: "from".into() },
        Measure { field: axis.clone(), statistic: Aggregate::Max, name: "to".into() },
    ]);
    let mut query = base.clone();
    use harmos::storage::query::transform::filter::{Clause, Compare, Condition, Filter, Term};
    let mut terms = vec![Term { field: value(key), condition: Condition::IsNull, negate: true }];
    if let Some(window) = window {
        if !window.from.is_finite() || !window.to.is_finite() || window.from > window.to {
            return Err(invalid("plot summary window is invalid"));
        }
        for (bound, op) in [(window.from, Compare::Ge), (window.to, Compare::Le)] {
            let scalar = if axis.fields == ["_storage", "source_time"] {
                ScalarValue::Datetime((bound * 1e9) as i128)
            } else {
                ScalarValue::Float(bound)
            };
            terms.push(Term { field: axis.clone(), condition: Condition::Compare { op, value: scalar }, negate: false });
        }
    }
    query.transforms.push(Transform::Filter(Filter { clauses: vec![Clause { terms }] }));
    query.transforms.push(Transform::Aggregate(Fold { groups: vec![], measures }));
    let mut reader = session.query(query).await?;
    let mut result = None;
    while let Some(output) = reader.next().await? {
        let QueryArrow::Batch(batch) = output else {
            continue;
        };
        let batch = batch.as_record_batch();
        for row in 0..batch.num_rows() {
            let scalar = |name: &str| FieldPath { fields: vec![name.into()] }.value(batch, row);
            let count = |name| -> Result<usize> {
                match scalar(name)? {
                    ScalarValue::Unsigned(value) => usize::try_from(value).map_err(|_| invalid("plot count exceeds native range")),
                    _ => Err(invalid("plot count is invalid")),
                }
            };
            let numeric = |name| -> Result<Option<f64>> { Ok(number(scalar(name)?)) };
            if result.is_some() {
                return Err(invalid("ungrouped plot summary emitted multiple rows"));
            }
            let window = window.or(numeric("from")?.zip(numeric("to")?).map(|(from, to)| PlotWindow { from, to }));
            result = Some((
                raw_rows,
                PlotSeriesSummary {
                    series_id: key.into(),
                    count: count("count")?,
                    min: numeric("min")?,
                    max: numeric("max")?,
                    mean: numeric("mean")?,
                    median: numeric("median")?,
                    std: numeric("std")?,
                    rms: numeric("rms")?,
                    window,
                },
            ));
        }
    }
    reader.completion()?;
    Ok(result.unwrap_or((
        raw_rows,
        PlotSeriesSummary { series_id: key.into(), count: 0, min: None, max: None, mean: None, median: None, std: None, rms: None, window },
    )))
}

async fn row_count(session: &Manager, base: &Query) -> Result<usize> {
    let mut query = base.clone();
    query.transforms.push(Transform::Aggregate(Fold {
        groups: vec![],
        measures: vec![Measure { field: path("_storage.position"), statistic: Aggregate::Count, name: "rows".into() }],
    }));
    let mut reader = session.query(query).await?;
    let mut count = 0usize;
    while let Some(output) = reader.next().await? {
        if let QueryArrow::Batch(batch) = output {
            let batch = batch.as_record_batch();
            for row in 0..batch.num_rows() {
                let ScalarValue::Unsigned(value) = path("rows").value(batch, row)? else {
                    return Err(invalid("plot row count is invalid"));
                };
                count = usize::try_from(value).map_err(|_| invalid("plot row count exceeds native range"))?;
            }
        }
    }
    reader.completion()?;
    Ok(count)
}

async fn histogram(session: &Manager, base: &Query, key: &str, bins: usize, summary: &PlotSeriesSummary) -> Result<super::HistogramSeries> {
    let count = bins.max(1);
    if count >= 65_536 {
        return Err(invalid("histogram bin count exceeds storage limit"));
    }
    let (minimum, maximum) = summary.min.zip(summary.max).unwrap_or((0.0, 0.0));
    let width = if minimum == maximum { 1.0 } else { (maximum - minimum) / count as f64 };
    let edges =
        (0..=count).map(|index| if index == count && minimum != maximum { maximum } else { minimum + index as f64 * width }).collect();
    let mut query = base.clone();
    query.transforms.push(Transform::Histogram(Histogram { field: value(key), edges }));
    let mut reader = session.query(query).await?;
    let mut bins = Vec::new();
    let mut actual = 0usize;
    while let Some(output) = reader.next().await? {
        let QueryArrow::Batch(batch) = output else {
            continue;
        };
        let batch = batch.as_record_batch();
        for row in 0..batch.num_rows() {
            let scalar = |name: &str| FieldPath { fields: vec![name.into()] }.value(batch, row);
            let ScalarValue::Unsigned(amount) = scalar("count")? else {
                return Err(invalid("histogram count is invalid"));
            };
            let amount = usize::try_from(amount).map_err(|_| invalid("histogram count exceeds native range"))?;
            let bounds = number(scalar("lower")?).zip(number(scalar("upper")?));
            let Some((lower, upper)) = bounds else {
                if amount != 0 {
                    return Err(invalid("histogram omitted finite values"));
                }
                continue;
            };
            actual = actual.checked_add(amount).ok_or_else(|| invalid("histogram count overflow"))?;
            bins.push(super::HistogramBin { lower, upper, count: amount });
        }
    }
    reader.completion()?;
    if actual != summary.count {
        return Err(invalid("histogram count differs from saved selection"));
    }
    let statistics = (|| {
        Some(super::HistogramStatistics {
            min: summary.min?,
            max: summary.max?,
            mean: summary.mean?,
            median: summary.median?,
            std: summary.std?,
            rms: summary.rms?,
        })
    })();
    Ok(super::HistogramSeries { key: key.into(), bins, count: actual, statistics })
}

fn number(value: ScalarValue) -> Option<f64> {
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

fn invalid(reason: &'static str) -> Error {
    Error::Storage(crate::storage::Error::Invalid(reason))
}

/* === Tests === */
