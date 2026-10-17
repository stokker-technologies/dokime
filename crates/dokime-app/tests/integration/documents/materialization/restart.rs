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
#[tokio::test]
async fn persisted_cycle_restart() {
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
    // A never-started member is a draft: skipped, without a finish.
    assert_eq!(before.test_executions[1].status, ExecutionStatus::Skipped);
    assert!(before.test_executions[1].finished_at.is_none());
    // The member's seal is durable at once; the open cycle's state installs
    // member seals only with its own seal.
    assert!(before.test_executions[0].saved.is_none());
    let seal = crate::support::native::open(&runtime, &checkpoint)
        .await
        .execution()
        .saved
        .clone()
        .unwrap();
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
    let reread = crate::support::native::checkpoint(&runtime, &first).await;
    assert_eq!(
        crate::support::native::open(&runtime, &reread)
            .await
            .execution()
            .saved
            .as_ref(),
        Some(&seal),
        "cycle composition must not reseal its member"
    );
    drop(reread);
    drop((checkpoint, runtime));
    owner.shutdown().await.unwrap();

    // Active Parent History Must Survive Reclamation Before Restart.
    let backend = dokime::storage::Backend::primary(root.path().to_owned())
        .await
        .unwrap();
    let at = backend.open().await.unwrap();
    let released = backend
        .local_state()
        .call(|connection| {
            let mut statement =
                connection.prepare("SELECT item, through FROM storage_object_releases")?;
            let items = statement
                .query_map([], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                })?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(items)
        })
        .await
        .unwrap();
    assert!(
        !released.iter().any(|(item, _)| item == cycle_id.as_str()),
        "active parent must retain its history: {released:?}"
    );
    // A sealed member releases exactly the revisions before the anchor its
    // report captured, which its seal records; the never-started member
    // releases nothing.
    let anchor = format!("{:016x}", seal.source.execution.position.sequence - 1);
    assert!(
        released
            .iter()
            .all(|(item, _)| item != before.test_executions[1].test_execution_id.as_str()),
        "active parent must retain its draft member's history: {released:?}"
    );
    assert_eq!(
        released
            .iter()
            .filter(|(item, _)| item == first.as_str())
            .map(|(_, through)| through.as_str())
            .collect::<Vec<_>>(),
        [anchor.as_str()],
        "a sealed member releases through its captured anchor: {released:?}"
    );
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
    let recovered_checkpoint = crate::support::native::checkpoint(&runtime, &first).await;
    assert_eq!(
        crate::support::native::open(&runtime, &recovered_checkpoint)
            .await
            .execution()
            .saved
            .as_ref(),
        Some(&seal)
    );
    drop(recovered_checkpoint);
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
