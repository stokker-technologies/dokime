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
mod cursor;
mod wire;

/* === Re-Exports === */
pub(super) use wire::{data_expression_wire, document_plot_definition, plot_filter_wire};

/* === Imports === */
// Standard Library Imports
use std::{sync::Arc, time::Duration};
// Third Party Imports
use futures_util::{StreamExt, stream};
use harmos::{
    MemorySize, Protobuf,
    operations::{Context, Operation, Response, StreamOutput},
};
use tokio::time::Instant;
use tonic::Status;
// Local Imports
use cursor::{decode_plot_requests, selection_for_plot};
use dokime_runtime::{
    RuntimeHandle,
    runtime::query::{
        RuntimePlotQueryMode, matching_plot_payload_request, merge_plot_payload_cursor,
        plot_payload_stream_envelope_visible, plot_payload_stream_kind,
    },
};
use dokime_contract::operations::plots as contract;
use wire::{ensure_message_budget, envelope, stream_frame};

/* === Constants === */
const PLOT_STREAM_THROTTLE: Duration = Duration::from_millis(100);
/// The first pause before a watch retries plots whose read was refused.
const PLOT_RETRY_FIRST: Duration = Duration::from_millis(250);
/// The longest pause between retries of refused plot reads.
const PLOT_RETRY_MAX: Duration = Duration::from_secs(5);

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct Resources {
    runtime: RuntimeHandle,
    maximum: usize,
}
struct Analyses {
    resources: Arc<Resources>,
}
struct Snapshots {
    resources: Arc<Resources>,
}
struct Watch {
    resources: Arc<Resources>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Resources {
    fn produce<T: MemorySize + Protobuf + Send + Sync + 'static>(
        &self,
        build: impl FnOnce() -> Result<T, Status>,
    ) -> Result<Response<T>, Status> {
        let output = build()?;
        ensure_message_budget(&output, self.maximum, "plot response")?;
        Ok(output.into())
    }
}
impl Resources {
    /// One stream frame for `value`, advancing its request's cursor. A frame
    /// that cannot be served is sent as the plot's error instead and leaves
    /// the cursor where it was; one refused for its size also takes its
    /// request out of the watch, so a later small append cannot hide that the
    /// plot never received its window.
    fn frame(
        &self,
        requests: &mut Vec<dokime_runtime::runtime::query::PlotPayloadRequest>,
        current: &dokime_runtime::runtime::query::PlotPayloadRequest,
        kind: dokime_runtime::runtime::query::PlotPayloadStreamKind,
        value: dokime_runtime::runtime::query::PlotPayloadEnvelope,
        epoch: &str,
        selection: &str,
    ) -> Result<Response<contract::PlotStreamFrame>, Status> {
        let mut advanced = requests.clone();
        merge_plot_payload_cursor(&mut advanced, &value);
        let served = stream_frame(kind, value, epoch, selection, self.maximum)
            .and_then(|frame| self.produce(|| Ok(frame)));
        match served {
            Ok(response) => {
                *requests = advanced;
                Ok(response)
            }
            Err(error) => {
                if error.code() == tonic::Code::InvalidArgument {
                    requests.retain(|request| request.plot_id != current.plot_id);
                }
                self.produce(|| Ok(plot_error_frame(current, error.message().to_owned())))
            }
        }
    }
}
impl Resources {
    /// Sends one cursorless plot's snapshot. A read that fails is sent as the
    /// plot's error instead, and false asks the watch to retry it.
    async fn snapshot(
        &self,
        output: &mut StreamOutput<contract::PlotStreamFrame>,
        requests: &mut Vec<dokime_runtime::runtime::query::PlotPayloadRequest>,
        selections: &[String],
        epoch: &str,
        current: &dokime_runtime::runtime::query::PlotPayloadRequest,
    ) -> Result<bool, Status> {
        let native = match self
            .runtime
            .query_plot_payloads(vec![current.clone()], RuntimePlotQueryMode::Snapshot)
            .await
        {
            Ok(native) => native,
            Err(error) => {
                let response = self.produce(|| Ok(plot_error_frame(current, error.to_string())))?;
                output.send_owned(response).await.map_err(failure)?;
                return Ok(false);
            }
        };
        for value in native {
            let selection = selection_for_plot(requests, selections, &value.plot_id)
                .ok_or_else(|| Status::internal("plot response does not match a request"))?
                .to_owned();
            let response = self.frame(
                requests,
                current,
                dokime_runtime::runtime::query::PlotPayloadStreamKind::Snapshot,
                value,
                epoch,
                &selection,
            )?;
            output.send_owned(response).await.map_err(failure)?;
        }
        Ok(true)
    }
}
impl Operation for Analyses {
    type Declaration = contract::ListAnalyses;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();
    async fn initialize<'ctx>(
        resources: Arc<Resources>,
        _: Self::Context<'ctx>,
    ) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self,
        _: Self::Context<'a>,
        request: &mut contract::ListAnalysesRequest,
        _: &mut (),
    ) -> Result<Response<contract::ListAnalysesReply>, Status> {
        super::value::text(&request.after_id)?;
        let limit = if request.limit == 0 {
            64
        } else {
            request.limit as usize
        };
        if limit > 128 {
            return Err(Status::resource_exhausted(
                "analysis page exceeds 128 definitions",
            ));
        }
        let definitions = self
            .resources
            .runtime
            .analysis_definitions()
            .await
            .map_err(|error| Status::unavailable(error.to_string()))?;
        let mut page = definitions
            .iter()
            .filter(|definition| definition.id > request.after_id)
            .take(limit + 1)
            .collect::<Vec<_>>();
        let next_after_id = (page.len() > limit).then(|| page[limit - 1].id.clone());
        page.truncate(limit);
        self.resources.produce(|| {
            Ok(contract::ListAnalysesReply {
                definitions: page
                    .into_iter()
                    .map(analysis::definition)
                    .collect::<Result<_, _>>()?,
                next_after_id,
            })
        })
    }
}
impl Operation for Snapshots {
    type Declaration = contract::QuerySnapshots;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();
    async fn initialize<'ctx>(
        resources: Arc<Resources>,
        _: Self::Context<'ctx>,
    ) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self,
        _: Self::Context<'a>,
        request: &mut contract::PlotSnapshotsRequest,
        _: &mut (),
    ) -> Result<Response<contract::PlotSnapshotsReply>, Status> {
        let epoch = self.resources.runtime.process_epoch().to_string();
        let (requests, selections) =
            decode_plot_requests(std::mem::take(&mut request.requests), &epoch)?;
        let selection_requests = requests.clone();
        let native = self
            .resources
            .runtime
            .query_plot_payloads(requests, RuntimePlotQueryMode::Snapshot)
            .await
            .map_err(|error| Status::unavailable(format!("plot query unavailable: {error}")))?;
        self.resources.produce(|| {
            let envelopes = native
                .into_iter()
                .map(|value| {
                    let selection =
                        selection_for_plot(&selection_requests, &selections, &value.plot_id)
                            .ok_or_else(|| {
                                Status::internal("plot response does not match a request")
                            })?;
                    envelope(value, &epoch, selection, self.resources.maximum)
                })
                .collect::<Result<Vec<_>, Status>>()?;
            Ok(contract::PlotSnapshotsReply { envelopes })
        })
    }
}
impl Operation for Watch {
    type Declaration = contract::Watch;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = StreamOutput<contract::PlotStreamFrame>;
    async fn initialize<'ctx>(
        resources: Arc<Resources>,
        _: Self::Context<'ctx>,
    ) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self,
        _: Self::Context<'a>,
        request: &mut contract::PlotWatchRequest,
        output: &mut Self::Endpoints,
    ) -> Result<Response<()>, Status> {
        let runtime = &self.resources.runtime;
        let epoch = runtime.process_epoch().to_string();
        let (mut requests, selections) =
            decode_plot_requests(std::mem::take(&mut request.requests), &epoch)?;
        // Subscribe Before Reading So A Racing Commit Advances The Same Native Watches.
        let mut states = runtime.watch_state();
        let mut watches = Vec::new();
        for current in &requests {
            match runtime
                .watch_plot_telemetry(std::slice::from_ref(current))
                .await
            {
                Ok(current_watches) => watches.extend(current_watches),
                Err(error) => {
                    let response = self
                        .resources
                        .produce(|| Ok(plot_error_frame(current, error.to_string())))?;
                    output.send_owned(response).await.map_err(failure)?;
                }
            }
        }
        let mut telemetry_watches = stream::select_all(watches);
        // A refused read is sent as its plot's error at once and retried with
        // backoff even while no new evidence arrives, so a contended plot
        // neither waits silently nor stays failed once capacity frees.
        let mut retry_delay = PLOT_RETRY_FIRST;
        let mut unserved = Vec::new();
        let initial = requests
            .iter()
            .filter(|request| request.cursor.is_none())
            .cloned()
            .collect::<Vec<_>>();
        for current in initial {
            if !self
                .resources
                .snapshot(output, &mut requests, &selections, &epoch, &current)
                .await?
            {
                unserved.push(current);
            }
        }
        let mut last_flush = Instant::now() - PLOT_STREAM_THROTTLE;
        let mut next_flush = None::<Instant>;
        if !unserved.is_empty() {
            next_flush = Some(Instant::now() + retry_delay);
            retry_delay = (retry_delay * 2).min(PLOT_RETRY_MAX);
        }
        loop {
            tokio::select! {
                biased;
                changed = states.next() => {
                    if changed.is_none() {
                        return Ok(().into());
                    }
                    if next_flush.is_none() {
                        next_flush = Some(last_flush + PLOT_STREAM_THROTTLE);
                    }
                }
                advanced = telemetry_watches.next(), if !telemetry_watches.is_empty() => {
                    if advanced.is_some() && next_flush.is_none() {
                        next_flush = Some(last_flush + PLOT_STREAM_THROTTLE);
                    }
                }
                _ = tokio::time::sleep_until(next_flush.unwrap_or_else(Instant::now)), if next_flush.is_some() => {
                    next_flush = None;
                    last_flush = Instant::now();
                    let mut refused = false;
                    for current in std::mem::take(&mut unserved) {
                        if !self.resources.snapshot(output, &mut requests, &selections, &epoch, &current).await? {
                            refused = true;
                            unserved.push(current);
                        }
                    }
                    let served = requests
                        .iter()
                        .filter(|request| !unserved.iter().any(|pending| pending.plot_id == request.plot_id))
                        .cloned()
                        .collect::<Vec<_>>();
                    for current in served {
                        let batches = match runtime.query_plot_stream_batches(std::slice::from_ref(&current)).await {
                            Ok(batches) => batches,
                            Err(error) => {
                                refused = true;
                                let response = self.resources.produce(|| Ok(plot_error_frame(&current, error.to_string())))?;
                                output.send_owned(response).await.map_err(failure)?;
                                continue;
                            }
                        };
                        for batch in batches {
                        let mode = batch.mode; let value = batch.envelope;
                        let Some(current) = matching_plot_payload_request(&requests, &value.plot_id).cloned() else { continue; };
                        if !plot_payload_stream_envelope_visible(mode, &value, &current) { continue; }
                        let selection = selection_for_plot(&requests, &selections, &value.plot_id).ok_or_else(|| Status::internal("plot response does not match a request"))?.to_owned();
                        let response = self.resources.frame(&mut requests, &current, plot_payload_stream_kind(mode), value, &epoch, &selection)?;
                        output.send_owned(response).await.map_err(failure)?;
                        }
                    }
                    if refused {
                        next_flush = Some(Instant::now() + retry_delay);
                        retry_delay = (retry_delay * 2).min(PLOT_RETRY_MAX);
                    } else {
                        retry_delay = PLOT_RETRY_FIRST;
                    }
                }
            }
        }
    }
}

/* === Public Functions === */
pub(super) fn register(runtime: &RuntimeHandle, maximum: usize) -> Result<(), super::Error> {
    let resources = Arc::new(Resources {
        runtime: runtime.clone(),
        maximum,
    });
    runtime.operations().register_wire_operation::<Analyses>(
        "dokime",
        dokime_runtime::app::Lanes::Open,
        resources.clone(),
    )?;
    runtime.operations().register_wire_operation::<Snapshots>(
        "dokime",
        dokime_runtime::app::Lanes::Open,
        resources.clone(),
    )?;
    runtime.operations().register_wire_operation::<Watch>(
        "dokime",
        dokime_runtime::app::Lanes::Open,
        resources,
    )?;
    Ok(())
}

/* === Private Functions === */
fn plot_error_frame(
    request: &dokime_runtime::runtime::query::PlotPayloadRequest,
    message: String,
) -> contract::PlotStreamFrame {
    contract::PlotStreamFrame {
        kind: contract::PlotStreamKind::Error,
        envelope: Some(contract::PlotEnvelope {
            plot_id: request
                .plot_id
                .clone()
                .unwrap_or_else(|| request.stream.clone()),
            revision: 0,
            cursor: None,
            payload: None,
            diagnostics: None,
            warnings: vec![message],
        }),
    }
}

fn failure(error: harmos::Error) -> Status {
    let status = if matches!(error, harmos::Error::ResourceExhausted { .. }) {
        Status::resource_exhausted("plot admission failed")
    } else {
        Status::internal("plot admission failed")
    };
    super::error::detailed(status, &error.into())
}

/* === Tests === */
