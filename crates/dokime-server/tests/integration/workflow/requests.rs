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
use dokime_domain::model;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

async fn started(gate: &Gate) {
    gate.started.notified().await;
}

fn start(owner: &Runtime, cycle: &str) -> contracts::execution::StartExecution {
    let cycle = dokime_runtime::TestCycleId::from_uuid(cycle).unwrap();
    let execution_id = owner.handle().watch_state().latest().state().cycles[&cycle].execution_ids[0].to_string();
    contracts::execution::StartExecution { execution_id, mode: model::execution::wire::ExecutionMode::Automatic }
}

async fn receipt(handle: &dokime_runtime::RuntimeHandle, id: &str) -> Arc<model::execution::request::RunReceipt> {
    let id = model::identity::CommandId::from_uuid(id).unwrap();
    handle.run_request(&id).await.unwrap().unwrap()
}

#[tokio::test]
async fn command_receipts_survive_restart() {
    let root = tempfile::tempdir().unwrap();
    let gate = Arc::new(Gate::default());
    let mut owner = fixture(root.path(), gate.clone()).await;
    let server = Server::start(owner.handle(), keyed()).await.unwrap();
    let client = WireClient::connect(server.local_endpoint()).await.unwrap();
    let cycle_id = support::planned_cycle(&client, authenticated(create()), selection_plan(&owner)).await.unwrap();
    let execution_id = client
        .unary::<contracts::execution::ListExecutions>(authenticated(contracts::execution::ListExecutions {
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
            .unary::<contracts::execution::StartExecution>(authenticated(contracts::execution::StartExecution {
                execution_id: execution_id.clone(),
                mode: model::execution::wire::ExecutionMode::Interactive,
            }))
            .await
            .unwrap()
            .into_inner(),
    );
    started(&gate).await;
    let detail = client
        .unary::<contracts::execution::GetExecution>(authenticated(contracts::execution::GetExecution {
            execution_id: execution_id.clone(),
        }))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(detail.controls.unwrap().mode, model::execution::wire::ExecutionMode::Interactive);
    let command = contracts::execution::SubmitExecutionCommand {
        request_id: admission.command_id.clone(),
        command_id: uuid::Uuid::now_v7().to_string(),
        execution_id,
        action: model::execution::wire::ExecutionCommandAction::Next,
        step_id: None,
    };
    let mut unknown = command.clone();
    unknown.request_id = uuid::Uuid::now_v7().to_string();
    assert_eq!(
        client.unary::<contracts::execution::SubmitExecutionCommand>(authenticated(unknown)).await.unwrap_err().code(),
        Code::NotFound
    );
    let receipt = client.unary::<contracts::execution::SubmitExecutionCommand>(authenticated(command.clone())).await.unwrap().into_inner();
    assert_eq!(receipt.phase, model::execution::wire::ExecutionCommandPhase::Accepted);
    assert_eq!(
        client.unary::<contracts::execution::SubmitExecutionCommand>(authenticated(command.clone())).await.unwrap().into_inner(),
        receipt
    );
    let lookup =
        contracts::execution::GetExecutionCommand { request_id: command.request_id.clone(), command_id: command.command_id.clone() };
    assert_eq!(
        client.unary::<contracts::execution::GetExecutionCommand>(authenticated(lookup.clone())).await.unwrap().into_inner(),
        receipt
    );
    let mut conflicting = command.clone();
    conflicting.action = model::execution::wire::ExecutionCommandAction::Finish;
    assert_eq!(
        client.unary::<contracts::execution::SubmitExecutionCommand>(authenticated(conflicting)).await.unwrap_err().code(),
        Code::FailedPrecondition
    );
    gate.release.notify_one();
    let terminal = support::on_state_change(&owner.handle(), async || {
        let response = client.unary::<contracts::execution::GetExecutionCommand>(authenticated(lookup.clone())).await;
        if let Err(error) = &response {
            let request_id = dokime_domain::model::identity::CommandId::from_uuid(command.request_id.clone()).unwrap();
            let original = owner.handle().run_request(&request_id).await;
            panic!("command lookup failed: {error}; original receipt: {original:?}");
        }
        let receipt = response.unwrap().into_inner();
        if receipt.phase == model::execution::wire::ExecutionCommandPhase::Interrupted {
            return Some(receipt);
        }
        None
    })
    .await;
    assert_eq!(terminal.accepted_at, receipt.accepted_at);
    assert!(terminal.attempt_id.is_none());
    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();

    let mut rebooted = runtime(root.path()).await;
    let server = Server::start(rebooted.handle(), keyed()).await.unwrap();
    let client = WireClient::connect(server.local_endpoint()).await.unwrap();
    assert_eq!(client.unary::<contracts::execution::GetExecutionCommand>(authenticated(lookup)).await.unwrap().into_inner(), terminal);
    assert_eq!(client.unary::<contracts::execution::SubmitExecutionCommand>(authenticated(command)).await.unwrap().into_inner(), terminal);
    server.shutdown().await.unwrap();
    rebooted.shutdown().await.unwrap();
}

#[tokio::test]
async fn execution_admissions_restart() {
    let root = tempfile::tempdir().unwrap();
    let gate = Arc::new(Gate::default());
    let mut owner = fixture(root.path(), gate.clone()).await;
    let server = Server::start(owner.handle(), keyed()).await.unwrap();
    let client = WireClient::connect(server.local_endpoint()).await.unwrap();
    let cycle_id = support::planned_cycle(&client, authenticated(create()), selection_plan(&owner)).await.unwrap();
    let request = start(&owner, &cycle_id);
    let admission =
        accepted(client.unary::<contracts::execution::StartExecution>(authenticated(request.clone())).await.unwrap().into_inner());
    started(&gate).await;
    // The Attempt UUID Resolves The Same Admission Without A Caller Request Key.
    let duplicate =
        accepted(client.unary::<contracts::execution::StartExecution>(authenticated(request.clone())).await.unwrap().into_inner());
    assert_eq!(duplicate.command_id, admission.command_id);
    assert_eq!(duplicate.accepted_at, admission.accepted_at);
    let running = receipt(&owner.handle(), &admission.command_id).await;
    assert_eq!(running.summary.phase, model::execution::request::RunPhase::Running);
    assert_eq!(running.submitted_by.as_deref(), Some("remote"));
    assert_eq!(running.summary.planned, 1);
    assert_eq!(running.summary.prepared, 1);
    assert_eq!(running.claimed, 1);
    assert_eq!(running.summary.finished, 0);
    assert!(running.summary.provider_started_at.is_some());
    assert!(running.summary.finished_at.is_none());
    let unknown = model::identity::CommandId::from_uuid(uuid::Uuid::now_v7().to_string()).unwrap();
    assert!(owner.handle().run_request(&unknown).await.unwrap().is_none());
    let claim = running.last_claim.as_ref().unwrap();
    let existing = client
        .unary::<contracts::execution::StartExecution>(authenticated(contracts::execution::StartExecution {
            mode: Default::default(),
            execution_id: claim.execution_id.to_string(),
        }))
        .await
        .unwrap()
        .into_inner();
    assert!(matches!(existing.outcome, Some(model::execution::wire::RunReplyOutcome::Accepted(_))));
    let other_cycle = support::planned_cycle(&client, authenticated(create()), selection_plan(&owner)).await.unwrap();
    let conflict = client
        .unary::<contracts::execution::StartExecution>(authenticated(contracts::execution::StartExecution {
            mode: model::execution::wire::ExecutionMode::Interactive,
            ..request.clone()
        }))
        .await
        .unwrap()
        .into_inner();
    assert!(
        matches!(conflict.outcome, Some(model::execution::wire::RunReplyOutcome::Rejected(ref error)) if error.code == "request_identity_conflict")
    );
    assert_eq!(
        owner.handle().watch_state().latest().state().cycles.values().map(|cycle| cycle.execution_ids.len()).sum::<usize>(),
        2 // Both Cycles Own An Attempt; Conflicting Options Cannot Start Another.
    );
    gate.release.notify_one();
    let completed = support::on_state_change(&owner.handle(), async || {
        let receipt = receipt(&owner.handle(), &admission.command_id).await;
        (receipt.summary.phase == model::execution::request::RunPhase::Completed).then_some(receipt)
    })
    .await;
    assert_eq!(completed.summary.finished, 1);
    assert_eq!(completed.summary.failed, 0);
    assert!(completed.summary.finished_at.is_some());
    assert!(completed.interruption.is_none());
    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();

    let mut rebooted = runtime(root.path()).await;
    let handle = rebooted.handle();
    assert!(handle.watch_state().latest().state().cycles.is_empty(), "shutdown seals every old cycle");
    let other_id = dokime_domain::model::identity::TestCycleId::from_uuid(other_cycle).unwrap();
    let stored_cycle = dokime_runtime::storage::objects::cycle::read(handle.storage(), &other_id, harmos::storage::ObjectVersion::Latest)
        .await
        .unwrap()
        .unwrap()
        .value;
    assert_eq!(stored_cycle.test_cycle_id, other_id);
    assert_eq!(stored_cycle.phase, dokime_domain::model::execution::result::CyclePhase::Sealed);
    assert!(stored_cycle.saved.is_some());
    assert_eq!(stored_cycle.execution_ids.len(), 1);
    let execution_id = &stored_cycle.execution_ids[0];
    let stored_execution =
        dokime_runtime::storage::objects::execution::read(handle.storage(), execution_id, harmos::storage::ObjectVersion::Latest)
            .await
            .unwrap()
            .unwrap()
            .value;
    assert_eq!(stored_execution.status, dokime_domain::model::execution::result::ExecutionStatus::Cancelled);
    assert!(stored_execution.started_at.is_none());
    assert!(stored_execution.finished_at.is_some());
    assert!(stored_execution.saved.is_some());
    let next = Server::start(rebooted.handle(), keyed()).await.unwrap();
    let client = WireClient::connect(next.local_endpoint()).await.unwrap();
    assert_eq!(receipt(&handle, &admission.command_id).await, completed);
    let retried = accepted(client.unary::<contracts::execution::StartExecution>(authenticated(request)).await.unwrap().into_inner());
    assert_eq!(retried.command_id, admission.command_id);
    assert_eq!(retried.accepted_at, admission.accepted_at);
    assert!(handle.watch_state().latest().state().cycles.is_empty());
    let restored_cycle = dokime_runtime::storage::objects::cycle::read(handle.storage(), &other_id, harmos::storage::ObjectVersion::Latest)
        .await
        .unwrap()
        .unwrap()
        .value;
    let restored_execution =
        dokime_runtime::storage::objects::execution::read(handle.storage(), execution_id, harmos::storage::ObjectVersion::Latest)
            .await
            .unwrap()
            .unwrap()
            .value;
    assert_eq!(restored_cycle, stored_cycle, "replay preserves the other cycle's durable seal");
    assert_eq!(restored_execution, stored_execution, "replay cannot start or rewrite the other execution");
    next.shutdown().await.unwrap();
    rebooted.shutdown().await.unwrap();
}

#[tokio::test]
async fn rotated_key_preserves_execution_admission() {
    let root = tempfile::tempdir().unwrap();
    let gate = Arc::new(Gate::default());
    let mut owner = fixture(root.path(), gate.clone()).await;
    let handle = owner.handle();
    let service = dokime_server::grpc::Service::new(handle.clone(), keyed()).await.unwrap();
    let server = service.listen().await.unwrap();
    let address = server.address();
    let client = WireClient::connect(server.local_endpoint()).await.unwrap();
    let cycle = support::planned_cycle(&client, authenticated(create()), selection_plan(&owner)).await.unwrap();
    let cycle_id = dokime_domain::model::identity::TestCycleId::from_uuid(cycle.clone()).unwrap();
    let request = start(&owner, &cycle);

    let admission = client.unary::<contracts::execution::StartExecution>(authenticated(request.clone())).await.unwrap();
    let admission = accepted(admission.into_inner());
    started(&gate).await;
    let running = receipt(&handle, &admission.command_id).await;
    assert_eq!(running.summary.phase, model::execution::request::RunPhase::Running);
    assert_eq!(running.submitted_by.as_deref(), Some("remote"));
    let selection = handle.watch_state().latest().state().cycles[&cycle_id].execution_ids.clone();

    // Replace Only The Receiving Listener; The Runtime Still Owns The Run.
    server.shutdown().await.unwrap();
    const ROTATED: &str = "rotated-access-key-012345678901234567890";
    let config = Config::remote(address, None, Some(ROTATED)).unwrap();
    let rotated = service.listen_with(config).await.unwrap();
    let client = WireClient::connect(rotated.local_endpoint()).await.unwrap();
    let refused = client.unary::<contracts::execution::StartExecution>(auth(request.clone(), TOKEN)).await.unwrap_err();
    assert_eq!(refused.code(), Code::Unauthenticated);
    assert_eq!(receipt(&handle, &admission.command_id).await, running);

    let reply = client.unary::<contracts::execution::StartExecution>(auth(request, ROTATED)).await.unwrap();
    let replay = accepted(reply.into_inner());
    assert_eq!(replay.command_id, admission.command_id);
    assert_eq!(replay.accepted_at, admission.accepted_at);
    assert_eq!(handle.watch_state().latest().state().cycles[&cycle_id].execution_ids, selection);
    assert_eq!(handle.watch_state().latest().state().run_requests.len(), 1);
    gate.release.notify_one();
    let completed = support::on_state_change(&handle, async || {
        let receipt = receipt(&handle, &admission.command_id).await;
        (receipt.summary.phase == model::execution::request::RunPhase::Completed).then_some(receipt)
    })
    .await;
    assert_eq!(completed.claimed, 1);
    assert_eq!(completed.summary.finished, 1);
    assert_eq!(completed.summary.failed, 0);

    rotated.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

#[tokio::test]
async fn invalid_execution_admission() {
    let root = tempfile::tempdir().unwrap();
    let gate = Arc::new(Gate::default());
    let mut owner = fixture(root.path(), gate).await;
    let server = Server::start(owner.handle(), keyed()).await.unwrap();
    let client = WireClient::connect(server.local_endpoint()).await.unwrap();
    for execution_id in ["".into(), " ".into(), "bad\nidentifier".into(), "x".repeat(257)] {
        let error = client
            .unary::<contracts::execution::StartExecution>(authenticated(contracts::execution::StartExecution {
                execution_id,
                mode: model::execution::wire::ExecutionMode::Automatic,
            }))
            .await
            .unwrap_err();
        assert_eq!(error.code(), Code::InvalidArgument);
    }
    assert!(owner.handle().watch_state().latest().state().run_requests.is_empty());
    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

/* === Tests === */
