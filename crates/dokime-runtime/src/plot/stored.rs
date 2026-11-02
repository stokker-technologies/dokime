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

// Standard Library Imports

// Third Party Imports
use harmos::{
    Aggregate,
    storage::{
        Query, ReadBatch,
        query::{
            Table, TableWriter,
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

// Local Imports
use super::{DownsampleTechnique, PlotPayload, PlotPayloadContext, PlotPayloadRequest, PlotSeriesSummary, PlotWindow};
use crate::{
    runtime::{Error, Result},
    storage::{Manager, telemetry::TypedCodec},
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(crate) struct Selection<'a> {
    pub query: Query,
    pub scope: &'a crate::evidence_runtime::ScopeSelection,
    pub producer_order: bool,
}

struct NativeTable {
    table: Table,
    ordered: bool,
    summaries: std::collections::BTreeMap<String, (usize, PlotSeriesSummary)>,
}

struct SeriesStatistics {
    moments: harmos::signal::Summary,
    eligible: usize,
    extent: Option<PlotWindow>,
    previous: Option<f64>,
    ascending: bool,
    values: Option<Vec<f64>>,
}

/* === Enums === */

enum NativeProjection {
    Empty,
    ExactRequired,
    Table(NativeTable),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) fn supports(request: &PlotPayloadRequest) -> bool {
    let Some(request) = fields(request) else { return false };
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
    let Some(request) = fields(request) else { return false };
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
    session: &Manager, local: &crate::storage::sqlite::State, selection: Selection<'_>, request: &PlotPayloadRequest, generated_at: &str,
    context: &PlotPayloadContext<'_>, records: crate::records::view::View<'_>,
) -> Result<Option<ReadBatch<PlotPayload>>> {
    let original = request;
    let normalized = fields(request).ok_or_else(|| invalid("stored plot requires plain fields"))?;
    let request = &normalized;
    let Selection { query, scope, producer_order } = selection;
    let maximum = request.max_points.unwrap_or(context.default_max_points);
    if maximum == 0 {
        return Err(invalid("plot output requires a finite point bound"));
    }
    let projection = native_table(session, local, &query, scope, request, producer_order, !original.y_expressions.is_empty()).await?;
    let NativeTable { table, ordered, summaries } = match projection {
        NativeProjection::ExactRequired => return Ok(None),
        NativeProjection::Table(table) => table,
        NativeProjection::Empty => {
            return super::finish_retained_plot_payload(
                super::project_retained_plot_payload(&[], records, request, generated_at, context)?,
                request,
            )
            .map(Some);
        }
    };
    let x = request.x_key.as_deref().unwrap_or("timestamp");
    let axis = if x == "timestamp" { path("_storage.source_time") } else { value(x) };
    let expression_time_order = original.x_expression.is_some() && original.x_key.as_deref().is_none_or(|key| key == "timestamp");
    let sort_axis = if expression_time_order { path("_storage.source_time") } else { axis.clone() };
    let ordered = if ordered && !expression_time_order {
        table.clone()
    } else {
        table.transform(
            &Transform::Sort(Sort {
                keys: vec![
                    SortKey {
                        field: sort_axis,
                        descending: false,
                        nulls_first: false,
                    },
                    SortKey {
                        field: path(if producer_order { "payload.metadata.sequence" } else { "_storage.position" }),
                        descending: false,
                        nulls_first: false,
                    },
                ],
            }),
            &mut || Ok(()),
        )?
    };
    let mut completed = None::<PlotPayload>;
    let mut histograms = Vec::new();
    for (index, key) in request.y_keys.iter().enumerate() {
        let (rows, eligible, summary) = if let Some((eligible, summary)) = summaries.get(key) {
            (table.len(), *eligible, summary.clone())
        } else {
            summary(&table, key, &axis, request.summary_window)?
        };
        let selected_input = if original.y_expressions.is_empty() {
            ordered.clone()
        } else {
            ordered.transform(&Transform::Filter(summary_filter(&value(key), &axis, None, x == "timestamp")?), &mut || Ok(()))?
        };
        let reduction_rows = selected_input.len();
        let technique = match request.technique {
            DownsampleTechnique::Auto | DownsampleTechnique::Unspecified if reduction_rows <= maximum.saturating_mul(4) => {
                DownsampleTechnique::Lttb
            }
            DownsampleTechnique::Auto | DownsampleTechnique::Unspecified => DownsampleTechnique::MinMax,
            technique => technique,
        };
        let algorithm = if eligible != reduction_rows
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
        let selected = selected_input.transform(
            &Transform::Downsample(Downsample {
                max_rows: maximum,
                algorithm,
                value: Some(value(key)),
                axis: Some(axis.clone()),
            }),
            &mut || Ok(()),
        )?;
        let mut points = Vec::new();
        let mut reader = selected.reader()?;
        while let Some(batch) = reader.next(&mut || Ok(()))? {
            use arrow_array::{Array, StringArray};
            points.try_reserve(batch.num_rows()).map_err(|_| invalid("plot point allocation failed"))?;
            let timestamps = batch
                .column_by_name("_timestamp")
                .and_then(|column| column.as_any().downcast_ref::<StringArray>())
                .ok_or_else(|| invalid("native plot timestamps are absent"))?;
            for row in 0..batch.num_rows() {
                if timestamps.is_null(row) {
                    return Err(invalid("native plot timestamp is null"));
                }
                let x = if x == "timestamp" {
                    serde_json::Value::String(timestamps.value(row).to_owned())
                } else {
                    number(value(x).value(&batch, row)?).map_or(serde_json::Value::Null, serde_json::Value::from)
                };
                let y = number(value(key).value(&batch, row)?);
                if !original.y_expressions.is_empty() && y.is_none() {
                    continue;
                }
                points.push(super::PlotPoint { x, y, z: None });
            }
        }
        let mut single = request.clone();
        single.y_keys = vec![key.clone()];
        let payload = super::project_retained_plot_payload(&[], records, &single, generated_at, context)?;
        let payload = payload.try_map(|mut payload| {
            payload.series[0].points = points;
            for series in &mut payload.series {
                series.raw_points = if original.y_expressions.is_empty() { rows } else { eligible };
                series.downsampled = series.points.len() < series.raw_points;
                if let Some(expression) = original.y_expressions.get(index) {
                    series.label = expression.label.clone().unwrap_or_else(|| key.clone());
                }
            }
            if matches!(request.plot_type, Some(crate::document::PlotType::TimeSeries | crate::document::PlotType::XyScatter)) {
                payload.summaries = vec![summary.clone()];
            }
            Ok::<_, Error>(payload)
        })?;
        if request.plot_type == Some(crate::document::PlotType::Histogram) {
            histograms.push(histogram(&table, key, request.histogram.bins, &summary)?);
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
            Some(current.map_or(window, |current| PlotWindow {
                from: current.from.min(window.from),
                to: current.to.max(window.to),
            }))
        });
        for summary in &mut payload.summaries {
            summary.window = window;
        }
    }
    let payload = super::finish_plot_frame(payload, request)?;
    Ok(Some(ReadBatch::new(payload)))
}

pub(crate) fn path(field: &str) -> FieldPath {
    FieldPath {
        fields: field.split('.').map(str::to_owned).collect(),
    }
}

/* === Private Functions === */

fn value(field: &str) -> FieldPath {
    FieldPath {
        fields: vec!["payload".into(), "values".into(), field.into()],
    }
}

// Stage Typed Arrow Values And Original Timestamps For Bounded Plot Reductions.
async fn native_table(
    session: &Manager, local: &crate::storage::sqlite::State, query: &Query, scope: &crate::evidence_runtime::ScopeSelection,
    request: &PlotPayloadRequest, producer_order: bool, exact_fields: bool,
) -> Result<NativeProjection> {
    use arrow_array::{ArrayRef, Float64Array, RecordBatch, StringArray, StructArray, TimestampNanosecondArray, UInt64Array};
    use arrow_schema::Field;
    use std::sync::Arc;
    let harmos::storage::query::Source::Stream(source) = &query.source else { return Ok(NativeProjection::Empty) };
    let x = request.x_key.as_deref().unwrap_or("timestamp");
    let mut keys = request.y_keys.clone();
    if x != "timestamp" && !keys.iter().any(|key| key == x) {
        keys.push(x.to_owned());
    }
    keys.sort();
    keys.dedup();
    if request.summary_window.is_some_and(|window| !window.from.is_finite() || !window.to.is_finite() || window.from > window.to) {
        return Err(invalid("plot summary window is invalid"));
    }
    // Share One Response Budget Across All Exact-Median Columns; Larger Selections Keep The Spill Path.
    let cache_limit = session.settings().response.batch_bytes.get() / std::mem::size_of::<f64>() / keys.len();
    let mut statistics = keys
        .iter()
        .map(|_| SeriesStatistics {
            moments: harmos::signal::Summary::new(),
            eligible: 0,
            extent: None,
            previous: None,
            ascending: true,
            values: Some(Vec::new()),
        })
        .collect::<Vec<_>>();
    let mut source = source.clone();
    let mut writer = None::<TableWriter>;
    let mut previous = None;
    let mut ordered = true;
    loop {
        let page = harmos::storage::ScanPage {
            after: source.after,
            rows: session.settings().response.batch_rows.max(std::num::NonZeroUsize::new(4096).expect("nonzero scan page")),
            bytes: session.settings().response.batch_bytes,
        };
        let rows = local.telemetry_numeric(query.at.clone(), source.clone(), page, keys.clone(), scope.clone()).await?;
        if rows.is_empty() {
            break;
        }
        let mut values = vec![Vec::with_capacity(rows.len()); keys.len()];
        let mut stamps = Vec::with_capacity(rows.len());
        let mut positions = Vec::with_capacity(rows.len());
        let mut sequences = Vec::with_capacity(rows.len());
        let mut timestamps = Vec::with_capacity(rows.len());
        for (position, sample) in rows {
            source.after = source.after.max(position);
            let projected = sample.values;
            // Numeric Projection Rounds Integers First; Typed Expressions Must Aggregate Them Exactly.
            if exact_fields && projected.iter().flatten().any(|value| value.abs() >= (1_u64 << f64::MANTISSA_DIGITS) as f64) {
                return Ok(NativeProjection::ExactRequired);
            }
            let sequence = sample.sequence;
            let stamp = sample.timestamp.to_datetime().map_err(crate::storage::Error::from)?.unix_timestamp_nanos();
            let timestamp = sample.timestamp.to_string();
            timestamps.push(timestamp);
            stamps.push(i64::try_from(stamp).map_err(|_| invalid("plot timestamp exceeds native range"))?);
            positions.push(position);
            sequences.push(sequence);
            for (value, column) in projected.iter().zip(&mut values) {
                column.push(*value);
            }
            let axis = if x == "timestamp" {
                ScalarValue::Datetime(stamp)
            } else {
                projected[keys.binary_search_by(|key| key.as_str().cmp(x)).expect("axis was projected")]
                    .map_or(ScalarValue::Null, ScalarValue::Float)
            };
            let coordinate = number(axis.clone());
            for (value, statistics) in projected.iter().zip(&mut statistics) {
                let Some(value) = value else { continue };
                statistics.eligible += 1;
                if request.summary_window.is_some_and(|window| match &axis {
                    ScalarValue::Datetime(stamp) => *stamp < (window.from * 1e9) as i128 || *stamp > (window.to * 1e9) as i128,
                    _ => coordinate.is_none_or(|x| x < window.from || x > window.to),
                }) {
                    continue;
                }
                statistics.ascending &= statistics.previous.is_none_or(|previous| previous <= *value);
                statistics.previous = Some(*value);
                statistics.moments.push(*value);
                if let Some(values) = &mut statistics.values {
                    if values.len() == cache_limit {
                        statistics.values = None;
                    } else {
                        if values.len() == values.capacity() {
                            values
                                .try_reserve_exact((cache_limit - values.len()).min(page.rows.get()))
                                .map_err(|_| invalid("plot median allocation failed"))?;
                        }
                        values.push(*value);
                    }
                }
                if let Some(x) = coordinate {
                    let range = statistics.extent.get_or_insert(PlotWindow { from: x, to: x });
                    range.from = range.from.min(x);
                    range.to = range.to.max(x);
                }
            }
            let tie = if producer_order { sequence } else { position };
            let current = (axis, ScalarValue::Unsigned(tie));
            if let Some(previous) = &previous {
                ordered &= ordered_pair(previous, &current);
            }
            previous = Some(current);
        }
        let field = |name: &str, array: ArrayRef| (Arc::new(Field::new(name, array.data_type().clone(), true)), array);
        let columns = keys.iter().zip(values).map(|(key, values)| field(key, Arc::new(Float64Array::from(values)))).collect::<Vec<_>>();
        let values = Arc::new(StructArray::from(columns));
        let metadata = Arc::new(StructArray::from(vec![field("sequence", Arc::new(UInt64Array::from(sequences)))]));
        let payload = Arc::new(StructArray::from(vec![field("values", values), field("metadata", metadata)]));
        let storage = Arc::new(StructArray::from(vec![
            field("source_time", Arc::new(TimestampNanosecondArray::from(stamps).with_timezone("UTC"))),
            field("position", Arc::new(UInt64Array::from(positions))),
        ]));
        let columns: Vec<(&str, ArrayRef)> =
            vec![("_storage", storage), ("payload", payload), ("_timestamp", Arc::new(StringArray::from(timestamps)))];
        let page = RecordBatch::try_from_iter(columns).map_err(|_| invalid("native plot page is invalid"))?;
        if writer.is_none() {
            writer = Some(TableWriter::new(page.schema())?);
        }
        writer.as_mut().expect("native plot writer was initialized").push(&page, &mut || Ok(()))?;
    }
    let Some(writer) = writer else { return Ok(NativeProjection::Empty) };
    let table = writer.finish()?;
    let axis = if x == "timestamp" { path("_storage.source_time") } else { value(x) };
    let mut summaries = std::collections::BTreeMap::new();
    for (key, statistics) in keys.into_iter().zip(statistics) {
        let SeriesStatistics {
            moments,
            eligible,
            extent,
            ascending,
            values,
            ..
        } = statistics;
        let median = if let Some(mut values) = values {
            if values.is_empty() {
                None
            } else {
                if !ascending {
                    // Stable Equal-Value Ordering Preserves The Sign Of Zero At The Middle Rank.
                    values.sort_by(|left, right| left.partial_cmp(right).expect("finite plot values"));
                }
                Some(midpoint(values[(values.len() - 1) / 2], values[values.len() / 2]))
            }
        } else {
            native_median(&table, &key, &axis, request.summary_window, moments.count() as usize, ascending)?
        };
        let summary = PlotSeriesSummary {
            series_id: key.clone(),
            count: moments.count() as usize,
            min: moments.min(),
            max: moments.max(),
            mean: moments.mean(),
            median,
            std: moments.std(),
            rms: moments.rms(),
            window: request.summary_window.or(extent),
        };
        if [summary.min, summary.max, summary.mean, summary.median, summary.std, summary.rms]
            .into_iter()
            .flatten()
            .any(|value| !value.is_finite())
        {
            return Err(invalid("plot summary exceeds finite range"));
        }
        summaries.insert(key, (eligible, summary));
    }
    Ok(NativeProjection::Table(NativeTable { table, ordered, summaries }))
}

// Select Exact Middle Ranks From The Existing Numeric Spill; Monotonic Values Need No Sort.
fn native_median(
    table: &Table, key: &str, axis: &FieldPath, window: Option<PlotWindow>, count: usize, ascending: bool,
) -> Result<Option<f64>> {
    use harmos::storage::query::transform::project::{Project, Projection};
    if count == 0 {
        return Ok(None);
    }
    let mut table = table.clone();
    if count != table.len() {
        table = table.transform(
            &Transform::Filter(summary_filter(&value(key), axis, window, axis.fields == ["_storage", "source_time"])?),
            &mut || Ok(()),
        )?;
    }
    let field = if ascending {
        value(key)
    } else {
        table = table.transform(
            &Transform::Project(Project {
                fields: vec![Projection { field: value(key), name: "y".into() }],
            }),
            &mut || Ok(()),
        )?;
        table = table.transform(
            &Transform::Sort(Sort {
                keys: vec![SortKey {
                    field: path("y"),
                    descending: false,
                    nulls_first: false,
                }],
            }),
            &mut || Ok(()),
        )?;
        path("y")
    };
    let (lower, upper) = ((count - 1) / 2, count / 2);
    let mut offset = 0;
    let mut low = None;
    let mut reader = table.reader()?;
    while let Some(page) = reader.next(&mut || Ok(()))? {
        if lower >= offset && lower < offset + page.num_rows() {
            low = number(field.value(&page, lower - offset)?);
        }
        if upper >= offset && upper < offset + page.num_rows() {
            let low = low.ok_or_else(|| invalid("plot median lower rank is absent"))?;
            let high = number(field.value(&page, upper - offset)?).ok_or_else(|| invalid("plot median upper rank is absent"))?;
            return Ok(Some(midpoint(low, high)));
        }
        offset += page.num_rows();
    }
    Err(invalid("plot median rank exceeds stored selection"))
}

fn midpoint(low: f64, high: f64) -> f64 {
    if low == high {
        low
    } else if low.signum() == high.signum() {
        low + (high - low) * 0.5
    } else {
        low * 0.5 + high * 0.5
    }
}

// Visit The Checked Graph Once Without Materializing Every Sample's Metadata And Value Maps.
fn summary(base: &Table, key: &str, axis: &FieldPath, window: Option<PlotWindow>) -> Result<(usize, usize, PlotSeriesSummary)> {
    let time_axis = axis.fields == ["_storage", "source_time"];
    let (rows, unfiltered) = aggregate_summary(base, key, axis, None, time_axis)?;
    let eligible = unfiltered.count;
    if window.is_none() && eligible == rows {
        return Ok((rows, eligible, unfiltered));
    }
    let (_, summary) = aggregate_summary(base, key, axis, Some(window), time_axis)?;
    Ok((rows, eligible, summary))
}

// Projects The Value, Axis And Position Before The Fold, So The Spooled Selection Holds Three Columns.
// `filter` Is `None` For The Whole Selection, Or The Optional Window That Rows With A Value Must Fall In.
fn aggregate_summary(
    base: &Table, key: &str, axis: &FieldPath, filter: Option<Option<PlotWindow>>, time_axis: bool,
) -> Result<(usize, PlotSeriesSummary)> {
    use harmos::storage::query::transform::project::{Project, Projection};
    let column = |name: &str| FieldPath { fields: vec![name.into()] };
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
    .map(|(name, statistic)| Measure {
        field: column("y"),
        statistic,
        name: name.into(),
    })
    .collect::<Vec<_>>();
    measures.extend([
        Measure {
            field: column("p"),
            statistic: Aggregate::Count,
            name: "rows".into(),
        },
        Measure {
            field: column("x"),
            statistic: Aggregate::Min,
            name: "from".into(),
        },
        Measure {
            field: column("x"),
            statistic: Aggregate::Max,
            name: "to".into(),
        },
    ]);

    let mut table = base.transform(
        &Transform::Project(Project {
            fields: vec![
                Projection { field: value(key), name: "y".into() },
                Projection { field: axis.clone(), name: "x".into() },
                Projection {
                    field: path("_storage.position"),
                    name: "p".into(),
                },
            ],
        }),
        &mut || Ok(()),
    )?;
    if let Some(window) = filter {
        table = table.transform(&Transform::Filter(summary_filter(&column("y"), &column("x"), window, time_axis)?), &mut || Ok(()))?;
    }
    let table = table.transform(&Transform::Aggregate(Fold { groups: vec![], measures }), &mut || Ok(()))?;
    let window = filter.flatten();

    let mut reader = table.reader()?;
    let mut result = None;
    while let Some(batch) = reader.next(&mut || Ok(()))? {
        let batch = &batch;
        for row in 0..batch.num_rows() {
            let scalar = |name: &str| column(name).value(batch, row);
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
                count("rows")?,
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
    Ok(result.unwrap_or((
        0,
        PlotSeriesSummary {
            series_id: key.into(),
            count: 0,
            min: None,
            max: None,
            mean: None,
            median: None,
            std: None,
            rms: None,
            window,
        },
    )))
}

fn summary_filter(
    value: &FieldPath, axis: &FieldPath, window: Option<PlotWindow>, time_axis: bool,
) -> Result<harmos::storage::query::transform::filter::Filter> {
    use harmos::storage::query::transform::filter::{Clause, Compare, Condition, Filter, Term};
    let mut terms = vec![Term {
        field: value.clone(),
        condition: Condition::IsNull,
        negate: true,
    }];
    if let Some(window) = window {
        if !window.from.is_finite() || !window.to.is_finite() || window.from > window.to {
            return Err(invalid("plot summary window is invalid"));
        }
        for (bound, op) in [(window.from, Compare::Ge), (window.to, Compare::Le)] {
            let scalar = if time_axis { ScalarValue::Datetime((bound * 1e9) as i128) } else { ScalarValue::Float(bound) };
            terms.push(Term {
                field: axis.clone(),
                condition: Condition::Compare { op, value: scalar },
                negate: false,
            });
        }
    }
    Ok(Filter { clauses: vec![Clause { terms }] })
}

fn histogram(base: &Table, key: &str, bins: usize, summary: &PlotSeriesSummary) -> Result<super::HistogramSeries> {
    let count = bins.max(1);
    if count >= 65_536 {
        return Err(invalid("histogram bin count exceeds storage limit"));
    }
    let (minimum, maximum) = summary.min.zip(summary.max).unwrap_or((0.0, 0.0));
    let width = if minimum == maximum { 1.0 } else { (maximum - minimum) / count as f64 };
    let edges =
        (0..=count).map(|index| if index == count && minimum != maximum { maximum } else { minimum + index as f64 * width }).collect();
    let table = base.transform(&Transform::Histogram(Histogram { field: value(key), edges }), &mut || Ok(()))?;
    let mut reader = table.reader()?;
    let mut bins = Vec::new();
    let mut actual = 0usize;
    while let Some(batch) = reader.next(&mut || Ok(()))? {
        let batch = &batch;
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
    Ok(super::HistogramSeries {
        key: key.into(),
        bins,
        count: actual,
        statistics,
    })
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

fn ordered_pair(left: &(ScalarValue, ScalarValue), right: &(ScalarValue, ScalarValue)) -> bool {
    let compare = |left: &ScalarValue, right: &ScalarValue| match (left, right) {
        (ScalarValue::Datetime(left), ScalarValue::Datetime(right)) => Some(left.cmp(right)),
        (ScalarValue::Unsigned(left), ScalarValue::Unsigned(right)) => Some(left.cmp(right)),
        (ScalarValue::Signed(left), ScalarValue::Signed(right)) => Some(left.cmp(right)),
        (ScalarValue::Float(left), ScalarValue::Float(right)) => left.partial_cmp(right),
        _ => None,
    };
    match compare(&left.0, &right.0) {
        Some(std::cmp::Ordering::Less) => true,
        Some(std::cmp::Ordering::Equal) => compare(&left.1, &right.1).is_some_and(|order| order.is_le()),
        _ => false,
    }
}

fn fields(request: &PlotPayloadRequest) -> Option<PlotPayloadRequest> {
    let field = |expression: &crate::expressions::DataExpression| match &expression.source {
        crate::expressions::DataSource::Telemetry { stream, field }
            if super::stream_matches_alias(stream, &request.stream) && expression.transforms.is_empty() =>
        {
            Some(field.clone())
        }
        _ => None,
    };
    let mut normalized = request.clone();
    if let Some(expression) = &request.x_expression {
        let x = field(expression)?;
        if request.x_key.as_deref().is_some_and(|key| key != "timestamp" && key != x) {
            return None;
        }
        normalized.x_key = Some(x);
        normalized.x_expression = None;
    }
    if !request.y_expressions.is_empty() {
        normalized.y_keys = request.y_expressions.iter().map(field).collect::<Option<Vec<_>>>()?;
        normalized.y_expressions.clear();
    }
    Some(normalized)
}

fn invalid(reason: &'static str) -> Error {
    Error::Storage(crate::storage::Error::Invalid(reason))
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[harmos::message]
    #[derive(Clone, PartialEq)]
    struct Fields {
        easting: f64,
        northing: Option<f64>,
    }

    #[test]
    fn field_expressions() {
        let request = PlotPayloadRequest {
            stream: "tracking".into(),
            x_expression: Some(crate::expressions::telemetry("tracking", "easting")),
            y_expressions: vec![crate::expressions::telemetry("tracking", "northing").label("Northing")],
            plot_type: Some(crate::document::PlotType::XyLine),
            max_points: Some(17),
            ..Default::default()
        };
        assert!(supports(&request));
        let settings = harmos::config::SchemaSettings::default();
        let schema = harmos::schema::graph::Schema::from_declaration(<Fields as harmos::schema::Described>::DEFINITION, &settings).unwrap();
        let codec = TypedCodec::new(&crate::Identifier::new("tracking").unwrap(), 1, &schema, &settings).unwrap();
        assert!(supports_codec(&request, &codec));
        let mut missing = request.clone();
        missing.y_expressions[0] = crate::expressions::telemetry("tracking", "absent");
        assert!(!supports_codec(&missing, &codec));
        let normalized = fields(&request).unwrap();
        assert_eq!(normalized.x_key.as_deref(), Some("easting"));
        assert_eq!(normalized.y_keys, ["northing"]);
        let mut transformed = request.clone();
        transformed.y_expressions[0] = crate::expressions::telemetry("tracking", "northing").count();
        assert!(!supports(&transformed));
        transformed.y_expressions[0] = crate::expressions::telemetry("unrelated", "northing");
        assert!(!supports(&transformed));
    }

    #[test]
    fn stored_order_requires_axis_and_tie_order() {
        let point = |time, position| (ScalarValue::Datetime(time), ScalarValue::Unsigned(position));
        assert!(ordered_pair(&point(1, 2), &point(2, 1)));
        assert!(ordered_pair(&point(1, 2), &point(1, 3)));
        assert!(!ordered_pair(&point(2, 1), &point(1, 2)));
        assert!(!ordered_pair(&point(1, 3), &point(1, 2)));
        let missing = (ScalarValue::Float(f64::NAN), ScalarValue::Unsigned(1));
        assert!(!ordered_pair(&missing, &missing));
    }
}
