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
use harmos::{
    operations::Context,
    storage::query::transform::{
        FieldPath, Transform,
        sort::{Sort, SortKey},
    },
};

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
use dokime_domain::model::records::evidence::RecordEntry;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(in crate::report) async fn materialize(
    ctx: &Context<'_>, settings: &crate::config::StorageSettings, source: &CapturedSource, execution: &TestExecution,
    records: &[RecordEntry], request: PlotPayloadRequest,
) -> Result<crate::report::value::Value> {
    let context = PlotPayloadContext {
        x_format: request.x_format.clone(),
        series_format: request.y_format.clone(),
        run_started_at: execution.started_at.as_ref().map(ToString::to_string),
        cycle_started_at: source.context.cycle.cycle.started_at.as_ref().map(ToString::to_string),
    };
    let payload = match direct(&request, &source.telemetry_contracts) {
        Some((request, contract)) => {
            optimized(super::ReadContext { ctx, settings }, source, execution, records, &request, contract, &context).await?
        }
        None => {
            let streams = crate::plot::plot_streams(&request);
            let samples = telemetry::streams(
                ctx,
                settings,
                &source.evidence_through,
                execution.test_execution_id.as_str(),
                &source.telemetry_contracts,
                &streams,
            )
            .await?;
            if request.analysis.is_some() {
                let projected = crate::plot::project_plot_payload(&samples, records, &request, source.captured_at.as_str(), &context);
                let contract = source
                    .telemetry_contracts
                    .get(&Identifier::new(&request.stream).map_err(|_| invalid("invalid plot stream"))?)
                    .and_then(|versions| versions.values().next());
                crate::report::analysis::materialize(ctx, &request, &samples, projected, contract).await?
            } else {
                full(&samples, records, &request, source.captured_at.as_str(), &context)
            }
        }
    };
    crate::report::value::Value::from_serializable(&payload)
}

/* === Private Functions === */

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
        canonical.y_keys = canonical.y_expressions.iter().map(field).collect::<Option<Vec<_>>>()?;
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
    read: super::ReadContext<'_, '_>, source: &CapturedSource, execution: &TestExecution, records: &[RecordEntry],
    request: &PlotPayloadRequest, contract: &TelemetryContract, context: &PlotPayloadContext,
) -> Result<PlotPayload> {
    let super::ReadContext { ctx, settings } = read;
    let at = &source.evidence_through;
    let scope = execution.test_execution_id.as_str();
    let time_axis = request.x_key.as_deref().is_none_or(|key| key == "timestamp");
    let x = if time_axis {
        FieldPath { fields: vec!["_storage".into(), "source_time".into()] }
    } else {
        statistics::field(request.x_key.as_deref().unwrap())
    };
    let mut payload = crate::plot::build_plot_payload(&[], records, request.clone(), source.captured_at.as_str(), context);
    payload.series.clear();
    payload.summaries.clear();
    let mut summaries = Vec::new();
    let mut histograms = Vec::new();
    for key in &request.y_keys {
        let statistics = statistics::collect(read, at, scope, contract, key, x.clone()).await?;
        // Missing Values Preserve Existing Gap Semantics Through The Raw Path.
        if statistics.count != statistics.rows && request.plot_type != Some(PlotType::Histogram) {
            let samples = telemetry::read(ctx, settings, at, scope, contract, vec![]).await?;
            return Ok(full(&samples, records, request, source.captured_at.as_str(), context));
        }
        let transforms = vec![Transform::Sort(Sort { keys: vec![SortKey { field: x.clone(), descending: false, nulls_first: false }] })];
        let samples = telemetry::read(ctx, settings, at, scope, contract, transforms).await?;
        let mut single = request.clone();
        single.y_keys = vec![key.clone()];
        single.max_points = Some(0);
        let one = full(&samples, records, &single, source.captured_at.as_str(), context);
        for mut series in one.series {
            series.raw_points = statistics.count;
            series.downsampled = series.points.len() < statistics.count;
            payload.series.push(series);
        }
        if matches!(request.plot_type, Some(PlotType::TimeSeries | PlotType::XyScatter)) {
            summaries.push(statistics.summary(key.clone()));
        }
        if request.plot_type == Some(PlotType::Histogram) {
            histograms.push(statistics::histogram(read, at, scope, contract, key, request.histogram.bins, &statistics).await?);
        }
    }
    let budget = std::mem::take(&mut payload.max_points);
    payload = crate::plot::finish_plot_payload(payload, request);
    payload.max_points = budget;
    payload.summaries = summaries;
    for series in &mut payload.series {
        series.downsampled = series.points.len() < series.raw_points;
    }
    if request.plot_type == Some(PlotType::Histogram) {
        payload.analysis = Some(PlotAnalysis::Histogram { series: histograms });
    }
    Ok(payload)
}

// Captured Series Stay Whole. Only The Viewer Applies A Display Budget To Its
// Visible Window; Dropping An Earlier Point Here Would Make Zoom Irrecoverable.
fn full(
    samples: &[crate::telemetry::TelemetrySample], records: &[RecordEntry], request: &PlotPayloadRequest, generated_at: &str,
    context: &PlotPayloadContext,
) -> PlotPayload {
    let mut payload = crate::plot::project_plot_payload(samples, records, request, generated_at, context);
    for series in &mut payload.series {
        series.raw_points = series.raw_points.max(series.points.len());
    }
    // The Display Budget Stays Stated; The Viewer Reduces To It Per Visible Window.
    let budget = std::mem::take(&mut payload.max_points);
    let mut payload = crate::plot::finish_plot_payload(payload, request);
    payload.max_points = budget;
    for series in &mut payload.series {
        series.downsampled = series.points.len() < series.raw_points;
    }
    payload
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Value;
    use crate::model::identity::new_id;
    use crate::telemetry::TelemetrySample;
    use dokime_domain::model::time::Timestamp;

    #[test]
    fn frozen_series_retains_points_beyond_the_live_plot_cap() {
        let count = crate::config::MAX_PLOT_POINTS as usize + 101;
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

        let mut payload = full(&samples, &[], &request, timestamp.as_str(), &PlotPayloadContext::default());
        let series = &payload.series[0];
        assert_eq!(series.points.len(), count);
        assert_eq!(series.raw_points, count);
        assert!(!series.downsampled);
        assert_eq!(series.points.first().unwrap().y, Some(0.0));
        assert_eq!(series.points.last().unwrap().y, Some((count - 1) as f64));
        assert_eq!(payload.max_points, 32);

        // An Early Viewport Can Still Recover Its Original Spike And Endpoints.
        payload.series[0].points.retain(|point| point.x.as_f64().is_some_and(|x| x <= 100.0));
        assert_eq!(payload.series[0].points.len(), 101);
        let viewport = crate::plot::finish_plot_payload(payload, &request);
        let points = &viewport.series[0].points;
        assert!(points.len() <= 32);
        assert!(points.iter().any(|point| point.y == Some(1_000_000.0)));
        assert_eq!(points.first().unwrap().y, Some(0.0));
        assert_eq!(points.last().unwrap().y, Some(100.0));
    }
}
