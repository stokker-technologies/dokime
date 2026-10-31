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
pub(crate) use request::{request_record_types, request_telemetry_streams};
pub use stream::{RuntimePlotQueryMode, RuntimePlotStreamBatch, matching_plot_payload_request, plot_payload_stream_kind};

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::plot::{
    PlotPayloadContext, finish_retained_plot_payload, plot_payload_diagnostics, project_retained_plot_payload, resolved_format,
};
use crate::telemetry::{TelemetryColumn, TelemetryContract};
use crate::{evidence_runtime::TelemetrySelection, handle::RuntimeHandle};
use context::RuntimePlotEnvelopeContext;
use dokime_domain::model::time::Timestamp;
use request::{canonical_plot_request, scope_selection};

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

impl RuntimeHandle {
    /// Telemetry Streams With Newly Durable Rows Between Two Saved Frontiers.
    pub async fn plot_streams_saved_between(
        &self, after: &harmos::storage::StorageCursor, at: &harmos::storage::StorageCursor,
    ) -> crate::runtime::Result<std::collections::BTreeSet<String>> {
        if !at.covers(after)? {
            return Err(crate::runtime::Error::Operation("plot saved frontier moved backwards".into()));
        }
        let mut position = after.sequence;
        let mut streams = std::collections::BTreeSet::new();
        while position < at.sequence {
            let page = self.local_storage.telemetry_page(at.clone(), position, None, None).await?;
            if page.is_empty() {
                break;
            }
            for (at, entry) in page {
                position = at;
                streams.insert(entry.sample.stream.to_string());
            }
        }
        Ok(streams)
    }
}

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
    let at = runtime.storage().cursor_accepted();
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
        .map(|envelope| RuntimePlotStreamBatch {
            mode: RuntimePlotQueryMode::Replace,
            envelope,
        })
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
    let mut completed = Vec::with_capacity(requests.len());
    let mut shared = Vec::<(String, harmos::storage::ReadBatch<PlotPayloadEnvelope>)>::new();
    for request in requests {
        let streams = request_telemetry_streams(&request);
        let request = canonical_plot_request(request, &streams);
        let mut key = serde_json::to_value(&request).map_err(|error| crate::runtime::Error::Operation(error.to_string()))?;
        key.as_object_mut().expect("plot request is an object").remove("plot_id");
        let key = key.to_string();
        if let Some((_, previous)) = shared.iter().find(|(candidate, _)| candidate == &key) {
            let id = request.plot_id.as_deref().unwrap_or(&request.stream).to_owned();
            let previous = harmos::storage::ReadBatch::new((**previous).clone());
            completed.push(previous.try_map(|mut envelope| {
                envelope.plot_id = id;
                Ok::<_, crate::runtime::Error>(envelope)
            })?);
            continue;
        }
        let payload = if crate::plot::stored::supports(&request) && request_record_types(&request).is_empty() {
            let mut schema_at = at.clone();
            schema_at.sequence = schema_at.sequence.min(session.cursor_saved().sequence);
            let codecs = crate::runtime::storage::telemetry::saved_codecs(session, &schema_at, None, runtime.schema_settings(), |id, _| {
                id.strip_prefix("dokime.telemetry.") == Some(request.stream.as_str())
            })
            .await?;
            if codecs.len() == 1 && crate::plot::stored::supports_codec(&request, &codecs[0]) {
                let codec = &codecs[0];
                let selection = TelemetrySelection {
                    streams: vec![
                        dokime_domain::model::identity::IdentifierPattern::new(request.stream.clone())
                            .map_err(|error| crate::runtime::Error::Operation(error.to_string()))?,
                    ],
                    scope: scope_selection(&request),
                    ..Default::default()
                };
                let query = crate::runtime::storage::telemetry::plot_query(
                    at,
                    &selection,
                    codec,
                    request.window_s.map(|seconds| (request.stream.as_str(), seconds)),
                    &runtime.local_storage,
                )
                .await?;
                let maximum = runtime.read_application_settings().await?.settings.report.plot_points as usize;
                let contract = telemetry_contract(context, &request.stream);
                let (run, cycle) = context.run_and_cycle_start(request.scope.as_ref());
                let format = plot_payload_context(&request, contract, run, cycle, maximum);
                let payload = match query {
                    Some(query) => {
                        crate::plot::stored::materialize(
                            session,
                            &runtime.local_storage,
                            crate::plot::stored::Selection {
                                query,
                                scope: &selection.scope,
                                producer_order: false,
                            },
                            &request,
                            context.generated_at.as_str(),
                            &format,
                            crate::records::view::View::default(),
                        )
                        .await?
                    }
                    None => finish_retained_plot_payload(
                        project_retained_plot_payload(
                            &[],
                            crate::records::view::View::default(),
                            &request,
                            context.generated_at.as_str(),
                            &format,
                        )?,
                        &request,
                    )?,
                };
                let diagnostics = plot_payload_diagnostics(&payload);
                Some(payload.try_map(|payload| {
                    Ok::<_, crate::runtime::Error>(PlotPayloadEnvelope {
                        plot_id: request.plot_id.as_deref().unwrap_or(&request.stream).to_owned(),
                        revision: at.sequence,
                        payload,
                        diagnostics,
                        warnings: Vec::new(),
                    })
                })?)
            } else {
                None
            }
        } else {
            None
        };
        let envelope = match payload {
            Some(payload) => payload,
            None => query_spilled_plot(context, runtime, request, at, session).await?,
        };
        shared.push((key, envelope.clone()));
        completed.push(envelope);
    }
    Ok(completed)
}

async fn query_spilled_plot(
    context: &RuntimePlotEnvelopeContext, runtime: &RuntimeHandle, request: PlotPayloadRequest, at: &harmos::storage::StorageCursor,
    session: &crate::storage::Manager,
) -> crate::runtime::Result<harmos::storage::ReadBatch<PlotPayloadEnvelope>> {
    let analysis = analysis::Prepared::resolve(runtime, context.snapshot.state(), &request)?;
    let input = crate::plot::spilled::Input::new()?;
    input.reverse_records()?;
    let record_types = request_record_types(&request);
    let scope = scope_selection(&request);
    let captured = super::records::execution_capture(
        context.snapshot.state().executions.values(),
        request.scope.as_ref().and_then(|scope| scope.test_execution_id.as_ref()),
    );
    let visible = super::records::view_filter(&context.snapshot, None, captured);
    if !record_types.is_empty() {
        let mut records_at = at.clone();
        records_at.sequence = records_at.sequence.min(session.cursor_saved().sequence);
        runtime
            .record_source
            .visit_saved_records_at_in(session, context.snapshot.revision(), &records_at, |page| {
                for record in page.iter() {
                    let subscribed = captured.and_then(|captures| captures.captured_scope(record));
                    if !visible(record)
                        || !record_types.contains(&record.record_type)
                        || !(scope.matches_evidence(record.scope.as_ref(), &record.captured_scopes)
                            || subscribed.is_some_and(|scope_value| scope.matches(Some(scope_value))))
                    {
                        continue;
                    }
                    let mut record = record.clone();
                    if let Some(scope) =
                        subscribed.or_else(|| record.captured_scopes.iter().find(|candidate| scope.matches(Some(candidate))))
                    {
                        record.scope = Some(scope.clone());
                    }
                    input.append_record(&record)?;
                    if !request::request_needs_dimension_context(&request) {
                        continue;
                    }
                    let stream =
                        dokime_domain::model::identity::Identifier::new(crate::plot::RECORD_CONTEXT_STREAM).expect("internal stream");
                    let mut sample = crate::telemetry::TelemetrySample::new(
                        crate::model::identity::new_id(),
                        stream,
                        record.sequence,
                        record.timestamp.clone(),
                        "plot_context",
                    );
                    sample.scope = record.scope.clone();
                    context.enrich_sample(&mut sample, &request);
                    input.append(&sample)?;
                }
                Ok(())
            })
            .await?;
    }
    let streams = request_telemetry_streams(&request);
    if !streams.is_empty() {
        let selection = TelemetrySelection {
            streams: streams.into_iter().map(Into::into).collect(),
            scope,
            ..Default::default()
        };
        let options = crate::runtime::storage::telemetry::ReadOptions {
            scan_limit: analysis.as_ref().map(|_| dokime_domain::model::analysis::MAX_SAMPLES as usize),
            window: request.window_s.map(|seconds| (request.stream.as_str(), seconds)),
            sequence: crate::runtime::storage::telemetry::Sequence::StoragePosition,
            ..Default::default()
        };
        crate::runtime::storage::telemetry::visit_saved(
            session,
            at,
            &selection,
            runtime.schema_settings(),
            options,
            &runtime.local_storage,
            |mut sample| {
                if let Some(selected) = &request.scope
                    && let Some(captured) = sample.captured_scopes.iter().find(|scope| scope.is_visible_in(selected))
                {
                    sample.scope = Some(captured.clone());
                }
                context.enrich_sample(&mut sample, &request);
                input.append(&sample)
            },
        )
        .await?;
    }
    if let Some(analysis) = &analysis {
        let samples = input.collect(dokime_domain::model::analysis::MAX_SAMPLES as usize)?;
        analysis.check_evidence(&samples, &request)?;
    }
    let maximum = runtime.read_application_settings().await?.settings.report.plot_points as usize;
    let worker_context = context.clone();
    let (payload, request) = tokio::task::spawn_blocking(move || {
        let contract = telemetry_contract(&worker_context, &request.stream);
        let (run, cycle) = worker_context.run_and_cycle_start(request.scope.as_ref());
        let format = plot_payload_context(&request, contract, run, cycle, maximum);
        let payload = crate::plot::spilled::project(&input, &request, worker_context.generated_at.as_str(), &format)?;
        Ok::<_, crate::runtime::Error>((payload, request))
    })
    .await??;
    let payload = if let Some(analysis) = analysis {
        analysis.execute(runtime, &request, at, payload, telemetry_contract(context, &request.stream)).await?
    } else {
        payload
    };
    let diagnostics = plot_payload_diagnostics(&payload);
    payload.try_map(|payload| {
        Ok::<_, crate::runtime::Error>(PlotPayloadEnvelope {
            plot_id: request.plot_id.as_deref().unwrap_or(&request.stream).to_owned(),
            revision: at.sequence,
            payload,
            diagnostics,
            warnings: Vec::new(),
        })
    })
}

fn telemetry_contract<'a>(context: &'a RuntimePlotEnvelopeContext, stream: &str) -> Option<&'a TelemetryContract> {
    context.snapshot.state().telemetry_definitions.iter().find(|(id, _)| id.as_str() == stream).map(|(_, contract)| contract)
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plot_refresh_follows_declared_inputs() {
        let mut request = PlotPayloadRequest {
            stream: "fixture.progress".to_owned(),
            y_keys: vec!["progress".to_owned()],
            ..Default::default()
        };
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
