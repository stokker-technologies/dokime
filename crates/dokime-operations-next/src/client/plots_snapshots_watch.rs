/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Plots Snapshots Watch

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::BTreeMap;

// Third Party Imports
use harmos_protocol::v1::{Failure, OperationContract};

// Local Imports
use crate::{
    Result,
    client::{ClientSettings, Context, Output},
};
use dokime_protocol::client::plots::{
    PlotStreamKind, PlotStreamUpdate, SnapshotsListRequest, SnapshotsWatchRequest, SnapshotsWatchResponse,
};
use tokio_stream::wrappers::ReceiverStream;

use dokime_storage::ExecutionState;

/* === Constants === */

pub const ID: &str = "dokime.client.v1.plots.Snapshots/Watch";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &ClientSettings) -> Result<OperationContract> {
    super::declaration::<dokime_protocol::client::plots::SnapshotsWatchRequest, dokime_protocol::client::plots::SnapshotsWatchResponse>(
        ID,
        settings.watch_timeout,
    )
}
pub async fn handle(context: &Context, request: SnapshotsWatchRequest) -> Result<Output<SnapshotsWatchResponse>> {
    super::complete(ID, context.settings.watch_timeout, async {
        let mut storage = context.host.store().subscribe();
        let mut runtime = context.host.changes();
        let mut selection = SnapshotsListRequest {
            requests: request.requests,
            at: request.at,
            page: request.page,
        };
        // Keep the Query State off the Watch's Inline Future and Poll Stack.
        let first = Box::pin(super::plots_snapshots_list::handle(context, selection.clone())).await?;
        let context = context.clone();
        let (sender, output) = tokio::sync::mpsc::channel(context.settings.stream_buffer);
        tokio::spawn(async move {
            let work =
                async {
                    let mut page = first;
                    let mut opening = true;
                    let mut observed = BTreeMap::new();
                    loop {
                        let cut = page.resolved_at.clone();
                        let token = page.next_page_token;
                        for envelope in &page.envelopes {
                            observed.insert(envelope.plot_id.clone(), envelope.revision);
                        }
                        let response = SnapshotsWatchResponse {
                            updates: page
                                .envelopes
                                .into_iter()
                                .map(|envelope| PlotStreamUpdate {
                                    kind: PlotStreamKind::Snapshot as i32,
                                    envelope: Some(envelope),
                                })
                                .collect(),
                            resolved_at: cut.clone(),
                            opening,
                            resync_required: None,
                        };
                        if sender.send(Ok(response)).await.is_err() {
                            return;
                        }
                        if !token.is_empty() {
                            selection.at = cut;
                            selection.page.get_or_insert_default().page_token = token;
                        } else {
                            if context.host.shutting_down()
                                || selection.requests.iter().all(|request| {
                                    request.scope.as_ref().and_then(|scope| scope.execution_id.as_deref()).is_some_and(|id| {
                                        context.host.store().metadata(id).is_ok_and(|cut| cut.state != ExecutionState::Open)
                                    })
                                })
                            {
                                return;
                            }
                            loop {
                                tokio::select! {
                                    _ = sender.closed() => return,
                                    changed = storage.changed() => if changed.is_err() { return; },
                                    changed = runtime.changed() => if changed.is_err() { return; },
                                }
                                if context.host.shutting_down() {
                                    return;
                                }
                                let changed = selection.requests.iter().any(|request| {
                                    if request.analysis.is_some() {
                                        return true;
                                    }
                                    let Some(execution) = request.scope.as_ref().and_then(|scope| scope.execution_id.as_deref()) else {
                                        return true;
                                    };
                                    let Ok(cut) = context.host.store().metadata(execution) else {
                                        return true;
                                    };
                                    let through = request
                                        .at
                                        .as_ref()
                                        .and_then(|at| at.positions.first())
                                        .map_or(cut.frontier, |position| position.sequence);
                                    observed.get(request.plot_id.as_deref().unwrap_or(&request.stream)) != Some(&through)
                                        || cut.state != ExecutionState::Open
                                });
                                if changed {
                                    break;
                                }
                            }
                            opening = false;
                            selection.at = None;
                            selection.page.get_or_insert_default().page_token.clear();
                        }
                        page = match Box::pin(super::plots_snapshots_list::handle(&context, selection.clone())).await {
                            Ok(page) => page,
                            Err(error) => {
                                let _ = sender
                                    .send(Ok(SnapshotsWatchResponse {
                                        resync_required: Some(Failure {
                                            code: "resync_required".into(),
                                            message: format!("{ID}: {error}"),
                                            ..Default::default()
                                        }),
                                        ..Default::default()
                                    }))
                                    .await;
                                return;
                            }
                        };
                    }
                };
            if tokio::time::timeout(context.settings.watch_timeout, work).await.is_err() {
                let _ = sender.try_send(Err(tonic::Status::deadline_exceeded(format!("{ID}: Reopen After Its Deadline"))));
            }
        });
        Ok(ReceiverStream::new(output))
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
