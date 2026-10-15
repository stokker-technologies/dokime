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
use std::{sync::Arc, time::Duration};

// Third Party Imports
use harmos::{
    Application, Runtime,
    state::Never,
    storage::{StorageBackends, StorageSettings},
};

// Local Imports
use dokime::{
    core::{CaseId, Identifier, Metadata, TemplateId, Value},
    definition::{TestCase, TestSuite},
    storage::{
        Backend, Manager,
        objects::case::{self, Case},
    },
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct Reader;

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Application for Reader {
    type State = ();
    type Transaction = Never;
    type Record = Never;
    type Streams = ();
    type Resources = ();
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

pub(crate) async fn materialized_cases_reader() {
    let root = tempfile::tempdir().unwrap();
    let primary = Arc::new(
        Backend::primary(root.path().join("primary"), Default::default())
            .await
            .unwrap(),
    );
    let recovery = Arc::new(primary.recovery(Default::default()));
    let storage = Manager::new(
        StorageBackends {
            primary,
            recovery,
            replicas: vec![],
        },
        StorageSettings {
            flush_interval: Duration::from_secs(60),
            ..Default::default()
        },
    )
    .unwrap();
    let runtime = Runtime::<Reader>::builder((), ())
        .runtime_with_storage(&storage)
        .await
        .unwrap();
    let expected = TestCase {
        suite_id: "fixture.registration".into(),
        test_case_id: CaseId::from_wire("fixture.case:1234567890").unwrap(),
        test_case_template_id: TemplateId::from_seed("template"),
        metadata: Metadata::new(
            Identifier::new("fixture.case").unwrap(),
            "Stored Case",
            None,
        ),
        suite: None,
        parameters: std::collections::BTreeMap::from([("count".into(), Value::Unsigned(u64::MAX))]),
        generation_index: 17,
        label_number: 3,
    };
    let row = Case::try_from(&expected).unwrap();
    let receipt = storage.write_object(&row).await.unwrap();
    storage.flush_until(&receipt.position).await.unwrap();
    assert_eq!(
        case::read(
            &storage,
            &expected.test_case_id,
            super::FRAME_BYTES.try_into().unwrap()
        )
        .await
        .unwrap()
        .as_deref(),
        Some(&expected)
    );
    runtime.stop().await.unwrap();

    let primary = Arc::new(
        Backend::primary(root.path().join("primary"), Default::default())
            .await
            .unwrap(),
    );
    let recovery = Arc::new(primary.recovery(Default::default()));
    let storage = Manager::new(
        StorageBackends {
            primary,
            recovery,
            replicas: vec![],
        },
        StorageSettings::default(),
    )
    .unwrap();
    let runtime = Runtime::<Reader>::builder((), ())
        .runtime_with_storage(&storage)
        .await
        .unwrap();
    assert_eq!(
        case::read(
            &storage,
            &expected.test_case_id,
            super::FRAME_BYTES.try_into().unwrap()
        )
        .await
        .unwrap()
        .as_deref(),
        Some(&expected)
    );
    assert_eq!(
        case::read(
            &storage,
            &CaseId::from_wire("fixture.absent:1234567890").unwrap(),
            super::FRAME_BYTES.try_into().unwrap()
        )
        .await
        .unwrap()
        .as_deref(),
        None
    );
    let absent = case::read(
        &storage,
        &CaseId::from_wire("fixture.absent:1234567890").unwrap(),
        super::FRAME_BYTES.try_into().unwrap(),
    )
    .await
    .unwrap();
    assert!(absent.is_none());
    runtime.stop().await.unwrap();
}

pub(crate) async fn suite_definitions_producer() {
    let root = tempfile::tempdir().unwrap();
    let primary = Arc::new(
        Backend::primary(root.path().join("primary"), Default::default())
            .await
            .unwrap(),
    );
    let recovery = Arc::new(primary.recovery(Default::default()));
    let backends = StorageBackends {
        primary,
        recovery,
        replicas: vec![],
    };
    let storage = Manager::new(backends, StorageSettings::default()).unwrap();
    let runtime = Runtime::<Reader>::builder((), ())
        .runtime_with_storage(&storage)
        .await
        .unwrap();

    let metadata = Metadata::new(
        Identifier::new("fixture.authored").unwrap(),
        "Stored Suite",
        None,
    );
    let definition = serde_json::json!({
        "metadata": metadata,
        "test_case_templates": [{
            "test_case_template_id": TemplateId::from_seed("native-suite-template"),
            "metadata": metadata,
            "parameters": [],
            "document": {
                "document_id": dokime::core::DocumentId::from_seed("native-suite-document"),
                "metadata": metadata,
                "cards": [{"metadata": metadata, "type": "text", "data": {"markdown": "Saved Definition"}}]
            },
            "subscriptions": [{"kind": "stream", "identifier": "fixture.voltage"}]
        }]
    });
    let mut expected: TestSuite = serde_json::from_value(definition).unwrap();
    let stored =
        dokime::storage::objects::suite::Suite::new("fixture.registration", &expected).unwrap();
    let receipt = storage.write_object(&stored).await.unwrap();
    storage.flush_until(&receipt.position).await.unwrap();

    expected.metadata.identifier = Identifier::new("fixture.changed_author").unwrap();
    expected.metadata.label = "Latest Suite".into();
    let stored =
        dokime::storage::objects::suite::Suite::new("fixture.registration", &expected).unwrap();
    let receipt = storage.write_object(&stored).await.unwrap();
    storage.flush_until(&receipt.position).await.unwrap();
    runtime.stop().await.unwrap();

    let primary = Arc::new(
        Backend::primary(root.path().join("primary"), Default::default())
            .await
            .unwrap(),
    );
    let recovery = Arc::new(primary.recovery(Default::default()));
    let backends = StorageBackends {
        primary,
        recovery,
        replicas: vec![],
    };
    let storage = Manager::new(backends, StorageSettings::default()).unwrap();
    let runtime = Runtime::<Reader>::builder((), ())
        .runtime_with_storage(&storage)
        .await
        .unwrap();

    let limit = super::FRAME_BYTES.try_into().unwrap();
    let saved = dokime::storage::objects::suite::read(&storage, "fixture.registration", limit)
        .await
        .unwrap();
    assert_eq!(saved, Some(expected));
    let absent = dokime::storage::objects::suite::read(&storage, "fixture.absent", limit)
        .await
        .unwrap();
    assert!(absent.is_none());
    drop(saved);
    runtime.stop().await.unwrap();
}

pub(crate) async fn cycles_reopen_producers() {
    let root = tempfile::tempdir().unwrap();
    let primary = Arc::new(
        Backend::primary(root.path().join("primary"), Default::default())
            .await
            .unwrap(),
    );
    let recovery = Arc::new(primary.recovery(Default::default()));
    let backends = StorageBackends {
        primary,
        recovery,
        replicas: vec![],
    };
    let storage = Manager::new(backends, StorageSettings::default()).unwrap();
    let runtime = Runtime::<Reader>::builder((), ())
        .runtime_with_storage(&storage)
        .await
        .unwrap();

    let metadata = Metadata::new(
        Identifier::new("fixture.cycle").unwrap(),
        "Stored Cycle",
        None,
    );
    let at = dokime::core::Timestamp::new("2026-10-05T09:00:00Z");
    let mut expected = dokime::execution::TestCycle {
        saved: None,
        test_cycle_id: dokime::core::TestCycleId::from_seed("native-latest-cycle"),
        session_id: None,
        metadata,
        lifecycle: dokime::execution::CycleStatus::Ready,
        created_at: at.clone(),
        started_at: None,
        updated_at: at,
        execution_ids: vec![
            dokime::core::TestExecutionId::from_seed("native-first"),
            dokime::core::TestExecutionId::from_seed("native-second"),
        ],
        records: Default::default(),
        continue_on_failure: false,
        result: None,
    };
    let row = dokime::storage::objects::cycle::Cycle::try_from(&expected).unwrap();
    let receipt = storage.write_object(&row).await.unwrap();
    storage.flush_until(&receipt.position).await.unwrap();

    expected.lifecycle = dokime::execution::CycleStatus::Completed;
    expected.updated_at = dokime::core::Timestamp::new("2026-10-05T09:01:00Z");
    expected.execution_ids.reverse();
    expected.result = Some(dokime::execution::ResultSummary {
        status: dokime::execution::ResultStatus::Passed,
        passed: 2,
        failed: 0,
        blocked: 0,
        skipped: 0,
    });
    let row = dokime::storage::objects::cycle::Cycle::try_from(&expected).unwrap();
    let receipt = storage.write_object(&row).await.unwrap();
    storage.flush_until(&receipt.position).await.unwrap();
    runtime.stop().await.unwrap();

    let primary = Arc::new(
        Backend::primary(root.path().join("primary"), Default::default())
            .await
            .unwrap(),
    );
    let recovery = Arc::new(primary.recovery(Default::default()));
    let backends = StorageBackends {
        primary,
        recovery,
        replicas: vec![],
    };
    let storage = Manager::new(backends, StorageSettings::default()).unwrap();
    let runtime = Runtime::<Reader>::builder((), ())
        .runtime_with_storage(&storage)
        .await
        .unwrap();

    let limit = super::FRAME_BYTES.try_into().unwrap();
    let read_bytes_before = storage.status().read_bytes;
    let view = storage
        .view_with_limits(StorageSettings::default().query_execution)
        .await
        .unwrap();
    let saved = dokime::storage::objects::cycle::read_in(
        &storage,
        &view,
        &expected.test_cycle_id,
        harmos::storage::ObjectVersion::Latest,
        limit,
    )
    .await
    .unwrap();
    assert_eq!(
        saved.as_deref().map(|revision| &revision.value),
        Some(&expected)
    );
    assert_eq!(saved.as_ref().unwrap().position, receipt.position);
    assert_eq!(saved.as_ref().unwrap().version, receipt.version);
    let held_read_bytes = storage.status().read_bytes;
    assert!(held_read_bytes > read_bytes_before);
    assert!(
        held_read_bytes - read_bytes_before
            >= harmos::MemorySize::memory_size(saved.as_deref().unwrap())
    );
    let missing = dokime::core::TestCycleId::from_seed("native-missing-cycle");
    let absent = dokime::storage::objects::cycle::read_in(
        &storage,
        &view,
        &missing,
        harmos::storage::ObjectVersion::Latest,
        limit,
    )
    .await
    .unwrap();
    assert!(absent.is_none());
    assert_eq!(storage.status().read_bytes, held_read_bytes);
    let value_bytes = harmos::MemorySize::memory_size(saved.as_deref().unwrap());
    drop(saved);
    let released_read_bytes = storage.status().read_bytes;
    assert!(held_read_bytes - released_read_bytes >= value_bytes);
    drop(view);
    runtime.stop().await.unwrap();
    assert_eq!(storage.status().read_bytes, read_bytes_before);
}

pub(crate) async fn executions_reopen_results() {
    let root = tempfile::tempdir().unwrap();
    let primary = Arc::new(
        Backend::primary(root.path().join("primary"), Default::default())
            .await
            .unwrap(),
    );
    let recovery = Arc::new(primary.recovery(Default::default()));
    let backends = StorageBackends {
        primary,
        recovery,
        replicas: vec![],
    };
    let storage = Manager::new(backends, StorageSettings::default()).unwrap();
    let runtime = Runtime::<Reader>::builder((), ())
        .runtime_with_storage(&storage)
        .await
        .unwrap();

    let case = TestCase {
        suite_id: "fixture.registration".into(),
        test_case_id: CaseId::from_wire("fixture.frozen:1234567890").unwrap(),
        test_case_template_id: TemplateId::from_seed("native-frozen-template"),
        metadata: Metadata::new(
            Identifier::new("fixture.frozen").unwrap(),
            "Frozen Case",
            None,
        ),
        suite: None,
        parameters: std::collections::BTreeMap::from([("count".into(), Value::Unsigned(u64::MAX))]),
        generation_index: 2,
        label_number: 0,
    };
    let snapshot = dokime::execution::ExecutionSnapshot::materialize(
        &case,
        None,
        Default::default(),
        None,
        Default::default(),
    )
    .unwrap()
    .with_host_version(Some("0.1.0".into()));
    let cycle_id = dokime::core::TestCycleId::from_seed("native-frozen-cycle");
    let execution_id = dokime::core::TestExecutionId::from_seed("native-frozen-execution");
    let scope = dokime::core::RecordScope::default()
        .with_cycle(cycle_id.clone())
        .with_test_case(case.test_case_id.clone())
        .with_test_execution(execution_id.clone());
    let mut expected = dokime::execution::TestExecution {
        saved: None,
        mode: Default::default(),
        active_step_attempt: None,
        capture_phase: Default::default(),
        step_attempts: Vec::new(),
        test_cycle_id: cycle_id,
        test_execution_id: execution_id,
        test_case_id: case.test_case_id.clone(),
        suite: None,
        status: dokime::execution::ExecutionStatus::Pending,
        snapshot,
        invalidation: None,
        scope,
        output_parameters: Default::default(),
        documents: Default::default(),
        records: Default::default(),
        started_at: None,
        finished_at: None,
        steps: vec![],
        traceability: vec![],
        result: None,
    };
    let row = dokime::storage::objects::execution::Execution::try_from(&expected).unwrap();
    let receipt = storage.write_object(&row).await.unwrap_or_else(|error| {
        let schema =
            crate::sqlite::storage::declared::<dokime::storage::objects::execution::Execution>();
        panic!(
            "execution admission failed: {error}; payload={} resident={} schema={}",
            harmos::Message::encoded_len(&row),
            harmos::MemorySize::memory_size(&row),
            serde_json::to_vec(&schema).unwrap().len(),
        )
    });
    storage.flush_until(&receipt.position).await.unwrap();

    let mut output = expected.snapshot.parameters()["count"].clone();
    output.source = dokime::definition::ParameterSource::Output;
    output.value = Value::Integer(i64::MIN);
    expected.output_parameters.insert("count".into(), output);
    expected.status = dokime::execution::ExecutionStatus::Failed;
    expected.started_at = Some(dokime::core::Timestamp::new("2026-10-05T09:00:00Z"));
    expected.finished_at = Some(dokime::core::Timestamp::new("2026-10-05T09:01:00Z"));
    expected.result = Some(dokime::execution::ResultSummary {
        status: dokime::execution::ResultStatus::Failed,
        passed: 0,
        failed: 1,
        blocked: 0,
        skipped: 0,
    });
    let row = dokime::storage::objects::execution::Execution::try_from(&expected).unwrap();
    let receipt = storage.write_object(&row).await.unwrap();
    storage.flush_until(&receipt.position).await.unwrap();
    runtime.stop().await.unwrap();

    let primary = Arc::new(
        Backend::primary(root.path().join("primary"), Default::default())
            .await
            .unwrap(),
    );
    let recovery = Arc::new(primary.recovery(Default::default()));
    let backends = StorageBackends {
        primary,
        recovery,
        replicas: vec![],
    };
    let storage = Manager::new(backends, StorageSettings::default()).unwrap();
    let runtime = Runtime::<Reader>::builder((), ())
        .runtime_with_storage(&storage)
        .await
        .unwrap();

    let limit = super::FRAME_BYTES.try_into().unwrap();
    let view = storage
        .view_with_limits(StorageSettings::default().query_execution)
        .await
        .unwrap();
    let saved = dokime::storage::objects::execution::read_in(
        &storage,
        &view,
        &expected.test_execution_id,
        harmos::storage::ObjectVersion::Latest,
        limit,
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(&saved.value, &expected);
    assert_eq!(saved.position, receipt.position);
    assert_eq!(saved.version, receipt.version);
    assert_eq!(saved.value.snapshot.case(), &case);
    assert_eq!(
        saved.value.snapshot.parameters()["count"].value,
        Value::Unsigned(u64::MAX)
    );
    assert_eq!(
        saved.value.effective_parameters()["count"].value,
        Value::Integer(i64::MIN)
    );
    let missing = dokime::core::TestExecutionId::from_seed("native-frozen-missing");
    let absent = dokime::storage::objects::execution::read_in(
        &storage,
        &view,
        &missing,
        harmos::storage::ObjectVersion::Latest,
        limit,
    )
    .await
    .unwrap();
    assert!(absent.is_none());
    drop(saved);
    drop(view);
    runtime.stop().await.unwrap();
}

pub(crate) fn prompt_level_rows() {
    use dokime::prompts::{ConfirmationPrompt, Prompt, PromptDefinition};
    use dokime::records::RecordSeverity;
    use dokime::storage::objects::presentation::PromptDefinition as PromptRow;

    // === Store A Raised Prompt === //

    let expected =
        ConfirmationPrompt::new("fixture.level", "Raised").level(RecordSeverity::Warning);
    let expected = Prompt::definition(&expected);
    let row = PromptRow::try_from(&expected).unwrap();
    let restored = PromptDefinition::try_from(row.clone()).unwrap();

    // === Read A Row Written Before Levels === //

    let mut baseline = serde_json::to_value(&row).unwrap();
    baseline.as_object_mut().unwrap().remove("level");
    let baseline: PromptRow = serde_json::from_value(baseline).unwrap();
    let defaulted = PromptDefinition::try_from(baseline).unwrap();

    // === Compare === //

    assert_eq!(restored, expected);
    assert_eq!(restored.level, RecordSeverity::Warning);
    assert_eq!(defaulted.level, RecordSeverity::Info);
}
