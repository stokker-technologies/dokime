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
use crate::support::prelude::*;
use dokime::Message;
use dokime::core::{CommandId, Value};
use dokime_app::domains::evidence::TelemetrySelection;
use dokime_app::domains::evidence::notes::{CreateNoteRequest, EditNoteRequest, NoteError};
use dokime_app::execution::CycleView;
use dokime_app::execution::notes::NoteOwner;

/* === Constants === */
const STREAM: &str = "fixture-sidecar.reading";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[dokime::message]
#[derive(Clone, PartialEq)]
struct Publish {
    context: String,
    values: Vec<f64>,
}

#[dokime::message]
#[derive(Clone, PartialEq)]
struct Signals {
    starts: u64,
    cancelled: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */
async fn snapshot_records(
    handle: &RuntimeHandle,
    snapshot: &dokime_app::runtime::ExecutionView,
) -> Vec<dokime_app::runtime::ContextRecord> {
    let settings = crate::support::budgets::story_settings();
    let page = harmos::storage::ReadLimits {
        rows: settings.page_rows,
        bytes: settings.page_bytes,
    };
    let mut reader = snapshot
        .context_records(
            handle.storage(),
            page,
            snapshot.view().decoded_bytes().unwrap(),
        )
        .unwrap();
    let mut records = Vec::new();
    while let Some(page) = reader.next().await.unwrap() {
        records.extend(page.iter().cloned());
    }
    records
}

async fn snapshot_samples(
    handle: &RuntimeHandle,
    snapshot: &dokime_app::runtime::ExecutionView,
) -> Vec<dokime::telemetry::TelemetrySample> {
    let settings = crate::support::budgets::story_settings();
    let page = harmos::storage::ReadLimits {
        rows: settings.page_rows,
        bytes: settings.page_bytes,
    };
    let mut reader = snapshot
        .context_telemetry(
            handle.storage(),
            page,
            snapshot.view().decoded_bytes().unwrap(),
        )
        .unwrap();
    let mut samples = Vec::new();
    while let Some(page) = reader.next().await.unwrap() {
        samples.extend(page.iter().map(|entry| entry.sample.clone()));
    }
    samples
}

async fn signals(handle: &RuntimeHandle) -> Signals {
    let bytes = handle
        .operations()
        .call_encoded("fixture-sidecar.status", Vec::new())
        .await
        .unwrap();
    Signals::decode(bytes.as_slice()).unwrap()
}

async fn begin(handle: &RuntimeHandle) -> TestCycleId {
    let before = signals(handle).await.starts;
    let mut suite = handle
        .test_suites()
        .await
        .into_iter()
        .find(|entry| entry.source == "fixture-sidecar")
        .unwrap()
        .suite;
    suite
        .test_case_templates
        .retain(|template| template.metadata.identifier.as_str() == "fixture.sidecar.capture");
    assert_eq!(suite.test_case_templates.len(), 1);
    let id = start_cases(
        handle,
        metadata("native.capture", "Native Capture"),
        materialize_cases("fixture.sidecar", &suite),
        false,
    )
    .await;
    let receipt = handle
        .run_cycle_cases(id.clone(), CommandContext::default())
        .await;
    assert_eq!(receipt.status, CommandStatus::Accepted);
    tokio::time::timeout(Duration::from_secs(5), async {
        while signals(handle).await.starts <= before {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    id
}

async fn cycle(handle: &RuntimeHandle, id: &TestCycleId, status: CycleStatus) -> CycleView {
    tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            if let Some(cycle) = handle
                .cycles()
                .await
                .into_iter()
                .find(|cycle| &cycle.test_cycle_id == id && cycle.lifecycle == status)
            {
                return cycle;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("native cycle reaches requested state")
}

async fn publish(handle: &RuntimeHandle, context: &str, values: &[f64]) {
    handle
        .operations()
        .call_encoded(
            "fixture-sidecar.publish-telemetry",
            Publish {
                context: context.into(),
                values: values.to_vec(),
            }
            .encode_to_vec(),
        )
        .await
        .unwrap();
}

async fn samples(handle: &RuntimeHandle, count: usize) -> Vec<dokime::telemetry::TelemetrySample> {
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let rows = handle
                .collect_telemetry(&TelemetrySelection {
                    streams: vec![Identifier::new(STREAM).unwrap()],
                    ..Default::default()
                })
                .await;
            if rows.len() >= count {
                // Snapshots Select Persisted Evidence, Not Merely Accepted Publications.
                handle.storage().flush().await.unwrap();
                return rows;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("native rows reach public telemetry")
}

/* === Tests === */

pub(crate) async fn operator_notes_until_seal() {
    let root = tempfile::tempdir().unwrap();
    let mut owner = Runtime::start(
        config(root.path()).with_plugins([fixture_bundle(root.path(), "fixture-sidecar-protocol")]),
    )
    .await
    .unwrap();
    let handle = owner.handle();
    let id = begin(&handle).await;
    let running = cycle(&handle, &id, CycleStatus::Running).await;
    let execution = &running.test_executions[0];
    let request = CreateNoteRequest {
        epoch: handle.process_epoch(),
        mutation_id: CommandId::new(),
        owner: NoteOwner::Execution {
            cycle_id: id.clone(),
            execution_id: execution.test_execution_id.clone(),
        },
        actor: "operator".into(),
        message: "Fan **audible** at start.".into(),
    };
    let filed = handle
        .create_current_note(request.clone())
        .await
        .unwrap()
        .admission
        .note;
    assert_eq!(filed.author.as_deref(), Some("operator"));
    assert_eq!(
        filed.scope.test_execution_id,
        Some(execution.test_execution_id.clone())
    );
    let edit = EditNoteRequest {
        epoch: handle.process_epoch(),
        note_id: filed.note_id.clone(),
        actor: "operator".into(),
        message: "Fan **audible** at start; quiet after warm-up.".into(),
    };
    assert!(matches!(
        handle
            .edit_current_note(EditNoteRequest {
                actor: "someone-else".into(),
                ..edit.clone()
            })
            .await,
        Err(NoteError::NotAuthor)
    ));
    let edited = handle.edit_current_note(edit.clone()).await.unwrap();
    assert_eq!(edited.note_id, filed.note_id);
    assert_eq!(edited.created_at, filed.created_at);
    assert_eq!(edited.message, edit.message);

    let cycle_note = handle
        .create_current_note(CreateNoteRequest {
            mutation_id: CommandId::new(),
            owner: NoteOwner::Cycle {
                cycle_id: id.clone(),
            },
            message: "Bench ambient 23 °C.".into(),
            ..request.clone()
        })
        .await
        .unwrap()
        .admission
        .note;

    handle
        .stop_cycle(id.clone(), CommandContext::default())
        .await
        .unwrap();
    let stopped = cycle(&handle, &id, CycleStatus::Stopped).await;
    let seal = stopped
        .saved
        .as_ref()
        .expect("stopping finalizes the cycle report");
    let payload = crate::support::reports::payload(handle.storage(), &seal.report).await;
    let notes = payload["snapshot"]["notes"]
        .as_array()
        .expect("the sealed report carries its notes");
    assert_eq!(
        notes
            .iter()
            .filter(|note| note["note_id"] == filed.note_id.as_str())
            .map(|note| (note["author"].as_str(), note["text"].as_str()))
            .collect::<Vec<_>>(),
        [(Some("operator"), Some(edit.message.as_str()))],
        "the sealed report keeps the note as last edited"
    );

    let late = CreateNoteRequest {
        mutation_id: CommandId::new(),
        message: "Filed after the seal.".into(),
        ..request
    };
    assert!(matches!(
        handle.create_current_note(late.clone()).await,
        Err(NoteError::Sealed)
    ));
    assert!(matches!(
        handle
            .create_current_note(CreateNoteRequest {
                owner: NoteOwner::Cycle {
                    cycle_id: id.clone()
                },
                ..late
            })
            .await,
        Err(NoteError::Sealed)
    ));
    assert!(matches!(
        handle
            .edit_current_note(EditNoteRequest {
                note_id: cycle_note.note_id,
                message: "Bench ambient 24 °C.".into(),
                ..edit.clone()
            })
            .await,
        Err(NoteError::Sealed)
    ));
    assert!(matches!(
        handle.edit_current_note(edit).await,
        Err(NoteError::Sealed)
    ));
    owner.shutdown().await.unwrap();
}

pub(crate) async fn undeclared_record_types_settle_finalization() {
    const UNDECLARED: &str = "operator.note_deleted";
    let root = tempfile::tempdir().unwrap();
    let mut owner = Runtime::start(
        config(root.path()).with_plugins([fixture_bundle(root.path(), "fixture-sidecar-protocol")]),
    )
    .await
    .unwrap();
    let handle = owner.handle();
    let id = begin(&handle).await;
    let running = cycle(&handle, &id, CycleStatus::Running).await;
    let execution = running.test_executions[0].clone();
    let mut request =
        RecordPublishRequest::new(Identifier::new(UNDECLARED).unwrap(), RecordSeverity::Info);
    request.scope = Some(execution.scope.clone());

    // The publish boundary refuses a type no contract declares, naming it.
    let refused = handle
        .publish_record(request.clone())
        .await
        .expect_err("an undeclared record type is refused at publication");
    assert!(
        refused.to_string().contains(UNDECLARED),
        "the refusal names the type: {refused}"
    );

    // One that reaches finalization anyway settles the run, naming the type,
    // instead of retrying a report that can never be sealed.
    handle.publish_undeclared_record(request).await.unwrap();
    handle
        .stop_cycle(id.clone(), CommandContext::default())
        .await
        .unwrap();
    let settled = cycle(&handle, &id, CycleStatus::Failed).await;
    assert!(
        settled.saved.is_none(),
        "an unsealable cycle claims no report"
    );
    let evidence = handle.evidence_snapshot().await.unwrap();
    for (record_type, scoped) in [
        (
            "test.execution.finalization.failed",
            Some(&execution.test_execution_id),
        ),
        ("test.cycle.finalization.failed", None),
    ] {
        let record = evidence
            .records
            .iter()
            .find(|record| {
                record.record_type.as_str() == record_type
                    && record
                        .scope
                        .as_ref()
                        .is_some_and(|scope| scope.test_cycle_id.as_ref() == Some(&id))
            })
            .unwrap_or_else(|| panic!("{record_type} is recorded as evidence"));
        assert_eq!(record.severity, RecordSeverity::Error);
        assert_eq!(
            record
                .scope
                .as_ref()
                .and_then(|scope| scope.test_execution_id.as_ref()),
            scoped,
            "{record_type} scope"
        );
        assert!(
            record
                .message
                .as_deref()
                .is_some_and(|message| message.contains(UNDECLARED)),
            "{record_type} names the undeclared type: {record:?}"
        );
    }
    owner.shutdown().await.unwrap();
}

pub(crate) async fn native_capture_contexts() {
    let root = tempfile::tempdir().unwrap();
    let mut owner = Runtime::start(
        config(root.path()).with_plugins([fixture_bundle(root.path(), "fixture-sidecar-protocol")]),
    )
    .await
    .unwrap();
    let handle = owner.handle();
    let id = begin(&handle).await;
    let running = cycle(&handle, &id, CycleStatus::Running).await;
    let execution = &running.test_executions[0];
    let mut notes = Vec::new();
    for scope in [
        execution.scope.clone(),
        RecordScope::default().with_cycle(id.clone()),
    ] {
        notes.push(
            handle
                .add_operator_note(OperatorNoteRequest {
                    scope: Some(scope),
                    author: Some("capture-fixture".into()),
                    message: "Frozen report evidence".into(),
                })
                .await
                .unwrap(),
        );
    }

    publish(&handle, "current", &[1.0, 1.5]).await;
    assert_eq!(samples(&handle, 2).await.len(), 2);
    let captured = handle
        .execution_snapshot(&execution.test_execution_id)
        .await
        .unwrap();
    assert!(captured.execution().saved.is_none());
    assert_eq!(captured.context().unwrap().scope, execution.scope);
    let records = snapshot_records(&handle, &captured).await;
    for note in &notes {
        assert_eq!(
            records
                .iter()
                .map(|entry| &entry.record)
                .filter(|record| {
                    record.details.get("note_id") == Some(&Value::from(note.note_id.as_str()))
                        && record.scope.as_ref().is_some_and(|scope| {
                            scope.test_cycle_id == note.scope.test_cycle_id
                                && scope.test_case_id == note.scope.test_case_id
                                && scope.test_execution_id == note.scope.test_execution_id
                        })
                })
                .count(),
            1,
            "execution and ancestor records retain their owner exactly once"
        );
    }
    assert_eq!(captured.execution().status, ExecutionStatus::Running);
    assert_eq!(
        snapshot_samples(&handle, &captured)
            .await
            .iter()
            .filter(|row| row.stream.as_str() == STREAM)
            .count(),
        2
    );
    publish(&handle, "current", &[1.75, f64::NAN, 1.875]).await;
    publish(&handle, "unknown-execution", &[2.0]).await;
    assert_eq!(samples(&handle, 4).await.len(), 4);
    let later = handle
        .execution_snapshot(&execution.test_execution_id)
        .await
        .unwrap();
    assert!(
        later.evidence_through().unwrap().sequence > captured.evidence_through().unwrap().sequence
    );
    assert_eq!(
        snapshot_samples(&handle, &later)
            .await
            .iter()
            .filter(|row| row.stream.as_str() == STREAM)
            .count(),
        4
    );
    assert_eq!(
        snapshot_samples(&handle, &captured)
            .await
            .iter()
            .filter(|row| row.stream.as_str() == STREAM)
            .count(),
        2
    );
    assert!(later.execution().saved.is_none());
    drop(later);
    drop(captured);
    let current = cycle(&handle, &id, CycleStatus::Running).await;
    assert!(current.saved.is_none());
    assert!(current.test_executions[0].saved.is_none());

    handle
        .stop_cycle(id.clone(), CommandContext::default())
        .await
        .unwrap();
    let stopped = cycle(&handle, &id, CycleStatus::Stopped).await;
    let seal = stopped
        .saved
        .as_ref()
        .expect("stopping finalizes the cycle report");
    let payload = crate::support::reports::payload(handle.storage(), &seal.report).await;
    assert_eq!(payload["snapshot"]["telemetry"][STREAM]["total_samples"], 4);
    assert_eq!(
        payload["test_executions"][0]["test_execution_id"],
        execution.test_execution_id.as_str()
    );
    publish(&handle, "current", &[3.0]).await;

    let paused_id = begin(&handle).await;
    handle
        .pause_cycle(paused_id.clone(), CommandContext::default())
        .await
        .unwrap();
    cycle(&handle, &paused_id, CycleStatus::Paused).await;
    publish(&handle, "current", &[5.0]).await;
    publish(&handle, "", &[4.0]).await;
    let all = samples(&handle, 5).await;
    assert_eq!(
        all.len(),
        5,
        "foreign, revoked and paused contexts never become global evidence"
    );
    // Starting The Paused Cycle Retired The Stopped One; Its Saved History Answers.
    let checkpoint =
        crate::support::native::checkpoint(&handle, &execution.test_execution_id).await;
    let scoped: Vec<_> = crate::support::native::samples(&handle, &checkpoint)
        .await
        .into_iter()
        .filter(|sample| sample.stream.as_str() == STREAM)
        .collect();
    assert_eq!(scoped.len(), 4);
    assert_eq!(
        scoped
            .iter()
            .map(|sample| sample.values["value"].clone())
            .collect::<Vec<_>>(),
        [1.0, 1.5, 1.75, 1.875].map(Value::Number)
    );
    // The rows were published while the capture template's one step ran, so
    // each is filed under that step's attempt.
    let captured = scoped[0].scope.clone().expect("a captured row is scoped");
    assert!(
        matches!(captured.origin, dokime::core::EvidenceOrigin::Step { .. }),
        "{captured:?}"
    );
    let mut expected_scope = execution.scope.clone();
    expected_scope.step_id = Some(dokime::core::StepId::from_seed(
        "fixture.sidecar.capture.stream",
    ));
    expected_scope.origin = captured.origin.clone();
    assert_eq!(captured, expected_scope);
    assert_eq!(scoped[0].source, "fixture-sidecar");
    assert_eq!(scoped[0].values["value"], Value::Number(1.0));
    let global = all.iter().find(|sample| sample.scope.is_none()).unwrap();
    assert_eq!(global.values["value"], Value::Number(4.0));

    let stream = handle.streams().publication(STREAM, 1).unwrap();
    let native: Vec<_> = stream.query(..).await.unwrap().collect();
    assert_eq!(
        native.len(),
        9,
        "the physical stream retains even rejected publications"
    );
    let micros = native[0].value.published_unix_micros.unwrap();
    assert_eq!(
        scoped[0].timestamp,
        Timestamp::from_system_time(
            std::time::UNIX_EPOCH + Duration::from_micros(u64::try_from(micros).unwrap())
        )
    );
    drop(stream);
    handle
        .stop_cycle(paused_id.clone(), CommandContext::default())
        .await
        .unwrap();
    cycle(&handle, &paused_id, CycleStatus::Stopped).await;
    drop(handle);
    owner.shutdown().await.unwrap();
}
