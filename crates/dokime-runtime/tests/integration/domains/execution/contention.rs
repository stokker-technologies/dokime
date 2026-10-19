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
use std::sync::Arc;

// Third Party Imports
use tokio::sync::Barrier;

// Local Imports
use crate::service_support::prelude::*;
use crate::service_support::smoke::{Behaviour, PULSE_STREAM, SOURCE, SUITE, Smoke};
use dokime_domain::model::execution::request::RunPhase;
use dokime_runtime::runtime::query::{PlotPayloadRequest, RuntimePlotQueryMode};

/* === Constants === */
const LIVE_PLOTS: usize = 15;
const PULSE_ROWS: u32 = 2_000;
const ROUNDS: usize = 2;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */
async fn start(handle: &RuntimeHandle, suite: &TestSuite, id: &str) -> TestCycleId {
    start_cases(handle, metadata(id, id), materialize_cases(SUITE, suite), false).await
}

/* === Tests === */
// Fifteen Plot Followers, A Report And Operator Admission Share The Incident Read Pool.
#[tokio::test]
async fn contended_reads_interactive() {
    let root = tempdir().unwrap();
    let provider = Smoke::new(Behaviour::new(PULSE_ROWS, 0, true));
    let behaviour = provider.behaviour();
    let mut configuration = config(root.path()).register(Arc::new(provider));
    configuration.storage.budget.read_memory_bytes = std::num::NonZeroUsize::new(256 * 1024 * 1024).unwrap();
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
        scope: ScopeSelection { test_execution_id: Some(execution.clone()), ..Default::default() },
        ..Default::default()
    };
    until_telemetry(&handle, || async { (handle.selected_telemetry(selection.clone()).await.len() == PULSE_ROWS as usize).then_some(()) })
        .await;
    // Storage accepts a row before the window shows it; the running report
    // reads only what is saved, so the accepted rows are flushed first.
    handle.storage().flush().await.unwrap();

    // Each Round Starts All Readers And The Operator Together; No Warmup Timer.
    let barrier = Arc::new(Barrier::new(LIVE_PLOTS + 2));
    let scope = RecordScope { test_cycle_id: Some(running.clone()), test_execution_id: Some(execution.clone()), ..Default::default() };
    let mut followers = Vec::new();
    for index in 0..LIVE_PLOTS {
        let handle = handle.clone();
        let barrier = barrier.clone();
        let scope = scope.clone();
        followers.push(tokio::spawn(async move {
            let mut request = PlotPayloadRequest {
                plot_id: Some(format!("fixture.checks.smoke.pulse.{index}")),
                stream: PULSE_STREAM.into(),
                y_keys: vec!["value".into()],
                marker_record_types: vec!["test.cycle.started".into()],
                max_points: Some(400),
                scope: Some(scope),
                ..Default::default()
            };
            for round in 0..ROUNDS {
                barrier.wait().await;
                let mode = if round == 0 { RuntimePlotQueryMode::Snapshot } else { RuntimePlotQueryMode::Append };
                let envelopes = handle.query_plot_payloads(vec![request.clone()], mode).await.unwrap();
                assert_eq!(envelopes.len(), 1, "plot {index} round {round}");
                request.cursor = Some(envelopes.into_iter().next().unwrap().cursor);
                barrier.wait().await;
            }
        }));
    }
    let report = {
        let handle = handle.clone();
        let barrier = barrier.clone();
        tokio::spawn(async move {
            for _ in 0..ROUNDS {
                barrier.wait().await;
                let opened = handle.open_execution_report(&execution).await.unwrap();
                assert!(!opened.html.is_empty());
                let payload = crate::service_support::reports::parse(std::str::from_utf8(&opened.html).unwrap());
                assert_eq!(
                    payload["case_runs"][0]["plots"]["fixture.checks.smoke.pulse"]["series"][0]["raw_points"],
                    serde_json::json!(PULSE_ROWS)
                );
                barrier.wait().await;
            }
        })
    };
    let mut case_run = None;
    for _ in 0..ROUNDS {
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
