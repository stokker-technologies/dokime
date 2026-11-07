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
async fn scoped_evidence_ignores_corrupt_other_executions() {
    use dokime_domain::model::execution::saved::{ExecutionOutput, ExecutionOutputContent};
    use dokime_domain::model::records::product::{ProductCalibrationDetails, ProductCalibrationKind, ProductCalibrationPhase};
    use dokime_runtime::storage::objects::execution::{self, Execution};
    use harmos::storage::{FileMetadata, ObjectVersion};

    let root = tempdir().unwrap();
    let executor = Arc::new(SecondExecutionHoldExecutor::default());
    use dokime_runtime::prompts::{FormPrompt, Prompt};
    let mut question = FormPrompt::<BTreeMap<String, bool>>::new("fixture.scoped.confirm", "Confirm Evidence").definition();
    question.actions[0].requires_condition = true;
    question.condition = Some(dokime_runtime::definition::RuntimeCondition::Record {
        condition: dokime_runtime::records::RecordCondition {
            record_type: Identifier::new("calibration.field.completed").unwrap(),
            match_fields: BTreeMap::new(),
        },
    });
    let mut domains = harmos::sidecar::Registrar::new();
    use dokime_runtime::plugin::registration::{PROMPTS_NAMESPACE, domain_schema};
    domains.register(PROMPTS_NAMESPACE, domain_schema(PROMPTS_NAMESPACE), &vec![question]).unwrap();
    let settings = config(root.path()).with_declared_plugin("scoped_prompt_fixture", "1.0.0", "Questions", domains).unwrap();
    let (mut owner, runtime) = started_with_fixture(settings, executor.clone()).await;
    let mut suite = fixture_suite(&["passed"]);
    use dokime_runtime::document::{
        DocumentCard, DocumentDefinition, DocumentSection, TelemetrySummaryDefinition, TelemetrySummaryGroupDefinition,
        TelemetrySummaryTileDefinition,
    };
    let mut group = TelemetrySummaryGroupDefinition::new("records", "Records");
    group.tile(TelemetrySummaryTileDefinition::new("Count", dokime_runtime::expressions::record("calibration.field.completed").count()));
    group.tile(TelemetrySummaryTileDefinition::new("Files", dokime_runtime::expressions::artifact("selected.txt").count()));
    let mut summary = TelemetrySummaryDefinition::new();
    summary.group(group);
    let mut section = DocumentSection::new("Captured Records");
    section.card(DocumentCard::telemetry_summary(summary));
    let mut document = DocumentDefinition::new("Evidence");
    document.section(section);
    suite.test_case_templates[0].document = Some(document);
    suite.test_case_templates[0]
        .subscriptions
        .push(dokime_runtime::definition::Subscription::Record(Identifier::new("calibration.field.completed").unwrap()));
    let mut other_suite = suite.clone();
    other_suite.metadata = metadata("foreign.suite", "Foreign Suite");
    other_suite.test_case_templates[0].test_case_template_id = TemplateId::from_seed("foreign.template");
    register_python_suite_pair(&runtime, suite.clone(), other_suite.clone()).await;
    let other_cycle_id = start_cycle(&runtime, other_suite.clone()).await;
    runtime.start_next_execution(other_cycle_id.clone(), CommandContext::default()).await;
    wait_for_cycle_outcome(&runtime, &other_cycle_id, ResultStatus::Passed).await;
    let other_cycle = cycle_snapshot(&runtime, &other_cycle_id).await;
    let other_id = &other_cycle.test_executions[0].test_execution_id;
    runtime.wait_execution_report(other_id).await.unwrap();
    runtime
        .publish_product_record(
            ProductCalibrationDetails::new("foreign-case-context", ProductCalibrationKind::Field, ProductCalibrationPhase::Completed)
                .into_draft("fixture")
                .with_scope(RecordScope {
                    test_case_id: Some(other_cycle.test_executions[0].test_case_id.clone()),
                    ..Default::default()
                }),
        )
        .await
        .unwrap();
    let cycle_id = start_cycle(&runtime, suite.clone()).await;
    let cycle = cycle_snapshot(&runtime, &cycle_id).await;
    let execution_id = &cycle.test_executions[0].test_execution_id;
    runtime
        .publish_product_record(
            ProductCalibrationDetails::new("selected-case-context", ProductCalibrationKind::Field, ProductCalibrationPhase::Completed)
                .into_draft("fixture")
                .with_scope(RecordScope {
                    test_case_id: Some(cycle.test_executions[0].test_case_id.clone()),
                    ..Default::default()
                }),
        )
        .await
        .unwrap();
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

    let artifact_path = root.path().join("selected.txt");
    std::fs::write(&artifact_path, b"selected artifact").unwrap();
    let artifact = runtime
        .upload_artifact_file(cycle.test_executions[0].scope.clone(), &artifact_path, "selected.txt".into(), Some("text/plain".into()))
        .await
        .unwrap();

    let other_request = runtime
        .watch_state()
        .latest()
        .state()
        .run_requests
        .values()
        .find(|request| request.summary.cycle_id == other_cycle_id)
        .unwrap()
        .summary
        .request_id
        .clone();
    crate::service_support::waits::wait_for_run_settled(&runtime, &other_request).await;
    let command_id = dokime_domain::model::identity::CommandId::from_seed("corrupt-foreign-command");
    let command = dokime_runtime::storage::requests::Command::new(
        &other_request,
        &dokime_domain::model::execution::request::StepCommandReceipt {
            command_id: command_id.clone(),
            execution_id: other_id.clone(),
            action: dokime_domain::model::execution::request::StepCommand::Next,
            phase: dokime_domain::model::execution::request::StepCommandPhase::Completed,
            attempt_id: None,
            accepted_at: dokime_runtime::model::time::now_utc(),
            updated_at: dokime_runtime::model::time::now_utc(),
            detail: None,
        },
    )
    .unwrap();
    let saved_command = runtime.storage().write_object(&command).await.unwrap();
    runtime.storage().flush_until(&saved_command.position).await.unwrap();

    let scoped_records = runtime.captured_records().select_scope(dokime_runtime::domains::evidence::ScopeSelection {
        test_execution_id: Some(execution_id.clone()),
        ..Default::default()
    });
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
    let command_key = format!("{other_request}/{command_id}");
    let (position, mut body): (String, Vec<u8>) = database
        .query_row(
            "SELECT position,body FROM storage_data_entries WHERE kind='object' AND item=?1 ORDER BY position DESC LIMIT 1",
            [&command_key],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    let original_command = (position.clone(), body.clone());
    let header = u32::from_le_bytes(body[..4].try_into().unwrap()) as usize;
    body.truncate(4 + header);
    body.extend_from_slice(&[0xff; 3]);
    let digest = Sha256::digest(&body);
    database
        .execute(
            "UPDATE storage_data_entries SET body=?1,digest=?2 WHERE position=?3",
            dokime_runtime::storage::sqlite::rusqlite::params![body, digest.as_slice(), position],
        )
        .unwrap();
    assert!(
        runtime.storage().read_object::<dokime_runtime::storage::requests::Command>(&command_key, ObjectVersion::Latest).await.is_err()
    );

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
    let (position, body): (String, Vec<u8>) = database
        .query_row(
            "SELECT e.position,e.body FROM storage_record_visibility v JOIN storage_data_entries e ON e.position=v.position
             WHERE v.execution IS NULL AND json_extract(v.scope,'$.test_case_id')=?1",
            [other_cycle.test_executions[0].test_case_id.as_str()],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    let mut record: harmos::storage::StoredRecord = serde_json::from_slice(&body).unwrap();
    let original_context_record = (position.clone(), body);
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
    for selected_scope in [
        cycle.test_executions[0].scope.clone(),
        RecordScope {
            test_cycle_id: Some(cycle_id.clone()),
            ..Default::default()
        },
        RecordScope {
            test_case_id: Some(cycle.test_executions[0].test_case_id.clone()),
            ..Default::default()
        },
        RecordScope {
            template_id: cycle.test_executions[0].scope.template_id.clone(),
            ..Default::default()
        },
        RecordScope {
            suite_id: Some(dokime_domain::model::identity::SuiteId::from_seed("unselected-suite")),
            ..Default::default()
        },
    ] {
        for topic in [RuntimeViewTopic::Records, RuntimeViewTopic::Artifacts, RuntimeViewTopic::Overview] {
            let frame = runtime
                .query_runtime_view(RuntimeViewQuery {
                    scope: Some(selected_scope.clone()),
                    topic: topic.clone(),
                    window: RuntimeWindow::Full,
                    resolution: RuntimeResolution::Raw,
                    cursor: None,
                })
                .await
                .unwrap_or_else(|error| panic!("{selected_scope:?} {topic:?}: {error}"));
            if let RuntimeViewPayload::Artifacts { frame } = frame.payload {
                if selected_scope.suite_id.is_some() {
                    assert!(frame.artifacts.is_empty());
                } else {
                    assert!(frame.artifacts.iter().any(|record| record.artifact_id == artifact.artifact_id));
                }
                assert!(frame.artifacts.iter().all(|record| record.scope.test_cycle_id.as_ref() == Some(&cycle_id)));
            }
        }
    }
    let plots = runtime
        .query_plot_payloads(vec![dokime_runtime::runtime::query::PlotPayloadRequest {
            scope: Some(cycle.test_executions[0].scope.clone()),
            marker_record_types: vec!["calibration.field.completed".into()],
            ..Default::default()
        }])
        .await
        .unwrap();
    assert_eq!(plots[0].payload.markers.len(), 1);
    drop(plots);
    let aggregate = runtime
        .aggregate(dokime_runtime::runtime::query::aggregate::AggregateQuery {
            source: dokime_runtime::runtime::query::aggregate::AggregateSource::Executions,
            filters: vec![dokime_runtime::runtime::query::aggregate::AggregateFilter {
                column: "cycle_identifier".into(),
                op: dokime_runtime::runtime::query::aggregate::AggregateOperator::Eq,
                values: vec![Value::Text(cycle_id.to_string())],
            }],
            group_by: vec![],
            aggregates: vec![dokime_runtime::runtime::query::aggregate::AggregateColumn {
                aggregate: harmos::Aggregate::Count,
                column: String::new(),
                alias: "count".into(),
            }],
            limit: 0,
        })
        .await
        .unwrap();
    assert_eq!(aggregate.rows, [vec![Value::Integer(cycle.test_executions.len() as i64)]]);
    let records = runtime
        .selected_records(dokime_runtime::domains::evidence::RecordSelection {
            scope: dokime_runtime::domains::evidence::ScopeSelection {
                test_cycle_id: Some(cycle_id.clone()),
                ..Default::default()
            },
            ..Default::default()
        })
        .await
        .unwrap();
    let native_records =
        records.iter().filter(|record| record.scope.as_ref().and_then(|scope| scope.test_cycle_id.as_ref()) == Some(&cycle_id)).count();
    let aggregate = runtime
        .aggregate(dokime_runtime::runtime::query::aggregate::AggregateQuery {
            source: dokime_runtime::runtime::query::aggregate::AggregateSource::Records,
            filters: vec![dokime_runtime::runtime::query::aggregate::AggregateFilter {
                column: "cycle_identifier".into(),
                op: dokime_runtime::runtime::query::aggregate::AggregateOperator::Eq,
                values: vec![Value::Text(cycle_id.to_string())],
            }],
            group_by: vec![],
            aggregates: vec![dokime_runtime::runtime::query::aggregate::AggregateColumn {
                aggregate: harmos::Aggregate::Count,
                column: String::new(),
                alias: "count".into(),
            }],
            limit: 0,
        })
        .await
        .unwrap();
    assert_eq!(aggregate.rows, [vec![Value::Integer(native_records as i64)]]);
    let case_records = runtime
        .selected_records(dokime_runtime::domains::evidence::RecordSelection {
            scope: dokime_runtime::domains::evidence::ScopeSelection {
                test_case_id: Some(cycle.test_executions[0].test_case_id.clone()),
                ..Default::default()
            },
            ..Default::default()
        })
        .await
        .unwrap();
    assert!(case_records.iter().any(|record| record.details.get("resource_id") == Some(&Value::Text("selected-case-context".into()))));
    {
        use dokime_runtime::runtime::query::aggregate::{
            AggregateColumn, AggregateFilter, AggregateOperator, AggregateQuery, AggregateSource,
        };
        let steps = execution::read(runtime.storage(), execution_id, ObjectVersion::Latest).await.unwrap().unwrap().value.steps.len();
        let database = dokime_runtime::storage::sqlite::rusqlite::Connection::open(runtime.storage_root().join("local.sqlite")).unwrap();
        let mut originals = Vec::new();
        for id in [other_cycle_id.as_str(), other_cycle.test_executions[0].test_case_id.as_str()] {
            let (position, original): (String, Vec<u8>) = database
                .query_row(
                    "SELECT position,body FROM storage_data_entries WHERE kind='object' AND item=?1 ORDER BY position DESC LIMIT 1",
                    [id],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .unwrap();
            let mut body = original.clone();
            let header = u32::from_le_bytes(body[..4].try_into().unwrap()) as usize;
            body.truncate(4 + header);
            body.extend_from_slice(&[0xff; 3]);
            let digest = Sha256::digest(&body);
            database
                .execute(
                    "UPDATE storage_data_entries SET body=?1,digest=?2 WHERE position=?3",
                    dokime_runtime::storage::sqlite::rusqlite::params![body, digest.as_slice(), position],
                )
                .unwrap();
            originals.push((position, original));
        }
        for (source, column, id, count) in [
            (AggregateSource::Cases, "identifier", cycle.test_executions[0].test_case_id.to_string(), 1),
            (AggregateSource::Cycles, "identifier", cycle_id.to_string(), 1),
            (AggregateSource::Steps, "execution_identifier", execution_id.to_string(), steps),
        ] {
            let aggregate = runtime
                .aggregate(AggregateQuery {
                    source,
                    filters: vec![AggregateFilter {
                        column: column.into(),
                        op: AggregateOperator::Eq,
                        values: vec![Value::Text(id)],
                    }],
                    group_by: vec![],
                    aggregates: vec![AggregateColumn {
                        aggregate: harmos::Aggregate::Count,
                        column: String::new(),
                        alias: "count".into(),
                    }],
                    limit: 0,
                })
                .await
                .unwrap();
            assert_eq!(aggregate.rows, [vec![Value::Integer(count as i64)]]);
        }
        for (position, body) in originals {
            let digest = Sha256::digest(&body);
            database
                .execute(
                    "UPDATE storage_data_entries SET body=?1,digest=?2 WHERE position=?3",
                    dokime_runtime::storage::sqlite::rusqlite::params![body, digest.as_slice(), position],
                )
                .unwrap();
        }
    }

    assert!(scoped_records.next(0, scoped_records.frontier()).await.unwrap().is_some());
    drop(scoped_records);
    let browse = runtime
        .browse_executions(dokime_runtime::runtime::query::execution_history::ExecutionBrowseQuery {
            filter: dokime_runtime::runtime::query::ExecutionFilter {
                cycle_ids: vec![cycle_id.to_string()],
                ..Default::default()
            },
            ..Default::default()
        })
        .await
        .unwrap();
    assert!(browse.discovery_complete && browse.issues.is_empty());
    assert!(browse.entries.iter().all(|entry| entry.metadata.cycle_id == cycle_id));
    drop(browse);
    let content = runtime.read_artifact_content(&artifact.artifact_id, 1024).await.unwrap().unwrap();
    assert_eq!(content.bytes, b"selected artifact");
    let browse = runtime.browse_executions(Default::default()).await.unwrap();
    assert!(!browse.discovery_complete);
    assert!(browse.issues.iter().any(|issue| issue.source.as_deref() == Some(other_id.as_str())));
    assert!(browse.entries.iter().any(|entry| entry.metadata.execution_id == *execution_id));
    drop(browse);
    let request = runtime
        .watch_state()
        .latest()
        .state()
        .run_requests
        .values()
        .find(|request| request.summary.cycle_id == cycle_id)
        .unwrap()
        .summary
        .request_id
        .clone();
    assert_eq!(runtime.run_request(&request).await.unwrap().unwrap().summary.cycle_id, cycle_id);

    let prompt_id = dokime_runtime::model::identity::new_id::<dokime_runtime::PromptId>();
    runtime
        .request_prompt(dokime_runtime::prompts::PromptRequest {
            prompt_id: prompt_id.clone(),
            prompt_type: Identifier::new("fixture.scoped.confirm").unwrap(),
            values: BTreeMap::new(),
            scope: RecordScope {
                test_cycle_id: Some(cycle_id.clone()),
                ..Default::default()
            },
        })
        .await
        .unwrap();
    runtime.submit_prompt_response(prompt_id, "submit".into(), "operator".into(), BTreeMap::new(), true).await.unwrap();
    // HTML Publication Must Acquire The Same Execution's Evidence Successfully.
    executor.release();
    wait_for_cycle_outcome(&runtime, &cycle_id, ResultStatus::Passed).await;
    runtime.wait_execution_report(execution_id).await.unwrap();
    crate::service_support::waits::wait_for_run_settled(&runtime, &request).await;

    runtime.inspect_execution(execution_id).await.unwrap();
    runtime.execution_saved_checkpoint(execution_id.clone()).await.unwrap();
    runtime.open_execution_report(execution_id).await.unwrap();
    let bundle =
        runtime.export_evidence(dokime_domain::model::exports::EvidenceSubject::Execution(execution_id.to_string())).await.unwrap();
    assert!(bundle.metadata().size_bytes > 0);
    drop(bundle);
    let original = execution::read(runtime.storage(), execution_id, ObjectVersion::Latest).await.unwrap().unwrap();
    let original_execution = original.value.clone();
    drop(original);
    {
        let suite_id = dokime_domain::model::identity::SuiteId::from_seed("frozen-artifact-suite");
        let mut tagged = original_execution.clone();
        tagged.scope.suite_id = Some(suite_id.clone());
        for output in &mut tagged.saved.as_mut().unwrap().source.outputs {
            if let ExecutionOutputContent::Artifact { record, .. } = &mut output.content {
                record.scope.suite_id = Some(suite_id.clone());
            }
        }
        let row = Execution::try_from(&tagged).unwrap();
        let receipt = runtime.storage().write_object(&row).await.unwrap();
        runtime.storage().flush_until(&receipt.position).await.unwrap();
        runtime.remove_suites("provider:testdb").await.unwrap();
        let frame = runtime
            .query_runtime_view(RuntimeViewQuery {
                scope: Some(RecordScope {
                    suite_id: Some(suite_id),
                    ..Default::default()
                }),
                topic: RuntimeViewTopic::Artifacts,
                window: RuntimeWindow::Full,
                resolution: RuntimeResolution::Raw,
                cursor: None,
            })
            .await
            .unwrap();
        let RuntimeViewPayload::Artifacts { frame } = frame.payload else { panic!("expected artifacts") };
        assert!(frame.artifacts.iter().any(|record| record.artifact_id == artifact.artifact_id));
        register_python_suite_pair(&runtime, suite, other_suite).await;
        let row = Execution::try_from(&original_execution).unwrap();
        let receipt = runtime.storage().write_object(&row).await.unwrap();
        runtime.storage().flush_until(&receipt.position).await.unwrap();
    }
    let mut expected = original_execution.clone();
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
    assert!(captured.artifacts_sha256.is_some(), "document acquisition must read the stored artifact inventory");
    assert_eq!(captured.capture.selection.scope.test_execution_id.as_ref(), Some(execution_id));
    let refused = runtime.prepare_current_document(&other_cycle_id, other_id, &document_id).unwrap().capture().await.unwrap();
    assert!(
        refused.capture.cards.iter().any(|card| matches!(card.result, dokime_runtime::document::capture::CardBindingResult::Failed { .. }))
    );

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
    let (events, mut received) = tokio::sync::mpsc::channel(8);
    let structure = runtime
        .prepare_current_document(&cycle_id, execution_id, &document_id)
        .unwrap()
        .query_stream(
            dokime_runtime::CurrentDocumentQuery {
                structure_only: true,
                cards: Vec::new(),
                page: None,
            },
            events,
        )
        .await
        .unwrap();
    assert!(
        structure.capture.cards.iter().any(|card| matches!(card.result, dokime_runtime::document::capture::CardBindingResult::Pending))
    );
    match received.recv().await.unwrap() {
        dokime_runtime::CurrentDocumentEvent::Structure { document, .. } => assert!(document.evidence_cut.is_none()),
        _ => panic!("structure precedes evidence"),
    }
    let refused = runtime.prepare_current_document(&cycle_id, execution_id, &document_id).unwrap().capture().await.unwrap();
    assert!(refused.capture.cards.iter().any(|card| matches!(&card.result,
        dokime_runtime::document::capture::CardBindingResult::Failed { failure } if failure.code == "document_capture_incomplete")));

    record.payload = vec![0xff; 3];
    let body = serde_json::to_vec(&record).unwrap();
    let digest = Sha256::digest(&body);
    database
        .execute(
            "UPDATE storage_data_entries SET body=?1,digest=?2 WHERE position=?3",
            dokime_runtime::storage::sqlite::rusqlite::params![body, digest.as_slice(), position],
        )
        .unwrap();
    let refused = runtime.prepare_current_document(&cycle_id, execution_id, &document_id).unwrap().capture().await.unwrap();
    assert!(
        refused.capture.cards.iter().any(|card| matches!(card.result, dokime_runtime::document::capture::CardBindingResult::Failed { .. }))
    );
    let digest = Sha256::digest(&original);
    database
        .execute(
            "UPDATE storage_data_entries SET body=?1,digest=?2 WHERE position=?3",
            dokime_runtime::storage::sqlite::rusqlite::params![original, digest.as_slice(), position],
        )
        .unwrap();
    let original = Execution::try_from(&original_execution).unwrap();
    let receipt = runtime.storage().write_object(&original).await.unwrap();
    runtime.storage().flush_until(&receipt.position).await.unwrap();
    crate::service_support::waits::wait_for_run_settled(&runtime, &request).await;
    runtime.seal_cycle(cycle_id.clone(), CommandContext::default()).await.unwrap();
    let sealed = dokime_runtime::storage::objects::cycle::read(runtime.storage(), &cycle_id, ObjectVersion::Latest).await.unwrap().unwrap();
    assert!(sealed.value.saved.is_some());
    runtime.open_cycle_report(&cycle_id).await.unwrap();
    let bundle = runtime.export_evidence(dokime_domain::model::exports::EvidenceSubject::Cycle(cycle_id.to_string())).await.unwrap();
    assert!(bundle.metadata().size_bytes > 0);
    drop(bundle);

    for (position, body) in [&original_object, &original_record, &original_context_record, &original_command] {
        let digest = Sha256::digest(body);
        database
            .execute(
                "UPDATE storage_data_entries SET body=?1,digest=?2 WHERE position=?3",
                dokime_runtime::storage::sqlite::rusqlite::params![body, digest.as_slice(), position],
            )
            .unwrap();
    }
    drop(database);
    owner.shutdown().await.unwrap();

    // A Restart Does Not Open Terminal Evidence Or Another Request's Commands.
    let database = dokime_runtime::storage::sqlite::rusqlite::Connection::open(runtime.storage_root().join("local.sqlite")).unwrap();
    for (position, original) in [&original_object, &original_command] {
        let mut body = original.clone();
        let header = u32::from_le_bytes(body[..4].try_into().unwrap()) as usize;
        body.truncate(4 + header);
        body.extend_from_slice(&[0xff; 3]);
        let digest = Sha256::digest(&body);
        database
            .execute(
                "UPDATE storage_data_entries SET body=?1,digest=?2 WHERE position=?3",
                dokime_runtime::storage::sqlite::rusqlite::params![body, digest.as_slice(), position],
            )
            .unwrap();
    }
    for (position, original) in [&original_record, &original_context_record] {
        let mut record: harmos::storage::StoredRecord = serde_json::from_slice(original).unwrap();
        record.payload = vec![0xff; 3];
        let body = serde_json::to_vec(&record).unwrap();
        let digest = Sha256::digest(&body);
        database
            .execute(
                "UPDATE storage_data_entries SET body=?1,digest=?2 WHERE position=?3",
                dokime_runtime::storage::sqlite::rusqlite::params![body, digest.as_slice(), position],
            )
            .unwrap();
    }
    drop(database);
    let (mut owner, restarted) = booted_with_fixture(root.path(), Arc::new(FakePythonExecutor::fixed("passed"))).await;
    assert_eq!(restarted.run_request(&request).await.unwrap().unwrap().summary.cycle_id, cycle_id);
    restarted.inspect_execution(execution_id).await.unwrap();
    restarted.open_cycle_report(&cycle_id).await.unwrap();
    let content = restarted.read_artifact_content(&artifact.artifact_id, 1024).await.unwrap().unwrap();
    assert_eq!(content.bytes, b"selected artifact");
    let bundle = restarted.export_evidence(dokime_domain::model::exports::EvidenceSubject::Cycle(cycle_id.to_string())).await.unwrap();
    assert!(bundle.metadata().size_bytes > 0);
    drop(bundle);
    let database = dokime_runtime::storage::sqlite::rusqlite::Connection::open(runtime.storage_root().join("local.sqlite")).unwrap();
    for (position, body) in [&original_object, &original_record, &original_context_record, &original_command] {
        let digest = Sha256::digest(body);
        database
            .execute(
                "UPDATE storage_data_entries SET body=?1,digest=?2 WHERE position=?3",
                dokime_runtime::storage::sqlite::rusqlite::params![body, digest.as_slice(), position],
            )
            .unwrap();
    }
    drop(database);
    owner.shutdown().await.unwrap();
}

#[dokime_tracing::trace_test]
#[tokio::test]
async fn cycle_inspection_retains_control_projections() {
    let root = tempdir().unwrap();
    let (mut owner, runtime) = booted_with_fixture(root.path(), Arc::new(FakePythonExecutor::fixed("passed"))).await;
    let suite = fixture_suite(&["passed", "passed"]);
    register_python_suite(&runtime, suite.clone()).await;
    let cycle_id = start_cycle(&runtime, suite.clone()).await;
    let cycle = cycle_snapshot(&runtime, &cycle_id).await;
    for execution in &cycle.test_executions {
        let receipt = runtime.start_execution(execution.test_execution_id.clone(), Default::default(), CommandContext::default()).await;
        crate::service_support::waits::wait_for_run_settled(&runtime, &receipt.command_id).await;
        runtime.wait_execution_report(&execution.test_execution_id).await.unwrap();
    }
    let selected = &cycle.test_executions[0].test_execution_id;
    let parameters = runtime.inspect_execution(selected).await.unwrap().execution.output_parameters.clone();
    assert!(parameters.values().any(|parameter| parameter.source == dokime_runtime::definition::ParameterSource::Telemetry));
    let published = runtime.watch_state().latest();
    let inspected = runtime.inspect_cycle(&cycle_id).await.unwrap();
    for execution in &cycle.test_executions {
        let id = &execution.test_execution_id;
        assert_eq!(inspected.state().executions[id], published.state().executions[id]);
    }
    drop(inspected);

    runtime.seal_cycle(cycle_id.clone(), CommandContext::default()).await.unwrap();
    start_cycle(&runtime, suite).await;
    assert!(!runtime.watch_state().latest().state().cycles.contains_key(&cycle_id));
    let inspected = runtime.inspect_cycle(&cycle_id).await.unwrap();
    for execution in &cycle.test_executions {
        let id = &execution.test_execution_id;
        assert_eq!(inspected.state().executions[id], published.state().executions[id]);
    }
    assert_eq!(runtime.inspect_execution(selected).await.unwrap().execution.output_parameters, parameters);
    drop(inspected);
    owner.shutdown().await.unwrap();
}

#[dokime_tracing::trace_test]
#[tokio::test]
async fn execution_inspection_ignores_corrupt_sibling() {
    use dokime_runtime::storage::sqlite::{rusqlite, rusqlite::params};
    use sha2::{Digest, Sha256};
    let root = tempdir().unwrap();
    let (mut owner, runtime) = booted_with_fixture(root.path(), Arc::new(FakePythonExecutor::fixed("passed"))).await;
    let suite = fixture_suite(&["passed", "passed"]);
    register_python_suite(&runtime, suite.clone()).await;
    let cycle_id = start_cycle(&runtime, suite).await;
    let cycle = cycle_snapshot(&runtime, &cycle_id).await;
    for execution in &cycle.test_executions {
        let receipt = runtime.start_execution(execution.test_execution_id.clone(), Default::default(), CommandContext::default()).await;
        crate::service_support::waits::wait_for_run_settled(&runtime, &receipt.command_id).await;
        runtime.wait_execution_report(&execution.test_execution_id).await.unwrap();
    }
    runtime.seal_cycle(cycle_id.clone(), CommandContext::default()).await.unwrap();
    owner.shutdown().await.unwrap();
    let foreign = &cycle.test_executions[0].test_execution_id;
    let selected = &cycle.test_executions[1].test_execution_id;
    let database = rusqlite::Connection::open(runtime.storage_root().join("local.sqlite")).unwrap();
    let (position, mut body): (String, Vec<u8>) = database
        .query_row(
            "SELECT position,body FROM storage_data_entries WHERE kind='object' AND item=?1 ORDER BY position DESC LIMIT 1",
            [foreign.as_str()],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    let header = u32::from_le_bytes(body[..4].try_into().unwrap()) as usize;
    let original = body.clone();
    body.truncate(4 + header);
    body.extend_from_slice(&[0xff; 3]);
    let digest = Sha256::digest(&body);
    database
        .execute("UPDATE storage_data_entries SET body=?1,digest=?2 WHERE position=?3", params![body, digest.as_slice(), position])
        .unwrap();
    drop(database);
    let (mut owner, runtime) = booted_with_fixture(root.path(), Arc::new(FakePythonExecutor::fixed("passed"))).await;
    let inspection = runtime.inspect_execution(selected).await.unwrap();
    assert_eq!(&inspection.execution.test_execution_id, selected);
    assert_eq!(&inspection.cycle.execution_ids[inspection.ordinal], selected);
    assert_eq!(
        inspection.cycle.execution_ids,
        cycle.test_executions.iter().map(|execution| execution.test_execution_id.clone()).collect::<Vec<_>>()
    );
    let case = runtime.inspect_case(&cycle_id, &cycle.test_executions[1].test_case_id).await.unwrap();
    assert_eq!(case.case.test_case_id, cycle.test_executions[1].test_case_id);
    assert_eq!(case.projection.latest_execution_id.as_ref(), Some(selected));
    assert_eq!(case.ordinal, inspection.ordinal);
    drop(case);
    assert!(runtime.inspect_cycle(&cycle_id).await.is_err());
    drop(inspection);
    let database = rusqlite::Connection::open(runtime.storage_root().join("local.sqlite")).unwrap();
    let digest = Sha256::digest(&original);
    database
        .execute("UPDATE storage_data_entries SET body=?1,digest=?2 WHERE position=?3", params![original, digest.as_slice(), position])
        .unwrap();
    drop(database);
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

#[dokime_tracing::trace_test]
#[tokio::test]
async fn document_query_scale() {
    construction::summary_queries().await;
}

#[dokime_tracing::trace_test]
#[tokio::test]
async fn document_repeat_pages() {
    construction::repeated_pages_keep_their_cut().await;
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
    construction::document_queries_captured_scope().await;
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
    for operation in ["finish", "stop", "seal"] {
        let root = tempdir().unwrap();
        let executor = Arc::new(BlockingPublishingExecutor::new());
        let (mut owner, runtime) = booted_with_fixture(root.path(), executor.clone()).await;
        let suite = fixture_suite(&["passed"]);
        register_python_suite(&runtime, suite.clone()).await;
        let cycle = start_cycle(&runtime, suite).await;
        let execution = cycle_snapshot(&runtime, &cycle).await.test_executions[0].test_execution_id.clone();
        let report_gate = dokime_runtime::testing::hold_report(&execution, false);
        runtime.start_next_execution(cycle.clone(), CommandContext::default()).await;
        executor.wait_until_entered().await;
        executor.release();
        report_gate.entered().await;

        let wait_gate = dokime_runtime::testing::hold_report_wait(&execution);
        let joined = async {
            match operation {
                "finish" => runtime.finish_execution_for_test(&execution).await,
                "stop" => runtime.stop_execution(execution.clone(), CommandContext::default()).await,
                "seal" => runtime.seal_cycle(cycle.clone(), CommandContext::default()).await,
                _ => unreachable!(),
            }
        };
        tokio::pin!(joined);
        tokio::select! {
            result = &mut joined => {
                result.unwrap();
                let durable = dokime_runtime::storage::objects::execution::read(
                    runtime.storage(), &execution, harmos::storage::ObjectVersion::Latest,
                ).await.unwrap().unwrap();
                assert!(durable.value.saved.is_some(), "{operation} returned before the in-flight seal was durable");
                panic!("{operation} did not join the held seal");
            }
            _ = wait_gate.entered() => {}
        }
        wait_gate.release();
        assert!(futures_util::poll!(&mut joined).is_pending());
        report_gate.release();
        joined.await.unwrap();
        let durable =
            dokime_runtime::storage::objects::execution::read(runtime.storage(), &execution, harmos::storage::ObjectVersion::Latest)
                .await
                .unwrap()
                .unwrap();
        assert!(durable.value.saved.is_some(), "{operation} must expose the durable seal immediately");
        drop(durable);
        owner.shutdown().await.unwrap();
    }
}

#[dokime_tracing::trace_test]
#[tokio::test]
#[cfg(feature = "testing")]
async fn checkpoint_preserves_inflight_seal() {
    let root = tempdir().unwrap();
    let executor = Arc::new(BlockingPublishingExecutor::new());
    let (mut owner, runtime) = booted_with_fixture(root.path(), executor.clone()).await;
    let suite = fixture_suite(&["passed"]);
    register_python_suite(&runtime, suite.clone()).await;
    let cycle = start_cycle(&runtime, suite).await;
    let execution = cycle_snapshot(&runtime, &cycle).await.test_executions[0].test_execution_id.clone();
    let report_gate = dokime_runtime::testing::hold_report(&execution, false);
    let seal_gate = dokime_runtime::testing::hold_execution_seal(&execution);
    let receipt = runtime.start_next_execution(cycle.clone(), CommandContext::default()).await;
    executor.wait_until_entered().await;
    executor.release();
    report_gate.entered().await;
    crate::service_support::waits::wait_for_run_settled(&runtime, &receipt.command_id).await;

    // Admit A Checkpoint While The Member Is Still Unsealed.
    let checkpoint_gate = dokime_runtime::testing::hold_execution_checkpoint(&execution);
    let checkpoint = runtime.update_cycle_policy(cycle.clone(), false, CommandContext::default());
    tokio::pin!(checkpoint);
    tokio::select! {
        result = &mut checkpoint => panic!("checkpoint missed its gate: {result:?}"),
        _ = checkpoint_gate.entered() => {}
    }
    report_gate.release();
    seal_gate.entered().await;
    let sealed = dokime_runtime::storage::objects::execution::read(runtime.storage(), &execution, harmos::storage::ObjectVersion::Latest)
        .await
        .unwrap()
        .unwrap();
    let seal = sealed.value.saved.clone().expect("native sealing precedes managed adoption");
    let position = sealed.position.clone();
    drop(sealed);

    // Resume The Older Checkpoint Between Native Durability And Managed Adoption.
    checkpoint_gate.release();
    checkpoint.await.unwrap();
    let finish = runtime.finish_execution_for_test(&execution);
    tokio::pin!(finish);
    assert!(futures_util::poll!(&mut finish).is_pending());
    seal_gate.release();
    finish.await.unwrap();
    let durable = dokime_runtime::storage::objects::execution::read(runtime.storage(), &execution, harmos::storage::ObjectVersion::Latest)
        .await
        .unwrap()
        .unwrap();
    assert!(durable.value.saved.is_some(), "finish must leave the latest execution revision sealed durably");
    assert_eq!(durable.value.saved, Some(seal));
    assert_eq!(durable.position, position, "a late checkpoint must not replace an immutable seal");
    drop(durable);
    owner.shutdown().await.unwrap();
}
