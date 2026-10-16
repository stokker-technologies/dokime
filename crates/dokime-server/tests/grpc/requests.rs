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
const READER: &str = "owner-reader-token-012345678901234567890123456";
const OTHER: &str = "other-reader-token-012345678901234567890123456";
const OTHER_CONTROL: &str = "other-control-token-012345678901234567890123456";
const ADMIN: &str = "admin-reader-token-012345678901234567890123456";
pub(crate) const CRASH_ROOT: &str = "DOKIME_TEST_RUN_REQUEST_CRASH_ROOT";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct Child(std::process::Child);

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Drop for Child {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn config() -> Config {
    Config::local()
        .with_bearer_auth(vec![
            grant(Role::Control),
            BearerGrant::new("fixture".into(), Role::Read, READER).unwrap(),
            BearerGrant::new("other".into(), Role::Read, OTHER).unwrap(),
            BearerGrant::new("other".into(), Role::Control, OTHER_CONTROL).unwrap(),
            BearerGrant::new("administrator".into(), Role::Admin, ADMIN).unwrap(),
        ])
        .unwrap()
}

fn accepted(reply: contracts::execution::RunReply) -> contracts::execution::Admission {
    match reply.outcome.unwrap() {
        contracts::execution::RunReplyOutcome::Accepted(admission) => admission,
        contracts::execution::RunReplyOutcome::Rejected(rejection) => {
            panic!("run rejected: {rejection:?}")
        }
    }
}

async fn lookup(
    client: &mut WireClient,
    id: &str,
    token: &str,
) -> Result<contracts::execution::RunRequestReceipt, tonic::Status> {
    client
        .unary::<contracts::execution::GetRunRequest>(auth(
            contracts::execution::GetRunRequestRequest {
                request_id: id.into(),
            },
            token,
        ))
        .await
        .map(|response| response.into_inner())
}

async fn started(gate: &Gate) {
    tokio::time::timeout(Duration::from_secs(5), gate.started.notified())
        .await
        .unwrap();
}

pub(crate) async fn commands_owner_receipts() {
    let root = tempfile::tempdir().unwrap();
    let gate = Arc::new(Gate::default());
    let mut owner = fixture(root.path(), gate.clone()).await;
    let server = Server::start(owner.handle(), config()).await.unwrap();
    let client = WireClient::connect(server.local_endpoint()).await.unwrap();
    let cycle_id = common::planned_cycle(&client, authenticated(create()), selection_plan(&owner))
        .await
        .unwrap();
    let execution_id = client
        .unary::<contracts::execution::ListExecutions>(authenticated(
            contracts::execution::ListExecutionsRequest {
                cycle_id,
                ..Default::default()
            },
        ))
        .await
        .unwrap()
        .into_inner()
        .executions[0]
        .execution_id
        .clone();
    let admission = accepted(
        client
            .unary::<contracts::execution::RunExecution>(authenticated(
                contracts::execution::RunExecutionRequest {
                    execution_id: execution_id.clone(),
                    mode: contracts::execution::ExecutionMode::Interactive,
                    context: None,
                    idempotency_key: Some("interactive-command-owner".into()),
                },
            ))
            .await
            .unwrap()
            .into_inner(),
    );
    started(&gate).await;
    let detail = client
        .unary::<contracts::execution::GetExecution>(authenticated(
            contracts::execution::GetExecutionRequest {
                execution_id: execution_id.clone(),
            },
        ))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(
        detail.controls.unwrap().mode,
        contracts::execution::ExecutionMode::Interactive
    );
    let command = contracts::execution::SubmitExecutionCommandRequest {
        request_id: admission.command_id.clone(),
        command_id: uuid::Uuid::now_v7().to_string(),
        execution_id,
        action: contracts::execution::ExecutionCommandAction::Next,
        step_id: None,
    };
    assert_eq!(
        client
            .unary::<contracts::execution::SubmitExecutionCommand>(auth(
                command.clone(),
                OTHER_CONTROL
            ))
            .await
            .unwrap_err()
            .code(),
        Code::NotFound
    );
    let mut unknown = command.clone();
    unknown.request_id = uuid::Uuid::now_v7().to_string();
    assert_eq!(
        client
            .unary::<contracts::execution::SubmitExecutionCommand>(auth(unknown, OTHER_CONTROL))
            .await
            .unwrap_err()
            .code(),
        Code::NotFound
    );
    let receipt = client
        .unary::<contracts::execution::SubmitExecutionCommand>(authenticated(command.clone()))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(
        receipt.phase,
        contracts::execution::ExecutionCommandPhase::Accepted
    );
    assert_eq!(
        client
            .unary::<contracts::execution::SubmitExecutionCommand>(authenticated(command.clone()))
            .await
            .unwrap()
            .into_inner(),
        receipt
    );
    let lookup = contracts::execution::GetExecutionCommandRequest {
        request_id: command.request_id.clone(),
        command_id: command.command_id.clone(),
    };
    assert_eq!(
        client
            .unary::<contracts::execution::GetExecutionCommand>(auth(lookup.clone(), READER))
            .await
            .unwrap()
            .into_inner(),
        receipt
    );
    assert_eq!(
        client
            .unary::<contracts::execution::GetExecutionCommand>(auth(lookup.clone(), ADMIN))
            .await
            .unwrap()
            .into_inner(),
        receipt
    );
    assert_eq!(
        client
            .unary::<contracts::execution::GetExecutionCommand>(auth(lookup.clone(), OTHER))
            .await
            .unwrap_err()
            .code(),
        Code::NotFound
    );
    let mut conflicting = command.clone();
    conflicting.action = contracts::execution::ExecutionCommandAction::Finish;
    assert_eq!(
        client
            .unary::<contracts::execution::SubmitExecutionCommand>(authenticated(conflicting))
            .await
            .unwrap_err()
            .code(),
        Code::FailedPrecondition
    );
    gate.release.notify_one();
    let terminal = tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            let response = client
                .unary::<contracts::execution::GetExecutionCommand>(auth(lookup.clone(), READER))
                .await;
            if let Err(error) = &response {
                let request_id =
                    dokime_app::core::CommandId::from_uuid(command.request_id.clone()).unwrap();
                let original = owner.handle().run_request(&request_id).await;
                panic!("command lookup failed: {error}; original receipt: {original:?}");
            }
            let receipt = response.unwrap().into_inner();
            if receipt.phase == contracts::execution::ExecutionCommandPhase::Interrupted {
                break receipt;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    assert_eq!(terminal.accepted_at, receipt.accepted_at);
    assert!(terminal.attempt_id.is_none());
    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();

    let mut rebooted = runtime(root.path()).await;
    let server = Server::start(rebooted.handle(), config()).await.unwrap();
    let client = WireClient::connect(server.local_endpoint()).await.unwrap();
    assert_eq!(
        client
            .unary::<contracts::execution::GetExecutionCommand>(auth(lookup, READER))
            .await
            .unwrap()
            .into_inner(),
        terminal
    );
    assert_eq!(
        client
            .unary::<contracts::execution::SubmitExecutionCommand>(authenticated(command))
            .await
            .unwrap()
            .into_inner(),
        terminal
    );
    server.shutdown().await.unwrap();
    rebooted.shutdown().await.unwrap();
}

pub(crate) async fn keyed_runs_restart() {
    let root = tempfile::tempdir().unwrap();
    let gate = Arc::new(Gate::default());
    let mut owner = fixture(root.path(), gate.clone()).await;
    let server = Server::start(owner.handle(), config()).await.unwrap();
    let mut client = WireClient::connect(server.local_endpoint()).await.unwrap();
    let cycle_id = common::planned_cycle(&client, authenticated(create()), selection_plan(&owner))
        .await
        .unwrap();
    let request = contracts::execution::RunCycleRequest {
        idempotency_key: Some("operator-operation-1".into()),
        ..run(&cycle_id)
    };
    let admission = accepted(
        client
            .unary::<contracts::execution::RunCycle>(authenticated(request.clone()))
            .await
            .unwrap()
            .into_inner(),
    );
    started(&gate).await;
    // A different correlation label is not a different operation or identity.
    let mut retry = request.clone();
    retry.context = Some(contracts::execution::MutationContext {
        correlation_id: Some(uuid::Uuid::now_v7().to_string()),
    });
    let duplicate = accepted(
        client
            .unary::<contracts::execution::RunCycle>(authenticated(retry))
            .await
            .unwrap()
            .into_inner(),
    );
    assert_eq!(duplicate.command_id, admission.command_id);
    assert_eq!(duplicate.accepted_at, admission.accepted_at);
    let running = lookup(&mut client, &admission.command_id, READER)
        .await
        .unwrap();
    assert_eq!(
        running.phase,
        contracts::execution::RunRequestPhase::Running
    );
    assert_eq!(running.submitted_by.as_deref(), Some("fixture"));
    assert_eq!(running.planned_count, 1);
    assert_eq!(running.prepared_count, 1);
    assert_eq!(running.claimed_count, 1);
    assert_eq!(running.finished_count, 0);
    assert!(running.provider_started_at.is_some());
    assert!(running.finished_at.is_none());
    assert_eq!(
        lookup(&mut client, &admission.command_id, OTHER)
            .await
            .unwrap_err()
            .code(),
        Code::NotFound
    );
    assert_eq!(
        lookup(&mut client, &uuid::Uuid::now_v7().to_string(), OTHER)
            .await
            .unwrap_err()
            .code(),
        Code::NotFound
    );
    assert_eq!(
        lookup(&mut client, &admission.command_id, ADMIN)
            .await
            .unwrap(),
        running
    );
    let claim = running.last_claim.as_ref().unwrap();
    let conflict = client
        .unary::<contracts::execution::RunExecution>(authenticated(
            contracts::execution::RunExecutionRequest {
                mode: Default::default(),
                execution_id: claim.execution_id.clone(),
                context: None,
                idempotency_key: request.idempotency_key.clone(),
            },
        ))
        .await
        .unwrap()
        .into_inner();
    assert!(
        matches!(conflict.outcome, Some(contracts::execution::RunReplyOutcome::Rejected(ref error)) if error.code == "request_identity_conflict")
    );
    let other_cycle =
        common::planned_cycle(&client, authenticated(create()), selection_plan(&owner))
            .await
            .unwrap();
    let conflict = client
        .unary::<contracts::execution::RunCycle>(authenticated(
            contracts::execution::RunCycleRequest {
                cycle_id: other_cycle.clone(),
                ..request.clone()
            },
        ))
        .await
        .unwrap()
        .into_inner();
    assert!(
        matches!(conflict.outcome, Some(contracts::execution::RunReplyOutcome::Rejected(ref error)) if error.code == "request_identity_conflict")
    );
    assert_eq!(
        owner
            .handle()
            .watch_state()
            .latest()
            .state()
            .cycles
            .values()
            .map(|cycle| cycle.execution_ids.len())
            .sum::<usize>(),
        2 // Both cycles own a planned execution; conflicting admission starts neither again.
    );
    gate.release.notify_one();
    let mut latest_receipt = Some(running);
    let completed = tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            let receipt = lookup(&mut client, &admission.command_id, READER)
                .await
                .unwrap();
            if receipt.phase == contracts::execution::RunRequestPhase::Completed {
                break receipt;
            }
            latest_receipt = Some(receipt);
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await;
    if completed.is_err() {
        eprintln!(
            "Keyed Completion Timeout: provider_outcome_ready={}",
            gate.returned.load(Ordering::Acquire),
        );
        if let Some(receipt) = latest_receipt {
            eprintln!(
                "Last Receipt: phase={:?} planned={} prepared={} claimed={} finished={} failed={}",
                receipt.phase,
                receipt.planned_count,
                receipt.prepared_count,
                receipt.claimed_count,
                receipt.finished_count,
                receipt.failed_count,
            );
        }
        let snapshot = owner.handle().watch_state().latest();
        if let Some(cycle) = snapshot
            .state()
            .cycle_views()
            .find(|cycle| cycle.test_cycle_id.as_str() == cycle_id)
        {
            eprintln!("Cycle State: {:?}", cycle.lifecycle);
            for execution in cycle.test_executions.iter().take(16) {
                eprintln!(
                    "Execution State: id={} status={:?}",
                    execution.test_execution_id, execution.status,
                );
            }
        }
        drop(snapshot);

        // Teardown Evidence Does Not Extend The Failed Completion Deadline.
        let teardown = tokio::time::timeout(Duration::from_secs(5), owner.shutdown()).await;
        match teardown {
            Ok(Err(error)) => {
                let label = if matches!(&error, dokime_app::Error::ProviderCompletion { .. }) {
                    "Post-Failure Teardown Provider Completion"
                } else {
                    "Post-Failure Teardown Error (Inconclusive)"
                };
                let message: String = error.to_string().chars().take(4096).collect();
                eprintln!("{label}: {message}");
            }
            Ok(Ok(_)) => {
                eprintln!("Post-Failure Teardown: Completed; Timeout Cause Remains Unresolved",)
            }
            Err(_) => eprintln!(
                "Post-Failure Teardown: Diagnostic Cap Expired; Timeout Cause Remains Unresolved",
            ),
        }
    }
    let completed = completed.unwrap();
    assert_eq!(completed.finished_count, 1);
    assert_eq!(completed.failed_count, 0);
    assert!(completed.finished_at.is_some());
    assert!(completed.interruption.is_none());
    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();

    let mut rebooted = runtime(root.path()).await;
    let restored = rebooted.handle().watch_state().latest();
    assert_eq!(restored.state().cycles.len(), 1);
    let plan = restored.state().cycle_views().next().unwrap();
    assert_eq!(plan.test_cycle_id.as_str(), other_cycle);
    assert_eq!(plan.test_executions.len(), 1);
    assert_eq!(
        plan.test_executions[0].status,
        dokime_app::execution::ExecutionStatus::Interrupted
    );
    assert!(plan.test_executions[0].started_at.is_none());
    assert!(plan.test_executions[0].finished_at.is_some());
    let next = Server::start(rebooted.handle(), config()).await.unwrap();
    let mut client = WireClient::connect(next.local_endpoint()).await.unwrap();
    assert_eq!(
        lookup(&mut client, &admission.command_id, READER)
            .await
            .unwrap(),
        completed
    );
    assert_eq!(
        lookup(&mut client, &admission.command_id, OTHER)
            .await
            .unwrap_err()
            .code(),
        Code::NotFound
    );
    let retried = accepted(
        client
            .unary::<contracts::execution::RunCycle>(authenticated(request))
            .await
            .unwrap()
            .into_inner(),
    );
    assert_eq!(retried.command_id, admission.command_id);
    assert_eq!(retried.accepted_at, admission.accepted_at);
    let restored = rebooted.handle().watch_state().latest();
    assert_eq!(restored.state().cycles.len(), 1);
    let plan = restored.state().cycle_views().next().unwrap();
    assert_eq!(plan.test_cycle_id.as_str(), other_cycle);
    assert_eq!(plan.test_executions.len(), 1);
    assert_eq!(
        plan.test_executions[0].status,
        dokime_app::execution::ExecutionStatus::Interrupted
    );
    assert!(plan.test_executions[0].started_at.is_none());
    assert!(plan.test_executions[0].finished_at.is_some());
    next.shutdown().await.unwrap();
    rebooted.shutdown().await.unwrap();
}

pub(crate) async fn invalid_run_admission() {
    let root = tempfile::tempdir().unwrap();
    let gate = Arc::new(Gate::default());
    let mut owner = fixture(root.path(), gate).await;
    let server = Server::start(owner.handle(), config()).await.unwrap();
    let mut client = WireClient::connect(server.local_endpoint()).await.unwrap();
    let id = common::planned_cycle(&client, authenticated(create()), selection_plan(&owner))
        .await
        .unwrap();
    for key in ["".into(), " ".into(), "bad\nkey".into(), "x".repeat(257)] {
        let error = client
            .unary::<contracts::execution::RunCycle>(authenticated(
                contracts::execution::RunCycleRequest {
                    idempotency_key: Some(key),
                    ..run(&id)
                },
            ))
            .await
            .unwrap_err();
        assert_eq!(error.code(), Code::InvalidArgument);
    }
    assert_eq!(
        lookup(&mut client, "invalid", READER)
            .await
            .unwrap_err()
            .code(),
        Code::InvalidArgument
    );
    let denied = client
        .unary::<contracts::execution::RunCycle>(auth(
            contracts::execution::RunCycleRequest {
                idempotency_key: Some("valid".into()),
                ..run(&id)
            },
            READER,
        ))
        .await
        .unwrap_err();
    assert_eq!(denied.code(), Code::PermissionDenied);
    assert!(
        owner
            .handle()
            .watch_state()
            .latest()
            .state()
            .run_requests
            .is_empty()
    );
    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

pub(crate) async fn crashed_run_replay() {
    if let Some(root) = std::env::var_os(CRASH_ROOT) {
        let root = std::path::PathBuf::from(root);
        let gate = Arc::new(Gate::default());
        let owner = fixture(&root, gate.clone()).await;
        let server = Server::start(owner.handle(), config()).await.unwrap();
        let mut client = WireClient::connect(server.local_endpoint()).await.unwrap();
        let cycle = common::planned_cycle(&client, authenticated(create()), selection_plan(&owner))
            .await
            .unwrap();
        let cases = client
            .unary::<contracts::execution::ListCases>(authenticated(
                contracts::execution::ListCasesRequest {
                    cycle_id: cycle.clone(),
                    ..Default::default()
                },
            ))
            .await
            .unwrap()
            .into_inner();
        let case = cases.cases[0].case_id.clone();
        let admission = accepted(
            client
                .unary::<contracts::execution::RunCase>(authenticated(
                    contracts::execution::RunCaseRequest {
                        cycle_id: cycle.clone(),
                        case_id: case.clone(),
                        context: None,
                        idempotency_key: Some("crash-operation".into()),
                    },
                ))
                .await
                .unwrap()
                .into_inner(),
        );
        started(&gate).await;
        let receipt = lookup(&mut client, &admission.command_id, READER)
            .await
            .unwrap();
        assert_eq!(
            receipt.phase,
            contracts::execution::RunRequestPhase::Running
        );
        // Crash recovery applies to saved work; ordinary admission may still be buffered.
        owner.handle().storage().flush().await.unwrap();
        owner.handle().storage().flush_recovery().await.unwrap();
        std::fs::write(
            root.join("ready.json"),
            serde_json::to_vec(&(admission.command_id, cycle, case)).unwrap(),
        )
        .unwrap();
        std::future::pending::<()>().await;
        unreachable!();
    }
    let root = tempfile::tempdir().unwrap();
    let mut child = Child(
        std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "execution_contract", "--nocapture"])
            .env(CRASH_ROOT, root.path())
            .stdout(std::process::Stdio::null())
            .spawn()
            .unwrap(),
    );
    let (request_id, cycle_id, case_id): (String, String, String) =
        tokio::time::timeout(Duration::from_secs(15), async {
            loop {
                if let Ok(bytes) = std::fs::read(root.path().join("ready.json"))
                    && let Ok(ready) = serde_json::from_slice(&bytes)
                {
                    break ready;
                }
                assert!(
                    child.0.try_wait().unwrap().is_none(),
                    "crash fixture exited before readiness"
                );
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .unwrap();
    child.0.kill().unwrap();
    child.0.wait().unwrap();
    let mut owner = runtime(root.path()).await;
    let server = Server::start(owner.handle(), config()).await.unwrap();
    let mut client = WireClient::connect(server.local_endpoint()).await.unwrap();
    let receipt = lookup(&mut client, &request_id, READER).await.unwrap();
    assert_eq!(
        receipt.phase,
        contracts::execution::RunRequestPhase::Interrupted
    );
    assert_eq!(receipt.prepared_count, 1);
    assert_eq!(receipt.finished_count, 0);
    assert!(receipt.finished_at.is_some());
    assert!(
        receipt
            .interruption
            .as_ref()
            .is_some_and(|reason| !reason.is_empty())
    );
    let replay = accepted(
        client
            .unary::<contracts::execution::RunCase>(authenticated(
                contracts::execution::RunCaseRequest {
                    cycle_id,
                    case_id,
                    context: None,
                    idempotency_key: Some("crash-operation".into()),
                },
            ))
            .await
            .unwrap()
            .into_inner(),
    );
    assert_eq!(replay.command_id, request_id);
    let restored = owner.handle().watch_state().latest();
    assert_eq!(restored.state().cycles.len(), 1);
    let cycle = restored.state().cycle_views().next().unwrap();
    assert_eq!(cycle.test_executions.len(), 1);
    let execution = &cycle.test_executions[0];
    assert_eq!(
        execution.status,
        dokime_app::execution::ExecutionStatus::Interrupted
    );
    assert_eq!(
        execution.test_execution_id.as_str(),
        receipt.last_claim.as_ref().unwrap().execution_id
    );
    assert!(execution.finished_at.is_some());
    assert_eq!(restored.state().run_requests.len(), 1);
    let recovered = restored.state().run_requests.values().next().unwrap();
    assert_eq!(
        recovered.summary.phase,
        dokime::execution::request::RunPhase::Interrupted
    );
    assert!(recovered.pending.is_empty());
    assert_eq!(
        lookup(&mut client, &request_id, READER).await.unwrap(),
        receipt
    );
    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

/* === Tests === */
