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
use std::time::{Duration, Instant};

// Third Party Imports
use tempfile::tempdir;

// Local Imports
use crate::support::plugins::fixture_bundle;
use crate::support::prelude::*;
use dokime::core::{Identifier, Metadata};
use dokime::execution::ExecutionStatus;
use dokime_app::capabilities::execution::requests::RunPhase;
use dokime_app::execution::CriterionStatus;
use dokime_app::runtime::command::{CommandContext, CommandStatus};
use dokime_app::transactions::CycleDefinition;

/* === Constants === */
/// Rows each case pushes onto its own stream: many storage pages per case.
const PULSE_ROWS: u32 = 2_000;

/// Cases run back to back in one store. Before the pager kept its cursor in
/// the index range, every page of a later case rescanned the earlier cases'
/// rows and the terminal capture exhausted its query time by the third case.
const CASES: usize = 6;

/// How much slower the last case may settle than the first. Work that grows
/// with the store's history settles the sixth case several times slower.
const GROWTH: u32 = 3;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */
async fn run_case(handle: &RuntimeHandle, label: &str) -> Duration {
    let suite = handle
        .test_suites()
        .await
        .into_iter()
        .find(|suite| suite.source == "fixture-suites")
        .unwrap()
        .suite;
    let cycle_id = handle
        .start_cycle(
            CycleDefinition {
                suite: suite.clone(),
                metadata: Metadata::new(Identifier::new(label).unwrap(), label, None),
                cases: materialize_cases("fixture.checks", &suite),
                continue_on_failure: false,
            },
            CommandContext::default(),
        )
        .await
        .unwrap();
    let started = Instant::now();
    let receipt = handle
        .run_cycle_cases(cycle_id.clone(), CommandContext::default())
        .await;
    assert_eq!(receipt.status, CommandStatus::Accepted, "{receipt:?}");
    // A terminal capture refused by its query budget leaves the receipt
    // nonterminal for good, so the bound here is the failure, not a wait.
    let phase = tokio::time::timeout(Duration::from_secs(60), async {
        loop {
            let receipt = handle
                .run_request(&receipt.command_id)
                .await
                .unwrap()
                .unwrap();
            if receipt.summary.phase.terminal() {
                break receipt.summary.phase;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap_or_else(|_| panic!("{label} records its terminal provider state"));
    let settled = started.elapsed();
    assert_eq!(phase, RunPhase::Completed, "{label}");
    let execution = handle
        .cycles()
        .await
        .into_iter()
        .find(|cycle| cycle.test_cycle_id == cycle_id)
        .unwrap()
        .test_executions
        .remove(0);
    assert_eq!(execution.status, ExecutionStatus::Passed, "{label}");
    let criteria = execution
        .steps
        .iter()
        .flat_map(|step| step.criteria.iter())
        .collect::<Vec<_>>();
    assert_eq!(criteria.len(), 1, "{label}");
    assert_eq!(
        criteria[0].status,
        CriterionStatus::Passed,
        "{label}: the run was handed back every pulse row exactly once"
    );
    settled
}

/* === Tests === */
// cases whose own stream spans many pages replay and seal independent of history.
pub(crate) async fn paged_evidence_settles() {
    let root = tempdir().unwrap();
    let (mut runtime, handle) = started(
        config(root.path()).with_plugins([fixture_bundle(root.path(), "fixture-suites")
            .configured(
                "fixture-suites",
                serde_json::json!({ "pulse_rows": PULSE_ROWS }),
            )
            .unwrap()]),
    )
    .await;
    let mut settled = Vec::with_capacity(CASES);
    for case in 0..CASES {
        settled.push(run_case(&handle, &format!("pulse.case{case}")).await);
    }
    let first = settled[0];
    let last = settled[CASES - 1];
    assert!(
        last <= first * GROWTH,
        "the last case settled in {last:?} against the first case's {first:?}: \
         replay and terminal capture grew with the store's history ({settled:?})"
    );
    drop(handle);
    runtime.shutdown().await.unwrap();
}
