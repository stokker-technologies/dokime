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

// Third Party Imports

// Local Imports
use crate::support::prelude::*;
use dokime::Message;
use dokime_app::execution::CycleView;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[dokime::message]
#[derive(Clone, PartialEq)]
struct Signals {
    starts: u64,
    cancelled: bool,
}

/* === Enums === */
#[derive(Clone, Copy, PartialEq)]
enum CancelSettlement {
    Complete,
    Delayed,
    Refused,
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */
async fn wait_started(handle: &RuntimeHandle) {
    tokio::time::timeout(Duration::from_secs(5), async {
        while signals(handle).await.starts == 0 {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("the native execution starts");
}

async fn check_stopping_cycle(settle_mode: CancelSettlement) {
    let root = tempdir().unwrap();
    let mut runtime = fixture_runtime(root.path()).await;
    let handle = runtime.handle();
    let mode = match settle_mode {
        CancelSettlement::Complete => "cancel_complete",
        CancelSettlement::Delayed => "cancel_delayed",
        CancelSettlement::Refused => "cancel_refused",
    };
    let suite = fixture_suite(&handle, &[mode]).await;
    let cycle_id = handle
        .start_cycle(
            CycleDefinition {
                suite: suite.clone(),
                metadata: metadata("cycle.sidecar.cancel", "Sidecar Cancellation Cycle"),
                cases: materialize_cases("fixture.sidecar", &suite),
                continue_on_failure: false,
            },
            CommandContext::default(),
        )
        .await
        .unwrap();

    let receipt = handle
        .run_cycle_cases(cycle_id.clone(), CommandContext::default())
        .await;
    assert_eq!(receipt.status, CommandStatus::Accepted);
    wait_started(&handle).await;
    let cancellation_started = std::time::Instant::now();
    let budget = Duration::from_secs(if std::env::var("STKR_COVERAGE").as_deref() == Ok("1") {
        25
    } else {
        5
    });
    let stopped = tokio::time::timeout(budget, async {
        handle
            .stop_cycle(cycle_id.clone(), CommandContext::default())
            .await
            .unwrap();
        wait_for_cycle(&handle, &cycle_id, CycleStatus::Stopped).await
    })
    .await
    .expect("cancellation acknowledgement and terminal settlement exceed the budget");
    if settle_mode == CancelSettlement::Delayed {
        assert!(cancellation_started.elapsed() >= Duration::from_millis(1_500));
    }
    assert!(signals(&handle).await.cancelled);
    assert_eq!(stopped.test_executions.len(), 1);
    assert_eq!(
        stopped.test_executions[0].status,
        ExecutionStatus::Cancelled
    );
    if settle_mode != CancelSettlement::Refused {
        // A cancelled run's verdict is "we never found out", and that has to
        // survive to the record rather than being dropped with the outcome.
        let criteria = stopped.test_executions[0]
            .steps
            .iter()
            .flat_map(|step| step.criteria.iter())
            .collect::<Vec<_>>();
        assert_eq!(criteria.len(), 1);
        assert_eq!(
            criteria[0].status,
            dokime_app::execution::CriterionStatus::Indeterminate
        );
        assert_eq!(
            criteria[0].message.as_deref(),
            Some("cancelled before deciding")
        );
    }
    let execution_id = &stopped.test_executions[0].test_execution_id;
    let files = handle.execution_files(execution_id).await.unwrap();
    let log = files
        .iter()
        .find(|file| file.relative_path == "fixture.sidecar/stream.log")
        .expect("the cancellation settle log is materialized");
    let content = handle
        .execution_file_content(execution_id, log.relative_path.clone(), 0, 4_096)
        .await
        .unwrap()
        .expect("the cancellation settle log remains readable");
    let timestamped = String::from_utf8(content.bytes.clone()).unwrap();
    let log = without_timestamps(&timestamped);
    if settle_mode != CancelSettlement::Refused {
        assert_eq!(
            log,
            "[TRACE] trace detail\n[DEBUG] debug detail\n[INFO] fixture cancellation settled\n[WARN] warning\n[WARN] continued\n[ERROR] error detail\n[INFO] \n[UNKNOWN(99)] unknown detail\n"
        );
    } else {
        assert!(log.starts_with("[ERROR] "), "{log}");
        assert!(log.contains("run refused during cancellation"), "{log}");
    }

    drop(handle);
    runtime.shutdown().await.unwrap();
}

async fn fixture_runtime(root: &Path) -> Runtime {
    let runtime = Runtime::start(
        config(root)
            .with_installed_plugins([dokime_host::installed_plugin()])
            .with_plugins([fixture_bundle(root, "fixture-sidecar-protocol")]),
    )
    .await
    .unwrap();
    runtime
        .handle()
        .bind_host_telemetry_source(dokime_host::host_telemetry_collector(
            dokime_host::HostTelemetryConfig::default()
                .with_source(dokime_host::HostTelemetrySourceKind::Fixture),
        ));
    runtime
}

async fn fixture_suite(handle: &RuntimeHandle, modes: &[&str]) -> TestSuite {
    let mut suite = handle
        .test_suites()
        .await
        .into_iter()
        .find(|entry| entry.source == "fixture-sidecar")
        .unwrap()
        .suite;
    suite.test_case_templates.retain(|template| {
        modes
            .iter()
            .any(|mode| template.metadata.identifier.as_str() == format!("fixture.sidecar.{mode}"))
    });
    assert_eq!(suite.test_case_templates.len(), modes.len());
    suite
}

async fn signals(handle: &RuntimeHandle) -> Signals {
    let bytes = handle
        .operations()
        .call_encoded("fixture-sidecar.status", Vec::new())
        .await
        .unwrap();
    Signals::decode(bytes.as_slice()).unwrap()
}

fn without_timestamps(log: &str) -> String {
    log.lines()
        .map(|line| {
            let (timestamp, message) = line.split_once(' ').expect("timestamp precedes severity");
            assert_eq!(timestamp.len(), 24, "UTC milliseconds: {timestamp}");
            assert!(timestamp.ends_with('Z'));
            assert!(
                dokime::core::Timestamp::new(timestamp)
                    .unix_millis()
                    .is_some()
            );
            format!("{message}\n")
        })
        .collect()
}

async fn wait_for_cycle(
    handle: &RuntimeHandle,
    cycle_id: &TestCycleId,
    wanted: CycleStatus,
) -> CycleView {
    // These fixtures publish several executions through real storage I/O.
    // Allow parallel-suite contention; elapsed time is not the assertion.
    tokio::time::timeout(Duration::from_secs(30), async {
        loop {
            if let Some(cycle) = handle
                .cycles()
                .await
                .into_iter()
                .find(|cycle| &cycle.test_cycle_id == cycle_id && cycle.lifecycle == wanted)
            {
                return cycle;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("sidecar cycle reached terminal state")
}

/* === Tests === */

pub(crate) async fn abandoned_transfer_log() {
    let root = tempdir().unwrap();
    let mut runtime = fixture_runtime(root.path()).await;
    let handle = runtime.handle();
    let suite = fixture_suite(&handle, &["abandon"]).await;
    let cycle_id = handle
        .start_cycle(
            CycleDefinition {
                suite: suite.clone(),
                metadata: metadata("cycle.sidecar.abandon", "Sidecar Abandonment Cycle"),
                cases: materialize_cases("fixture.sidecar", &suite),
                continue_on_failure: false,
            },
            CommandContext::default(),
        )
        .await
        .unwrap();

    let receipt = handle
        .run_cycle_cases(cycle_id.clone(), CommandContext::default())
        .await;
    assert_eq!(receipt.status, CommandStatus::Accepted);
    let completed = wait_for_cycle(&handle, &cycle_id, CycleStatus::Completed).await;

    // Abandoning evidence is the producer's own statement, so the run keeps
    // the verdict its steps earned and the file becomes a line in the log.
    let execution = &completed.test_executions[0];
    assert_eq!(execution.status, ExecutionStatus::Passed);
    assert!(
        handle
            .artifacts()
            .await
            .expect("artifact query succeeds")
            .is_empty(),
        "an aborted transfer never opens an upload"
    );
    let execution_id = &execution.test_execution_id;
    let files = handle.execution_files(execution_id).await.unwrap();
    let log = files
        .iter()
        .find(|file| file.relative_path == "fixture.sidecar/stream.log")
        .expect("the abandonment is recorded in the execution's own log");
    let content = handle
        .execution_file_content(execution_id, log.relative_path.clone(), 0, 4_096)
        .await
        .unwrap()
        .expect("the stream log remains readable");
    assert_eq!(
        without_timestamps(&String::from_utf8(content.bytes.clone()).unwrap()),
        "[WARN] dropped execution artifact `partial.bin`: the producer aborted the transfer after 8 bytes\n"
    );

    drop(handle);
    runtime.shutdown().await.unwrap();
}

pub(crate) async fn stopping_cycle_execution() {
    for settle_mode in [
        CancelSettlement::Complete,
        CancelSettlement::Delayed,
        CancelSettlement::Refused,
    ] {
        check_stopping_cycle(settle_mode).await;
    }
}

pub(crate) async fn stopping_ignored_run_settles() {
    // === Start A Run That Never Answers Its Stop === //

    let root = tempdir().unwrap();
    let mut runtime = fixture_runtime(root.path()).await;
    let handle = runtime.handle();
    let suite = fixture_suite(&handle, &["cancel_ignored"]).await;
    let cycle_id = handle
        .start_cycle(
            CycleDefinition {
                suite: suite.clone(),
                metadata: metadata("cycle.sidecar.ignored", "Ignored Stop Cycle"),
                cases: materialize_cases("fixture.sidecar", &suite),
                continue_on_failure: false,
            },
            CommandContext::default(),
        )
        .await
        .unwrap();
    handle
        .run_cycle_cases(cycle_id.clone(), CommandContext::default())
        .await;
    wait_started(&handle).await;

    // === Stop And Wait Out The Settlement Budget === //

    let stopped = tokio::time::timeout(Duration::from_secs(90), async {
        handle
            .stop_cycle(cycle_id.clone(), CommandContext::default())
            .await
            .unwrap();
        loop {
            if let Some(cycle) = handle.cycles().await.into_iter().find(|cycle| {
                cycle.test_cycle_id == cycle_id && cycle.lifecycle == CycleStatus::Stopped
            }) {
                return cycle;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    })
    .await
    .expect("a run that ignores its stop still settles within the host budget");
    handle
        .operations()
        .call_encoded("fixture-sidecar.release", Vec::new())
        .await
        .unwrap();

    assert!(signals(&handle).await.cancelled);
    assert_eq!(
        stopped.test_executions[0].status,
        ExecutionStatus::Cancelled
    );
    assert!(
        stopped.saved.is_some(),
        "a host-settled stop still seals the cycle's final report"
    );
    assert!(handle.pending_prompt_projection().await.is_empty());

    drop(handle);
    runtime.shutdown().await.unwrap();
}

pub(crate) async fn execution_scope_error() {
    let root = tempdir().unwrap();
    let mut runtime = fixture_runtime(root.path()).await;
    let handle = runtime.handle();
    let suite = fixture_suite(&handle, &["scope_error"]).await;
    let cycle_id = handle
        .start_cycle(
            CycleDefinition {
                suite: suite.clone(),
                metadata: metadata("cycle.sidecar.error", "Sidecar Error Cycle"),
                cases: materialize_cases("fixture.sidecar", &suite),
                continue_on_failure: false,
            },
            CommandContext::default(),
        )
        .await
        .unwrap();
    let receipt = handle
        .run_cycle_cases(cycle_id.clone(), CommandContext::default())
        .await;
    assert_eq!(receipt.status, CommandStatus::Accepted);
    wait_started(&handle).await;
    let cycle = handle
        .cycles()
        .await
        .into_iter()
        .find(|cycle| cycle.test_cycle_id == cycle_id)
        .unwrap();
    let scope = &cycle.test_executions[0].scope;
    let leases = handle.stream_leases_for(scope).await;
    assert_eq!(leases.len(), 1);
    assert!(leases[0].closed_at.is_none());

    handle
        .operations()
        .call_encoded("fixture-sidecar.release", Vec::new())
        .await
        .unwrap();
    let failed = wait_for_cycle(&handle, &cycle_id, CycleStatus::Failed).await;
    assert_eq!(failed.test_executions[0].status, ExecutionStatus::Failed);
    assert!(signals(&handle).await.cancelled);
    let released = handle.stream_leases_for(scope).await;
    assert_eq!(released.len(), 1);
    assert!(released[0].closed_at.is_some());
    assert!(
        handle
            .watch_state()
            .latest()
            .state()
            .installed_plugin_errors
            .iter()
            .any(|error| error.code.as_deref() == Some("provider_execution_run"))
    );
    drop(handle);
    runtime.shutdown().await.unwrap();
}
