/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

mod wifi;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::BTreeMap;

// Third Party Imports
use harmos::signal::{Series, Summary};
use serde::Serialize;

// Local Imports
use super::{PlotPayloadRequest, PlotPoint, PlotSeries, PlotSeriesSummary, PlotWindow, parse_timestamp};
use crate::document::PlotType;

#[cfg(feature = "fft")]
use crate::document::{FftDetrend, FftWindow, FrequencyLevelUnit};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct HistogramBin {
    pub lower: f64,
    pub upper: f64,
    pub count: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct HistogramSeries {
    pub key: String,
    pub bins: Vec<HistogramBin>,
    pub count: usize,
    pub statistics: Option<HistogramStatistics>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct HistogramStatistics {
    pub mean: f64,
    pub median: f64,
    pub std: f64,
    pub rms: f64,
    pub min: f64,
    pub max: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct BoxSummary {
    pub key: String,
    pub label: String,
    pub count: usize,
    pub min: f64,
    pub q1: f64,
    pub median: f64,
    pub q3: f64,
    pub max: f64,
    pub lower_whisker: f64,
    pub upper_whisker: f64,
    pub outliers: Vec<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SigmaBin {
    pub x: f64,
    pub count: usize,
    pub mean: f64,
    pub lower: f64,
    pub upper: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SigmaSeries {
    pub key: String,
    pub bins: Vec<SigmaBin>,
    pub raw_points: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct WifiChannelCount {
    pub channel: i64,
    pub count: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct WifiBandScan {
    pub band: String,
    pub occupied_channels: usize,
    pub total_networks: usize,
    pub peak_count: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct WifiScan {
    pub time: String,
    pub channels: Vec<WifiChannelCount>,
    pub bands: Vec<WifiBandScan>,
    pub occupied_channels: usize,
    pub total_networks: usize,
    pub peak_count: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct WifiBandWindow {
    pub band: String,
    pub occupied_channels: usize,
}

/* === Enums === */

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PlotAnalysis {
    Error { message: String },
    Histogram { series: Vec<HistogramSeries> },
    BoxSummary { series: Vec<BoxSummary>, total: usize },
    SigmaInterval { series: Vec<SigmaSeries> },
    WifiOccupancy { channels: Vec<i64>, occupied_channels: Vec<i64>, bands: Vec<WifiBandWindow>, scans: Vec<WifiScan> },
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) fn analyze(series: &mut [PlotSeries], request: &PlotPayloadRequest) -> Option<PlotAnalysis> {
    if request.z_expression.as_ref().is_some_and(|expression| !expression.transforms.is_empty()) {
        return Some(PlotAnalysis::Error { message: "Z expressions must select a scalar field without transforms".into() });
    }
    match request.plot_type.as_ref()? {
        PlotType::FrequencySeries => {
            for entry in series {
                if let Err(message) = fft(entry, request) {
                    return Some(PlotAnalysis::Error { message });
                }
            }
            None
        }
        PlotType::Histogram => {
            if request.y_keys.is_empty() && request.y_expressions.is_empty() {
                return Some(PlotAnalysis::Error { message: "histogram plots bin one x field or x expression; none was declared".into() });
            }
            if let Some(unit) = &request.histogram.unit {
                for entry in series.iter_mut() {
                    entry.unit = Some(unit.clone());
                }
            }
            Some(PlotAnalysis::Histogram { series: series.iter().map(|entry| histogram(entry, request.histogram.bins)).collect() })
        }
        PlotType::BoxSummary => {
            let boxes: Vec<BoxSummary> = series.iter().flat_map(|entry| grouped_boxes(entry, request)).collect();
            let total = boxes.iter().map(|summary| summary.count).sum();
            Some(PlotAnalysis::BoxSummary { series: boxes, total })
        }
        PlotType::XyInterval => Some(match interval_analysis(series, request) {
            Ok(analysis) => analysis,
            Err(message) => PlotAnalysis::Error { message: message.into() },
        }),
        PlotType::WifiChannelOccupancy => Some(match wifi::analyze(series) {
            Ok(analysis) => analysis,
            Err(message) => PlotAnalysis::Error { message: message.into() },
        }),
        _ => None,
    }
}

#[cfg(feature = "storage")]
pub(super) fn summarize(series: &[PlotSeries], request: &PlotPayloadRequest) -> Vec<PlotSeriesSummary> {
    if !summarizes(request) {
        return Vec::new();
    }
    let inside = |x: f64| request.summary_window.is_none_or(|window| window.from <= x && x <= window.to);
    let shown = |entry: &PlotSeries| {
        entry.points.iter().filter_map(|point| Some((point_x(point)?, point.y?))).filter(|(x, _)| inside(*x)).collect::<Vec<_>>()
    };
    let rows = series.iter().map(shown).collect::<Vec<_>>();
    let window = request.summary_window.or_else(|| {
        let mut xs = rows.iter().flatten().map(|(x, _)| *x);
        let first = xs.next()?;
        let (from, to) = xs.fold((first, first), |(from, to), x| (from.min(x), to.max(x)));
        Some(PlotWindow { from, to })
    });
    series
        .iter()
        .zip(rows)
        .map(|(entry, rows)| {
            let series = Series::from(rows.into_iter().map(|(_, y)| y).collect::<Vec<_>>());
            // One pass answers the moments; the median alone needs the order.
            let summary = series.summary();
            PlotSeriesSummary {
                series_id: entry.key.clone(),
                count: summary.count() as usize,
                min: summary.min(),
                max: summary.max(),
                mean: summary.mean(),
                median: series.median(),
                std: summary.std(),
                rms: summary.rms(),
                window,
            }
        })
        .collect()
}

pub(super) fn summarizes(request: &PlotPayloadRequest) -> bool {
    request.analysis.is_none() && matches!(request.plot_type, Some(PlotType::TimeSeries | PlotType::XyScatter))
}

pub(super) fn point_x(point: &PlotPoint) -> Option<f64> {
    point
        .x
        .as_f64()
        .or_else(|| {
            point
                .x
                .as_str()
                .and_then(|text| parse_timestamp(text).map(|value| value.unix_timestamp_nanos() as f64 / 1e9).or_else(|| text.parse().ok()))
        })
        .filter(|value| value.is_finite())
}

#[cfg(feature = "fft")]
pub(super) fn fft(entry: &mut PlotSeries, request: &PlotPayloadRequest) -> Result<(), String> {
    let (n, size, rate, scale) = match fft_input(entry, request) {
        Ok(input) => input,
        Err(error) => {
            entry.points.clear();
            return Err(error.to_owned());
        }
    };
    let options = &request.fft;
    let mean = if matches!(options.detrend, FftDetrend::Mean) {
        entry.points.iter().map(|point| point.y.expect("finite samples checked") * scale).sum::<f64>() / n as f64
    } else {
        0.0
    };
    let mut real = fft_buffer(size)?;
    let mut imaginary = fft_buffer(size)?;
    let mut gain = 0.0;
    for (index, point) in entry.points.iter().enumerate() {
        let sample = point.y.expect("finite samples checked") * scale;
        let weight = match options.window {
            FftWindow::Rectangular => 1.0,
            FftWindow::Hann => 0.5 * (1.0 - (std::f64::consts::TAU * index as f64 / n as f64).cos()),
        };
        gain += weight;
        real[index] = (sample - mean) * weight;
    }
    // The synchronous analysis interface has no cancellation context. This preserves
    // its existing uninterrupted execution; the kernel itself allocates no workspace.
    harmos::signal::fft_in_place(&mut real, &mut imaginary, || true).map_err(|_| "FFT exceeds finite range".to_owned())?;
    let bins = size / 2 + 1;
    let selected_frequency = |index| fft_frequency(index, size, rate, request);
    let count = (0..bins).filter_map(selected_frequency).count();
    let mut points = Vec::new();
    points.try_reserve_exact(count).map_err(|_| "FFT output allocation failed".to_owned())?;
    for index in 0..bins {
        let Some(frequency) = selected_frequency(index) else {
            continue;
        };
        // Match transform's forward normalization before applying coherent gain.
        let mut real = real[index] / size as f64 * 2.0;
        let mut imaginary = imaginary[index] / size as f64 * 2.0;
        if index == 0 || index == size / 2 {
            real /= 2.0;
            imaginary /= 2.0;
        }
        let amplitude = real.hypot(imaginary) * size as f64 / gain;
        let rms = if index == 0 || index == size / 2 { amplitude } else { amplitude / std::f64::consts::SQRT_2 };
        let level = match options.level_unit {
            FrequencyLevelUnit::Linear => amplitude,
            FrequencyLevelUnit::Db => 20.0 * amplitude.max(f64::MIN_POSITIVE).log10(),
            FrequencyLevelUnit::Dbu => 20.0 * (rms / 0.775).max(f64::MIN_POSITIVE).log10(),
            // Voltage amplitude into the conventional 50-ohm reference load.
            FrequencyLevelUnit::Dbm => 10.0 * (rms * rms / 0.05).max(f64::MIN_POSITIVE).log10(),
        };
        points.push(PlotPoint { x: frequency.into(), y: Some(level), z: None });
    }
    entry.points = points;
    entry.unit = match options.level_unit {
        FrequencyLevelUnit::Linear => entry.unit.take(),
        FrequencyLevelUnit::Db => Some("dB".into()),
        FrequencyLevelUnit::Dbu => Some("dBu".into()),
        FrequencyLevelUnit::Dbm => Some("dBm".into()),
    };
    Ok(())
}

#[cfg(not(feature = "fft"))]
pub(super) fn fft(entry: &mut PlotSeries, _: &PlotPayloadRequest) -> Result<(), String> {
    entry.points.clear();
    Err("FFT analysis is unavailable in this build".into())
}

/* === Private Functions === */

#[cfg(feature = "fft")]
fn fft_frequency(index: usize, size: usize, rate: f64, request: &PlotPayloadRequest) -> Option<f64> {
    let frequency = index as f64 * rate / size as f64;
    let options = &request.fft;
    let omitted_dc = index == 0 && !options.include_dc;
    let below = options.min_frequency_hz.is_some_and(|min| frequency < min);
    let above = options.max_frequency_hz.is_some_and(|max| frequency > max);
    (!omitted_dc && !below && !above).then_some(frequency)
}

#[cfg(feature = "fft")]
fn fft_input(entry: &PlotSeries, request: &PlotPayloadRequest) -> Result<(usize, usize, f64, f64), &'static str> {
    let options = &request.fft;
    let scale = if matches!(options.level_unit, FrequencyLevelUnit::Dbu | FrequencyLevelUnit::Dbm) {
        match entry.unit.as_deref() {
            Some("V" | "volt" | "volts") => 1.0,
            Some("mV") => 0.001,
            _ => {
                return Err("dBu and dBm spectra require voltage samples in V or mV; dBm uses a 50-ohm load");
            }
        }
    } else {
        1.0
    };
    let n = entry.points.len();
    if entry.points.iter().any(|point| point.y.is_none_or(|value| !value.is_finite())) {
        return Err("FFT requires finite samples without missing rows");
    }
    let rate = options.sample_rate_hz.or_else(|| {
        let origin = entry.points.first()?.x.as_str().and_then(parse_timestamp);
        let time = |point: &PlotPoint| {
            if let Some(origin) = origin { Some((parse_timestamp(point.x.as_str()?)? - origin).as_seconds_f64()) } else { point_x(point) }
        };
        if n < 2 {
            return None;
        }
        let first = time(entry.points.first()?)?;
        let last = time(entry.points.last()?)?;
        let step = (last - first) / (n - 1) as f64;
        // Missing/irregular rows must not silently compress time.
        if step <= 0.0 {
            return None;
        }
        let mut previous = first;
        for point in &entry.points[1..] {
            let current = time(point)?;
            if ((current - previous) - step).abs() > step.abs() * 1e-4 + 1e-9 {
                return None;
            }
            previous = current;
        }
        Some(1.0 / step)
    });
    let Some(rate) = rate.filter(|rate| rate.is_finite() && *rate > 0.0 && n >= 2) else {
        return Err("FFT requires at least two finite samples and a positive sample rate or uniformly spaced timestamps");
    };
    let size = n.checked_next_power_of_two().ok_or("FFT sample count exceeds addressable buffer size")?;
    Ok((n, size, rate, scale))
}

#[cfg(feature = "fft")]
fn fft_buffer(size: usize) -> Result<Vec<f64>, String> {
    let mut buffer = Vec::new();
    buffer.try_reserve_exact(size).map_err(|_| "FFT buffer allocation failed".to_owned())?;
    buffer.resize(size, 0.0);
    Ok(buffer)
}

fn values(entry: &PlotSeries) -> Vec<f64> {
    entry.points.iter().filter_map(|point| point.y.filter(|value| value.is_finite())).collect()
}

fn histogram(entry: &PlotSeries, bins: usize) -> HistogramSeries {
    let values = values(entry);
    let min = values.iter().copied().reduce(f64::min).unwrap_or(0.0);
    let max = values.iter().copied().reduce(f64::max).unwrap_or(0.0);
    let bins = bins.max(1);
    let width = if min == max { 1.0 } else { (max - min) / bins as f64 };
    let mut result = (0..bins)
        .map(|i| HistogramBin { lower: min + i as f64 * width, upper: min + (i + 1) as f64 * width, count: 0 })
        .collect::<Vec<_>>();
    for value in &values {
        result[(((value - min) / width).floor() as usize).min(bins - 1)].count += 1;
    }
    HistogramSeries { key: entry.key.clone(), bins: result, count: values.len(), statistics: histogram_statistics(values) }
}

fn histogram_statistics(values: Vec<f64>) -> Option<HistogramStatistics> {
    let series = Series::from(values);
    Some(HistogramStatistics {
        mean: series.mean()?,
        median: series.median()?,
        std: series.std()?,
        rms: series.rms()?,
        min: series.min()?,
        max: series.max()?,
    })
}

fn grouped_boxes(entry: &PlotSeries, request: &PlotPayloadRequest) -> Vec<BoxSummary> {
    if request.x_expression.is_none() && request.x_key.as_deref().is_none_or(|key| key == "timestamp") {
        return box_summary(entry).into_iter().collect();
    }
    let mut groups = BTreeMap::<String, Vec<PlotPoint>>::new();
    for point in &entry.points {
        let label = point.x.as_str().map(ToOwned::to_owned).unwrap_or_else(|| point.x.to_string());
        groups.entry(label).or_default().push(point.clone());
    }
    groups
        .into_iter()
        .filter_map(|(label, points)| {
            let group = PlotSeries {
                key: serde_json::json!([entry.key, label]).to_string(),
                label: format!("{} / {label}", entry.label),
                points,
                ..entry.clone()
            };
            box_summary(&group)
        })
        .collect()
}

fn box_summary(entry: &PlotSeries) -> Option<BoxSummary> {
    let mut series = Series::from(values(entry));
    series.sort();
    let q1 = series.percentile(25.0)?;
    let median = series.median()?;
    let q3 = series.percentile(75.0)?;
    let values = series.values();
    let iqr = q3 - q1;
    let lower = values.iter().copied().find(|value| *value >= q1 - 1.5 * iqr)?;
    let upper = values.iter().rev().copied().find(|value| *value <= q3 + 1.5 * iqr)?;
    Some(BoxSummary {
        key: entry.key.clone(),
        label: entry.label.clone(),
        count: values.len(),
        min: values[0],
        max: values[values.len() - 1],
        q1,
        q3,
        median,
        lower_whisker: lower,
        upper_whisker: upper,
        outliers: values.iter().copied().filter(|value| *value < lower || *value > upper).collect(),
    })
}

fn interval_analysis(series: &[PlotSeries], request: &PlotPayloadRequest) -> Result<PlotAnalysis, &'static str> {
    let mut result = interval_slots(series.len())?;
    for entry in series {
        result.push(interval(entry, request)?);
    }
    Ok(PlotAnalysis::SigmaInterval { series: result })
}

fn interval(entry: &PlotSeries, request: &PlotPayloadRequest) -> Result<SigmaSeries, &'static str> {
    let options = &request.interval;
    let mut groups = BTreeMap::<i64, Summary>::new();
    for (index, value) in interval_values(entry, request) {
        groups.entry(index).or_default().push(value);
    }
    let mut bins = interval_slots(groups.len())?;
    let mut raw_points = 0usize;
    for (index, summary) in groups {
        let count = usize::try_from(summary.count()).map_err(|_| "interval size overflows")?;
        raw_points = raw_points.checked_add(count).ok_or("interval size overflows")?;
        let mean = summary.mean().unwrap_or(0.0);
        let sigma = summary.std().unwrap_or(0.0);
        let mut lower = mean - options.sigma * sigma;
        let mut upper = mean + options.sigma * sigma;
        if options.clip_to_observed {
            lower = lower.max(summary.min().unwrap_or(mean));
            upper = upper.min(summary.max().unwrap_or(mean));
        }
        bins.push(SigmaBin { x: (index as f64 + 0.5) * options.bin_width, count, mean, lower, upper });
    }
    let mut key = String::new();
    key.try_reserve_exact(entry.key.len()).map_err(|_| "interval allocation failed")?;
    key.push_str(&entry.key);
    Ok(SigmaSeries { key, raw_points, bins })
}

fn interval_values<'a>(entry: &'a PlotSeries, request: &'a PlotPayloadRequest) -> impl Iterator<Item = (i64, f64)> + 'a {
    let options = &request.interval;
    let valid = options.bin_width.is_finite() && options.bin_width > 0.0 && options.sigma.is_finite() && options.sigma > 0.0;
    entry.points.iter().filter_map(move |point| {
        if !valid {
            return None;
        }
        let x = point_x(point)?;
        let y = point.y.filter(|value| value.is_finite())?;
        Some(((x / options.bin_width).floor() as i64, y))
    })
}

fn interval_slots<T>(count: usize) -> Result<Vec<T>, &'static str> {
    let mut values = Vec::new();
    values.try_reserve_exact(count).map_err(|_| "interval allocation failed")?;
    Ok(values)
}

#[cfg(feature = "storage")]
#[cfg(feature = "storage")]
/* === Tests === */
#[cfg(all(test, feature = "fft"))]
mod tests {
    use super::*;

    fn series(samples: &[f64]) -> PlotSeries {
        PlotSeries {
            key: "spectrum".into(),
            label: "Spectrum".into(),
            unit: Some("V".into()),
            points: samples.iter().enumerate().map(|(index, &value)| PlotPoint { x: index.into(), y: Some(value), z: None }).collect(),
            raw_points: samples.len(),
            downsampled: false,
            format: None,
        }
    }

    fn request() -> PlotPayloadRequest {
        let mut request = PlotPayloadRequest::default();
        request.fft.window = FftWindow::Rectangular;
        request.fft.detrend = FftDetrend::None;
        request.fft.include_dc = true;
        request.fft.level_unit = FrequencyLevelUnit::Linear;
        request
    }

    fn assert_levels(series: &PlotSeries, expected: &[f64]) {
        assert_eq!(series.points.len(), expected.len());
        for (point, expected) in series.points.iter().zip(expected) {
            assert!((point.y.unwrap() - expected).abs() < 1e-12);
        }
    }

    #[test]
    fn fft_forward_levels() {
        // DC=2, interior cosine amplitude=3, Nyquist amplitude=4.
        let mut input = series(&[9.0, -2.0, 3.0, -2.0]);
        fft(&mut input, &request()).unwrap();
        assert_levels(&input, &[2.0, 3.0, 4.0]);
        assert_eq!(input.points.iter().map(|point| point.x.as_f64().unwrap()).collect::<Vec<_>>(), [0.0, 0.25, 0.5]);

        let mut options = request();
        options.fft.detrend = FftDetrend::Mean;
        let mut input = series(&[9.0, -2.0, 3.0, -2.0]);
        fft(&mut input, &options).unwrap();
        assert_levels(&input, &[0.0, 3.0, 4.0]);
    }

    #[test]
    fn fft_hann_gain_before_padding() {
        // Three original samples have weights [0, .75, .75], then pad to four.
        let mut input = series(&[1.0, 1.0, 1.0]);
        let mut options = request();
        options.fft.window = FftWindow::Hann;
        fft(&mut input, &options).unwrap();
        assert_levels(&input, &[1.0, std::f64::consts::SQRT_2, 0.0]);
    }

    #[test]
    fn fft_voltage_levels() {
        for (unit, expected) in [
            (FrequencyLevelUnit::Db, 0.0),
            (FrequencyLevelUnit::Dbu, 20.0 * (1.0_f64 / 0.775).log10()),
            (FrequencyLevelUnit::Dbm, 10.0 * (1.0_f64 / 0.05).log10()),
        ] {
            let mut input = series(&[1000.0; 4]);
            input.unit = Some("mV".into());
            let mut options = request();
            options.fft.level_unit = unit;
            options.fft.max_frequency_hz = Some(0.0);
            // Plain dB preserves the original measurement unit.
            if matches!(options.fft.level_unit, FrequencyLevelUnit::Db) {
                input = series(&[1.0; 4]);
            }
            fft(&mut input, &options).unwrap();
            assert_levels(&input, &[expected]);
        }
    }

    #[test]
    fn fft_rejects_missing_or_irregular_samples() {
        let mut missing = series(&[1.0; 4]);
        missing.points[1].y = None;
        assert!(fft(&mut missing, &request()).is_err());
        assert!(missing.points.is_empty());

        let mut irregular = series(&[1.0; 4]);
        irregular.points[1].x = 1.5.into();
        assert!(fft(&mut irregular, &request()).is_err());
        assert!(irregular.points.is_empty());
    }
}

#[cfg(test)]
mod interval_tests {
    use super::*;

    fn input() -> PlotSeries {
        PlotSeries {
            key: "interval".into(),
            label: "Interval".into(),
            unit: None,
            points: vec![
                PlotPoint { x: (-0.1).into(), y: Some(1.0), z: None },
                PlotPoint { x: 0.0.into(), y: Some(2.0), z: None },
                PlotPoint { x: 0.9.into(), y: Some(4.0), z: None },
                PlotPoint { x: 1.0.into(), y: Some(10.0), z: None },
                PlotPoint { x: 2.0.into(), y: Some(f64::INFINITY), z: None },
                PlotPoint { x: 3.0.into(), y: None, z: None },
            ],
            raw_points: 6,
            downsampled: false,
            format: None,
        }
    }

    #[test]
    fn interval_floor_and_finite_rows() {
        let mut request = PlotPayloadRequest::default();
        request.interval.bin_width = 1.0;
        request.interval.sigma = 1.0;
        request.interval.clip_to_observed = false;
        let result = interval(&input(), &request).unwrap();
        assert_eq!(result.raw_points, 4);
        assert_eq!(
            result.bins.iter().map(|bin| (bin.x, bin.count, bin.mean)).collect::<Vec<_>>(),
            [(-0.5, 1, 1.0), (0.5, 2, 3.0), (1.5, 1, 10.0)]
        );
        assert_eq!((result.bins[1].lower, result.bins[1].upper), (2.0, 4.0));
    }

    #[test]
    fn invalid_interval_options_keep_empty_bins() {
        for invalid in [0.0, -1.0, f64::INFINITY, f64::NAN] {
            let mut request = PlotPayloadRequest::default();
            request.interval.bin_width = invalid;
            let result = interval(&input(), &request).unwrap();
            assert!(result.bins.is_empty());
            assert_eq!(result.raw_points, 0);
            request.interval.bin_width = 1.0;
            request.interval.sigma = invalid;
            let result = interval(&input(), &request).unwrap();
            assert!(result.bins.is_empty());
            assert_eq!(result.raw_points, 0);
        }
    }
    #[test]
    fn interval_rejects_transformed_z() {
        let mut request = PlotPayloadRequest::default();
        request.plot_type = Some(PlotType::XyInterval);
        request.z_expression = Some(crate::expressions::DataExpression {
            label: None,
            source: crate::expressions::DataSource::Dimension { name: "dimension".into() },
            transforms: vec![crate::expressions::DataTransform::PerCase],
        });
        let mut series = [input()];
        assert!(matches!(analyze(&mut series, &request), Some(PlotAnalysis::Error { .. })));
    }
}
