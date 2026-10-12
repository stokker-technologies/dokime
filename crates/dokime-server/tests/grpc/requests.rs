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

fn accepted(reply: v1::RunReply) -> v1::Admission {
    match reply.outcome.unwrap() {
        v1::run_reply::Outcome::Accepted(admission) => admission,
        v1::run_reply::Outcome::Rejected(rejection) => panic!("run rejected: {rejection:?}"),
    }
}

async fn lookup(
    client: &mut ExecutionClient<Channel>,
    id: &str,
    token: &str,
) -> Result<v1::RunRequestReceipt, tonic::Status> {
    client
        .get_run_request(auth(
            v1::GetRunRequestRequest {
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
    let mut client = ExecutionClient::connect(server.local_endpoint())
        .await
        .unwrap();
    let cycle_id = client
        .create_cycle(authenticated(create(&owner)))
        .await
        .unwrap()
        .into_inner()
        .cycle_id;
    let execution_id = client
        .list_executions(authenticated(v1::ListExecutionsRequest {
            cycle_id,
            ..Default::default()
        }))
        .await
        .unwrap()
        .into_inner()
        .executions[0]
        .execution_id
        .clone();
    let admission = accepted(
        client
            .run_execution(authenticated(v1::RunExecutionRequest {
                execution_id: execution_id.clone(),
                mode: v1::ExecutionMode::Interactive as i32,
                context: None,
                idempotency_key: Some("interactive-command-owner".into()),
            }))
            .await
            .unwrap()
            .into_inner(),
    );
    started(&gate).await;
    let detail = client
        .get_execution(authenticated(v1::GetExecutionRequest {
            execution_id: execution_id.clone(),
        }))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(
        detail.controls.unwrap().mode,
        v1::ExecutionMode::Interactive as i32
    );
    let command = v1::SubmitExecutionCommandRequest {
        request_id: admission.command_id.clone(),
        command_id: uuid::Uuid::now_v7().to_string(),
        execution_id,
        action: v1::ExecutionCommandAction::Next as i32,
        step_id: None,
    };
    assert_eq!(
        client
            .submit_execution_command(auth(command.clone(), OTHER_CONTROL))
            .await
            .unwrap_err()
            .code(),
        Code::NotFound
    );
    let mut unknown = command.clone();
    unknown.request_id = uuid::Uuid::now_v7().to_string();
    assert_eq!(
        client
            .submit_execution_command(auth(unknown, OTHER_CONTROL))
            .await
            .unwrap_err()
            .code(),
        Code::NotFound
    );
    let receipt = client
        .submit_execution_command(authenticated(command.clone()))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(receipt.phase, v1::ExecutionCommandPhase::Accepted as i32);
    assert_eq!(
        client
            .submit_execution_command(authenticated(command.clone()))
            .await
            .unwrap()
            .into_inner(),
        receipt
    );
    let lookup = v1::GetExecutionCommandRequest {
        request_id: command.request_id.clone(),
        command_id: command.command_id.clone(),
    };
    assert_eq!(
        client
            .get_execution_command(auth(lookup.clone(), READER))
            .await
            .unwrap()
            .into_inner(),
        receipt
    );
    assert_eq!(
        client
            .get_execution_command(auth(lookup.clone(), ADMIN))
            .await
            .unwrap()
            .into_inner(),
        receipt
    );
    assert_eq!(
        client
            .get_execution_command(auth(lookup.clone(), OTHER))
            .await
            .unwrap_err()
            .code(),
        Code::NotFound
    );
    let mut conflicting = command.clone();
    conflicting.action = v1::ExecutionCommandAction::Finish as i32;
    assert_eq!(
        client
            .submit_execution_command(authenticated(conflicting))
            .await
            .unwrap_err()
            .code(),
        Code::FailedPrecondition
    );
    gate.release.notify_one();
    let terminal = tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            let response = client
                .get_execution_command(auth(lookup.clone(), READER))
                .await;
            if let Err(error) = &response {
                let request_id =
                    dokime_app::core::CommandId::from_uuid(command.request_id.clone()).unwrap();
                let original = owner.handle().run_request(&request_id).await;
                panic!("command lookup failed: {error}; original receipt: {original:?}");
            }
            let receipt = response.unwrap().into_inner();
            if receipt.phase == v1::ExecutionCommandPhase::Interrupted as i32 {
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
    let mut client = ExecutionClient::connect(server.local_endpoint())
        .await
        .unwrap();
    assert_eq!(
        client
            .get_execution_command(auth(lookup, READER))
            .await
            .unwrap()
            .into_inner(),
        terminal
    );
    assert_eq!(
        client
            .submit_execution_command(authenticated(command))
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
    let mut client = ExecutionClient::connect(server.local_endpoint())
        .await
        .unwrap();
    let cycle_id = client
        .create_cycle(authenticated(create(&owner)))
        .await
        .unwrap()
        .into_inner()
        .cycle_id;
    let request = v1::RunCycleRequest {
        idempotency_key: Some("operator-operation-1".into()),
        ..run(&cycle_id)
    };
    let admission = accepted(
        client
            .run_cycle(authenticated(request.clone()))
            .await
            .unwrap()
            .into_inner(),
    );
    started(&gate).await;
    // A different correlation label is not a different operation or identity.
    let mut retry = request.clone();
    retry.context = Some(v1::MutationContext {
        correlation_id: Some(uuid::Uuid::now_v7().to_string()),
    });
    let duplicate = accepted(
        client
            .run_cycle(authenticated(retry))
            .await
            .unwrap()
            .into_inner(),
    );
    assert_eq!(duplicate.command_id, admission.command_id);
    assert_eq!(duplicate.accepted_at, admission.accepted_at);
    let running = lookup(&mut client, &admission.command_id, READER)
        .await
        .unwrap();
    assert_eq!(running.phase, v1::RunRequestPhase::Running as i32);
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
        .run_execution(authenticated(v1::RunExecutionRequest {
            mode: Default::default(),
            execution_id: claim.execution_id.clone(),
            context: None,
            idempotency_key: request.idempotency_key.clone(),
        }))
        .await
        .unwrap()
        .into_inner();
    assert!(
        matches!(conflict.outcome, Some(v1::run_reply::Outcome::Rejected(ref error)) if error.code == "request_identity_conflict")
    );
    let other_cycle = client
        .create_cycle(authenticated(create(&owner)))
        .await
        .unwrap()
        .into_inner()
        .cycle_id;
    let conflict = client
        .run_cycle(authenticated(v1::RunCycleRequest {
            cycle_id: other_cycle.clone(),
            ..request.clone()
        }))
        .await
        .unwrap()
        .into_inner();
    assert!(
        matches!(conflict.outcome, Some(v1::run_reply::Outcome::Rejected(ref error)) if error.code == "request_identity_conflict")
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
    let completed = tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            let receipt = lookup(&mut client, &admission.command_id, READER)
                .await
                .unwrap();
            if receipt.phase == v1::RunRequestPhase::Completed as i32 {
                break receipt;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
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
    let mut client = ExecutionClient::connect(next.local_endpoint())
        .await
        .unwrap();
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
            .run_cycle(authenticated(request))
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
    let mut client = ExecutionClient::connect(server.local_endpoint())
        .await
        .unwrap();
    let id = client
        .create_cycle(authenticated(create(&owner)))
        .await
        .unwrap()
        .into_inner()
        .cycle_id;
    for key in ["".into(), " ".into(), "bad\nkey".into(), "x".repeat(257)] {
        let error = client
            .run_cycle(authenticated(v1::RunCycleRequest {
                idempotency_key: Some(key),
                ..run(&id)
            }))
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
        .run_cycle(auth(
            v1::RunCycleRequest {
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
        let mut client = ExecutionClient::connect(server.local_endpoint())
            .await
            .unwrap();
        let cycle = client
            .create_cycle(authenticated(create(&owner)))
            .await
            .unwrap()
            .into_inner()
            .cycle_id;
        let cases = client
            .list_cases(authenticated(v1::ListCasesRequest {
                cycle_id: cycle.clone(),
                ..Default::default()
            }))
            .await
            .unwrap()
            .into_inner();
        let case = cases.cases[0].case_id.clone();
        let admission = accepted(
            client
                .run_case(authenticated(v1::RunCaseRequest {
                    cycle_id: cycle.clone(),
                    case_id: case.clone(),
                    context: None,
                    idempotency_key: Some("crash-operation".into()),
                }))
                .await
                .unwrap()
                .into_inner(),
        );
        started(&gate).await;
        let receipt = lookup(&mut client, &admission.command_id, READER)
            .await
            .unwrap();
        assert_eq!(receipt.phase, v1::RunRequestPhase::Running as i32);
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
    let mut client = ExecutionClient::connect(server.local_endpoint())
        .await
        .unwrap();
    let receipt = lookup(&mut client, &request_id, READER).await.unwrap();
    assert_eq!(receipt.phase, v1::RunRequestPhase::Interrupted as i32);
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
            .run_case(authenticated(v1::RunCaseRequest {
                cycle_id,
                case_id,
                context: None,
                idempotency_key: Some("crash-operation".into()),
            }))
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
