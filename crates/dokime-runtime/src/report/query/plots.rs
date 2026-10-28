/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::collections::BTreeMap;

use tokio::sync::OnceCell;

use super::invalid;
use crate::document::PlotType;
use crate::execution::context::captured::source::CapturedSource;
use crate::expressions::DataSource;

use crate::plot::PlotPayload;
use crate::plot::PlotPayloadContext;
use crate::plot::PlotPayloadRequest;
use crate::runtime::Result;
use crate::telemetry::TelemetryContract;
use dokime_domain::model::execution::result::TestExecution;
use dokime_domain::model::identity::Identifier;

pub(in crate::report) struct ReadCache {
    reduced: OnceCell<harmos::storage::ReadBatch<PlotPayload>>,
}

impl ReadCache {
    pub(in crate::report) fn new() -> Self {
        Self { reduced: OnceCell::new() }
    }
}

#[derive(PartialEq, Eq)]
pub(in crate::report) enum ReadPlan {
    Raw(Vec<(Identifier, u32)>),
    Reduced(String),
}

pub(in crate::report) async fn materialize(
    ctx: &crate::report::Scope, source: &CapturedSource, execution: &TestExecution, request: PlotPayloadRequest,
    input: &crate::plot::spilled::Input, telemetry: &crate::execution::evidence_rows::EvidenceRows<crate::telemetry::TelemetrySample>,
    shared: &ReadCache,
) -> Result<harmos::storage::ReadBatch<crate::report::value::Value>> {
    let payload = if let Some((canonical, _)) = direct(&request, &source.telemetry_contracts) {
        shared
            .reduced
            .get_or_try_init(|| async {
                let context = payload_context(source, execution, &canonical, source.report_defaults.plot_points as usize);
                saved(input, &canonical, source.captured_at.as_str(), &context)
            })
            .await?
            .clone()
    } else {
        let context = payload_context(source, execution, &request, source.report_defaults.plot_points as usize);
        let samples = if request.analysis.is_some() {
            let streams = crate::plot::plot_streams(&request);
            let mut samples = Vec::new();
            for sample in telemetry
                .iter()
                .filter(|sample| streams.iter().any(|stream| crate::plot::stream_matches_alias(sample.stream.as_str(), stream)))
            {
                if samples.len() == dokime_domain::model::analysis::MAX_SAMPLES as usize {
                    return Err(invalid("telemetry scan exceeds the analysis sample budget"));
                }
                samples.try_reserve(1).map_err(|_| invalid("report analysis allocation failed"))?;
                samples.push(sample);
            }
            samples.sort_by_key(|sample| sample.sequence);
            Some(samples)
        } else {
            None
        };
        let payload = saved(input, &request, source.captured_at.as_str(), &context)?;
        if let Some(samples) = samples {
            let contract = source
                .telemetry_contracts
                .get(&Identifier::new(&request.stream).map_err(|_| invalid("invalid plot stream"))?)
                .and_then(|versions| versions.values().next());
            crate::report::analysis::materialize(ctx, &request, &samples, payload, contract, &source.evidence_through).await?
        } else {
            payload
        }
    };
    let value = crate::report::value::Value::from_serializable(&*payload, 128)?;
    Ok(harmos::storage::ReadBatch::new(value))
}

pub(in crate::report) fn read_plan(
    request: &PlotPayloadRequest, contracts: &BTreeMap<Identifier, BTreeMap<u32, TelemetryContract>>,
) -> Result<ReadPlan> {
    if let Some((request, _)) = direct(request, contracts) {
        let mut key = serde_json::to_value(&request).map_err(|_| invalid("plot selection cannot be encoded"))?;
        key.as_object_mut().expect("plot request is an object").remove("plot_id");
        return Ok(ReadPlan::Reduced(key.to_string()));
    }
    let selected = crate::plot::plot_streams(request);
    let matches = |stream: &Identifier| selected.iter().any(|name| crate::plot::stream_matches_alias(stream.as_str(), name));
    let mut count = 0_usize;
    for (stream, versions) in contracts {
        if !matches(stream) {
            continue;
        }
        count = count.checked_add(versions.len()).ok_or_else(|| invalid("report read plan count overflows"))?;
    }
    let mut streams = Vec::with_capacity(count);
    for (stream, versions) in contracts {
        if matches(stream) {
            streams.extend(versions.values().map(|contract| (contract.stream.clone(), contract.schema_version)));
        }
    }
    Ok(ReadPlan::Raw(streams))
}

fn saved(
    input: &crate::plot::spilled::Input, request: &PlotPayloadRequest, generated_at: &str, context: &PlotPayloadContext,
) -> Result<harmos::storage::ReadBatch<PlotPayload>> {
    let mut complete = request.clone();
    complete.max_points = Some(0);
    crate::plot::spilled::project(input, &complete, generated_at, context)?.try_map(|mut payload| {
        payload.max_points = request.max_points.unwrap_or(context.default_max_points);
        Ok(payload)
    })
}

fn payload_context<'a>(
    source: &'a CapturedSource, execution: &'a TestExecution, request: &'a PlotPayloadRequest, default_max_points: usize,
) -> PlotPayloadContext<'a> {
    PlotPayloadContext {
        default_max_points,
        x_format: request.x_format.as_ref(),
        series_format: request.y_format.as_ref(),
        run_started_at: execution.started_at.as_ref().map(|timestamp| timestamp.as_str()),
        cycle_started_at: source.context.cycle.cycle.started_at.as_ref().map(|timestamp| timestamp.as_str()),
    }
}

fn direct<'a>(
    request: &PlotPayloadRequest, contracts: &'a BTreeMap<Identifier, BTreeMap<u32, TelemetryContract>>,
) -> Option<(PlotPayloadRequest, &'a TelemetryContract)> {
    if request.analysis.is_some()
        || !request.marker_record_types.is_empty()
        || request.series_by.is_some()
        || request.series_by_expression.is_some()
        || request.z_expression.is_some()
        || request.z_stream.is_some()
        || request.z_field.is_some()
        || !request.filters.is_empty()
        || request.window_s.is_some()
        || request.summary_window.is_some()
        || request.scope.as_ref().is_some_and(|scope| scope.step_id.is_some() || scope.criterion_id.is_some())
        || !matches!(request.plot_type, Some(PlotType::TimeSeries | PlotType::XyScatter | PlotType::XyLine | PlotType::Histogram))
    {
        return None;
    }
    let versions = contracts.iter().find(|(stream, _)| stream.as_str() == request.stream)?.1;
    if versions.len() != 1 {
        return None;
    }
    let contract = versions.values().next()?;
    let mut canonical = request.clone();
    let field = |expression: &crate::expressions::DataExpression| match &expression.source {
        DataSource::Telemetry { stream, field } if stream == &request.stream && expression.transforms.is_empty() => Some(field.clone()),
        _ => None,
    };
    if canonical.y_expressions.iter().any(|expression| expression.label.is_some()) {
        return None;
    }
    if !canonical.y_expressions.is_empty() {
        let mut keys = Vec::with_capacity(canonical.y_expressions.len());
        for expression in &canonical.y_expressions {
            keys.push(field(expression)?);
        }
        canonical.y_keys = keys;
        canonical.y_expressions.clear();
    }
    if let Some(expression) = canonical.x_expression.as_ref() {
        canonical.x_key = Some(field(expression)?);
        canonical.x_expression = None;
    }
    if canonical.y_keys.is_empty() {
        return None;
    }
    Some((canonical, contract))
}

#[cfg(test)]
fn full_retained(
    samples: &[crate::telemetry::TelemetrySample], records: crate::records::view::View<'_>, request: &PlotPayloadRequest,
    generated_at: &str, context: &PlotPayloadContext,
) -> Result<harmos::storage::ReadBatch<PlotPayload>> {
    let input = crate::plot::spilled::Input::new()?;
    for sample in samples {
        input.append(sample)?;
    }
    for record in records.iter() {
        input.append_record(record)?;
    }
    saved(&input, request, generated_at, context)
}

// Saved Series Preserve Zoomable Evidence; The Viewer Applies The Display Bound.
#[cfg(test)]
fn full(
    samples: &[crate::telemetry::TelemetrySample], records: crate::records::view::View<'_>, request: &PlotPayloadRequest,
    generated_at: &str, context: &PlotPayloadContext,
) -> harmos::storage::ReadBatch<PlotPayload> {
    full_retained(samples, records, request, generated_at, context).expect("fixture plot construction")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Value;
    use crate::model::identity::new_id;
    use crate::telemetry::TelemetrySample;
    use dokime_domain::model::time::Timestamp;

    #[tokio::test]
    async fn captured_series_preserves_original_points_for_zoom() {
        let count = 100_000 + 101;
        let timestamp = Timestamp::new("2026-10-21T00:00:00Z");
        let stream = Identifier::new("fixture.capture").unwrap();
        let samples = (0..count)
            .map(|index| {
                let mut sample = TelemetrySample::new(new_id(), stream.clone(), index as u64, timestamp.clone(), "fixture");
                let value = if index == 25 { 1_000_000.0 } else { index as f64 };
                sample.values.insert("index".into(), Value::Number(index as f64));
                sample.values.insert("value".into(), Value::Number(value));
                sample
            })
            .collect::<Vec<_>>();
        let request = PlotPayloadRequest {
            stream: stream.to_string(),
            x_key: Some("index".into()),
            y_keys: vec!["value".into()],
            plot_type: Some(PlotType::TimeSeries),
            max_points: Some(32),
            technique: crate::plot::DownsampleTechnique::MinMax,
            ..Default::default()
        };

        let payload = full(
            &samples,
            crate::records::view::View::default(),
            &request,
            timestamp.as_str(),
            &PlotPayloadContext {
                default_max_points: 2000,
                x_format: None,
                series_format: None,
                run_started_at: None,
                cycle_started_at: None,
            },
        );
        let series = &payload.series[0];
        assert_eq!(series.points.len(), count);
        assert_eq!(series.raw_points, count);
        assert!(!series.downsampled);
        assert!(series.points.iter().any(|point| point.y == Some(1_000_000.0)));
        assert_eq!(payload.max_points, 32);
    }
}
