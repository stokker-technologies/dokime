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
            QueryArrow, Table, TableWriter,
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

struct NativeTable {
    table: Table,
    ordered: bool,
    summaries: std::collections::BTreeMap<String, (usize, PlotSeriesSummary)>,
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
    let native = native_table(session, &query, codec, request, producer_order).await?;
    let (table, ordered, native, summaries) = if let Some(NativeTable { table, ordered, summaries }) = native {
        (table, ordered, true, summaries)
    } else {
        // Read Durable Evidence Once; Reuse Its Immutable Spill For Every Series And Summary.
        let mut reader = session.query(query).await?;
        let mut writer = None::<TableWriter>;
        let x = request.x_key.as_deref().unwrap_or("timestamp");
        let axis = if x == "timestamp" { path("_storage.source_time") } else { value(x) };
        let tie = path(if producer_order { "payload.metadata.sequence" } else { "_storage.position" });
        let mut previous = None;
        let mut ordered = true;
        while let Some(output) = reader.next().await? {
            let QueryArrow::Batch(batch) = output else { continue };
            let page = batch.as_record_batch();
            for row in 0..page.num_rows() {
                let current = (axis.value(page, row)?, tie.value(page, row)?);
                if let Some(previous) = &previous {
                    ordered &= ordered_pair(previous, &current);
                }
                previous = Some(current);
            }
            if writer.is_none() {
                writer = Some(TableWriter::new(page.schema())?);
            }
            writer.as_mut().expect("table writer was initialized").push(page, &mut || Ok(()))?;
        }
        reader.completion()?;
        let Some(writer) = writer else {
            return super::finish_retained_plot_payload(
                super::project_retained_plot_payload(&[], records, request, generated_at, context)?,
                request,
            );
        };
        let table = writer.finish()?;
        (table, ordered, false, Default::default())
    };

    let x = request.x_key.as_deref().unwrap_or("timestamp");
    let axis = if x == "timestamp" { path("_storage.source_time") } else { value(x) };
    let ordered = if ordered {
        table.clone()
    } else {
        table.transform(
            &Transform::Sort(Sort {
                keys: vec![
                    SortKey {
                        field: axis.clone(),
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
    for key in &request.y_keys {
        let (rows, eligible, summary) = if let Some((eligible, summary)) = summaries.get(key) {
            (table.len(), *eligible, summary.clone())
        } else {
            summary(&table, key, &axis, request.summary_window)?
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
        let selected = ordered.transform(
            &Transform::Downsample(Downsample {
                max_rows: maximum,
                algorithm,
                value: Some(value(key)),
                axis: Some(axis.clone()),
            }),
            &mut || Ok(()),
        )?;
        let mut samples = Vec::new();
        let mut points = Vec::new();
        let mut reader = selected.reader()?;
        while let Some(batch) = reader.next(&mut || Ok(()))? {
            if native {
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
                    points.push(super::PlotPoint {
                        x,
                        y: number(value(key).value(&batch, row)?),
                        z: None,
                    });
                }
                continue;
            }
            let batch = harmos::storage::arrow::ArrowBatch::try_new(batch)?;
            let entries = crate::storage::telemetry::decode_positioned(codec, batch)?;
            samples.try_reserve(entries.len()).map_err(|_| invalid("plot output workspace allocation failed"))?;
            for (_, entry) in entries.iter() {
                let mut sample = entry.sample.clone();
                sample.storage_payload = None;
                sample.values.retain(|field, _| field == key || field == x);
                samples.push(sample);
            }
        }
        let mut single = request.clone();
        single.y_keys = vec![key.clone()];
        let payload = super::project_retained_plot_payload(&samples, records, &single, generated_at, context)?;
        let payload = payload.try_map(|mut payload| {
            if native {
                payload.series[0].points = points;
            }
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
    Ok(ReadBatch::new(payload))
}

pub(crate) fn path(field: &str) -> FieldPath {
    FieldPath {
        fields: field.split('.').map(str::to_owned).collect(),
    }
}
fn value(field: &str) -> FieldPath {
    FieldPath {
        fields: vec!["payload".into(), "values".into(), field.into()],
    }
}

/* === Private Functions === */

// Float Series Stage Only Their Numeric Columns And Original Timestamps.
// Integer Schemas Keep Their Native Exact Scalar Representation In The General Path.
async fn native_table(
    session: &Manager, query: &Query, codec: &TypedCodec, request: &PlotPayloadRequest, producer_order: bool,
) -> Result<Option<NativeTable>> {
    use arrow_array::{ArrayRef, Float64Array, RecordBatch, StringArray, StructArray, TimestampNanosecondArray, UInt64Array};
    use arrow_schema::Field;
    use harmos::schema::graph::Kind;
    use std::sync::Arc;
    let harmos::storage::query::Source::Stream(source) = &query.source else { return Ok(None) };
    if !query.transforms.is_empty() || query.slice != Default::default() {
        return Ok(None);
    }
    let x = request.x_key.as_deref().unwrap_or("timestamp");
    let mut keys = request.y_keys.clone();
    if x != "timestamp" && !keys.iter().any(|key| key == x) {
        keys.push(x.to_owned());
    }
    keys.sort();
    keys.dedup();
    let graph = codec.graph();
    let values = graph.definitions[graph.root].iter().find(|field| field.name == "values");
    let Some(Kind::Message { definition }) = values.map(|field| &field.kind) else { return Ok(None) };
    if !keys
        .iter()
        .all(|key| graph.definitions[*definition].iter().any(|field| field.name == *key && matches!(field.kind, Kind::F32 | Kind::F64)))
    {
        return Ok(None);
    }

    let Ok(validator) = harmos::schema::Validator::from_schema(graph, &Default::default()) else { return Ok(None) };
    if request.summary_window.is_some_and(|window| !window.from.is_finite() || !window.to.is_finite() || window.from > window.to) {
        return Err(invalid("plot summary window is invalid"));
    }
    let mut statistics =
        keys.iter().map(|_| (harmos::signal::Summary::new(), 0usize, None::<PlotWindow>, None::<f64>, true)).collect::<Vec<_>>();
    let mut reader = session
        .read_stream(harmos::storage::StreamRead {
            at: query.at.clone(),
            selection: source.clone(),
        })
        .await?;
    let mut writer = None::<TableWriter>;
    let mut previous = None;
    let mut ordered = true;
    while let Some(rows) = reader.next().await? {
        let mut values = vec![Vec::with_capacity(rows.len()); keys.len()];
        let mut stamps = Vec::with_capacity(rows.len());
        let mut positions = Vec::with_capacity(rows.len());
        let mut sequences = Vec::with_capacity(rows.len());
        let mut timestamps = Vec::with_capacity(rows.len());
        for row in rows.iter() {
            let (projected, sequence, stamp, timestamp) = project_native_row(&validator, codec, row, &keys)?;
            timestamps.push(timestamp);
            stamps.push(i64::try_from(stamp).map_err(|_| invalid("plot timestamp exceeds native range"))?);
            positions.push(row.record.position.sequence);
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
            for (value, (moments, eligible, extent, previous, ascending)) in projected.iter().zip(&mut statistics) {
                let Some(value) = value else { continue };
                *eligible += 1;
                if request.summary_window.is_some_and(|window| match &axis {
                    ScalarValue::Datetime(stamp) => *stamp < (window.from * 1e9) as i128 || *stamp > (window.to * 1e9) as i128,
                    _ => coordinate.is_none_or(|x| x < window.from || x > window.to),
                }) {
                    continue;
                }
                *ascending &= previous.is_none_or(|previous| previous <= *value);
                *previous = Some(*value);
                moments.push(*value);
                if let Some(x) = coordinate {
                    let range = extent.get_or_insert(PlotWindow { from: x, to: x });
                    range.from = range.from.min(x);
                    range.to = range.to.max(x);
                }
            }
            let tie = if producer_order { sequence } else { row.record.position.sequence };
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
    let Some(writer) = writer else { return Ok(None) };
    let table = writer.finish()?;
    let axis = if x == "timestamp" { path("_storage.source_time") } else { value(x) };
    let mut summaries = std::collections::BTreeMap::new();
    for (key, (moments, eligible, extent, _, ascending)) in keys.into_iter().zip(statistics) {
        let summary = PlotSeriesSummary {
            series_id: key.clone(),
            count: moments.count() as usize,
            min: moments.min(),
            max: moments.max(),
            mean: moments.mean(),
            median: native_median(&table, &key, &axis, request.summary_window, moments.count() as usize, ascending)?,
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
    Ok(Some(NativeTable { table, ordered, summaries }))
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
            return Ok(Some(if low == high {
                low
            } else if low.signum() == high.signum() {
                low + (high - low) * 0.5
            } else {
                low * 0.5 + high * 0.5
            }));
        }
        offset += page.num_rows();
    }
    Err(invalid("plot median rank exceeds stored selection"))
}

// Visit The Checked Graph Once Without Materializing Every Sample's Metadata And Value Maps.
fn project_native_row(
    validator: &harmos::schema::Validator, codec: &TypedCodec, row: &harmos::storage::StoredRow, keys: &[String],
) -> Result<(Vec<Option<f64>>, u64, i128, String)> {
    use harmos::schema::{Event, PathElement, Scalar};
    let mut values = vec![None; keys.len()];
    let mut sequence = None;
    let mut timestamp = None;
    let mut stamp_text = None;
    let mut scope = None;
    if &row.record.schema != codec.schema() {
        return Err(invalid("typed telemetry schema differs from storage envelope"));
    }
    validator
        .visit(&row.record.payload, |path, event| {
            let Event::Scalar(scalar) = event else { return Ok(()) };
            if let [PathElement::Field { name: "values", .. }, PathElement::Field { name, .. }] = path
                && let Ok(index) = keys.binary_search_by(|key| key.as_str().cmp(name))
            {
                let value = match scalar {
                    Scalar::F32(value) => f64::from(value),
                    Scalar::F64(value) => value,
                    _ => return Err(invalid("native plot value is not floating point")),
                };
                values[index] = value.is_finite().then_some(value);
            }
            if let [PathElement::Field { name: "metadata", .. }, PathElement::Field { name, .. }] = path {
                let matches = match (*name, scalar) {
                    ("sequence", Scalar::U64(value)) => {
                        sequence = Some(value);
                        true
                    }
                    ("ordinal", Scalar::U64(value)) => value == row.sequence,
                    ("schema_version", Scalar::U32(value)) => value == codec.schema().version.get(),
                    ("epoch", Scalar::String(value)) => row.record.source.as_ref().map(|id| id.as_str()) == Some(value),
                    ("sample_id", Scalar::String(value)) => row.record.item.as_str() == value,
                    ("stream", Scalar::String(value)) => row.stream.as_str() == value,
                    ("timestamp", Scalar::String(value)) => {
                        stamp_text = Some(value);
                        let parsed = time::OffsetDateTime::parse(value, &time::format_description::well_known::Rfc3339)
                            .map_err(|_| invalid("native plot timestamp is invalid"))?;
                        timestamp = Some(parsed.unix_timestamp_nanos());
                        row.record.source_time == Some(parsed.into())
                    }
                    _ => true,
                };
                if !matches {
                    return Err(invalid("typed telemetry row differs from storage envelope"));
                }
            }
            if let [
                PathElement::Field { name: "metadata", .. },
                PathElement::Field { name: "scope", .. },
                PathElement::Field { name: "test_execution_id", .. },
            ] = path
                && let Scalar::String(value) = scalar
            {
                scope = Some(value);
            }
            Ok::<_, Error>(())
        })
        .map_err(|_| invalid("native plot row is invalid"))?;
    if scope != row.record.scope.as_ref().map(|id| id.as_str()) {
        return Err(invalid("typed telemetry scope differs from storage envelope"));
    }
    Ok((
        values,
        sequence.ok_or_else(|| invalid("native plot sequence is absent"))?,
        timestamp.ok_or_else(|| invalid("native plot timestamp is absent"))?,
        stamp_text.ok_or_else(|| invalid("native plot timestamp is absent"))?.to_owned(),
    ))
}

// One Projected Scan Counts The Selection And Its Finite Values. A Second Scan
// Runs Only When A Window Or A Missing Value Narrows What The Summary Covers.
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

fn invalid(reason: &'static str) -> Error {
    Error::Storage(crate::storage::Error::Invalid(reason))
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

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
