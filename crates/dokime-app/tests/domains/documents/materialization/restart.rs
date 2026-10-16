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
use std::{
    num::NonZeroUsize,
    sync::atomic::{AtomicUsize, Ordering},
};

// Third Party Imports
use harmos::{
    operations::{Context, Operation, OperationDeclaration, StreamOutput},
    storage::{
        StorageBackend,
        query::{Query, QueryCompletion, QueryOutput},
    },
};

// Local Imports
use crate::support::prelude::*;
use dokime::operations::reports::{MaterializeReport, MaterializeReportRequest, ReportSubject};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct RejectRawQuery {
    calls: Arc<AtomicUsize>,
}
struct Open;

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl harmos::operations::Lane for Open {
    fn name(&self) -> &'static str {
        "Open"
    }
    fn capacity(&self) -> harmos::operations::Capacity {
        harmos::operations::Capacity::unbounded()
    }
}
impl Operation for RejectRawQuery {
    type Declaration = harmos::storage::operations::Query;
    type Resources = AtomicUsize;
    type Error = harmos::Error;
    type Context<'a> = Context<'a>;
    type Endpoints = StreamOutput<QueryOutput>;
    async fn initialize<'a>(calls: Arc<AtomicUsize>, _: Self::Context<'a>) -> harmos::Result<Self> {
        Ok(Self { calls })
    }
    async fn run<'a>(
        &mut self,
        _: Self::Context<'a>,
        _: &mut Query,
        _: &mut Self::Endpoints,
    ) -> harmos::Result<harmos::operations::Response<QueryCompletion>> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Err(harmos::Error::InvalidInput {
            field: "sealed member",
            reason: "raw queries forbidden by restart story",
        })
    }
}

/* === Public Functions === */
pub(crate) async fn persisted_cycle_restart() {
    let root = tempdir().unwrap();
    let (mut owner, runtime) = booted_with_fixture(root.path(), Arc::new(PublishingExecutor)).await;
    let suite = fixture_suite(&["passed", "passed"]);
    register_python_suite(&runtime, suite.clone()).await;
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
    let input_bytes =
        crate::support::reports::bytes(runtime.storage(), &seal.materialized_input).await;
    let calls = Arc::new(AtomicUsize::new(0));
    runtime
        .operations()
        .unregister_from("dokime.storage", harmos::storage::operations::Query::ID)
        .await
        .unwrap();
    runtime
        .operations()
        .register_wire_operation::<RejectRawQuery>("fixture.reject.raw", Open, calls.clone())
        .unwrap();
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
        calls.load(Ordering::SeqCst),
        0,
        "sealed members must load persisted materialized inputs"
    );
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

    // Exercise Supported Stale Object History Reclamation; Raw Evidence Is Preserved.
    let backend = dokime::storage::Backend::primary(
        root.path().to_owned(),
        dokime::storage::sqlite::Limits::default(),
    )
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
        crate::support::reports::bytes(runtime.storage(), &seal.materialized_input).await,
        input_bytes
    );
    let view = runtime
        .storage()
        .view_with_limits(crate::support::budgets::story_settings().finalization())
        .await
        .unwrap();
    let input = dokime_app::report::Renderer
        .load_execution(runtime.storage(), &view, &seal)
        .await
        .unwrap();
    assert_eq!(input.execution_id, first);
    assert_eq!(input.evidence_through, seal.source.evidence_through);
    assert_eq!(
        recovered.test_executions[1].status,
        ExecutionStatus::Interrupted
    );
    // The Recovered Sibling Requires Its Own New Report; It Is Not A Sealed Member.
    drop((input, view));
    drop(runtime);
    owner.shutdown().await.unwrap();
}

/* === Private Functions === */

/* === Tests === */
