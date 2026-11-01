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
use dokime::harmos::domain::Portable;
use dokime::harmos::operations::StreamOutput;
use dokime::harmos::{Context, Response};
use futures_util::{Stream, StreamExt, stream};
use tokio::time::Instant;
use tonic::Status;

// Local Imports
use cursor::{decode_plot_requests, selection_for_plot};
use dokime::app::LiveRuntime;
use dokime::harmos::storage::{ReadBatch, StorageCursor};
use dokime::operations::hosting::{Host, Serve, Served};
use dokime::operations::plots as contract;
use dokime::runtime::query::{
    PlotPayloadEnvelope, PlotPayloadRequest, PlotPayloadStreamKind, PlotWatchKeys, RuntimePlotQueryMode, matching_plot_payload_request,
    reduce_plot_payload,
};
use wire::{FrameError, ensure_message_budget, envelope, stream_frame};

/* === Constants === */

const SERVER_PLOT_INITIAL_CONCURRENCY: u64 = 4;
const SERVER_PLOT_KEEPALIVE_MS: u64 = 3_000;
const SERVER_PLOT_RETRY_INITIAL_MS: u64 = 250;
const SERVER_PLOT_RETRY_MAX_MS: u64 = 5_000;
const SERVER_PLOT_THROTTLE_MS: u64 = 100;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct Resources {
    runtime: LiveRuntime,
    maximum: usize,
}

struct FrameContext<'a> {
    epoch: &'a str,
    selection: &'a str,
    kind: PlotPayloadStreamKind,
}

/* === Enums === */

// One step of a watch's opening phase: a result, or a keepalive while none is ready.
#[derive(Debug, PartialEq)]
enum Opening<T> {
    Ready(T),
    Keepalive,
}

/* === Traits === */

/* === Implementations === */

impl Resources {
    fn produce<T: Portable + Send + Sync + 'static>(&self, build: impl FnOnce() -> Result<T, Status>) -> Result<Response<T>, Status> {
        let output = build()?;
        ensure_message_budget(&output, self.maximum, "plot response")?;
        Ok(output.into())
    }

    fn frame(
        &self, requests: &mut Vec<PlotPayloadRequest>, current: &PlotPayloadRequest, value: ReadBatch<PlotPayloadEnvelope>,
        frame: &FrameContext<'_>,
    ) -> Result<Response<dokime::domain::model::plots::wire::PlotStreamFrame>, Status> {
        let served = stream_frame(frame.kind.clone(), ReadBatch::new((*value).clone()), frame.epoch, frame.selection, self.maximum)
            .and_then(|frame| {
                ensure_message_budget(&*frame, self.maximum, "plot response")?;
                Ok(Response::from_owner(frame.owner()))
            });
        let error = match served {
            Ok(response) => return Ok(response),
            Err(FrameError::TooManyPoints { maximum }) if reduce(requests, current, maximum) => {
                let mut reduced = current.clone();
                reduced.max_points = Some(maximum);
                let value = reduce_plot_payload(value, &reduced, maximum).map_err(|error| Status::internal(error.to_string()))?;
                let value = add_reduction_warning(value, current, &reduced)?;
                let frame =
                    stream_frame(frame.kind.clone(), value, frame.epoch, frame.selection, self.maximum).map_err(|error| match error {
                        FrameError::Status(error) => error,
                        FrameError::TooManyPoints { .. } => Status::invalid_argument("plot points do not fit one message"),
                    })?;
                return Ok(Response::from_owner(frame.owner()));
            }
            Err(FrameError::TooManyPoints { .. }) => Status::invalid_argument("plot points do not fit one message"),
            Err(FrameError::Status(error)) => error,
        };
        if error.code() == tonic::Code::InvalidArgument {
            requests.retain(|request| plot_key(request) != plot_key(current));
        }
        self.produce(|| Ok(plot_error_frame(current, error.message().to_owned(), frame.epoch, frame.selection)))
    }

    async fn query_snapshot(
        &self, current: &PlotPayloadRequest, mode: RuntimePlotQueryMode, at: &StorageCursor,
    ) -> Result<Vec<ReadBatch<PlotPayloadEnvelope>>, dokime::runtime::Error> {
        let started = Instant::now();
        let result = dokime::app::live_runtime(&self.runtime)
            .await
            .map_err(|error| dokime::runtime::Error::Operation(error.to_string()))?
            .query_plot_payloads_at(vec![current.clone()], mode, at)
            .await;
        tracing::debug!(
            plot_id = current.plot_id.as_deref().unwrap_or(current.stream.as_str()),
            elapsed_ms = started.elapsed().as_millis() as u64,
            ok = result.is_ok(),
            "plot snapshot query"
        );
        result
    }

    async fn deliver_snapshot(
        &self, output: &StreamOutput<dokime::domain::model::plots::wire::PlotStreamFrame>, requests: &mut Vec<PlotPayloadRequest>,
        current: &PlotPayloadRequest, result: Result<Vec<ReadBatch<PlotPayloadEnvelope>>, dokime::runtime::Error>,
        frame: &FrameContext<'_>,
    ) -> Result<bool, Status> {
        let native = match result {
            Ok(native) => native,
            Err(error) => {
                let response = self.produce(|| Ok(plot_error_frame(current, error.to_string(), frame.epoch, frame.selection)))?;
                output.send_owned(response).await.map_err(crate::grpc::chunked::output_failure)?;
                return Ok(false);
            }
        };
        for value in native {
            let response = self.frame(requests, current, value, frame)?;
            output.send_owned(response).await.map_err(crate::grpc::chunked::output_failure)?;
        }
        Ok(true)
    }

    async fn send_pending(
        &self, output: &StreamOutput<dokime::domain::model::plots::wire::PlotStreamFrame>, waiting: &[PlotPayloadRequest],
        selections: &[String], original: &[PlotPayloadRequest], epoch: &str,
    ) -> Result<(), Status> {
        for current in waiting {
            let selection = selection_for_plot(original, selections, &plot_key(current))
                .ok_or_else(|| Status::internal("plot selection is missing"))?;
            let response = self.produce(|| Ok(plot_pending_frame(current, epoch, selection)))?;
            output.send_owned(response).await.map_err(crate::grpc::chunked::output_failure)?;
        }
        Ok(())
    }
}

impl Serve<contract::ListAnalyses> for Resources {
    fn serve<'a>(
        &'a self, request: contract::ListAnalyses, _: Context<'a, contract::ListAnalyses>,
    ) -> Served<'a, dokime::domain::model::plots::wire::ListAnalysesReply> {
        Box::pin(async move {
            let limit = dokime::domain::model::execution::wire_cursor::page_size(request.limit, &request.after_id)
                .map_err(super::value::canonical::cursor_failure)?;
            let definitions = dokime::app::live_runtime(&self.runtime)
                .await?
                .analysis_definitions()
                .await
                .map_err(|error| Status::unavailable(error.to_string()))?;
            let mut page = definitions.iter().filter(|definition| definition.id > request.after_id).take(limit + 1).collect::<Vec<_>>();
            let next_after_id = (page.len() > limit).then(|| page[limit - 1].id.clone());
            page.truncate(limit);
            self.produce(|| {
                Ok(dokime::domain::model::plots::wire::ListAnalysesReply {
                    definitions: page.into_iter().map(analysis::definition).collect::<Result<_, _>>()?,
                    next_after_id,
                })
            })
        })
    }
}

impl Serve<contract::QuerySnapshots> for Resources {
    fn serve<'a>(
        &'a self, mut request: contract::QuerySnapshots, _: Context<'a, contract::QuerySnapshots>,
    ) -> Served<'a, dokime::domain::model::plots::wire::PlotSnapshotsReply> {
        Box::pin(async move {
            let runtime = &dokime::app::live_runtime(&self.runtime).await?;
            let epoch = runtime.process_epoch().as_str().to_owned();
            let (requests, selections) = decode_plot_requests(std::mem::take(&mut request.requests))?;
            let at = runtime.storage().cursor_saved();
            let native = runtime
                .query_plot_payloads_at(requests.clone(), RuntimePlotQueryMode::Snapshot, &at)
                .await
                .map_err(|error| Status::unavailable(format!("plot query unavailable: {error}")))?;
            let count = native.len();
            let mut reply: Option<ReadBatch<dokime::domain::model::plots::wire::PlotSnapshotsReply>> = None;
            for value in native {
                let selection = selection_for_plot(&requests, &selections, &value.plot_id)
                    .ok_or_else(|| Status::internal("plot response does not match a request"))?;
                let envelope = self.fitted_envelope(&requests, value, &epoch, selection)?;
                reply = Some(match reply {
                    None => envelope
                        .try_map(|envelope| {
                            let mut envelopes = Vec::new();
                            envelopes.try_reserve_exact(count).map_err(|_| plot_memory())?;
                            envelopes.push(envelope);
                            Ok::<_, dokime::harmos::Error>(dokime::domain::model::plots::wire::PlotSnapshotsReply { envelopes })
                        })
                        .map_err(failure)?,
                    Some(reply) => reply
                        .try_map_with(envelope, |mut reply, envelope| {
                            reply.envelopes.push(envelope);
                            Ok::<_, dokime::harmos::Error>(reply)
                        })
                        .map_err(failure)?,
                });
            }
            let reply = match reply {
                Some(reply) => reply,
                None => ReadBatch::new(dokime::domain::model::plots::wire::PlotSnapshotsReply { envelopes: Vec::new() }),
            };
            ensure_message_budget(&*reply, self.maximum, "plot response")?;
            Ok::<_, Status>(Response::from_owner(reply.owner()))
        })
    }
}

impl Serve<contract::Watch> for Resources {
    fn serve<'a>(&'a self, mut request: contract::Watch, ctx: Context<'a, contract::Watch>) -> Served<'a, ()> {
        Box::pin(async move {
            let call_context = ctx.call();
            let output = ctx.streams();
            let revoked = crate::grpc::access::revocation(call_context);
            crate::grpc::access::during(revoked, async {
                let runtime = &dokime::app::live_runtime(&self.runtime).await?;
                let epoch = runtime.process_epoch().as_str().to_owned();
                let (mut requests, selections) = decode_plot_requests(std::mem::take(&mut request.requests))?;
                // Live Plots Hold Their Streams; Streams Nobody Views Or Runs Stay Stopped.
                let _viewing = runtime.view_plot_streams(&requests, "plots.watch").await;
                let original = requests.clone();
                let mut waiting = requests.clone();
                self.send_pending(output, &waiting, &selections, &original, &epoch).await?;

                // The Passive Native Wait Subscribes Before Checking This Frontier,
                // So A Save Racing Any Initial Read Cannot Be Missed.
                let mut states = runtime.watch_state();
                let mut observed = runtime.storage().cursor_saved();
                let initial = observed.clone();
                let mut dirty = BTreeSet::new();
                let mut opened = BTreeSet::new();
                let resources = self;
                let at = &initial;
                let snapshots = stream::iter(waiting.clone())
                    .map(|current| async move {
                        let result = resources.query_snapshot(&current, RuntimePlotQueryMode::Snapshot, at).await;
                        (current, result)
                    })
                    .buffer_unordered(usize::try_from(SERVER_PLOT_INITIAL_CONCURRENCY).expect("validated plot concurrency"));
                let mut heartbeat = Instant::now() + super::WATCH_HEARTBEAT;
                let mut reading = std::pin::pin!(with_keepalive(snapshots, Duration::from_millis(SERVER_PLOT_KEEPALIVE_MS)));
                while let Some(event) = reading.next().await {
                    heartbeat = Instant::now() + super::WATCH_HEARTBEAT;
                    match event {
                        Opening::Keepalive => self.send_pending(output, &waiting, &selections, &original, &epoch).await?,
                        Opening::Ready((current, result)) => {
                            let key = plot_key(&current);
                            waiting.retain(|request| plot_key(request) != key);
                            let selection = selection_for_plot(&original, &selections, &key)
                                .ok_or_else(|| Status::internal("plot selection is missing"))?;
                            let frame = FrameContext {
                                epoch: &epoch,
                                selection,
                                kind: PlotPayloadStreamKind::Snapshot,
                            };
                            if resources.deliver_snapshot(output, &mut requests, &current, result, &frame).await? {
                                opened.insert(key);
                            } else {
                                dirty.insert(key);
                            }
                        }
                    }
                }
                let keys = original.iter().map(|request| (plot_key(request), PlotWatchKeys::for_request(request))).collect::<Vec<_>>();
                let throttle = Duration::from_millis(SERVER_PLOT_THROTTLE_MS);
                let mut retry_delay = Duration::from_millis(SERVER_PLOT_RETRY_INITIAL_MS);
                let mut last_flush = Instant::now() - throttle;
                let mut next_flush = (!dirty.is_empty()).then(|| Instant::now() + retry_delay);
                loop {
                    if requests.is_empty() {
                        return Ok(().into());
                    }
                    tokio::select! {
                        changed = states.next() => {
                            if changed.is_none() { return Ok(().into()); }
                            dirty.extend(keys.iter().filter(|(_, keys)| keys.follows_state()).map(|(key, _)| key.clone()));
                        }
                        _ = tokio::time::sleep_until(next_flush.unwrap_or_else(Instant::now)), if next_flush.is_some() => {
                            next_flush = None;
                            last_flush = Instant::now();
                            // Every Query And Any Reduced Retry In This Pass Uses One
                            // Durable Primary Cut; None Reads Accepted Pending Samples.
                            let at = runtime.storage().cursor_saved();
                            if at.sequence > observed.sequence {
                                let streams = runtime.plot_streams_saved_between(&observed, &at).await
                                    .map_err(|error| Status::unavailable(format!("plot storage watch unavailable: {error}")))?;
                                dirty.extend(keys.iter().filter(|(_, keys)| keys.follows_state() || streams.iter().any(|stream| keys.reads_stream(stream)))
                                    .map(|(key, _)| key.clone()));
                            }
                            observed = at.clone();
                            let served = requests.iter().filter(|request| dirty.contains(&plot_key(request))).cloned().collect::<Vec<_>>();
                            // Delivered Frames Keep The Watch Alive Until The Next Heartbeat.
                            if !served.is_empty() {
                                heartbeat = Instant::now() + super::WATCH_HEARTBEAT;
                            }
                            let mut refused = false;
                            for current in served {
                                let key = plot_key(&current);
                                dirty.remove(&key);
                                let selection = selection_for_plot(&original, &selections, &key)
                                    .ok_or_else(|| Status::internal("plot selection is missing"))?;
                                let (kind, mode) = if opened.contains(&key) {
                                    (PlotPayloadStreamKind::Replace, RuntimePlotQueryMode::Replace)
                                } else {
                                    (PlotPayloadStreamKind::Snapshot, RuntimePlotQueryMode::Snapshot)
                                };
                                let frame = FrameContext { epoch: &epoch, selection, kind };
                                let result = resources.query_snapshot(&current, mode, &at).await;
                                if resources.deliver_snapshot(output, &mut requests, &current, result, &frame).await? {
                                    opened.insert(key);
                                } else {
                                    dirty.insert(key);
                                    refused = true;
                                }
                            }
                            if refused {
                                next_flush = Some(Instant::now() + retry_delay);
                                retry_delay = (retry_delay * 2).min(Duration::from_millis(SERVER_PLOT_RETRY_MAX_MS));
                            } else { retry_delay = Duration::from_millis(SERVER_PLOT_RETRY_INITIAL_MS); }
                        }
                        advanced = runtime.storage().wait_saved_after(&observed) => {
                            let advanced = advanced.map_err(|error| Status::unavailable(format!("plot storage watch unavailable: {error}")))?;
                            let streams = runtime.plot_streams_saved_between(&observed, &advanced).await
                                .map_err(|error| Status::unavailable(format!("plot storage watch unavailable: {error}")))?;
                            dirty.extend(keys.iter().filter(|(_, keys)| keys.follows_state() || streams.iter().any(|stream| keys.reads_stream(stream)))
                                .map(|(key, _)| key.clone()));
                            observed = advanced;
                        }
                        () = tokio::time::sleep_until(heartbeat) => {
                            // A Quiet Watch Repeats Each Plot's Pending Notice, Which Leaves A Delivered Plot Unchanged.
                            heartbeat = Instant::now() + super::WATCH_HEARTBEAT;
                            self.send_pending(output, &requests, &selections, &original, &epoch).await?;
                        }
                    }
                    if next_flush.is_none() && !dirty.is_empty() {
                        next_flush = Some(last_flush + throttle);
                    }
                }
            })
            .await
        })
    }
}

impl Resources {
    fn fitted_envelope(
        &self, requests: &[PlotPayloadRequest], value: ReadBatch<PlotPayloadEnvelope>, epoch: &str, selection: &str,
    ) -> Result<ReadBatch<dokime::domain::model::plots::wire::PlotEnvelope>, Status> {
        let plot_id = value.plot_id.clone();
        let maximum = match envelope(ReadBatch::new((*value).clone()), epoch, selection, self.maximum) {
            Ok(envelope) => return Ok(envelope),
            Err(FrameError::Status(error)) => return Err(error),
            Err(FrameError::TooManyPoints { maximum }) => maximum,
        };
        let original =
            matching_plot_payload_request(requests, &plot_id).ok_or_else(|| Status::internal("plot response does not match a request"))?;
        let mut current = original.clone();
        current.max_points = Some(maximum);
        let value = reduce_plot_payload(value, &current, maximum).map_err(|error| Status::internal(error.to_string()))?;
        let value = add_reduction_warning(value, original, &current)?;
        envelope(value, epoch, selection, self.maximum).map_err(|error| match error {
            FrameError::Status(error) => error,
            FrameError::TooManyPoints { .. } => Status::invalid_argument(format!("plot {plot_id} points do not fit one message")),
        })
    }
}

/* === Public Functions === */

pub(super) fn prepare(
    setup: &mut dokime::harmos::Setup<dokime::app::Dokime>, runtime: &LiveRuntime, maximum: usize,
) -> Result<(), crate::grpc::Error> {
    let served = Arc::new(Resources { runtime: runtime.clone(), maximum });

    setup.resource::<Host<contract::ListAnalyses>>(Host::<contract::ListAnalyses>::new(served.clone()));
    setup.resource::<Host<contract::QuerySnapshots>>(Host::<contract::QuerySnapshots>::new(served.clone()));
    setup.resource::<Host<contract::Watch>>(Host::<contract::Watch>::new(served.clone()));

    Ok(())
}

/* === Private Functions === */

// Lower a plot's stored point budget to what fits; `false` when it already asks for no more.
fn reduce(
    requests: &mut [dokime::runtime::query::PlotPayloadRequest], current: &dokime::runtime::query::PlotPayloadRequest, maximum: usize,
) -> bool {
    let Some(request) = requests.iter_mut().find(|request| plot_key(request) == plot_key(current)) else {
        return false;
    };
    if request.max_points.is_some_and(|points| points != 0 && points <= maximum) {
        return false;
    }
    request.max_points = Some(maximum);
    true
}

fn add_reduction_warning(
    value: ReadBatch<PlotPayloadEnvelope>, original: &PlotPayloadRequest, reduced: &PlotPayloadRequest,
) -> Result<ReadBatch<PlotPayloadEnvelope>, Status> {
    let mut length = WarningLength(0);
    write_reduction_warning(&mut length, original, reduced).map_err(|_| plot_memory_status())?;
    value
        .try_map(|mut value| {
            let mut warning = String::new();
            warning.try_reserve_exact(length.0).map_err(|_| plot_memory())?;
            write_reduction_warning(&mut warning, original, reduced).map_err(|_| plot_memory())?;
            value.warnings.try_reserve_exact(1).map_err(|_| plot_memory())?;
            value.warnings.push(warning);
            Ok::<_, dokime::harmos::Error>(value)
        })
        .map_err(failure)
}

struct WarningLength(usize);

impl std::fmt::Write for WarningLength {
    fn write_str(&mut self, text: &str) -> std::fmt::Result {
        self.0 = self.0.checked_add(text.len()).ok_or(std::fmt::Error)?;
        Ok(())
    }
}

fn write_reduction_warning(
    output: &mut impl std::fmt::Write, original: &PlotPayloadRequest, reduced: &PlotPayloadRequest,
) -> std::fmt::Result {
    match original.max_points {
        Some(0) => output.write_str("every sample")?,
        Some(points) => write!(output, "{points} points")?,
        None => output.write_str("the default points")?,
    }
    write!(output, " do not fit one message; showing at most {} points", reduced.max_points.unwrap_or_default())
}

fn plot_memory() -> dokime::harmos::Error {
    dokime::harmos::Error::ResourceExhausted { resource: "plot response construction" }
}

fn plot_memory_status() -> Status {
    Status::resource_exhausted("plot response construction size overflows")
}

// The plot a request names, the key its frames and dirty marks share.
fn plot_key(request: &dokime::runtime::query::PlotPayloadRequest) -> String {
    request.plot_id.clone().unwrap_or_else(|| request.stream.clone())
}

// Interleaves `ready` with a keepalive every `every` that passes without a result; ends with `ready`.
fn with_keepalive<S: Stream + Unpin>(ready: S, every: Duration) -> impl Stream<Item = Opening<S::Item>> {
    stream::unfold((ready, Instant::now() + every), move |(mut ready, deadline)| async move {
        tokio::select! {
            biased;
            item = ready.next() => item.map(|item| (Opening::Ready(item), (ready, Instant::now() + every))),
            () = tokio::time::sleep_until(deadline) => Some((Opening::Keepalive, (ready, Instant::now() + every))),
        }
    })
}

// A plot that is subscribed and still reading, or a quiet watch's heartbeat: an unspecified frame naming the plot, with nothing else.
fn plot_pending_frame(
    request: &dokime::runtime::query::PlotPayloadRequest, epoch: &str, selection: &str,
) -> dokime::domain::model::plots::wire::PlotStreamFrame {
    dokime::domain::model::plots::wire::PlotStreamFrame {
        kind: dokime::domain::model::plots::wire::PlotStreamKind::Unspecified,
        envelope: Some(dokime::domain::model::plots::wire::PlotEnvelope {
            plot_id: request.plot_id.clone().unwrap_or_else(|| request.stream.clone()),
            revision: 0,
            epoch: epoch.to_owned(),
            selection: selection.to_owned(),
            payload: None,
            diagnostics: None,
            warnings: Vec::new(),
        }),
    }
}

fn plot_error_frame(
    request: &dokime::runtime::query::PlotPayloadRequest, message: String, epoch: &str, selection: &str,
) -> dokime::domain::model::plots::wire::PlotStreamFrame {
    dokime::domain::model::plots::wire::PlotStreamFrame {
        kind: dokime::domain::model::plots::wire::PlotStreamKind::Error,
        envelope: Some(dokime::domain::model::plots::wire::PlotEnvelope {
            plot_id: request.plot_id.clone().unwrap_or_else(|| request.stream.clone()),
            revision: 0,
            epoch: epoch.to_owned(),
            selection: selection.to_owned(),
            payload: None,
            diagnostics: None,
            warnings: vec![message],
        }),
    }
}

fn failure(error: dokime::harmos::Error) -> Status {
    let status = if matches!(error, dokime::harmos::Error::ResourceExhausted { .. }) {
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
        use dokime::document::{DocumentSelection, Plot, plot_descriptor_for_definition, plot_input_for_definition};
        let mut plot = Plot::histogram("Voltage Distribution");
        plot.id("obc.startup.voltage_distribution");
        plot.x(dokime::domain::model::expressions::telemetry("obc.startup", "voltage_v"));
        let input = plot_input_for_definition(&plot, &[], 0, &DocumentSelection::default());
        let descriptor = plot_descriptor_for_definition(input, plot.metadata.clone());
        let definition = wire::document_plot_definition(&descriptor).unwrap();
        let (decoded, _) = decode_plot_requests(vec![definition.request.unwrap()]).unwrap();
        assert_eq!(decoded[0].x_expression, descriptor.input.x_expression, "the declared x reaches the live query");
        assert!(decoded[0].y_expressions.is_empty() && decoded[0].y_keys.is_empty(), "nothing was mapped before the query");
    }

    #[test]
    fn pending_frame_names_the_plot() {
        let request = dokime::runtime::query::PlotPayloadRequest {
            plot_id: Some("power.voltage".to_owned()),
            ..Default::default()
        };
        let frame = plot_pending_frame(&request, "epoch", "selection");
        assert_eq!(frame.kind, dokime::domain::model::plots::wire::PlotStreamKind::Unspecified);
        let envelope = frame.envelope.unwrap();
        assert_eq!(envelope.plot_id, "power.voltage");
        assert_eq!(envelope.epoch, "epoch");
        assert_eq!(envelope.selection, "selection");
        assert!(envelope.payload.is_none() && envelope.warnings.is_empty());
    }
}
