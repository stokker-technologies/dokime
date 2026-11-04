/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Committed Running Work Remains Visible At Fixed History Cuts

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::service_support::prelude::*;
use dokime_domain::model::{
    execution::wire,
    query::{
        page::PageRequest,
        selection::{CycleSelection, ExecutionSelection},
        snapshot::SnapshotRef,
    },
};
use dokime_operations::query::{ListCycles, ListExecutions};

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

#[tokio::test]
async fn query_running_work() {
    let root = tempdir().unwrap();
    let executor = Arc::new(BlockingPublishingExecutor::new());
    let (_owner, runtime) = booted_with_fixture(root.path(), executor.clone()).await;
    let suite = plt400_suite();
    register_python_suite(&runtime, suite.clone()).await;
    let cycle = start_cycle(&runtime, suite).await;
    let receipt = runtime.start_next_execution(cycle.clone(), CommandContext::default()).await;
    assert_eq!(receipt.status, CommandStatus::Accepted);
    executor.wait_until_entered().await;
    let live = cycle_snapshot(&runtime, &cycle).await;
    let execution = &live.test_executions[0];
    assert_eq!(execution.status, ExecutionStatus::Running);

    let cycles = runtime
        .query_list_cycles(
            ListCycles {
                selection: CycleSelection {
                    cycle_ids: vec![cycle.to_string()],
                    ..Default::default()
                },
                page: PageRequest::default(),
                at: None,
            },
            usize::MAX,
        )
        .await
        .unwrap();
    let executions = runtime
        .query_list_executions(
            ListExecutions {
                selection: ExecutionSelection {
                    cycle_ids: vec![cycle.to_string()],
                    statuses: vec![wire::ExecutionStatus::Running],
                    ..Default::default()
                },
                page: PageRequest::default(),
                at: None,
            },
            usize::MAX,
        )
        .await
        .unwrap();
    runtime.pause_cycle(cycle.clone(), CommandContext::default()).await.unwrap();
    let paused = runtime
        .query_list_cycles(
            ListCycles {
                selection: CycleSelection {
                    cycle_ids: vec![cycle.to_string()],
                    statuses: vec![wire::CycleStatus::Paused],
                    ..Default::default()
                },
                page: PageRequest::default(),
                at: None,
            },
            usize::MAX,
        )
        .await
        .unwrap();
    assert_eq!(paused.rows.len(), 1);
    assert_eq!(paused.rows[0].running_count, 1);
    runtime.resume_cycle(cycle.clone(), CommandContext::default()).await.unwrap();
    executor.release();
    wait_for_cycle_outcome(&runtime, &cycle, ResultStatus::Passed).await;

    assert_eq!(cycles.rows.len(), 1);
    assert_eq!(cycles.rows[0].status, wire::CycleStatus::Running);
    assert_eq!((cycles.rows[0].running_count, executions.rows.len()), (1, 1));
    assert_eq!(executions.rows[0].execution_id, execution.test_execution_id.to_string());

    let frozen = runtime
        .query_list_cycles(
            ListCycles {
                selection: CycleSelection {
                    cycle_ids: vec![cycle.to_string()],
                    ..Default::default()
                },
                page: PageRequest::default(),
                at: Some(SnapshotRef::Token(cycles.snapshot.unwrap().token)),
            },
            usize::MAX,
        )
        .await
        .unwrap();
    assert_eq!(frozen.rows, cycles.rows);
    let frozen = runtime
        .query_list_executions(
            ListExecutions {
                selection: ExecutionSelection {
                    cycle_ids: vec![cycle.to_string()],
                    statuses: vec![wire::ExecutionStatus::Running],
                    ..Default::default()
                },
                page: PageRequest::default(),
                at: Some(SnapshotRef::Token(executions.snapshot.unwrap().token)),
            },
            usize::MAX,
        )
        .await
        .unwrap();
    assert_eq!(frozen.rows, executions.rows);
    let fresh = runtime
        .query_list_executions(
            ListExecutions {
                selection: ExecutionSelection {
                    cycle_ids: vec![cycle.to_string()],
                    ..Default::default()
                },
                page: PageRequest::default(),
                at: None,
            },
            usize::MAX,
        )
        .await
        .unwrap();
    assert_eq!(fresh.rows[0].status, wire::ExecutionStatus::Passed);
    let fresh = runtime
        .query_list_cycles(
            ListCycles {
                selection: CycleSelection {
                    cycle_ids: vec![cycle.to_string()],
                    ..Default::default()
                },
                page: PageRequest::default(),
                at: None,
            },
            usize::MAX,
        )
        .await
        .unwrap();
    assert_eq!(fresh.rows[0].running_count, 0);
}
