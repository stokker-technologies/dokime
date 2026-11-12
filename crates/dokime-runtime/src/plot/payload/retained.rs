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

// Third Party Imports
use harmos::storage::ReadBatch;

// Local Imports
use super::{DownsampleTechnique, PlotPayload, PlotPayloadRequest, PlotPoint, analyses, cartesian};
use crate::document::PlotType;
use crate::runtime::Result;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) fn project(
    telemetry: &[crate::telemetry::TelemetrySample], records: crate::records::view::View<'_>, request: &PlotPayloadRequest,
    generated_at: &str, context: &super::PlotPayloadContext,
) -> Result<ReadBatch<PlotPayload>> {
    if request.plot_type == Some(PlotType::GridChart) {
        let selected = crate::plot::record_samples::PlotSamples::new(telemetry, request).collect();
        let mut payload = super::payload_from_series(&selected, records, request, generated_at, context, Vec::new());
        let result = (|| {
            let mut frames = super::grid::Frames::new(request, payload.max_points)?;
            for sample in selected {
                frames.push(sample)?;
            }
            Ok::<_, String>(frames.finish())
        })();
        payload.analysis = Some(match result {
            Ok(data) => super::PlotAnalysis::GridChart { data },
            Err(message) => super::PlotAnalysis::Error { message },
        });
        return Ok(ReadBatch::new(payload));
    }
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
    let interval = request.plot_type == Some(PlotType::Xy) && request.xy_interval.is_some();
    // Other Non-FFT Analysis And Summary Intermediates Remain Separate Producer Gaps.
    // They Run Before Native Construction, Not Under A False FFT-Only Peak.
    let payload = payload.try_map(|mut payload| {
        if !spectrum && !interval && request.plot_type != Some(PlotType::GridChart) {
            payload.analysis = analyses::analyze(&mut payload.series, request);
        }
        payload.summaries = if request.defer_statistics { Vec::new() } else { analyses::summarize(&payload.series, request) };
        Ok::<_, crate::runtime::Error>(payload)
    })?;
    let payload = payload.try_map(|mut payload| {
        if spectrum || interval {
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
    if let Err(message) = super::totals::validate(&payload.series, request) {
        payload.analysis = Some(super::PlotAnalysis::Error { message });
        payload.series.clear();
        payload.cartesian = None;
        return Ok(payload);
    }
    if request.plot_type == Some(PlotType::Pie) || request.pareto {
        let count = payload.series.iter().map(|series| series.points.len()).sum::<usize>();
        if payload.max_points > 0 && count > payload.max_points {
            payload.analysis = Some(super::PlotAnalysis::Error {
                message: "Category totals exceed the display budget; aggregate samples explicitly".into(),
            });
            payload.series.clear();
        }
        payload.cartesian = None;
        return Ok(payload);
    }

    if payload.max_points > 0 && payload.markers.len() > payload.max_points {
        let mut selected = super::marker_indices(payload.markers.len(), payload.max_points).into_iter().peekable();
        let mut index = 0;
        payload.markers.retain(|_| {
            let keep = selected.peek() == Some(&index);
            if keep {
                selected.next();
            }
            index += 1;
            keep
        });
    }
    if request.plot_type == Some(PlotType::Contour) {
        if let Some(super::PlotAnalysis::Contour { grid }) = &mut payload.analysis {
            super::contour::fit(grid, &mut payload.contour_counts, request.reduction.unwrap_or_default(), payload.max_points);
        } else if !matches!(payload.analysis, Some(super::PlotAnalysis::Error { .. })) {
            payload.analysis = Some(match super::contour::bin(&payload.series, request, payload.max_points) {
                Ok((grid, counts)) => {
                    payload.contour_counts = counts;
                    super::PlotAnalysis::Contour { grid }
                }
                Err(message) => super::PlotAnalysis::Error { message },
            });
        }
        payload.series.clear();
        payload.cartesian = None;
        return Ok(payload);
    }
    if let Some(super::PlotAnalysis::GridChart { data }) = &mut payload.analysis {
        let cells = data.rows.len().saturating_mul(data.columns.len()).max(1);
        let maximum = if payload.max_points == 0 { data.frames.len() } else { (payload.max_points / cells).max(1) };
        if data.frames.len() > maximum {
            let mut selected = super::marker_indices(data.frames.len(), maximum).into_iter().peekable();
            let mut index = 0;
            data.frames.retain(|_| {
                let keep = selected.peek() == Some(&index);
                if keep {
                    selected.next();
                }
                index += 1;
                keep
            });
        }
        return Ok(payload);
    }
    let spectrum = matches!(request.plot_type, Some(PlotType::FrequencySeries));
    let timed = request.panel_width.filter(|_| {
        payload.max_points > 0
            && matches!(request.plot_type, None | Some(PlotType::TimeSeries))
            && request.x_key.as_deref().is_none_or(|key| key == "timestamp")
            && request.x_expression.is_none()
    });
    let columns = timed.map(|width| (width as usize).min((payload.max_points / 6).max(1)));
    let extent = request.viewport.or_else(|| {
        let mut xs = payload.series.iter().flat_map(|series| series.points.iter().filter_map(analyses::point_x));
        let first = xs.next()?;
        let window = xs.fold(super::PlotWindow { from: first, to: first }, |window, x| super::PlotWindow {
            from: window.from.min(x),
            to: window.to.max(x),
        });
        Some(super::PlotWindow {
            from: request.window_s.map_or(window.from, |seconds| window.to - seconds),
            to: window.to,
        })
    });
    if let (Some(columns), Some(window)) = (columns, extent) {
        payload.bucket_window = Some(window);
        let width = ((window.to - window.from) / columns.max(1) as f64).max(f64::EPSILON);
        payload.bucket_width_s = Some(width);
        payload.bucket_counts.clear();
        for point in payload.series.iter().flat_map(|series| &series.points) {
            if let Some(x) = analyses::point_x(point).filter(|x| window.from <= *x && *x <= window.to) {
                *payload.bucket_counts.entry((x / width).floor() as i128).or_insert(0) += 1;
            }
        }
    }
    let maximum = request.panel_width.map_or(payload.max_points, |width| {
        if payload.max_points == 0 { 0 } else { payload.max_points.min((width as usize).saturating_mul(4)) }
    });
    for series in &mut payload.series {
        if let Some(window) = request.viewport {
            series.points.retain(|point| analyses::point_x(point).is_some_and(|x| window.from <= x && x <= window.to));
        }
        let selected = if let Some(columns) = columns {
            Some(time_indices(&series.points, extent, columns))
        } else {
            indexes(&series.points, maximum, request.technique, spectrum)?
        };
        if let Some(selected) = selected {
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

// First, Last And Extrema Are Selected Within Equal Time Intervals.
pub(crate) fn time_indices(points: &[PlotPoint], viewport: Option<super::PlotWindow>, columns: usize) -> Vec<usize> {
    let extent = viewport.or_else(|| {
        let mut values = points.iter().filter_map(analyses::point_x);
        let first = values.next()?;
        Some(values.fold(super::PlotWindow { from: first, to: first }, |window, x| super::PlotWindow {
            from: window.from.min(x),
            to: window.to.max(x),
        }))
    });
    let Some(extent) = extent else { return Vec::new() };
    let width = ((extent.to - extent.from) / columns.max(1) as f64).max(f64::EPSILON);
    let mut buckets = std::collections::BTreeMap::<i64, Vec<usize>>::new();
    for (index, point) in points.iter().enumerate() {
        let Some(x) = analyses::point_x(point).filter(|x| *x >= extent.from && *x <= extent.to) else { continue };
        let key = (x / width).floor() as i64;
        buckets.entry(key).or_default().push(index);
    }
    let mut selected = Vec::new();
    for bucket in buckets.values() {
        selected.push(bucket[0]);
        selected.push(*bucket.last().expect("nonempty time bucket"));
        let finite = bucket.iter().copied().filter(|index| points[*index].y.is_some_and(f64::is_finite)).collect::<Vec<_>>();
        let values = finite.iter().map(|index| points[*index].y.expect("finite value")).collect::<Vec<_>>();
        selected.extend(crate::signal::m4(&values).into_iter().map(|index| finite[index]));
        let mut gaps = bucket.iter().copied().filter(|index| points[*index].y.is_none());
        if let Some(index) = gaps.clone().next() {
            selected.push(index);
        }
        if let Some(index) = gaps.next_back() {
            selected.push(index);
        }
    }
    selected.sort_unstable();
    selected.dedup();
    selected
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
        let mut selected = crate::signal::extrema(&values, (maximum / 2).max(1));
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
        return Ok(Some(crate::signal::decimate(points.len(), maximum)));
    }
    let mut values = Vec::new();
    values.try_reserve_exact(points.len()).map_err(|_| memory())?;
    for point in points {
        let Some(value) = point.y.filter(|value| value.is_finite()) else {
            drop(values);
            return Ok(Some(crate::signal::decimate(points.len(), maximum)));
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
                return Ok(Some(crate::signal::decimate(points.len(), maximum)));
            };
            x.push(value);
        }
        crate::signal::lttb(&x, &values, maximum)
    } else {
        crate::signal::extrema(&values, (maximum / 2).max(1))
    };
    selected.sort_unstable();
    selected.dedup();
    selected.retain(|index| *index < points.len());
    Ok(Some(selected))
}

fn memory() -> crate::runtime::Error {
    harmos::Error::ResourceExhausted { resource: "plot construction" }.into()
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn time_buckets_preserve_irregular_spikes_and_gaps() {
        let points = [
            (0.0, Some(1.0)),
            (0.01, Some(-50.0)),
            (0.02, Some(100.0)),
            (0.03, None),
            (0.04, Some(2.0)),
            (9.0, Some(4.0)),
            (9.5, Some(5.0)),
        ]
        .into_iter()
        .map(|(x, y)| PlotPoint {
            instant: None,
            x: serde_json::json!(x),
            y,
            z: None,
        })
        .collect::<Vec<_>>();
        let indices = time_indices(&points, Some(super::super::PlotWindow { from: 0.0, to: 10.0 }), 2);
        assert_eq!(indices, vec![0, 1, 2, 3, 4, 5, 6]);
        let indices = time_indices(&points, Some(super::super::PlotWindow { from: 9.0, to: 9.5 }), 2);
        assert_eq!(indices, vec![5, 6]);
        assert!(time_indices(&points, Some(super::super::PlotWindow { from: 20.0, to: 30.0 }), 2).is_empty());
    }

    #[test]
    fn marker_reduction() {
        let request = PlotPayloadRequest::default();
        let context = super::super::PlotPayloadContext {
            default_max_points: 31,
            x_format: None,
            series_format: None,
            run_started_at: None,
            cycle_started_at: None,
        };
        let mut payload = project(&[], Default::default(), &request, "", &context).unwrap().as_ref().clone();
        payload.markers = (0..31)
            .map(|index| super::super::PlotMarker {
                x: serde_json::json!(index),
                label: index.to_string(),
                record_type: "event".into(),
                color: "blue".into(),
            })
            .collect();
        for maximum in [1, 2, 7] {
            let mut candidate = payload.clone();
            candidate.max_points = maximum;
            let reduced = frame_owned(candidate, &request).unwrap();
            assert_eq!(reduced.markers.len(), maximum);
            if maximum > 1 {
                assert_eq!(reduced.markers.first().unwrap().x, serde_json::json!(0));
                assert_eq!(reduced.markers.last().unwrap().x, serde_json::json!(30));
            }
        }
    }
}
