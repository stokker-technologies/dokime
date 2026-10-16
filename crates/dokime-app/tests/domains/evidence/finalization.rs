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
use super::*;

/* === Constants === */
/// Ample for one interactive point read, but below what materializing Mini's
/// full report costs in a debug build, so a finalization that borrowed the
/// interactive query budget would fail.
const TIGHT_QUERY_BUDGET: Duration = Duration::from_millis(200);
const STEADY_SUITE: &str = "mini.charger.steady";
/// The steady suite sweeps three voltages across three loads.
const STEADY_CASES: usize = 9;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

// Finalization runs under its own budget: Mini's nine-case steady cycle seals
// every report although the interactive query budget is starved. Whether a
// simulated case passes is Mini's concern; every one must seal.
pub(crate) async fn tight_budget_seals() {
    let root = tempdir().unwrap();
    let mut tight = config(root.path()).with_plugins([fixture_bundle(root.path(), "mini-sidecar")]);
    tight.storage.interactive.timeout = TIGHT_QUERY_BUDGET;
    tight.storage.interactive.timeout = TIGHT_QUERY_BUDGET;
    let (mut owner, runtime) = started(tight).await;
    let registration = runtime
        .test_suites()
        .await
        .into_iter()
        .find(|suite| suite.source == "mini-sidecar" && suite.suite_id == STEADY_SUITE)
        .expect("Mini's steady suite is registered");
    let cases = materialize_cases(&registration.suite_id, &registration.suite);
    assert_eq!(cases.len(), STEADY_CASES);
    let test_cycle_id = runtime
        .start_cycle(
            CycleDefinition {
                metadata: metadata("fixture.finalization", "Finalization Budget"),
                suite: registration.suite,
                cases,
                continue_on_failure: true,
            },
            CommandContext::default(),
        )
        .await
        .unwrap();
    let receipt = runtime
        .run_cycle_cases(test_cycle_id.clone(), CommandContext::default())
        .await;
    assert_eq!(receipt.status, CommandStatus::Accepted, "{receipt:?}");

    // The operator accepts every steady inspection until the cycle settles.
    let settled = tokio::time::timeout(Duration::from_secs(300), async {
        loop {
            for prompt in runtime.pending_prompt_projection().await {
                let _ = runtime
                    .submit_prompt_response(
                        prompt.prompt_id.clone(),
                        "submit".to_owned(),
                        "operator".to_owned(),
                        BTreeMap::from([
                            ("operator_note".to_owned(), Value::from("steady")),
                            ("accepted".to_owned(), Value::from(true)),
                        ]),
                        true,
                    )
                    .await;
            }
            let cycle = cycle_snapshot(&runtime, &test_cycle_id).await;
            if cycle.saved.is_some() || cycle.lifecycle == CycleStatus::Failed {
                break cycle;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .expect("the steady cycle settles");
    let failure = runtime
        .run_request(&receipt.command_id)
        .await
        .unwrap()
        .and_then(|request| request.interruption.clone());
    assert!(
        settled.saved.is_some(),
        "the cycle report is sealed: {:?} {failure:?}",
        settled.lifecycle
    );
    assert_eq!(settled.test_executions.len(), STEADY_CASES);
    assert!(
        settled
            .test_executions
            .iter()
            .all(|execution| execution.saved.is_some()),
        "every steady execution seals: {:?}",
        settled
            .test_executions
            .iter()
            .map(|execution| (execution.status.clone(), execution.saved.is_some()))
            .collect::<Vec<_>>()
    );
    drop(runtime);
    owner.shutdown().await.unwrap();
}
