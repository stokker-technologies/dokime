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

use tokio::sync::OnceCell;

// Local Imports
use super::{invalid, telemetry};
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

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(in crate::report) struct ReadCache {
    raw: OnceCell<crate::plot::spilled::Input>,
    reduced: OnceCell<harmos::storage::ReadBatch<PlotPayload>>,
}

impl ReadCache {
    pub(in crate::report) fn new() -> Self {
        Self { raw: OnceCell::new(), reduced: OnceCell::new() }
    }
}

/* === Enums === */

#[derive(PartialEq, Eq)]
pub(in crate::report) enum ReadPlan {
    Raw(Vec<(Identifier, u32)>),
    Reduced(String),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(in crate::report) async fn materialize(
    read: super::ReadContext<'_, '_>, source: &CapturedSource, execution: &TestExecution,
    records: &crate::execution::evidence_rows::EvidenceRows<dokime_domain::model::records::evidence::RecordEntry>,
    request: PlotPayloadRequest, shared: &ReadCache,
) -> Result<harmos::storage::ReadBatch<crate::report::value::Value>> {
    let super::ReadContext { ctx, .. } = read;
    // The Caller Retains The Read-Plan Claim; Its Released Clone And Selection
    // Workspace Is Reused Here Without Charging The Same Peak A Second Time.
    let payload = match direct(&request, &source.telemetry_contracts) {
        Some((request, contract)) => optimized(read, source, execution, records, &request, contract, shared).await?,
        None => {
            let context = payload_context(source, execution, &request, source.report_defaults.plot_points as usize);
            let input = inputs(read, source, execution, records, &request, shared).await?;
            let samples =
                if request.analysis.is_some() { Some(input.collect(dokime_domain::model::analysis::MAX_SAMPLES as usize)?) } else { None };
            let payload = crate::plot::spilled::project(input, &request, source.captured_at.as_str(), &context)?;
            if request.analysis.is_some() {
                let samples = samples.as_ref().expect("analysis samples checked");
                let contract = source
                    .telemetry_contracts
                    .get(&Identifier::new(&request.stream).map_err(|_| invalid("invalid plot stream"))?)
                    .and_then(|versions| versions.values().next());
                crate::report::analysis::materialize(ctx, &request, samples, payload, contract, &source.evidence_through).await?
            } else {
                payload
            }
        }
    };
    let depth = 128;
    // Projection And Cartesian Claims Stay With The Consumed Payload Owner.
    // Other Analysis Workspace Remains A Separate Construction Obligation.
    // This Claim Covers Only The New Portable Value, Before Serialization Builds It.
    let value = crate::report::value::Value::from_serializable(&*payload, depth)?;
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

async fn optimized(
    read: super::ReadContext<'_, '_>, source: &CapturedSource, execution: &TestExecution,
    records: &crate::execution::evidence_rows::EvidenceRows<dokime_domain::model::records::evidence::RecordEntry>,
    request: &PlotPayloadRequest, contract: &TelemetryContract, shared: &ReadCache,
) -> Result<harmos::storage::ReadBatch<PlotPayload>> {
    shared
        .reduced
        .get_or_try_init(|| async {
            let super::ReadContext { ctx, session } = read;
            let codec = telemetry::saved_codec(ctx, session, &source.evidence_through, contract).await?;
            if !crate::plot::stored::supports_codec(request, &codec) {
                let input = inputs(read, source, execution, records, request, shared).await?;
                let context = payload_context(source, execution, request, source.report_defaults.plot_points as usize);
                return crate::plot::spilled::project(input, request, source.captured_at.as_str(), &context);
            }
            let query = telemetry::plan(&source.evidence_through, execution.test_execution_id.as_str(), &codec, Vec::new())?;
            let context = payload_context(source, execution, request, source.report_defaults.plot_points as usize);
            let selection = crate::plot::stored::Selection { query, codec: &codec, producer_order: true };
            crate::plot::stored::materialize(session, selection, request, source.captured_at.as_str(), &context, Default::default()).await
        })
        .await
        .cloned()
}

async fn inputs<'a>(
    read: super::ReadContext<'_, '_>, source: &CapturedSource, execution: &TestExecution,
    records: &crate::execution::evidence_rows::EvidenceRows<dokime_domain::model::records::evidence::RecordEntry>,
    request: &PlotPayloadRequest, shared: &'a ReadCache,
) -> Result<&'a crate::plot::spilled::Input> {
    shared
        .raw
        .get_or_try_init(|| async {
            let streams = crate::plot::plot_streams(request);
            let input = telemetry::streams(
                read.ctx,
                read.session,
                &source.evidence_through,
                execution.test_execution_id.as_str(),
                &source.telemetry_contracts,
                &streams,
            )
            .await?;
            for record in records.iter() {
                input.append_record(&record)?;
            }
            Ok(input)
        })
        .await
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
    crate::plot::spilled::project(&input, request, generated_at, context)
}

// Raw Evidence Stays In Storage; Materialized Views Apply Their Display Bound.
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
    async fn captured_series_reduces_original_spike_with_bounded_output() {
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
        assert!(series.points.len() <= 32);
        assert_eq!(series.raw_points, count);
        assert!(series.downsampled);
        assert!(series.points.iter().any(|point| point.y == Some(1_000_000.0)));
        assert_eq!(payload.max_points, 32);
    }
}
