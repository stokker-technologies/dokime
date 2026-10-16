/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Persisted Report Restart Story

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::num::NonZeroUsize;

// Third Party Imports
use harmos::storage::StorageBackend;

// Local Imports
use crate::support::prelude::*;
use dokime::operations::reports::{MaterializeReport, MaterializeReportRequest, ReportSubject};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(crate) async fn persisted_cycle_restart() {
    let root = tempdir().unwrap();
    let (mut owner, runtime) = booted_with_fixture(root.path(), Arc::new(PublishingExecutor)).await;
    let suite = fixture_suite(&["passed", "passed"]);
    register_custom_suite(&runtime, suite.clone()).await;
    let cycle_id = start_cycle(&runtime, suite, false).await;
    let source = cycle_snapshot(&runtime, &cycle_id).await;
    let first = source.test_executions[0].test_execution_id.clone();
    runtime
        .run_execution(
            first.clone(),
            dokime_app::execution::ExecutionMode::Automatic,
            CommandContext::default(),
        )
        .await;
    let checkpoint = crate::support::native::checkpoint(&runtime, &first).await;
    let before = cycle_snapshot(&runtime, &cycle_id).await;
    assert!(
        before.saved.is_none(),
        "the parent still has a pending execution"
    );
    assert_eq!(before.test_executions[1].status, ExecutionStatus::Pending);
    let seal = before.test_executions[0].saved.as_ref().unwrap().clone();
    let sealed_html = crate::support::reports::bytes(runtime.storage(), &seal.report).await;
    let input_bytes = crate::support::reports::bytes(runtime.storage(), &seal.source.context).await;
    let boundary = runtime.storage().cursor_saved();
    let report = runtime
        .operations()
        .call_as::<MaterializeReport>(MaterializeReportRequest {
            subject: ReportSubject::Cycle(cycle_id.to_string()),
            at: boundary.clone(),
        })
        .await
        .unwrap();
    let html = crate::support::reports::bytes(runtime.storage(), &report.report).await;
    assert!(!html.is_empty());
    assert_eq!(
        cycle_snapshot(&runtime, &cycle_id).await.test_executions[0]
            .saved
            .as_ref(),
        Some(&seal)
    );
    assert_eq!(
        seal.source.evidence_through,
        before.test_executions[0]
            .saved
            .as_ref()
            .unwrap()
            .source
            .evidence_through
    );
    drop((checkpoint, runtime));
    owner.shutdown().await.unwrap();

    // Active Parent History Must Survive Reclamation Before Restart.
    let backend = dokime::storage::Backend::primary(root.path().to_owned())
        .await
        .unwrap();
    let at = backend.open().await.unwrap();
    while !backend
        .prune_objects(&at, NonZeroUsize::new(128).unwrap())
        .await
        .unwrap()
    {}
    backend.close().await.unwrap();
    drop(backend);

    let (mut owner, runtime) = booted_with_fixture(root.path(), Arc::new(PublishingExecutor)).await;
    let recovered = cycle_snapshot(&runtime, &cycle_id).await;
    assert!(recovered.saved.is_none());
    assert_eq!(recovered.test_executions[0].saved.as_ref(), Some(&seal));
    assert_eq!(
        crate::support::reports::bytes(runtime.storage(), &seal.source.context).await,
        input_bytes
    );
    let report = runtime.open_cycle_report(&cycle_id).await.unwrap();
    assert!(!report.html.is_empty());
    assert_eq!(
        crate::support::reports::bytes(runtime.storage(), &seal.report).await,
        sealed_html,
        "cycle composition must not replace the sealed execution report"
    );
    assert_eq!(
        recovered.test_executions[1].status,
        ExecutionStatus::Interrupted
    );
    // The Recovered Sibling Requires Its Own New Report; It Is Not A Sealed Member.
    drop(runtime);
    owner.shutdown().await.unwrap();
}

/* === Private Functions === */

/* === Tests === */
