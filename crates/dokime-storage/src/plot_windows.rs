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

// Third Party Imports
use arrow_array::RecordBatch;

// Local Imports
use crate::{NumericWindow, Result, Store};
use dokime_protocol::v1::{self, Downsample, Value};

use dokime_protocol::client::plots::NullableNumber;
use dokime_protocol::client::plots::PlotBoxSummary;
use dokime_protocol::client::plots::PlotColumn;
use dokime_protocol::client::plots::PlotHistogramBin;
use dokime_protocol::client::plots::PlotHistogramSeries;
use dokime_protocol::client::plots::PlotHistogramStatistics;
use dokime_protocol::client::plots::PlotSeriesSummary;

use dokime_protocol::context::EvidenceStatisticsResponse;

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
        use arrow_array::{Float64Array, Int64Array, RecordBatch};
        use arrow_schema::{DataType, Field, Schema};
        use harmos_storage::query::{
            TableWriter,
            transform::{
                FieldPath, Transform,
                downsample::{Algorithm, Downsample as EngineDownsample},
            },
        };
        use std::sync::Arc;
        if limit == 0 || technique == Downsample::Lttb && limit < 3 {
            return Err(crate::Error::Data("supply a positive raw limit or at least three LTTB vertices"));
        }
        if !matches!(technique, Downsample::Raw | Downsample::Lttb) {
            return self.time_series_window(window);
        }
        let (_, summary, count) = self.time_series_window(window)?;
        if technique == Downsample::Raw && count > limit as u64 {
            return Err(crate::Error::Data("narrow the raw plot window or select downsampling"));
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
                &Transform::Downsample(EngineDownsample {
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
        let _ = std::marker::PhantomData::<RecordBatch>;
        Ok((column, summary, count))
    }
    fn read_numeric_batches(&self, window: NumericWindow<'_>, mut read: impl FnMut(RecordBatch) -> Result<()>) -> Result<()> {
        use arrow_array::{Float64Array, Int64Array, RecordBatch};
        use std::sync::Arc;
        let mut times = Vec::new();
        let mut values = Vec::new();
        let rows = self.storage_settings()?.flush_rows.get();
        self.numeric_values(window, |_, time, value| {
            times.push(time);
            values.push(value);
            if times.len() >= rows {
                let xs = times.iter().map(|time| (*time as i128 - window.start_ns as i128) as f64 / 1e9).collect::<Vec<_>>();
                read(RecordBatch::try_from_iter(vec![
                    ("time", Arc::new(Int64Array::from(std::mem::take(&mut times))) as _),
                    ("x", Arc::new(Float64Array::from(xs)) as _),
                    ("y", Arc::new(Float64Array::from(std::mem::take(&mut values))) as _),
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
    pub fn numeric_statistics(&self, window: &NumericWindow<'_>) -> Result<EvidenceStatisticsResponse> {
        let moments = self.moments(*window)?;
        if moments.count == 0 {
            return Ok(Default::default());
        }
        let ranks = self.ranks(*window, moments.count, &[5, 50, 95, 99])?;
        Ok(EvidenceStatisticsResponse {
            count: moments.count,
            min: crate::scalar::number(&moments.min).unwrap(),
            max: crate::scalar::number(&moments.max).unwrap(),
            mean: moments.mean()?,
            stddev: moments.stddev()?,
            median: crate::scalar::number(&ranks[&50]).unwrap(),
            p05: crate::scalar::number(&ranks[&5]).unwrap(),
            p95: crate::scalar::number(&ranks[&95]).unwrap(),
            p99: crate::scalar::number(&ranks[&99]).unwrap(),
            first: crate::scalar::number(&moments.first).unwrap(),
            last: crate::scalar::number(&moments.last).unwrap(),
            resolved_at: None,
        })
    }
    pub fn time_series_window(&self, window: NumericWindow<'_>) -> Result<(PlotColumn, PlotSeriesSummary, u64)> {
        let bins = self.numeric_bins(window)?;
        let count = bins.iter().map(|bin| bin.count).sum::<u64>();
        let mut points = Vec::with_capacity(bins.len() * 4);
        for bin in &bins {
            let mut extrema = [(bin.first_ns, bin.first), (bin.min_ns, bin.min), (bin.max_ns, bin.max), (bin.last_ns, bin.last)];
            extrema.sort_by_key(|point| point.0);
            for point in extrema {
                if points.last() != Some(&point) {
                    points.push(point);
                }
            }
        }
        let column = PlotColumn {
            x: points.iter().map(|(at, _)| number(*at as f64 / 1e9)).collect(),
            instants: points.iter().map(|(at, _)| instant(*at)).collect(),
            y: points.iter().map(|(_, value)| NullableNumber { value: Some(*value) }).collect(),
            ..Default::default()
        };
        let mean = (count > 0).then(|| bins.iter().map(|bin| bin.mean * bin.count as f64 / count as f64).sum::<f64>());
        let scale = bins.iter().map(|bin| bin.scale).fold(0.0, f64::max);
        let squares = bins.iter().map(|bin| if scale == 0.0 { 0.0 } else { bin.scaled_squares * (bin.scale / scale).powi(2) }).sum::<f64>();
        let summary = PlotSeriesSummary {
            count,
            min: bins.iter().map(|bin| bin.min).reduce(f64::min),
            max: bins.iter().map(|bin| bin.max).reduce(f64::max),
            mean,
            rms: (count > 0).then(|| (squares / count as f64).clamp(0.0, 1.0).sqrt() * scale),
            ..Default::default()
        };
        Ok((column, summary, count))
    }
    pub fn histogram_window(&self, window: NumericWindow<'_>, bins: usize) -> Result<PlotHistogramSeries> {
        let statistics = self.numeric_statistics(&window)?;
        if statistics.count == 0 {
            return Ok(Default::default());
        }
        if bins == 0 {
            return Err(crate::Error::Data("supply at least one histogram bin"));
        }
        let width = if statistics.max == statistics.min { 1.0 } else { (statistics.max - statistics.min) / bins as f64 };
        let mut histogram = (0..bins)
            .map(|index| PlotHistogramBin {
                lower: statistics.min + index as f64 * width,
                upper: statistics.min + (index + 1) as f64 * width,
                count: 0,
            })
            .collect::<Vec<_>>();
        self.numeric_values(window, |_, _, value| {
            histogram[(((value - statistics.min) / width) as usize).min(bins - 1)].count += 1;
            Ok(())
        })?;
        Ok(PlotHistogramSeries {
            bins: histogram,
            count: statistics.count,
            statistics: Some(PlotHistogramStatistics {
                mean: statistics.mean,
                median: statistics.median,
                std: statistics.stddev,
                rms: statistics.mean.hypot(statistics.stddev),
                min: statistics.min,
                max: statistics.max,
            }),
            ..Default::default()
        })
    }
    pub fn box_window(&self, window: NumericWindow<'_>, maximum_outliers: usize) -> Result<PlotBoxSummary> {
        let statistics = self.numeric_statistics(&window)?;
        if statistics.count == 0 {
            return Ok(Default::default());
        }
        let ranks = self.ranks(window, statistics.count, &[25, 75])?;
        let (q1, q3) = (crate::scalar::number(&ranks[&25]).unwrap(), crate::scalar::number(&ranks[&75]).unwrap());
        let lower = q1 - 1.5 * (q3 - q1);
        let upper = q3 + 1.5 * (q3 - q1);
        let mut lower_whisker = f64::INFINITY;
        let mut upper_whisker = f64::NEG_INFINITY;
        let mut outliers = Vec::<f64>::new();
        self.numeric_values(window, |_, _, value| {
            if value >= lower && value <= upper {
                lower_whisker = lower_whisker.min(value);
                upper_whisker = upper_whisker.max(value);
            } else if maximum_outliers > 0 {
                let at = outliers.partition_point(|other| other.total_cmp(&value).is_lt());
                if at < maximum_outliers {
                    outliers.insert(at, value);
                    outliers.truncate(maximum_outliers);
                }
            }
            Ok(())
        })?;
        Ok(PlotBoxSummary {
            count: statistics.count,
            min: statistics.min,
            max: statistics.max,
            q1,
            median: statistics.median,
            q3,
            lower_whisker,
            upper_whisker,
            outliers,
            ..Default::default()
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn number(value: f64) -> Value {
    Value {
        kind: Some(v1::value::Kind::Number(value)),
    }
}
fn instant(at_ns: i64) -> String {
    prost_types::Timestamp {
        seconds: at_ns.div_euclid(1_000_000_000),
        nanos: at_ns.rem_euclid(1_000_000_000) as i32,
    }
    .to_string()
}

/* === Tests === */
