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
use std::sync::{Mutex, atomic::AtomicBool};

// Third Party Imports
use harmos::storage::{StorageBackend, StorageBackends};

// Local Imports
use crate::service_support::prelude::*;
use crate::service_support::publication::{AdmissionClock, AdmissionWorker, AppendBoundary, PublicationBackend, PublicationFault};
use dokime_domain::model::execution::saved::ExecutionRevision;
use dokime_runtime::report::saved::CapturedExecutionReportInput;
use dokime_runtime::storage::{Backend, Manager, objects::execution::Execution};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

#[tokio::test]
async fn dependency_publication_retry() {
    let source_root = tempdir().unwrap();
    let (mut source_owner, runtime) = booted_with_fixture(source_root.path(), Arc::new(PublishingExecutor)).await;
    let suite = fixture_suite(&["passed"]);
    register_custom_suite(&runtime, suite.clone()).await;
    let cycle_id = start_cycle(&runtime, suite).await;
    runtime.start_next_execution(cycle_id.clone(), CommandContext::default()).await;
    let cycle = wait_for_cycle_outcome(&runtime, &cycle_id, ResultStatus::Passed).await;
    let source_seal = cycle.test_executions[0].saved.as_ref().unwrap();
    let inspection = runtime.execution_snapshot(&cycle.test_executions[0].test_execution_id).await.unwrap();
    let context = inspection.context().unwrap().clone();
    let mut lifecycle = inspection.execution().clone();
    lifecycle.saved = None;
    let mut member_cycle = context.cycle.cycle.clone();
    member_cycle.test_executions = vec![lifecycle];
    // This Story Exercises Dependency Publication, Not Plot Materialization.
    let template = CapturedExecutionReportInput {
        report_defaults: Default::default(),
        final_report: true,
        owner_node_id: source_seal.source.owner_node_id.clone(),
        execution_id: source_seal.source.execution.execution_id.clone(),
        execution: None,
        lifecycle_id: uuid::Uuid::now_v7().to_string(),
        evidence_through: source_seal.source.evidence_through.clone(),
        telemetry_contracts: Default::default(),
        record_contracts: Default::default(),
        input: dokime_runtime::report::CycleReportInput {
            session_id: context.session_id.clone(),
            cycle: member_cycle,
            documents: Default::default(),
            plots: Default::default(),
            case_documents: Vec::new(),
            records: Default::default(),
            telemetry: Vec::new().into(),
            traceability: context.traceability.clone(),
            artifact_links: Vec::new(),
            appearance: None,
            generated_at: source_seal.sealed_at.clone(),
        },
        context,
    };
    drop(inspection);
    // A Stop Leaves Its Active Cycles To The Next Start; This Story Needs The Parent Seal Now.
    runtime.seal_cycle(cycle_id.clone(), CommandContext::default()).await.unwrap();
    drop(runtime);
    source_owner.shutdown().await.unwrap();

    // A Durable Parent Seal Releases History For Both The Cycle And Its Member.
    let backend = Backend::primary(source_root.path().to_owned()).await.unwrap();
    backend.open().await.unwrap();
    let released = backend
        .local_state()
        .call(|connection| {
            let mut statement = connection.prepare("SELECT item FROM storage_object_releases")?;
            let items = statement.query_map([], |row| row.get::<_, String>(0))?.collect::<Result<Vec<_>, _>>()?;
            Ok(items)
        })
        .await
        .unwrap();
    assert!(released.iter().any(|item| item == cycle_id.as_str()));
    assert!(released.iter().any(|item| item == template.execution_id.as_str()));
    backend.close().await.unwrap();
    drop(backend);

    for suffix in [".context", ".report"] {
        let root = tempdir().unwrap();
        let primary = Arc::new(Backend::primary(root.path().to_owned()).await.unwrap());
        let fault = Arc::new(PublicationFault::new(suffix));
        let settings = crate::service_support::budgets::story_settings();
        let state = Arc::new(harmos::storage::state::file::FileBackend::new(
            root.path().to_owned(),
            harmos::storage::StorageBackend::history(primary.as_ref()).clone(),
            dokime_runtime::config::storage_state_defaults(),
        ));
        let backend = Arc::new(PublicationBackend::new(primary, fault.clone()));
        let manager = Manager::new(StorageBackends { primary: backend, state }, settings).unwrap();
        let owner = crate::service_support::reports::storage_runtime(&manager).await;
        let mut input = template.clone();
        let execution = input.input.cycle.test_executions[0].clone();
        assert!(execution.saved.is_none());
        let receipt = manager.write_object(&Execution::try_from(&execution).unwrap()).await.unwrap();
        let revision = ExecutionRevision {
            execution_id: execution.test_execution_id.clone(),
            version: receipt.version,
            position: receipt.position.clone(),
        };
        input.execution = Some(revision.clone());
        input.evidence_through = receipt.position.clone();
        manager.flush_until(&receipt.position).await.unwrap();
        let boundary = input.evidence_through.clone();
        let input = harmos::storage::ReadBatch::new(input);
        let outputs = Vec::new();
        let mut report = dokime_runtime::report::Renderer.execution(input, outputs, &manager.cursor_saved()).unwrap();
        let sealed_at = execution.finished_at.as_ref().unwrap();
        assert!(report.save_dependencies(&manager, sealed_at, &revision).await.is_err(), "{suffix} must interrupt dependency publication");
        assert!(fault.failures() > 0, "the selected dependency must reach the real backend");
        let partial = fault.finished();
        assert!(partial.len() < 2);
        assert_eq!(fault.releases(), 0, "partial publication cannot release execution history");
        // No Execution Seal Was Written: This API Returns A Seal Only After Every Dependency Is Durable.
        let stored = manager
            .read_object::<Execution>(execution.test_execution_id.as_str(), harmos::storage::ObjectVersion::Latest)
            .await
            .unwrap()
            .unwrap();
        assert!(stored.value.restore().unwrap().saved.is_none(), "partial dependencies cannot write the execution seal");
        drop(stored);
        fault.disarm();
        manager.retry_transient_persistence();
        let seal = report.save_dependencies(&manager, sealed_at, &revision).await.unwrap();
        assert_eq!(seal.source.evidence_through, boundary, "retry keeps the original evidence cursor");
        assert_eq!(&seal.sealed_at, sealed_at);
        for reference in [&seal.source.context, &seal.report] {
            assert!(seal.dependencies_saved.covers(&reference.position).unwrap());
            assert!(!crate::service_support::reports::bytes(&manager, reference).await.is_empty());
        }
        let finished = fault.finished();
        for reference in partial {
            assert_eq!(
                finished.iter().filter(|file| file.metadata.id == reference.metadata.id).count(),
                1,
                "retry must not regenerate finished dependencies"
            );
        }
        assert_eq!(fault.releases(), 0, "preparing dependencies never seals the lifecycle");
        drop((seal, report));
        owner.shutdown().await.unwrap();
    }

    // Native Report Writers Outlive Public Mutation Windows Without Changing Their Files.
    let root = tempdir().unwrap();
    let primary = Arc::new(Backend::primary(root.path().to_owned()).await.unwrap());
    let state = Arc::new(harmos::storage::state::file::FileBackend::new(
        root.path().to_owned(),
        primary.history().clone(),
        dokime_runtime::config::storage_state_defaults(),
    ));
    let fault = Arc::new(PublicationFault::new(".report"));
    fault.disarm();
    let backend = Arc::new(PublicationBackend::new(primary, fault.clone()));
    let manager = Manager::new(StorageBackends { primary: backend, state }, crate::service_support::budgets::story_settings()).unwrap();
    let owner = crate::service_support::reports::storage_runtime(&manager).await;
    let mut input = template;
    let execution = input.input.cycle.test_executions[0].clone();
    let stored = manager.write_object(&Execution::try_from(&execution).unwrap()).await.unwrap();
    let revision = ExecutionRevision {
        execution_id: execution.test_execution_id.clone(),
        version: stored.version,
        position: stored.position.clone(),
    };
    input.execution = Some(revision.clone());
    input.evidence_through = stored.position.clone();
    manager.flush_until(&stored.position).await.unwrap();
    let input = harmos::storage::ReadBatch::new(input);
    let boundary = Arc::new(AppendBoundary {
        advance_after_append: AtomicBool::new(true),
        lose_append_reply: AtomicBool::new(false),
        calls: Mutex::new(Vec::new()),
        lengths: Mutex::new(Default::default()),
    });
    *fault.append_boundary.lock().unwrap() = Some(boundary.clone());
    let initial_window = manager.mutation_window().unwrap().id;
    let mut report = dokime_runtime::report::Renderer.execution(input.clone(), Vec::new(), &manager.cursor_saved()).unwrap();
    let sealed_at = execution.finished_at.as_ref().unwrap();
    // Age The Public Retry Window Before Writing The Native Report.
    tokio::time::pause();
    tokio::time::advance(Duration::from_secs(250)).await;
    tokio::time::resume();
    let seal = report.save_dependencies(&manager, sealed_at, &revision).await.unwrap();
    for file in [&seal.source.context, &seal.report] {
        assert!(seal.dependencies_saved.covers(&file.position).unwrap());
        assert!(!crate::service_support::reports::bytes(&manager, file).await.is_empty());
    }
    assert_ne!(manager.mutation_window().unwrap().id, initial_window);
    assert!(!boundary.calls.lock().unwrap().is_empty());

    // A Lost Native Append Reply Retries The Same Offset And Bytes After Window Expiry.
    boundary.calls.lock().unwrap().clear();
    boundary.advance_after_append.store(true, std::sync::atomic::Ordering::SeqCst);
    boundary.lose_append_reply.store(true, std::sync::atomic::Ordering::SeqCst);
    let mut ambiguous = dokime_runtime::report::Renderer.execution(input, Vec::new(), &manager.cursor_saved()).unwrap();
    tokio::time::pause();
    tokio::time::advance(Duration::from_secs(250)).await;
    tokio::time::resume();
    let lost = ambiguous.save_dependencies(&manager, sealed_at, &revision).await.unwrap_err();
    assert!(lost.to_string().contains("controlled append reply loss"), "{lost}");
    let prior = boundary.calls.lock().unwrap().clone();
    boundary.lose_append_reply.store(false, std::sync::atomic::Ordering::SeqCst);
    manager.retry_transient_persistence();
    let recovered = ambiguous.save_dependencies(&manager, sealed_at, &revision).await.unwrap();
    let after = boundary.calls.lock().unwrap().clone();
    assert!(after[prior.len()..].contains(prior.last().unwrap()), "an ambiguous retry must reuse its offset and bytes");
    for file in [&recovered.source.context, &recovered.report] {
        assert!(recovered.dependencies_saved.covers(&file.position).unwrap());
        assert!(!crate::service_support::reports::bytes(&manager, file).await.is_empty());
    }
    drop((seal, report, ambiguous, recovered));
    owner.shutdown().await.unwrap();
}

#[tokio::test]
async fn admitted_create_window() {
    use harmos::storage::{FileMetadata, MutationId};
    use std::sync::atomic::Ordering;

    type Engine = <Manager as std::ops::Deref>::Target;

    let root = tempdir().unwrap();
    let primary = Arc::new(Backend::primary(root.path().to_owned()).await.unwrap());
    let state = Arc::new(harmos::storage::state::file::FileBackend::new(
        root.path().to_owned(),
        primary.history().clone(),
        dokime_runtime::config::storage_state_defaults(),
    ));
    let fault = Arc::new(PublicationFault::new(".report"));
    fault.disarm();
    fault.advance_after_create.store(true, Ordering::SeqCst);
    let backend = Arc::new(PublicationBackend::new(primary.clone(), fault.clone()));
    let settings = crate::service_support::budgets::story_settings();
    let manager = Engine::with_execution(StorageBackends { primary: backend, state }, settings, AdmissionClock::execution()).unwrap();
    manager
        .start_with(Arc::new(()), |mut worker| {
            let task = tokio::spawn(async move {
                worker.run().await?;
                worker.cleanup().await
            });
            Ok(Box::new(AdmissionWorker(Mutex::new(Some(task)))))
        })
        .await
        .unwrap();

    let initial = manager.mutation_window().unwrap();
    AdmissionClock::advance(Duration::from_secs(299));
    let issued = manager.mutation_window().unwrap();
    assert_eq!(initial.id, issued.id, "issuance retains the original global epoch");
    let metadata = FileMetadata {
        id: Identifier::new("admitted-create.report").unwrap(),
        scope: None,
        media_type: "application/octet-stream".into(),
        expected_length: Some(0),
        expected_sha256: None,
    };
    let mutation = MutationId {
        window: issued.id,
        key: Identifier::new("admitted-create.create").unwrap(),
    };
    let result = manager.create_file_once(&mutation, &metadata).await;

    assert_eq!(fault.created.load(Ordering::SeqCst), 1, "the real backend completed creation before the response");
    let exists = primary
        .local_state()
        .call(|connection| {
            connection.query_row("SELECT EXISTS(SELECT 1 FROM storage_file_stages WHERE id='admitted-create.report')", [], |row| {
                row.get::<_, bool>(0)
            })
        })
        .await
        .unwrap();
    assert!(exists, "the admitted create durably staged its file");
    manager.close().await.unwrap();
    assert!(result.is_ok(), "an admitted create lost its outcome after only two seconds: {}", result.err().unwrap());
}

/* === Tests === */
