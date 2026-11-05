/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Document Reports Story: documents constructed from live and stored evidence,
// captured at their cut, rendered into cycle and execution reports, sealed,
// downloaded and read back after a reboot.

/* === Modules === */

mod capture;
mod construction;
mod report_payloads;
mod reports;

/* === Re-Exports === */
pub(crate) use crate::service_support::prelude::*;

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[dokime_tracing::trace_test]
#[tokio::test]
async fn document_evidence_ignores_corrupt_other_executions() {
    use dokime_domain::model::execution::saved::{ExecutionOutput, ExecutionOutputContent};
    use dokime_domain::model::records::product::{ProductCalibrationDetails, ProductCalibrationKind, ProductCalibrationPhase};
    use dokime_runtime::storage::objects::execution::{self, Execution};
    use harmos::storage::{FileMetadata, ObjectVersion};

    let root = tempdir().unwrap();
    let executor = Arc::new(SecondExecutionHoldExecutor::default());
    let (mut owner, runtime) = started_with_fixture(config(root.path()), executor.clone()).await;
    let mut suite = fixture_suite(&["passed"]);
    use dokime_runtime::document::{
        DocumentCard, DocumentDefinition, DocumentSection, TelemetrySummaryDefinition, TelemetrySummaryGroupDefinition,
        TelemetrySummaryTileDefinition,
    };
    let mut group = TelemetrySummaryGroupDefinition::new("records", "Records");
    group.tile(TelemetrySummaryTileDefinition::new("Count", dokime_runtime::expressions::record("calibration.field.completed").count()));
    let mut summary = TelemetrySummaryDefinition::new();
    summary.group(group);
    let mut section = DocumentSection::new("Captured Records");
    section.card(DocumentCard::telemetry_summary(summary));
    let mut document = DocumentDefinition::new("default", "Evidence");
    document.section(section);
    suite.test_case_templates[0].document = Some(document);
    suite.test_case_templates[0]
        .subscriptions
        .push(dokime_runtime::definition::Subscription::Record(Identifier::new("calibration.field.completed").unwrap()));
    register_python_suite(&runtime, suite.clone()).await;
    let other_cycle_id = start_cycle(&runtime, suite.clone()).await;
    runtime.start_next_execution(other_cycle_id.clone(), CommandContext::default()).await;
    wait_for_cycle_outcome(&runtime, &other_cycle_id, ResultStatus::Passed).await;
    let other_cycle = cycle_snapshot(&runtime, &other_cycle_id).await;
    let other_id = &other_cycle.test_executions[0].test_execution_id;
    runtime.wait_execution_report(other_id).await.unwrap();
    let cycle_id = start_cycle(&runtime, suite).await;
    let cycle = cycle_snapshot(&runtime, &cycle_id).await;
    let execution_id = &cycle.test_executions[0].test_execution_id;
    let mut file = runtime
        .storage()
        .create_file(FileMetadata {
            id: harmos::Identifier::new("fixture.file-output").unwrap(),
            scope: Some(harmos::Identifier::new(execution_id.as_str()).unwrap()),
            media_type: "text/plain".into(),
            expected_length: None,
            expected_sha256: None,
        })
        .await
        .unwrap();
    file.write_all(b"file output evidence".to_vec()).await.unwrap();
    let file = file.finish().await.unwrap();
    runtime.storage().flush_until(&file.position).await.unwrap();
    runtime.start_next_execution(cycle_id.clone(), CommandContext::default()).await;
    executor.wait_until_entered().await;
    runtime
        .publish_product_record(
            ProductCalibrationDetails::new("selected", ProductCalibrationKind::Field, ProductCalibrationPhase::Completed)
                .into_draft("fixture"),
        )
        .await
        .unwrap();

    // Preserve The Storage Envelope And Digest; Only The Historical Postcard Value Is Invalid.
    let database = dokime_runtime::storage::sqlite::rusqlite::Connection::open(runtime.storage_root().join("local.sqlite")).unwrap();
    let (position, mut body): (String, Vec<u8>) = database
        .query_row(
            "SELECT position,body FROM storage_data_entries WHERE kind='object' AND item=?1 ORDER BY position DESC LIMIT 1",
            [other_id.as_str()],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    let header = u32::from_le_bytes(body[..4].try_into().unwrap()) as usize;
    let original_object = (position.clone(), body.clone());
    body.truncate(4 + header);
    body.extend_from_slice(&[0xff; 3]);
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(&body);
    database
        .execute(
            "UPDATE storage_data_entries SET body=?1,digest=?2 WHERE position=?3",
            dokime_runtime::storage::sqlite::rusqlite::params![body, digest.as_slice(), position],
        )
        .unwrap();
    assert!(execution::read(runtime.storage(), other_id, ObjectVersion::Latest).await.is_err());

    let document_id = dokime_runtime::DocumentId::from_identifier("default").unwrap();
    runtime.prepare_current_document(&cycle_id, execution_id, &document_id).unwrap().capture().await.unwrap();

    let (position, body): (String, Vec<u8>) = database
        .query_row(
            "SELECT e.position,e.body FROM storage_record_visibility v JOIN storage_data_entries e ON e.position=v.position
         WHERE v.execution=?1 ORDER BY e.position LIMIT 1",
            [other_id.as_str()],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    let mut record: harmos::storage::StoredRecord = serde_json::from_slice(&body).unwrap();
    let original_record = (position.clone(), body);
    record.payload = vec![0xff; 3];
    let body = serde_json::to_vec(&record).unwrap();
    let digest = Sha256::digest(&body);
    database
        .execute(
            "UPDATE storage_data_entries SET body=?1,digest=?2 WHERE position=?3",
            dokime_runtime::storage::sqlite::rusqlite::params![body, digest.as_slice(), position],
        )
        .unwrap();
    drop(database);
    runtime.prepare_current_document(&cycle_id, execution_id, &document_id).unwrap().capture().await.unwrap();
    runtime.prepare_current_cycle_document(&cycle_id, &document_id).unwrap().capture().await.unwrap();
    let selection = dokime_runtime::document::DocumentSelection {
        scope: cycle.test_executions[0].scope.clone(),
    };
    assert!(runtime.resolve_cycle_document(&cycle_id, &document_id, selection.clone()).await.unwrap().is_some());
    let view = runtime
        .query_runtime_view(RuntimeViewQuery {
            scope: Some(selection.scope),
            topic: RuntimeViewTopic::Document { document_id: document_id.clone() },
            window: RuntimeWindow::Full,
            resolution: RuntimeResolution::Summary,
            cursor: None,
        })
        .await
        .unwrap();
    let RuntimeViewPayload::Document { frame } = view.payload else { panic!("expected document frame") };
    assert!(frame.error.is_none(), "{:?}", frame.error);
    assert!(frame.document.is_some());

    // HTML Publication Must Acquire The Same Execution's Evidence Successfully.
    executor.release();
    wait_for_cycle_outcome(&runtime, &cycle_id, ResultStatus::Passed).await;
    runtime.wait_execution_report(execution_id).await.unwrap();

    let original = execution::read(runtime.storage(), execution_id, ObjectVersion::Latest).await.unwrap().unwrap();
    let original_execution = original.value.clone();
    let mut expected = original_execution.clone();
    drop(original);
    let seal = expected.saved.as_mut().unwrap();
    seal.source.outputs.push(ExecutionOutput {
        relative_path: "fixture/file.txt".into(),
        created_at: seal.sealed_at.clone(),
        file: Some(file),
        stream_state: None,
        content: ExecutionOutputContent::File,
    });
    let row = Execution::try_from(&expected).unwrap();
    let receipt = runtime.storage().write_object(&row).await.unwrap();
    runtime.storage().flush_until(&receipt.position).await.unwrap();

    let restored = execution::read(runtime.storage(), execution_id, ObjectVersion::Latest).await;
    let captured = runtime.prepare_current_document(&cycle_id, execution_id, &document_id).unwrap().capture().await;
    assert!(captured.is_ok(), "file output must remain readable during document acquisition: {:?}", captured.as_ref().err());
    assert_eq!(restored.unwrap().unwrap().value, expected);
    let captured = captured.unwrap();
    let count = captured.capture.cards.iter().find_map(|card| match &card.result {
        dokime_runtime::document::capture::CardBindingResult::Summary { groups } => groups.first()?.tiles.first()?.value.clone(),
        _ => None,
    });
    assert_eq!(count, Some(Value::Unsigned(1)));
    let resolved = runtime
        .resolve_cycle_document(
            &cycle_id,
            &document_id,
            dokime_runtime::document::DocumentSelection {
                scope: cycle.test_executions[0].scope.clone(),
            },
        )
        .await
        .unwrap()
        .unwrap();
    let count = resolved.cards.iter().find_map(|card| match &card.body {
        dokime_runtime::document::DocumentCardBodyPayload::TelemetrySummary { groups } => groups.first()?.tiles.first()?.value.clone(),
        _ => None,
    });
    assert_eq!(count, Some(Value::Unsigned(1)));
    assert!(captured.artifacts_sha256.is_some(), "document acquisition must read the stored artifact inventory");
    assert_eq!(captured.capture.selection.scope.test_execution_id.as_ref(), Some(execution_id));
    assert!(runtime.prepare_current_document(&other_cycle_id, other_id, &document_id).unwrap().capture().await.is_err());

    // An Indexed Member With A Foreign Payload Still Refuses The Selected Capture.
    let database = dokime_runtime::storage::sqlite::rusqlite::Connection::open(runtime.storage_root().join("local.sqlite")).unwrap();
    let (position, original): (String, Vec<u8>) = database
        .query_row(
            "SELECT e.position,e.body FROM storage_record_visibility v JOIN storage_data_entries e ON e.position=v.position
         WHERE v.execution=?1 ORDER BY e.position LIMIT 1",
            [execution_id.as_str()],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    let mut record: harmos::storage::StoredRecord = serde_json::from_slice(&original).unwrap();
    let mut entry = dokime_runtime::storage::records::RecordRow::decode_entry(&record.payload).unwrap();
    entry.scope.as_mut().unwrap().test_cycle_id = Some(other_cycle_id.clone());
    for scope in &mut entry.captured_scopes {
        if scope.test_execution_id.as_ref() == Some(execution_id) {
            scope.test_cycle_id = Some(other_cycle_id.clone());
        }
    }
    use harmos::Message;
    record.payload = dokime_runtime::storage::records::RecordRow::from_entry(&entry).unwrap().encode();
    let body = serde_json::to_vec(&record).unwrap();
    let digest = Sha256::digest(&body);
    database
        .execute(
            "UPDATE storage_data_entries SET body=?1,digest=?2 WHERE position=?3",
            dokime_runtime::storage::sqlite::rusqlite::params![body, digest.as_slice(), position],
        )
        .unwrap();
    let failure = runtime
        .prepare_current_document(&cycle_id, execution_id, &document_id)
        .unwrap()
        .capture()
        .await
        .err()
        .expect("foreign selected record");
    assert!(matches!(failure, dokime_runtime::runtime::Error::Admission(
        dokime_domain::model::diagnostics::admission::AdmissionFailure::Conflict { ref code, .. }
    ) if code == "document_capture_incomplete"));

    record.payload = vec![0xff; 3];
    let body = serde_json::to_vec(&record).unwrap();
    let digest = Sha256::digest(&body);
    database
        .execute(
            "UPDATE storage_data_entries SET body=?1,digest=?2 WHERE position=?3",
            dokime_runtime::storage::sqlite::rusqlite::params![body, digest.as_slice(), position],
        )
        .unwrap();
    assert!(runtime.prepare_current_document(&cycle_id, execution_id, &document_id).unwrap().capture().await.is_err());
    let digest = Sha256::digest(&original);
    database
        .execute(
            "UPDATE storage_data_entries SET body=?1,digest=?2 WHERE position=?3",
            dokime_runtime::storage::sqlite::rusqlite::params![original, digest.as_slice(), position],
        )
        .unwrap();
    for (position, body) in [original_object, original_record] {
        let digest = Sha256::digest(&body);
        database
            .execute(
                "UPDATE storage_data_entries SET body=?1,digest=?2 WHERE position=?3",
                dokime_runtime::storage::sqlite::rusqlite::params![body, digest.as_slice(), position],
            )
            .unwrap();
    }
    drop(database);
    let original = Execution::try_from(&original_execution).unwrap();
    let receipt = runtime.storage().write_object(&original).await.unwrap();
    runtime.storage().flush_until(&receipt.position).await.unwrap();
    owner.shutdown().await.unwrap();
}

// One runtime carries the stages that build and read reports of executions the
// publishing executor settles on its own; each stage runs its own cycle.
#[dokime_tracing::trace_test]
#[tokio::test]
async fn document_reports() {
    let root = tempdir().unwrap();
    let (mut owner, runtime) = booted_with_fixture(root.path(), Arc::new(PublishingExecutor)).await;
    construction::default_document_order(&runtime).await;
    construction::current_cycle_state(&runtime).await;
    construction::cycle_report_executions(&runtime).await;
    construction::saved_report_evidence(&runtime).await;
    report_payloads::latest_member_annotations(&runtime).await;
    report_payloads::cycle_report_planned(&runtime).await;
    drop(runtime);
    owner.shutdown().await.unwrap();
}

// The rendered payload shapes need no runtime: they render fixture input.
#[dokime_tracing::trace_test]
#[tokio::test]
async fn document_report_payloads() {
    report_payloads::rendered_html_data().await;
    report_payloads::rendered_html_payload().await;
    report_payloads::materialized_report_axis().await;
    report_payloads::rendered_cycle_out().await;
    report_payloads::execution_template_cases().await;
    report_payloads::rendered_case_plots().await;
    report_payloads::rendered_case_one().await;
    report_payloads::rendered_report_interval().await;
    report_payloads::rendered_html_groups().await;
    report_payloads::stated_section_wrote().await;
    report_payloads::rendered_report_happened().await;
    report_payloads::failed_step_frames().await;
    report_payloads::normalised_report_rows().await;
    report_payloads::template_parameter_units().await;
    report_payloads::criterion_step_order().await;
    report_payloads::telemetry_index_endpoints().await;
}

// Each of these needs its own executor, configuration or cycle held open, so each boots its own runtime.
#[dokime_tracing::trace_test]
#[tokio::test]
async fn document_report_live() {
    capture::notes_until_seal().await;
    #[cfg(feature = "fault-injection")]
    capture::undeclared_records_settle().await;
    construction::section_run_report().await;
    construction::current_fact_frontiers().await;
    construction::document_routes_samples().await;
    construction::repeated_sections_follow().await;
    construction::repeated_sections_freeze().await;
    reports::report_opens_anytime().await;
    reports::open_report_repeatedly().await;
    reports::cycle_report_reboot().await;
}

// The saved receipt builds its storage by hand and the download story holds a second execution.
#[dokime_tracing::trace_test]
#[tokio::test]
async fn document_report_seal() {
    reports::report_save_receipt().await;
    reports::report_download_serves().await;
}

#[dokime_tracing::trace_test]
#[tokio::test]
#[cfg(feature = "testing")]
async fn concurrent_seal_is_durable() {
    reports::concurrent_seal_is_durable().await;
}

#[dokime_tracing::trace_test]
#[tokio::test]
#[cfg(feature = "testing")]
async fn checkpoint_preserves_inflight_seal() {
    reports::checkpoint_preserves_inflight_seal().await;
}
