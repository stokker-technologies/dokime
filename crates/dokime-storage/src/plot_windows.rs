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

// Local Imports
use crate::scalar::{self, ScalarValue};
use crate::stream_windows::{self, WindowBins};
use crate::telemetry::aggregates::{self, NumericSummary};
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
            let connection = self.lock()?;
            let mut telemetry = self.telemetry.lock().map_err(|_| Error::Data("restart after the telemetry reader lock failed"))?;
            let codec = telemetry.codec(&connection, window.execution, window.stream)?;
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
        let rows = self.storage_settings()?.flush_rows.get();
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
    /// A Series' Moments; Order Statistics Belong to Explicit Aggregates, So Every Plot Refresh Stays Bounded.
    pub fn series_summary(&self, window: &NumericWindow<'_>) -> Result<PlotSeriesSummary> {
        series(&self.numeric_summary(*window)?, self.boolean_window(*window)?)
    }
    /// A Series' Moments and Display Bins: the Field's Incremental Statistics When the Window Selects Every Value,
    /// Otherwise One Pass Over the Window.
    #[tracing::instrument(name = "plot.preview", skip_all)]
    fn series_window(&self, window: NumericWindow<'_>, boolean: bool) -> Result<(PlotSeriesSummary, Vec<NumericBin>)> {
        let (moments, bins) = match self.envelope(window)? {
            Some(mut envelope) => {
                envelope.fit(window.bins.get())?;
                (envelope.summary.finish()?, envelope.buckets.values().map(Bucket::bin).collect())
            }
            None => {
                let (mut running, mut bins) = (Running::default(), WindowBins::new(&window)?);
                self.numeric_batches(window, |batch, column| {
                    running.extend(&column.values, |row| column.valid(row));
                    stream_windows::visit(batch, column, |position, time, value| bins.push(position, time, value))
                })?;
                (running.finish()?, bins.finish())
            }
        };
        Ok((series(&moments, boolean)?, bins))
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
        let (median, ranks) = self.numeric_order_statistics(*window, &[5, 95, 99])?;
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
    /// Exact Counts on the Narrowest Power-of-Two Bins Within `bins`, From the Field's Incremental Histogram When the
    /// Window Selects Every Value, Otherwise From One Pass Over the Window.
    pub fn histogram_window(&self, window: NumericWindow<'_>, bins: usize) -> Result<PlotHistogramSeries> {
        if bins == 0 {
            return Err(Error::Data("supply at least one histogram bin"));
        }
        let (summary, grid) = match self.envelope(window)? {
            Some(envelope) => (envelope.summary.finish()?, envelope.histogram),
            None => {
                let (mut running, mut grid) = (Running::default(), Grid::new(bins));
                self.numeric_batches(window, |_, column| {
                    running.extend(&column.values, |row| column.valid(row));
                    grid.extend(&column.values, |row| column.valid(row));
                    Ok(())
                })?;
                (running.finish()?, grid)
            }
        };
        histogram(&summary, &grid, bins)
    }
    pub fn box_window(&self, window: NumericWindow<'_>, maximum_outliers: usize) -> Result<PlotBoxSummary> {
        let summary = self.numeric_summary(window)?;
        if summary.count == 0 {
            return Ok(Default::default());
        }
        let budget = self.storage_settings()?.query_bytes.get();
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
        let field = FieldPath::from(format!("values.{}", window.field).as_str());
        let mut values = aggregates::rank_values(
            |push| {
                self.evaluated_batches(window, &[window.field.into()], |batch| {
                    for row in 0..batch.num_rows() {
                        push(field.value(&batch, row)?)?;
                    }
                    Ok(true)
                })
            },
            &statistics,
            self.storage_settings()?.query_bytes.get(),
        )?;
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

fn series(moments: &NumericSummary, boolean: bool) -> Result<PlotSeriesSummary> {
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

fn histogram(summary: &NumericSummary, grid: &Grid, bins: usize) -> Result<PlotHistogramSeries> {
    if summary.count == 0 {
        return Ok(Default::default());
    }
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
            std: summary.standard_deviation,
            rms: summary.rms,
            min: Some(scalar::wire(&summary.min)?),
            max: Some(scalar::wire(&summary.max)?),
            ..Default::default()
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
