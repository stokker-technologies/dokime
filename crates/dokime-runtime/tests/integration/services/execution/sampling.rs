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
use std::path::Path;
use std::sync::Arc;

// Third Party Imports
use tempfile::tempdir;

// Local Imports
use super::smoke::{Behaviour, PULSE_DISPLAY_POINTS, PULSE_PLOT, PULSE_STREAM, SOURCE, SUITE, Smoke};
use crate::service_support::prelude::*;
use dokime_domain::model::execution::result::CriterionStatus;
use dokime_domain::model::execution::result::ExecutionStatus;
use dokime_runtime::definition::TestCase;
use dokime_runtime::runtime::command::{CommandContext, CommandStatus};
use dokime_runtime::runtime::progress::{LiveOperation, OperationOutcome};
use dokime_runtime::runtime::query::RuntimeChange;
use dokime_runtime::{CommandId, Identifier, Metadata, TestCycleId, TestExecutionId};
use {dokime_domain::model::execution::request::RunPhase, dokime_domain::model::execution::request::RunReceipt};

/* === Constants === */
/// Rows each case pushes onto its own stream: many storage pages per case.
const PULSE_ROWS: u32 = 640;

/// Cases run back to back in one store. Before the pager kept its cursor in
/// the index range, every page of a later case rescanned the earlier cases'
/// rows and the terminal capture exhausted its query time by the third case.
const CASES: usize = 6;

/// Rows one large case pushes: more than the report could hold as samples
/// under its 32 MiB materialization budget.
const LARGE_PULSE_ROWS: u32 = 20_000;

/// Points an operator sets on the live pulse plot, above what it declares.
const LIVE_DISPLAY_POINTS: u32 = 1_500;

/// Rows of the stream the frequency plot transforms whole: many storage
/// pages, yet a fraction of the 32 MiB report budget.
const SPECTRUM_ROWS: u32 = 1_024;

/// Rows of the whole-capture stream that alone exceed the 32 MiB report budget.
const OVERSIZED_SPECTRUM_ROWS: u32 = 8_192;

/// How a report too large for its whole budget names what it ran out of.
const EXHAUSTED: &str = "memory";

/// Members of the cycle whose every report the constrained pool refuses.
const POOLED_MEMBERS: usize = 9;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */
/// A runtime serving the in-process smoke check with `behaviour`.
async fn smoke_runtime(root: &Path, behaviour: Behaviour, constrained: bool) -> (Runtime, RuntimeHandle, Arc<Behaviour>) {
    let smoke = Smoke::new(behaviour);
    let behaviour = smoke.behaviour();
    let mut config = config(root).register(Arc::new(smoke));
    if constrained {
        constrain_report_memory(&mut config);
    }
    let (runtime, handle) = started(config).await;
    (runtime, handle, behaviour)
}

async fn smoke_case(handle: &RuntimeHandle) -> TestCase {
    let suite = handle.test_suites().await.into_iter().find(|suite| suite.source == SOURCE).expect("the smoke suite is registered").suite;
    materialize_cases(SUITE, &suite).remove(0)
}

async fn start_smoke(handle: &RuntimeHandle, label: &str, members: usize) -> TestCycleId {
    let case = smoke_case(handle).await;
    start_cases(handle, Metadata::new(Identifier::new(label).unwrap(), label, None), vec![case; members], false).await
}

async fn run(handle: &RuntimeHandle, cycle_id: &TestCycleId) -> CommandId {
    let receipt = handle.run_cycle_cases(cycle_id.clone(), CommandContext::default()).await;
    assert_eq!(receipt.status, CommandStatus::Accepted, "{receipt:?}");
    receipt.command_id
}

async fn settled(handle: &RuntimeHandle, run: &CommandId) -> Arc<RunReceipt> {
    wait_for_run_settled(handle, run).await;
    handle.run_request(run).await.unwrap().unwrap()
}

/// Waits Until Every Row The Step Pushed Onto `stream` Is Durable.
async fn rows_saved(handle: &RuntimeHandle, cycle_id: &TestCycleId, stream: &str, rows: u32) {
    let selection = TelemetrySelection {
        streams: vec![Identifier::new(stream).unwrap()],
        scope: ScopeSelection { test_cycle_id: Some(cycle_id.clone()), ..ScopeSelection::default() },
        ..TelemetrySelection::default()
    };
    until_telemetry(handle, || async { (handle.selected_telemetry(selection.clone()).await.len() >= rows as usize).then_some(()) }).await;
}

/// Runs One Case To Completion And Checks Its Criterion Saw Every Row Once.
async fn run_case(handle: &RuntimeHandle, label: &str) -> TestExecutionId {
    let cycle_id = start_smoke(handle, label, 1).await;
    let command = run(handle, &cycle_id).await;
    let receipt = settled(handle, &command).await;
    assert_eq!(receipt.summary.phase, RunPhase::Completed, "{label}: {:?}", receipt.interruption);
    let execution = cycle_snapshot(handle, &cycle_id).await.test_executions.remove(0);
    assert_eq!(execution.status, ExecutionStatus::Passed, "{label}");
    let criteria = execution.steps.iter().flat_map(|step| step.criteria.iter()).collect::<Vec<_>>();
    assert_eq!(criteria.len(), 1, "{label}");
    assert_eq!(criteria[0].status, CriterionStatus::Passed, "{label}: the run was handed back every pulse row exactly once");
    execution.test_execution_id
}

async fn report_payload(handle: &RuntimeHandle, execution: &TestExecutionId) -> serde_json::Value {
    let opened = handle.open_execution_report(execution).await.unwrap();
    crate::service_support::reports::parse(std::str::from_utf8(&opened.html).unwrap())
}

fn assert_live_plot(payload: &serde_json::Value, plot: &str, attempt: usize) {
    let plot = &payload["case_runs"][0]["plots"][plot];
    assert_eq!(
        plot["max_points"],
        serde_json::json!(LIVE_DISPLAY_POINTS),
        "render {attempt} uses the live plot's points: {}",
        payload["warnings"]
    );
    let series = &plot["series"][0];
    assert_eq!(series["raw_points"], serde_json::json!(LARGE_PULSE_ROWS));
    let points = series["points"].as_array().unwrap().len();
    assert!(
        points <= LIVE_DISPLAY_POINTS as usize && points > PULSE_DISPLAY_POINTS as usize,
        "render {attempt} draws the live plot's {LIVE_DISPLAY_POINTS} points, not the authored {PULSE_DISPLAY_POINTS}: {points}"
    );
    // The frozen summary covers every pulse row, 0 to n - 1, not the drawn
    // points: the numbers the live table shows over the same window.
    let summaries = plot["summaries"].as_array().unwrap();
    assert_eq!(summaries.len(), 1, "one summary per series: {summaries:?}");
    let summary = &summaries[0];
    assert_eq!(summary["series_id"], series["key"]);
    assert_eq!(summary["count"], serde_json::json!(LARGE_PULSE_ROWS));
    let rows = f64::from(LARGE_PULSE_ROWS);
    for (statistic, expected) in [
        ("min", 0.0),
        ("max", rows - 1.0),
        ("mean", (rows - 1.0) / 2.0),
        ("median", (rows - 1.0) / 2.0),
        ("std", ((rows * rows - 1.0) / 12.0).sqrt()),
        ("rms", ((rows - 1.0) * (2.0 * rows - 1.0) / 6.0).sqrt()),
    ] {
        let actual = summary[statistic].as_f64().unwrap();
        assert!((actual - expected).abs() <= 1e-9 * expected.abs().max(1.0), "render {attempt} {statistic}: {actual} != {expected}");
    }
    let window = &summary["window"];
    assert!(window["from"].as_f64().unwrap() <= window["to"].as_f64().unwrap());
}

fn constrain_report_memory(config: &mut dokime_runtime::RuntimeConfig) {
    let memory = std::num::NonZeroUsize::new(32 * 1024 * 1024).unwrap();
    config.storage.budget.read_memory_bytes = memory;
    config.storage.response.batch_bytes = std::num::NonZeroUsize::new(512 * 1024).unwrap();
}

/* === Tests === */
// cases whose own stream spans many pages replay and seal independent of history.
#[tokio::test]
async fn paged_evidence_settles() {
    let root = tempdir().unwrap();
    let behaviour = Behaviour::new(PULSE_ROWS, 0, false);
    let (mut runtime, handle, _) = smoke_runtime(root.path(), behaviour, false).await;
    for case in 0..CASES {
        run_case(&handle, &format!("pulse.case{case}")).await;
    }
    drop(handle);
    runtime.shutdown().await.unwrap();
}

// one case far larger than the report budget seals a reduced plot over every row.
#[tokio::test]
async fn large_case_seals() {
    let root = tempdir().unwrap();
    let behaviour = Behaviour::new(LARGE_PULSE_ROWS, 0, false);
    let (mut runtime, handle, _) = smoke_runtime(root.path(), behaviour, false).await;
    let execution_id = run_case(&handle, "pulse.large").await;
    let html = crate::service_support::native::report(&handle, &execution_id).await;
    let payload = crate::service_support::reports::parse(&html);
    let series = &payload["case_runs"][0]["plots"][PULSE_PLOT]["series"][0];
    let points = series["points"].as_array().expect("the sealed plot carries its points");
    assert!(
        !points.is_empty() && points.len() as u32 <= PULSE_DISPLAY_POINTS,
        "the sealed plot shows {} points against its declared {PULSE_DISPLAY_POINTS}",
        points.len()
    );
    assert_eq!(series["raw_points"], serde_json::json!(LARGE_PULSE_ROWS), "the plot was reduced from every captured row, none truncated");
    assert_eq!(series["downsampled"], serde_json::json!(true));
    drop(handle);
    runtime.shutdown().await.unwrap();
}

// a plot that needs its whole capture keeps only its own stream's rows, so a
// case whose other stream would overflow the report budget still seals both.
#[tokio::test]
async fn whole_capture_seals() {
    let root = tempdir().unwrap();
    let behaviour = Behaviour::new(LARGE_PULSE_ROWS, SPECTRUM_ROWS, false);
    let (mut runtime, handle, _) = smoke_runtime(root.path(), behaviour, false).await;
    let execution_id = run_case(&handle, "spectrum.whole").await;
    let html = crate::service_support::native::report(&handle, &execution_id).await;
    let payload = crate::service_support::reports::parse(&html);
    let plots = &payload["case_runs"][0]["plots"];
    assert_eq!(
        plots[PULSE_PLOT]["series"][0]["raw_points"],
        serde_json::json!(LARGE_PULSE_ROWS),
        "the streamed plot was reduced from every pulse row"
    );
    assert_eq!(
        plots["fixture.checks.smoke.spectrum"]["series"][0]["raw_points"],
        serde_json::json!(SPECTRUM_ROWS),
        "the whole-capture plot read every spectrum row across its pages"
    );
    drop(handle);
    runtime.shutdown().await.unwrap();
}

// a running execution's report follows the plot settings the operator chose on
// the live plot, renders again and again from the same evidence, and the
// sealed report keeps those settings.
#[tokio::test]
async fn running_report_follows() {
    let root = tempdir().unwrap();
    let behaviour = Behaviour::new(LARGE_PULSE_ROWS, 0, true);
    let (mut runtime, handle, behaviour) = smoke_runtime(root.path(), behaviour, false).await;
    let cycle_id = start_smoke(&handle, "pulse.live", 1).await;
    let command = run(&handle, &cycle_id).await;
    let execution = handle.watch_state().latest().state().cycles[&cycle_id].execution_ids[0].clone();
    rows_saved(&handle, &cycle_id, PULSE_STREAM, LARGE_PULSE_ROWS).await;
    let payload = report_payload(&handle, &execution).await;
    assert_eq!(
        payload["case_runs"][0]["plots"][PULSE_PLOT]["series"][0]["raw_points"],
        serde_json::json!(LARGE_PULSE_ROWS),
        "the running report reads every saved pulse row"
    );

    handle
        .update_plot_settings(
            &cycle_id,
            &execution,
            &dokime_runtime::DocumentId::from_identifier("default").unwrap(),
            &Identifier::new(PULSE_PLOT).unwrap(),
            Some(LIVE_DISPLAY_POINTS),
            dokime_runtime::document::DownsampleTechnique::Lttb,
        )
        .await
        .unwrap();
    for attempt in 1..=3 {
        let payload = report_payload(&handle, &execution).await;
        assert_live_plot(&payload, PULSE_PLOT, attempt);
        let steps = payload["test_executions"][0]["steps"].as_array().unwrap();
        assert!(
            steps.iter().any(|row| row["record_type"] == "step" && row["status"] == "running"),
            "render {attempt} carries the running step: {steps:?}"
        );
    }

    behaviour.release();
    assert_eq!(settled(&handle, &command).await.summary.phase, RunPhase::Completed);
    let html = crate::service_support::native::report(&handle, &execution).await;
    assert_live_plot(&crate::service_support::reports::parse(&html), PULSE_PLOT, 4);
    drop(handle);
    runtime.shutdown().await.unwrap();
}

// opening the report of a sizeable running cycle shows on the runtime state
// watch as one operation: it starts, moves through its phases and finishes.
#[tokio::test]
async fn report_shows_progress() {
    let root = tempdir().unwrap();
    let behaviour = Behaviour::new(LARGE_PULSE_ROWS, 0, true);
    let (mut runtime, handle, behaviour) = smoke_runtime(root.path(), behaviour, false).await;
    let cycle_id = start_smoke(&handle, "pulse.progress", 1).await;
    let command = run(&handle, &cycle_id).await;
    rows_saved(&handle, &cycle_id, PULSE_STREAM, LARGE_PULSE_ROWS).await;

    let mut watch = handle.watch_state();
    let earlier: std::collections::BTreeSet<String> =
        watch.operations().operations().iter().map(|operation| operation.operation_id.clone()).collect();
    let observer = tokio::spawn(async move {
        let mut seen = Vec::<LiveOperation>::new();
        loop {
            let snapshot = match watch.next_change().await {
                Some(RuntimeChange::Operations(snapshot)) => snapshot,
                Some(RuntimeChange::State(_)) => watch.operations(),
                None => break seen,
            };
            let Some(operation) = snapshot
                .operations()
                .iter()
                .find(|operation| operation.label == "Materializing Cycle Report" && !earlier.contains(&operation.operation_id))
            else {
                continue;
            };
            if seen.last() != Some(operation) {
                seen.push(operation.clone());
            }
            if operation.outcome != OperationOutcome::Running {
                break seen;
            }
        }
    });
    let opened = handle.open_cycle_report(&cycle_id).await.unwrap();
    assert!(!opened.html.is_empty());
    let seen = observer.await.unwrap();

    let identities: std::collections::BTreeSet<_> = seen.iter().map(|operation| &operation.operation_id).collect();
    assert_eq!(identities.len(), 1, "one operation opened the report: {seen:?}");
    assert_eq!(
        seen.first().map(|operation| &operation.outcome),
        Some(&OperationOutcome::Running),
        "the watch shows the operation start: {seen:?}"
    );
    let finished = seen.last().expect("the watch shows the operation");
    assert_eq!(finished.phases.last(), Some(&finished.phase), "the history ends at the current phase: {finished:?}");
    assert!(finished.phases.len() >= 2, "the operation changed phase: {:?}", finished.phases);
    for phase in ["Capturing Evidence", "Materializing Evidence", "Rendering"] {
        assert!(finished.phases.iter().any(|passed| passed == phase), "the history keeps {phase}: {:?}", finished.phases);
    }
    assert!(
        matches!(seen.last().map(|operation| &operation.outcome), Some(OperationOutcome::Finished { .. })),
        "the watch shows the operation finish: {seen:?}"
    );

    // Retention Is Time Semantics: Advance The Paused Clock Past It.
    let mut operations = handle.watch_state();
    tokio::time::pause();
    tokio::time::advance(dokime_runtime::runtime::progress::SETTLED_RETENTION * 2).await;
    tokio::time::resume();
    let retired = loop {
        let latest = operations.operations();
        if latest.operations().iter().all(|operation| !identities.contains(&operation.operation_id)) {
            break true;
        }
        operations.next_change().await.expect("the runtime state watch closes only with the runtime");
    };
    assert!(retired, "a finished operation retires after its retention");

    behaviour.release();
    settled(&handle, &command).await;
    drop(handle);
    runtime.shutdown().await.unwrap();
}

// a permanent materialization refusal leaves evidence and the execution unsealed.
#[tokio::test]
async fn oversized_report_unsealed() {
    let root = tempdir().unwrap();
    let behaviour = Behaviour::new(0, OVERSIZED_SPECTRUM_ROWS, true);
    let (mut runtime, handle, behaviour) = smoke_runtime(root.path(), behaviour, true).await;
    let cycle_id = start_smoke(&handle, "spectrum.oversized", 1).await;
    run(&handle, &cycle_id).await;
    let execution = handle.watch_state().latest().state().cycles[&cycle_id].execution_ids[0].clone();

    // The Report Fails Once The Whole Capture Outgrows The Budget; The Budget
    // Also Refuses Record Subscriptions, So Re-Check On Each State Change.
    let failure = settle(&handle, || async {
        handle.storage().flush().await.ok()?;
        handle.open_execution_report(&execution).await.err().map(|error| error.to_string())
    })
    .await;
    assert!(failure.contains(EXHAUSTED) || failure.contains("exhaust"), "{failure}");
    behaviour.release();
    let cycle = cycle_snapshot(&handle, &cycle_id).await;
    assert!(cycle.saved.is_none());
    assert!(cycle.test_executions.iter().all(|execution| execution.saved.is_none()));
    assert!(handle.open_execution_report(&execution).await.is_err());
    drop(handle);
    assert!(runtime.shutdown().await.is_err(), "shutdown reports unfinished materialization");
}

// a refused pool never publishes a degraded cycle report. Members are
// materialized one at a time, so the refusal is each member's own
// whole-capture read exceeding the constrained report budget.
#[tokio::test]
async fn pool_refusal_settles() {
    let root = tempdir().unwrap();
    let behaviour = Behaviour::new(0, OVERSIZED_SPECTRUM_ROWS, false);
    let (mut runtime, handle, _) = smoke_runtime(root.path(), behaviour, true).await;
    let cycle_id = start_smoke(&handle, "spectrum.refused", POOLED_MEMBERS).await;
    let command = run(&handle, &cycle_id).await;
    let receipt = settled(&handle, &command).await;
    assert_ne!(receipt.summary.phase, RunPhase::Completed, "permanent refusal cannot complete a report");
    let cause = format!("{:?}", receipt.interruption);
    assert!(cause.contains(EXHAUSTED) || cause.contains("exhaust"), "the run names the refused pool: {cause}");
    let cycle = cycle_snapshot(&handle, &cycle_id).await;
    assert!(cycle.saved.is_none(), "refused report remains unsealed");
    assert!(handle.open_cycle_report(&cycle_id).await.is_err());
    drop(handle);
    // The Refusal Is A Clean Failure: Nothing Remains Stuck To Report At Shutdown.
    runtime.shutdown().await.unwrap();
}
