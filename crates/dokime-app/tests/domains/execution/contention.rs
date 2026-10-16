/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Instant;

// Third Party Imports

// Local Imports
use crate::support::plugins::fixture_bundle;
use crate::support::prelude::*;
use dokime_app::config::StorageSettings;
use dokime_app::domains::execution::requests::RunPhase;
use dokime_app::runtime::query::{PlotPayloadRequest, RuntimePlotQueryMode};

/* === Constants === */
/// The read pool the desktop ran with when its interactive reads were refused:
/// the old fixed default, smaller than any machine's sized pool today.
const INCIDENT_READ_MEMORY: std::num::NonZeroUsize =
    std::num::NonZeroUsize::new(256 * 1024 * 1024).unwrap();

/// Rows the running case pushes onto its pulse stream before it holds.
const PULSE_ROWS: u32 = 2_000;

/// Live plots an operator keeps open on the running cycle, as many as a plugin
/// such as Mini shows: about twice what that pool held when every
/// interactive view reserved the report profile.
const LIVE_PLOTS: usize = 15;

/// The plot every live watch follows.
const PULSE_PLOT: &str = "fixture.checks.smoke.pulse";

/// The pace of one live plot: the plot stream flushes a card at most this often.
const PLOT_STREAM_THROTTLE: Duration = Duration::from_millis(100);

/// How long the watches and the report loop contend before the operator acts.
const WARMUP: Duration = Duration::from_millis(500);

/// Receipt polls the operator's re-run makes while the reads contend.
const RECEIPT_POLLS: usize = 20;

/// The longest the story waits for a cycle to settle.
const SETTLE: Duration = Duration::from_secs(300);

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
/// What one live plot watch saw while it followed the running case.
#[derive(Debug, Default)]
struct WatchOutcome {
    served: usize,
    failures: Vec<String>,
}

/// What the open report saw while it rendered the running case again and again.
#[derive(Debug, Default)]
struct ReportOutcome {
    rendered: usize,
    failures: Vec<String>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
// one running cycle with fifteen live plot watches and an open report, as the
// desktop keeps them: an operator's case run and its receipt polls are
// admitted within the shipped interactive deadline at the incident's read pool,
// every watch keeps receiving payloads instead of waiting forever, and the
// report renders whole, leaving nothing out for want of read memory.
pub(crate) async fn contended_reads_stay_interactive() {
    let root = tempdir().unwrap();
    let release = root.path().join("release");
    let mut config =
        config(root.path()).with_plugins([fixture_bundle(root.path(), "fixture-suites")
            .configured(
                "fixture-suites",
                serde_json::json!({
                    "pulse_rows": PULSE_ROWS,
                    "block_execution": true,
                    "execution_release": release,
                }),
            )
            .unwrap()]);
    config.storage.budget.read_memory_bytes = INCIDENT_READ_MEMORY;
    config.storage.interactive.memory_bytes = INCIDENT_READ_MEMORY;
    config.storage.ceiling.memory_bytes = INCIDENT_READ_MEMORY;
    config.storage.page_bytes = std::num::NonZeroUsize::new(64 * 1024).unwrap();
    config.storage.page_bytes_ceiling = INCIDENT_READ_MEMORY;
    config.storage.decoded_bytes_ceiling = INCIDENT_READ_MEMORY;
    let interactive = StorageSettings::default().interactive.timeout;
    let (mut runtime, handle) = started(config).await;
    let suite = handle
        .test_suites()
        .await
        .into_iter()
        .find(|suite| suite.source == "fixture-suites")
        .unwrap()
        .suite;

    // A Queued Cycle Whose Case The Operator Runs Later.
    let queued = start(&handle, &suite, "contention.queued").await;
    let case = cycle_snapshot(&handle, &queued)
        .await
        .cases()
        .next()
        .unwrap()
        .test_case_id
        .clone();

    // The Running Cycle Holds Once Its Pulse Is Published.
    let running = start(&handle, &suite, "contention.running").await;
    let held = handle
        .run_cycle_cases(running.clone(), CommandContext::default())
        .await;
    assert_eq!(held.status, CommandStatus::Accepted, "{held:?}");
    let execution = handle.watch_state().latest().state().cycles[&running].execution_ids[0].clone();
    tokio::time::timeout(SETTLE, async {
        loop {
            if let Ok(opened) = handle.open_execution_report(&execution).await {
                let payload =
                    crate::support::reports::parse(std::str::from_utf8(&opened.html).unwrap());
                if payload["case_runs"][0]["plots"][PULSE_PLOT]["series"][0]["raw_points"]
                    == serde_json::json!(PULSE_ROWS)
                {
                    break;
                }
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    })
    .await
    .expect("the running step publishes every pulse row");

    // Fifteen Live Plots And The Open Report Contend For The Read Ledger.
    let stop = Arc::new(AtomicBool::new(false));
    let first_payloads = Arc::new(AtomicUsize::new(0));
    let scope = RecordScope {
        test_cycle_id: Some(running.clone()),
        test_execution_id: Some(execution.clone()),
        ..RecordScope::default()
    };
    let watches = (0..LIVE_PLOTS)
        .map(|index| {
            tokio::spawn(watch(
                handle.clone(),
                pulse_request(index, scope.clone()),
                stop.clone(),
                first_payloads.clone(),
            ))
        })
        .collect::<Vec<_>>();
    tokio::time::timeout(SETTLE, async {
        while first_payloads.load(Ordering::Relaxed) < LIVE_PLOTS {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("every live plot receives its first payload");
    let report = tokio::spawn(report_loop(handle.clone(), execution.clone(), stop.clone()));
    tokio::time::sleep(WARMUP).await;

    // The Operator Runs The Queued Case And Polls Its Receipt: The Request A
    // Case Re-Run Makes Once It Is Scheduled.
    let admitted = Instant::now();
    let case_run = handle
        .run_cycle_case(queued.clone(), case, CommandContext::default())
        .await;
    assert_within(admitted, interactive, "admitting the case run");
    assert_eq!(case_run.status, CommandStatus::Accepted, "{case_run:?}");
    for poll in 0..RECEIPT_POLLS {
        let polled = Instant::now();
        handle
            .run_request(&case_run.command_id)
            .await
            .unwrap_or_else(|error| panic!("receipt poll {poll}: {error}"))
            .expect("the admitted re-run has a receipt");
        assert_within(polled, interactive, "a receipt poll");
        let listed = Instant::now();
        assert!(
            handle
                .cycles()
                .await
                .iter()
                .any(|cycle| cycle.test_cycle_id == queued)
        );
        assert_within(listed, interactive, "the cycle list");
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    stop.store(true, Ordering::Relaxed);
    for (index, watch) in watches.into_iter().enumerate() {
        let outcome = watch.await.unwrap();
        assert!(
            outcome.failures.is_empty(),
            "live plot {index} was refused: {:?}",
            outcome.failures
        );
        assert!(
            outcome.served > 1,
            "live plot {index} kept receiving payloads"
        );
    }
    let report = report.await.unwrap();
    assert!(
        report.failures.is_empty(),
        "the open report degraded: {:?}",
        report.failures
    );
    assert!(report.rendered > 0, "the open report rendered");
    // Both Cycles Settle Once The Held Steps Are Released.
    std::fs::write(&release, b"").unwrap();
    assert_eq!(settle(&handle, &held.command_id).await, RunPhase::Completed);
    assert_eq!(
        settle(&handle, &case_run.command_id).await,
        RunPhase::Completed
    );
    drop(handle);
    runtime.shutdown().await.unwrap();
}

/* === Private Functions === */
async fn start(handle: &RuntimeHandle, suite: &TestSuite, id: &str) -> TestCycleId {
    handle
        .start_cycle(
            CycleDefinition {
                suite: suite.clone(),
                metadata: Metadata::new(Identifier::new(id).unwrap(), id, None),
                cases: materialize_cases("fixture.checks", suite),
                continue_on_failure: false,
            },
            CommandContext::default(),
        )
        .await
        .unwrap()
}

async fn settle(handle: &RuntimeHandle, command: &dokime::core::CommandId) -> RunPhase {
    tokio::time::timeout(SETTLE, async {
        loop {
            let receipt = handle.run_request(command).await.unwrap().unwrap();
            if receipt.summary.phase.terminal() {
                break receipt.summary.phase;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("the run request settles")
}

fn assert_within(started: Instant, deadline: Duration, read: &str) {
    let elapsed = started.elapsed();
    assert!(
        elapsed < deadline,
        "{read} took {elapsed:?}, past the {deadline:?} interactive deadline"
    );
}

// The pulse plot as one live card requests it; its markers read the cycle's
// records from storage, so every follow-up holds a storage view.
fn pulse_request(index: usize, scope: RecordScope) -> PlotPayloadRequest {
    PlotPayloadRequest {
        plot_id: Some(format!("{PULSE_PLOT}.{index}")),
        stream: "fixture-suites.pulse".to_owned(),
        y_keys: vec!["value".to_owned()],
        marker_record_types: vec!["test.cycle.started".to_owned()],
        max_points: Some(400),
        scope: Some(scope),
        ..Default::default()
    }
}

// A live plot watch: a first snapshot, then follow-ups from its cursor, as
// the plot stream serves a card until the operator closes it.
async fn watch(
    handle: RuntimeHandle,
    mut request: PlotPayloadRequest,
    stop: Arc<AtomicBool>,
    first_payloads: Arc<AtomicUsize>,
) -> WatchOutcome {
    let mut outcome = WatchOutcome::default();
    while !stop.load(Ordering::Relaxed) {
        let mode = if request.cursor.is_some() {
            RuntimePlotQueryMode::Append
        } else {
            RuntimePlotQueryMode::Snapshot
        };
        match handle
            .query_plot_payloads(vec![request.clone()], mode)
            .await
        {
            Ok(envelopes) => {
                if outcome.served == 0 {
                    first_payloads.fetch_add(1, Ordering::Relaxed);
                }
                outcome.served += 1;
                if let Some(envelope) = envelopes.into_iter().next() {
                    request.cursor = Some(envelope.cursor);
                }
            }
            Err(error) => outcome.failures.push(error.to_string()),
        }
        tokio::time::sleep(PLOT_STREAM_THROTTLE).await;
    }
    outcome
}

// The open report of the running case, rendered again and again.
async fn report_loop(
    handle: RuntimeHandle,
    execution: TestExecutionId,
    stop: Arc<AtomicBool>,
) -> ReportOutcome {
    let mut outcome = ReportOutcome::default();
    while !stop.load(Ordering::Relaxed) {
        match handle.open_execution_report(&execution).await {
            Ok(opened) => {
                outcome.rendered += 1;
                assert!(!opened.html.is_empty());
            }
            Err(error) => outcome.failures.push(error.to_string()),
        }
    }
    outcome
}

/* === Tests === */
