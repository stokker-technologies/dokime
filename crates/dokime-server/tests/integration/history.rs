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
use dokime_client::contracts::history::BrowseExecutions;
use dokime_client::contracts::history::DeleteExecution;
use dokime_domain::model;
use {dokime_runtime::CycleStatus, dokime_runtime::runtime::command::CommandContext};

/* === Constants === */
const CONTROL_TOKEN: &str = "history-control-token-012345678901234567890";
const ADMIN_TOKEN: &str = "history-admin-token-01234567890123456789012";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */
async fn checkpoint_for(
    handle: &dokime_runtime::RuntimeHandle, history: &mut WireClient, execution_id: &str,
) -> model::execution::wire_checkpoint::ExecutionCheckpoint {
    support::on_state_change(handle, async || {
        let checkpoint = history
            .unary::<contracts::history::BrowseExecutions>(auth(BrowseExecutions { limit: 100, ..Default::default() }, ADMIN_TOKEN))
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
    handle.run_cycle_cases(cycle_id.clone(), CommandContext::default()).await;
    gate.started.notified().await;

    let server = Server::start(
        handle.clone(),
        Config::local()
            .with_bearer_auth(vec![
                BearerGrant::new("control".into(), Role::Control, CONTROL_TOKEN).unwrap(),
                BearerGrant::new("admin".into(), Role::Admin, ADMIN_TOKEN).unwrap(),
            ])
            .unwrap(),
    )
    .await
    .unwrap();
    let mut history = WireClient::connect(server.local_endpoint()).await.unwrap();
    let live = history
        .unary::<contracts::history::BrowseExecutions>(auth(BrowseExecutions { limit: 100, ..Default::default() }, ADMIN_TOKEN))
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
    support::wait_for_cycle_lifecycle(&handle, &cycle_id, CycleStatus::Completed).await;
    let checkpoint = checkpoint_for(&handle, &mut history, &live).await;
    let request = DeleteExecution { checkpoint: Some(checkpoint.clone()), request_identifier: "delete-request-1".into() };

    assert_eq!(history.unary::<contracts::history::DeleteExecution>(request.clone()).await.unwrap_err().code(), Code::Unauthenticated);
    assert_eq!(
        history.unary::<contracts::history::DeleteExecution>(auth(request.clone(), CONTROL_TOKEN)).await.unwrap_err().code(),
        Code::PermissionDenied
    );
    for request_identifier in [String::new(), "x".repeat(257)] {
        assert_eq!(
            history
                .unary::<contracts::history::DeleteExecution>(auth(
                    DeleteExecution { checkpoint: Some(checkpoint.clone()), request_identifier },
                    ADMIN_TOKEN,
                ))
                .await
                .unwrap_err()
                .code(),
            Code::InvalidArgument
        );
    }
    assert_eq!(
        history.unary::<contracts::history::DeleteExecution>(auth(request.clone(), ADMIN_TOKEN)).await.unwrap_err().code(),
        Code::Unimplemented,
        "native storage has no deletion contract"
    );

    assert_eq!(
        history.unary::<contracts::history::DeleteExecution>(auth(request, ADMIN_TOKEN)).await.unwrap_err().code(),
        Code::Unimplemented,
        "native storage has no deletion contract"
    );

    let sealed = checkpoint_for(&handle, &mut history, &live).await;
    let deletion = DeleteExecution { checkpoint: Some(sealed.clone()), request_identifier: "delete-request-1".into() };
    for request_identifier in ["delete-request-1", "delete-request-2"] {
        let refusal = history
            .unary::<contracts::history::DeleteExecution>(auth(
                DeleteExecution { request_identifier: request_identifier.into(), ..deletion.clone() },
                ADMIN_TOKEN,
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
