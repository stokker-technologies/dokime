/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Report Publication Is Independent Of Execution Settlement.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[tokio::test(flavor = "multi_thread")]
async fn report_background() {
    for fail in [false, true] {
        let root = tempdir().unwrap();
        let executor = Arc::new(BlockingPublishingExecutor::new());
        let (mut owner, runtime) = booted_with_fixture(root.path(), executor.clone()).await;
        let suite = custom_suite();
        register_custom_suite(&runtime, suite.clone()).await;
        let cycle = start_cycle(&runtime, suite).await;
        let execution = runtime.watch_state().latest().state().cycles[&cycle].execution_ids[0].clone();
        let gate = dokime_runtime::testing::hold_report(&execution, fail);
        let receipt = runtime.start_execution(execution.clone(), Default::default(), CommandContext::default()).await;
        assert_eq!(receipt.status, CommandStatus::Accepted);
        executor.wait_until_entered().await;
        runtime.stop_execution(execution.clone(), CommandContext::default()).await.unwrap();
        gate.entered().await;
        {
            let snapshot = runtime.watch_state().latest();
            assert!(snapshot.state().executions[&execution].finished_at.is_some());
            assert!(snapshot.state().executions[&execution].saved.is_none());
            let intent = &snapshot.state().report_finalizations[&execution];
            assert!(intent.lifecycle.is_some());
            assert!(intent.failure.is_none());
            assert!(snapshot.state().run_requests.values().all(|request| request.summary.phase.terminal()));
        }
        let selection = TelemetrySelection {
            scope: ScopeSelection {
                test_execution_id: Some(execution.clone()),
                ..Default::default()
            },
            ..Default::default()
        };
        let (_, closed) = runtime.telemetry_page_at(runtime.storage(), &selection, &runtime.storage().cursor_saved(), 0).await.unwrap();
        assert!(closed);
        let download = {
            let runtime = runtime.clone();
            let execution = execution.clone();
            tokio::spawn(async move { runtime.open_execution_report(&execution).await })
        };
        assert!(!download.is_finished());
        gate.release();
        assert_eq!(runtime.wait_execution_report(&execution).await.is_err(), fail);
        assert_eq!(download.await.unwrap().is_err(), fail);
        // An Already-Settled Stop Never Becomes A Report Error.
        runtime.stop_execution(execution, CommandContext::default()).await.unwrap();
        owner.shutdown().await.unwrap();
    }

    let root = tempdir().unwrap();
    let executor = Arc::new(BlockingPublishingExecutor::new());
    let (mut owner, runtime) = booted_with_fixture(root.path(), executor.clone()).await;
    let suite = custom_suite();
    register_custom_suite(&runtime, suite.clone()).await;
    let cycle = start_cycle(&runtime, suite).await;
    let execution = runtime.watch_state().latest().state().cycles[&cycle].execution_ids[0].clone();
    let gate = dokime_runtime::testing::hold_report(&execution, false);
    let receipt = runtime.start_execution(execution.clone(), Default::default(), CommandContext::default()).await;
    assert_eq!(receipt.status, CommandStatus::Accepted);
    executor.wait_until_entered().await;
    runtime.stop_execution(execution.clone(), CommandContext::default()).await.unwrap();
    gate.entered().await;
    owner.shutdown().await.unwrap();
    drop(runtime);
    drop(owner);
    let (mut owner, runtime) = booted_with_fixture(root.path(), Arc::new(BlockingPublishingExecutor::new())).await;
    // Startup Completes The Persisted Report Intent Without Replaying The Provider.
    let mut opened = runtime.open_execution_report(&execution).await.unwrap();
    assert!(!crate::service_support::reports::opened_bytes(&mut opened).await.is_empty());
    owner.shutdown().await.unwrap();
}
