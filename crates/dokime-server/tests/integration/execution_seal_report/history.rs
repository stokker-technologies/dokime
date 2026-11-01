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
use std::sync::Arc;
// Third Party Imports
// Local Imports
use super::*;
use dokime::client::contracts::history::BrowseExecutions;
use dokime::client::contracts::history::DeleteExecution;
use dokime::domain::model;
use dokime::domains::ExecutionProviderTarget;
use dokime::runtime::command::CommandContext;
use dokime::{ExecutionFixture, ExecutionFixtureContext, ExecutionFixtureFuture};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct RestartGate(Arc<Gate>);

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl ExecutionFixture for RestartGate {
    fn run_cycle<'a>(
        &'a self, context: ExecutionFixtureContext, cycle: &'a model::execution::result::CycleView, target: &'a ExecutionProviderTarget,
    ) -> ExecutionFixtureFuture<'a> {
        Box::pin(async move {
            for execution in &cycle.test_executions {
                if execution.status == model::execution::result::ExecutionStatus::Pending {
                    context
                        .update_execution_phase(
                            execution.scope.clone(),
                            Identifier::new("fixture.phase").unwrap(),
                            "Fixture Phase".to_owned(),
                            "running".to_owned(),
                        )
                        .await
                        .unwrap();
                }
            }
            self.0.run_cycle(context, cycle, target).await
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn copy_cycle_files(source: &std::path::Path, destination: &std::path::Path) {
    std::fs::create_dir_all(destination).unwrap();
    for entry in std::fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let name = entry.file_name();
        if matches!(name.to_str(), Some("local.lock" | "local.sqlite" | "local.sqlite-wal" | "local.sqlite-shm")) {
            continue;
        }
        let target = destination.join(name);
        if entry.file_type().unwrap().is_dir() {
            copy_cycle_files(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).unwrap();
        }
    }
}

async fn checkpoint_for(
    handle: &dokime::RuntimeHandle, history: &mut WireClient, execution_id: &str,
) -> model::execution::wire_checkpoint::ExecutionCheckpoint {
    support::on_state_change(handle, async || {
        let checkpoint = history
            .unary::<contracts::history::BrowseExecutions>(auth(BrowseExecutions { limit: 100, ..Default::default() }, TOKEN))
            .await
            .unwrap()
            .into_inner()
            .entries
            .into_iter()
            .find(|entry| entry.metadata.as_ref().is_some_and(|metadata| metadata.execution_id == execution_id))
            .and_then(|entry| entry.checkpoint);
        if let Some(checkpoint) = checkpoint {
            return Some(checkpoint);
        }
        None
    })
    .await
}

pub(crate) async fn deletion_validates_unsupported() {
    let root = tempfile::tempdir().unwrap();
    let gate = Arc::new(Gate::reporting(|cycle| support::fixture_provider_outcome(cycle, false)));
    let prepared_owner = dokime_server::grpc::Service::prepare(keyed()).unwrap();
    let mut owner = Runtime::with_execution_fixture(
        prepared_owner.configuration(crate::support::budgets::runtime_config(root.path(), support::metadata("grpc.history", "History"))),
        "fixture",
        gate.clone(),
    )
    .await
    .unwrap();
    let handle = owner.handle();
    let suite = support::fixture_suite(&["passed"]);
    support::register_fixture_execution(&handle, suite.clone()).await;
    let cycle_id = support::cycle_with_cases(
        &handle,
        support::metadata("history.fixture", "History fixture"),
        support::materialize_cases(suite.metadata.identifier.as_str(), &suite),
    )
    .await;
    let execution_id = handle.watch_state().latest().state().cycles[&cycle_id].execution_ids[0].clone();
    handle.start_execution(execution_id, model::execution::ExecutionMode::Automatic, CommandContext::default()).await;
    gate.started.notified().await;

    let server = prepared_owner.ready().await.unwrap().listen().await.unwrap();
    let mut history = WireClient::connect(server.local_endpoint()).await.unwrap();
    let live = history
        .unary::<contracts::history::BrowseExecutions>(auth(BrowseExecutions { limit: 100, ..Default::default() }, TOKEN))
        .await
        .unwrap()
        .into_inner()
        .entries
        .into_iter()
        .find(|entry| entry.metadata.as_ref().is_some_and(|metadata| metadata.cycle_id.as_deref() == Some(cycle_id.as_str())))
        .expect("active execution is visible in history")
        .metadata
        .unwrap()
        .execution_id;
    gate.release.notify_one();
    support::wait_for_cycle_outcome(&handle, &cycle_id, dokime::domain::model::execution::result::ResultStatus::Passed).await;
    let checkpoint = checkpoint_for(&handle, &mut history, &live).await;
    let request = DeleteExecution {
        checkpoint: Some(checkpoint.clone()),
        request_identifier: "delete-request-1".into(),
    };

    assert_eq!(history.unary::<contracts::history::DeleteExecution>(request.clone()).await.unwrap_err().code(), Code::Unauthenticated);
    for request_identifier in [String::new(), "x".repeat(257)] {
        assert_eq!(
            history
                .unary::<contracts::history::DeleteExecution>(auth(
                    DeleteExecution {
                        checkpoint: Some(checkpoint.clone()),
                        request_identifier
                    },
                    TOKEN,
                ))
                .await
                .unwrap_err()
                .code(),
            Code::InvalidArgument
        );
    }
    assert_eq!(
        history.unary::<contracts::history::DeleteExecution>(auth(request.clone(), TOKEN)).await.unwrap_err().code(),
        Code::Unimplemented,
        "native storage has no deletion contract"
    );

    assert_eq!(
        history.unary::<contracts::history::DeleteExecution>(auth(request, TOKEN)).await.unwrap_err().code(),
        Code::Unimplemented,
        "native storage has no deletion contract"
    );

    let sealed = checkpoint_for(&handle, &mut history, &live).await;
    let deletion = DeleteExecution {
        checkpoint: Some(sealed.clone()),
        request_identifier: "delete-request-1".into(),
    };
    for request_identifier in ["delete-request-1", "delete-request-2"] {
        let refusal = history
            .unary::<contracts::history::DeleteExecution>(auth(
                DeleteExecution {
                    request_identifier: request_identifier.into(),
                    ..deletion.clone()
                },
                TOKEN,
            ))
            .await
            .unwrap_err();
        assert_eq!(refusal.code(), Code::Unimplemented);
        assert_eq!(checkpoint_for(&handle, &mut history, &live).await, sealed);
    }

    drop(history);
    server.shutdown().await.unwrap();
    drop(handle);
    owner.shutdown().await.unwrap();
}

/* === Tests === */

#[dokime_tracing::trace_test]
#[test]
fn cycles_restart() {
    use model::execution::{ExecutionMode, result::ResultStatus, wire::CycleStatus};

    let root = tempfile::tempdir().unwrap();
    let image = tempfile::tempdir().unwrap();
    let (sealed, interrupted, execution) = {
        let session = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        session.block_on(async {
            let prepared = dokime_server::grpc::Service::prepare(Config::local()).unwrap();
            let gate = Arc::new(Gate::default());
            let mut owner = fixture_with_provider(root.path(), Arc::new(RestartGate(gate.clone())), &prepared).await;
            let handle = owner.handle();
            let suite = support::fixture_suite(&["passed", "passed"]);
            support::register_fixture_execution(&handle, suite.clone()).await;
            let sealed = support::cycle_with_cases(
                &handle,
                support::metadata("restart.sealed", "Completed Cycle"),
                support::materialize_cases(suite.metadata.identifier.as_str(), &suite),
            )
            .await;
            let removed = handle.watch_state().latest().state().cycle_view(&sealed).unwrap().test_executions[1].test_case_id.clone();
            handle.delete_cycle_case(sealed.clone(), removed, Default::default()).await.unwrap();
            let first = handle.watch_state().latest().state().cycles[&sealed].execution_ids[0].clone();
            handle.start_execution(first, ExecutionMode::Automatic, Default::default()).await;
            gate.started.notified().await;
            gate.release.notify_one();
            support::wait_for_cycle_outcome(&handle, &sealed, ResultStatus::Passed).await;
            handle.seal_cycle(sealed.clone(), Default::default()).await.unwrap();

            let interrupted = support::cycle_with_cases(
                &handle,
                support::metadata("restart.interrupted", "Interrupted Cycle"),
                support::materialize_cases(suite.metadata.identifier.as_str(), &suite).into_iter().take(1).collect(),
            )
            .await;
            let execution = handle.watch_state().latest().state().cycles[&interrupted].execution_ids[0].clone();
            handle.start_execution(execution.clone(), ExecutionMode::Automatic, Default::default()).await;
            gate.started.notified().await;
            let before = handle.operations().call::<contracts::execution::ListCycles>(Default::default()).await.unwrap();
            assert_eq!(before.cycles.len(), 2, "starting another cycle preserves completed rows");
            assert_eq!(
                handle.watch_state().latest().state().executions[&execution].status,
                model::execution::result::ExecutionStatus::Running
            );
            assert!(before.cycles.iter().any(|cycle| cycle.cycle_id == interrupted.as_str() && cycle.status == CycleStatus::Running));
            handle.storage().flush().await.unwrap();
            handle.storage().flush_recovery().await.unwrap();
            // Freeze A Durable Running Image, Then Release The In-Process Storage Lease.
            let database = dokime::storage::sqlite::rusqlite::Connection::open(root.path().join("local.sqlite")).unwrap();
            database.execute("VACUUM INTO ?1", [image.path().join("local.sqlite").to_str().unwrap()]).unwrap();
            drop(database);
            copy_cycle_files(root.path(), image.path());
            drop(handle);
            owner.shutdown().await.unwrap();
            (sealed, interrupted, execution)
        })
    };

    // Reopen The Same Root At Its Pre-Shutdown Cut, As After An Abrupt Stop.
    std::fs::remove_dir_all(root.path()).unwrap();
    copy_cycle_files(image.path(), root.path());
    std::fs::copy(image.path().join("local.sqlite"), root.path().join("local.sqlite")).unwrap();

    for _ in 0..2 {
        let session = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        session.block_on(async {
            let prepared = dokime_server::grpc::Service::prepare(Config::local()).unwrap();
            let mut owner = runtime(root.path(), &prepared).await;
            let handle = owner.handle();
            let listed = handle.operations().call::<contracts::execution::ListCycles>(Default::default()).await.unwrap();
            assert_eq!(listed.cycles.len(), 2);
            let completed = listed.cycles.iter().find(|cycle| cycle.cycle_id == sealed.as_str()).unwrap();
            assert_eq!(completed.phase, model::execution::result::CyclePhase::Sealed);
            assert_eq!(completed.status, CycleStatus::Completed);
            assert_eq!(completed.case_count, 1);
            let stopped = listed.cycles.iter().find(|cycle| cycle.cycle_id == interrupted.as_str()).unwrap();
            assert_eq!(stopped.phase, model::execution::result::CyclePhase::Sealed);
            assert_eq!(stopped.status, CycleStatus::Stopped);
            assert_eq!(stopped.running_count, 0);
            let saved =
                dokime::storage::objects::execution::read(handle.storage(), &execution, dokime::harmos::storage::ObjectVersion::Latest)
                    .await
                    .unwrap()
                    .unwrap();
            assert_eq!(saved.value.status, model::execution::result::ExecutionStatus::Interrupted);
            assert!(saved.value.finished_at.is_some());
            drop(saved);

            let records = handle
                .selected_records(dokime::domains::evidence::RecordSelection {
                    record_types: vec![dokime::records::BuiltinRecordType::TestExecutionInterrupted.identifier()],
                    scope: dokime::domains::evidence::ScopeSelection {
                        test_execution_id: Some(execution.clone()),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .await
                .unwrap();
            assert_eq!(records.len(), 1, "repeated recovery must not duplicate the interruption");
            assert_eq!(records[0].record.details.get("reason"), Some(&dokime::Value::from("runtime_restarted")));
            assert!(records[0].record.message.as_ref().is_some_and(|message| !message.is_empty()));

            let first = handle
                .operations()
                .call::<contracts::execution::ListCycles>(contracts::execution::ListCycles { limit: 1, ..Default::default() })
                .await
                .unwrap();
            let second = handle
                .operations()
                .call::<contracts::execution::ListCycles>(contracts::execution::ListCycles {
                    limit: 1,
                    after_cycle_id: first.next_after_cycle_id.clone().unwrap(),
                    cursor: first.cursor.clone(),
                    ..Default::default()
                })
                .await
                .unwrap();
            assert_eq!(first.cycles.len(), 1);
            assert_eq!(second.cycles.len(), 1);
            assert_ne!(first.cycles[0].cycle_id, second.cycles[0].cycle_id);
            assert!(second.next_after_cycle_id.is_none());

            // Every Read Used By The Cycle Workspace Opens The Retired Container.
            let cycle = handle
                .operations()
                .call::<contracts::execution::GetCycle>(contracts::execution::GetCycle { cycle_id: interrupted.to_string() })
                .await
                .unwrap();
            assert_eq!(cycle.status, stopped.status);
            assert_eq!(cycle.case_count, 1);
            let cases = handle
                .operations()
                .call::<contracts::execution::ListCases>(contracts::execution::ListCases {
                    cycle_id: interrupted.to_string(),
                    ..Default::default()
                })
                .await
                .unwrap();
            assert_eq!(cases.cases.len(), 1);
            assert_eq!(cases.cases[0].latest_execution_status, Some(model::execution::wire::ExecutionStatus::Interrupted));
            let case_id = cases.cases[0].case_id.clone();
            handle
                .operations()
                .call::<contracts::execution::GetCase>(contracts::execution::GetCase {
                    cycle_id: interrupted.to_string(),
                    case_id: case_id.clone(),
                })
                .await
                .unwrap();
            handle
                .operations()
                .call::<contracts::execution::ListCaseParameters>(contracts::execution::ListCaseParameters {
                    cycle_id: interrupted.to_string(),
                    case_id,
                    ..Default::default()
                })
                .await
                .unwrap();
            let executions = handle
                .operations()
                .call::<contracts::execution::ListExecutions>(contracts::execution::ListExecutions {
                    cycle_id: interrupted.to_string(),
                    ..Default::default()
                })
                .await
                .unwrap();
            assert_eq!(executions.executions.len(), 1);
            assert!(executions.executions[0].sealed);
            let detail = handle
                .operations()
                .call::<contracts::execution::GetExecution>(contracts::execution::GetExecution { execution_id: execution.to_string() })
                .await
                .unwrap();
            assert_eq!(detail.summary.as_ref().unwrap().status, model::execution::wire::ExecutionStatus::Interrupted);
            handle
                .operations()
                .call::<contracts::execution::ListExecutionParameters>(contracts::execution::ListExecutionParameters {
                    execution_id: execution.to_string(),
                    set: model::execution::wire::ExecutionParameterSet::Inputs,
                    limit: 0,
                    after_name: String::new(),
                    cursor: None,
                })
                .await
                .unwrap();
            handle
                .operations()
                .call::<contracts::execution::ListExecutionSteps>(contracts::execution::ListExecutionSteps {
                    execution_id: execution.to_string(),
                    ..Default::default()
                })
                .await
                .unwrap();

            handle.set_cycle_archived(sealed.clone(), true, Default::default()).await.unwrap();
            let visible = handle.operations().call::<contracts::execution::ListCycles>(Default::default()).await.unwrap();
            assert_eq!(visible.cycles.len(), 1);
            assert_eq!(visible.cycles[0].cycle_id, interrupted.as_str());
            let all = handle
                .operations()
                .call::<contracts::execution::ListCycles>(contracts::execution::ListCycles {
                    filter: Some(model::execution::wire::CycleFilter {
                        archive: Some(model::execution::wire::ArchiveScope::All),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await
                .unwrap();
            assert_eq!(all.cycles.len(), 2);
            assert!(all.cycles.iter().any(|cycle| cycle.cycle_id == sealed.as_str() && cycle.archived));
            handle.set_cycle_archived(sealed.clone(), false, Default::default()).await.unwrap();
            drop(handle);
            owner.shutdown().await.unwrap();
        });
    }
}

pub(crate) async fn empty_cycles_retain_history() {
    use dokime::client::contracts::history::BrowseCycles;
    use dokime::domain::model::execution::wire::{ArchiveScope, CycleFilter};
    use std::collections::BTreeSet;

    // Read An Empty Catalog
    let root = tempfile::tempdir().unwrap();
    let prepared_owner = dokime_server::grpc::Service::prepare(Config::local()).unwrap();
    let mut owner = runtime(root.path(), &prepared_owner).await;
    let handle = owner.handle();
    let server = prepared_owner.ready().await.unwrap().listen().await.unwrap();
    let client = connect(&server).await;
    let all = CycleFilter {
        archive: Some(ArchiveScope::All),
        ..Default::default()
    };
    let request = BrowseCycles {
        limit: 1,
        filter: Some(all.clone()),
        ..Default::default()
    };
    let empty = client.unary::<BrowseCycles>(request.clone()).await.unwrap().into_inner();
    assert!(empty.entries.is_empty());
    assert!(empty.at.is_some());
    assert!(empty.next_after_sequence.is_none());

    // Retain Independent Empty Containers
    let mut ids = BTreeSet::new();
    let mut archived_id = None;
    for label in ["Empty Alpha", "Empty Beta", "Empty Gamma"] {
        let definition = dokime::transactions::CycleDefinition {
            metadata: support::metadata("history.empty", label),
            continue_automatically: false,
        };
        let id = handle.create_cycle(definition, Default::default()).await.unwrap();
        handle.seal_cycle(id.clone(), Default::default()).await.unwrap();
        if archived_id.is_none() {
            handle.set_cycle_archived(id.clone(), true, Default::default()).await.unwrap();
            archived_id = Some(id.clone());
        }
        ids.insert(id.to_string());
    }
    let mut page = client.unary::<BrowseCycles>(request.clone()).await.unwrap().into_inner();
    assert_eq!(page.entries.len(), 1);
    let snapshot = page.at.clone().unwrap();

    // Intervening Writes Cannot Change The Selected Saved Image
    let definition = dokime::transactions::CycleDefinition {
        metadata: support::metadata("history.later", "Later Empty"),
        continue_automatically: false,
    };
    let later = handle.create_cycle(definition.clone(), Default::default()).await.unwrap();
    handle.seal_cycle(later.clone(), Default::default()).await.unwrap();
    let active = handle.create_cycle(definition, Default::default()).await.unwrap();
    let mut discovered = BTreeSet::new();
    loop {
        assert_eq!(page.at.as_ref(), Some(&snapshot));
        assert_eq!(page.entries.len(), 1);
        let entry = &page.entries[0];
        assert!(discovered.insert(entry.cycle_id.clone()), "a page must not repeat a cycle");
        assert_eq!(entry.execution_count, 0);
        assert_eq!(entry.phase, model::execution::result::CyclePhase::Sealed);
        assert!(entry.metadata.is_some());
        assert!(entry.created_at.is_some());
        assert!(entry.updated_at.is_some());
        let Some(after_sequence) = page.next_after_sequence else { break };
        page = client
            .unary::<BrowseCycles>(BrowseCycles {
                at: Some(snapshot.clone()),
                after_sequence,
                ..request.clone()
            })
            .await
            .unwrap()
            .into_inner();
    }
    assert_eq!(discovered, ids);
    assert!(!discovered.contains(later.as_str()));
    assert!(!discovered.contains(active.as_str()));

    // Reject Continuations That Cannot Name That Snapshot
    let mut foreign = snapshot.clone();
    foreign.history.as_mut().unwrap().epoch = "another.history".into();
    let invalid = [
        BrowseCycles { limit: 1001, ..request.clone() },
        BrowseCycles { after_sequence: 1, ..request.clone() },
        BrowseCycles {
            at: Some(snapshot.clone()),
            after_sequence: snapshot.sequence + 1,
            ..request.clone()
        },
        BrowseCycles { at: Some(foreign), ..request.clone() },
    ];
    for invalid in invalid {
        assert_eq!(client.unary::<BrowseCycles>(invalid).await.unwrap_err().code(), Code::InvalidArgument);
    }
    let direct = dokime::runtime::query::cycle_history::CycleBrowseQuery {
        at: None,
        after: 1,
        limit: Some(1),
        max_bytes: 4 * 1024 * 1024,
        filter: Default::default(),
    };
    assert!(handle.browse_cycles(direct).await.is_err());
    let oversized = dokime::runtime::query::cycle_history::CycleBrowseQuery {
        at: None,
        after: 0,
        limit: Some(1),
        max_bytes: 1,
        filter: Default::default(),
    };
    let refusal = handle.browse_cycles(oversized).await;
    assert!(matches!(refusal, Err(dokime::runtime::Error::Harmos(dokime::harmos::Error::ResourceExhausted { .. }))));

    // Restart And Preserve Archive Visibility And Labels
    drop(client);
    server.shutdown().await.unwrap();
    drop(handle);
    owner.shutdown().await.unwrap();
    let prepared_owner = dokime_server::grpc::Service::prepare(Config::local()).unwrap();
    let mut owner = runtime(root.path(), &prepared_owner).await;
    let handle = owner.handle();
    let server = prepared_owner.ready().await.unwrap().listen().await.unwrap();
    let client = connect(&server).await;
    let saved = client.unary::<BrowseCycles>(BrowseCycles { limit: 100, ..request.clone() }).await.unwrap().into_inner();
    assert!(ids.iter().all(|id| saved.entries.iter().any(|entry| &entry.cycle_id == id)));
    assert!(saved.entries.iter().any(|entry| entry.cycle_id == later.as_str()));
    let archived = client
        .unary::<BrowseCycles>(BrowseCycles {
            filter: Some(CycleFilter {
                archive: Some(ArchiveScope::Archived),
                ..Default::default()
            }),
            ..request.clone()
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(archived.entries.len(), 1);
    assert_eq!(archived.entries[0].cycle_id, archived_id.unwrap().as_str());
    assert!(archived.entries[0].archived);
    let filtered = client
        .unary::<BrowseCycles>(BrowseCycles {
            filter: Some(CycleFilter { text: "empty beta".into(), ..all }),
            ..request
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(filtered.entries.len(), 1);
    assert_eq!(filtered.entries[0].metadata.as_ref().unwrap().label, "Empty Beta");

    drop(client);
    server.shutdown().await.unwrap();
    drop(handle);
    owner.shutdown().await.unwrap();
}
