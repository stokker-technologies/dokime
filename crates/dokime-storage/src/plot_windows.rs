/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Reduced Plot Windows

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{collections::BTreeMap, mem, sync::Arc};

// Third Party Imports
use arrow_array::{Float64Array, Int64Array, ListArray, RecordBatch};
use arrow_schema::{DataType, Field, Schema};
use harmos_domain::model::storage::query::transform::statistics;
use harmos_query::transform::{
    FieldPath, FieldPathExecution, Transform,
    downsample::{self, Algorithm},
};
use harmos_query::{Aggregate, TableWriter};
use prost_types::Timestamp;
use rusqlite::{OptionalExtension, params};
use serde::{Deserialize, Serialize};

// Local Imports
use crate::scalar::{self, ScalarValue};
use crate::stream_windows::{self, WindowBins};
use crate::telemetry::aggregates::{NumericSummary, Ranked};
use crate::telemetry::previews::Bucket;
use crate::telemetry::summary::{Grid, Running};
use crate::{Error, NumericBin, NumericWindow, Result, Store};
use dokime_protocol::model::Downsample;
use dokime_protocol::model::Value;

use dokime_protocol::host::plots::NullableNumber;
use dokime_protocol::host::plots::PlotBoxSummary;
use dokime_protocol::host::plots::PlotColumn;
use dokime_protocol::host::plots::PlotHistogramBin;
use dokime_protocol::host::plots::PlotHistogramSeries;
use dokime_protocol::host::plots::PlotHistogramStatistics;
use dokime_protocol::host::plots::PlotSeriesSummary;

use dokime_protocol::host::Statistics;

use dokime_protocol::model::value;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/// One Window's Moments, Its Exact Median (None While a Live Full History Waits for Its Seal) and What a Plot Draws.
struct Selected {
    moments: NumericSummary,
    median: Option<ScalarValue>,
    bins: Vec<NumericBin>,
    grid: Option<Grid>,
}

/// A Sealed Field's Exact Full-Range Median and Percentiles.
#[derive(Serialize, Deserialize)]
struct Ranks {
    median: ScalarValue,
    percentiles: BTreeMap<u8, ScalarValue>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Store {
    pub fn sampled_time_series_window(
        &self, window: NumericWindow<'_>, technique: Downsample, limit: usize,
    ) -> Result<(PlotColumn, PlotSeriesSummary, u64)> {
        if limit == 0 {
            return Err(Error::Data("supply a positive raw limit or at least three LTTB vertices"));
        }
        let boolean = self.boolean_window(window)?;
        let (summary, bins) = self.series_window(window, boolean)?;
        let count = summary.count;
        if count > limit as u64 && !boolean && technique == Downsample::Lttb && limit < 3 {
            return Err(Error::Data("supply at least three LTTB vertices or include every selected sample"));
        }
        if technique == Downsample::Raw && count > limit as u64 {
            return Err(Error::Data("narrow the raw plot window or select downsampling"));
        }
        if count > limit as u64 && (boolean || !matches!(technique, Downsample::Raw | Downsample::Lttb)) {
            return Ok((binned(&bins, boolean), summary, count));
        }
        let schema = Arc::new(Schema::new(vec![
            Field::new("time", DataType::Int64, false),
            Field::new("x", DataType::Float64, false),
            Field::new("y", DataType::Float64, false),
        ]));
        let mut writer = TableWriter::new(schema.clone())?;
        self.read_numeric_batches(window, |batch| {
            writer.push(&batch, &mut || Ok(()))?;
            Ok(())
        })?;
        let table = writer.finish()?;
        let table = if count > limit as u64 {
            table.transform(
                &Transform::Downsample(downsample::Downsample {
                    max_rows: limit,
                    algorithm: Algorithm::Lttb,
                    axis: Some(FieldPath { fields: vec!["x".into()] }),
                    value: Some(FieldPath { fields: vec!["y".into()] }),
                }),
                &mut || Ok(()),
            )?
        } else {
            table
        };
        let mut reader = table.reader()?;
        let mut column = PlotColumn::default();
        while let Some(batch) = reader.next(&mut || Ok(()))? {
            let times = batch.column(0).as_any().downcast_ref::<Int64Array>().unwrap();
            let ys = batch.column(2).as_any().downcast_ref::<Float64Array>().unwrap();
            for row in 0..batch.num_rows() {
                column.instants.push(instant(times.value(row)));
                column.x.push(number(times.value(row) as f64 / 1e9));
                column.y.push(NullableNumber { value: Some(ys.value(row)) });
            }
        }
        Ok((column, summary, count))
    }
    fn boolean_window(&self, window: NumericWindow<'_>) -> Result<bool> {
        let field = FieldPath::from(format!("values.{}", window.field).as_str());
        if !self.has_projection(window)? {
            let codec = self.codec(&*self.lock()?, window.execution, window.stream)?;
            return Ok(field.field(codec.schema.as_ref())?.data_type() == &DataType::Boolean);
        }
        let mut boolean = false;
        self.evaluated_batches(window, &[window.field.into()], |batch| {
            boolean = field.field(batch.schema().as_ref())?.data_type() == &DataType::Boolean;
            Ok(false)
        })?;
        Ok(boolean)
    }
    fn read_numeric_batches(&self, window: NumericWindow<'_>, mut read: impl FnMut(RecordBatch) -> Result<()>) -> Result<()> {
        let mut times = Vec::new();
        let mut values = Vec::new();
        let rows = self.storage_settings().flush_rows.get();
        self.numeric_values(window, |_, time, value| {
            times.push(time);
            values.push(value);
            if times.len() >= rows {
                let xs = times.iter().map(|time| (*time as i128 - window.start_ns as i128) as f64 / 1e9).collect::<Vec<_>>();
                read(RecordBatch::try_from_iter(vec![
                    ("time", Arc::new(Int64Array::from(mem::take(&mut times))) as _),
                    ("x", Arc::new(Float64Array::from(xs)) as _),
                    ("y", Arc::new(Float64Array::from(mem::take(&mut values))) as _),
                ])?)?;
            }
            Ok(())
        })?;
        if !times.is_empty() {
            let xs = times.iter().map(|time| (*time as i128 - window.start_ns as i128) as f64 / 1e9).collect::<Vec<_>>();
            read(RecordBatch::try_from_iter(vec![
                ("time", Arc::new(Int64Array::from(times)) as _),
                ("x", Arc::new(Float64Array::from(xs)) as _),
                ("y", Arc::new(Float64Array::from(values)) as _),
            ])?)?;
        }
        Ok(())
    }
    pub fn numeric_rms(&self, window: &NumericWindow<'_>) -> Result<f64> {
        Ok(self.numeric_summary(*window)?.rms)
    }
    /// A Series' Moments and Exact Median Over Its Window.
    pub fn series_summary(&self, window: &NumericWindow<'_>) -> Result<PlotSeriesSummary> {
        series(&self.selected(*window, false, false)?, self.boolean_window(*window)?)
    }
    #[tracing::instrument(name = "plot.preview", skip_all)]
    fn series_window(&self, window: NumericWindow<'_>, boolean: bool) -> Result<(PlotSeriesSummary, Vec<NumericBin>)> {
        let selected = self.selected(window, true, false)?;
        Ok((series(&selected, boolean)?, selected.bins))
    }
    /// A Window's Moments, Exact Median and, When Asked, Display Bins or Value Histogram: the Field's Incremental
    /// Statistics When the Window Selects Every Value, Whose Median Waits for the Seal, Otherwise One Pass Over It.
    fn selected(&self, window: NumericWindow<'_>, bins: bool, histogram: bool) -> Result<Selected> {
        if let Some(mut envelope) = self.envelope(window)? {
            if bins {
                envelope.fit(window.bins.get())?;
            }
            return Ok(Selected {
                moments: envelope.summary.finish()?,
                median: self.sealed_ranks(window)?.map(|ranks| ranks.median),
                bins: if bins { envelope.buckets.values().map(Bucket::bin).collect() } else { Vec::new() },
                grid: histogram.then_some(envelope.histogram),
            });
        }
        let settings = self.storage_settings();
        let mut running = Running::default();
        let mut ranked = Ranked::new(settings.query_bytes.get());
        let mut display = bins.then(|| WindowBins::new(&window)).transpose()?;
        let mut grid = histogram.then(|| Grid::new(settings.previews.histogram_bins.get()));
        self.numeric_batches(window, |batch, column| {
            running.extend(&column.values, |row| column.valid(row));
            ranked.extend(&column.values, |row| column.valid(row))?;
            if let Some(grid) = &mut grid {
                grid.extend(&column.values, |row| column.valid(row));
            }
            match &mut display {
                Some(display) => stream_windows::visit(batch, column, |position, time, value| display.push(position, time, value)),
                None => Ok(()),
            }
        })?;
        Ok(Selected {
            moments: running.finish()?,
            median: Some(ranked.finish(&[Aggregate::Median])?.remove(0)),
            bins: display.map_or_else(Vec::new, WindowBins::finish),
            grid,
        })
    }
    /// A Sealed Field's Exact Full-Range Ranks, Ranked Once; None While Its Execution Is Open.
    fn sealed_ranks(&self, window: NumericWindow<'_>) -> Result<Option<Ranks>> {
        let sealed =
            self.lock()?.query_row("SELECT state=2 FROM executions WHERE id=?1", [window.execution], |row| row.get(0)).optional()?;
        if sealed != Some(true) {
            return Ok(None);
        }
        let key =
            format!("{}{}:{}/{}:{}", ranks_prefix(window.execution), window.stream.len(), window.stream, window.field.len(), window.field);
        let failed = || Error::Data("restart after the field rank cache failed");
        let shared = self.ranks.lock().map_err(|_| failed())?.entry(key.clone()).or_default().clone();
        // A Field Ranked by Another Reader Is Awaited, Never Ranked Twice; the Store Lock Is Not Held Meanwhile.
        let mut ranked = shared.lock().map_err(|_| failed())?;
        if ranked.is_none() {
            *ranked = self.lock()?.query_row("SELECT payload FROM administration WHERE id=?1", [&key], |row| row.get(0)).optional()?;
        }
        if ranked.is_none() {
            // One Pass Over the Whole Field Gains Nothing From Retaining Its Source Blocks for Later Readers.
            let mut reader = self.clone();
            reader.sample_reads = None;
            let (median, percentiles) = reader.numeric_order_statistics(window, &[5, 95, 99])?;
            *ranked = Some(postcard::to_allocvec(&Ranks { median, percentiles })?);
        }
        Ok(Some(postcard::from_bytes(ranked.as_deref().unwrap_or_default())?))
    }
    /// Retain a Sealed Execution's Ranked Fields So Reads After a Restart Need Not Rank Them Again.
    pub fn retain_ranks(&self, execution: &str) -> Result<()> {
        let prefix = ranks_prefix(execution);
        let failed = || Error::Data("restart after the field rank cache failed");
        let ranks = self.ranks.lock().map_err(|_| failed())?.clone();
        let mut connection = self.lock()?;
        let transaction = connection.transaction()?;
        for (key, ranked) in ranks.iter().filter(|(key, _)| key.starts_with(&prefix)) {
            if let Some(payload) = ranked.lock().map_err(|_| failed())?.as_ref() {
                transaction.execute(
                    "INSERT INTO administration VALUES (?1,?2) ON CONFLICT(id) DO UPDATE SET payload=excluded.payload",
                    params![key, payload],
                )?;
            }
        }
        transaction.commit()?;
        Ok(())
    }
    #[tracing::instrument(name = "plot.statistics", skip_all)]
    pub fn numeric_statistics(&self, window: &NumericWindow<'_>) -> Result<Statistics> {
        let moments = self.numeric_summary(*window)?;
        if moments.count == 0 || self.boolean_window(*window)? {
            return Ok(Statistics {
                count: moments.count,
                ..Default::default()
            });
        }
        let sealed = if self.envelope(*window)?.is_some() { self.sealed_ranks(*window)? } else { None };
        let (median, ranks) = match sealed {
            Some(ranks) => (ranks.median, ranks.percentiles),
            None => self.numeric_order_statistics(*window, &[5, 95, 99])?,
        };
        Ok(Statistics {
            count: moments.count,
            min: Some(scalar::wire(&moments.min)?),
            max: Some(scalar::wire(&moments.max)?),
            mean: Some(scalar::wire(&moments.average)?),
            sum: Some(scalar::wire(&moments.sum)?),
            stddev: moments.standard_deviation,
            median: Some(scalar::wire(&median)?),
            p05: Some(scalar::wire(&ranks[&5])?),
            p95: Some(scalar::wire(&ranks[&95])?),
            p99: Some(scalar::wire(&ranks[&99])?),
            first: Some(scalar::wire(&moments.first)?),
            last: Some(scalar::wire(&moments.last)?),
        })
    }
    pub fn time_series_window(&self, window: NumericWindow<'_>) -> Result<(PlotColumn, PlotSeriesSummary, u64)> {
        let boolean = self.boolean_window(window)?;
        let (summary, bins) = self.series_window(window, boolean)?;
        let count = summary.count;
        Ok((binned(&bins, boolean), summary, count))
    }
    /// The Window's Values in `bins` Equal Bins From Its Lowest to Its Highest Value, With Its Moments and Median.
    pub fn histogram_window(&self, window: NumericWindow<'_>, bins: usize) -> Result<PlotHistogramSeries> {
        if bins == 0 {
            return Err(Error::Data("supply at least one histogram bin"));
        }
        histogram(&self.selected(window, false, true)?, bins)
    }
    pub fn box_window(&self, window: NumericWindow<'_>, maximum_outliers: usize) -> Result<PlotBoxSummary> {
        let summary = self.numeric_summary(window)?;
        if summary.count == 0 {
            return Ok(Default::default());
        }
        let budget = self.storage_settings().query_bytes.get();
        let batch = self.reduce_numeric(
            window,
            &Transform::BoxSummary(statistics::BoxSummary {
                field: FieldPath::from("y"),
                max_outliers: maximum_outliers,
                memory_bytes: budget / 2,
                scratch_bytes: budget / 2,
            }),
        )?;
        let value = |name| batch.column_by_name(name).unwrap().as_any().downcast_ref::<Float64Array>().unwrap().value(0);
        let selected = batch.column_by_name("outliers").unwrap().as_any().downcast_ref::<ListArray>().unwrap().value(0);
        let mut outliers = selected.as_any().downcast_ref::<Float64Array>().unwrap().values().to_vec();
        outliers.sort_by(f64::total_cmp);
        Ok(PlotBoxSummary {
            count: summary.count,
            min: Some(scalar::wire(&summary.min)?),
            max: Some(scalar::wire(&summary.max)?),
            q1: Some(number(value("q1"))),
            median: Some(number(value("median"))),
            q3: Some(number(value("q3"))),
            lower_whisker: value("lower_whisker"),
            upper_whisker: value("upper_whisker"),
            outliers,
            ..Default::default()
        })
    }
    fn numeric_order_statistics(&self, window: NumericWindow<'_>, percentiles: &[u8]) -> Result<(ScalarValue, BTreeMap<u8, ScalarValue>)> {
        let statistics = [Aggregate::Median].into_iter().chain(percentiles.iter().copied().map(Aggregate::Percentile)).collect::<Vec<_>>();
        let mut ranked = Ranked::new(self.storage_settings().query_bytes.get());
        self.numeric_batches(window, |_, column| ranked.extend(&column.values, |row| column.valid(row)))?;
        let mut values = ranked.finish(&statistics)?;
        let median = values.remove(0);
        Ok((median, percentiles.iter().copied().zip(values).collect()))
    }

    fn reduce_numeric(&self, window: NumericWindow<'_>, transform: &Transform) -> Result<RecordBatch> {
        let schema = Arc::new(Schema::new(vec![
            Field::new("time", DataType::Int64, false),
            Field::new("x", DataType::Float64, false),
            Field::new("y", DataType::Float64, false),
        ]));
        let mut writer = TableWriter::new(schema)?;
        self.read_numeric_batches(window, |batch| {
            writer.push(&batch, &mut || Ok(()))?;
            Ok(())
        })?;
        let table = writer.finish()?.transform(transform, &mut || Ok(()))?;
        table.reader()?.next(&mut || Ok(()))?.ok_or(Error::Corrupt("numeric transform returned no result"))
    }
}

/* === Public Functions === */

/* === Private Functions === */

pub(crate) fn ranks_prefix(execution: &str) -> String {
    format!("field-ranks/{}:{execution}/", execution.len())
}

fn wired(median: &Option<ScalarValue>) -> Result<Option<Value>> {
    median.as_ref().filter(|median| !matches!(median, ScalarValue::Missing | ScalarValue::Null)).map(scalar::wire).transpose()
}

fn series(selected: &Selected, boolean: bool) -> Result<PlotSeriesSummary> {
    let moments = &selected.moments;
    if boolean || moments.count == 0 {
        return Ok(PlotSeriesSummary {
            count: moments.count,
            ..Default::default()
        });
    }
    Ok(PlotSeriesSummary {
        count: moments.count,
        min: Some(scalar::wire(&moments.min)?),
        max: Some(scalar::wire(&moments.max)?),
        mean: Some(scalar::wire(&moments.average)?),
        median: wired(&selected.median)?,
        std: Some(moments.standard_deviation),
        rms: Some(moments.rms),
        ..Default::default()
    })
}

/// A Series Drawn Through Each Bin's First, Extreme and Last Values in Time Order; Booleans Keep Their Edges.
fn binned(bins: &[NumericBin], boolean: bool) -> PlotColumn {
    let mut points = Vec::with_capacity(bins.len() * if boolean { 2 } else { 4 });
    for bin in bins {
        let mut extrema = [(bin.first_ns, bin.first), (bin.min_ns, bin.min), (bin.max_ns, bin.max), (bin.last_ns, bin.last)];
        if boolean {
            extrema[1] = extrema[3];
        }
        let extrema = if boolean { &mut extrema[..2] } else { &mut extrema[..] };
        extrema.sort_by_key(|point| point.0);
        for &point in extrema.iter() {
            if points.last() != Some(&point) {
                points.push(point);
            }
        }
    }
    PlotColumn {
        x: points.iter().map(|(at, _)| number(*at as f64 / 1e9)).collect(),
        instants: points.iter().map(|(at, _)| instant(*at)).collect(),
        y: points.iter().map(|(_, value)| NullableNumber { value: Some(*value) }).collect(),
        ..Default::default()
    }
}

fn histogram(selected: &Selected, bins: usize) -> Result<PlotHistogramSeries> {
    let summary = &selected.moments;
    let Some(grid) = selected.grid.as_ref().filter(|_| summary.count > 0) else {
        return Ok(Default::default());
    };
    Ok(PlotHistogramSeries {
        bins: grid
            .bins(bins)
            .into_iter()
            .map(|bin| PlotHistogramBin {
                lower: bin.lower,
                upper: bin.upper,
                count: bin.count,
            })
            .collect(),
        count: summary.count,
        statistics: Some(PlotHistogramStatistics {
            mean: Some(scalar::wire(&summary.average)?),
            median: wired(&selected.median)?,
            std: summary.standard_deviation,
            rms: summary.rms,
            min: Some(scalar::wire(&summary.min)?),
            max: Some(scalar::wire(&summary.max)?),
        }),
        ..Default::default()
    })
}

fn number(value: f64) -> Value {
    Value { kind: Some(value::Kind::Number(value)) }
}
fn instant(at_ns: i64) -> String {
    Timestamp {
        seconds: at_ns.div_euclid(1_000_000_000),
        nanos: at_ns.rem_euclid(1_000_000_000) as i32,
    }
    .to_string()
}

/* === Tests === */
