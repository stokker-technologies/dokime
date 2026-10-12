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
pub(super) use cursor::record_scope;
pub(super) use wire::{data_expression_wire, document_plot_definition, plot_filter_wire, values};

/* === Imports === */
// Standard Library Imports
use std::{collections::VecDeque, pin::Pin, time::Duration};
// Third Party Imports
use futures_util::{Stream, StreamExt, stream};
use tokio::{sync::watch, time::Instant};
use tonic::{Request, Response, Status};
// Local Imports
use cursor::{decode_plot_requests, selection_for_plot};
use dokime_app::{
    RuntimeHandle,
    runtime::query::{
        PlotPayloadRequest, PlotPayloadStreamKind, RuntimePlotQueryMode,
        matching_plot_payload_request, merge_plot_payload_cursor,
        plot_payload_stream_envelope_visible, plot_payload_stream_kind,
    },
};
use dokime_protocol::v1::{self, plots_server::Plots};
use wire::{ensure_message_budget, envelope, stream_frame};

/* === Constants === */
const PLOT_STREAM_THROTTLE: Duration = Duration::from_millis(100);

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub(super) struct Service {
    pub(super) runtime: RuntimeHandle,
    pub(super) stopping: watch::Receiver<bool>,
    pub(super) max_message_bytes: usize,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
#[tonic::async_trait]
impl Plots for Service {
    type WatchStream =
        Pin<Box<dyn Stream<Item = Result<v1::PlotStreamFrame, Status>> + Send + 'static>>;

    async fn list_analyses(
        &self,
        request: Request<v1::ListAnalysesRequest>,
    ) -> Result<Response<v1::ListAnalysesReply>, Status> {
        let request = request.into_inner();
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
        let definitions = page
            .into_iter()
            .map(analysis::definition)
            .collect::<Result<_, _>>()?;
        let reply = v1::ListAnalysesReply {
            definitions,
            next_after_id,
        };
        ensure_message_budget(&reply, self.max_message_bytes, "analysis definitions")?;
        Ok(Response::new(reply))
    }

    async fn query_snapshots(
        &self,
        request: Request<v1::PlotSnapshotsRequest>,
    ) -> Result<Response<v1::PlotSnapshotsReply>, Status> {
        let epoch = self.runtime.process_epoch().to_string();
        let (requests, selections) = decode_plot_requests(request.into_inner().requests, &epoch)?;
        let selection_requests = requests.clone();
        let envelopes = self
            .runtime
            .query_plot_payloads(requests, RuntimePlotQueryMode::Snapshot)
            .await
            .map_err(|error| Status::unavailable(format!("plot query unavailable: {error}")))?
            .into_iter()
            .map(|envelope_value| {
                let selection =
                    selection_for_plot(&selection_requests, &selections, &envelope_value.plot_id)
                        .ok_or_else(|| Status::internal("plot response does not match a request"))?;
                envelope(envelope_value, &epoch, selection)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let reply = v1::PlotSnapshotsReply { envelopes };
        ensure_message_budget(&reply, self.max_message_bytes, "plot snapshot response")?;
        Ok(Response::new(reply))
    }

    async fn watch(
        &self,
        request: Request<v1::PlotWatchRequest>,
    ) -> Result<Response<Self::WatchStream>, Status> {
        if *self.stopping.borrow() {
            return Err(Status::unavailable("listener is stopping"));
        }

        let epoch = self.runtime.process_epoch().to_string();
        let (mut requests, selections) =
            decode_plot_requests(request.into_inner().requests, &epoch)?;
        // Subscribe before the initial read. An evidence commit that races the
        // snapshot is therefore observed by the watch and causes a subsequent
        // re-read, just as the legacy live plot route did.
        let states = self.runtime.watch_state();
        let watches = self
            .runtime
            .watch_plot_telemetry(&requests)
            .await
            .map_err(|error| Status::unavailable(format!("plot watch unavailable: {error}")))?;
        let telemetry_watches = stream::select_all(watches);

        let mut pending = VecDeque::new();
        queue_initial_snapshots(
            &self.runtime,
            &mut requests,
            &selections,
            &epoch,
            &mut pending,
            self.max_message_bytes,
        )
        .await?;

        let runtime = self.runtime.clone();
        let max_message_bytes = self.max_message_bytes;
        let stopping = self.stopping.clone();
        let stream = stream::try_unfold(
            (
                runtime,
                requests,
                selections,
                epoch,
                max_message_bytes,
                states,
                telemetry_watches,
                stopping,
                pending,
                false,
                Instant::now() - PLOT_STREAM_THROTTLE,
                None::<Instant>,
            ),
            move |(
                runtime,
                mut requests,
                selections,
                epoch,
                max_message_bytes,
                mut states,
                mut telemetry_watches,
                mut stopping,
                mut pending,
                mut dirty,
                mut last_flush,
                mut next_flush,
            )| async move {
                loop {
                    if let Some(frame) = pending.pop_front() {
                        return Ok(Some((
                            frame,
                            (
                                runtime,
                                requests,
                                selections,
                                epoch,
                                max_message_bytes,
                                states,
                                telemetry_watches,
                                stopping,
                                pending,
                                dirty,
                                last_flush,
                                next_flush,
                            ),
                        )));
                    }
                    if *stopping.borrow() {
                        return Ok(None);
                    }

                    tokio::select! {
                        biased;
                        _ = stopping.changed() => return Ok(None),
                        changed = states.next() => {
                            if changed.is_none() {
                                return Ok(None);
                            }
                            dirty = true;
                        }
                        advanced = telemetry_watches.next(), if !telemetry_watches.is_empty() => {
                            if advanced.is_none() {
                                continue;
                            }
                            dirty = true;
                        }
                        _ = tokio::time::sleep_until(next_flush.unwrap_or_else(Instant::now)), if next_flush.is_some() => {
                            dirty = false;
                            next_flush = None;
                            last_flush = Instant::now();
                            let batches = match runtime.query_plot_stream_batches(&requests).await {
                                Ok(batches) => batches,
                                Err(error) => return Err(Status::unavailable(format!("plot watch unavailable: {error}"))),
                            };
                            for batch in batches {
                                let mode = batch.mode;
                                let envelope = batch.envelope;
                                let Some(current) = matching_plot_payload_request(&requests, &envelope.plot_id) else {
                                    continue;
                                };
                                if !plot_payload_stream_envelope_visible(mode, &envelope, current) {
                                    continue;
                                }
                                let Some(selection) = selection_for_plot(
                                    &requests,
                                    &selections,
                                    &envelope.plot_id,
                                ).map(str::to_owned) else {
                                    return Err(Status::internal("plot response does not match a request"));
                                };
                                merge_plot_payload_cursor(&mut requests, &envelope);
                                let frame = match stream_frame(
                                    plot_payload_stream_kind(mode),
                                    envelope,
                                    &epoch,
                                    &selection,
                                ) {
                                    Ok(frame) => frame,
                                    Err(error) => return Err(error),
                                };
                                ensure_message_budget(
                                    &frame,
                                    max_message_bytes,
                                    "plot stream frame",
                                )?;
                                pending.push_back(frame);
                            }
                        }
                    }

                    // Arm the timer only after a watch advance. With no dirty
                    // state, this stream has no periodic wake and no polling.
                    if dirty && next_flush.is_none() {
                        next_flush = Some(last_flush + PLOT_STREAM_THROTTLE);
                    }
                }
            },
        );
        Ok(Response::new(Box::pin(stream)))
    }
}

/* === Public Functions === */

/* === Private Functions === */
async fn queue_initial_snapshots(
    runtime: &RuntimeHandle,
    requests: &mut [PlotPayloadRequest],
    selections: &[String],
    epoch: &str,
    pending: &mut VecDeque<v1::PlotStreamFrame>,
    max_message_bytes: usize,
) -> Result<(), Status> {
    if !requests.iter().any(|request| request.cursor.is_none()) {
        return Ok(());
    }
    let snapshot_requests = requests
        .iter()
        .filter(|request| request.cursor.is_none())
        .cloned()
        .collect::<Vec<_>>();
    let snapshots = runtime
        .query_plot_payloads(snapshot_requests, RuntimePlotQueryMode::Snapshot)
        .await
        .map_err(|error| Status::unavailable(format!("plot snapshot unavailable: {error}")))?;
    for envelope in snapshots {
        let selection = selection_for_plot(requests, selections, &envelope.plot_id)
            .ok_or_else(|| Status::internal("plot response does not match a request"))?;
        merge_plot_payload_cursor(requests, &envelope);
        let frame = stream_frame(PlotPayloadStreamKind::Snapshot, envelope, epoch, selection)?;
        ensure_message_budget(&frame, max_message_bytes, "plot stream frame")?;
        pending.push_back(frame);
    }
    Ok(())
}

/* === Tests === */
