/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Exact Plot Statistics Over Paged Series

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos::{
    Aggregate,
    signal::Summary,
    storage::query::{
        Accumulator,
        scalar::{Agg, ScalarValue},
    },
};
use rusqlite::params;

// Local Imports
use super::super::{
    PlotPayloadRequest,
    analyses::{self, BoxSummary, HistogramBin, HistogramSeries, HistogramStatistics, SigmaBin, SigmaSeries},
};
use super::{
    input::{Read, failure},
    series::{Series, scalar_number},
};
use crate::runtime::Result;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) fn histogram(series: &Series, bins: usize) -> Result<HistogramSeries> {
    let mut moments = Summary::new();
    let mut median = Accumulator::new(Agg::Of(Aggregate::Median));
    series.visit(|point| {
        if let Some(value) = point.y.filter(|value| value.is_finite()) {
            median.push(moments.count(), &ScalarValue::Float(value))?;
            moments.push(value);
        }
        Ok(())
    })?;
    let low = moments.min().unwrap_or(0.0);
    let high = moments.max().unwrap_or(0.0);
    let bins = bins.max(1);
    let width = if low == high { 1.0 } else { (high - low) / bins as f64 };
    let mut output: Vec<_> = (0..bins)
        .map(|index| HistogramBin { lower: low + index as f64 * width, upper: low + (index + 1) as f64 * width, count: 0 })
        .collect();
    series.visit(|point| {
        if let Some(value) = point.y.filter(|value| value.is_finite()) {
            output[(((value - low) / width).floor() as usize).min(bins - 1)].count += 1;
        }
        Ok(())
    })?;
    let median = scalar_number(median.finish(&mut || Ok(()))?);
    let statistics = moments.mean().zip(median).map(|(mean, median)| HistogramStatistics {
        mean,
        median,
        std: moments.std().unwrap(),
        rms: moments.rms().unwrap(),
        min: low,
        max: high,
    });
    Ok(HistogramSeries { key: series.metadata.key.clone(), bins: output, count: moments.count() as usize, statistics })
}

pub(super) fn boxes(read: &Read<'_>, series: &Series, request: &PlotPayloadRequest) -> Result<Vec<BoxSummary>> {
    read.connection.execute_batch("DELETE FROM box_values;").map_err(failure)?;
    let grouped = request.x_expression.is_some() || request.x_key.as_deref().is_some_and(|key| key != "timestamp");
    series.visit(|point| {
        let Some(value) = point.y.filter(|value| value.is_finite()) else {
            return Ok(());
        };
        let label = if grouped { point.x.as_str().map(str::to_owned).unwrap_or_else(|| point.x.to_string()) } else { String::new() };
        read.connection.execute("INSERT INTO box_values(label,value) VALUES (?1,?2)", params![label, value]).map_err(failure)?;
        Ok(())
    })?;
    let mut statement = read
        .connection
        .prepare("SELECT label,COUNT(*),MIN(value),MAX(value) FROM box_values GROUP BY label ORDER BY label")
        .map_err(failure)?;
    let mut groups = statement.query([]).map_err(failure)?;
    let mut output = Vec::new();
    while let Some(row) = groups.next().map_err(failure)? {
        let label: String = row.get(0).map_err(failure)?;
        let count: i64 = row.get(1).map_err(failure)?;
        let count = usize::try_from(count).map_err(failure)?;
        let min = row.get(2).map_err(failure)?;
        let max = row.get(3).map_err(failure)?;
        let q1 = quantile(read, &label, count, 0.25)?;
        let median = quantile(read, &label, count, 0.5)?;
        let q3 = quantile(read, &label, count, 0.75)?;
        let lower = q1 - 1.5 * (q3 - q1);
        let upper = q3 + 1.5 * (q3 - q1);
        let lower_whisker: Option<f64> = read
            .connection
            .query_row("SELECT MIN(value) FROM box_values WHERE label=?1 AND value>=?2", params![label, lower], |row| row.get(0))
            .map_err(failure)?;
        let upper_whisker: Option<f64> = read
            .connection
            .query_row("SELECT MAX(value) FROM box_values WHERE label=?1 AND value<=?2", params![label, upper], |row| row.get(0))
            .map_err(failure)?;
        let Some((lower_whisker, upper_whisker)) = lower_whisker.zip(upper_whisker) else {
            continue;
        };
        let mut statement = read
            .connection
            .prepare("SELECT value FROM box_values WHERE label=?1 AND (value<?2 OR value>?3) ORDER BY value")
            .map_err(failure)?;
        let mut rows = statement.query(params![label, lower_whisker, upper_whisker]).map_err(failure)?;
        let mut outliers = Vec::new();
        while let Some(row) = rows.next().map_err(failure)? {
            outliers.push(row.get(0).map_err(failure)?);
        }
        let (key, label) = if grouped {
            (serde_json::json!([series.metadata.key, label]).to_string(), format!("{} / {label}", series.metadata.label))
        } else {
            (series.metadata.key.clone(), series.metadata.label.clone())
        };
        output.push(BoxSummary { key, label, count, min, max, q1, median, q3, lower_whisker, upper_whisker, outliers });
    }
    Ok(output)
}

pub(super) fn interval(read: &Read<'_>, series: &Series, request: &PlotPayloadRequest) -> Result<SigmaSeries> {
    read.connection.execute_batch("DELETE FROM interval_values;").map_err(failure)?;
    let options = &request.interval;
    let valid = options.bin_width.is_finite() && options.bin_width > 0.0 && options.sigma.is_finite() && options.sigma > 0.0;
    if valid {
        series.visit(|point| {
            let Some((x, y)) = analyses::point_x(&point).zip(point.y.filter(|value| value.is_finite())) else {
                return Ok(());
            };
            let bin = (x / options.bin_width).floor() as i64;
            read.connection.execute("INSERT INTO interval_values(bin,value) VALUES (?1,?2)", params![bin, y]).map_err(failure)?;
            Ok(())
        })?;
    }
    let mut statement = read.connection.prepare("SELECT bin,value FROM interval_values ORDER BY bin,ordinal").map_err(failure)?;
    let mut rows = statement.query([]).map_err(failure)?;
    let mut output = SigmaSeries { key: series.metadata.key.clone(), bins: Vec::new(), raw_points: 0 };
    let mut current = None;
    let mut summary = Summary::new();
    while let Some(row) = rows.next().map_err(failure)? {
        let bin: i64 = row.get(0).map_err(failure)?;
        if let Some(previous) = current.filter(|previous| *previous != bin) {
            interval_bin(&mut output, previous, &summary, request);
            summary = Summary::new();
        }
        current = Some(bin);
        summary.push(row.get(1).map_err(failure)?);
    }
    if let Some(bin) = current {
        interval_bin(&mut output, bin, &summary, request);
    }
    Ok(output)
}

/* === Private Functions === */

fn quantile(read: &Read<'_>, label: &str, count: usize, fraction: f64) -> Result<f64> {
    let rank = fraction * (count - 1) as f64;
    let value = |offset: usize| -> Result<f64> {
        read.connection
            .query_row(
                "SELECT value FROM box_values WHERE label=?1 ORDER BY value LIMIT 1 OFFSET ?2",
                params![label, offset as i64],
                |row| row.get(0),
            )
            .map_err(failure)
    };
    let low = value(rank.floor() as usize)?;
    let high = value(rank.ceil() as usize)?;
    let fraction = rank - rank.floor();
    Ok(if fraction == 0.0 || low == high {
        low
    } else if low.signum() == high.signum() {
        low + (high - low) * fraction
    } else {
        low * (1.0 - fraction) + high * fraction
    })
}

fn interval_bin(output: &mut SigmaSeries, bin: i64, summary: &Summary, request: &PlotPayloadRequest) {
    let options = &request.interval;
    let mean = summary.mean().unwrap_or(0.0);
    let sigma = summary.std().unwrap_or(0.0);
    let mut lower = mean - options.sigma * sigma;
    let mut upper = mean + options.sigma * sigma;
    if options.clip_to_observed {
        lower = lower.max(summary.min().unwrap_or(mean));
        upper = upper.min(summary.max().unwrap_or(mean));
    }
    let count = summary.count() as usize;
    output.raw_points += count;
    output.bins.push(SigmaBin { x: (bin as f64 + 0.5) * options.bin_width, count, mean, lower, upper });
}

/* === Tests === */
