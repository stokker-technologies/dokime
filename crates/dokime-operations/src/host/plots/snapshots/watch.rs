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
use harmos_protocol::model::Failure;
use tokio::sync::mpsc;
use tokio::time;
use tokio_stream::wrappers::ReceiverStream;
use tonic::Status;

// Local Imports
use super::list;
use crate::{
    Result,
    host::{self, Context, Output},
};
use dokime_protocol::host::plots::PlotRequest;
use dokime_protocol::host::plots::PlotStreamKind;
use dokime_protocol::host::plots::PlotStreamUpdate;
use dokime_protocol::host::plots::SnapshotsListRequest;
use dokime_protocol::host::plots::SnapshotsWatchRequest;
use dokime_protocol::host::plots::SnapshotsWatchResponse;
use dokime_storage::ExecutionState;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.plots.Snapshots/Watch";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

#[derive(Debug, PartialEq, Eq)]
enum WatchProgress {
    Refresh,
    Waiting,
    Complete,
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub async fn handle(context: &Context, request: SnapshotsWatchRequest) -> Result<Output<SnapshotsWatchResponse>> {
    host::complete(ID, context.settings.watch_timeout, async {
        let mut storage = context.host.store().subscribe();
        let mut runtime = context.host.changes();
        let mut selection = SnapshotsListRequest {
            requests: request.requests,
            at: request.at,
            page: request.page,
        };
        // Keep the Query State off the Watch's Inline Future and Poll Stack.
        let first = Box::pin(list::handle(context, selection.clone())).await?;
        let context = context.clone();
        let (sender, output) = mpsc::channel(context.settings.stream_buffer);
        tokio::spawn(async move {
            let work = async {
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
                        loop {
                            if context.host.shutting_down() {
                                return;
                            }
                            let progress = progress(&selection.requests, &observed, |execution, live| {
                                let store = context.host.store();
                                let cut = store.metadata(execution).ok()?;
                                let frontier = if live { store.accepted_frontier(execution).ok()? } else { cut.frontier };
                                Some((cut.state, frontier))
                            });
                            match progress {
                                WatchProgress::Complete => return,
                                WatchProgress::Refresh => break,
                                WatchProgress::Waiting => {}
                            }
                            tokio::select! {
                                _ = sender.closed() => return,
                                changed = storage.changed() => if changed.is_err() { return; },
                                changed = runtime.changed() => if changed.is_err() { return; },
                            }
                        }
                        opening = false;
                        selection.at = None;
                        selection.page.get_or_insert_default().page_token.clear();
                    }
                    page = match Box::pin(list::handle(&context, selection.clone())).await {
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
            if time::timeout(context.settings.watch_timeout, work).await.is_err() {
                let _ = sender.try_send(Err(Status::deadline_exceeded(format!("{ID}: Reopen After Its Deadline"))));
            }
        });
        Ok(ReceiverStream::new(output))
    })
    .await
}

/* === Private Functions === */

fn progress(
    requests: &[PlotRequest], observed: &BTreeMap<String, u64>, mut metadata: impl FnMut(&str, bool) -> Option<(ExecutionState, u64)>,
) -> WatchProgress {
    let mut complete = true;
    for request in requests {
        let Some(execution) = request.scope.as_ref().and_then(|scope| scope.execution_id.as_deref()) else {
            return WatchProgress::Refresh;
        };
        let live = request.window_s.is_some() && request.at.is_none();
        let Some((state, frontier)) = metadata(execution, live) else {
            return WatchProgress::Refresh;
        };
        let through = request.at.as_ref().and_then(|at| at.positions.first()).map_or(frontier, |position| position.sequence);
        if observed.get(request.plot_id.as_deref().unwrap_or(&request.stream)) != Some(&through) {
            return WatchProgress::Refresh;
        }
        complete &= state != ExecutionState::Open;
    }
    if complete { WatchProgress::Complete } else { WatchProgress::Waiting }
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{WatchProgress, progress};
    use dokime_protocol::host::plots::PlotRequest;
    use dokime_protocol::model::EvidenceScope;
    use dokime_storage::ExecutionState;

    #[test]
    fn sealed_watch_refreshes_until_every_plot_reaches_its_final_frontier() {
        let requests = ["first", "last"].map(|plot| PlotRequest {
            plot_id: Some(plot.into()),
            scope: Some(EvidenceScope {
                execution_id: Some("execution".into()),
                ..Default::default()
            }),
            ..Default::default()
        });
        let mut observed = BTreeMap::from([("first".into(), 3), ("last".into(), 3)]);
        assert_eq!(progress(&requests, &observed, |_, _| Some((ExecutionState::Open, 3))), WatchProgress::Waiting);

        // Seal After Delivering a Pre-Seal Page, Then Drain Every Final Page.
        assert_eq!(progress(&requests, &observed, |_, _| Some((ExecutionState::Sealed, 5))), WatchProgress::Refresh);
        observed.insert("first".into(), 5);
        assert_eq!(progress(&requests, &observed, |_, _| Some((ExecutionState::Sealed, 5))), WatchProgress::Refresh);
        observed.insert("last".into(), 5);
        assert_eq!(progress(&requests, &observed, |_, _| Some((ExecutionState::Sealed, 5))), WatchProgress::Complete);
    }
}

crate::native::download!(SnapshotsWatchRequest, SnapshotsWatchResponse, "dokime.host.v1.plots", "Snapshots", "Watch");
