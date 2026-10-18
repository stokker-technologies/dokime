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
use std::sync::Arc;

// Third Party Imports
use harmos::{Application, Runtime, state::Never, storage::StorageBackends};

// Local Imports
use {
    dokime_runtime::CaseId, dokime_runtime::Identifier, dokime_runtime::Metadata, dokime_runtime::TemplateId, dokime_runtime::Value,
    dokime_runtime::definition::TestCase, dokime_runtime::definition::TestSuite, dokime_runtime::storage::Backend,
    dokime_runtime::storage::Manager, dokime_runtime::storage::objects::case, dokime_runtime::storage::objects::case::Case,
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

#[tokio::test]
async fn materialized_cases_reader() {
    let root = tempfile::tempdir().unwrap();
    let primary = Arc::new(Backend::primary(root.path().join("primary")).await.unwrap());
    let recovery = Arc::new(harmos::storage::state::file::FileBackend::new(
        root.path().to_owned(),
        harmos::storage::StorageBackend::history(primary.as_ref()).clone(),
        dokime_runtime::storage::state_settings(),
    ));
    let storage = Manager::new(StorageBackends { primary, state: recovery }, super::sqlite::storage::storage_settings()).unwrap();
    let runtime = Runtime::<Reader>::builder((), ()).runtime_with_storage(&storage).await.unwrap();
    let expected = TestCase {
        suite_id: "fixture.registration".into(),
        test_case_id: CaseId::from_wire("fixture.case:1234567890").unwrap(),
        test_case_template_id: TemplateId::from_seed("template"),
        metadata: Metadata::new(Identifier::new("fixture.case").unwrap(), "Stored Case", None),
        suite: None,
        parameters: std::collections::BTreeMap::from([("count".into(), Value::Unsigned(u64::MAX))]),
        generation_index: 17,
        label_number: 3,
    };
    let row = Case::try_from(&expected).unwrap();
    let receipt = storage.write_object(&row).await.unwrap();
    storage.flush_until(&receipt.position).await.unwrap();
    assert_eq!(case::read(&storage, &expected.test_case_id).await.unwrap().as_ref(), Some(&expected));
    runtime.stop().await.unwrap();

    let primary = Arc::new(Backend::primary(root.path().join("primary")).await.unwrap());
    let recovery = Arc::new(harmos::storage::state::file::FileBackend::new(
        root.path().to_owned(),
        harmos::storage::StorageBackend::history(primary.as_ref()).clone(),
        dokime_runtime::storage::state_settings(),
    ));
    let storage =
        Manager::new(StorageBackends { primary, state: recovery }, crate::engine_support::local_storage::storage_settings()).unwrap();
    let runtime = Runtime::<Reader>::builder((), ()).runtime_with_storage(&storage).await.unwrap();
    assert_eq!(case::read(&storage, &expected.test_case_id).await.unwrap().as_ref(), Some(&expected));
    assert_eq!(case::read(&storage, &CaseId::from_wire("fixture.absent:1234567890").unwrap()).await.unwrap().as_ref(), None);
    let absent = case::read(&storage, &CaseId::from_wire("fixture.absent:1234567890").unwrap()).await.unwrap();
    assert!(absent.is_none());
    runtime.stop().await.unwrap();
}

#[tokio::test]
async fn suite_definitions_producer() {
    let root = tempfile::tempdir().unwrap();
    let primary = Arc::new(Backend::primary(root.path().join("primary")).await.unwrap());
    let recovery = Arc::new(harmos::storage::state::file::FileBackend::new(
        root.path().to_owned(),
        harmos::storage::StorageBackend::history(primary.as_ref()).clone(),
        dokime_runtime::storage::state_settings(),
    ));
    let backends = StorageBackends { primary, state: recovery };
    let storage = Manager::new(backends, crate::engine_support::local_storage::storage_settings()).unwrap();
    let runtime = Runtime::<Reader>::builder((), ()).runtime_with_storage(&storage).await.unwrap();

    let metadata = Metadata::new(Identifier::new("fixture.authored").unwrap(), "Stored Suite", None);
    let definition = serde_json::json!({
        "metadata": metadata,
        "test_case_templates": [{
            "test_case_template_id": TemplateId::from_seed("native-suite-template"),
            "metadata": metadata,
            "parameters": [],
            "document": {
                "document_id": dokime_runtime::DocumentId::from_seed("native-suite-document"),
                "metadata": metadata,
                "cards": [{"metadata": metadata, "type": "text", "data": {"markdown": "Saved Definition"}}]
            },
            "subscriptions": [{"kind": "stream", "identifier": "fixture.voltage"}]
        }]
    });
    let mut expected: TestSuite = serde_json::from_value(definition).unwrap();
    let stored = dokime_runtime::storage::objects::suite::Suite::new("fixture.registration", &expected).unwrap();
    let receipt = storage.write_object(&stored).await.unwrap();
    storage.flush_until(&receipt.position).await.unwrap();

    expected.metadata.identifier = Identifier::new("fixture.changed_author").unwrap();
    expected.metadata.label = "Latest Suite".into();
    let stored = dokime_runtime::storage::objects::suite::Suite::new("fixture.registration", &expected).unwrap();
    let receipt = storage.write_object(&stored).await.unwrap();
    storage.flush_until(&receipt.position).await.unwrap();
    runtime.stop().await.unwrap();

    let primary = Arc::new(Backend::primary(root.path().join("primary")).await.unwrap());
    let recovery = Arc::new(harmos::storage::state::file::FileBackend::new(
        root.path().to_owned(),
        harmos::storage::StorageBackend::history(primary.as_ref()).clone(),
        dokime_runtime::storage::state_settings(),
    ));
    let backends = StorageBackends { primary, state: recovery };
    let storage = Manager::new(backends, crate::engine_support::local_storage::storage_settings()).unwrap();
    let runtime = Runtime::<Reader>::builder((), ()).runtime_with_storage(&storage).await.unwrap();

    let saved = dokime_runtime::storage::objects::suite::read(&storage, "fixture.registration").await.unwrap();
    assert_eq!(saved, Some(expected));
    let absent = dokime_runtime::storage::objects::suite::read(&storage, "fixture.absent").await.unwrap();
    assert!(absent.is_none());
    drop(saved);
    runtime.stop().await.unwrap();
}

#[tokio::test]
async fn cycles_reopen_producers() {
    let root = tempfile::tempdir().unwrap();
    let primary = Arc::new(Backend::primary(root.path().join("primary")).await.unwrap());
    let recovery = Arc::new(harmos::storage::state::file::FileBackend::new(
        root.path().to_owned(),
        harmos::storage::StorageBackend::history(primary.as_ref()).clone(),
        dokime_runtime::storage::state_settings(),
    ));
    let backends = StorageBackends { primary, state: recovery };
    let storage = Manager::new(backends, crate::engine_support::local_storage::storage_settings()).unwrap();
    let runtime = Runtime::<Reader>::builder((), ()).runtime_with_storage(&storage).await.unwrap();

    let metadata = Metadata::new(Identifier::new("fixture.cycle").unwrap(), "Stored Cycle", None);
    let at = dokime_runtime::Timestamp::new("2026-10-05T09:00:00Z");
    let mut expected = dokime_domain::model::execution::result::TestCycle {
        saved: None,
        test_cycle_id: dokime_runtime::TestCycleId::from_seed("native-latest-cycle"),
        session_id: None,
        metadata,
        lifecycle: dokime_domain::model::execution::result::CycleStatus::Ready,
        phase: Default::default(),
        created_at: at.clone(),
        started_at: None,
        updated_at: at,
        execution_ids: vec![
            dokime_runtime::TestExecutionId::from_seed("native-first"),
            dokime_runtime::TestExecutionId::from_seed("native-second"),
        ],
        records: Default::default(),
        continue_on_failure: false,
        result: None,
    };
    let row = dokime_runtime::storage::objects::cycle::Cycle::try_from(&expected).unwrap();
    let receipt = storage.write_object(&row).await.unwrap();
    storage.flush_until(&receipt.position).await.unwrap();

    expected.lifecycle = dokime_domain::model::execution::result::CycleStatus::Completed;
    expected.updated_at = dokime_runtime::Timestamp::new("2026-10-05T09:01:00Z");
    expected.execution_ids.reverse();
    expected.result = Some(dokime_domain::model::execution::result::ResultSummary {
        status: dokime_domain::model::execution::result::ResultStatus::Passed,
        passed: 2,
        failed: 0,
        blocked: 0,
        skipped: 0,
    });
    let row = dokime_runtime::storage::objects::cycle::Cycle::try_from(&expected).unwrap();
    let receipt = storage.write_object(&row).await.unwrap();
    storage.flush_until(&receipt.position).await.unwrap();
    runtime.stop().await.unwrap();

    let primary = Arc::new(Backend::primary(root.path().join("primary")).await.unwrap());
    let recovery = Arc::new(harmos::storage::state::file::FileBackend::new(
        root.path().to_owned(),
        harmos::storage::StorageBackend::history(primary.as_ref()).clone(),
        dokime_runtime::storage::state_settings(),
    ));
    let backends = StorageBackends { primary, state: recovery };
    let storage = Manager::new(backends, crate::engine_support::local_storage::storage_settings()).unwrap();
    let runtime = Runtime::<Reader>::builder((), ()).runtime_with_storage(&storage).await.unwrap();

    let saved = dokime_runtime::storage::objects::cycle::read(&storage, &expected.test_cycle_id, harmos::storage::ObjectVersion::Latest)
        .await
        .unwrap();
    assert_eq!(saved.as_ref().map(|revision| &revision.value), Some(&expected));
    assert_eq!(saved.as_ref().unwrap().position, receipt.position);
    assert_eq!(saved.as_ref().unwrap().version, receipt.version);
    let missing = dokime_runtime::TestCycleId::from_seed("native-missing-cycle");
    let absent = dokime_runtime::storage::objects::cycle::read(&storage, &missing, harmos::storage::ObjectVersion::Latest).await.unwrap();
    assert!(absent.is_none());
    drop(saved);
    runtime.stop().await.unwrap();
}

#[tokio::test]
async fn executions_reopen_results() {
    let root = tempfile::tempdir().unwrap();
    let primary = Arc::new(Backend::primary(root.path().join("primary")).await.unwrap());
    let recovery = Arc::new(harmos::storage::state::file::FileBackend::new(
        root.path().to_owned(),
        harmos::storage::StorageBackend::history(primary.as_ref()).clone(),
        dokime_runtime::storage::state_settings(),
    ));
    let backends = StorageBackends { primary, state: recovery };
    let storage = Manager::new(backends, crate::engine_support::local_storage::storage_settings()).unwrap();
    let runtime = Runtime::<Reader>::builder((), ()).runtime_with_storage(&storage).await.unwrap();

    let case = TestCase {
        suite_id: "fixture.registration".into(),
        test_case_id: CaseId::from_wire("fixture.frozen:1234567890").unwrap(),
        test_case_template_id: TemplateId::from_seed("native-frozen-template"),
        metadata: Metadata::new(Identifier::new("fixture.frozen").unwrap(), "Frozen Case", None),
        suite: None,
        parameters: std::collections::BTreeMap::from([("count".into(), Value::Unsigned(u64::MAX))]),
        generation_index: 2,
        label_number: 0,
    };
    let snapshot = dokime_domain::model::execution::snapshot::ExecutionSnapshot::materialize(
        &case,
        None,
        Default::default(),
        None,
        Default::default(),
    )
    .unwrap()
    .with_host_version(Some("0.1.0".into()));
    let cycle_id = dokime_runtime::TestCycleId::from_seed("native-frozen-cycle");
    let execution_id = dokime_runtime::TestExecutionId::from_seed("native-frozen-execution");
    let scope = dokime_runtime::RecordScope::default()
        .with_cycle(cycle_id.clone())
        .with_test_case(case.test_case_id.clone())
        .with_test_execution(execution_id.clone());
    let mut expected = dokime_domain::model::execution::result::TestExecution {
        saved: None,
        mode: Default::default(),
        active_step_attempt: None,
        capture_phase: Default::default(),
        step_attempts: Vec::new(),
        test_cycle_id: cycle_id,
        test_execution_id: execution_id,
        test_case_id: case.test_case_id.clone(),
        suite: None,
        status: dokime_domain::model::execution::result::ExecutionStatus::Pending,
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
    let row = dokime_runtime::storage::objects::execution::Execution::try_from(&expected).unwrap();
    let receipt = storage.write_object(&row).await.unwrap_or_else(|error| {
        let schema = crate::engine_support::local_storage::declared::<dokime_runtime::storage::objects::execution::Execution>();
        panic!(
            "execution admission failed: {error}; payload={} resident={} schema={}",
            harmos::Message::encoded_len(&row),
            harmos::MemorySize::memory_size(&row),
            serde_json::to_vec(&schema).unwrap().len(),
        )
    });
    storage.flush_until(&receipt.position).await.unwrap();

    let mut output = expected.snapshot.parameters()["count"].clone();
    output.source = dokime_runtime::definition::ParameterSource::Output;
    output.value = Value::Integer(i64::MIN);
    expected.output_parameters.insert("count".into(), output);
    expected.status = dokime_domain::model::execution::result::ExecutionStatus::Failed;
    expected.started_at = Some(dokime_runtime::Timestamp::new("2026-10-05T09:00:00Z"));
    expected.finished_at = Some(dokime_runtime::Timestamp::new("2026-10-05T09:01:00Z"));
    expected.result = Some(dokime_domain::model::execution::result::ResultSummary {
        status: dokime_domain::model::execution::result::ResultStatus::Failed,
        passed: 0,
        failed: 1,
        blocked: 0,
        skipped: 0,
    });
    let row = dokime_runtime::storage::objects::execution::Execution::try_from(&expected).unwrap();
    let receipt = storage.write_object(&row).await.unwrap();
    storage.flush_until(&receipt.position).await.unwrap();
    runtime.stop().await.unwrap();

    let primary = Arc::new(Backend::primary(root.path().join("primary")).await.unwrap());
    let recovery = Arc::new(harmos::storage::state::file::FileBackend::new(
        root.path().to_owned(),
        harmos::storage::StorageBackend::history(primary.as_ref()).clone(),
        dokime_runtime::storage::state_settings(),
    ));
    let backends = StorageBackends { primary, state: recovery };
    let storage = Manager::new(backends, crate::engine_support::local_storage::storage_settings()).unwrap();
    let runtime = Runtime::<Reader>::builder((), ()).runtime_with_storage(&storage).await.unwrap();

    let saved =
        dokime_runtime::storage::objects::execution::read(&storage, &expected.test_execution_id, harmos::storage::ObjectVersion::Latest)
            .await
            .unwrap()
            .unwrap();
    assert_eq!(&saved.value, &expected);
    assert_eq!(saved.position, receipt.position);
    assert_eq!(saved.version, receipt.version);
    assert_eq!(saved.value.snapshot.case(), &case);
    assert_eq!(saved.value.snapshot.parameters()["count"].value, Value::Unsigned(u64::MAX));
    assert_eq!(saved.value.effective_parameters()["count"].value, Value::Integer(i64::MIN));
    let missing = dokime_runtime::TestExecutionId::from_seed("native-frozen-missing");
    let absent =
        dokime_runtime::storage::objects::execution::read(&storage, &missing, harmos::storage::ObjectVersion::Latest).await.unwrap();
    assert!(absent.is_none());
    drop(saved);
    runtime.stop().await.unwrap();
}

#[test]
fn prompt_level_rows() {
    use dokime_runtime::prompts::{ConfirmationPrompt, Prompt, PromptDefinition};
    use dokime_runtime::records::RecordSeverity;
    use dokime_runtime::storage::objects::presentation::PromptDefinition as PromptRow;

    // === Store A Raised Prompt === //

    let expected = ConfirmationPrompt::new("fixture.level", "Raised").level(RecordSeverity::Warning);
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
