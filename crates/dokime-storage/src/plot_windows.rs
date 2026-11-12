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

// Local Imports
use crate::{NumericWindow, Result, Store};
use dokime_protocol::v1;
use rusqlite::params;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Store {
    pub fn numeric_statistics(&self, window: &NumericWindow<'_>) -> Result<v1::Statistics> {
        let connection = self.lock()?;
        let condition = "execution=?1 AND stream=?2 AND field=?3 AND source_ns>=?4 AND source_ns<=?5 AND position>?6 AND position<=?7 AND number IS NOT NULL";
        let parameters = params![
            window.execution,
            window.stream,
            window.field,
            window.start_ns,
            window.end_ns,
            window.after as i64,
            window.through as i64
        ];
        let (count,min,max,mean,squares): (i64,Option<f64>,Option<f64>,Option<f64>,Option<f64>) = connection.query_row(&format!("SELECT COUNT(*),MIN(number),MAX(number),AVG(number),SUM(number*number) FROM sample_fields INDEXED BY sample_fields_time WHERE {condition}"), parameters, |row| Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?,row.get(4)?)))?;
        if count == 0 {
            return Ok(v1::Statistics::default());
        }
        let mean = mean.unwrap();
        let variance: f64 = connection.query_row(
            &format!("SELECT AVG((number-?8)*(number-?8)) FROM sample_fields INDEXED BY sample_fields_time WHERE {condition}"),
            params![
                window.execution,
                window.stream,
                window.field,
                window.start_ns,
                window.end_ns,
                window.after as i64,
                window.through as i64,
                mean
            ],
            |row| row.get(0),
        )?;
        let percentile = |rank: f64| -> Result<f64> {
            let index = rank * (count - 1) as f64;
            let values = connection
                .prepare(&format!(
                    "SELECT number FROM sample_fields INDEXED BY sample_fields_time WHERE {condition} ORDER BY number LIMIT 2 OFFSET ?8"
                ))?
                .query_map(
                    params![
                        window.execution,
                        window.stream,
                        window.field,
                        window.start_ns,
                        window.end_ns,
                        window.after as i64,
                        window.through as i64,
                        index.floor() as i64
                    ],
                    |row| row.get::<_, f64>(0),
                )?
                .collect::<std::result::Result<Vec<_>, _>>()?;
            Ok(values[0] + (values.get(1).copied().unwrap_or(values[0]) - values[0]) * index.fract())
        };
        let first = connection.query_row(
            &format!("SELECT number FROM sample_fields INDEXED BY sample_fields_time WHERE {condition} ORDER BY position LIMIT 1"),
            parameters,
            |row| row.get(0),
        )?;
        let last = connection.query_row(
            &format!("SELECT number FROM sample_fields INDEXED BY sample_fields_time WHERE {condition} ORDER BY position DESC LIMIT 1"),
            parameters,
            |row| row.get(0),
        )?;
        let _ = squares;
        Ok(v1::Statistics {
            count: count as u64,
            min: min.unwrap(),
            max: max.unwrap(),
            mean,
            stddev: variance.max(0.0).sqrt(),
            median: percentile(0.5)?,
            p05: percentile(0.05)?,
            p95: percentile(0.95)?,
            p99: percentile(0.99)?,
            first,
            last,
        })
    }
    pub fn time_series_window(&self, window: NumericWindow<'_>) -> Result<(v1::PlotColumn, v1::PlotSeriesSummary, u64)> {
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
        let column = v1::PlotColumn {
            x: points.iter().map(|(at, _)| number(*at as f64 / 1e9)).collect(),
            instants: points.iter().map(|(at, _)| instant(*at)).collect(),
            y: points.iter().map(|(_, value)| v1::NullableNumber { value: Some(*value) }).collect(),
            ..Default::default()
        };
        let mean = (count > 0).then(|| bins.iter().map(|bin| bin.mean * bin.count as f64 / count as f64).sum::<f64>());
        let summary = v1::PlotSeriesSummary {
            count,
            min: bins.iter().map(|bin| bin.min).reduce(f64::min),
            max: bins.iter().map(|bin| bin.max).reduce(f64::max),
            mean,
            rms: (count > 0).then(|| (bins.iter().map(|bin| bin.sum_squares).sum::<f64>() / count as f64).sqrt()),
            ..Default::default()
        };
        Ok((column, summary, count))
    }
    pub fn histogram_window(&self, window: NumericWindow<'_>, bins: usize) -> Result<v1::PlotHistogramSeries> {
        let statistics = self.numeric_statistics(&window)?;
        let count = statistics.count;
        if count == 0 {
            return Ok(v1::PlotHistogramSeries::default());
        }
        let width = if statistics.max == statistics.min { 1.0 } else { (statistics.max - statistics.min) / bins as f64 };
        let connection = self.lock()?;
        let mut query = connection.prepare("SELECT MIN(CAST((number-?8)/?9 AS INTEGER),?10),COUNT(*) FROM sample_fields INDEXED BY sample_fields_time WHERE execution=?1 AND stream=?2 AND field=?3 AND source_ns>=?4 AND source_ns<=?5 AND position>?6 AND position<=?7 AND number IS NOT NULL GROUP BY 1 ORDER BY 1")?;
        let mut histogram = (0..bins)
            .map(|index| v1::PlotHistogramBin {
                lower: statistics.min + index as f64 * width,
                upper: statistics.min + (index + 1) as f64 * width,
                count: 0,
            })
            .collect::<Vec<_>>();
        for entry in query.query_map(
            params![
                window.execution,
                window.stream,
                window.field,
                window.start_ns,
                window.end_ns,
                window.after as i64,
                window.through as i64,
                statistics.min,
                width,
                (bins - 1) as i64
            ],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)),
        )? {
            let (index, count) = entry?;
            histogram[index as usize].count = count as u64;
        }
        Ok(v1::PlotHistogramSeries {
            bins: histogram,
            count,
            statistics: Some(v1::PlotHistogramStatistics {
                mean: statistics.mean,
                median: statistics.median,
                std: statistics.stddev,
                rms: (statistics.stddev * statistics.stddev + statistics.mean * statistics.mean).sqrt(),
                min: statistics.min,
                max: statistics.max,
            }),
            ..Default::default()
        })
    }
    pub fn box_window(&self, window: NumericWindow<'_>, maximum_outliers: usize) -> Result<v1::PlotBoxSummary> {
        let statistics = self.numeric_statistics(&window)?;
        if statistics.count == 0 {
            return Ok(v1::PlotBoxSummary::default());
        }
        let connection = self.lock()?;
        let condition = "execution=?1 AND stream=?2 AND field=?3 AND source_ns>=?4 AND source_ns<=?5 AND position>?6 AND position<=?7 AND number IS NOT NULL";
        let quantile = |fraction: f64| -> Result<f64> {
            let index = fraction * (statistics.count - 1) as f64;
            let values = connection
                .prepare(&format!(
                    "SELECT number FROM sample_fields INDEXED BY sample_fields_time WHERE {condition} ORDER BY number LIMIT 2 OFFSET ?8"
                ))?
                .query_map(
                    params![
                        window.execution,
                        window.stream,
                        window.field,
                        window.start_ns,
                        window.end_ns,
                        window.after as i64,
                        window.through as i64,
                        index.floor() as i64
                    ],
                    |row| row.get::<_, f64>(0),
                )?
                .collect::<std::result::Result<Vec<_>, _>>()?;
            Ok(values[0] + (values.get(1).copied().unwrap_or(values[0]) - values[0]) * index.fract())
        };
        let q1 = quantile(0.25)?;
        let q3 = quantile(0.75)?;
        let lower = q1 - 1.5 * (q3 - q1);
        let upper = q3 + 1.5 * (q3 - q1);
        let (lower_whisker,upper_whisker) = connection.query_row(&format!("SELECT MIN(number),MAX(number) FROM sample_fields INDEXED BY sample_fields_time WHERE {condition} AND number>=?8 AND number<=?9"), params![window.execution,window.stream,window.field,window.start_ns,window.end_ns,window.after as i64,window.through as i64,lower,upper], |row| Ok((row.get(0)?,row.get(1)?)))?;
        let outliers = connection.prepare(&format!("SELECT number FROM sample_fields INDEXED BY sample_fields_time WHERE {condition} AND (number<?8 OR number>?9) ORDER BY number LIMIT ?10"))?
            .query_map(params![window.execution,window.stream,window.field,window.start_ns,window.end_ns,window.after as i64,window.through as i64,lower,upper,maximum_outliers as i64], |row| row.get(0))?.collect::<std::result::Result<_,_>>()?;
        Ok(v1::PlotBoxSummary {
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

fn number(value: f64) -> v1::Value {
    v1::Value {
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
