/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Verify Durable Attempt Admission Without Caller Keys

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

async fn planned(client: &WireClient, owner: &Runtime) -> (String, String) {
    let cycle = support::planned_cycle(client, authenticated(create()), selection_plan(owner)).await.unwrap();
    let id = dokime::TestCycleId::from_uuid(cycle.clone()).unwrap();
    let execution = owner.handle().watch_state().latest().state().cycles[&id].execution_ids[0].to_string();
    (cycle, execution)
}

fn start(id: &str) -> contracts::execution::StartExecution {
    contracts::execution::StartExecution {
        execution_id: id.to_owned(),
        mode: model::execution::wire::ExecutionMode::Automatic,
    }
}

async fn sealed(handle: &dokime::RuntimeHandle, id: &str) {
    let id = dokime::TestExecutionId::from_uuid(id).unwrap();
    support::on_state_change(handle, async || handle.watch_state().latest().state().executions[&id].saved.is_some().then_some(())).await;
}

/* === Tests === */

#[tokio::test]
async fn blocking_run_drains_and_preserves_work_after_callback_error() {
    let root = tempfile::tempdir().unwrap();
    let gate = Arc::new(Gate::default());
    let prepared_owner = dokime_server::grpc::Service::prepare(keyed()).unwrap();
    let mut owner = fixture(root.path(), gate.clone(), &prepared_owner).await;
    let server = prepared_owner.ready().await.unwrap().listen().await.unwrap();
    let wire = WireClient::connect(server.local_endpoint()).await.unwrap();
    let client =
        dokime::client::Client::connect(dokime::client::Target::local(server.local_endpoint()).unwrap(), Some(TOKEN), None).await.unwrap();
    let (_, first) = planned(&wire, &owner).await;
    let execution = client.execution(dokime::TestExecutionId::from_uuid(first.clone()).unwrap());

    // Invalid Observation Filters Cannot Admit Hardware Work.
    let invalid = dokime::client::RunOptions {
        record_types: Some(vec!["bad[regex]".into()]),
        ..Default::default()
    };
    let error = execution.run(invalid, Default::default()).await.unwrap_err();
    assert!(matches!(error, dokime::client::Error::Rpc(ref status) if status.code() == Code::InvalidArgument));
    assert!(owner.handle().watch_state().latest().state().run_requests.is_empty());
    assert_eq!(gate.calls.load(std::sync::atomic::Ordering::SeqCst), 0);

    // Blocking Run Returns Only After Both Selected Readers Drain Their Final Pages.
    let mut records = Vec::new();
    let mut samples = Vec::new();
    let mut on_records = |page: &model::evidence::wire::RecordPage| {
        records.push(page.replay_complete);
        Ok(())
    };
    let mut on_samples = |page: &model::telemetry::wire::TelemetrySamplesReply| {
        samples.push(page.complete);
        Ok(())
    };
    let options = dokime::client::RunOptions {
        record_types: Some(Vec::new()),
        streams: Some(Vec::new()),
        ..Default::default()
    };
    let running = execution.run(
        options,
        dokime::client::RunCallbacks {
            records: Some(&mut on_records),
            samples: Some(&mut on_samples),
        },
    );
    let release = async {
        gate.started.notified().await;
        gate.release.notify_one();
    };
    let (saved, ()) = tokio::join!(running, release);
    saved.unwrap();
    assert_eq!(records.last(), Some(&true));
    assert_eq!(samples.last(), Some(&true));
    sealed(&owner.handle(), &first).await;
    assert_eq!(gate.calls.load(std::sync::atomic::Ordering::SeqCst), 1);

    // A Callback Error Ends Observation While The Admitted Execution Remains Running.
    let (_, second) = planned(&wire, &owner).await;
    let execution = client.execution(dokime::TestExecutionId::from_uuid(second.clone()).unwrap());
    let mut refuse = |_: &model::evidence::wire::RecordPage| -> dokime::client::Result<()> {
        Err(dokime::client::Error::Rpc(tonic::Status::aborted("operator callback refused page")))
    };
    let options = dokime::client::RunOptions {
        record_types: Some(Vec::new()),
        ..Default::default()
    };
    let running = execution.run(
        options,
        dokime::client::RunCallbacks {
            records: Some(&mut refuse),
            samples: None,
        },
    );
    let (error, ()) = tokio::join!(running, async {
        gate.started.notified().await;
    });
    assert!(matches!(error.unwrap_err(), dokime::client::Error::Rpc(ref status) if status.code() == Code::Aborted));
    let id = dokime::TestExecutionId::from_uuid(second.clone()).unwrap();
    assert!(owner.handle().watch_state().latest().state().executions[&id].saved.is_none());
    assert_eq!(gate.calls.load(std::sync::atomic::Ordering::SeqCst), 2);
    gate.release.notify_one();
    sealed(&owner.handle(), &second).await;
    client.close_and_wait().await;
    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

#[tokio::test]
async fn queued_run_outlives_the_idle_deadline() {
    let root = tempfile::tempdir().unwrap();
    let gate = Arc::new(Gate::default());
    let prepared_owner = dokime_server::grpc::Service::prepare(keyed()).unwrap();
    let mut owner = fixture(root.path(), gate.clone(), &prepared_owner).await;
    let server = prepared_owner.ready().await.unwrap().listen().await.unwrap();
    let wire = WireClient::connect(server.local_endpoint()).await.unwrap();
    let client =
        dokime::client::Client::connect(dokime::client::Target::local(server.local_endpoint()).unwrap(), Some(TOKEN), None).await.unwrap();
    let (_, first) = planned(&wire, &owner).await;
    let (_, second) = planned(&wire, &owner).await;
    accepted(wire.unary::<contracts::execution::StartExecution>(authenticated(start(&first))).await.unwrap().into_inner());
    gate.started.notified().await;

    // A Run Queued Behind Held Work Keeps Both Followed Readers Alive Past The Client's Idle Deadline.
    let records = std::sync::atomic::AtomicUsize::new(0);
    let samples = std::sync::atomic::AtomicUsize::new(0);
    let mut on_records = |page: &model::evidence::wire::RecordPage| {
        if page.records.is_empty() && page.replay_complete {
            records.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        }
        Ok(())
    };
    let mut on_samples = |page: &model::telemetry::wire::TelemetrySamplesReply| {
        if page.samples.is_empty() && page.complete {
            samples.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        }
        Ok(())
    };
    let options = dokime::client::RunOptions {
        record_types: Some(Vec::new()),
        streams: Some(Vec::new()),
        ..Default::default()
    };
    let execution = client.execution(dokime::TestExecutionId::from_uuid(second.clone()).unwrap());
    // The Default Client Idle Deadline Applies; The Wait Outlasts It By One Server Heartbeat.
    let idle = std::time::Duration::from_millis(client.settings().request_timeout_ms);
    let heartbeat = dokime_server::grpc::WATCH_HEARTBEAT;
    let beats = usize::try_from(idle.as_millis() / heartbeat.as_millis()).unwrap();
    let running = execution.run(
        options,
        dokime::client::RunCallbacks {
            records: Some(&mut on_records),
            samples: Some(&mut on_samples),
        },
    );
    let release = async {
        tokio::time::sleep(idle + heartbeat).await;
        let queued = gate.calls.load(std::sync::atomic::Ordering::SeqCst);
        let quiet = (records.load(std::sync::atomic::Ordering::SeqCst), samples.load(std::sync::atomic::Ordering::SeqCst));
        gate.release.notify_one();
        gate.started.notified().await;
        gate.release.notify_one();
        (queued, quiet)
    };
    let (saved, (queued, (quiet_records, quiet_samples))) = tokio::join!(running, release);

    // The Run Then Returns The Saved Outcome Of The Execution It Waited For.
    saved.unwrap();
    assert_eq!(queued, 1, "the observed execution stayed queued past the idle deadline");
    assert!(quiet_records >= beats, "queued record watch delivered {quiet_records} of {beats} quiet frames");
    assert!(quiet_samples >= beats, "queued sample watch delivered {quiet_samples} of {beats} quiet frames");
    sealed(&owner.handle(), &second).await;
    client.close_and_wait().await;
    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

#[tokio::test]
async fn start_identity_survives() {
    let root = tempfile::tempdir().unwrap();
    let gate = Arc::new(Gate::default());
    let prepared_owner = dokime_server::grpc::Service::prepare(keyed()).unwrap();
    let mut owner = fixture(root.path(), gate.clone(), &prepared_owner).await;
    let handle = owner.handle();
    let server = prepared_owner.ready().await.unwrap().listen().await.unwrap();
    let wire = WireClient::connect(server.local_endpoint()).await.unwrap();
    let (first_cycle, first) = planned(&wire, &owner).await;
    let (_, second) = planned(&wire, &owner).await;

    // Concurrent Starts Resolve One Durable Admission And One Provider Effect.
    let request = start(&first);
    let left = wire.unary::<contracts::execution::StartExecution>(authenticated(request.clone()));
    let right = wire.unary::<contracts::execution::StartExecution>(authenticated(request.clone()));
    let (left, right) = tokio::join!(left, right);
    let original = accepted(left.unwrap().into_inner());
    let duplicate = accepted(right.unwrap().into_inner());
    assert_eq!(original.command_id, duplicate.command_id);
    assert_eq!(original.accepted_at, duplicate.accepted_at);
    gate.started.notified().await;
    assert_eq!(gate.calls.load(std::sync::atomic::Ordering::SeqCst), 1);

    // Another Listener Attribution Cannot Create A Second Attempt Admission.
    let native = handle
        .start_execution(
            dokime::TestExecutionId::from_uuid(first.clone()).unwrap(),
            dokime::domain::model::execution::ExecutionMode::Automatic,
            dokime::CommandContext {
                submitted_by: Some("another-listener".into()),
                ..Default::default()
            },
        )
        .await;
    assert_eq!(native.command_id.to_string(), original.command_id);
    assert_eq!(native.status, dokime::runtime::command::CommandStatus::Accepted);

    let queued = accepted(wire.unary::<contracts::execution::StartExecution>(authenticated(start(&second))).await.unwrap().into_inner());
    let duplicate = accepted(wire.unary::<contracts::execution::StartExecution>(authenticated(start(&second))).await.unwrap().into_inner());
    assert_eq!(queued.command_id, duplicate.command_id);
    assert_eq!(gate.calls.load(std::sync::atomic::Ordering::SeqCst), 1);
    assert_eq!(handle.watch_state().latest().state().run_requests.len(), 2);

    // Conflicting Options Are Refused Without Dispatching Or Mutating The Seal.
    let conflict = contracts::execution::StartExecution {
        mode: model::execution::wire::ExecutionMode::Interactive,
        ..request.clone()
    };
    let refused = wire.unary::<contracts::execution::StartExecution>(authenticated(conflict)).await.unwrap().into_inner();
    assert!(
        matches!(refused.outcome, Some(model::execution::wire::RunReplyOutcome::Rejected(ref error)) if error.code == "request_identity_conflict")
    );
    gate.release.notify_one();
    gate.started.notified().await;
    assert_eq!(gate.calls.load(std::sync::atomic::Ordering::SeqCst), 2);
    gate.release.notify_one();
    sealed(&handle, &first).await;
    sealed(&handle, &second).await;

    let checkpoint = wire
        .unary::<contracts::history::GetExecutionCheckpoint>(authenticated(contracts::history::GetExecutionCheckpoint {
            execution_identifier: first.clone(),
        }))
        .await
        .unwrap()
        .into_inner();
    let complete = accepted(wire.unary::<contracts::execution::StartExecution>(authenticated(request.clone())).await.unwrap().into_inner());
    assert_eq!(complete.command_id, original.command_id);
    assert_eq!(complete.accepted_at, original.accepted_at);
    wire.unary::<contracts::execution::SealCycle>(authenticated(contracts::execution::SealCycle { cycle_id: first_cycle, context: None }))
        .await
        .unwrap();

    // A Sealed Unstarted Attempt Also Resolves Without Hardware Or A New Receipt.
    let (cancelled_cycle, cancelled) = planned(&wire, &owner).await;
    let detail = wire
        .unary::<contracts::execution::GetExecution>(authenticated(contracts::execution::GetExecution { execution_id: cancelled.clone() }))
        .await
        .unwrap()
        .into_inner();
    let controls = detail.controls.unwrap();
    assert!(controls.can_stop);
    assert!(controls.request_id.is_none());
    for _ in 0..2 {
        wire.unary::<contracts::execution::StopExecution>(authenticated(contracts::execution::StopExecution {
            execution_id: cancelled.clone(),
            context: None,
        }))
        .await
        .unwrap();
    }
    sealed(&handle, &cancelled).await;
    let container = wire
        .unary::<contracts::execution::GetCycle>(authenticated(contracts::execution::GetCycle { cycle_id: cancelled_cycle.clone() }))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(container.phase, model::execution::result::CyclePhase::Active);
    wire.unary::<contracts::execution::SealCycle>(authenticated(contracts::execution::SealCycle {
        cycle_id: cancelled_cycle,
        context: None,
    }))
    .await
    .unwrap();
    let resolved =
        accepted(wire.unary::<contracts::execution::StartExecution>(authenticated(start(&cancelled))).await.unwrap().into_inner());
    assert!(resolved.job_id.is_none());
    assert_eq!(gate.calls.load(std::sync::atomic::Ordering::SeqCst), 2);
    assert_eq!(handle.watch_state().latest().state().run_requests.len(), 2);
    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();

    // Restarted Recovery Reads Saved Admission And Evidence Without A Provider.
    let prepared_rebooted = dokime_server::grpc::Service::prepare(keyed()).unwrap();
    let mut rebooted = runtime(root.path(), &prepared_rebooted).await;
    let server = prepared_rebooted.ready().await.unwrap().listen().await.unwrap();
    let wire = WireClient::connect(server.local_endpoint()).await.unwrap();
    let recovered = accepted(wire.unary::<contracts::execution::StartExecution>(authenticated(request)).await.unwrap().into_inner());
    assert_eq!(recovered.command_id, original.command_id);
    assert_eq!(recovered.accepted_at, original.accepted_at);
    let after = wire
        .unary::<contracts::history::GetExecutionCheckpoint>(authenticated(contracts::history::GetExecutionCheckpoint {
            execution_identifier: first,
        }))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(after, checkpoint);
    let unknown = wire
        .unary::<contracts::execution::StartExecution>(authenticated(start(&uuid::Uuid::now_v7().to_string())))
        .await
        .unwrap()
        .into_inner();
    assert!(
        matches!(unknown.outcome, Some(model::execution::wire::RunReplyOutcome::Rejected(ref error)) if error.code == "execution_not_found")
    );
    assert_eq!(gate.calls.load(std::sync::atomic::Ordering::SeqCst), 2);
    server.shutdown().await.unwrap();
    rebooted.shutdown().await.unwrap();
}
