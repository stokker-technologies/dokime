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
    time::{Duration, Instant},
};

// Third Party Imports
use criterion::{Criterion, SamplingMode};
use tempfile::TempDir;
use tokio::runtime::Runtime;

// Local Imports
use super::{boundary, publication, report, reports};
use dokime_client_internal::ExecutionOutcome;
use dokime_operations::host::documents::plot;
use dokime_operations::host::plots::snapshots::list;
use dokime_protocol::host::ReportsCreateRequest;
use dokime_protocol::host::plots::PlotRequest;
use dokime_protocol::model::{EvidenceScope, PlotKind, Termination, card::Body};
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
        // A Report Over Every Scan Exceeds the Heat Strip's Cell Budget, So the Sealed Document Omits It.
        let mut document = reports::document();
        for section in &mut document.sections {
            section.cards.retain(|card| !matches!(&card.body, Some(Body::Plot(plot)) if plot.kind == PlotKind::HeatStrip as i32));
        }
        reports::open(&store, document);
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

pub(super) fn benchmark(criterion: &mut Criterion, executor: &Runtime, times: &reports::Times) {
    let mut group = criterion.benchmark_group("plots/live");
    group.sample_size(10);
    group.sampling_mode(SamplingMode::Flat);
    let mut live = None;
    for minutes in MINUTES {
        for (case, window_s) in [("window", Some(WINDOW_S)), ("all", None)] {
            boundary(
                &format!("Live Plot Refresh, {minutes} min ({case})"),
                "open execution: three 1 kHz telemetry streams, eight 10 Hz status streams, Wi-Fi scans and records saved every 5 s with 2 s pending; one Snapshots.List of the document's 15 plots at the desktop's point budget over a 300 s window, or its 14 plots other than the heat strip over all samples; 5 s more capture before each refresh, excluded",
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

/* === Private Functions === */

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
            // A Heat Strip of Every Scan Exceeds Its Cell Budget Within Minutes, So Only Its Window Is Plotted.
            Some(Body::Plot(declared)) if window_s.is_some() || declared.kind != PlotKind::HeatStrip as i32 => Some(PlotRequest {
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
