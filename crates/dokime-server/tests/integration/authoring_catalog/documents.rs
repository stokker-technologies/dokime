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
use model::documents::wire::DocumentFrameFrame as Frame;
use {
    dokime::Value, dokime::document::DocumentCard, dokime::document::DocumentCardBody, dokime::document::DocumentDefinition,
    dokime::document::DocumentSection, dokime::runtime::command::CommandContext,
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[dokime::harmos::message(harmos_path = dokime::harmos)]
struct BenchRow {
    elapsed_s: f64,
    voltage_v: f64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

pub(crate) async fn current_document_rows() {
    let root = tempfile::tempdir().unwrap();
    let prepared_owner = dokime_server::grpc::Service::prepare(keyed()).unwrap();
    let mut owner = runtime(root.path(), &prepared_owner).await;
    let handle = owner.handle();
    let mut suite = support::fixture_suite(&["passed"]);
    let mut measurements = DocumentSection::new("Measurements");
    measurements.card(DocumentCard {
        metadata: support::metadata("table", "All rows"),
        layout: Default::default(),
        filters: Vec::new(),
        body: DocumentCardBody::Table {
            rows: (0..300)
                .map(|index| std::collections::BTreeMap::from([("ordinal".into(), Value::Integer(index)), ("absent".into(), Value::Null)]))
                .collect(),
        },
    });
    let mut authored = DocumentDefinition::new("default", "Authored");
    authored.section(measurements);
    suite.test_case_templates[0].document = Some(authored);
    support::register_fixture_execution(&handle, suite.clone()).await;
    let cycle = support::cycle_with_cases(
        &handle,
        support::metadata("document.fixture", "Document"),
        support::materialize_cases(suite.metadata.identifier.as_str(), &suite),
    )
    .await;
    let server = prepared_owner.ready().await.unwrap().listen().await.unwrap();
    let client = WireClient::connect(server.local_endpoint()).await.unwrap();
    let selected = model::documents::wire::DocumentScope {
        cycle_id: cycle.to_string(),
        execution_id: handle.watch_state().latest().state().cycles[&cycle].execution_ids[0].to_string(),
    };
    assert_eq!(
        client
            .download::<contracts::documents::CaptureCurrentDocument>(contracts::documents::CaptureCurrentDocument {
                scope: Some(selected.clone()),
                document_id: "default".into(),
            })
            .await
            .unwrap_err()
            .code(),
        Code::Unauthenticated
    );
    // A scope without an execution names the cycle's own document: its first
    // frame is a header scoped to the cycle alone.
    let cycle_header = client
        .download::<contracts::documents::CaptureCurrentDocument>(authenticated(contracts::documents::CaptureCurrentDocument {
            scope: Some(model::documents::wire::DocumentScope {
                cycle_id: cycle.to_string(),
                execution_id: String::new(),
            }),
            document_id: "default".into(),
        }))
        .await
        .unwrap()
        .into_inner()
        .message()
        .await
        .unwrap()
        .unwrap();
    let Some(Frame::Header(cycle_presentation)) = cycle_header.frame else {
        panic!("a cycle document opens with its header");
    };
    let cycle_scope = cycle_presentation.scope.unwrap();
    assert_eq!(cycle_scope.cycle_id, Some(cycle.to_string()));
    assert_eq!(cycle_scope.execution_id, None);
    let epoch = handle.process_epoch().to_string();
    let mut stream = client
        .download::<contracts::documents::CaptureCurrentDocument>(authenticated(contracts::documents::CaptureCurrentDocument {
            scope: Some(selected.clone()),
            document_id: "default".into(),
        }))
        .await
        .unwrap()
        .into_inner();
    let mut rows = Vec::new();
    let mut parents = std::collections::BTreeSet::new();
    let mut complete = None;
    let mut frames = 0;
    while let Some(frame) = stream.message().await.unwrap() {
        frames += 1;
        match frame.frame.unwrap() {
            Frame::Header(presentation) => {
                assert_eq!(frames, 1);
                assert_eq!(presentation.scope.unwrap().cycle_id, Some(cycle.to_string()));
                let frontiers = presentation.frontiers.unwrap();
                assert_eq!(frontiers.process_epoch, epoch);
                assert!(frontiers.telemetry.is_none());
            }
            Frame::LayoutNode(node) => {
                if let Some(parent) = node.parent_ordinal {
                    assert!(parents.contains(&parent));
                }
                assert!(parents.insert(node.ordinal));
            }
            Frame::TableRow(row) => {
                assert_eq!(row.card_id, "table");
                let Value::Object(fields) = row.values.unwrap().to_core().unwrap() else {
                    panic!("table row values must be an object");
                };
                assert_eq!(fields["ordinal"], Value::Integer(row.ordinal as i64));
                assert_eq!(fields["absent"], Value::Null);
                rows.push(row.ordinal);
            }
            Frame::Completion(completion) => complete = Some(completion),
            _ => {}
        }
    }
    assert_eq!(rows, (0..300).collect::<Vec<_>>());
    assert_eq!(complete.unwrap().frame_count, frames);
    let unknown = client
        .download::<contracts::documents::CaptureCurrentDocument>(authenticated(contracts::documents::CaptureCurrentDocument {
            scope: Some(selected),
            document_id: "missing".into(),
        }))
        .await
        .unwrap()
        .into_inner()
        .message()
        .await
        .unwrap_err();
    assert_eq!(unknown.code(), Code::NotFound);
    server.shutdown().await.unwrap();
    assert!(!handle.is_shutting_down());
    assert_eq!(handle.watch_state().latest().state().cycles[&cycle].test_cycle_id, cycle);
    drop(handle);
    owner.shutdown().await.unwrap();
}

pub(crate) async fn summary_complete_telemetry() {
    use {
        dokime::Timestamp, dokime::document::TelemetrySummaryDefinition, dokime::document::TelemetrySummaryGroupDefinition,
        dokime::document::TelemetrySummaryTileDefinition, dokime::telemetry::TelemetryBatch, dokime::telemetry::TelemetrySample,
    };
    let root = tempfile::tempdir().unwrap();
    let gate = std::sync::Arc::new(Gate::default());
    let prepared_owner = dokime_server::grpc::Service::prepare(Config::local()).unwrap();
    let mut owner = Runtime::with_execution_fixture(
        prepared_owner.configuration(crate::support::budgets::runtime_config(root.path(), support::metadata("grpc.summary", "Summary"))),
        "fixture",
        gate.clone(),
    )
    .await
    .unwrap();
    let handle = owner.handle();
    let mut suite = support::fixture_suite(&["passed"]);
    let mut group = TelemetrySummaryGroupDefinition::new("summary", "Progress");
    group.tile(TelemetrySummaryTileDefinition::new("Latest", dokime::expressions::telemetry("fixture.progress", "progress")));
    let mut summary = TelemetrySummaryDefinition::new();
    summary.group(group);

    let mut progress = DocumentSection::new("Progress");
    progress.card(DocumentCard::telemetry_summary(summary));
    let mut telemetry_document = DocumentDefinition::new("default", "Telemetry");
    telemetry_document.section(progress);
    suite.test_case_templates[0].document = Some(telemetry_document);
    support::register_fixture_progress_execution(&handle, suite.clone()).await;
    let cycle = support::cycle_with_cases(
        &handle,
        support::metadata("summary.fixture", "Summary"),
        support::materialize_cases(suite.metadata.identifier.as_str(), &suite),
    )
    .await;
    let execution_id = handle.watch_state().latest().state().cycles[&cycle].execution_ids[0].clone();
    handle.start_execution(execution_id, model::execution::ExecutionMode::Automatic, CommandContext::default()).await;
    gate.started.notified().await;
    let scope = handle.watch_state().latest().state().cycle_view(&cycle).unwrap().test_executions[0].scope.clone();
    assert!(scope.test_execution_id.is_some());
    let stream = Identifier::new("fixture.progress").unwrap();
    let mut samples: Vec<_> = (0..300)
        .map(|index| {
            let mut sample = TelemetrySample::new(
                dokime::model::identity::new_id(),
                stream.clone(),
                index,
                Timestamp::new("2026-09-13T00:00:00Z"),
                "fixture",
            );
            sample.values.insert("progress".into(), Value::Number(index as f64));
            sample.scope = Some(scope.clone());
            sample.with_storage_payload(dokime::harmos::Message::encode(&support::ProgressRow { progress: index as f64 }))
        })
        .collect();
    let mut unrelated =
        TelemetrySample::new(dokime::model::identity::new_id(), stream.clone(), 300, Timestamp::new("2026-09-13T00:00:00Z"), "fixture");
    unrelated.values.insert("progress".into(), Value::Number(999.0));
    samples.push(unrelated.with_storage_payload(dokime::harmos::Message::encode(&support::ProgressRow { progress: 999.0 })));
    handle.publish_telemetry_batch(TelemetryBatch::new(dokime::model::identity::new_id(), stream, samples)).await.unwrap();
    let durable = handle.storage().flush().await.unwrap();
    let server = prepared_owner.ready().await.unwrap().listen().await.unwrap();
    let client = WireClient::connect(server.local_endpoint()).await.unwrap();
    let mut stream = client
        .download::<contracts::documents::CaptureCurrentDocument>(contracts::documents::CaptureCurrentDocument {
            scope: Some(model::documents::wire::DocumentScope {
                cycle_id: cycle.to_string(),
                execution_id: handle.watch_state().latest().state().cycles[&cycle].execution_ids[0].to_string(),
            }),
            document_id: "default".into(),
        })
        .await
        .unwrap()
        .into_inner();
    let mut tile = None;
    let mut complete = false;
    while let Some(frame) = stream.message().await.unwrap() {
        match frame.frame.unwrap() {
            Frame::Header(presentation) => {
                let cut = presentation.frontiers.unwrap().telemetry.unwrap();
                assert_eq!(cut.store, durable.history.store.as_str());
                assert_eq!(cut.epoch, durable.history.epoch.as_str());
                assert!(cut.position >= durable.sequence);
                assert!(cut.position <= handle.storage().cursor_saved().sequence);
            }
            Frame::SummaryTile(value) => tile = value.value,
            Frame::Completion(_) => complete = true,
            _ => {}
        }
    }
    assert!(complete);
    assert_eq!(tile.unwrap().to_core().unwrap(), Value::Number(299.0));
    gate.release.notify_one();
    server.shutdown().await.unwrap();
    drop(handle);
    owner.shutdown().await.unwrap();
}

pub(crate) async fn duration_x_axis() {
    use dokime::definition::SuiteSet;
    use dokime::document::Plot;
    use dokime::domains::TestSuiteDefinition;
    use dokime::expressions::telemetry;
    use dokime::telemetry::{TelemetryColumn, TelemetryContract, TelemetryRole};
    use dokime::{Identifier, MeasurementUnit, Metadata, ValueType};

    let root = tempfile::tempdir().unwrap();
    let prepared_owner = dokime_server::grpc::Service::prepare(Config::local()).unwrap();
    let mut owner = runtime(root.path(), &prepared_owner).await;
    let handle = owner.handle();
    let mut suite = support::fixture_suite(&["passed"]);
    let mut voltage = Plot::xy_line("Voltage");
    voltage.id("bench.voltage");
    voltage.x(telemetry("bench.samples", "elapsed_s"));
    voltage.y(telemetry("bench.samples", "voltage_v"));
    let mut waveform = DocumentSection::new("Waveforms");
    waveform.plot(voltage);
    let mut authored = DocumentDefinition::new("default", "Authored");
    authored.section(waveform);
    suite.test_case_templates[0].document = Some(authored);

    let mut contract = TelemetryContract::declared("bench.samples", "Bench Samples");
    contract.storage_schema = Some(
        dokime::harmos::schema::graph::Schema::from_declaration(
            <BenchRow as dokime::harmos::schema::Described>::DEFINITION,
            &dokime::harmos::config::SchemaSettings::default(),
        )
        .unwrap(),
    );
    contract.columns = vec![
        TelemetryColumn::declared("elapsed_s", "Elapsed", ValueType::Number, TelemetryRole::Value).with_unit(MeasurementUnit::custom(
            Identifier::new("unit.s").unwrap(),
            "s",
            "s",
        )),
        TelemetryColumn::new(
            Metadata::new(Identifier::new("voltage_v").unwrap(), "Voltage", None),
            ValueType::Number,
            TelemetryRole::Value,
        ),
    ];
    let registration = TestSuiteDefinition {
        suite_id: suite.metadata.identifier.to_string(),
        tags: vec!["fixture".to_owned()],
        workflow: Some("execute_cycle".to_owned()),
        suite: suite.clone(),
        record_contracts: Vec::new(),
        telemetry_contracts: vec![contract],
    };
    handle.set_suites("provider:fixture", SuiteSet::new([registration])).await.unwrap();
    let cycle = support::cycle_with_cases(
        &handle,
        support::metadata("axis.fixture", "Axis"),
        support::materialize_cases(suite.metadata.identifier.as_str(), &suite),
    )
    .await;
    let server = prepared_owner.ready().await.unwrap().listen().await.unwrap();
    let client = WireClient::connect(server.local_endpoint()).await.unwrap();
    let mut stream = client
        .download::<contracts::documents::CaptureCurrentDocument>(contracts::documents::CaptureCurrentDocument {
            scope: Some(model::documents::wire::DocumentScope {
                cycle_id: cycle.to_string(),
                execution_id: handle.watch_state().latest().state().cycles[&cycle].execution_ids[0].to_string(),
            }),
            document_id: "default".into(),
        })
        .await
        .unwrap()
        .into_inner();
    let mut presentations = Vec::new();
    while let Some(frame) = stream.message().await.unwrap() {
        if let Frame::Binding(binding) = frame.frame.unwrap()
            && let Some(model::documents::wire::DocumentBindingResult::Plot(plot)) = binding.result
        {
            presentations.push(plot.presentation.unwrap());
        }
    }

    assert_eq!(presentations.len(), 1, "one plot card crosses the wire");
    assert_eq!(presentations[0].x_axis, model::documents::wire::PlotAxisKind::Elapsed, "the declared duration unit reached the wire");
    assert_eq!(presentations[0].x_axis_unit.as_deref(), Some("s"));

    server.shutdown().await.unwrap();
    drop(handle);
    owner.shutdown().await.unwrap();
}

/* === Tests === */
