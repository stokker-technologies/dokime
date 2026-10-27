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
use harmos::storage::query::transform::{
    FieldPath, Transform,
    sort::{Sort, SortKey},
};
use tokio::sync::OnceCell;

// Local Imports
use super::{invalid, statistics, telemetry};
use crate::document::PlotType;
use crate::execution::context::captured::source::CapturedSource;
use crate::expressions::DataSource;
use crate::plot::PlotAnalysis;
use crate::plot::PlotPayload;
use crate::plot::PlotPayloadContext;
use crate::plot::PlotPayloadRequest;
use crate::runtime::Result;
use crate::telemetry::TelemetryContract;
use dokime_domain::model::execution::result::TestExecution;
use dokime_domain::model::identity::Identifier;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

#[derive(PartialEq, Eq)]
pub(in crate::report) enum ReadPlan {
    Raw(Vec<(Identifier, u32)>),
    Sorted { stream: Identifier, version: u32, field: Vec<String> },
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(in crate::report) async fn materialize(
    read: super::ReadContext<'_, '_>, source: &CapturedSource, execution: &TestExecution, records: crate::records::view::View<'_>,
    request: PlotPayloadRequest, shared: &OnceCell<harmos::storage::ReadBatch<Vec<crate::telemetry::TelemetrySample>>>,
) -> Result<harmos::storage::ReadBatch<crate::report::value::Value>> {
    let super::ReadContext { ctx, session, .. } = read;
    // The Caller Retains The Read-Plan Claim; Its Released Clone And Selection
    // Workspace Is Reused Here Without Charging The Same Peak A Second Time.
    let payload = match direct(&request, &source.telemetry_contracts) {
        Some((request, contract)) => optimized(read, source, execution, records, &request, contract, shared).await?,
        None => {
            let context = payload_context(source, execution, &request, source.report_defaults.plot_points as usize);
            let streams = crate::plot::plot_streams(&request);
            let samples = shared
                .get_or_try_init(|| {
                    telemetry::streams(
                        ctx,
                        session,
                        &source.evidence_through,
                        execution.test_execution_id.as_str(),
                        &source.telemetry_contracts,
                        &streams,
                    )
                })
                .await?;
            if request.analysis.is_some() {
                let projected =
                    crate::plot::project_retained_plot_payload(samples, records, &request, source.captured_at.as_str(), &context)?;
                let contract = source
                    .telemetry_contracts
                    .get(&Identifier::new(&request.stream).map_err(|_| invalid("invalid plot stream"))?)
                    .and_then(|versions| versions.values().next());
                crate::report::analysis::materialize(ctx, &request, samples, projected, contract, &source.evidence_through).await?
            } else {
                full_retained(samples, records, &request, source.captured_at.as_str(), &context)?
            }
        }
    };
    let depth = 128;
    // Projection And Cartesian Claims Stay With The Consumed Payload Owner.
    // Other Analysis Workspace Remains A Separate Construction Obligation.
    // This Claim Covers Only The New Portable Value, Before Serialization Builds It.
    payload.try_map(|payload| crate::report::value::Value::from_serializable(&payload, depth))
}

pub(in crate::report) fn read_plan(
    request: &PlotPayloadRequest, contracts: &BTreeMap<Identifier, BTreeMap<u32, TelemetryContract>>,
) -> Result<ReadPlan> {
    if let Some((request, contract)) = direct(request, contracts) {
        return Ok(ReadPlan::Sorted {
            stream: contract.stream.clone(),
            version: contract.schema_version,
            field: sort_field(&request).fields,
        });
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

/* === Private Functions === */

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

fn sort_field(request: &PlotPayloadRequest) -> FieldPath {
    match request.x_key.as_deref() {
        None | Some("timestamp") => FieldPath { fields: vec!["_storage".into(), "source_time".into()] },
        Some(key) => statistics::field(key),
    }
}

fn direct<'a>(
    request: &PlotPayloadRequest, contracts: &'a BTreeMap<Identifier, BTreeMap<u32, TelemetryContract>>,
) -> Option<(PlotPayloadRequest, &'a TelemetryContract)> {
    if request.analysis.is_some()
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

async fn optimized(
    read: super::ReadContext<'_, '_>, source: &CapturedSource, execution: &TestExecution, records: crate::records::view::View<'_>,
    request: &PlotPayloadRequest, contract: &TelemetryContract,
    shared: &OnceCell<harmos::storage::ReadBatch<Vec<crate::telemetry::TelemetrySample>>>,
) -> Result<harmos::storage::ReadBatch<PlotPayload>> {
    let super::ReadContext { ctx, session, .. } = read;
    let at = &source.evidence_through;
    let scope = execution.test_execution_id.as_str();
    let context = payload_context(source, execution, request, source.report_defaults.plot_points as usize);
    let x = sort_field(request);
    let samples = shared
        .get_or_try_init(|| {
            let transforms =
                vec![Transform::Sort(Sort { keys: vec![SortKey { field: x.clone(), descending: false, nulls_first: false }] })];
            telemetry::read(ctx, session, at, scope, contract, transforms)
        })
        .await?;
    let statistics = statistics::collect(read, samples, contract, &request.y_keys, &x)?;
    // Missing Values Preserve Existing Gap Semantics Through The Raw Path.
    let missing = statistics.iter().any(|statistics| statistics.count != statistics.rows);
    if missing && request.plot_type != Some(PlotType::Histogram) {
        let samples = telemetry::read(ctx, session, at, scope, contract, vec![]).await?;
        return full_retained(&samples, records, request, source.captured_at.as_str(), &context);
    }

    let payload = crate::plot::project_retained_plot_payload(samples, records, request, source.captured_at.as_str(), &context)?;
    let budget = payload.max_points;
    let payload = payload.try_map(|mut payload| {
        let summary = matches!(request.plot_type, Some(PlotType::TimeSeries | PlotType::XyScatter));
        let histogram = request.plot_type == Some(PlotType::Histogram);
        let mut summaries = Vec::new();
        let mut histograms = Vec::new();
        if summary {
            summaries.try_reserve_exact(request.y_keys.len()).map_err(|_| invalid("report summary allocation failed"))?;
        }
        if histogram {
            histograms.try_reserve_exact(request.y_keys.len()).map_err(|_| invalid("report histogram allocation failed"))?;
        }
        for ((key, statistics), series) in request.y_keys.iter().zip(&statistics).zip(&mut payload.series) {
            series.raw_points = statistics.count;
            if summary {
                summaries.push(statistics.summary(key.clone()));
            }
            if histogram {
                // This Admits The Public Result, Not The Separate Native
                // Statistics/Histogram Transform And Encoder Workspace Gaps.
                histograms.push(statistics::histogram(read, samples, contract, key, request.histogram.bins, statistics)?);
            }
        }
        payload.summaries = summaries;
        payload.analysis = histogram.then_some(PlotAnalysis::Histogram { series: histograms });
        payload.max_points = 0;
        Ok::<_, crate::runtime::Error>(payload)
    })?;
    let payload = crate::plot::finish_retained_plot_frame(payload, request)?;
    payload.try_map(|mut payload| {
        payload.max_points = budget;
        for series in &mut payload.series {
            series.downsampled = series.points.len() < series.raw_points;
        }
        Ok(payload)
    })
}

fn full_retained(
    samples: &[crate::telemetry::TelemetrySample], records: crate::records::view::View<'_>, request: &PlotPayloadRequest,
    generated_at: &str, context: &PlotPayloadContext,
) -> Result<harmos::storage::ReadBatch<PlotPayload>> {
    let payload = crate::plot::project_retained_plot_payload(samples, records, request, generated_at, context)?;
    let budget = payload.max_points;
    let payload = payload.try_map(|mut payload| {
        for series in &mut payload.series {
            series.raw_points = series.raw_points.max(series.points.len());
        }
        payload.max_points = 0;
        Ok::<_, crate::runtime::Error>(payload)
    })?;
    let payload = crate::plot::finish_retained_plot_payload(payload, request)?;
    payload.try_map(|mut payload| {
        payload.max_points = budget;
        for series in &mut payload.series {
            series.downsampled = series.points.len() < series.raw_points;
        }
        Ok(payload)
    })
}

// Captured Series Stay Whole. Only The Viewer Applies A Display Budget To Its
// Visible Window; Dropping An Earlier Point Here Would Make Zoom Irrecoverable.
#[cfg(test)]
fn full(
    samples: &[crate::telemetry::TelemetrySample], records: crate::records::view::View<'_>, request: &PlotPayloadRequest,
    generated_at: &str, context: &PlotPayloadContext,
) -> harmos::storage::ReadBatch<PlotPayload> {
    full_retained(samples, records, request, generated_at, context).expect("fixture plot construction")
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Value;
    use crate::model::identity::new_id;
    use crate::telemetry::TelemetrySample;
    use dokime_domain::model::time::Timestamp;

    #[tokio::test]
    async fn frozen_series_retains_points_beyond_the_live_plot_cap() {
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
        assert_eq!(series.points.first().unwrap().y, Some(0.0));
        assert_eq!(series.points.last().unwrap().y, Some((count - 1) as f64));
        assert_eq!(payload.max_points, 32);

        // An Early Viewport Can Still Recover Its Original Spike And Endpoints.
        let payload = payload
            .try_map(|mut payload| {
                payload.series[0].points.retain(|point| point.x.as_f64().is_some_and(|x| x <= 100.0));
                Ok::<_, crate::runtime::Error>(payload)
            })
            .unwrap();
        assert_eq!(payload.series[0].points.len(), 101);
        let viewport = crate::plot::finish_retained_plot_payload(payload, &request).unwrap();
        let points = &viewport.series[0].points;
        assert!(points.len() <= 32);
        assert!(points.iter().any(|point| point.y == Some(1_000_000.0)));
        assert_eq!(points.first().unwrap().y, Some(0.0));
        assert_eq!(points.last().unwrap().y, Some(100.0));
    }
}
