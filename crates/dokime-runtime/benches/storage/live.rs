/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Live Plot Refresh Measurements

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    env,
    ops::Range,
    path::Path,
    thread,
    time::{Duration, Instant},
};

// Third Party Imports
use criterion::{Criterion, SamplingMode, Throughput};
use prost::Message;
use tempfile::TempDir;
use tokio::runtime::Runtime;
use tokio::task::JoinSet;

// Local Imports
use super::{boundary, publication, report, reports};
use dokime_client_internal::ExecutionOutcome;
use dokime_operations::host::documents::plot;
use dokime_operations::host::plots::snapshots::list;
use dokime_protocol::builders::Plot;
use dokime_protocol::host::ReportsCreateRequest;
use dokime_protocol::host::plots::snapshots_client::SnapshotsClient;
use dokime_protocol::host::plots::{PlotRequest, SnapshotsListRequest, SnapshotsWatchRequest};
use dokime_protocol::model::{Document, EvidenceScope, FieldReference, Termination, card::Body};
use dokime_runtime::services;
use dokime_storage::{StorageSettings, Store, storage_defaults};

/* === Constants === */

/// Live Capture Lengths at Which a Refresh Is Measured.
const MINUTES: [usize; 3] = [1, 10, 60];
/// Seconds Between Flushes, as the Product's Flush Interval Saves Them.
const FLUSH_SECONDS: usize = 5;
/// Seconds of Accepted Rows Still Pending When a Refresh Reads.
const PENDING_SECONDS: usize = 2;
/// The Desktop's Default Live Plot Window.
const WINDOW_S: f64 = 300.0;
/// The Desktop's Default Point Budget.
const POINTS: u64 = 2000;
/// Streams at 1 kHz Saved Together by One Flush.
const STREAMS: usize = 128;
/// Paced Pages Whose Start Times Give the Live Cadence.
const PAGES: usize = 8;
/// Signal Rates in Hz, Cycled Over the Signals.
const RATES_HZ: [usize; 4] = [1, 2, 5, 10];
/// Seconds of Signals Captured, So the Live Window Cuts the Oldest Rows.
const SIGNAL_SECONDS: usize = 360;
/// Signals, One Plot Each.
const SIGNALS: usize = 16;
/// Plots per Measured Refresh.
const PLOTS: [usize; 3] = [1, 4, SIGNALS];
/// Refreshes Averaged per Stage Row.
const REFRESHES: usize = 20;
const BASE_NS: i64 = 1_788_220_800_000_000_000;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/// One Open Execution Growing in Five-Second Flushes With a Pending Tail.
struct Live {
    store: Store,
    seconds: usize,
    root: TempDir,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Live {
    fn new() -> Self {
        let root = tempfile::tempdir().unwrap();
        // Flushes Are Driven Explicitly, So a Slow Generator Never Saves the Pending Tail Early.
        let settings = StorageSettings {
            flush_interval: Duration::from_secs(86_400),
            ..storage_defaults()
        };
        let store = Store::open_with(root.path(), settings).unwrap();
        reports::open(&store, reports::document());
        Self { store, seconds: 0, root }
    }

    /// Flush Everything Pending, Then Capture Five More Seconds Leaving the Last Two Pending.
    fn advance(&mut self) {
        let saved = self.seconds + FLUSH_SECONDS - PENDING_SECONDS;
        reports::capture(&self.store, self.seconds..saved);
        self.store.flush_samples().unwrap();
        reports::capture(&self.store, saved..self.seconds + FLUSH_SECONDS);
        self.seconds += FLUSH_SECONDS;
    }

    fn grow(&mut self, minutes: usize) {
        let started = Instant::now();
        while self.seconds + FLUSH_SECONDS <= minutes * 60 {
            self.advance();
        }
        if env::var_os("DOKIME_BENCH_ONCE").is_some() {
            println!("Live Source | {minutes} min | {:.3} s to capture", started.elapsed().as_secs_f64());
        }
    }

    /// Seal the Grown Execution, Restart, Then Freeze Its Report and Evidence Archive as the First Download Does.
    async fn freeze(self, times: &reports::Times) {
        let Self { store, seconds, root } = self;
        store.close_streams("execution").unwrap();
        store.close("execution", Termination::Completed, ExecutionOutcome::default()).unwrap();
        store.seal("execution", Default::default()).unwrap();
        store.shutdown_samples().await.unwrap();
        drop(store);
        let mut delivery = reports::Delivery::serve(root, Vec::new()).await;
        times.0.lock().unwrap().clear();
        let started = Instant::now();
        let request = ReportsCreateRequest {
            subject: Some(reports::subject()),
            request_id: "freeze".into(),
            ..Default::default()
        };
        delivery.reports.create(request).await.unwrap();
        println!("Freeze Report | {} min | {:.6} ms", seconds / 60, started.elapsed().as_secs_f64() * 1000.0);
        reports::stages(times, "Freeze", 0);
        let started = Instant::now();
        delivery.archive().await;
        println!("Freeze Evidence Archive | {} min | {:.6} ms", seconds / 60, started.elapsed().as_secs_f64() * 1000.0);
        delivery.server.close().await.unwrap();
        services::remove_credentials(delivery.root.path(), &publication::credentials()).unwrap();
    }

    /// Page Starts of Watch-Paced Refreshes With Rows Always Pending: Each Starts Once the Configured Interval Since
    /// the Previous Start Has Passed, or at Once After a Slower Page.
    fn cadence(&mut self, executor: &Runtime, requests: &[PlotRequest], case: &str) {
        let interval = storage_defaults().previews.refresh_interval;
        let origin = Instant::now();
        let mut starts = Vec::new();
        for _ in 0..PAGES {
            let started = Instant::now();
            starts.push(started - origin);
            self.refresh(executor, requests);
            thread::sleep((started + interval).saturating_duration_since(Instant::now()));
        }
        let gaps = starts.windows(2).map(|pair| (pair[1] - pair[0]).as_secs_f64() * 1000.0).collect::<Vec<_>>();
        println!(
            "Live Cadence | {case} | mean={:.1} ms | min={:.1} ms | max={:.1} ms | interval={} ms",
            gaps.iter().sum::<f64>() / gaps.len() as f64,
            gaps.iter().copied().fold(f64::INFINITY, f64::min),
            gaps.iter().copied().fold(0.0, f64::max),
            interval.as_millis()
        );
    }

    fn refresh(&self, executor: &Runtime, requests: &[PlotRequest]) -> Duration {
        let message_bytes = publication::settings().placement.message_bytes.get();
        let started = Instant::now();
        let envelopes = executor.block_on(list::snapshots(&self.store, requests.to_vec(), 1000, 0, "epoch", Some(message_bytes))).unwrap();
        let elapsed = started.elapsed();
        assert_eq!(envelopes.len(), requests.len(), "Refresh Live Plots: Return Every Plot");
        assert!(envelopes.iter().all(|envelope| envelope.payload.is_some()), "Refresh Live Plots: Return Every Payload");
        elapsed
    }
}

/* === Public Functions === */

/// The Refresh Stage Table Runs Alone, Selected by Its Name.
pub(super) fn selected() -> bool {
    env::args().any(|argument| argument == "refresh")
}

/// One Refresh of 1, 4 and 16 Live Plots, Opened as a Watch or Read as a List, and of Overlapping Watches, Stage by Stage.
pub(super) fn stages(executor: &Runtime, times: &reports::Times) {
    boundary(
        "Live Plot Refresh Stages",
        "open execution of sixteen signals at 1, 2, 5 and 10 Hz over six minutes, flushed, served by a restarted host; one Snapshots.Watch to its first page, or one Snapshots.List, of 1, 4 or 16 time series over a 300 s window at the desktop's point budget, then sixteen one-plot watches opened at once as the desktop's plot groups overlap; each refresh asks a distinct budget, so each reply is new as advancing rows make it live; wall is the mean page latency, stages are milliseconds per page, plot.query summed over parallel plots",
    );
    let root = tempfile::tempdir().unwrap();
    executor.block_on(signals(root.path()));
    executor.block_on(async {
        let server = reports::host(root.path()).await;
        let snapshots = SnapshotsClient::connect(format!("http://{}", server.endpoint())).await.unwrap();
        let mut refresh = 0;
        let cases = PLOTS.into_iter().flat_map(|plots| [("watch", 1, plots), ("list", 1, plots)]).chain([("watches", SIGNALS, 1)]);
        for (case, watches, plots) in cases {
            let before = views(root.path());
            times.0.lock().unwrap().clear();
            let mut elapsed = Duration::ZERO;
            for _ in 0..REFRESHES {
                let mut pages = JoinSet::new();
                for watch in 0..watches {
                    refresh += 1;
                    let requests = signal_plots(watch * plots..(watch + 1) * plots, refresh);
                    let mut snapshots = snapshots.clone();
                    pages.spawn(async move {
                        let started = Instant::now();
                        let served = if case == "list" {
                            let request = SnapshotsListRequest { requests, ..Default::default() };
                            snapshots.list(request).await.unwrap().into_inner().envelopes.len()
                        } else {
                            let request = SnapshotsWatchRequest { requests, ..Default::default() };
                            let mut frames = snapshots.watch(request).await.unwrap().into_inner();
                            frames.message().await.unwrap().unwrap().updates.len()
                        };
                        assert_eq!(served, plots, "Refresh Live Plots: Return Every Plot");
                        started.elapsed()
                    });
                }
                while let Some(latency) = pages.join_next().await {
                    elapsed += latency.unwrap();
                }
            }
            let pages = (REFRESHES * watches) as f64;
            let line = std::mem::take(&mut *times.0.lock().unwrap())
                .iter()
                .map(|(phase, values)| format!("{phase}={:.3}", values.iter().sum::<f64>() / pages))
                .collect::<Vec<_>>()
                .join(" | ");
            println!(
                "Refresh Stages | {case} | {watches} x {plots} plots | wall={:.3} ms | {line} | views +{}",
                elapsed.as_secs_f64() * 1000.0 / pages,
                views(root.path()) - before
            );
        }
        server.close().await.unwrap();
    });
    services::remove_credentials(root.path(), &publication::credentials()).unwrap();
}

pub(super) fn benchmark(criterion: &mut Criterion, executor: &Runtime, times: &reports::Times) {
    let mut group = criterion.benchmark_group("plots/live");
    group.sample_size(10);
    group.sampling_mode(SamplingMode::Flat);
    let mut live = None;
    for minutes in MINUTES {
        for (case, window_s) in [("window", Some(WINDOW_S)), ("all", None)] {
            boundary(
                &format!("Live Plot Refresh, {minutes} min ({case})"),
                "open execution: three 1 kHz telemetry streams, eight 10 Hz status streams, Wi-Fi scans and records saved every 5 s with 2 s pending; one Snapshots.List of the document's 15 plots at the desktop's point budget over a 300 s window or all samples; 5 s more capture before each refresh, excluded",
            );
            group.bench_function(format!("{case}/{minutes}min"), |b| {
                let live = live.get_or_insert_with(Live::new);
                live.grow(minutes);
                let requests = requests(window_s);
                b.iter_custom(|iterations| {
                    let mut elapsed = Duration::ZERO;
                    for _ in 0..iterations {
                        live.advance();
                        times.0.lock().unwrap().clear();
                        elapsed += live.refresh(executor, &requests);
                        reports::stages(times, &format!("Live {case} {minutes} min"), 0);
                    }
                    report(elapsed, iterations)
                });
                if minutes > 1 {
                    live.cadence(executor, &requests, &format!("{case}, {minutes} min"));
                }
            });
        }
    }
    if let Some(live) = live {
        boundary(
            "Seal-Time Freeze, 60 min",
            "the grown execution sealed, the host restarted, then the first Reports.Create freezes the report and Evidence.Archive the archive, as sealing does; printed once, outside the statistical samples",
        );
        executor.block_on(live.freeze(times));
    }
    group.finish();
}

/// Flush One Second of 128 Streams at 1 kHz, the Throughput Target, With Every Field's Statistics.
pub(super) fn ingestion(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("storage/flush");
    group.sample_size(10);
    group.sampling_mode(SamplingMode::Flat);
    group.throughput(Throughput::Elements((STREAMS * 1000) as u64));
    boundary(
        "Flush 128 Streams at 1 kHz",
        "one flush_samples of one second of 128 one-field streams at 1 kHz admitted without a flush; previews and statistics advanced; durable; admission excluded",
    );
    group.bench_function("128_streams_1khz", |b| {
        let root = tempfile::tempdir().unwrap();
        let settings = StorageSettings {
            flush_interval: Duration::from_secs(86_400),
            ..storage_defaults()
        };
        let store = Store::open_with(root.path(), settings).unwrap();
        let streams = (0..STREAMS).map(|index| format!("telemetry.{index}")).collect::<Vec<_>>();
        store.create(&reports::execution(&streams, Document::default())).unwrap();
        store.open_streams("execution", &streams).unwrap();
        let mut second = 0;
        b.iter_custom(|iterations| {
            let mut elapsed = Duration::ZERO;
            for _ in 0..iterations {
                for stream in &streams {
                    let batch = (second * 1000..(second + 1) * 1000)
                        .map(|index| {
                            let value = reports::Reading {
                                value: 10.0 * (index as f64 * 0.01).sin(),
                            };
                            reports::sample(stream, index, index as i64 * 1_000_000, value.encode_to_vec())
                        })
                        .collect::<Vec<_>>();
                    store.sample_batch("execution", &batch).unwrap();
                }
                second += 1;
                let started = Instant::now();
                store.flush_samples().unwrap();
                elapsed += started.elapsed();
            }
            report(elapsed, iterations)
        });
    });
    group.finish();
}

/* === Private Functions === */

/// Write an Open Execution of 1-10 Hz Signals Into a Fresh Storage Root and Flush It.
async fn signals(root: &Path) {
    let store = Store::open(root).unwrap();
    let streams = (0..SIGNALS).map(|index| format!("signal.{index}")).collect::<Vec<_>>();
    store.create(&reports::execution(&streams, Document::default())).unwrap();
    store.open_streams("execution", &streams).unwrap();
    for (index, stream) in streams.iter().enumerate() {
        let rate = RATES_HZ[index % RATES_HZ.len()];
        let batch = (0..SIGNAL_SECONDS * rate)
            .map(|ordinal| {
                let value = reports::Reading { value: (ordinal as f64 * 0.1).sin() };
                reports::sample(stream, ordinal, BASE_NS + ordinal as i64 * 1_000_000_000 / rate as i64, value.encode_to_vec())
            })
            .collect::<Vec<_>>();
        store.sample_batch("execution", &batch).unwrap();
    }
    store.flush_samples().unwrap();
    store.shutdown_samples().await.unwrap();
}

/// Live Time Series of the Selected Signals; a Distinct Point Budget per Refresh Gives Each Refresh Its Own Reply.
fn signal_plots(signals: Range<usize>, refresh: usize) -> Vec<PlotRequest> {
    let scope = EvidenceScope {
        execution_id: Some("execution".into()),
        cycle_id: Some("cycle".into()),
        ..Default::default()
    };
    signals
        .map(|index| {
            let declared = Plot::time_series(FieldReference::new(format!("signal.{index}")).field("value")).message;
            PlotRequest {
                plot_id: Some(format!("signal.{index}")),
                max_points: Some(POINTS + refresh as u64),
                window_s: Some(WINDOW_S),
                ..plot::request(&declared, scope.clone()).unwrap()
            }
        })
        .collect()
}

/// Retained View Files in the Root.
fn views(root: &Path) -> i64 {
    let database = rusqlite::Connection::open(root.join("storage.sqlite3")).unwrap();
    database.query_row("SELECT COUNT(*) FROM storage_files WHERE id LIKE 'view/%'", [], |row| row.get(0)).unwrap()
}

/// The Document's Plots as the Desktop Requests Them While an Execution Runs.
fn requests(window_s: Option<f64>) -> Vec<PlotRequest> {
    let scope = EvidenceScope {
        execution_id: Some("execution".into()),
        cycle_id: Some("cycle".into()),
        ..Default::default()
    };
    reports::document()
        .sections
        .iter()
        .flat_map(|section| &section.cards)
        .filter_map(|card| match &card.body {
            Some(Body::Plot(declared)) => Some(PlotRequest {
                plot_id: card.metadata.as_ref().map(|metadata| metadata.identifier.clone()),
                max_points: Some(POINTS),
                window_s,
                // The Desktop Sends a Live Window's Panel Width Only Below the Point Budget.
                panel_width: window_s.is_none().then_some(640),
                ..plot::request(declared, scope.clone()).unwrap()
            }),
            _ => None,
        })
        .collect()
}

/* === Tests === */
