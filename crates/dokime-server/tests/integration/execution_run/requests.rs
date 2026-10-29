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
use dokime::domain::model;

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
    let cycle = dokime::TestCycleId::from_uuid(cycle).unwrap();
    let execution_id = owner.handle().watch_state().latest().state().cycles[&cycle].execution_ids[0].to_string();
    contracts::execution::StartExecution {
        execution_id,
        mode: model::execution::wire::ExecutionMode::Automatic,
    }
}

async fn receipt(handle: &dokime::RuntimeHandle, id: &str) -> impl std::ops::Deref<Target = model::execution::request::RunReceipt> + use<> {
    let id = model::identity::CommandId::from_uuid(id).unwrap();
    handle.run_request(&id).await.unwrap().unwrap()
}

pub(crate) async fn command_receipts_survive_restart() {
    let root = tempfile::tempdir().unwrap();
    let gate = Arc::new(Gate::default());
    let prepared_owner = dokime_server::grpc::Service::prepare(keyed()).unwrap();
    let mut owner = fixture(root.path(), gate.clone(), &prepared_owner).await;
    let server = prepared_owner.ready().await.unwrap().listen().await.unwrap();
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
    let lookup = contracts::execution::GetExecutionCommand {
        request_id: command.request_id.clone(),
        command_id: command.command_id.clone(),
    };
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
            let request_id = dokime::domain::model::identity::CommandId::from_uuid(command.request_id.clone()).unwrap();
            let original = owner.handle().run_request(&request_id).await;
            let original = original.as_ref().map(|receipt| receipt.as_deref());
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

    let prepared_rebooted = dokime_server::grpc::Service::prepare(keyed()).unwrap();
    let mut rebooted = runtime(root.path(), &prepared_rebooted).await;
    let server = prepared_rebooted.ready().await.unwrap().listen().await.unwrap();
    let client = WireClient::connect(server.local_endpoint()).await.unwrap();
    assert_eq!(client.unary::<contracts::execution::GetExecutionCommand>(authenticated(lookup)).await.unwrap().into_inner(), terminal);
    assert_eq!(client.unary::<contracts::execution::SubmitExecutionCommand>(authenticated(command)).await.unwrap().into_inner(), terminal);
    server.shutdown().await.unwrap();
    rebooted.shutdown().await.unwrap();
}

pub(crate) async fn execution_admissions_restart() {
    let root = tempfile::tempdir().unwrap();
    let gate = Arc::new(Gate::default());
    let prepared_owner = dokime_server::grpc::Service::prepare(keyed()).unwrap();
    let mut owner = fixture(root.path(), gate.clone(), &prepared_owner).await;
    let server = prepared_owner.ready().await.unwrap().listen().await.unwrap();
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

    let prepared_rebooted = dokime_server::grpc::Service::prepare(keyed()).unwrap();
    let mut rebooted = runtime(root.path(), &prepared_rebooted).await;
    let handle = rebooted.handle();
    assert!(handle.watch_state().latest().state().cycles.is_empty(), "shutdown seals every old cycle");
    let other_id = dokime::domain::model::identity::TestCycleId::from_uuid(other_cycle).unwrap();
    let stored_cycle_owner =
        dokime::storage::objects::cycle::read(handle.storage(), &other_id, dokime::harmos::storage::ObjectVersion::Latest)
            .await
            .unwrap()
            .unwrap();
    let stored_cycle = &stored_cycle_owner.value;
    assert_eq!(stored_cycle.test_cycle_id, other_id);
    assert_eq!(stored_cycle.phase, dokime::domain::model::execution::result::CyclePhase::Sealed);
    assert!(stored_cycle.saved.is_some());
    assert_eq!(stored_cycle.execution_ids.len(), 1);
    let execution_id = &stored_cycle.execution_ids[0];
    let stored_execution_owner =
        dokime::storage::objects::execution::read(handle.storage(), execution_id, dokime::harmos::storage::ObjectVersion::Latest)
            .await
            .unwrap()
            .unwrap();
    let stored_execution = &stored_execution_owner.value;
    assert_eq!(stored_execution.status, dokime::domain::model::execution::result::ExecutionStatus::Cancelled);
    assert!(stored_execution.started_at.is_none());
    assert!(stored_execution.finished_at.is_some());
    assert!(stored_execution.saved.is_some());
    let next = prepared_rebooted.ready().await.unwrap().listen().await.unwrap();
    let client = WireClient::connect(next.local_endpoint()).await.unwrap();
    let repeated = receipt(&handle, &admission.command_id).await;
    assert_eq!(&*repeated, &*completed);
    let retried = accepted(client.unary::<contracts::execution::StartExecution>(authenticated(request)).await.unwrap().into_inner());
    assert_eq!(retried.command_id, admission.command_id);
    assert_eq!(retried.accepted_at, admission.accepted_at);
    assert!(handle.watch_state().latest().state().cycles.is_empty());
    let restored_cycle_owner =
        dokime::storage::objects::cycle::read(handle.storage(), &other_id, dokime::harmos::storage::ObjectVersion::Latest)
            .await
            .unwrap()
            .unwrap();
    let restored_cycle = &restored_cycle_owner.value;
    let restored_execution_owner =
        dokime::storage::objects::execution::read(handle.storage(), execution_id, dokime::harmos::storage::ObjectVersion::Latest)
            .await
            .unwrap()
            .unwrap();
    let restored_execution = &restored_execution_owner.value;
    assert_eq!(restored_cycle, stored_cycle, "replay preserves the other cycle's durable seal");
    assert_eq!(restored_execution, stored_execution, "replay cannot start or rewrite the other execution");
    next.shutdown().await.unwrap();
    rebooted.shutdown().await.unwrap();
}

pub(crate) async fn pairing_revocation_preserves_execution_receipts() {
    let root = tempfile::tempdir().unwrap();
    let gate = Arc::new(Gate::default());
    let prepared_owner = dokime_server::grpc::Service::prepare(Config::local()).unwrap();
    let mut owner = fixture(root.path(), gate.clone(), &prepared_owner).await;
    let handle = owner.handle();
    let server = prepared_owner.ready().await.unwrap().listen().await.unwrap();
    let native = WireClient::connect(server.local_endpoint()).await.unwrap();
    let settings = model::remote::RemoteListenerSettings {
        enabled: true,
        address: "127.0.0.1:0".into(),
        tls_enabled: false,
        authentication_enabled: true,
        certificate_names: vec![],
    };
    let listening = native
        .unary::<contracts::remote::UpdateListener>(contracts::remote::UpdateListener { settings: Some(settings) })
        .await
        .unwrap()
        .into_inner();
    let endpoint = format!("http://{}", listening.listening_address.unwrap());
    let first = native
        .unary::<contracts::remote::CreatePairing>(contracts::remote::CreatePairing {
            label: "Revoked Client".into(),
            endpoint: endpoint.clone(),
        })
        .await
        .unwrap()
        .into_inner();
    let second = native
        .unary::<contracts::remote::CreatePairing>(contracts::remote::CreatePairing {
            label: "Surviving Client".into(),
            endpoint: endpoint.clone(),
        })
        .await
        .unwrap()
        .into_inner();
    let key = first.pairing.access_key.unwrap();
    let surviving_key = second.pairing.access_key.unwrap();
    let client = WireClient::connect(endpoint).await.unwrap();
    let cycle = support::planned_cycle(&client, auth(create(), &key), selection_plan(&owner)).await.unwrap();
    let cycle_id = dokime::domain::model::identity::TestCycleId::from_uuid(cycle.clone()).unwrap();
    let request = start(&owner, &cycle);

    let admission = client.unary::<contracts::execution::StartExecution>(auth(request.clone(), &key)).await.unwrap();
    let admission = accepted(admission.into_inner());
    started(&gate).await;
    let running = receipt(&handle, &admission.command_id).await;
    assert_eq!(running.summary.phase, model::execution::request::RunPhase::Running);
    assert_eq!(running.submitted_by.as_deref(), Some(first.client.as_ref().unwrap().identifier.as_str()));
    let selection = handle.watch_state().latest().state().cycles[&cycle_id].execution_ids.clone();

    // Revoke One Principal While Its Authenticated Stream Is Open.
    let mut watching = client
        .download::<contracts::observation::WatchRuntime>(auth(contracts::observation::WatchRuntime { cursor: None }, &key))
        .await
        .unwrap()
        .into_inner();
    assert!(watching.message().await.unwrap().is_some());
    native
        .unary::<contracts::remote::RevokePairedClient>(contracts::remote::RevokePairedClient {
            identifier: first.client.unwrap().identifier,
        })
        .await
        .unwrap();
    while let Ok(Some(frame)) = watching.message().await {
        assert!(!frame.shutting_down, "revocation leaves the runtime alive");
    }
    let refused = client.unary::<contracts::execution::StartExecution>(auth(request.clone(), &key)).await.unwrap_err();
    assert_eq!(refused.code(), Code::Unauthenticated);
    let unchanged = receipt(&handle, &admission.command_id).await;
    assert_eq!(&*unchanged, &*running);

    let reply = client.unary::<contracts::execution::StartExecution>(auth(request, &surviving_key)).await.unwrap();
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

    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

pub(crate) async fn invalid_execution_admission() {
    let root = tempfile::tempdir().unwrap();
    let gate = Arc::new(Gate::default());
    let prepared_owner = dokime_server::grpc::Service::prepare(keyed()).unwrap();
    let mut owner = fixture(root.path(), gate, &prepared_owner).await;
    let server = prepared_owner.ready().await.unwrap().listen().await.unwrap();
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
