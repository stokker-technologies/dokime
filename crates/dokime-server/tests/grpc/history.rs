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
use std::{sync::Arc, time::Duration};
// Third Party Imports
// Local Imports
use super::*;
use dokime_app::{core::TestCycleId, execution::CycleStatus, runtime::command::CommandContext};
use dokime_client::contracts::history::BrowseExecutionsRequest;
use dokime_client::contracts::history::DeleteExecutionRequest;

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
async fn wait_completed(runtime: &dokime_app::RuntimeHandle, id: &TestCycleId) {
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            if runtime.cycles().await.into_iter().any(|cycle| {
                cycle.test_cycle_id == *id && cycle.lifecycle == CycleStatus::Completed
            }) {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
}

async fn checkpoint_for(
    history: &mut WireClient,
    execution_id: &str,
) -> contracts::checkpoint::ExecutionCheckpoint {
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let checkpoint = history
                .unary::<contracts::history::BrowseExecutions>(auth(
                    BrowseExecutionsRequest {
                        limit: 100,
                        ..Default::default()
                    },
                    ADMIN_TOKEN,
                ))
                .await
                .unwrap()
                .into_inner()
                .entries
                .into_iter()
                .find(|entry| {
                    entry
                        .metadata
                        .as_ref()
                        .is_some_and(|metadata| metadata.execution_id == execution_id)
                })
                .and_then(|entry| entry.checkpoint);
            if let Some(checkpoint) = checkpoint {
                return checkpoint;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("execution seal is published")
}

pub(crate) async fn deletion_validates_unsupported() {
    let root = tempfile::tempdir().unwrap();
    let gate = Arc::new(Gate::reporting(|cycle| {
        common::fixture_provider_outcome(cycle, false)
    }));
    let mut owner = Runtime::with_execution_fixture(
        crate::common::budgets::runtime_config(
            root.path(),
            common::metadata("grpc.history", "History"),
        ),
        "fixture",
        gate.clone(),
    )
    .await
    .unwrap();
    let handle = owner.handle();
    let suite = common::fixture_suite(&["passed"]);
    common::register_fixture_execution(&handle, suite.clone()).await;
    let cycle_id = common::cycle_with_cases(
        &handle,
        common::metadata("history.fixture", "History fixture"),
        common::materialize_cases(suite.metadata.identifier.as_str(), &suite),
    )
    .await;
    handle
        .run_cycle_cases(cycle_id.clone(), CommandContext::default())
        .await;
    tokio::time::timeout(Duration::from_secs(5), gate.started.notified())
        .await
        .unwrap();

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
        .unary::<contracts::history::BrowseExecutions>(auth(
            BrowseExecutionsRequest {
                limit: 100,
                ..Default::default()
            },
            ADMIN_TOKEN,
        ))
        .await
        .unwrap()
        .into_inner()
        .entries
        .into_iter()
        .find(|entry| {
            entry
                .metadata
                .as_ref()
                .is_some_and(|metadata| metadata.cycle_id.as_deref() == Some(cycle_id.as_str()))
        })
        .expect("active execution is visible in history")
        .metadata
        .unwrap()
        .execution_id;
    gate.release.notify_one();
    wait_completed(&handle, &cycle_id).await;
    let checkpoint = checkpoint_for(&mut history, &live).await;
    let request = DeleteExecutionRequest {
        checkpoint: Some(checkpoint.clone()),
        request_identifier: "delete-request-1".into(),
    };

    assert_eq!(
        history
            .unary::<contracts::history::DeleteExecution>(request.clone())
            .await
            .unwrap_err()
            .code(),
        Code::Unauthenticated
    );
    assert_eq!(
        history
            .unary::<contracts::history::DeleteExecution>(auth(request.clone(), CONTROL_TOKEN))
            .await
            .unwrap_err()
            .code(),
        Code::PermissionDenied
    );
    for request_identifier in [String::new(), "x".repeat(257)] {
        assert_eq!(
            history
                .unary::<contracts::history::DeleteExecution>(auth(
                    DeleteExecutionRequest {
                        checkpoint: Some(checkpoint.clone()),
                        request_identifier,
                    },
                    ADMIN_TOKEN,
                ))
                .await
                .unwrap_err()
                .code(),
            Code::InvalidArgument
        );
    }
    assert_eq!(
        history
            .unary::<contracts::history::DeleteExecution>(auth(request.clone(), ADMIN_TOKEN))
            .await
            .unwrap_err()
            .code(),
        Code::Unimplemented,
        "native storage has no deletion contract"
    );

    assert_eq!(
        history
            .unary::<contracts::history::DeleteExecution>(auth(request, ADMIN_TOKEN))
            .await
            .unwrap_err()
            .code(),
        Code::Unimplemented,
        "native storage has no deletion contract"
    );

    let sealed = checkpoint_for(&mut history, &live).await;
    let deletion = DeleteExecutionRequest {
        checkpoint: Some(sealed.clone()),
        request_identifier: "delete-request-1".into(),
    };
    for request_identifier in ["delete-request-1", "delete-request-2"] {
        let refusal = history
            .unary::<contracts::history::DeleteExecution>(auth(
                DeleteExecutionRequest {
                    request_identifier: request_identifier.into(),
                    ..deletion.clone()
                },
                ADMIN_TOKEN,
            ))
            .await
            .unwrap_err();
        assert_eq!(refusal.code(), Code::Unimplemented);
        assert_eq!(checkpoint_for(&mut history, &live).await, sealed);
    }

    drop(history);
    tokio::time::timeout(Duration::from_secs(5), server.shutdown())
        .await
        .unwrap()
        .unwrap();
    drop(handle);
    tokio::time::timeout(Duration::from_secs(5), owner.shutdown())
        .await
        .unwrap()
        .unwrap();
}

/* === Tests === */
