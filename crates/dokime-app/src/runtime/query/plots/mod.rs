/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod analysis;
mod context;
mod request;
mod stream;

/* === Re-Exports === */
pub use crate::plot::{
    BoxSummary, DownsampleTechnique, HistogramBin, HistogramSeries, HistogramStatistics,
    PlotAnalysis, PlotCartesianFrame, PlotCartesianMode, PlotCartesianSeriesFrame, PlotComputation,
    PlotComputedSeries, PlotDomain, PlotEvidence, PlotMarker, PlotPayload, PlotPayloadCursor,
    PlotPayloadDiagnostics, PlotPayloadEnvelope, PlotPayloadRequest, PlotPayloadStreamKind,
    PlotPoint, PlotSeries, PlotSeriesSummary, PlotWindow, SigmaBin, SigmaSeries,
    plot_payload_request_for_descriptor,
};
pub(crate) use analysis::definitions as analysis_definitions;
pub(crate) use request::request_telemetry_streams;
pub use stream::{
    RuntimePlotQueryMode, RuntimePlotStreamBatch, matching_plot_payload_request,
    merge_plot_payload_cursor, plot_payload_stream_envelope_visible, plot_payload_stream_kind,
};

/* === Imports === */
// Standard Library Imports

// Third Party Imports
use futures_util::future::join_all;
// Local Imports

use crate::plot::{
    PlotPayloadContext, build_plot_payload, plot_payload_diagnostics, project_plot_payload,
    resolved_format, summarizes, window_summaries,
};
use crate::records::RecordEntry;
use crate::{
    evidence::{RecordSelection, TelemetrySelection},
    handle::RuntimeHandle,
};
use context::RuntimePlotEnvelopeContext;
use dokime::core::{Identifier, Timestamp};
use dokime::telemetry::{TelemetryColumn, TelemetryContract, TelemetrySample};
use request::{
    canonical_plot_request, request_record_types, scope_selection, telemetry_read_groups,
};
use stream::{cursor_revision, response_cursor, stream_requests_by_mode};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */
/// Each request's selected rows, and for an append frame whose series are
/// summarized, every row of its window.
type WindowedTelemetry = (Vec<Vec<TelemetrySample>>, Vec<Option<Vec<TelemetrySample>>>);

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl RuntimeHandle {
    pub(crate) async fn analyze_projected_plot(
        &self,
        request: &PlotPayloadRequest,
        telemetry: &[dokime::telemetry::TelemetrySample],
        payload: crate::plot::PlotPayload,
        contract: Option<&dokime::telemetry::TelemetryContract>,
    ) -> crate::Result<crate::plot::PlotPayload> {
        let state = self.state_read();
        let Some(prepared) = analysis::Prepared::resolve(self, &state, request)? else {
            return Ok(payload);
        };
        prepared.check_evidence(telemetry, request)?;
        let mut cursor = crate::plot::PlotPayloadCursor::default();
        for sample in telemetry.iter().filter(|sample| {
            sample.stream.as_str() == request.stream
                && crate::plot::sample_matches_scope(sample.scope.as_ref(), request.scope.as_ref())
        }) {
            let key = crate::domains::telemetry::telemetry_stream_id(
                sample.scope.as_ref(),
                &sample.stream,
            );
            let ordinal = cursor.telemetry_streams.entry(key).or_default();
            *ordinal = (*ordinal).max(sample.sequence);
        }
        prepared
            .execute(self, request, &cursor, payload, contract)
            .await
    }
}

/* === Public Functions === */

pub async fn query_plot_payloads(
    runtime: &RuntimeHandle,
    requests: Vec<PlotPayloadRequest>,
    mode: RuntimePlotQueryMode,
) -> crate::Result<Vec<PlotPayloadEnvelope>> {
    let context = RuntimePlotEnvelopeContext::load(runtime, &requests).await?;
    query_plot_payloads_with_context(&context, runtime, requests, mode).await
}

pub async fn query_plot_stream_batches(
    runtime: &RuntimeHandle,
    requests: &[PlotPayloadRequest],
) -> crate::Result<Vec<RuntimePlotStreamBatch>> {
    let (append_requests, replace_requests) = stream_requests_by_mode(requests);
    let mut batches = Vec::with_capacity(append_requests.len() + replace_requests.len());
    let context = RuntimePlotEnvelopeContext::load(runtime, requests).await?;

    if !append_requests.is_empty() {
        batches.extend(
            query_plot_payloads_with_context(
                &context,
                runtime,
                append_requests,
                RuntimePlotQueryMode::Append,
            )
            .await?
            .into_iter()
            .map(|envelope| RuntimePlotStreamBatch {
                mode: RuntimePlotQueryMode::Append,
                envelope,
            }),
        );
    }
    if !replace_requests.is_empty() {
        batches.extend(
            query_plot_payloads_with_context(
                &context,
                runtime,
                replace_requests,
                RuntimePlotQueryMode::Replace,
            )
            .await?
            .into_iter()
            .map(|envelope| RuntimePlotStreamBatch {
                mode: RuntimePlotQueryMode::Replace,
                envelope,
            }),
        );
    }

    Ok(batches)
}

/* === Private Functions === */
/// Reads the plot's records once shared storage read capacity is free.
/// Concurrent readers, such as finalization captures, release it as they
/// settle, so a plot backs off for them within the interactive deadline; past
/// it the refusal reaches the plot as its error, and a watch retries later.
async fn records_when_admitted(
    runtime: &RuntimeHandle,
    snapshot: &crate::queries::RuntimeStateSnapshot,
) -> crate::Result<Vec<RecordEntry>> {
    let deadline = tokio::time::Instant::now() + runtime.config().storage_reads.interactive.timeout;
    let mut backoff = crate::runtime::storage::record_capture::CapacityBackoff::new();
    loop {
        match runtime.records_at(snapshot).await {
            Err(error)
                if crate::runtime::storage::record_capture::read_capacity(&error)
                    && backoff.wait_until(deadline).await => {}
            result => return result,
        }
    }
}

fn plot_payload_context(
    request: &PlotPayloadRequest,
    contract: Option<&TelemetryContract>,
    run_started_at: Option<&Timestamp>,
    cycle_started_at: Option<&Timestamp>,
) -> PlotPayloadContext {
    let column = |name: &str| -> Option<&TelemetryColumn> {
        contract?
            .columns
            .iter()
            .find(|column| column.metadata.identifier.as_str() == name)
    };
    let x_key = request.x_key.as_deref().unwrap_or("timestamp");
    let x_format = resolved_format(request.x_format.as_ref(), column(x_key));
    let series_format = resolved_format(
        request.y_format.as_ref(),
        request.y_keys.first().and_then(|key| column(key)),
    );

    PlotPayloadContext {
        x_format,
        series_format,
        run_started_at: run_started_at.map(|timestamp| timestamp.as_str().to_owned()),
        cycle_started_at: cycle_started_at.map(|timestamp| timestamp.as_str().to_owned()),
    }
}

async fn query_plot_payloads_with_context(
    context: &RuntimePlotEnvelopeContext,
    runtime: &RuntimeHandle,
    requests: Vec<PlotPayloadRequest>,
    mode: RuntimePlotQueryMode,
) -> crate::Result<Vec<PlotPayloadEnvelope>> {
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
        .collect::<crate::Result<Vec<_>>>()?;
    let telemetry_reads = collect_plot_telemetry(runtime, &requests, mode);
    let record_reads = async {
        if requests
            .iter()
            .any(|request| !request_record_types(request).is_empty())
        {
            records_when_admitted(runtime, &context.snapshot).await
        } else {
            Ok(Vec::new())
        }
    };
    let (telemetry, records) = tokio::join!(telemetry_reads, record_reads);
    let (telemetry, window_telemetry) = telemetry?;
    let records = records?;
    let selected_records = requests
        .iter()
        .map(|request| {
            let captured = super::records::execution_capture(
                context.snapshot.state().executions.values(),
                request
                    .scope
                    .as_ref()
                    .and_then(|scope| scope.test_execution_id.as_ref()),
            );
            plot_records(&records, request, mode, captured)
        })
        .collect::<Vec<_>>();
    // An append frame carries only the rows after its cursor, but its series
    // summaries cover the whole window: they read the rows the cursor filter
    // dropped from the same read.
    let window_reads = window_telemetry
        .into_iter()
        .zip(&requests)
        .map(|(telemetry, request)| {
            let telemetry = telemetry?;
            let captured = super::records::execution_capture(
                context.snapshot.state().executions.values(),
                request
                    .scope
                    .as_ref()
                    .and_then(|scope| scope.test_execution_id.as_ref()),
            );
            let records = plot_records(&records, request, RuntimePlotQueryMode::Replace, captured);
            Some((telemetry, records))
        })
        .collect::<Vec<_>>();

    join_all(
        requests
            .into_iter()
            .zip(telemetry)
            .zip(selected_records)
            .zip(window_reads.into_iter().zip(analyses))
            .map(|(((request, telemetry), records), reads)| async move {
                let (window, analysis) = reads;
                if let Some(analysis) = &analysis {
                    analysis.check_evidence(&telemetry, &request)?;
                }
                let cursor = response_cursor(&telemetry, &records, request.cursor.as_ref());
                let mut telemetry = telemetry;
                telemetry.extend(record_context_samples(&records, &request));
                let telemetry = context.enrich_telemetry(telemetry, &request);
                let window = window.map(|(mut telemetry, records)| {
                    telemetry.extend(record_context_samples(&records, &request));
                    (context.enrich_telemetry(telemetry, &request), records)
                });
                let revision = cursor_revision(&cursor);
                let plot_id = request
                    .plot_id
                    .clone()
                    .unwrap_or_else(|| request.stream.clone());
                let delta = mode == RuntimePlotQueryMode::Append;
                let generated_at = context.generated_at.clone();
                let contract = Identifier::new(&request.stream).ok().and_then(|stream| {
                    context
                        .snapshot
                        .state()
                        .telemetry_definitions
                        .get(&stream)
                        .cloned()
                });
                let (run_started_at, cycle_started_at) =
                    context.run_and_cycle_start(request.scope.as_ref());
                let format_context = plot_payload_context(
                    &request,
                    contract.as_ref(),
                    run_started_at.as_ref(),
                    cycle_started_at.as_ref(),
                );
                let custom = analysis.is_some();
                let (payload, request) = tokio::task::spawn_blocking(move || {
                    let mut payload = if custom {
                        project_plot_payload(
                            &telemetry,
                            &records,
                            &request,
                            &generated_at,
                            &format_context,
                        )
                    } else {
                        build_plot_payload(
                            &telemetry,
                            &records,
                            request.clone(),
                            &generated_at,
                            &format_context,
                        )
                    };
                    if let Some((telemetry, records)) = window {
                        payload.summaries = window_summaries(
                            &telemetry,
                            &records,
                            &request,
                            &generated_at,
                            &format_context,
                        );
                    }
                    (payload, request)
                })
                .await?;
                let payload = if let Some(analysis) = analysis {
                    analysis
                        .execute(runtime, &request, &cursor, payload, contract.as_ref())
                        .await?
                } else {
                    payload
                };
                let diagnostics = plot_payload_diagnostics(&payload, delta);
                Ok(PlotPayloadEnvelope {
                    plot_id,
                    revision,
                    cursor,
                    payload,
                    diagnostics,
                    warnings: Vec::new(),
                })
            }),
    )
    .await
    .into_iter()
    .collect()
}

async fn collect_plot_telemetry(
    runtime: &RuntimeHandle,
    requests: &[PlotPayloadRequest],
    mode: RuntimePlotQueryMode,
) -> crate::Result<WindowedTelemetry> {
    let groups = telemetry_read_groups(requests);
    let collected = join_all(groups.iter().map(|group| {
        let bounded = group
            .request_indexes
            .iter()
            .any(|&index| requests[index].analysis.is_some());
        let selection = TelemetrySelection {
            streams: group.streams.clone(),
            scope: group.scope.clone(),
            ..TelemetrySelection::default()
        };
        async move {
            runtime
                .try_collect_telemetry_bounded(
                    &selection,
                    bounded.then_some(dokime::analysis::MAX_SAMPLES as usize),
                )
                .await
        }
    }))
    .await;
    let mut telemetry = vec![Vec::new(); requests.len()];
    let mut window = vec![None; requests.len()];
    for (group, samples) in groups.into_iter().zip(collected) {
        let samples = samples?;
        for request_index in group.request_indexes {
            let request = &requests[request_index];
            telemetry[request_index] = select_plot_telemetry(&samples, request, mode);
            if mode == RuntimePlotQueryMode::Append && summarizes(request) {
                window[request_index] = Some(select_plot_telemetry(
                    &samples,
                    request,
                    RuntimePlotQueryMode::Replace,
                ));
            }
        }
    }
    Ok((telemetry, window))
}

fn select_plot_telemetry(
    samples: &[TelemetrySample],
    request: &PlotPayloadRequest,
    mode: RuntimePlotQueryMode,
) -> Vec<TelemetrySample> {
    let after_stream_cursors = if mode == RuntimePlotQueryMode::Append {
        request
            .cursor
            .as_ref()
            .map(|cursor| &cursor.telemetry_streams)
    } else {
        None
    };
    samples
        .iter()
        .filter(|sample| {
            let Some(after_stream_cursors) = after_stream_cursors else {
                return true;
            };
            let stream = crate::telemetry_runtime::telemetry_stream_id(
                sample.scope.as_ref(),
                &sample.stream,
            );
            after_stream_cursors
                .get(&stream)
                .is_none_or(|after| sample.sequence > *after)
        })
        .cloned()
        .collect::<Vec<_>>()
}

fn record_context_samples(
    records: &[RecordEntry],
    request: &PlotPayloadRequest,
) -> Vec<TelemetrySample> {
    if !request::request_needs_dimension_context(request) {
        return Vec::new();
    }
    let stream =
        Identifier::new(crate::plot::RECORD_CONTEXT_STREAM).expect("internal stream is valid");
    records
        .iter()
        .map(|record| {
            let mut sample = TelemetrySample::new(
                stream.clone(),
                record.sequence,
                record.timestamp.clone(),
                "plot_context",
            );
            sample.scope = record.scope.clone();
            sample
        })
        .collect()
}

fn plot_records(
    records: &[RecordEntry],
    request: &PlotPayloadRequest,
    mode: RuntimePlotQueryMode,
    captured: Option<&dokime::records::TestRecordSet>,
) -> Vec<RecordEntry> {
    let record_types = request_record_types(request);
    if record_types.is_empty() {
        return Vec::new();
    }

    let selection = RecordSelection {
        after_sequence: if mode == RuntimePlotQueryMode::Append {
            request
                .cursor
                .as_ref()
                .and_then(|cursor| cursor.record_sequence)
        } else {
            None
        },
        limit: None,
        record_types,
        scope: scope_selection(request),
        ..RecordSelection::default()
    };

    selection.apply_with_subscriptions(records, |record| {
        captured.and_then(|captures| captures.subscribed_records().get(&record.record_id))
    })
}

/* === Tests === */
