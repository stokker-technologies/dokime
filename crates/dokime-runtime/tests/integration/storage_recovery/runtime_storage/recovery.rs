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
use base64::Engine;

// Local Imports
use crate::service_support::prelude::*;
use crate::service_support::system_reports::Runtime;
use dokime_domain::model::execution::result::CycleView;
use dokime_runtime::ValueType;
use dokime_runtime::domains::ExecutionProviderTarget;
use dokime_runtime::domains::execution::provider::{ExecutionFixtureContext, ExecutionFixtureFuture};
use dokime_runtime::telemetry::{TelemetryColumn, TelemetryContract, TelemetryRole};

/* === Constants === */
// Each Member Captures Wide Telemetry, A Large Shared Plan And Eight Artifacts.
const MEMBERS: usize = 9;
const COLUMNS: usize = 49;
const STREAM: &str = "fixture.wide";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
// Publishes Wide Telemetry and Artifacts into Every Member
struct WideTelemetryExecutor;

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl ExecutionFixture for WideTelemetryExecutor {
    fn run_cycle<'a>(
        &'a self, context: ExecutionFixtureContext, cycle: &'a dokime_domain::model::execution::result::CycleView,
        _target: &'a ExecutionProviderTarget,
    ) -> ExecutionFixtureFuture<'a> {
        Box::pin(async move {
            for execution in runnable_executions(cycle) {
                activate_fixture_execution(&context, execution).await;
                let scope = execution.scope.clone();
                context.publish_telemetry_batch(wide_batch(scope.clone())).await.unwrap();
                for index in 0..8 {
                    let upload = context
                        .begin_artifact_upload(dokime_runtime::artifacts::ArtifactUploadInput {
                            scope: scope.clone(),
                            filename: format!("measurement-{index}.bin"),
                            content_type: None,
                            size_bytes: 3,
                            sha256: None,
                        })
                        .await
                        .unwrap();
                    upload.send(vec![index, 1, 2]).await.unwrap();
                    upload.finish().await.unwrap();
                }
            }
            Ok(python_outcome(cycle, "passed", false))
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */
// Run One Wide Cycle to Its Seal and Return Its Final Resident View
async fn sealed_cycle(handle: &RuntimeHandle, suite: TestSuite) -> CycleView {
    let cycle_id = start_cycle(handle, suite).await;
    handle.start_next_execution(cycle_id.clone(), CommandContext::default()).await;
    wait_for_cycle_outcome(handle, &cycle_id, ResultStatus::Passed).await;
    handle.seal_cycle(cycle_id.clone(), CommandContext::default()).await.unwrap();
    until_state(handle, |state| state.cycle_views().any(|cycle| cycle.test_cycle_id == cycle_id && cycle.saved.is_some())).await;
    let cycle = cycle_snapshot(handle, &cycle_id).await;
    assert_eq!(cycle.test_executions.len(), MEMBERS);
    cycle
}

async fn resident(handle: &RuntimeHandle, cycle_id: &TestCycleId) -> bool {
    handle.cycles().await.unwrap().iter().any(|cycle| &cycle.test_cycle_id == cycle_id)
}

// Every Sealed Member Reopens from Native History with Its Wide Aggregates
async fn assert_history(handle: &RuntimeHandle, cycles: &[&CycleView], entries: usize) {
    let history = handle.browse_executions(Default::default()).await.unwrap();
    assert!(history.discovery_complete && history.conflicts.is_empty());
    assert_eq!(history.entries.len(), entries);
    drop(history);
    for cycle in cycles {
        for (index, member) in cycle.test_executions.iter().enumerate() {
            let checkpoint = crate::service_support::native::checkpoint(handle, &member.test_execution_id).await;
            let view = crate::service_support::native::open(handle, &checkpoint).await;
            assert_eq!(view.execution().test_cycle_id, cycle.test_cycle_id);
            assert!(view.execution().saved.is_some());
            if index == 0 {
                let aggregates = view.execution().output_parameters.keys().filter(|key| key.starts_with("telemetry.")).count();
                assert!(aggregates >= COLUMNS * 13);
            }
            let artifacts: Vec<_> = view
                .execution()
                .saved
                .as_ref()
                .unwrap()
                .source
                .outputs
                .iter()
                .filter_map(|output| match &output.content {
                    dokime_domain::model::execution::saved::ExecutionOutputContent::Artifact { record, .. } => Some(record),
                    _ => None,
                })
                .collect();
            assert_eq!(artifacts.len(), 8);
            for artifact in artifacts {
                let content = handle.read_artifact_content(&artifact.artifact_id, artifact.size_bytes as usize).await.unwrap().unwrap();
                let index = artifact.filename.strip_prefix("measurement-").unwrap().strip_suffix(".bin").unwrap().parse::<u8>().unwrap();
                assert_eq!(content.bytes, vec![index, 1, 2]);
            }
        }
    }
}

async fn reboot(root: &Path) -> (Runtime, RuntimeHandle) {
    started_with_fixture(wide_config(root), Arc::new(WideTelemetryExecutor)).await
}

fn wide_config(root: &Path) -> RuntimeConfig {
    let columns = (0..COLUMNS)
        .map(|index| {
            TelemetryColumn::new(metadata(&format!("value_{index:02}"), &format!("Value {index}")), ValueType::Number, TelemetryRole::Value)
                .optional()
        })
        .collect();
    let contract = TelemetryContract {
        columns,
        interval: Some(dokime_runtime::DurationMillis(1_000)),
        ..TelemetryContract::declared(STREAM, "Fixture Wide").with_storage_schema(wide_schema())
    };
    let mut settings = config(root).with_installed_plugins([dokime_runtime::InstalledPlugin {
        source: "fixture_host".to_owned(),
        version: "test".to_owned(),
        description: "Fixture Host".to_owned(),
        enabled: true,
        telemetry_contracts: vec![contract],
        ..Default::default()
    }]);
    settings.managed_storage = dokime_runtime::config::managed_storage_defaults();
    settings
}

fn wide_schema() -> harmos::schema::graph::Schema {
    use harmos::schema::{
        Cardinality,
        graph::{Field, Kind, Schema},
    };
    let mut fields = vec![Field {
        name: "timestamp".into(),
        tag: 1,
        kind: Kind::String,
        cardinality: Cardinality::Required,
    }];
    fields.extend((0..COLUMNS).map(|index| Field {
        name: format!("value_{index:02}").into(),
        tag: index as u32 + 2,
        kind: Kind::F64,
        cardinality: Cardinality::Required,
    }));
    Schema { root: 0, definitions: vec![fields] }
}

fn wide_batch(scope: RecordScope) -> TelemetryBatch {
    let samples = (0..4)
        .map(|sequence| {
            let mut sample = TelemetrySample::new(
                dokime_runtime::model::identity::new_id(),
                identifier(STREAM),
                sequence,
                dokime_runtime::model::time::now_utc(),
                "fixture_host",
            );
            let mut payload = Vec::new();
            harmos::message::postcard::Value::write(&sample.timestamp.as_str().to_owned(), &mut payload);
            for index in 0..COLUMNS {
                let value = index as f64 + sequence as f64 / 4.0;
                harmos::message::postcard::Value::write(&value, &mut payload);
                sample.values.insert(format!("value_{index:02}"), Value::Number(value));
            }
            sample.storage_payload = Some(payload);
            sample.scope = Some(scope.clone());
            sample
        })
        .collect();
    TelemetryBatch::new(dokime_runtime::model::identity::new_id(), identifier(STREAM), samples)
}

// Shutdown Publishes A Snapshot And Compacts Every Covered Journal Entry.
async fn shut_down(mut owner: Runtime, handle: RuntimeHandle) -> u64 {
    let root = handle.storage_root();
    let limits = dokime_runtime::config::storage_state_defaults();
    let journal = std::fs::read(root.join("state.jsonl")).unwrap();
    let largest = journal.split_inclusive(|byte| *byte == b'\n').map(<[u8]>::len).max().unwrap();
    assert!(
        largest <= limits.max_entry_bytes.get(),
        "the session's largest journal entry of {largest} bytes against the state entry bound"
    );
    drop(handle);
    owner.shutdown().await.unwrap();

    let snapshot = std::fs::read(root.join("state.json")).expect("shutdown writes a state snapshot");
    assert!(snapshot.len() <= limits.max_checkpoint_bytes.get());
    let record: serde_json::Value = serde_json::from_slice(&snapshot).unwrap();
    let compacted = std::fs::read(root.join("state.jsonl")).unwrap();
    assert!(compacted.len() < journal.len(), "snapshot compacts the journal");
    let rows: Vec<serde_json::Value> =
        compacted.split(|byte| *byte == b'\n').filter(|row| !row.is_empty()).map(|row| serde_json::from_slice(row).unwrap()).collect();
    assert_eq!(rows[0]["value"]["after"], record["value"]["through"]);
    assert!(rows.iter().all(|row| row["value"].get("payload").is_none()), "restart has no old journal entries to replay");
    let payload = record["value"]["payload"].as_str().unwrap();
    let bytes = base64::engine::general_purpose::STANDARD_NO_PAD.decode(payload).unwrap();
    let checkpoint: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let position = checkpoint["position"].as_u64().unwrap();
    eprintln!("SNAPSHOT payload_bytes={} position={position} journal_bytes={}", bytes.len(), compacted.len());
    position
}

// The Snapshot Encoding Stores Shared Catalogs And Plans Once; Decoding Restores Them
fn assert_round_trip(handle: &RuntimeHandle) {
    let latest = handle.watch_state().latest();
    let encoded = serde_json::to_vec(latest.state()).unwrap();
    let fields: serde_json::Value = serde_json::from_slice(&encoded).unwrap();
    eprintln!("STATE bytes={} executions={} cycles={}", encoded.len(), latest.state().executions.len(), latest.state().cycles.len());
    for (name, value) in fields.as_object().unwrap() {
        let size = serde_json::to_vec(value).unwrap().len();
        if size > 1000 {
            eprintln!("STATE field={name} bytes={size}");
        }
    }
    if let Some(execution) = latest.state().executions.values().next() {
        let value = serde_json::to_value(execution).unwrap();
        for (name, value) in value.as_object().unwrap() {
            eprintln!("EXECUTION field={name} bytes={}", serde_json::to_vec(value).unwrap().len());
        }
    }
    let decoded: dokime_runtime::state::RuntimeState = serde_json::from_slice(&encoded).unwrap();
    assert!(&decoded == latest.state(), "state survives its snapshot encoding");
}

/* === Tests === */

// Sealed Evidence Stays In Native History; Snapshots And Journal Compaction
// Recover Control State Without Replaying The Session's Complete History.
pub(crate) async fn bounded_snapshot_recovery() {
    let root = tempdir().unwrap();
    let mut suite = fixture_suite(&["passed"]);
    let template = &mut suite.test_case_templates[0];
    let mut member = template.parameters[0].clone();
    member.parameter_id = dokime_runtime::model::identity::new_id();
    member.metadata = metadata("member", "Member");
    member.value_type = ValueType::Integer;
    member.expansion = dokime_runtime::definition::ParameterExpansion::Cartesian;
    member.expression = dokime_runtime::definition::ParameterExpr::Array {
        values: (0..MEMBERS)
            .map(|index| dokime_runtime::definition::ParameterExpr::Literal { value: Value::Integer(index as i64) })
            .collect(),
    };
    template.parameters.push(member);
    let mut document = dokime_runtime::document::DocumentDefinition::new("Session");
    let mut section = dokime_runtime::document::DocumentSection::new("Definition");
    section.card(dokime_runtime::document::DocumentCard {
        metadata: metadata("definition", "Definition"),
        layout: Default::default(),
        filters: Vec::new(),
        body: dokime_runtime::document::DocumentCardBody::Text {
            markdown: "0123456789abcdef".repeat(4096),
        },
    });
    let mut plot = dokime_runtime::document::Plot::time_series("Samples");
    plot.y(dokime_runtime::expressions::telemetry(STREAM, "value_00"));
    section.plot(plot);
    document.section(section);
    template.document = Some(document);

    // The Latest Sealed Cycle Stays Resident Within Its Session
    let (owner, handle) = reboot(root.path()).await;
    register_python_suite(&handle, suite.clone()).await;
    let first = sealed_cycle(&handle, suite.clone()).await;
    let registration = dokime_runtime::model::identity::new_id::<dokime_domain::model::identity::CommandId>();
    let cases = first.test_executions.iter().map(|execution| execution.snapshot.case().clone()).collect::<Vec<_>>();
    handle.register_cases(registration.clone(), cases.clone()).await.unwrap();
    let requests = handle.watch_state().latest().state().run_requests.keys().cloned().collect::<Vec<_>>();
    assert!(resident(&handle, &first.test_cycle_id).await);
    assert_round_trip(&handle);
    let mut before = handle.watch_state().latest().state().clone();
    for execution in &first.test_executions {
        before.executions.insert(execution.test_execution_id.clone(), execution.clone());
    }
    let encoded = serde_json::to_vec(&before).unwrap();
    let decoded: dokime_runtime::state::RuntimeState = serde_json::from_slice(&encoded).unwrap();
    assert!(decoded == before, "snapshot encoding retains complete generated outputs while work is still resident");
    let mut original = serde_json::to_value(&before).unwrap();
    for execution in &first.test_executions {
        let member = &mut original["executions"]["members"][execution.test_execution_id.as_str()];
        member.as_object_mut().unwrap().remove("telemetry_outputs");
        member["execution"]["output_parameters"] = serde_json::to_value(&execution.output_parameters).unwrap();
    }
    let before_bytes = serde_json::to_vec(&original).unwrap().len();
    let after_bytes = serde_json::to_vec(handle.watch_state().latest().state()).unwrap().len();
    for (field, value) in serde_json::to_value(&first.test_executions[0]).unwrap().as_object().unwrap() {
        eprintln!("BEFORE EXECUTION field={field} bytes={}", serde_json::to_vec(value).unwrap().len());
    }
    eprintln!(
        "STATE COMPARISON executions={MEMBERS} before={before_bytes} after={after_bytes} before_per_execution={} after_per_execution={}",
        before_bytes / MEMBERS,
        after_bytes / MEMBERS
    );
    let after = serde_json::to_value(handle.watch_state().latest().state()).unwrap();
    eprintln!(
        "EXECUTION POOL before_per_execution={} after_per_execution={}",
        serde_json::to_vec(&original["executions"]).unwrap().len() / MEMBERS,
        serde_json::to_vec(&after["executions"]).unwrap().len() / MEMBERS,
    );
    assert!(after_bytes < before_bytes, "sealed state references durable evidence");
    let saved_position = shut_down(owner, handle).await;

    // A Restart Retires It from Live State Without Touching Its History
    let (owner, handle) = reboot(root.path()).await;
    assert!(handle.watch_state().latest().revision() > saved_position, "restart restores the compacted snapshot frontier");
    assert!(!resident(&handle, &first.test_cycle_id).await);
    for request in requests {
        assert!(handle.run_request(&request).await.unwrap().unwrap().summary.phase.terminal());
    }
    assert!(
        handle.register_cases(registration, cases[..1].to_vec()).await.is_err(),
        "retired registrations keep their immutable membership"
    );
    assert_history(&handle, &[&first], MEMBERS).await;

    // Starting Another Cycle Retires the Previous Sealed One in Session
    register_python_suite(&handle, suite.clone()).await;
    let second = sealed_cycle(&handle, suite.clone()).await;
    assert!(resident(&handle, &second.test_cycle_id).await);
    for _ in 0..2 {
        let current = sealed_cycle(&handle, suite.clone()).await;
        let state = handle.watch_state().latest();
        assert_eq!(state.state().executions.len(), current.test_executions.len());
        assert!(state.state().run_requests.values().all(|request| request.summary.cycle_id == current.test_cycle_id));
        assert_eq!(
            state.state().suites.cases_for_suite(&current.test_executions[0].snapshot.case().suite_id).count(),
            current.cases().count()
        );
    }
    let third = handle
        .create_cycle(
            dokime_runtime::transactions::CycleDefinition {
                metadata: metadata("reused", "Reused Cases"),
                continue_automatically: true,
            },
            CommandContext::default(),
        )
        .await
        .unwrap();
    let mut changed = first.test_executions[0].snapshot.case().clone();
    changed.parameters.insert("member".into(), Value::Integer(-1));
    assert!(handle.register_cases(dokime_runtime::model::identity::new_id(), vec![changed]).await.is_err());
    let reused =
        first.test_executions.iter().map(|execution| (dokime_runtime::model::identity::new_id(), execution.test_case_id.clone())).collect();
    handle.append_executions(third.clone(), reused).await.unwrap();
    assert!(
        handle
            .append_executions(
                third.clone(),
                vec![(first.test_executions[0].test_execution_id.clone(), first.test_executions[0].test_case_id.clone())]
            )
            .await
            .is_err(),
        "retired execution identities remain unavailable"
    );
    assert!(!resident(&handle, &second.test_cycle_id).await);
    assert!(resident(&handle, &third).await);
    let saved_position = shut_down(owner, handle).await;

    // The Next Boot Restores That Snapshot and Writes Its Own
    let (owner, handle) = reboot(root.path()).await;
    assert!(handle.watch_state().latest().revision() > saved_position, "restart restores the compacted snapshot frontier");
    assert!(!resident(&handle, &second.test_cycle_id).await);
    // The Third Cycle Was Started But Never Run; Its Planned Members Are Listed Too
    assert_history(&handle, &[&first, &second], 5 * MEMBERS).await;
    shut_down(owner, handle).await;
}
