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
use std::{collections::BTreeSet, sync::Arc, time::Duration};

// Third Party Imports
use futures_util::{Stream, StreamExt, stream};
use harmos::domain::Portable;
use harmos::{
    MemorySize,
    operations::{Context, Operation, Response, StreamOutput},
};
use tokio::time::Instant;
use tonic::Status;

// Local Imports
use cursor::{decode_plot_requests, selection_for_plot};
use dokime_domain::operations::plots as contract;
use dokime_runtime::{
    RuntimeHandle,
    runtime::query::{
        PlotWatchKeys, RuntimePlotQueryMode, matching_plot_payload_request, merge_plot_payload_cursor,
        plot_payload_stream_envelope_visible, plot_payload_stream_kind,
    },
};
use wire::{FrameError, ensure_message_budget, envelope, stream_frame};

/* === Constants === */

const PLOT_STREAM_THROTTLE: Duration = Duration::from_millis(100);
const PLOT_RETRY_FIRST: Duration = Duration::from_millis(250);
const PLOT_RETRY_MAX: Duration = Duration::from_secs(5);
/// How often a watch says it is alive while a plot still waits for its first frame.
const PLOT_KEEPALIVE: Duration = Duration::from_secs(3);
/// Initial subscriptions and snapshots read at once, so one slow plot does not queue the rest.
const PLOT_INITIAL_CONCURRENCY: usize = 4;

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

/// One step of a watch's opening phase: a result, or a keepalive while none is ready.
#[derive(Debug, PartialEq)]
enum Opening<T> {
    Ready(T),
    Keepalive,
}

/* === Traits === */

/* === Implementations === */

impl Resources {
    fn produce<T: MemorySize + Portable + Send + Sync + 'static>(
        &self, build: impl FnOnce() -> Result<T, Status>,
    ) -> Result<Response<T>, Status> {
        let output = build()?;
        ensure_message_budget(&output, self.maximum, "plot response")?;
        Ok(output.into())
    }
}

impl Resources {
    // `None` when the plot's points do not fit one message: its stored request
    // now asks for as many as fit, and the caller reads it again.
    fn frame(
        &self, requests: &mut Vec<dokime_runtime::runtime::query::PlotPayloadRequest>,
        current: &dokime_runtime::runtime::query::PlotPayloadRequest, kind: dokime_runtime::runtime::query::PlotPayloadStreamKind,
        value: dokime_runtime::runtime::query::PlotPayloadEnvelope, epoch: &str, selection: &str,
    ) -> Result<Option<Response<dokime_domain::model::plots::wire::PlotStreamFrame>>, Status> {
        let mut advanced = requests.clone();
        merge_plot_payload_cursor(&mut advanced, &value);
        let served = stream_frame(kind, value, epoch, selection, self.maximum)
            .and_then(|frame| self.produce(|| Ok(frame)).map_err(FrameError::Status));
        let error = match served {
            Ok(response) => {
                *requests = advanced;
                return Ok(Some(response));
            }
            Err(FrameError::TooManyPoints { maximum }) if reduce(requests, current, maximum) => return Ok(None),
            Err(FrameError::TooManyPoints { .. }) => Status::invalid_argument("plot points do not fit one message"),
            Err(FrameError::Status(error)) => error,
        };
        if error.code() == tonic::Code::InvalidArgument {
            requests.retain(|request| request.plot_id != current.plot_id);
        }
        self.produce(|| Ok(plot_error_frame(current, error.message().to_owned()))).map(Some)
    }
}

impl Resources {
    /// Reads one plot's snapshot, timing it so the first-frame cost is measurable.
    async fn query_snapshot(
        &self, current: &dokime_runtime::runtime::query::PlotPayloadRequest,
    ) -> Result<Vec<dokime_runtime::runtime::query::PlotPayloadEnvelope>, dokime_runtime::runtime::Error> {
        let started = Instant::now();
        let result = self.runtime.query_plot_payloads(vec![current.clone()], RuntimePlotQueryMode::Snapshot).await;
        tracing::debug!(
            plot_id = current.plot_id.as_deref().unwrap_or(current.stream.as_str()),
            elapsed_ms = started.elapsed().as_millis() as u64,
            ok = result.is_ok(),
            "plot snapshot query"
        );
        result
    }

    async fn snapshot(
        &self, output: &mut StreamOutput<dokime_domain::model::plots::wire::PlotStreamFrame>,
        requests: &mut Vec<dokime_runtime::runtime::query::PlotPayloadRequest>, selections: &[String], epoch: &str,
        current: &dokime_runtime::runtime::query::PlotPayloadRequest,
    ) -> Result<bool, Status> {
        let result = self.query_snapshot(current).await;
        self.deliver_snapshot(output, requests, selections, epoch, current, result).await
    }

    /// Sends one plot's snapshot, or its error frame; `false` when the read was refused.
    async fn deliver_snapshot(
        &self, output: &mut StreamOutput<dokime_domain::model::plots::wire::PlotStreamFrame>,
        requests: &mut Vec<dokime_runtime::runtime::query::PlotPayloadRequest>, selections: &[String], epoch: &str,
        current: &dokime_runtime::runtime::query::PlotPayloadRequest,
        result: Result<Vec<dokime_runtime::runtime::query::PlotPayloadEnvelope>, dokime_runtime::runtime::Error>,
    ) -> Result<bool, Status> {
        let native = match result {
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
            let kind = dokime_runtime::runtime::query::PlotPayloadStreamKind::Snapshot;
            let mut response = self.frame(requests, current, kind, value, epoch, &selection)?;
            if response.is_none() {
                response = self.reduced_snapshot(requests, current, epoch, &selection).await?;
            }
            output.send_owned(response.ok_or_else(|| Status::internal("reduced plot did not settle"))?).await.map_err(failure)?;
        }
        Ok(true)
    }

    // Reads a plot again under the point budget its last frame exceeded, once.
    async fn reduced_snapshot(
        &self, requests: &mut Vec<dokime_runtime::runtime::query::PlotPayloadRequest>,
        original: &dokime_runtime::runtime::query::PlotPayloadRequest, epoch: &str, selection: &str,
    ) -> Result<Option<Response<dokime_domain::model::plots::wire::PlotStreamFrame>>, Status> {
        let Some(current) = requests.iter().find(|request| request.plot_id == original.plot_id).cloned() else {
            return Ok(None);
        };
        let kind = dokime_runtime::runtime::query::PlotPayloadStreamKind::Snapshot;
        let native = match self.query_snapshot(&current).await {
            Ok(native) => native,
            Err(error) => return self.produce(|| Ok(plot_error_frame(&current, error.to_string()))).map(Some),
        };
        let Some(mut value) = native.into_iter().next() else {
            return Ok(None);
        };
        value.warnings.push(reduction_warning(original, &current));
        match self.frame(requests, &current, kind, value, epoch, selection)? {
            Some(response) => Ok(Some(response)),
            None => self.produce(|| Ok(plot_error_frame(&current, "plot points do not fit one message".to_owned()))).map(Some),
        }
    }

    /// Tells the client each of `waiting` is subscribed and still reading.
    async fn send_pending(
        &self, output: &mut StreamOutput<dokime_domain::model::plots::wire::PlotStreamFrame>,
        waiting: &[dokime_runtime::runtime::query::PlotPayloadRequest],
    ) -> Result<(), Status> {
        for current in waiting {
            let response = self.produce(|| Ok(plot_pending_frame(current)))?;
            output.send_owned(response).await.map_err(failure)?;
        }
        Ok(())
    }
}

impl Operation for Analyses {
    type Declaration = contract::ListAnalyses;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();
    async fn initialize<'ctx>(resources: Arc<Resources>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self, _: Self::Context<'a>, request: &mut contract::ListAnalyses, _: &mut (),
    ) -> Result<Response<dokime_domain::model::plots::wire::ListAnalysesReply>, Status> {
        super::value::text(&request.after_id)?;
        let limit = if request.limit == 0 { 64 } else { request.limit as usize };
        if limit > 128 {
            return Err(Status::resource_exhausted("analysis page exceeds 128 definitions"));
        }
        let definitions = self.resources.runtime.analysis_definitions().await.map_err(|error| Status::unavailable(error.to_string()))?;
        let mut page = definitions.iter().filter(|definition| definition.id > request.after_id).take(limit + 1).collect::<Vec<_>>();
        let next_after_id = (page.len() > limit).then(|| page[limit - 1].id.clone());
        page.truncate(limit);
        self.resources.produce(|| {
            Ok(dokime_domain::model::plots::wire::ListAnalysesReply {
                definitions: page.into_iter().map(analysis::definition).collect::<Result<_, _>>()?,
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
    async fn initialize<'ctx>(resources: Arc<Resources>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self, _: Self::Context<'a>, request: &mut contract::QuerySnapshots, _: &mut (),
    ) -> Result<Response<dokime_domain::model::plots::wire::PlotSnapshotsReply>, Status> {
        let epoch = self.resources.runtime.process_epoch().to_string();
        let (requests, selections) = decode_plot_requests(std::mem::take(&mut request.requests), &epoch)?;
        let selection_requests = requests.clone();
        let native = self
            .resources
            .runtime
            .query_plot_payloads(requests, RuntimePlotQueryMode::Snapshot)
            .await
            .map_err(|error| Status::unavailable(format!("plot query unavailable: {error}")))?;
        let mut envelopes = Vec::with_capacity(native.len());
        for value in native {
            let selection = selection_for_plot(&selection_requests, &selections, &value.plot_id)
                .ok_or_else(|| Status::internal("plot response does not match a request"))?
                .to_owned();
            envelopes.push(self.fitted_envelope(&selection_requests, value, &epoch, &selection).await?);
        }
        self.resources.produce(|| Ok(dokime_domain::model::plots::wire::PlotSnapshotsReply { envelopes }))
    }
}

impl Snapshots {
    // One plot's envelope, read again once under the point budget it exceeded.
    async fn fitted_envelope(
        &self, requests: &[dokime_runtime::runtime::query::PlotPayloadRequest], value: dokime_runtime::runtime::query::PlotPayloadEnvelope,
        epoch: &str, selection: &str,
    ) -> Result<dokime_domain::model::plots::wire::PlotEnvelope, Status> {
        let plot_id = value.plot_id.clone();
        let maximum = match envelope(value, epoch, selection, self.resources.maximum) {
            Ok(envelope) => return Ok(envelope),
            Err(FrameError::Status(error)) => return Err(error),
            Err(FrameError::TooManyPoints { maximum }) => maximum,
        };
        let original =
            matching_plot_payload_request(requests, &plot_id).ok_or_else(|| Status::internal("plot response does not match a request"))?;
        let mut current = original.clone();
        current.max_points = Some(maximum);
        let native = self
            .resources
            .query_snapshot(&current)
            .await
            .map_err(|error| Status::unavailable(format!("plot query unavailable: {error}")))?;
        let mut value = native.into_iter().next().ok_or_else(|| Status::internal("reduced plot returned no envelope"))?;
        value.warnings.push(reduction_warning(original, &current));
        envelope(value, epoch, selection, self.resources.maximum).map_err(|error| match error {
            FrameError::Status(error) => error,
            FrameError::TooManyPoints { .. } => Status::invalid_argument(format!("plot {plot_id} points do not fit one message")),
        })
    }
}

impl Operation for Watch {
    type Declaration = contract::Watch;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = StreamOutput<dokime_domain::model::plots::wire::PlotStreamFrame>;
    async fn initialize<'ctx>(resources: Arc<Resources>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self, _: Self::Context<'a>, request: &mut contract::Watch, output: &mut Self::Endpoints,
    ) -> Result<Response<()>, Status> {
        let runtime = &self.resources.runtime;
        let epoch = runtime.process_epoch().to_string();
        let (mut requests, selections) = decode_plot_requests(std::mem::take(&mut request.requests), &epoch)?;
        // Acknowledge At Once: Every Plot Hears It Is Subscribed Before Any Read Starts.
        let opened = Instant::now();
        let mut waiting = requests.iter().filter(|request| request.cursor.is_none()).cloned().collect::<Vec<_>>();
        self.resources.send_pending(output, &requests).await?;

        // Subscribe Before Reading So A Racing Commit Advances The Same Native Watches.
        let mut states = runtime.watch_state();
        let mut watches = Vec::new();
        let subscriptions = stream::iter(requests.clone())
            .map(|current| async move {
                let watched = runtime.watch_plot_telemetry(std::slice::from_ref(&current)).await;
                (current, watched)
            })
            .buffered(PLOT_INITIAL_CONCURRENCY);
        let mut subscribing = std::pin::pin!(with_keepalive(subscriptions, PLOT_KEEPALIVE));
        while let Some(event) = subscribing.next().await {
            match event {
                Opening::Keepalive => self.resources.send_pending(output, &waiting).await?,
                Opening::Ready((_, Ok(current_watches))) => watches.extend(current_watches),
                Opening::Ready((current, Err(error))) => {
                    let response = self.resources.produce(|| Ok(plot_error_frame(&current, error.to_string())))?;
                    output.send_owned(response).await.map_err(failure)?;
                }
            }
        }
        let mut telemetry_watches = stream::select_all(watches);

        // Read Every Initial Snapshot Concurrently, Keeping The Unserved Ones Alive.
        // A refused read is sent as its plot's error at once and retried with
        // backoff even while no new evidence arrives, so a contended plot
        // neither waits silently nor stays failed once capacity frees.
        let mut retry_delay = PLOT_RETRY_FIRST;
        let mut unserved = Vec::new();
        let resources = &self.resources;
        let snapshots = stream::iter(waiting.clone())
            .map(|current| async move {
                let result = resources.query_snapshot(&current).await;
                (current, result)
            })
            .buffer_unordered(PLOT_INITIAL_CONCURRENCY);
        let mut reading = std::pin::pin!(with_keepalive(snapshots, PLOT_KEEPALIVE));
        while let Some(event) = reading.next().await {
            match event {
                Opening::Keepalive => self.resources.send_pending(output, &waiting).await?,
                Opening::Ready((current, result)) => {
                    waiting.retain(|request| request.plot_id != current.plot_id);
                    if !self.resources.deliver_snapshot(output, &mut requests, &selections, &epoch, &current, result).await? {
                        unserved.push(current);
                    }
                }
            }
        }
        tracing::debug!(plots = requests.len(), elapsed_ms = opened.elapsed().as_millis() as u64, "plot watch opened");
        let mut last_flush = Instant::now() - PLOT_STREAM_THROTTLE;
        let mut next_flush = None::<Instant>;
        if !unserved.is_empty() {
            next_flush = Some(Instant::now() + retry_delay);
            retry_delay = (retry_delay * 2).min(PLOT_RETRY_MAX);
        }
        // Only a plot whose inputs advanced is read again, and every such plot
        // is read in one batch per tick.
        let keys = requests.iter().map(|request| (plot_key(request), PlotWatchKeys::for_request(request))).collect::<Vec<_>>();
        let mut dirty = BTreeSet::<String>::new();
        loop {
            tokio::select! {
                biased;
                changed = states.next() => {
                    if changed.is_none() {
                        return Ok(().into());
                    }
                    dirty.extend(keys.iter().filter(|(_, keys)| keys.follows_state()).map(|(key, _)| key.clone()));
                }
                advanced = telemetry_watches.next(), if !telemetry_watches.is_empty() => {
                    match advanced {
                        Some((_, Some(stream))) => dirty.extend(keys.iter().filter(|(_, keys)| keys.reads_stream(&stream)).map(|(key, _)| key.clone())),
                        Some((_, None)) => dirty.extend(keys.iter().map(|(key, _)| key.clone())),
                        None => {}
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
                        .filter(|request| dirty.contains(&plot_key(request)))
                        .filter(|request| !unserved.iter().any(|pending| pending.plot_id == request.plot_id))
                        .cloned()
                        .collect::<Vec<_>>();
                    if !served.is_empty() {
                        for request in &served {
                            dirty.remove(&plot_key(request));
                        }
                        tracing::debug!(plots = served.len(), plot_ids = %served.iter().map(plot_key).collect::<Vec<_>>().join(","), "plot stream query");
                        match runtime.query_plot_stream_batches(&served).await {
                            Ok(batches) => {
                                for batch in batches {
                                    let mode = batch.mode;
                                    let value = batch.envelope;
                                    let Some(current) = matching_plot_payload_request(&requests, &value.plot_id).cloned() else { continue };
                                    if !plot_payload_stream_envelope_visible(mode, &value, &current) {
                                        continue;
                                    }
                                    let selection = selection_for_plot(&requests, &selections, &value.plot_id)
                                        .ok_or_else(|| Status::internal("plot response does not match a request"))?
                                        .to_owned();
                                    let kind = plot_payload_stream_kind(mode);
                                    match self.resources.frame(&mut requests, &current, kind, value, &epoch, &selection)? {
                                        Some(response) => output.send_owned(response).await.map_err(failure)?,
                                        // Read again on the next tick under the budget that fits.
                                        None => {
                                            dirty.insert(plot_key(&current));
                                            refused = true;
                                        }
                                    }
                                }
                            }
                            Err(error) => {
                                // The batch failed as one: every plot in it hears why and is read again.
                                refused = true;
                                for current in &served {
                                    dirty.insert(plot_key(current));
                                    let response = self.resources.produce(|| Ok(plot_error_frame(current, error.to_string())))?;
                                    output.send_owned(response).await.map_err(failure)?;
                                }
                            }
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
            if next_flush.is_none() && !dirty.is_empty() {
                next_flush = Some(last_flush + PLOT_STREAM_THROTTLE);
            }
        }
    }
}

/* === Public Functions === */

pub(super) fn register(runtime: &RuntimeHandle, maximum: usize) -> Result<(), super::Error> {
    let resources = Arc::new(Resources { runtime: runtime.clone(), maximum });
    runtime.operations().register_wire_operation::<Analyses>("dokime", dokime_runtime::app::Lanes::Open, resources.clone())?;
    runtime.operations().register_wire_operation::<Snapshots>("dokime", dokime_runtime::app::Lanes::Open, resources.clone())?;
    runtime.operations().register_wire_operation::<Watch>("dokime", dokime_runtime::app::Lanes::Open, resources)?;
    Ok(())
}

/* === Private Functions === */

// Lower a plot's stored point budget to what fits; `false` when it already asks for no more.
fn reduce(
    requests: &mut [dokime_runtime::runtime::query::PlotPayloadRequest], current: &dokime_runtime::runtime::query::PlotPayloadRequest,
    maximum: usize,
) -> bool {
    let Some(request) = requests.iter_mut().find(|request| request.plot_id == current.plot_id) else {
        return false;
    };
    if request.max_points.is_some_and(|points| points != 0 && points <= maximum) {
        return false;
    }
    request.max_points = Some(maximum);
    true
}

fn reduction_warning(
    original: &dokime_runtime::runtime::query::PlotPayloadRequest, reduced: &dokime_runtime::runtime::query::PlotPayloadRequest,
) -> String {
    let requested = match original.max_points {
        Some(0) => "every sample".to_owned(),
        Some(points) => format!("{points} points"),
        None => "the default points".to_owned(),
    };
    format!("{requested} do not fit one message; showing at most {} points", reduced.max_points.unwrap_or_default())
}

/// The plot a request names, the key its frames and dirty marks share.
fn plot_key(request: &dokime_runtime::runtime::query::PlotPayloadRequest) -> String {
    request.plot_id.clone().unwrap_or_else(|| request.stream.clone())
}

/// Interleaves `ready` with a keepalive every `every` that passes without a result; ends with `ready`.
fn with_keepalive<S: Stream + Unpin>(ready: S, every: Duration) -> impl Stream<Item = Opening<S::Item>> {
    stream::unfold((ready, Instant::now() + every), move |(mut ready, deadline)| async move {
        tokio::select! {
            biased;
            item = ready.next() => item.map(|item| (Opening::Ready(item), (ready, Instant::now() + every))),
            () = tokio::time::sleep_until(deadline) => Some((Opening::Keepalive, (ready, Instant::now() + every))),
        }
    })
}

/// A plot that is subscribed and still reading: an unspecified frame naming the plot, with nothing else.
fn plot_pending_frame(request: &dokime_runtime::runtime::query::PlotPayloadRequest) -> dokime_domain::model::plots::wire::PlotStreamFrame {
    dokime_domain::model::plots::wire::PlotStreamFrame {
        kind: dokime_domain::model::plots::wire::PlotStreamKind::Unspecified,
        envelope: Some(dokime_domain::model::plots::wire::PlotEnvelope {
            plot_id: request.plot_id.clone().unwrap_or_else(|| request.stream.clone()),
            revision: 0,
            cursor: None,
            payload: None,
            diagnostics: None,
            warnings: Vec::new(),
        }),
    }
}

fn plot_error_frame(
    request: &dokime_runtime::runtime::query::PlotPayloadRequest, message: String,
) -> dokime_domain::model::plots::wire::PlotStreamFrame {
    dokime_domain::model::plots::wire::PlotStreamFrame {
        kind: dokime_domain::model::plots::wire::PlotStreamKind::Error,
        envelope: Some(dokime_domain::model::plots::wire::PlotEnvelope {
            plot_id: request.plot_id.clone().unwrap_or_else(|| request.stream.clone()),
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

#[cfg(test)]
mod tests {
    use super::*;

    // A slow first read is preceded by a keepalive every interval, and the
    // stream ends with its last result rather than ticking on.
    #[tokio::test(start_paused = true)]
    async fn keepalive_ordering() {
        let every = Duration::from_secs(3);
        let slow = Box::pin(stream::once(async {
            tokio::time::sleep(Duration::from_secs(7)).await;
            "first"
        }));
        let events = with_keepalive(slow, every).collect::<Vec<_>>().await;
        assert_eq!(events, vec![Opening::Keepalive, Opening::Keepalive, Opening::Ready("first")]);

        let ready = with_keepalive(stream::iter(["a", "b"]), every).collect::<Vec<_>>().await;
        assert_eq!(ready, vec![Opening::Ready("a"), Opening::Ready("b")], "results ready at once need no keepalive");
    }

    // A document hands the live view the histogram as declared, value on x,
    // and the watch decodes that back unmapped: the query maps it once, as
    // the report does, instead of mapping an already mapped request to nothing.
    #[test]
    fn histogram_definition_declared() {
        use dokime_runtime::document::{DocumentSelection, Plot, plot_descriptor_for_definition, plot_input_for_definition};
        let mut plot = Plot::histogram("Voltage Distribution");
        plot.id("obc.startup.voltage_distribution");
        plot.x(dokime_domain::model::expressions::telemetry("obc.startup", "voltage_v"));
        let input = plot_input_for_definition(&plot, &[], 0, &DocumentSelection::default());
        let descriptor = plot_descriptor_for_definition(input, plot.metadata.clone());

        let definition = wire::document_plot_definition(&descriptor).unwrap();
        let (decoded, _) = decode_plot_requests(vec![definition.request.unwrap()], "epoch").unwrap();
        assert_eq!(decoded[0].x_expression, descriptor.input.x_expression, "the declared x reaches the live query");
        assert!(decoded[0].y_expressions.is_empty() && decoded[0].y_keys.is_empty(), "nothing was mapped before the query");
    }

    #[test]
    fn pending_frame_names_the_plot() {
        let request =
            dokime_runtime::runtime::query::PlotPayloadRequest { plot_id: Some("power.voltage".to_owned()), ..Default::default() };
        let frame = plot_pending_frame(&request);
        assert_eq!(frame.kind, dokime_domain::model::plots::wire::PlotStreamKind::Unspecified);
        let envelope = frame.envelope.unwrap();
        assert_eq!(envelope.plot_id, "power.voltage");
        assert!(envelope.payload.is_none() && envelope.warnings.is_empty());
    }
}
