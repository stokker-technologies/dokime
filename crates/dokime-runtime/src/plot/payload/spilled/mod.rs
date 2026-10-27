/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Exact Spilled Plot Projection

/* === Modules === */

mod analysis;
mod correlation;
mod input;
mod projection;
mod series;
mod spectrum;
mod wifi;

/* === Re-Exports === */

pub(crate) use input::Input;

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::{PlotAnalysis, PlotPayload, PlotPayloadContext, PlotPayloadRequest, PlotWindow, analyses, cartesian};
use crate::{document::PlotType, runtime::Result};
use harmos::storage::ReadBatch;
use input::Read;
use series::Series;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) fn project(
    input: &Input, request: &PlotPayloadRequest, generated_at: &str, context: &PlotPayloadContext,
) -> Result<ReadBatch<PlotPayload>> {
    input.read(|read| {
        let mut payload = super::payload_from_series(&[], &[], Default::default(), request, generated_at, context, Vec::new());
        if request.analysis.is_none() {
            payload.analysis = analyses::analyze(&mut [], request);
        }
        read.records(false, |_, record| {
            let rows = std::slice::from_ref(&record);
            payload.markers.extend(super::plot_markers(rows.into(), &request.marker_record_types, request.scope.as_ref()));
            Ok(())
        })?;
        if request.plot_type == Some(PlotType::WifiChannelOccupancy) {
            wifi::reset(&read)?;
            payload.context = wifi_context(&read, request)?;
        }
        let mut visit = |series| append(&read, &mut payload, series, request, context);
        if !correlation::project(&read, request, &mut visit)? {
            let projection = projection::Projection::telemetry(&read, request)?;
            projection.series(&read, &mut visit)?;
        }
        if request.plot_type == Some(PlotType::WifiChannelOccupancy) && !matches!(payload.analysis, Some(PlotAnalysis::Error { .. })) {
            payload.analysis = Some(wifi::finish(&read)?);
        }
        let window = payload
            .summaries
            .iter()
            .filter_map(|summary| summary.window)
            .reduce(|left, right| PlotWindow { from: left.from.min(right.from), to: left.to.max(right.to) });
        for summary in &mut payload.summaries {
            summary.window = request.summary_window.or(window);
        }
        if request.plot_type == Some(PlotType::FrequencySeries) {
            payload.x_key = "frequency_hz".into();
            payload.x_format = None;
        }
        if request.analysis.is_none() {
            payload.cartesian = cartesian::cartesian_frame(&payload.series, request.plot_type.as_ref(), &request.x_scale, &request.y_scale)
                .map_err(crate::runtime::Error::Operation)?;
        }
        Ok(ReadBatch::new(payload))
    })
}

/* === Private Functions === */

fn append(
    read: &Read<'_>, payload: &mut PlotPayload, mut series: Series, request: &PlotPayloadRequest, context: &PlotPayloadContext,
) -> Result<()> {
    series.metadata.format = context.series_format.cloned();
    if analyses::summarizes(request) {
        payload.summaries.push(series.summary(request.summary_window)?);
    }
    let invalid_z = request.z_expression.as_ref().is_some_and(|expression| !expression.transforms.is_empty());
    if invalid_z {
        payload.analysis = Some(PlotAnalysis::Error { message: "Z expressions must select a scalar field without transforms".into() });
    }
    if request.analysis.is_some() {
        payload.series.push(series.reduce(0, request.technique)?);
        return Ok(());
    }
    if !invalid_z && !matches!(payload.analysis, Some(PlotAnalysis::Error { .. })) {
        analyze(read, payload, &mut series, request)?;
    }
    let technique =
        if request.plot_type == Some(PlotType::FrequencySeries) { super::DownsampleTechnique::MinMax } else { request.technique };
    let maximum = if request.plot_type == Some(PlotType::FrequencySeries) && payload.max_points == 1 {
        series.metadata.downsampled = series.table.len() > 1;
        2
    } else {
        payload.max_points
    };
    payload.series.push(series.reduce(maximum, technique)?);
    Ok(())
}

fn analyze(read: &Read<'_>, payload: &mut PlotPayload, series: &mut Series, request: &PlotPayloadRequest) -> Result<()> {
    match request.plot_type {
        Some(PlotType::Histogram) => {
            if request.y_keys.is_empty() && request.y_expressions.is_empty() {
                payload.analysis =
                    Some(PlotAnalysis::Error { message: "histogram plots bin one x field or x expression; none was declared".into() });
                return Ok(());
            }
            if let Some(unit) = &request.histogram.unit {
                series.metadata.unit = Some(unit.clone());
            }
            let result = analysis::histogram(series, request.histogram.bins)?;
            let analysis = payload.analysis.get_or_insert_with(|| PlotAnalysis::Histogram { series: Vec::new() });
            if let PlotAnalysis::Histogram { series } = analysis {
                series.push(result);
            }
        }
        Some(PlotType::BoxSummary) => {
            let boxes = analysis::boxes(read, series, request)?;
            let analysis = payload.analysis.get_or_insert_with(|| PlotAnalysis::BoxSummary { series: Vec::new(), total: 0 });
            if let PlotAnalysis::BoxSummary { series, total } = analysis {
                for summary in &boxes {
                    *total += summary.count;
                }
                series.extend(boxes);
            }
        }
        Some(PlotType::XyInterval) => {
            let interval = analysis::interval(read, series, request)?;
            let analysis = payload.analysis.get_or_insert_with(|| PlotAnalysis::SigmaInterval { series: Vec::new() });
            if let PlotAnalysis::SigmaInterval { series } = analysis {
                series.push(interval);
            }
        }
        Some(PlotType::FrequencySeries) => {
            if let Some(message) = spectrum::transform(series, request)? {
                payload.analysis = Some(PlotAnalysis::Error { message });
            }
            series.metadata.format = None;
        }
        Some(PlotType::WifiChannelOccupancy) => wifi::append(read, series)?,
        _ => {}
    }
    Ok(())
}

fn wifi_context(read: &Read<'_>, request: &PlotPayloadRequest) -> Result<std::collections::BTreeMap<String, crate::Value>> {
    let mut occupancy = None;
    let mut wifi = None;
    read.samples(|sample| {
        if sample.stream.as_str() == super::WIFI_CONTEXT_STREAM {
            wifi = Some(sample.clone());
        }
        if projection::selected(&sample, request) {
            let newer = occupancy.as_ref().is_none_or(|previous: &crate::telemetry::TelemetrySample| {
                (sample.timestamp.as_str(), sample.sequence) >= (previous.timestamp.as_str(), previous.sequence)
            });
            if newer {
                occupancy = Some(sample);
            }
        }
        Ok(())
    })?;
    let samples: Vec<_> = occupancy.iter().collect();
    let telemetry: Vec<_> = wifi.into_iter().collect();
    Ok(super::plot_context(request, &samples, &telemetry))
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::identity::new_id;
    use crate::{Value, telemetry::TelemetrySample};
    use dokime_domain::model::{identity::Identifier, time::Timestamp};

    #[test]
    fn spilled_records_preserve_capture_order_and_scope_correlation() {
        use dokime_domain::model::{
            execution::RecordScope,
            expressions::{record, telemetry},
            identity::TestExecutionId,
            records::{RecordPublishRequest, RecordSeverity, evidence::RecordEntry},
        };
        let input = Input::new().unwrap();
        let mut samples = Vec::new();
        let mut records = Vec::new();
        for index in 0..12_u64 {
            let scope = RecordScope { test_execution_id: Some(TestExecutionId::from_seed((index % 2).to_string())), ..Default::default() };
            let timestamp = Timestamp::new(format!("2026-10-01T00:00:{index:02}Z"));
            let mut sample =
                TelemetrySample::new(new_id(), Identifier::new("fixture.signal").unwrap(), index, timestamp.clone(), "fixture");
            sample.scope = Some(scope.clone());
            sample.values.insert("y".into(), Value::Number(index as f64));
            sample.values.insert("group".into(), Value::Text((index % 2).to_string()));
            input.append(&sample).unwrap();
            samples.push(sample);
            let kind = if index % 4 < 2 { "fixture.start" } else { "fixture.end" };
            let publication = RecordPublishRequest::new(Identifier::new(kind).unwrap(), RecordSeverity::Info);
            let mut record = publication.into_record(new_id(), 1, timestamp, "fixture");
            record.scope = Some(scope);
            records.push(RecordEntry::from_record(index / 4, record));
        }
        records.reverse();
        for record in &records {
            input.append_record(record).unwrap();
        }
        let context = PlotPayloadContext {
            default_max_points: 31,
            x_format: None,
            series_format: None,
            run_started_at: None,
            cycle_started_at: None,
        };
        let timing = record("fixture.end").time_between("fixture.start");
        let at_end = record("fixture.end").telemetry_at_or_before("fixture.signal", "y");
        for reverse in [false, true] {
            if reverse {
                input.reverse_records().unwrap();
            }
            for expression in [timing.clone(), at_end.clone()] {
                for aggregate in [false, true] {
                    let expression = if aggregate { expression.clone().mean() } else { expression.clone() };
                    let request = PlotPayloadRequest {
                        stream: "fixture.signal".into(),
                        y_expressions: vec![expression],
                        series_by_expression: Some(telemetry("fixture.signal", "group")),
                        plot_type: Some(PlotType::TimeSeries),
                        marker_record_types: vec!["fixture.start".into()],
                        ..Default::default()
                    };
                    let eager = super::super::retained::project(&samples, records.as_slice().into(), &request, "now", &context).unwrap();
                    let eager = super::super::retained::finish(eager, &request).unwrap();
                    let spilled = project(&input, &request, "now", &context).unwrap();
                    assert_eq!(serde_json::to_value(&*spilled).unwrap(), serde_json::to_value(&*eager).unwrap());
                }
            }
        }
    }

    #[test]
    fn spilled_projection_matches_eager_analyses_and_reductions() {
        let mut samples = Vec::new();
        let input = Input::new().unwrap();
        for index in 0_i32..2057 {
            let mut sample = TelemetrySample::new(
                new_id(),
                Identifier::new("fixture.signal").unwrap(),
                index as u64,
                Timestamp::new("2026-10-01T00:00:00Z"),
                "fixture",
            );
            sample.values.insert("x".into(), Value::Number(f64::from(index)));
            sample.values.insert("y".into(), Value::Number((f64::from(index) * 0.03).sin()));
            sample.values.insert("group".into(), Value::Text((index % 3).to_string()));
            input.append(&sample).unwrap();
            samples.push(sample);
        }
        let context = PlotPayloadContext {
            default_max_points: 31,
            x_format: None,
            series_format: None,
            run_started_at: None,
            cycle_started_at: None,
        };
        for plot_type in [
            PlotType::TimeSeries,
            PlotType::XyScatter,
            PlotType::Trajectory,
            PlotType::Histogram,
            PlotType::BoxSummary,
            PlotType::XyInterval,
            PlotType::FrequencySeries,
        ] {
            for grouped in [false, true] {
                for technique in [
                    super::super::DownsampleTechnique::Raw,
                    super::super::DownsampleTechnique::Lttb,
                    super::super::DownsampleTechnique::MinMax,
                ] {
                    let request = PlotPayloadRequest {
                        stream: "fixture.signal".into(),
                        x_key: Some("x".into()),
                        y_keys: vec!["y".into()],
                        series_by: grouped.then(|| "group".into()),
                        plot_type: Some(plot_type.clone()),
                        technique,
                        max_points: Some(31),
                        ..Default::default()
                    };
                    let eager = super::super::retained::project(&samples, Default::default(), &request, "now", &context).unwrap();
                    let eager = super::super::retained::finish(eager, &request).unwrap();
                    let spilled = project(&input, &request, "now", &context).unwrap();
                    let actual = serde_json::to_value(&*spilled).unwrap();
                    let expected = serde_json::to_value(&*eager).unwrap();
                    assert_eq!(actual, expected, "{plot_type:?}, grouped={grouped}, {technique:?}");
                }
            }
        }
    }
}
