/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Streaming Plot Spectrum Projection

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::super::PlotPayloadRequest;
#[cfg(feature = "fft")]
use super::super::{PlotPoint, analyses, parse_timestamp};
#[cfg(feature = "fft")]
use super::input::failure;
use super::series::{Points, Series};
#[cfg(feature = "fft")]
use crate::document::{FftDetrend, FftWindow, FrequencyLevelUnit};
use crate::runtime::Result;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[cfg(feature = "fft")]
struct Parameters {
    n: usize,
    size: usize,
    rate: f64,
    scale: f64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

#[cfg(feature = "fft")]
pub(super) fn transform(series: &mut Series, request: &PlotPayloadRequest) -> Result<Option<String>> {
    let input = input(series, request)?;
    let Parameters { n, size, rate, scale } = match input {
        Ok(input) => input,
        Err(message) => {
            let raw_points = series.metadata.raw_points;
            *series = Points::new()?.finish(series.metadata.clone())?;
            series.metadata.raw_points = raw_points;
            return Ok(Some(message.into()));
        }
    };
    let options = &request.fft;
    let mut mean = 0.0;
    if matches!(options.detrend, FftDetrend::Mean) {
        series.visit(|point| {
            mean += point.y.expect("validated samples") * scale;
            Ok(())
        })?;
        mean /= n as f64;
    }
    let mut buffer = harmos::signal::FftBuffer::new(size).map_err(failure)?;
    let mut index = 0;
    let mut gain = 0.0;
    series.visit(|point| {
        let weight = match options.window {
            FftWindow::Rectangular => 1.0,
            FftWindow::Hann => 0.5 * (1.0 - (std::f64::consts::TAU * index as f64 / n as f64).cos()),
        };
        gain += weight;
        let sample = point.y.expect("validated samples") * scale;
        buffer.set(index, ((sample - mean) * weight, 0.0)).map_err(failure)?;
        index += 1;
        Ok(())
    })?;
    match buffer.transform(|| true) {
        Ok(()) => {}
        Err(harmos::signal::Error::InvalidInput { .. }) => return Ok(Some("FFT exceeds finite range".into())),
        Err(error) => return Err(failure(error)),
    }
    let mut points = Points::new()?;
    for index in 0..size / 2 + 1 {
        let frequency = index as f64 * rate / size as f64;
        let omitted = (index == 0 && !options.include_dc)
            || options.min_frequency_hz.is_some_and(|min| frequency < min)
            || options.max_frequency_hz.is_some_and(|max| frequency > max);
        if omitted {
            continue;
        }
        let (real, imaginary) = buffer.get(index).map_err(failure)?;
        let mut real = real / size as f64 * 2.0;
        let mut imaginary = imaginary / size as f64 * 2.0;
        let edge = index == 0 || index == size / 2;
        if edge {
            real /= 2.0;
            imaginary /= 2.0;
        }
        let amplitude = real.hypot(imaginary) * size as f64 / gain;
        let rms = if edge { amplitude } else { amplitude / std::f64::consts::SQRT_2 };
        let level = match options.level_unit {
            FrequencyLevelUnit::Linear => amplitude,
            FrequencyLevelUnit::Db => 20.0 * amplitude.max(f64::MIN_POSITIVE).log10(),
            FrequencyLevelUnit::Dbu => 20.0 * (rms / 0.775).max(f64::MIN_POSITIVE).log10(),
            FrequencyLevelUnit::Dbm => 10.0 * (rms * rms / 0.05).max(f64::MIN_POSITIVE).log10(),
        };
        points.push(PlotPoint { x: frequency.into(), y: Some(level), z: None })?;
    }
    let mut metadata = series.metadata.clone();
    metadata.unit = match options.level_unit {
        FrequencyLevelUnit::Linear => metadata.unit,
        FrequencyLevelUnit::Db => Some("dB".into()),
        FrequencyLevelUnit::Dbu => Some("dBu".into()),
        FrequencyLevelUnit::Dbm => Some("dBm".into()),
    };
    let raw_points = metadata.raw_points;
    *series = points.finish(metadata)?;
    series.metadata.raw_points = raw_points;
    Ok(None)
}

#[cfg(not(feature = "fft"))]
pub(super) fn transform(series: &mut Series, _: &PlotPayloadRequest) -> Result<Option<String>> {
    let raw_points = series.metadata.raw_points;
    *series = Points::new()?.finish(series.metadata.clone())?;
    series.metadata.raw_points = raw_points;
    Ok(Some("FFT analysis is unavailable in this build".into()))
}

/* === Private Functions === */

#[cfg(feature = "fft")]
fn input(series: &Series, request: &PlotPayloadRequest) -> Result<std::result::Result<Parameters, &'static str>> {
    let options = &request.fft;
    let scale = if matches!(options.level_unit, FrequencyLevelUnit::Dbu | FrequencyLevelUnit::Dbm) {
        match series.metadata.unit.as_deref() {
            Some("V" | "volt" | "volts") => 1.0,
            Some("mV") => 0.001,
            _ => return Ok(Err("dBu and dBm spectra require voltage samples in V or mV; dBm uses a 50-ohm load")),
        }
    } else {
        1.0
    };
    let n = series.table.len();
    let mut finite = true;
    let mut first = None;
    let mut last = None;
    series.visit(|point| {
        finite &= point.y.is_some_and(|value| value.is_finite());
        if first.is_none() {
            first = Some(point.clone());
        }
        last = Some(point);
        Ok(())
    })?;
    if !finite {
        return Ok(Err("FFT requires finite samples without missing rows"));
    }
    let origin = first.as_ref().and_then(|point| point.x.as_str()).and_then(parse_timestamp);
    let time = |point: &PlotPoint| {
        if let Some(origin) = origin {
            Some((parse_timestamp(point.x.as_str()?)? - origin).as_seconds_f64())
        } else {
            analyses::point_x(point)
        }
    };
    let mut rate = options.sample_rate_hz;
    if rate.is_none()
        && n >= 2
        && let Some((first, last)) = first.as_ref().and_then(time).zip(last.as_ref().and_then(time))
    {
        let step = (last - first) / (n - 1) as f64;
        let mut previous: Option<f64> = None;
        let mut uniform = step > 0.0;
        series.visit(|point| {
            let Some(current) = time(&point) else {
                uniform = false;
                return Ok(());
            };
            if let Some(previous) = previous {
                uniform &= ((current - previous) - step).abs() <= step.abs() * 1e-4 + 1e-9;
            }
            previous = Some(current);
            Ok(())
        })?;
        if uniform {
            rate = Some(1.0 / step);
        }
    }
    let Some(rate) = rate.filter(|rate| rate.is_finite() && *rate > 0.0 && n >= 2) else {
        return Ok(Err("FFT requires at least two finite samples and a positive sample rate or uniformly spaced timestamps"));
    };
    let Some(size) = n.checked_next_power_of_two() else {
        return Ok(Err("FFT sample count exceeds addressable buffer size"));
    };
    Ok(Ok(Parameters { n, size, rate, scale }))
}

/* === Tests === */
