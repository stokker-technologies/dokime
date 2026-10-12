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
use dokime::core::Value;
use dokime_app::capabilities::evidence::{ScopeSelection, TelemetrySelection};
use dokime_app::execution::CycleView;

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
    let settings = dokime_app::config::StorageReadSettings::default();
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
    let settings = dokime_app::config::StorageReadSettings::default();
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
    let id = handle
        .start_cycle(
            CycleDefinition {
                suite: suite.clone(),
                metadata: metadata("native.capture", "Native Capture"),
                cases: materialize_cases("fixture.sidecar", &suite),
                continue_on_failure: false,
            },
            CommandContext::default(),
        )
        .await
        .unwrap();
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
                return rows;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("native rows reach public telemetry")
}

/* === Tests === */

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
    let scoped = handle
        .collect_telemetry(&TelemetrySelection {
            streams: vec![Identifier::new(STREAM).unwrap()],
            scope: ScopeSelection {
                test_execution_id: Some(execution.test_execution_id.clone()),
                ..Default::default()
            },
            ..Default::default()
        })
        .await;
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
