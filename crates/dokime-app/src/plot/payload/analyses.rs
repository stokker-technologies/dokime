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
use std::collections::BTreeMap;

// Third Party Imports
use harmos::signal::{Series, Summary};
use serde::Serialize;

// Local Imports
use super::{PlotPayloadRequest, PlotPoint, PlotSeries, parse_timestamp};
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
}

/* === Enums === */
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PlotAnalysis {
    Error { message: String },
    Histogram { series: Vec<HistogramSeries> },
    BoxSummary { series: Vec<BoxSummary> },
    SigmaInterval { series: Vec<SigmaSeries> },
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(super) fn analyze(
    series: &mut [PlotSeries],
    request: &PlotPayloadRequest,
) -> Option<PlotAnalysis> {
    if request
        .z_expression
        .as_ref()
        .is_some_and(|expression| !expression.transforms.is_empty())
    {
        return Some(PlotAnalysis::Error {
            message: "Z expressions must select a scalar field without transforms".into(),
        });
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
                return Some(PlotAnalysis::Error {
                    message: "histogram plots bin one x field or x expression; none was declared"
                        .into(),
                });
            }
            if let Some(unit) = &request.histogram.unit {
                for entry in series.iter_mut() {
                    entry.unit = Some(unit.clone());
                }
            }
            Some(PlotAnalysis::Histogram {
                series: series
                    .iter()
                    .map(|entry| histogram(entry, request.histogram.bins))
                    .collect(),
            })
        }
        PlotType::BoxSummary => Some(PlotAnalysis::BoxSummary {
            series: series
                .iter()
                .flat_map(|entry| grouped_boxes(entry, request))
                .collect(),
        }),
        PlotType::XyInterval => Some(PlotAnalysis::SigmaInterval {
            series: series
                .iter()
                .map(|entry| interval(entry, request))
                .collect(),
        }),
        _ => None,
    }
}

pub(super) fn reduce_spectrum(points: Vec<PlotPoint>, max_points: usize) -> (Vec<PlotPoint>, bool) {
    if max_points == 0 || points.len() <= max_points {
        return (points, false);
    }
    let magnitudes = points
        .iter()
        .map(|point| {
            point
                .y
                .filter(|value| value.is_finite())
                .unwrap_or(f64::NEG_INFINITY)
        })
        .collect::<Vec<_>>();
    let mut selected = harmos::signal::extrema(&magnitudes, (max_points / 2).max(1));
    selected.sort_unstable();
    selected.dedup();
    let reduced = selected
        .into_iter()
        .filter_map(|index| points.get(index).cloned())
        .collect();
    (reduced, true)
}

/* === Private Functions === */
fn values(entry: &PlotSeries) -> Vec<f64> {
    entry
        .points
        .iter()
        .filter_map(|point| point.y.filter(|value| value.is_finite()))
        .collect()
}

fn histogram(entry: &PlotSeries, bins: usize) -> HistogramSeries {
    let values = values(entry);
    let min = values.iter().copied().reduce(f64::min).unwrap_or(0.0);
    let max = values.iter().copied().reduce(f64::max).unwrap_or(0.0);
    let bins = bins.max(1);
    let width = if min == max {
        1.0
    } else {
        (max - min) / bins as f64
    };
    let mut result = (0..bins)
        .map(|i| HistogramBin {
            lower: min + i as f64 * width,
            upper: min + (i + 1) as f64 * width,
            count: 0,
        })
        .collect::<Vec<_>>();
    for value in &values {
        result[(((value - min) / width).floor() as usize).min(bins - 1)].count += 1;
    }
    HistogramSeries {
        key: entry.key.clone(),
        bins: result,
        count: values.len(),
        statistics: histogram_statistics(values),
    }
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
    if request.x_expression.is_none()
        && request
            .x_key
            .as_deref()
            .is_none_or(|key| key == "timestamp")
    {
        return box_summary(entry).into_iter().collect();
    }
    let mut groups = BTreeMap::<String, Vec<PlotPoint>>::new();
    for point in &entry.points {
        let label = point
            .x
            .as_str()
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| point.x.to_string());
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
    let lower = values
        .iter()
        .copied()
        .find(|value| *value >= q1 - 1.5 * iqr)?;
    let upper = values
        .iter()
        .rev()
        .copied()
        .find(|value| *value <= q3 + 1.5 * iqr)?;
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
        outliers: values
            .iter()
            .copied()
            .filter(|value| *value < lower || *value > upper)
            .collect(),
    })
}

pub(super) fn point_x(point: &PlotPoint) -> Option<f64> {
    point
        .x
        .as_f64()
        .or_else(|| {
            point.x.as_str().and_then(|text| {
                parse_timestamp(text)
                    .map(|value| value.unix_timestamp_nanos() as f64 / 1e9)
                    .or_else(|| text.parse().ok())
            })
        })
        .filter(|value| value.is_finite())
}

fn interval(entry: &PlotSeries, request: &PlotPayloadRequest) -> SigmaSeries {
    let options = &request.interval;
    let mut groups = BTreeMap::<i64, Vec<f64>>::new();
    if options.bin_width.is_finite()
        && options.bin_width > 0.0
        && options.sigma.is_finite()
        && options.sigma > 0.0
    {
        for point in &entry.points {
            if let (Some(x), Some(y)) = (point_x(point), point.y.filter(|value| value.is_finite()))
            {
                groups
                    .entry((x / options.bin_width).floor() as i64)
                    .or_default()
                    .push(y);
            }
        }
    }
    let bins = groups
        .into_iter()
        .map(|(index, values)| {
            let count = values.len();
            let summary = Summary::of(&values);
            let mean = summary.mean().unwrap_or(0.0);
            let sigma = summary.std().unwrap_or(0.0);
            let mut lower = mean - options.sigma * sigma;
            let mut upper = mean + options.sigma * sigma;
            if options.clip_to_observed {
                lower = lower.max(summary.min().unwrap_or(mean));
                upper = upper.min(summary.max().unwrap_or(mean));
            }
            SigmaBin {
                x: (index as f64 + 0.5) * options.bin_width,
                count,
                mean,
                lower,
                upper,
            }
        })
        .collect();
    SigmaSeries {
        key: entry.key.clone(),
        bins,
    }
}

#[cfg(feature = "fft")]
pub(super) fn fft(entry: &mut PlotSeries, request: &PlotPayloadRequest) -> Result<(), String> {
    let options = &request.fft;
    let scale = if matches!(
        options.level_unit,
        FrequencyLevelUnit::Dbu | FrequencyLevelUnit::Dbm
    ) {
        match entry.unit.as_deref() {
            Some("V" | "volt" | "volts") => 1.0,
            Some("mV") => 0.001,
            _ => {
                entry.points.clear();
                return Err("dBu and dBm spectra require voltage samples in V or mV; dBm uses a 50-ohm load".into());
            }
        }
    } else {
        1.0
    };
    let samples = values(entry)
        .into_iter()
        .map(|value| value * scale)
        .collect::<Vec<_>>();
    let n = samples.len();
    let rate = options.sample_rate_hz.or_else(|| {
        let origin = entry.points.first()?.x.as_str().and_then(parse_timestamp);
        let times = entry
            .points
            .iter()
            .map(|point| {
                if let Some(origin) = origin {
                    Some((parse_timestamp(point.x.as_str()?)? - origin).as_seconds_f64())
                } else {
                    point_x(point)
                }
            })
            .collect::<Option<Vec<_>>>()?;
        if times.len() != n || n < 2 {
            return None;
        }
        let step = (times[n - 1] - times[0]) / (n - 1) as f64;
        // A Fourier spectrum needs uniform sampling. Missing/irregular rows must not silently compress time.
        if step <= 0.0
            || times
                .windows(2)
                .any(|pair| ((pair[1] - pair[0]) - step).abs() > step.abs() * 1e-4 + 1e-9)
        {
            return None;
        }
        Some(1.0 / step)
    });
    let Some(rate) =
        rate.filter(|rate| rate.is_finite() && *rate > 0.0 && n >= 2 && n == entry.points.len())
    else {
        entry.points.clear();
        return Err("FFT requires at least two finite samples and a positive sample rate or uniformly spaced timestamps".into());
    };
    let size = n.next_power_of_two();
    let mean = if matches!(options.detrend, FftDetrend::Mean) {
        samples.iter().sum::<f64>() / n as f64
    } else {
        0.0
    };
    let mut windowed = vec![0.0; size];
    let mut gain = 0.0;
    for (index, sample) in samples.into_iter().enumerate() {
        let weight = match options.window {
            FftWindow::Rectangular => 1.0,
            FftWindow::Hann => {
                0.5 * (1.0 - (std::f64::consts::TAU * index as f64 / n as f64).cos())
            }
        };
        gain += weight;
        windowed[index] = (sample - mean) * weight;
    }
    // Window Before Padding; Preserve The Original Sample Coherent Gain.
    let spectrum = harmos::signal::transform(&windowed, rate, harmos::signal::Taper::Rectangular);
    entry.points = spectrum
        .real
        .iter()
        .zip(&spectrum.imag)
        .enumerate()
        .take(size / 2 + 1)
        .filter_map(|(index, (&real, &imaginary))| {
            let frequency = index as f64 * rate / size as f64;
            if (index == 0 && !options.include_dc)
                || options.min_frequency_hz.is_some_and(|min| frequency < min)
                || options.max_frequency_hz.is_some_and(|max| frequency > max)
            {
                return None;
            }
            let amplitude = real.hypot(imaginary) * size as f64 / gain;
            let rms = if index == 0 || index == size / 2 {
                amplitude
            } else {
                amplitude / std::f64::consts::SQRT_2
            };
            let level = match options.level_unit {
                FrequencyLevelUnit::Linear => amplitude,
                FrequencyLevelUnit::Db => 20.0 * amplitude.max(f64::MIN_POSITIVE).log10(),
                FrequencyLevelUnit::Dbu => 20.0 * (rms / 0.775).max(f64::MIN_POSITIVE).log10(),
                // Voltage amplitude into the conventional 50-ohm reference load.
                FrequencyLevelUnit::Dbm => 10.0 * (rms * rms / 0.05).max(f64::MIN_POSITIVE).log10(),
            };
            Some(PlotPoint {
                x: frequency.into(),
                y: Some(level),
                z: None,
            })
        })
        .collect();
    entry.unit = match options.level_unit {
        FrequencyLevelUnit::Linear => entry.unit.clone(),
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

/* === Tests === */
