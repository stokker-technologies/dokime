/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

use super::{DownsampleTechnique, PlotPayload, PlotPayloadRequest, PlotPoint, analyses, cartesian};
use crate::document::PlotType;
use crate::runtime::Result;
use harmos::storage::ReadBatch;

/* === Public Functions === */

pub(crate) fn project(
    telemetry: &[crate::telemetry::TelemetrySample], records: crate::records::view::View<'_>, request: &PlotPayloadRequest,
    generated_at: &str, context: &super::PlotPayloadContext,
) -> Result<ReadBatch<PlotPayload>> {
    if let Some(payload) = super::records::project(telemetry, records, request, generated_at, context)? {
        return Ok(payload);
    }
    if let Some(payload) = super::groups::project(telemetry, records, request, generated_at, context)? {
        return Ok(payload);
    }
    super::projection::plain(telemetry, records, request, generated_at, context)
}

pub(crate) fn finish(payload: ReadBatch<PlotPayload>, request: &PlotPayloadRequest) -> Result<ReadBatch<PlotPayload>> {
    let spectrum = matches!(request.plot_type, Some(PlotType::FrequencySeries));
    let interval = matches!(request.plot_type, Some(PlotType::XyInterval));
    let wifi = matches!(request.plot_type, Some(PlotType::WifiChannelOccupancy));
    // Other Non-FFT Analysis And Summary Intermediates Remain Separate Producer Gaps.
    // They Run Before Native Construction, Not Under A False FFT-Only Peak.
    let payload = payload.try_map(|mut payload| {
        if !spectrum && !interval && !wifi {
            payload.analysis = analyses::analyze(&mut payload.series, request);
        }
        payload.summaries = analyses::summarize(&payload.series, request);
        Ok::<_, crate::runtime::Error>(payload)
    })?;
    let payload = payload.try_map(|mut payload| {
        if spectrum || interval || wifi {
            payload.analysis = analyses::analyze(&mut payload.series, request);
        }
        if spectrum {
            payload.x_key = "frequency_hz".into();
            payload.x_format = None;
        }
        Ok::<_, crate::runtime::Error>(payload)
    })?;
    frame(payload, request)
}

pub(crate) fn frame(payload: ReadBatch<PlotPayload>, request: &PlotPayloadRequest) -> Result<ReadBatch<PlotPayload>> {
    payload.try_map(|payload| frame_owned(payload, request))
}

pub(crate) fn frame_owned(mut payload: PlotPayload, request: &PlotPayloadRequest) -> Result<PlotPayload> {
    let spectrum = matches!(request.plot_type, Some(PlotType::FrequencySeries));
    for series in &mut payload.series {
        if let Some(selected) = indexes(&series.points, payload.max_points, request.technique, spectrum)? {
            let mut selected = selected.into_iter().peekable();
            let mut ordinal = 0usize;
            series.points.retain(|_| {
                let keep = selected.peek() == Some(&ordinal);
                if keep {
                    selected.next();
                }
                ordinal += 1;
                keep
            });
            series.downsampled = true;
        }
        if spectrum {
            series.format = None;
        }
    }
    payload.cartesian = cartesian::cartesian_frame(&payload.series, request.plot_type.as_ref(), &request.x_scale, &request.y_scale)
        .map_err(crate::runtime::Error::Operation)?;
    Ok(payload)
}

/* === Private Functions === */

fn indexes(points: &[PlotPoint], maximum: usize, technique: DownsampleTechnique, spectrum: bool) -> Result<Option<Vec<usize>>> {
    if maximum == 0 || points.len() <= maximum {
        return Ok(None);
    }
    if spectrum {
        let mut values = Vec::new();
        values.try_reserve_exact(points.len()).map_err(|_| memory())?;
        values.extend(points.iter().map(|point| point.y.filter(|value| value.is_finite()).unwrap_or(f64::NEG_INFINITY)));
        let mut selected = harmos::signal::extrema(&values, (maximum / 2).max(1));
        selected.sort_unstable();
        selected.dedup();
        selected.retain(|index| *index < points.len());
        return Ok(Some(selected));
    }
    let technique = match technique {
        DownsampleTechnique::Auto | DownsampleTechnique::Unspecified if points.len() <= maximum.saturating_mul(4) => {
            DownsampleTechnique::Lttb
        }
        DownsampleTechnique::Auto | DownsampleTechnique::Unspecified => DownsampleTechnique::MinMax,
        technique => technique,
    };
    if maximum == 1 || matches!(technique, DownsampleTechnique::Raw) || (matches!(technique, DownsampleTechnique::Lttb) && maximum < 3) {
        return Ok(Some(harmos::signal::decimate(points.len(), maximum)));
    }
    let mut values = Vec::new();
    values.try_reserve_exact(points.len()).map_err(|_| memory())?;
    for point in points {
        let Some(value) = point.y.filter(|value| value.is_finite()) else {
            drop(values);
            return Ok(Some(harmos::signal::decimate(points.len(), maximum)));
        };
        values.push(value);
    }
    let mut selected = if matches!(technique, DownsampleTechnique::Lttb) {
        let mut x = Vec::new();
        x.try_reserve_exact(points.len()).map_err(|_| memory())?;
        for point in points {
            let Some(value) = analyses::point_x(point) else {
                drop(x);
                drop(values);
                return Ok(Some(harmos::signal::decimate(points.len(), maximum)));
            };
            x.push(value);
        }
        harmos::signal::lttb(&x, &values, maximum)
    } else {
        harmos::signal::extrema(&values, (maximum / 2).max(1))
    };
    selected.sort_unstable();
    selected.dedup();
    selected.retain(|index| *index < points.len());
    Ok(Some(selected))
}

fn memory() -> crate::runtime::Error {
    harmos::Error::ResourceExhausted { resource: "plot construction" }.into()
}
