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
use std::time::Duration;

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

/// Live Page Pacing: a Refresh Starts Once the Interval Since the Previous Page Started Has Passed, So Pages Keep a
/// Steady Cadence; a Page Slower Than the Interval Is Followed at Once, Without a Backlog.
struct Pacer {
    interval: Duration,
    started: time::Instant,
}

/* === Enums === */

#[derive(Debug, PartialEq, Eq)]
enum WatchProgress {
    Refresh,
    Waiting,
    Complete,
}

/* === Traits === */

/* === Implementations === */

impl Pacer {
    fn start(interval: Duration) -> Self {
        Self { interval, started: time::Instant::now() }
    }

    async fn wait(&self) {
        time::sleep_until(self.started + self.interval).await;
    }
}

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
        let mut pacer = Pacer::start(context.host.store().storage_settings()?.previews.refresh_interval);
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
                        // Changes While the Interval Runs Coalesce Into This Refresh.
                        tokio::select! {
                            _ = sender.closed() => return,
                            () = pacer.wait() => {},
                        }
                        pacer = Pacer::start(pacer.interval);
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
    use std::time::Duration;

    use tokio::sync::watch;
    use tokio::time::{self, Instant};

    use super::{Pacer, WatchProgress, progress};
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

    #[tokio::test(start_paused = true)]
    async fn refreshes_keep_their_interval_coalesce_changes_and_follow_slow_pages_at_once() {
        // Change Every 10 ms, Far More Often Than Pages
        let (changes, mut changed) = watch::channel(0u64);
        tokio::spawn(async move {
            loop {
                time::sleep(Duration::from_millis(10)).await;
                changes.send_modify(|revision| *revision += 1);
            }
        });
        let interval = Duration::from_millis(200);
        for (page, expected) in [(50, [0, 200, 400, 600]), (300, [0, 300, 600, 900])] {
            let origin = Instant::now();
            let mut starts = Vec::new();
            let mut pacer = Pacer::start(interval);
            for _ in 0..4 {
                starts.push((Instant::now() - origin).as_millis());
                time::sleep(Duration::from_millis(page)).await;
                changed.changed().await.unwrap();
                pacer.wait().await;
                pacer = Pacer::start(pacer.interval);
            }
            assert_eq!(starts, expected);
        }
    }
}

crate::native::download!(SnapshotsWatchRequest, SnapshotsWatchResponse, "dokime.host.v1.plots", "Snapshots", "Watch");
