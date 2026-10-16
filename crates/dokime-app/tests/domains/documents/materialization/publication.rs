/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Report Dependency Retry Story

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::num::NonZeroUsize;

// Third Party Imports
use harmos::MemorySize;
use harmos::storage::{ReadLimits, ReadView, StorageBackend, StorageBackends};

// Local Imports
use crate::support::prelude::*;
use crate::support::publication::{PublicationBackend, PublicationFault};
use dokime::execution::saved::ExecutionRevision;
use dokime::storage::{
    Backend, Manager,
    objects::execution::Execution,
    sqlite::{Limits, StateLimits},
};
use dokime_app::report::saved::CapturedExecutionReportInput;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct ReportOwner;

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl harmos::Application for ReportOwner {
    type State = ();
    type Transaction = harmos::state::Never;
    type Record = harmos::state::Never;
    type Streams = ();
    type Resources = ();
}

/* === Public Functions === */
pub(crate) async fn dependency_publication_retry() {
    let source_root = tempdir().unwrap();
    let (mut source_owner, runtime) =
        booted_with_fixture(source_root.path(), Arc::new(PublishingExecutor)).await;
    let suite = fixture_suite(&["passed"]);
    register_custom_suite(&runtime, suite.clone()).await;
    let cycle_id = start_cycle(&runtime, suite, false).await;
    runtime
        .run_cycle_cases(cycle_id.clone(), CommandContext::default())
        .await;
    let cycle = wait_for_cycle_lifecycle(&runtime, &cycle_id, CycleStatus::Completed).await;
    let source_seal = cycle.test_executions[0].saved.as_ref().unwrap();
    let inspection = runtime
        .execution_snapshot(&cycle.test_executions[0].test_execution_id)
        .await
        .unwrap();
    let context = inspection.context().unwrap().clone();
    let mut lifecycle = inspection.execution().clone();
    lifecycle.saved = None;
    let mut member_cycle = context.cycle.cycle.clone();
    member_cycle.test_executions = vec![lifecycle];
    // This Story Exercises Dependency Publication, Not Plot Materialization.
    let template = CapturedExecutionReportInput {
        final_report: true,
        owner_node_id: source_seal.source.owner_node_id.clone(),
        execution_id: source_seal.source.execution.execution_id.clone(),
        execution: None,
        lifecycle_id: uuid::Uuid::now_v7().to_string(),
        evidence_through: source_seal.source.evidence_through.clone(),
        telemetry_contracts: Default::default(),
        record_contracts: Default::default(),
        input: dokime_app::report::CycleReportInput {
            session_id: context.session_id.clone(),
            cycle: member_cycle,
            documents: Default::default(),
            plots: Default::default(),
            case_documents: Vec::new(),
            records: Vec::new(),
            telemetry: Vec::new(),
            traceability: context.traceability.clone(),
            artifact_links: Vec::new(),
            appearance: None,
            generated_at: source_seal.sealed_at.clone(),
        },
        context,
    };
    drop(inspection);
    drop(runtime);
    source_owner.shutdown().await.unwrap();

    // A Durable Parent Seal Releases History For Both The Cycle And Its Member.
    let backend = Backend::primary(source_root.path().to_owned(), Limits::default())
        .await
        .unwrap();
    let saved = backend.open().await.unwrap();
    let snapshot = backend.read_view(&saved).await.unwrap();
    let view = ReadView::new(
        Identifier::new("sealed-cycle-releases").unwrap(),
        saved.clone(),
        saved.clone(),
        snapshot,
    )
    .unwrap();
    let releases = backend
        .read_object_releases(
            &view,
            None,
            ReadLimits {
                rows: NonZeroUsize::new(128).unwrap(),
                bytes: NonZeroUsize::new(1024 * 1024).unwrap(),
            },
        )
        .await
        .unwrap();
    assert!(
        releases
            .entries
            .iter()
            .any(|release| release.item.as_str() == cycle_id.as_str())
    );
    assert!(
        releases
            .entries
            .iter()
            .any(|release| release.item.as_str() == template.execution_id.as_str())
    );
    drop(view);
    backend.close().await.unwrap();
    drop(backend);

    for suffix in [".context", ".report"] {
        let root = tempdir().unwrap();
        let primary = Arc::new(
            Backend::primary(root.path().to_owned(), Limits::default())
                .await
                .unwrap(),
        );
        let state = Arc::new(primary.state(StateLimits::default()));
        let fault = Arc::new(PublicationFault::new(suffix));
        let backend = Arc::new(PublicationBackend::new(primary, fault.clone()));
        let mut settings = crate::support::budgets::story_settings().harmos(
            Duration::from_secs(60),
            NonZeroUsize::new(8 * 1024 * 1024).unwrap(),
        );
        settings.max_retries = 0;
        let manager = Manager::new(
            StorageBackends {
                primary: backend,
                state,
                replicas: vec![],
            },
            settings,
        )
        .unwrap();
        let owner = harmos::Runtime::<ReportOwner>::builder((), ())
            .runtime_with_storage(&manager)
            .await
            .unwrap();
        crate::support::reports::register_storage(&owner.operations, &manager);
        let mut input = template.clone();
        let execution = input.input.cycle.test_executions[0].clone();
        assert!(execution.saved.is_none());
        let receipt = manager
            .write_object(&Execution::try_from(&execution).unwrap())
            .await
            .unwrap();
        let revision = ExecutionRevision {
            execution_id: execution.test_execution_id.clone(),
            version: receipt.version,
            position: receipt.position.clone(),
        };
        input.execution = Some(revision.clone());
        input.evidence_through = receipt.position.clone();
        manager.flush_until(&receipt.position).await.unwrap();
        let boundary = input.evidence_through.clone();
        let input = manager
            .produce(NonZeroUsize::new(input.memory_size()).unwrap(), |_| {
                Ok(input)
            })
            .unwrap()
            .try_into_value()
            .unwrap()
            .try_into_shared()
            .unwrap();
        let outputs = manager
            .produce(
                NonZeroUsize::new(std::mem::size_of::<
                    Vec<dokime::execution::saved::ExecutionOutput>,
                >())
                .unwrap(),
                |_| Ok(Vec::new()),
            )
            .unwrap()
            .try_into_value()
            .unwrap();
        let mut report = dokime_app::report::Renderer
            .execution(manager.clone(), input, outputs)
            .unwrap();
        let sealed_at = execution.finished_at.as_ref().unwrap();
        assert!(
            report
                .save_dependencies(&owner.operations, sealed_at, &revision)
                .await
                .is_err(),
            "{suffix} must interrupt dependency publication"
        );
        assert!(
            fault.failures() > 0,
            "the selected dependency must reach the real backend"
        );
        let partial = fault.finished();
        assert!(partial.len() < 2);
        assert_eq!(
            fault.releases(),
            0,
            "partial publication cannot release execution history"
        );
        // No Execution Seal Was Written: This API Returns A Seal Only After Every Dependency Is Durable.
        let stored = manager
            .read_object::<Execution>(
                execution.test_execution_id.as_str(),
                harmos::storage::ObjectVersion::Latest,
                crate::support::budgets::story_settings().decoded_bytes,
            )
            .await
            .unwrap()
            .unwrap();
        assert!(
            stored.value.restore().unwrap().saved.is_none(),
            "partial dependencies cannot write the execution seal"
        );
        drop(stored);
        fault.disarm();
        manager.retry_transient_persistence();
        let seal = report
            .save_dependencies(&owner.operations, sealed_at, &revision)
            .await
            .unwrap();
        assert_eq!(
            seal.source.evidence_through, boundary,
            "retry keeps the original evidence cursor"
        );
        assert_eq!(&seal.sealed_at, sealed_at);
        for reference in [&seal.source.context, &seal.report] {
            assert!(seal.dependencies_saved.covers(&reference.position).unwrap());
            assert!(
                !crate::support::reports::bytes(&manager, reference)
                    .await
                    .is_empty()
            );
        }
        let finished = fault.finished();
        for reference in partial {
            assert_eq!(
                finished
                    .iter()
                    .filter(|file| file.metadata.id == reference.metadata.id)
                    .count(),
                1,
                "retry must not regenerate finished dependencies"
            );
        }
        assert_eq!(
            fault.releases(),
            0,
            "preparing dependencies never seals the lifecycle"
        );
        drop((seal, report));
        owner.stop().await.unwrap();
    }
}

/* === Private Functions === */

/* === Tests === */
