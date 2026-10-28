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
use dokime::runtime::command::CommandContext;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

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

#[tokio::test]
async fn deletion_validates_unsupported() {
    let root = tempfile::tempdir().unwrap();
    let gate = Arc::new(Gate::reporting(|cycle| support::fixture_provider_outcome(cycle, false)));
    let mut owner = Runtime::with_execution_fixture(
        crate::support::budgets::runtime_config(root.path(), support::metadata("grpc.history", "History")),
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

    let server = Server::start(handle.clone(), keyed()).await.unwrap();
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

#[tokio::test]
async fn empty_cycles_retain_history() {
    use dokime::client::contracts::history::BrowseCycles;
    use dokime::domain::model::execution::wire::{ArchiveScope, CycleFilter};
    use std::collections::BTreeSet;

    // Read An Empty Catalog
    let root = tempfile::tempdir().unwrap();
    let mut owner = runtime(root.path()).await;
    let handle = owner.handle();
    let server = Server::start(handle.clone(), Config::local()).await.unwrap();
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
    let mut owner = runtime(root.path()).await;
    let handle = owner.handle();
    let server = Server::start(handle.clone(), Config::local()).await.unwrap();
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
