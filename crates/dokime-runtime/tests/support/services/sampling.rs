/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// The Smoke-Provider Sampling Paths, Shared By Their Acceptance Guards And Full-Scale
// System Stories: Paged Replay, Whole-Capture Seals, Running Reports And Report Progress.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::path::Path;
use std::sync::Arc;

// Third Party Imports
use tempfile::tempdir;
use tokio::sync::Barrier;

// Local Imports
use super::prelude::*;
use super::smoke::{Behaviour, PULSE_DISPLAY_POINTS, PULSE_PLOT, PULSE_STREAM, SOURCE, SUITE, Smoke};
use super::system_reports::Runtime;
use dokime_domain::model::execution::result::CriterionStatus;
use dokime_domain::model::execution::result::ExecutionStatus;
use dokime_domain::model::identity::CommandId;
use dokime_domain::model::values::Value;
use dokime_runtime::definition::TestCase;
use dokime_runtime::runtime::command::{CommandContext, CommandStatus};
use dokime_runtime::runtime::progress::{LiveOperation, OperationOutcome};
use dokime_runtime::runtime::query::{PlotPayloadRequest, RuntimeChange, RuntimePlotQueryMode};
use dokime_runtime::{Identifier, Metadata, TestCycleId, TestExecutionId};
use {dokime_domain::model::execution::request::RunPhase, dokime_domain::model::execution::request::RunReceipt};

/* === Constants === */

const LIVE_DISPLAY_POINTS: u32 = 1_500;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

// One Size Of The Sampling Paths.
#[derive(Clone, Copy, Debug)]
pub struct Sampling {
    // Rows of each paged-replay case, and how many cases replay one after another.
    pub paged_rows: u32,
    pub cases: usize,
    // Rows of the one case that exceeds its display points.
    pub rows: u32,
    pub spectrum_rows: u32,
    // The storage page of the whole-capture story.
    pub page_bytes: usize,
}

// One Size Of The Contended Read Path.
#[derive(Clone, Copy, Debug)]
pub struct Contention {
    pub plots: usize,
    pub rows: u32,
    pub rounds: usize,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
// cases whose own stream spans many pages replay and seal independent of history.
pub async fn paged_evidence_settles(size: Sampling) {
    let root = tempdir().unwrap();
    let behaviour = Behaviour::new(size.paged_rows, 0, false);
    let (mut runtime, handle, _) = smoke_runtime(root.path(), behaviour, None).await;
    for case in 0..size.cases {
        run_case(&handle, &format!("pulse.case{case}")).await;
    }
    drop(handle);
    runtime.shutdown().await.unwrap();
}

// A Large Case Seals A Bounded Envelope While Retaining Its Exact Summary.
pub async fn large_case_seals(size: Sampling) {
    replay_diagnostics();
    let root = tempdir().unwrap();
    let behaviour = Behaviour::new(size.rows, 0, false);
    let (mut runtime, handle, _) = smoke_runtime(root.path(), behaviour, None).await;
    let execution_id = run_case(&handle, "pulse.large").await;
    let html = super::native::report(&handle, &execution_id).await;
    let payload = super::reports::parse(&html);
    let series = &payload["case_runs"][0]["plots"][PULSE_PLOT]["series"][0];
    let points = series["points"].as_array().expect("the sealed plot carries its points");
    assert!(size.rows > PULSE_DISPLAY_POINTS, "the series exceeds its declared display points");
    assert!(points.len() <= PULSE_DISPLAY_POINTS as usize);
    assert_eq!(series["raw_points"], serde_json::json!(size.rows));
    assert_eq!(series["downsampled"], serde_json::json!(true));
    assert_eq!(payload["case_runs"][0]["plots"][PULSE_PLOT]["max_points"], serde_json::json!(PULSE_DISPLAY_POINTS));
    drop(handle);
    runtime.shutdown().await.unwrap();
}

// Whole-History Analyses Are Explicitly Unavailable In Bounded Reports.
pub async fn whole_capture_seals(size: Sampling) {
    replay_diagnostics();
    let root = tempdir().unwrap();
    let behaviour = Behaviour::new(size.rows, size.spectrum_rows, false).artifacts(false);
    let (mut runtime, handle, _) = smoke_runtime(root.path(), behaviour, Some(size.page_bytes)).await;
    let execution_id = run_case(&handle, "spectrum.whole").await;
    let html = super::native::report(&handle, &execution_id).await;
    let payload = super::reports::parse(&html);
    assert_eq!(payload["snapshot"]["artifacts"].as_array().unwrap().len(), 2);
    let plots = &payload["case_runs"][0]["plots"];
    assert_eq!(
        plots[PULSE_PLOT]["series"][0]["raw_points"],
        serde_json::json!(size.rows),
        "the streamed plot was reduced from every pulse row"
    );
    let spectrum = &plots["fixture.checks.smoke.spectrum"];
    assert!(spectrum["series"].as_array().unwrap().is_empty());
    assert_eq!(spectrum["analysis"]["kind"], "error");
    drop(handle);
    runtime.shutdown().await.unwrap();
}

// a running execution's report follows the plot settings the operator chose on
// the live plot, renders again and again from the same evidence, and the
// sealed report keeps those settings.
pub async fn running_report_follows(size: Sampling) {
    let root = tempdir().unwrap();
    let behaviour = Behaviour::new(size.rows, 0, true);
    let (mut runtime, handle, behaviour) = smoke_runtime(root.path(), behaviour, None).await;
    let cycle_id = start_smoke(&handle, "pulse.live", 1).await;
    let command = run(&handle, &cycle_id).await;
    let execution = handle.watch_state().latest().state().cycles[&cycle_id].execution_ids[0].clone();
    rows_saved(&handle, &cycle_id, PULSE_STREAM, size.rows).await;
    let payload = report_payload(&handle, &execution).await;
    assert_eq!(
        payload["case_runs"][0]["plots"][PULSE_PLOT]["series"][0]["raw_points"],
        serde_json::json!(size.rows),
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
        assert_live_plot(&payload, PULSE_PLOT, attempt, size.rows);
        let steps = payload["test_executions"][0]["steps"].as_array().unwrap();
        assert!(
            steps.iter().any(|row| row["record_type"] == "step" && row["status"] == "running"),
            "render {attempt} carries the running step: {steps:?}"
        );
    }

    behaviour.release();
    let receipt = settled(&handle, &command).await;
    assert_eq!(receipt.summary.phase, RunPhase::Completed, "{:?}", *receipt);
    drop(receipt);
    let html = super::native::report(&handle, &execution).await;
    assert_live_plot(&super::reports::parse(&html), PULSE_PLOT, 4, size.rows);
    drop(handle);
    runtime.shutdown().await.unwrap();
}

// opening the report of a sizeable running cycle shows on the runtime state
// watch as one operation: it starts, moves through its phases and finishes.
pub async fn report_shows_progress(size: Sampling) {
    let root = tempdir().unwrap();
    let behaviour = Behaviour::new(size.rows, 0, true);
    let (mut runtime, handle, behaviour) = smoke_runtime(root.path(), behaviour, None).await;
    let cycle_id = start_smoke(&handle, "pulse.progress", 1).await;
    let command = run(&handle, &cycle_id).await;
    rows_saved(&handle, &cycle_id, PULSE_STREAM, size.rows).await;

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
    assert!(opened.file.length > 0);
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
    tokio::time::advance(std::time::Duration::from_millis(3_000) * 2).await;
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

// `plots` Plot Followers, A Report And Operator Admission Share The Incident Read Pool
// For `rounds` Rounds Over `rows` Saved Pulse Rows.
pub async fn contended_reads(size: Contention) {
    let root = tempdir().unwrap();
    let provider = Smoke::new(Behaviour::new(size.rows, 0, true));
    let behaviour = provider.behaviour();
    let configuration = provider.configure(config(root.path()), root.path());

    let (mut owner, handle) = started(configuration).await;
    let suite = handle.test_suites().await.into_iter().find(|suite| suite.source == SOURCE).unwrap().suite;
    let queued = start(&handle, &suite, "contention.queued").await;
    let case = cycle_snapshot(&handle, &queued).await.cases().next().unwrap().test_case_id.clone();
    let running = start(&handle, &suite, "contention.running").await;
    let held = handle.start_next_execution(running.clone(), CommandContext::default()).await;
    assert_eq!(held.status, CommandStatus::Accepted);
    let snapshot = until_state(&handle, |state| !state.cycles[&running].execution_ids.is_empty()).await;
    let execution = snapshot.state().cycles[&running].execution_ids[0].clone();
    let selection = TelemetrySelection {
        streams: vec![Identifier::new(PULSE_STREAM).unwrap().into()],
        scope: ScopeSelection {
            test_execution_id: Some(execution.clone()),
            ..Default::default()
        },
        ..Default::default()
    };
    until_telemetry(&handle, || async {
        (handle.selected_telemetry(selection.clone()).await.unwrap().len() == size.rows as usize).then_some(())
    })
    .await;
    // Plot And Report Queries Read Saved Evidence At An Explicit Durable Cut.
    handle.storage().flush().await.unwrap();

    // Each Round Starts All Readers And The Operator Together; No Warmup Timer.
    let barrier = Arc::new(Barrier::new(size.plots + 2));
    let scope = RecordScope {
        test_cycle_id: Some(running.clone()),
        test_execution_id: Some(execution.clone()),
        ..Default::default()
    };
    let mut followers = Vec::new();
    for index in 0..size.plots {
        let handle = handle.clone();
        let barrier = barrier.clone();
        let scope = scope.clone();
        followers.push(tokio::spawn(async move {
            let request = PlotPayloadRequest {
                plot_id: Some(format!("fixture.checks.smoke.pulse.{index}")),
                stream: PULSE_STREAM.into(),
                y_keys: vec!["value".into()],
                marker_record_types: vec!["test.cycle.started".into()],
                max_points: Some(400),
                scope: Some(scope),
                ..Default::default()
            };
            for round in 0..size.rounds {
                barrier.wait().await;
                let mode = if round == 0 { RuntimePlotQueryMode::Snapshot } else { RuntimePlotQueryMode::Replace };
                let at = handle.storage().cursor_saved();
                let envelopes = handle.query_plot_payloads_at(vec![request.clone()], mode, &at).await.unwrap();
                assert_eq!(envelopes.len(), 1, "plot {index} round {round}");
                let envelope = &envelopes[0];
                assert_eq!(envelope.revision, at.sequence);
                assert_eq!(envelope.payload.stream, PULSE_STREAM);
                assert_eq!(envelope.payload.series[0].raw_points, size.rows as usize);
                barrier.wait().await;
            }
        }));
    }
    let report = {
        let handle = handle.clone();
        let barrier = barrier.clone();
        tokio::spawn(async move {
            for _ in 0..size.rounds {
                barrier.wait().await;
                let mut opened = handle.open_execution_report(&execution).await.unwrap();
                assert!(opened.file.length > 0);
                let payload = super::reports::parse(std::str::from_utf8(&super::reports::opened_bytes(&mut opened).await).unwrap());
                assert_eq!(
                    payload["case_runs"][0]["plots"]["fixture.checks.smoke.pulse"]["series"][0]["raw_points"],
                    serde_json::json!(size.rows)
                );
                barrier.wait().await;
            }
        })
    };
    let mut case_run = None;
    for _ in 0..size.rounds {
        barrier.wait().await;
        if case_run.is_none() {
            let receipt = handle.start_case_execution(queued.clone(), case.clone(), CommandContext::default()).await;
            assert_eq!(receipt.status, CommandStatus::Accepted);
            case_run = Some(receipt);
        }
        for _ in 0..10 {
            handle.run_request(&case_run.as_ref().unwrap().command_id).await.unwrap().expect("admitted request stays queryable");
            assert!(handle.cycles().await.iter().any(|cycle| cycle.test_cycle_id == queued));
        }
        barrier.wait().await;
    }
    for follower in followers {
        follower.await.unwrap();
    }
    report.await.unwrap();
    behaviour.release();
    let case_run = case_run.unwrap();
    for command in [&held.command_id, &case_run.command_id] {
        wait_for_run_settled(&handle, command).await;
        assert_eq!(handle.run_request(command).await.unwrap().unwrap().summary.phase, RunPhase::Completed);
    }
    drop(handle);
    owner.shutdown().await.unwrap();
}

/* === Private Functions === */

fn replay_diagnostics() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("dokime_runtime::domains::execution::run=debug,dokime_runtime::execution::run=debug")
        .with_ansi(false)
        .try_init();
}

async fn smoke_runtime(root: &Path, behaviour: Behaviour, page_bytes: Option<usize>) -> (Runtime, RuntimeHandle, Arc<Behaviour>) {
    let smoke = Smoke::new(behaviour);
    let behaviour = smoke.behaviour();
    let mut config = smoke.configure(config(root), root);
    if let Some(bytes) = page_bytes {
        small_storage_pages(&mut config, bytes);
    }
    tracing::debug!(target: "dokime_runtime::domains::execution::run",
                batch_rows = config.storage.response.batch_rows.get(),
        batch_bytes = config.storage.response.batch_bytes.get(),
        active_queries = config.storage.request.max_active_queries.get(),
        "sampling fixture storage settings");
    let (runtime, handle) = started(config).await;
    (runtime, handle, behaviour)
}

async fn smoke_case(handle: &RuntimeHandle) -> TestCase {
    let suite = handle.test_suites().await.into_iter().find(|suite| suite.source == SOURCE).expect("the smoke suite is registered").suite;
    materialize_cases(SUITE, &suite).remove(0)
}

async fn start_smoke(handle: &RuntimeHandle, label: &str, members: usize) -> TestCycleId {
    let case = smoke_case(handle).await;
    start_cases(handle, Metadata::new(Identifier::new(label).unwrap(), label, None), vec![case; members]).await
}

async fn start(handle: &RuntimeHandle, suite: &TestSuite, id: &str) -> TestCycleId {
    start_cases(handle, metadata(id, id), materialize_cases(SUITE, suite)).await
}

async fn run(handle: &RuntimeHandle, cycle_id: &TestCycleId) -> CommandId {
    let receipt = handle.start_next_execution(cycle_id.clone(), CommandContext::default()).await;
    assert_eq!(receipt.status, CommandStatus::Accepted, "{receipt:?}");
    receipt.command_id
}

async fn settled(handle: &RuntimeHandle, run: &CommandId) -> impl std::ops::Deref<Target = RunReceipt> + use<> {
    wait_for_run_settled(handle, run).await;
    handle.run_request(run).await.unwrap().unwrap()
}

async fn rows_saved(handle: &RuntimeHandle, cycle_id: &TestCycleId, stream: &str, rows: u32) {
    let selection = TelemetrySelection {
        streams: vec![Identifier::new(stream).unwrap().into()],
        scope: ScopeSelection {
            test_cycle_id: Some(cycle_id.clone()),
            ..ScopeSelection::default()
        },
        ..TelemetrySelection::default()
    };
    // The live window holds only the latest rows, so a large case outgrows it.
    // Rows are ingested in the order the step pushed them, and each carries
    // its index, so the window holding the last row means every row is in.
    let last = Value::Number(f64::from(rows - 1));
    until_telemetry(handle, || async {
        handle
            .selected_telemetry(selection.clone())
            .await
            .unwrap()
            .iter()
            .any(|sample| sample.values.get("value") == Some(&last))
            .then_some(())
    })
    .await;
    // Storage accepts a row before the window shows it; a running report reads
    // only what is saved, so the accepted rows are flushed first.
    handle.storage().flush().await.unwrap();
}

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
    let mut opened = handle.open_execution_report(execution).await.unwrap();
    super::reports::parse(std::str::from_utf8(&super::reports::opened_bytes(&mut opened).await).unwrap())
}

fn assert_live_plot(payload: &serde_json::Value, plot: &str, attempt: usize, rows: u32) {
    let plot = &payload["case_runs"][0]["plots"][plot];
    assert_eq!(
        plot["max_points"],
        serde_json::json!(LIVE_DISPLAY_POINTS),
        "render {attempt} uses the live plot's points: {}",
        payload["warnings"]
    );
    let series = &plot["series"][0];
    assert_eq!(series["raw_points"], serde_json::json!(rows));
    // The Report Reads Durable Envelopes Within The Stated Display Budget.
    let points = series["points"].as_array().unwrap().len();
    assert!(points <= LIVE_DISPLAY_POINTS as usize, "render {attempt} exceeds its display budget");
    assert_eq!(series["downsampled"], serde_json::json!(true));
    // The frozen summary covers every pulse row, 0 to n - 1, not the drawn
    // points: the numbers the live table shows over the same window.
    let summaries = plot["summaries"].as_array().unwrap();
    assert_eq!(summaries.len(), 1, "one summary per series: {summaries:?}");
    let summary = &summaries[0];
    assert_eq!(summary["series_id"], series["key"]);
    assert_eq!(summary["count"], serde_json::json!(rows));
    let rows = f64::from(rows);
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

fn small_storage_pages(config: &mut dokime_runtime::RuntimeConfig, bytes: usize) {
    config.storage.response.batch_bytes = std::num::NonZeroUsize::new(bytes).unwrap();
}

/* === Tests === */
