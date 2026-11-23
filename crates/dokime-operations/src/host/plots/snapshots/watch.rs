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
use std::collections::{BTreeMap, BTreeSet};
use std::ops::Bound;
use std::time::Duration;

// Third Party Imports
use harmos_protocol::model::Failure;
use tokio::sync::{mpsc, watch};
use tokio::time;
use tokio_stream::wrappers::ReceiverStream;
use tonic::Status;

// Local Imports
use super::list;
use crate::{
    Result,
    host::{self, Context, Output, evidence::selection},
};
use dokime_protocol::host::plots::PlotRequest;
use dokime_protocol::host::plots::PlotStreamKind;
use dokime_protocol::host::plots::PlotStreamUpdate;
use dokime_protocol::host::plots::SnapshotsListRequest;
use dokime_protocol::host::plots::SnapshotsWatchRequest;
use dokime_protocol::host::plots::SnapshotsWatchResponse;
use dokime_protocol::model::ExpressionSource;
use dokime_storage::ExecutionState;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.plots.Snapshots/Watch";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/// Live Page Pacing: a Refresh Starts Once the Interval Since the Previous Page Started Has Passed, So Pages Keep a
/// Steady Cadence; a Page Slower Than the Interval Is Followed at Once, Without a Backlog. An Evaluation That Finds No
/// Plot Behind Starts the Interval Again.
struct Pacer {
    interval: Duration,
    started: time::Instant,
}

/// One Watched Plot and the Sources It Reads, Keyed as Storage Publishes Them: Streams by Identifier, Records by
/// Their Evidence Kind.
struct Plot {
    request: PlotRequest,
    sources: BTreeSet<String>,
}

/* === Enums === */

#[derive(Debug, PartialEq, Eq)]
enum WatchProgress {
    /// The Plots, by Request Index, Whose Own Sources Published Past Their Last Frame.
    Refresh(Vec<usize>),

    Waiting,
    Complete,
}

/* === Traits === */

/* === Implementations === */

impl Pacer {
    fn start(interval: Duration) -> Self {
        Self { interval, started: time::Instant::now() }
    }

    fn due(&self) -> bool {
        time::Instant::now() >= self.started + self.interval
    }

    async fn wait(&self) {
        time::sleep_until(self.started + self.interval).await;
    }
}

impl Plot {
    fn new(request: PlotRequest) -> Self {
        let mut sources = request
            .sources()
            .into_iter()
            .map(|source| match source {
                ExpressionSource::Stream(stream) => stream.to_owned(),
                ExpressionSource::Record(record) => format!("record:{record}"),
            })
            .collect::<BTreeSet<_>>();
        // A Step's Boundaries Bound the Rows a Step-Scoped Plot Reads.
        if request.scope.as_ref().and_then(|scope| scope.step_id.as_deref()).is_some_and(|step| !step.is_empty()) {
            sources.extend(["step:start".to_owned(), "step:end".to_owned()]);
        }
        Self { request, sources }
    }

    fn id(&self) -> &str {
        self.request.plot_id.as_deref().unwrap_or(&self.request.stream)
    }

    /// The Last Position Any Source of This Plot Published, Including Records Its Markers Match; a Plot Naming No
    /// Source Follows Every Source.
    fn latest(&self, published: &BTreeMap<String, u64>) -> u64 {
        if self.sources.is_empty() {
            return published.values().copied().max().unwrap_or(0);
        }
        let marked = published
            .range::<str, _>((Bound::Included("record:"), Bound::Unbounded))
            .map_while(|(kind, position)| Some((kind.strip_prefix("record:")?, position)))
            .filter(|(record, _)| self.request.marker_record_types.iter().any(|pattern| selection::matches(pattern, record)));
        self.sources
            .iter()
            .filter_map(|source| published.get(source))
            .chain(marked.map(|(_, position)| position))
            .copied()
            .max()
            .unwrap_or(0)
    }
}

/* === Public Functions === */

pub async fn handle(context: &Context, request: SnapshotsWatchRequest) -> Result<Output<SnapshotsWatchResponse>> {
    host::complete(ID, context.settings.watch_timeout, async {
        let mut changes = [context.host.store().subscribe(), context.host.changes()];
        let plots = request.requests.into_iter().map(Plot::new).collect::<Vec<_>>();
        let mut selection = SnapshotsListRequest {
            requests: plots.iter().map(|plot| plot.request.clone()).collect(),
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
                        let evaluate = || {
                            if context.host.shutting_down() {
                                return WatchProgress::Complete;
                            }
                            let store = context.host.store();
                            progress(
                                &plots,
                                &observed,
                                |execution, plot| store.published(execution, |published| plot.latest(published)).ok().flatten(),
                                |execution, live| {
                                    let cut = store.metadata(execution).ok()?;
                                    let frontier = if live { store.accepted_frontier(execution).ok()? } else { cut.frontier };
                                    Some((cut.state, frontier))
                                },
                            )
                        };
                        let refresh = tokio::select! {
                            _ = sender.closed() => return,
                            refresh = behind(&pacer, &mut changes, evaluate) => refresh,
                        };
                        let Some(refresh) = refresh else { return };
                        pacer = Pacer::start(pacer.interval);
                        opening = false;
                        // Only Plots Behind Their Own Sources Are Queried and Sent; the Rest Keep Their Frames.
                        selection.requests = refresh.into_iter().map(|index| plots[index].request.clone()).collect();
                        selection.at = None;
                        selection.page.get_or_insert_default().page_token.clear();
                    }
                    let query = time::Instant::now();
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
                    let elapsed = query.elapsed();
                    if elapsed > pacer.interval {
                        tracing::info!(
                            plots = selection.requests.len(),
                            elapsed_ms = elapsed.as_secs_f64() * 1000.0,
                            "Plot Refresh Outlasted Its Interval"
                        );
                    }
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

/// Wait Until Some Plot Is Behind Its Own Sources and the Pacer Allows a Page; Changes Meanwhile Join That Refresh.
/// None Once Every Plot Is Final or the Changes End.
async fn behind(pacer: &Pacer, changes: &mut [watch::Receiver<u64>; 2], mut evaluate: impl FnMut() -> WatchProgress) -> Option<Vec<usize>> {
    loop {
        match evaluate() {
            WatchProgress::Complete => return None,
            // The Interval Runs From the Last Refresh; Changes While It Runs Join That Refresh.
            WatchProgress::Refresh(plots) if pacer.due() => return Some(plots),
            WatchProgress::Refresh(_) => pacer.wait().await,
            // A Change No Watched Plot Reads Costs One Evaluation, a Lookup Without a Query, and Never Delays a Refresh.
            WatchProgress::Waiting => {
                let [storage, runtime] = changes;
                let changed = tokio::select! {
                    changed = storage.changed() => changed,
                    changed = runtime.changed() => changed,
                };
                if changed.is_err() {
                    return None;
                }
            }
        }
    }
}

/// Which Plots Show Less Than Their Own Sources Published.
///
/// `published` reads, without a query, the last position a plot's sources published while its execution takes data,
/// and None once it closed. Only a closed execution, or a plot of saved rows found behind, makes `metadata` read the
/// execution's state and the frontier the plot reads through.
fn progress(
    plots: &[Plot], observed: &BTreeMap<String, u64>, mut published: impl FnMut(&str, &Plot) -> Option<u64>,
    mut metadata: impl FnMut(&str, bool) -> Option<(ExecutionState, u64)>,
) -> WatchProgress {
    let mut refresh = Vec::new();
    let mut complete = true;
    for (index, plot) in plots.iter().enumerate() {
        let Some(execution) = plot.request.scope.as_ref().and_then(|scope| scope.execution_id.as_deref()) else {
            refresh.push(index);
            continue;
        };
        let shown = observed.get(plot.id()).copied();
        let latest = published(execution, plot);
        let at = plot.request.at.as_ref().and_then(|at| at.positions.first()).map(|position| position.sequence);
        // Every Uncut Plot Follows Published Rows, With or Without a Live Window.
        let live = at.is_none();
        let cut = at.is_some();
        let current = shown.zip(latest).is_some_and(|(shown, latest)| latest <= shown);
        // While the Execution Takes Data, a Live Plot Reads Every Published Position and Another Plot at Most Its Cut.
        if let Some(latest) = latest
            && (live || cut || current)
        {
            if shown.is_none_or(|shown| at.map_or(latest, |at| latest.min(at)) > shown) {
                refresh.push(index);
            }
            complete = false;
            continue;
        }
        let Some((state, frontier)) = metadata(execution, live) else {
            refresh.push(index);
            continue;
        };
        // A Plot Reads Through Its Cut, Its Live Frontier or the Saved Frontier, Whichever Its Request Selects.
        let through = at.unwrap_or(frontier);
        if shown.is_none_or(|shown| latest.map_or(through, |latest| latest.min(through)) > shown) {
            refresh.push(index);
        }
        complete &= latest.is_none() && state != ExecutionState::Open;
    }
    if !refresh.is_empty() {
        WatchProgress::Refresh(refresh)
    } else if complete {
        WatchProgress::Complete
    } else {
        WatchProgress::Waiting
    }
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use tokio::sync::watch;
    use tokio::time::{self, Instant};

    use super::{Pacer, Plot, WatchProgress, behind, progress};
    use dokime_protocol::host::plots::PlotRequest;
    use dokime_protocol::model::{EvidenceScope, FieldReference};
    use dokime_storage::ExecutionState;
    use harmos_protocol::model::{Position, ReadCut};

    /// A Live Plot of the Execution Under Test.
    fn plot(id: &str, request: PlotRequest) -> Plot {
        Plot::new(PlotRequest {
            plot_id: Some(id.into()),
            scope: Some(EvidenceScope {
                execution_id: Some("execution".into()),
                ..Default::default()
            }),
            window_s: Some(300.0),
            ..request
        })
    }

    /// Refreshing Plots Show Every Position Published So Far.
    fn shown(plots: &[Plot], refresh: &[usize], observed: &mut BTreeMap<String, u64>, published: &BTreeMap<String, u64>) {
        for index in refresh {
            observed.insert(plots[*index].id().into(), published.values().copied().max().unwrap_or(0));
        }
    }

    #[test]
    fn sealed_watch_refreshes_until_every_plot_reaches_its_final_frontier() {
        let plots = ["first", "last"].map(|id| plot(id, PlotRequest::default()));
        let mut observed = BTreeMap::from([("first".into(), 3), ("last".into(), 3)]);
        let closed = |_: &str, _: &Plot| None;
        assert_eq!(progress(&plots, &observed, closed, |_, _| Some((ExecutionState::Open, 3))), WatchProgress::Waiting);

        // Seal After Delivering a Pre-Seal Page, Then Drain Every Final Page.
        assert_eq!(progress(&plots, &observed, closed, |_, _| Some((ExecutionState::Sealed, 5))), WatchProgress::Refresh(vec![0, 1]));
        observed.insert("first".into(), 5);
        assert_eq!(progress(&plots, &observed, closed, |_, _| Some((ExecutionState::Sealed, 5))), WatchProgress::Refresh(vec![1]));
        observed.insert("last".into(), 5);
        assert_eq!(progress(&plots, &observed, closed, |_, _| Some((ExecutionState::Sealed, 5))), WatchProgress::Complete);
    }

    #[test]
    fn uncut_plots_follow_published_rows_and_cut_plots_stop_at_their_cut() {
        let mut saved = plot("saved", PlotRequest { stream: "a".into(), ..Default::default() });
        saved.request.window_s = None;
        let cut = ReadCut {
            positions: vec![Position { sequence: 150, ..Default::default() }],
            ..Default::default()
        };
        let cut = plot(
            "cut",
            PlotRequest {
                stream: "a".into(),
                at: Some(cut),
                ..Default::default()
            },
        );
        let plots = [saved, cut];
        let observed = BTreeMap::from([("saved".into(), 150), ("cut".into(), 150)]);

        // A Pending Row Refreshes a Plot Without a Live Window at Once, and a Cut Plot Never Reads Past Its Cut.
        let pending = |_: &str, _: &Plot| Some(160);
        assert_eq!(progress(&plots, &observed, pending, |_, _| Some((ExecutionState::Open, 150))), WatchProgress::Refresh(vec![0]));
        let current = |_: &str, _: &Plot| Some(150);
        assert_eq!(progress(&plots, &observed, current, |_, _| Some((ExecutionState::Open, 150))), WatchProgress::Waiting);
    }

    #[test]
    fn only_plots_reading_a_published_stream_or_record_refresh() {
        let a = FieldReference::new("a");
        let b = FieldReference::new("b");
        let plots = [
            plot("a", PlotRequest { stream: "a".into(), ..Default::default() }),
            plot("b", PlotRequest { stream: "b".into(), ..Default::default() }),
            plot(
                "xy",
                PlotRequest {
                    x_expression: Some(a.field("x")),
                    y_expressions: vec![b.field("y")],
                    ..Default::default()
                },
            ),
            plot(
                "sampled",
                PlotRequest {
                    y_expressions: vec![FieldReference::new("c").field("value").at("event")],
                    ..Default::default()
                },
            ),
            plot(
                "marked",
                PlotRequest {
                    stream: "d".into(),
                    marker_record_types: vec!["phase.*".into()],
                    ..Default::default()
                },
            ),
            Plot::new(PlotRequest {
                plot_id: Some("step".into()),
                stream: "e".into(),
                scope: Some(EvidenceScope {
                    execution_id: Some("execution".into()),
                    step_id: Some("step".into()),
                    ..Default::default()
                }),
                window_s: Some(300.0),
                ..Default::default()
            }),
        ];
        let mut observed = plots.iter().map(|plot| (plot.id().to_owned(), 0)).collect();
        let mut published = BTreeMap::new();
        for (source, refreshed) in [
            ("a", vec![0, 2]),
            ("b", vec![1, 2]),
            ("record:event", vec![3]),
            ("c", vec![3]),
            ("record:phase.started", vec![4]),
            ("step:end", vec![5]),
            ("record:unrelated", vec![]),
        ] {
            published.insert(source.to_owned(), published.len() as u64 + 1);
            let progress = progress(&plots, &observed, |_, plot| Some(plot.latest(&published)), |_, _| panic!("Read No Execution"));
            if refreshed.is_empty() {
                assert_eq!(progress, WatchProgress::Waiting, "{source}");
            } else {
                assert_eq!(progress, WatchProgress::Refresh(refreshed.clone()), "{source}");
            }
            shown(&plots, &refreshed, &mut observed, &published);
        }
    }

    #[tokio::test(start_paused = true)]
    async fn refreshes_keep_their_interval_coalesce_changes_and_follow_slow_pages_at_once() {
        let plots = ["a", "b"].map(|stream| {
            plot(
                stream,
                PlotRequest {
                    stream: stream.into(),
                    ..Default::default()
                },
            )
        });
        let interval = Duration::from_millis(200);
        for (page, expected) in [(50, [0, 200, 400, 600]), (300, [0, 300, 600, 900])] {
            let published = Arc::new(Mutex::new(BTreeMap::new()));
            let (change, changed) = watch::channel(0);
            let change = Arc::new(change);
            let (_runtime, runtime) = watch::channel(0);
            let mut changes = [changed, runtime];
            let mut observed = BTreeMap::from([("a".to_owned(), 0), ("b".to_owned(), 0)]);
            let evaluate = |observed: &BTreeMap<String, u64>| {
                progress(&plots, observed, |_, plot| Some(plot.latest(&published.lock().unwrap())), |_, _| panic!("Read No Execution"))
            };
            let source = |source: &'static str| {
                let (published, change) = (published.clone(), change.clone());
                move || {
                    *published.lock().unwrap().entry(source.to_owned()).or_insert(0) += 1;
                    change.send_modify(|revision| *revision += 1);
                }
            };
            // Stream A Publishes Every 10 ms Between Pages, Far More Often Than Pages; B Never Does Meanwhile.
            let a = source("a");
            let publisher = tokio::spawn(async move {
                let mut ticks = time::interval_at(Instant::now() + Duration::from_millis(5), Duration::from_millis(10));
                loop {
                    ticks.tick().await;
                    a();
                }
            });
            let origin = Instant::now();
            let mut starts = Vec::new();
            let mut pacer = Pacer::start(interval);
            for _ in 0..4 {
                starts.push((Instant::now() - origin).as_millis());
                time::sleep(Duration::from_millis(page)).await;
                let refresh = behind(&pacer, &mut changes, || evaluate(&observed)).await.unwrap();
                pacer = Pacer::start(pacer.interval);
                assert_eq!(refresh, [0]);
                shown(&plots, &refresh, &mut observed, &published.lock().unwrap());
            }
            publisher.abort();
            assert_eq!(starts, expected);

            // An Unrelated Source Costs One Evaluation Without Delaying B, Which Refreshes Alone as Soon as It Publishes.
            let (unrelated, b) = (source("c"), source("b"));
            let quiet = Instant::now();
            tokio::spawn(async move {
                time::sleep(Duration::from_millis(100)).await;
                unrelated();
                time::sleep(Duration::from_millis(150)).await;
                b();
            });
            assert_eq!(behind(&pacer, &mut changes, || evaluate(&observed)).await.unwrap(), [1]);
            assert_eq!(quiet.elapsed(), Duration::from_millis(250));
        }
    }
}

crate::native::download!(SnapshotsWatchRequest, SnapshotsWatchResponse, "dokime.host.v1.plots", "Snapshots", "Watch");
