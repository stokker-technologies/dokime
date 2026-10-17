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

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */
/// A runtime whose in-process provider passes every case of the fixture suite.
async fn provider_runtime(root: &Path) -> (Runtime, RuntimeHandle) {
    let executor = Arc::new(FakePythonExecutor::fixed("passed"));
    let (owner, runtime) = booted_with_fixture(root, executor).await;
    register_python_suite(&runtime, fixture_suite(&["passed"])).await;
    (owner, runtime)
}

/// A fresh cycle of the fixture suite once no earlier cycle is running.
async fn provider_cycle(runtime: &RuntimeHandle, suffix: &str) -> TestCycleId {
    until_state(runtime, |state| state.cycle_views().all(|cycle| cycle.lifecycle != CycleStatus::Running)).await;
    let registration = runtime
        .test_suites()
        .await
        .into_iter()
        .find(|entry| entry.suite.metadata.identifier.as_str() == "fixture.suite")
        .expect("fixture suite registered");
    start_cases(
        runtime,
        metadata(&format!("fixture.cycle.{suffix}"), &format!("Fixture Cycle {suffix}")),
        materialize_cases(&registration.suite_id, &registration.suite),
        false,
    )
    .await
}

#[tokio::test]
async fn receipts_respect_idempotency() {
    let root = tempdir().unwrap();
    let (_owner, runtime) = provider_runtime(root.path()).await;
    let context = CommandContext { idempotency_key: Some("client-key-1".to_owned()), ..CommandContext::default() };
    let test_cycle_id = provider_cycle(&runtime, "idempotent").await;

    let first = runtime.run_cycle_cases(test_cycle_id.clone(), context.clone()).await;
    let duplicate = runtime.run_cycle_cases(test_cycle_id, context.clone()).await;
    let fresh_cycle_id = provider_cycle(&runtime, "fresh").await;
    let fresh = runtime.run_cycle_cases(fresh_cycle_id, CommandContext::default()).await;

    assert_eq!(first.status, CommandStatus::Accepted);
    assert_eq!(duplicate.command_id, first.command_id);
    assert_eq!(duplicate.accepted_at, first.accepted_at);
    assert_eq!(duplicate.job_id, first.job_id);
    assert_ne!(fresh.command_id, first.command_id);
}

#[tokio::test]
async fn accepted_provider_revision() {
    let root = tempdir().unwrap();
    let (_owner, runtime) = provider_runtime(root.path()).await;
    let test_cycle_id = provider_cycle(&runtime, "replay").await;
    let before = runtime.runtime_overview().await.unwrap().revision;
    let receipt = runtime.run_cycle_cases(test_cycle_id, CommandContext::default()).await;
    let revision = runtime.runtime_overview().await.expect("runtime overview query succeeds").revision;

    assert_eq!(receipt.status, CommandStatus::Accepted);
    assert!(receipt.runtime_revision > before);
    // Provider progress can commit while the caller fetches the next overview.
    assert!(revision >= receipt.runtime_revision);
}

#[tokio::test]
async fn refusals_refuses_processor() {
    let root = tempdir().unwrap();
    let (_owner, runtime) = provider_runtime(root.path()).await;
    let revision = || async { runtime.runtime_overview().await.expect("runtime overview query succeeds").revision };
    let before = revision().await;

    let rejected = runtime.run_cycle_cases(dokime_runtime::model::identity::new_id::<TestCycleId>(), CommandContext::default()).await;
    let after = revision().await;

    assert_eq!(rejected.status, CommandStatus::Rejected);
    assert_eq!(before, after, "refused commands must not be journaled");

    let accepted_cycle_id = provider_cycle(&runtime, "after-rejection").await;
    let accepted = runtime.run_cycle_cases(accepted_cycle_id, CommandContext::default()).await;

    assert_eq!(accepted.status, CommandStatus::Accepted);
    assert!(accepted.runtime_revision > rejected.runtime_revision);
    // Provider progress can commit while the caller fetches the next overview.
    assert!(revision().await >= accepted.runtime_revision);
}

/* === Tests === */
#[tokio::test]
async fn idempotency_survives_restart() {
    let root = tempdir().unwrap();
    let context = CommandContext { idempotency_key: Some("restart-key-1".to_owned()), ..CommandContext::default() };
    let definition = CycleDefinition { metadata: metadata("fixture.cycle", "Fixture Cycle"), continue_on_failure: false };

    let cycle_id = {
        let (mut runtime, handle) = booted(root.path()).await;
        let created = handle.create_cycle(definition.clone(), context.clone()).await;
        assert!(created.is_ok());
        let cycles = handle.runtime_overview().await.expect("runtime overview query succeeds").cycles;
        assert_eq!(cycles.len(), 1);
        let cycle_id = cycles[0].test_cycle_id.clone();
        drop(handle);
        runtime.shutdown().await.unwrap();
        cycle_id
    };

    let (mut runtime, handle) = booted(root.path()).await;
    let restored = handle.runtime_overview().await.unwrap().cycles;
    assert_eq!(restored.len(), 1);
    assert_eq!(restored[0].test_cycle_id, cycle_id);
    assert_eq!(restored[0].lifecycle, CycleStatus::Stopped);
    let fresh = handle.create_cycle(definition.clone(), context.clone()).await.unwrap();
    assert_eq!(fresh, cycle_id);
    assert_eq!(handle.create_cycle(definition, context).await.unwrap(), fresh);
    assert_eq!(handle.runtime_overview().await.expect("runtime overview query succeeds").cycles.len(), 1);
    drop(handle);
    runtime.shutdown().await.unwrap();
}
