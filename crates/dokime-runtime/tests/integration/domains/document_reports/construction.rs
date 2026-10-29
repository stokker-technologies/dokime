/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use super::*;
use dokime_domain::model;

// the default document shows every section in one composed order.

pub(super) async fn default_document_order(runtime: &RuntimeHandle) {
    let suite = suite_with_authored_document();
    register_custom_suite(runtime, suite.clone()).await;
    let test_cycle_id = start_cycle(runtime, suite).await;
    runtime.start_next_execution(test_cycle_id.clone(), CommandContext::default()).await;
    let cycle = wait_for_cycle_outcome(runtime, &test_cycle_id, ResultStatus::Passed).await;
    let execution = cycle.test_executions.first().unwrap();
    let default_document_id = dokime_runtime::DocumentId::from_identifier("default").unwrap();

    let execution_document = runtime
        .resolve_cycle_document(
            &test_cycle_id,
            &default_document_id,
            dokime_runtime::document::DocumentSelection {
                scope: RecordScope::default()
                    .with_cycle(test_cycle_id.clone())
                    .with_test_case(execution.test_case_id.clone())
                    .with_test_execution(execution.test_execution_id.clone()),
            },
        )
        .await
        .unwrap()
        .unwrap();

    // Nothing is selected and nothing is hidden: every section the definition
    // carries is in the one resolved document, in composition order.
    assert_eq!(
        section_identifiers(&execution_document),
        [
            "execution.result",
            "execution.traceability.notes",
            "custom.signals",
            "execution.events",
            "execution.traceability.artifacts",
            "execution.traceability.media",
            "execution.traceability.sections",
        ]
    );
    assert_eq!(
        card_kinds(&execution_document),
        ["steps", "criteria", "notes", "custom.signals.text", "records", "artifacts", "media", "traceability",]
    );
    // The runtime composes host sections only. A product's own card reaches the
    // document by being authored into a template, never by being injected here.
    let section_labels = section_metadata(&execution_document).into_iter().map(|metadata| metadata.label.as_str()).collect::<Vec<_>>();
    assert!(!section_labels.contains(&"Tracking Reliability"));
    assert!(execution_document.plot_descriptors.is_empty());
    assert!(execution_document.plot_inputs.is_empty());

    let unselected = runtime
        .resolve_cycle_document(
            &test_cycle_id,
            &default_document_id,
            dokime_runtime::document::DocumentSelection {
                scope: RecordScope::default().with_cycle(test_cycle_id.clone()),
            },
        )
        .await;
    assert!(unselected.is_err());
}

pub(super) async fn current_cycle_state(runtime: &RuntimeHandle) {
    use dokime_runtime::document::DocumentCardBody;
    let suite = suite_with_authored_document();
    register_custom_suite(runtime, suite.clone()).await;
    let cycle_id = start_cycle(runtime, suite).await;
    let document_id = dokime_runtime::DocumentId::from_identifier("default").unwrap();
    let prepared = runtime.prepare_current_cycle_document(&cycle_id, &document_id).unwrap();
    let revision = runtime.watch_state().latest().revision();
    runtime.start_next_execution(cycle_id.clone(), CommandContext::default()).await;
    wait_for_cycle_outcome(runtime, &cycle_id, ResultStatus::Passed).await;
    assert!(runtime.watch_state().latest().revision() > revision);
    let captured = prepared.capture().await.unwrap();
    assert_eq!(captured.state_revision, revision);
    assert_eq!(captured.capture.selection.scope.test_cycle_id.as_ref(), Some(&cycle_id));
    assert!(captured.capture.selection.scope.test_execution_id.is_none());
    assert!(captured.capture.selection.scope.test_case_id.is_none());
    // Host Sections Are Layout Nodes Over The Definition's Own Cards.
    let definition = &captured.capture.definition;
    let sections = definition
        .layout
        .nodes
        .iter()
        .filter_map(|node| match node {
            dokime_runtime::document::DocumentNode::Section(section) => Some(section),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        sections.iter().map(|section| section.metadata.identifier.as_str()).collect::<Vec<_>>(),
        ["cycle.events", "cycle.artifacts", "cycle.media"]
    );
    assert_eq!(
        sections
            .iter()
            .flat_map(|section| &section.children)
            .filter_map(|node| match node {
                dokime_runtime::document::DocumentNode::Card(placement) =>
                    definition.cards.iter().find(|card| card.metadata.identifier == placement.card_id),
                _ => None,
            })
            .map(|card| &card.body)
            .collect::<Vec<_>>(),
        [&DocumentCardBody::Records, &DocumentCardBody::Artifacts, &DocumentCardBody::Media]
    );
    assert_eq!(captured.capture.cards.len(), 3);
    assert!(captured.telemetry_cut.is_none());
    assert!(captured.artifacts_sha256.is_some());
    assert!(runtime.prepare_current_cycle_document(&cycle_id, &dokime_runtime::DocumentId::from_identifier("unknown").unwrap()).is_err());
    assert!(
        runtime
            .prepare_current_cycle_document(&dokime_runtime::model::identity::new_id::<dokime_runtime::TestCycleId>(), &document_id)
            .is_err()
    );
    // An Original Cycle Cut Cannot Acquire An Artifact From A Later Member.
    let original = runtime.prepare_current_cycle_document(&cycle_id, &document_id).unwrap();
    let case = runtime.watch_state().latest().state().cycle_view(&cycle_id).unwrap().test_executions[0].snapshot.case().clone();
    runtime.append_fixture_cases(cycle_id.clone(), vec![case], CommandContext::default()).await.unwrap();
    let scope = runtime.watch_state().latest().state().cycle_view(&cycle_id).unwrap().test_executions.last().unwrap().scope.clone();
    let upload = runtime
        .begin_artifact_upload(dokime_runtime::artifacts::ArtifactUploadInput {
            scope,
            filename: "later.bin".into(),
            content_type: Some("application/octet-stream".into()),
            size_bytes: 3,
            sha256: None,
        })
        .await
        .unwrap();
    upload.send(b"abc".to_vec()).await.unwrap();
    upload.finish().await.unwrap();
    drop(upload);
    let error = original.capture().await.err().expect("later member must be refused");
    assert!(
        matches!(error, dokime_runtime::runtime::Error::Admission(dokime_domain::model::diagnostics::admission::AdmissionFailure::Conflict{ ref code, .. }) if code == "document_capture_incomplete")
    );
    let current = runtime.prepare_current_cycle_document(&cycle_id, &document_id).unwrap().capture().await.unwrap();
    assert!(current.artifacts_sha256.is_some());
}

pub(super) async fn cycle_report_executions(runtime: &RuntimeHandle) {
    let suite = suite_with_authored_document();
    register_custom_suite(runtime, suite.clone()).await;
    let cycle_id = start_cycle(runtime, suite).await;
    let case = runtime.watch_state().latest().state().cycle_view(&cycle_id).unwrap().test_executions[0].snapshot.case().clone();
    runtime.append_fixture_cases(cycle_id.clone(), vec![case], CommandContext::default()).await.unwrap();
    runtime.start_next_execution(cycle_id.clone(), CommandContext::default()).await;
    wait_for_cycle_outcome(runtime, &cycle_id, ResultStatus::Passed).await;
    runtime.seal_cycle(cycle_id.clone(), CommandContext::default()).await.unwrap();
    let cycle = cycle_snapshot(runtime, &cycle_id).await;
    let mut member_payloads = Vec::new();
    let mut samples = Vec::new();
    for execution in &cycle.test_executions {
        let checkpoint = crate::service_support::native::checkpoint(runtime, &execution.test_execution_id).await;
        let captured_samples = crate::service_support::native::samples(runtime, &checkpoint).await;
        let html = crate::service_support::native::report(runtime, &execution.test_execution_id).await;
        let payload = crate::service_support::reports::parse(&html);
        samples
            .extend(captured_samples.into_iter().filter(|sample| payload["snapshot"]["telemetry"].get(sample.stream.as_str()).is_some()));
        let runs = payload["case_runs"].as_array().unwrap();
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0]["test_execution_id"], execution.test_execution_id.as_str());
        assert!(runs[0]["documents"]["default"].is_object());
        member_payloads.push(payload);
    }
    let payload = crate::service_support::reports::payload(runtime.storage(), &cycle.saved.as_ref().unwrap().report).await;
    let runs = member_payloads.iter().flat_map(|member| member["case_runs"].as_array().unwrap().iter().cloned()).collect::<Vec<_>>();
    assert_eq!(payload["case_runs"], serde_json::json!(runs));
    assert_eq!(payload["test_cycle_plan"]["cases"].as_array().unwrap().len(), 1);
    assert_eq!(payload["test_cycle_plan"]["queue"][0]["attempt_count"], 2);
    let artifacts =
        member_payloads.iter().flat_map(|member| member["snapshot"]["artifacts"].as_array().unwrap().iter().cloned()).collect::<Vec<_>>();
    assert_eq!(payload["snapshot"]["artifacts"], serde_json::json!(artifacts));
    let records =
        member_payloads.iter().flat_map(|member| member["snapshot"]["records"].as_array().unwrap().iter().cloned()).collect::<Vec<_>>();
    assert_eq!(payload["snapshot"]["records"], serde_json::json!(records));
    for (index, member) in member_payloads.iter().enumerate() {
        let mut expected = member["test_executions"][0].clone();
        expected["attempt_index"] = index.into();
        assert_eq!(payload["test_executions"][index], expected);
    }
    let telemetry = payload["snapshot"]["telemetry"].as_object().unwrap();
    assert_eq!(telemetry.values().map(|stream| stream["total_samples"].as_u64().unwrap() as usize).sum::<usize>(), samples.len());
    for stream in telemetry.values() {
        let values = stream["samples"].as_array().unwrap();
        assert!(values.windows(2).all(|pair| pair[0]["timestamp"].as_str().unwrap() <= pair[1]["timestamp"].as_str().unwrap()));
    }
}

// a section a run added is in the live document and in the stored report.

pub(super) async fn section_run_report() {
    let root = tempdir().unwrap();
    let (_owner, runtime) = booted_with_fixture(root.path(), Arc::new(DocumentAddingExecutor)).await;

    let suite = suite_with_authored_document();
    register_custom_suite(&runtime, suite.clone()).await;
    let test_cycle_id = start_cycle(&runtime, suite).await;
    runtime.start_next_execution(test_cycle_id.clone(), CommandContext::default()).await;
    let cycle = wait_for_cycle_outcome(&runtime, &test_cycle_id, ResultStatus::Passed).await;
    let execution = cycle.test_executions.first().unwrap();
    let default_document_id = dokime_runtime::DocumentId::from_identifier("default").unwrap();

    let live = runtime
        .resolve_cycle_document(
            &test_cycle_id,
            &default_document_id,
            dokime_runtime::document::DocumentSelection {
                scope: RecordScope::default()
                    .with_cycle(test_cycle_id.clone())
                    .with_test_case(execution.test_case_id.clone())
                    .with_test_execution(execution.test_execution_id.clone()),
            },
        )
        .await
        .unwrap()
        .unwrap();

    // The addition lands behind the section the template declared and in front
    // of the evidence built-ins, and the id repeated by the run appears once.
    assert_eq!(
        section_identifiers(&live),
        [
            "execution.result",
            "execution.traceability.notes",
            "custom.signals",
            "run.discovered",
            "execution.events",
            "execution.traceability.artifacts",
            "execution.traceability.media",
            "execution.traceability.sections",
        ]
    );
    assert_eq!(
        section_metadata(&live)
            .into_iter()
            .filter(|metadata| metadata.identifier.as_str() == "run.discovered")
            .map(|metadata| metadata.label.as_str())
            .collect::<Vec<_>>(),
        ["Discovered Streams"],
        "the label the run added is the one that survives a repeat"
    );

    // The same document, read back out of what the host stored rather than out
    // of the live cycle: the report is rendered from persisted checkpoints.
    let cycle = cycle_snapshot(&runtime, &test_cycle_id).await;
    let html = crate::service_support::native::report(&runtime, &cycle.test_executions[0].test_execution_id).await;

    assert!(html.contains("Discovered Streams"), "the stored report shows the section the run added");
    assert!(html.contains("The bench answered on one stream."), "the stored report shows the note that section carries");
}

pub(super) async fn current_fact_frontiers() {
    use dokime_runtime::document::{
        DocumentCard, TelemetrySummaryDefinition, TelemetrySummaryGroupDefinition, TelemetrySummaryTileDefinition,
    };
    use {
        model::records::product::ProductCalibrationDetails, model::records::product::ProductCalibrationKind,
        model::records::product::ProductCalibrationPhase,
    };

    fn readings(capture: &dokime_runtime::document::capture::Capture) -> Vec<Option<Value>> {
        capture
            .cards
            .iter()
            .filter_map(|binding| match &binding.result {
                dokime_runtime::document::capture::CardBindingResult::Summary { groups } => Some(groups),
                _ => None,
            })
            .flat_map(|groups| groups.iter().flat_map(|group| &group.tiles))
            .map(|tile| tile.value.clone())
            .collect()
    }

    let root = tempdir().unwrap();
    let executor = Arc::new(BlockingPublishingExecutor::new());
    let (mut owner, runtime) = started_with_fixture(config(root.path()), executor.clone()).await;
    let mut suite = fixture_suite(&["passed"]);
    suite.test_case_templates[0]
        .subscriptions
        .push(dokime_runtime::definition::Subscription::Record(Identifier::new("calibration.field.completed").unwrap()));
    let source = dokime_runtime::expressions::record("calibration.field.completed");
    let mut group = TelemetrySummaryGroupDefinition::new("records", "Captured Records");
    group.tile(TelemetrySummaryTileDefinition::new("Occurrences", source.clone().count()));
    group.tile(TelemetrySummaryTileDefinition::new("Resource", source.detail("resource_id")));
    group.tile(TelemetrySummaryTileDefinition::new("Files", dokime_runtime::expressions::artifact("trace.bin").count()));
    group.tile(TelemetrySummaryTileDefinition::new("Bytes", dokime_runtime::expressions::artifact("trace.bin").detail("size_bytes").max()));
    let mut summary = TelemetrySummaryDefinition::new();
    summary.group(group);
    let mut section = dokime_runtime::document::DocumentSection::new("Captured Records");
    section.card(DocumentCard::telemetry_summary(summary));
    let mut document = dokime_runtime::document::DocumentDefinition::new("default", "Capture");
    document.section(section);
    suite.test_case_templates[0].document = Some(document);
    register_python_suite(&runtime, suite.clone()).await;
    let cycle_id = start_cycle(&runtime, suite).await;
    runtime.start_next_execution(cycle_id.clone(), CommandContext::default()).await;
    executor.wait_until_entered().await;
    let cycle = wait_for_cycle_lifecycle(&runtime, &cycle_id, CycleStatus::Running).await;
    let execution_id = &cycle.test_executions[0].test_execution_id;
    let document_id = dokime_runtime::DocumentId::from_identifier("default").unwrap();
    let empty = runtime.prepare_current_document(&cycle_id, execution_id, &document_id).unwrap().capture().await.unwrap();
    assert_eq!(readings(&empty.capture), [Some(Value::Unsigned(0)), None, Some(Value::Unsigned(0)), None]);
    assert!(empty.telemetry_cut.is_none());

    runtime
        .publish_product_record(
            ProductCalibrationDetails::new("original", ProductCalibrationKind::Field, ProductCalibrationPhase::Completed)
                .into_draft("fixture"),
        )
        .await
        .unwrap();
    let prepared = runtime.prepare_current_document(&cycle_id, execution_id, &document_id).unwrap();
    runtime
        .publish_product_record(
            ProductCalibrationDetails::new("later", ProductCalibrationKind::Field, ProductCalibrationPhase::Completed)
                .into_draft("fixture"),
        )
        .await
        .unwrap();
    let original = prepared.capture().await.unwrap();
    assert_eq!(
        readings(&original.capture),
        [Some(Value::Unsigned(1)), Some(Value::Text("original".into())), Some(Value::Unsigned(0)), None,]
    );
    let upload = runtime
        .begin_artifact_upload(dokime_runtime::artifacts::ArtifactUploadInput {
            scope: cycle.test_executions[0].scope.clone(),
            filename: "trace.bin".into(),
            content_type: Some("application/octet-stream".into()),
            size_bytes: 3,
            sha256: None,
        })
        .await
        .unwrap();
    upload.send(b"abc".to_vec()).await.unwrap();
    upload.finish().await.unwrap();
    drop(upload);
    let current = runtime.prepare_current_document(&cycle_id, execution_id, &document_id).unwrap().capture().await.unwrap();
    assert_eq!(
        readings(&current.capture),
        [Some(Value::Unsigned(2)), Some(Value::Text("later".into())), Some(Value::Unsigned(1)), Some(Value::Unsigned(3))]
    );
    assert!(original.artifacts_sha256.is_some());
    assert_ne!(original.artifacts_sha256, current.artifacts_sha256);
    assert!(original.state_revision < current.state_revision);
    assert!(current.telemetry_cut.is_none());
    let evidence = runtime.evidence_snapshot().await.unwrap();
    let facts: Vec<_> = evidence.records.iter().filter(|record| record.record_type.as_str() == "calibration.field.completed").collect();
    assert_eq!(facts.len(), 2);
    assert!(facts.iter().all(|record| record.scope.is_none()));

    executor.release();
    wait_for_cycle_outcome(&runtime, &cycle_id, ResultStatus::Passed).await;
    drop(runtime);
    owner.shutdown().await.unwrap();
}

pub(super) async fn document_routes_samples() {
    use dokime_runtime::document::{
        DocumentCard, DocumentCardBodyPayload, DocumentPayload, DocumentSelection, TelemetrySummaryDefinition,
        TelemetrySummaryGroupDefinition, TelemetrySummaryTileDefinition,
    };

    fn reading(document: &DocumentPayload) -> Option<Value> {
        document.cards.iter().find_map(|card| match &card.body {
            DocumentCardBodyPayload::TelemetrySummary { groups } => groups.first()?.tiles.first()?.value.clone(),
            _ => None,
        })
    }

    let root = tempdir().unwrap();
    let executor = Arc::new(BlockingPublishingExecutor::new());
    let (mut owner, runtime) = started_with_fixture(wifi_config(root.path()), executor.clone()).await;
    let mut suite = fixture_suite(&["passed"]);
    let mut group = TelemetrySummaryGroupDefinition::new("captured", "Captured Sample");
    group.tile(TelemetrySummaryTileDefinition::new(
        "Sample Index",
        dokime_runtime::expressions::telemetry("host.wifi.scan", "sample_index"),
    ));
    let mut summary = TelemetrySummaryDefinition::new();
    summary.group(group);
    let mut section = dokime_runtime::document::DocumentSection::new("Captured Evidence");
    section.card(DocumentCard::telemetry_summary(summary));
    let mut document = dokime_runtime::document::DocumentDefinition::new("default", "Capture");
    document.section(section);
    suite.test_case_templates[0].document = Some(document);
    register_python_suite(&runtime, suite.clone()).await;
    let cycle_id = start_cycle(&runtime, suite).await;
    runtime.start_next_execution(cycle_id.clone(), CommandContext::default()).await;
    executor.wait_until_entered().await;
    let cycle = wait_for_cycle_lifecycle(&runtime, &cycle_id, CycleStatus::Running).await;
    let execution = &cycle.test_executions[0];
    let selection = DocumentSelection { scope: execution.scope.clone() };
    let document_id = dokime_runtime::DocumentId::from_identifier("default").unwrap();

    // Later uncaptured publications must not replace the execution's value.
    for (scope, count) in [(Some(execution.scope.clone()), 2), (Some(RecordScope::default().with_cycle(cycle_id.clone())), 5), (None, 7)] {
        let receipt = runtime.publish_telemetry_batch(samples("host.wifi.scan", count, scope)).await.unwrap();
        assert_eq!(receipt.accepted, count as u64);
    }

    runtime.storage().flush_until(&runtime.storage().cursor_accepted()).await.unwrap();
    let monitoring = TelemetrySelection {
        streams: vec![Identifier::new("host.wifi.scan").unwrap().into()],
        ..Default::default()
    };
    assert_eq!(runtime.collect_telemetry(&monitoring).await.unwrap().len(), 14);
    for test_cycle_id in [None, Some(cycle_id.clone())] {
        let captured = runtime
            .collect_telemetry(&TelemetrySelection {
                scope: dokime_runtime::domains::evidence::ScopeSelection {
                    test_cycle_id,
                    test_execution_id: Some(execution.test_execution_id.clone()),
                    ..Default::default()
                },
                ..monitoring.clone()
            })
            .await
            .unwrap();
        assert_eq!(captured.len(), 2);
        assert!(captured.iter().all(|sample| sample.scope.as_ref() == Some(&execution.scope)));
    }

    let direct = runtime.resolve_cycle_document(&cycle_id, &document_id, selection.clone()).await.unwrap().unwrap();
    assert_eq!(reading(&direct), Some(Value::Integer(1)));
    let current = runtime.prepare_current_document(&cycle_id, &execution.test_execution_id, &document_id).unwrap().capture().await.unwrap();
    let current_reading = current.capture.cards.iter().find_map(|binding| match &binding.result {
        dokime_runtime::document::capture::CardBindingResult::Summary { groups } => {
            groups.iter().flat_map(|group| &group.tiles).find_map(|tile| tile.value.clone())
        }
        _ => None,
    });
    assert_eq!(current_reading, Some(Value::Integer(1)));
    let view = runtime
        .query_runtime_view(RuntimeViewQuery {
            scope: Some(selection.scope),
            topic: RuntimeViewTopic::Document { document_id },
            window: RuntimeWindow::Full,
            resolution: RuntimeResolution::Summary,
            cursor: None,
        })
        .await
        .unwrap();
    let RuntimeViewPayload::Document { frame } = view.payload else {
        panic!("expected document frame");
    };
    assert!(frame.error.is_none(), "{:?}", frame.error);
    assert_eq!(reading(frame.document.as_ref().unwrap()), Some(Value::Integer(1)));

    executor.release();
    wait_for_cycle_outcome(&runtime, &cycle_id, ResultStatus::Passed).await;
    drop(runtime);
    owner.shutdown().await.unwrap();
}

pub(super) async fn repeated_sections_follow() {
    use dokime_runtime::document::{DocumentDefinition, DocumentSection, DocumentSelection};

    let root = tempdir().unwrap();
    let executor = Arc::new(BlockingPublishingExecutor::new());
    let (mut owner, runtime) = started_with_fixture(wifi_config(root.path()), executor.clone()).await;

    // Declare A Section Repeated Per Channel, Without An Explicit Order
    let channel = dokime_runtime::expressions::telemetry("host.wifi.scan", "wifi_channel");
    let mut section = DocumentSection::new("Channels");
    section.note("One section per channel the scan saw.");
    section.each(channel.clone(), "Channel {value}").only(channel.ne(13));
    let mut document = DocumentDefinition::new("default", "Channels");
    document.section(section);
    let mut suite = fixture_suite(&["passed"]);
    suite.test_case_templates[0].document = Some(document);
    register_python_suite(&runtime, suite.clone()).await;

    // Publish Channels Out Of Order Into The Running Execution
    let cycle_id = start_cycle(&runtime, suite).await;
    runtime.start_next_execution(cycle_id.clone(), CommandContext::default()).await;
    executor.wait_until_entered().await;
    let cycle = wait_for_cycle_lifecycle(&runtime, &cycle_id, CycleStatus::Running).await;
    let execution = &cycle.test_executions[0];
    let channels = [11, 1, 13, 6];
    let mut batch = samples("host.wifi.scan", channels.len(), Some(execution.scope.clone()));
    for (sample, channel) in batch.samples.iter_mut().zip(channels) {
        sample.values.insert("wifi_channel".to_owned(), Value::Integer(channel));
        // Saved Rows Come From The Typed Payload, So The Edited Sample Is Encoded Again.
        *sample = crate::service_support::contracts::typed_fixture_sample(sample.clone());
    }
    runtime.publish_telemetry_batch(batch).await.unwrap();

    runtime.storage().flush_until(&runtime.storage().cursor_accepted()).await.unwrap();

    // Materialized Sections Come Out In Ascending Channel Order
    let document = runtime
        .resolve_cycle_document(
            &cycle_id,
            &dokime_runtime::DocumentId::from_identifier("default").unwrap(),
            DocumentSelection { scope: execution.scope.clone() },
        )
        .await
        .unwrap()
        .unwrap();
    let labels = section_metadata(&document)
        .into_iter()
        .map(|metadata| metadata.label.as_str())
        .filter(|label| label.starts_with("Channel "))
        .collect::<Vec<_>>();
    assert_eq!(labels, ["Channel 1", "Channel 6", "Channel 11"]);

    executor.release();
    wait_for_cycle_outcome(&runtime, &cycle_id, ResultStatus::Passed).await;
    drop(runtime);
    owner.shutdown().await.unwrap();
}

fn document_nodes(document: &dokime_runtime::document::DocumentPayload) -> Vec<&dokime_runtime::document::DocumentNodePayload> {
    use dokime_runtime::document::DocumentNodePayload;
    let mut pending = document.layout.nodes.iter().rev().collect::<Vec<_>>();
    let mut ordered = Vec::new();
    while let Some(node) = pending.pop() {
        ordered.push(node);
        if let DocumentNodePayload::Section(section) = node {
            pending.extend(section.children.iter().rev());
        }
    }
    ordered
}

fn section_metadata(document: &dokime_runtime::document::DocumentPayload) -> Vec<&Metadata> {
    document_nodes(document)
        .into_iter()
        .filter_map(|node| match node {
            dokime_runtime::document::DocumentNodePayload::Section(section) => Some(&section.metadata),
            _ => None,
        })
        .collect()
}

fn section_identifiers(document: &dokime_runtime::document::DocumentPayload) -> Vec<&str> {
    section_metadata(document).into_iter().map(|metadata| metadata.identifier.as_str()).collect()
}

fn card_kinds(document: &dokime_runtime::document::DocumentPayload) -> Vec<&str> {
    document_nodes(document)
        .into_iter()
        .filter_map(|node| match node {
            dokime_runtime::document::DocumentNodePayload::Card(placement) => Some(
                document
                    .cards
                    .iter()
                    .find(|card| card.metadata.identifier == placement.card_id)
                    .expect("every placement references a card"),
            ),
            _ => None,
        })
        .map(|card| match &card.body {
            DocumentCardBodyPayload::Steps => "steps",
            DocumentCardBodyPayload::Criteria => "criteria",
            DocumentCardBodyPayload::Records => "records",
            DocumentCardBodyPayload::Artifacts { .. } => "artifacts",
            DocumentCardBodyPayload::Media { .. } => "media",
            DocumentCardBodyPayload::Traceability => "traceability",
            DocumentCardBodyPayload::Notes => "notes",
            _ => card.metadata.identifier.as_str(),
        })
        .collect()
}

fn suite_with_authored_document() -> TestSuite {
    let mut suite = custom_suite();
    let mut signals = dokime_runtime::document::DocumentSection::new("Custom Signals");
    signals.card(dokime_runtime::document::DocumentCard {
        metadata: metadata("custom.signals.text", "Custom Signals"),
        layout: Default::default(),
        filters: Vec::new(),
        body: dokime_runtime::document::DocumentCardBody::Text {
            markdown: "Authored by the template.".to_owned(),
        },
    });

    let mut document = dokime_runtime::document::DocumentDefinition::new("default", "Custom Template");
    document.section(signals);
    suite.test_case_templates[0].document = Some(document);
    suite
}

pub(super) async fn saved_report_evidence(runtime: &RuntimeHandle) {
    use dokime_runtime::document::{DocumentDefinition, DocumentSection, PlotDefinition, PlotType};
    let mut suite = custom_suite();
    let mut document = DocumentDefinition::new("default", "Analysis");
    let mut section = DocumentSection::new("Samples");
    let mut plot = PlotDefinition::with_type("Mean Distance", PlotType::TimeSeries);
    plot.id("analysis.mean-distance");
    plot.stream = Some("tracking".into());
    plot.x = Some("timestamp".into());
    plot.y = vec!["distance_m".into()];
    plot.analysis = Some(dokime_runtime::analysis::Selection {
        id: "dokime.analysis.mean".into(),
        version: 1,
        parameters: Default::default(),
    });
    section.plot(plot);
    document.section(section);
    suite.test_case_templates[0].document = Some(document);
    register_custom_suite(runtime, suite.clone()).await;
    let admitted =
        runtime.test_suites().await.into_iter().find(|entry| entry.suite.metadata.identifier == suite.metadata.identifier).unwrap().suite;
    let cycle_id = start_cycle(runtime, admitted).await;
    runtime.start_next_execution(cycle_id.clone(), CommandContext::default()).await;

    let cycle = wait_for_cycle_outcome(runtime, &cycle_id, ResultStatus::Passed).await;
    let execution = &cycle.test_executions[0];
    let html = crate::service_support::native::report(runtime, &execution.test_execution_id).await;
    let json = html.split_once(r#"<script id="hub-report-data" type="application/json">"#).unwrap().1.split_once("</script>").unwrap().0;
    let payload: serde_json::Value = serde_json::from_str(json).unwrap();
    assert_eq!(payload["plots"], serde_json::json!({}));
    assert_eq!(payload["documents"], serde_json::json!({}));
    let computed = payload["case_runs"][0]["plots"]
        .as_object()
        .unwrap()
        .values()
        .find_map(|plot| plot.get("computed"))
        .expect("saved analysis result");
    assert_eq!(computed["selection"]["id"], "dokime.analysis.mean");
    assert_eq!(computed["selection"]["version"], 1);
    assert_eq!(computed["series"][0]["input_samples"], 1);
    assert_eq!(computed["series"][0]["output"]["outputs"][0]["values"][0], 1.0);
    assert!(computed["source"]["position"]["sequence"].as_u64().is_some_and(|sequence| sequence > 0));
    assert!(computed["source"].get("cursor").is_none());
}
