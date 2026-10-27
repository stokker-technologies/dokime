/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

pub(crate) mod analysis;
mod context;
mod request;
mod stream;

/* === Re-Exports === */

pub use crate::plot::{
    BoxSummary, DownsampleTechnique, HistogramBin, HistogramSeries, HistogramStatistics, PlotAnalysis, PlotCartesianFrame,
    PlotCartesianMode, PlotCartesianSeriesFrame, PlotComputation, PlotComputedSeries, PlotDomain, PlotEvidence, PlotMarker, PlotPayload,
    PlotPayloadDiagnostics, PlotPayloadEnvelope, PlotPayloadRequest, PlotPayloadStreamKind, PlotPoint, PlotSeries, PlotSeriesSummary,
    PlotWindow, SigmaBin, SigmaSeries, WifiBandScan, WifiBandWindow, WifiChannelCount, WifiScan, declared_plot_request,
    plot_payload_request_for_descriptor,
};
pub(crate) use analysis::definitions as analysis_definitions;
pub(crate) use request::request_telemetry_streams;
pub use stream::{RuntimePlotQueryMode, RuntimePlotStreamBatch, matching_plot_payload_request, plot_payload_stream_kind};

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use futures_util::future::join_all;

// Local Imports
use crate::plot::{
    PlotPayloadContext, finish_retained_plot_payload, plot_payload_diagnostics, project_retained_plot_payload, resolved_format,
};
use crate::telemetry::{TelemetryColumn, TelemetryContract};
use crate::{evidence_runtime::TelemetrySelection, handle::RuntimeHandle};
use context::RuntimePlotEnvelopeContext;
use dokime_domain::model::records::evidence::RecordEntry;
use dokime_domain::model::time::Timestamp;
use request::{canonical_plot_request, request_record_types, scope_selection};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/// What a live plot watch re-reads a plot for: rows on the telemetry streams
/// it reads, and state changes when it reads records or no telemetry at all.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PlotWatchKeys {
    telemetry_streams: Vec<String>,
    records: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl PlotWatchKeys {
    pub fn for_request(request: &PlotPayloadRequest) -> Self {
        Self {
            telemetry_streams: request_telemetry_streams(request).into_iter().map(|stream| stream.as_str().to_owned()).collect(),
            records: !request_record_types(request).is_empty(),
        }
    }

    /// Whether a row committed on `stream` can change this plot.
    pub fn reads_stream(&self, stream: &str) -> bool {
        self.telemetry_streams
            .iter()
            .any(|declared| crate::plot::stream_matches_alias(stream, declared) || crate::plot::stream_matches_alias(declared, stream))
    }

    /// Whether a runtime state change can change this plot.
    pub fn follows_state(&self) -> bool {
        self.records || self.telemetry_streams.is_empty()
    }
}

/* === Public Functions === */

/// Reduces Display Points Without Repeating Evidence Reads Or Analysis.
pub fn reduce_plot_payload(
    envelope: harmos::storage::ReadBatch<PlotPayloadEnvelope>, request: &PlotPayloadRequest, maximum: usize,
) -> crate::runtime::Result<harmos::storage::ReadBatch<PlotPayloadEnvelope>> {
    envelope.try_map(|mut value| {
        value.payload.max_points = maximum;
        value.payload = crate::plot::finish_plot_frame(value.payload, request)?;
        value.diagnostics = plot_payload_diagnostics(&value.payload);
        Ok(value)
    })
}

pub async fn query_plot_payloads(
    runtime: &RuntimeHandle, requests: Vec<PlotPayloadRequest>, mode: RuntimePlotQueryMode,
) -> crate::runtime::Result<Vec<harmos::storage::ReadBatch<PlotPayloadEnvelope>>> {
    let at = runtime.storage().cursor_saved();
    query_plot_payloads_at(runtime, requests, mode, &at).await
}

pub async fn query_plot_payloads_at(
    runtime: &RuntimeHandle, requests: Vec<PlotPayloadRequest>, mode: RuntimePlotQueryMode, at: &harmos::storage::StorageCursor,
) -> crate::runtime::Result<Vec<harmos::storage::ReadBatch<PlotPayloadEnvelope>>> {
    let session = runtime.storage().clone();
    let context = RuntimePlotEnvelopeContext::load_at(runtime, &requests, at, &session).await?;
    query_plot_payloads_with_context(&context, runtime, requests, mode, at, &session).await
}

pub async fn query_plot_stream_batches(
    runtime: &RuntimeHandle, requests: &[PlotPayloadRequest],
) -> crate::runtime::Result<Vec<RuntimePlotStreamBatch>> {
    Ok(query_plot_payloads(runtime, requests.to_vec(), RuntimePlotQueryMode::Replace)
        .await?
        .into_iter()
        .map(|envelope| RuntimePlotStreamBatch { mode: RuntimePlotQueryMode::Replace, envelope })
        .collect())
}

/* === Private Functions === */

fn plot_payload_context<'a>(
    request: &'a PlotPayloadRequest, contract: Option<&'a TelemetryContract>, run_started_at: Option<&'a Timestamp>,
    cycle_started_at: Option<&'a Timestamp>, default_max_points: usize,
) -> PlotPayloadContext<'a> {
    let column =
        |name: &str| -> Option<&TelemetryColumn> { contract?.columns.iter().find(|column| column.metadata.identifier.as_str() == name) };
    let x_key = request.x_key.as_deref().unwrap_or("timestamp");
    let x_format = resolved_format(request.x_format.as_ref(), column(x_key));
    let series_format = resolved_format(request.y_format.as_ref(), request.y_keys.first().and_then(|key| column(key)));

    PlotPayloadContext {
        default_max_points,
        x_format,
        series_format,
        run_started_at: run_started_at.map(|timestamp| timestamp.as_str()),
        cycle_started_at: cycle_started_at.map(|timestamp| timestamp.as_str()),
    }
}

async fn query_plot_payloads_with_context(
    context: &RuntimePlotEnvelopeContext, runtime: &RuntimeHandle, requests: Vec<PlotPayloadRequest>, _mode: RuntimePlotQueryMode,
    at: &harmos::storage::StorageCursor, session: &crate::storage::Manager,
) -> crate::runtime::Result<Vec<harmos::storage::ReadBatch<PlotPayloadEnvelope>>> {
    let requests = requests
        .into_iter()
        .map(|request| {
            let telemetry_streams = request_telemetry_streams(&request);
            canonical_plot_request(request, &telemetry_streams)
        })
        .collect::<Vec<_>>();
    let analyses = requests
        .iter()
        .map(|request| analysis::Prepared::resolve(runtime, context.snapshot.state(), request))
        .collect::<crate::runtime::Result<Vec<_>>>()?;
    let mut telemetry = Vec::with_capacity(requests.len());
    for request in &requests {
        let streams = request_telemetry_streams(request);
        if streams.is_empty() {
            let empty = harmos::storage::ReadBatch::new(())
                .try_map(|_| Ok::<_, crate::runtime::Error>(Vec::<crate::telemetry::TelemetrySample>::new()))?;
            telemetry.push(empty);
            continue;
        }
        let selection = TelemetrySelection {
            streams: streams.into_iter().map(Into::into).collect(),
            scope: scope_selection(request),
            ..Default::default()
        };
        let rows = runtime
            .collect_telemetry_at_in(
                session,
                &selection,
                request.analysis.as_ref().map(|_| dokime_domain::model::analysis::MAX_SAMPLES as usize),
                request.window_s.map(|seconds| (request.stream.as_str(), seconds)),
                at,
            )
            .await?;
        telemetry.push(rows);
    }
    let records = if requests.iter().any(|request| !request_record_types(request).is_empty()) {
        crate::storage::ReadRows::from(runtime.record_pages_at_in(session, &context.snapshot, at).await?)
    } else {
        crate::storage::ReadRows::default()
    };
    let selected_records = requests
        .iter()
        .map(|request| {
            let captured = super::records::execution_capture(
                context.snapshot.state().executions.values(),
                request.scope.as_ref().and_then(|scope| scope.test_execution_id.as_ref()),
            );
            plot_records(&records, request, captured)
        })
        .collect::<crate::runtime::Result<Vec<_>>>()?;
    drop(records);

    join_all(requests.into_iter().zip(telemetry).zip(selected_records).zip(analyses).map(
        |(((request, telemetry), records), analysis)| async move {
            if let Some(analysis) = &analysis {
                analysis.check_evidence(&telemetry, &request)?;
            }
            let telemetry = context.enrich_telemetry(telemetry, &request, &records)?;
            let revision = at.sequence;
            let worker_context = context.clone();
            let default_max_points = runtime.read_application_settings().await?.settings.report.plot_points as usize;
            let custom = analysis.is_some();
            let (payload, request) = tokio::task::spawn_blocking(move || {
                let contract = telemetry_contract(&worker_context, &request.stream);
                let (run_started_at, cycle_started_at) = worker_context.run_and_cycle_start(request.scope.as_ref());
                let format_context = plot_payload_context(&request, contract, run_started_at, cycle_started_at, default_max_points);
                let payload = project_retained_plot_payload(
                    &telemetry,
                    records.as_slice().into(),
                    &request,
                    worker_context.generated_at.as_str(),
                    &format_context,
                )?;
                let payload = if custom { payload } else { finish_retained_plot_payload(payload, &request)? };
                Ok::<_, crate::runtime::Error>((payload, request))
            })
            .await??;
            let payload = if let Some(analysis) = analysis {
                analysis.execute(runtime, &request, at, payload, telemetry_contract(context, &request.stream)).await?
            } else {
                payload
            };
            let diagnostics = plot_payload_diagnostics(&payload);
            let plot_id = request.plot_id.as_deref().unwrap_or(&request.stream);
            payload.try_map(|payload| {
                let mut owned_id = String::new();
                owned_id.try_reserve_exact(plot_id.len()).map_err(|_| plot_record_memory())?;
                owned_id.push_str(plot_id);
                Ok(PlotPayloadEnvelope { plot_id: owned_id, revision, payload, diagnostics, warnings: Vec::new() })
            })
        },
    ))
    .await
    .into_iter()
    .collect()
}

fn telemetry_contract<'a>(context: &'a RuntimePlotEnvelopeContext, stream: &str) -> Option<&'a TelemetryContract> {
    context.snapshot.state().telemetry_definitions.iter().find(|(id, _)| id.as_str() == stream).map(|(_, contract)| contract)
}

fn plot_records(
    records: &crate::storage::ReadRows<RecordEntry>, request: &PlotPayloadRequest,
    captured: Option<&dokime_domain::state::records::set::TestRecordSet>,
) -> crate::runtime::Result<harmos::storage::ReadBatch<Vec<RecordEntry>>> {
    let record_types = request_record_types(request);
    let scope = scope_selection(request);
    let selected = |record: &&RecordEntry| {
        let subscribed = captured.and_then(|captures| captures.subscribed_records().get(&record.record_id));
        record_types.contains(&record.record_type) && scope.matches(subscribed.or(record.scope.as_ref()))
    };
    let count = records.iter().filter(selected).count();
    let mut selected = Vec::new();
    selected.try_reserve_exact(count).map_err(|_| plot_record_memory())?;
    selected.extend(records.iter().enumerate().filter(|(_, record)| {
        let subscribed = captured.and_then(|captures| captures.subscribed_records().get(&record.record_id));
        record_types.contains(&record.record_type) && scope.matches(subscribed.or(record.scope.as_ref()))
    }));
    selected.sort_unstable_by(|(li, left), (ri, right)| {
        right.timestamp.cmp(&left.timestamp).then(right.sequence.cmp(&left.sequence)).then(li.cmp(ri))
    });
    harmos::storage::ReadBatch::new(()).try_map(|_| {
        let mut records = Vec::<RecordEntry>::new();
        records.try_reserve_exact(count).map_err(|_| plot_record_memory())?;
        for (_, record) in selected {
            let mut record = record.clone();
            if let Some(scope) = captured.and_then(|captures| captures.subscribed_records().get(&record.record_id)) {
                record.scope = Some(scope.clone());
            }
            records.push(record);
        }
        Ok(records)
    })
}

fn plot_record_memory() -> crate::runtime::Error {
    harmos::Error::ResourceExhausted { resource: "plot record selection" }.into()
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plot_refresh_follows_declared_inputs() {
        let mut request =
            PlotPayloadRequest { stream: "fixture.progress".to_owned(), y_keys: vec!["progress".to_owned()], ..Default::default() };
        let telemetry = PlotWatchKeys::for_request(&request);
        assert!(telemetry.reads_stream("fixture.progress"));
        assert!(!telemetry.reads_stream("fixture.unrelated"));
        assert!(!telemetry.follows_state());

        request.marker_record_types.push("prompt.response.submitted".to_owned());
        let mixed = PlotWatchKeys::for_request(&request);
        assert!(mixed.reads_stream("fixture.progress"));
        assert!(mixed.follows_state());

        request.stream.clear();
        request.y_keys.clear();
        let records = PlotWatchKeys::for_request(&request);
        assert!(!records.reads_stream("fixture.progress"));
        assert!(records.follows_state());
    }
}
