/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::{collections::BTreeMap, sync::Arc};

use harmos::storage::FileReference;
use sha2::Digest;

use dokime_domain::model;
use dokime_domain::model::execution::saved::ExecutionRevision;
use dokime_runtime::records::RecordContract;
use dokime_runtime::report::{CycleReportInput, saved::CapturedExecutionReportInput};
use dokime_runtime::storage::{Backend, Manager, objects::execution::Execution};
use dokime_runtime::telemetry::contract::TelemetryContract;
use harmos::storage::StorageBackends;
use model::execution::{HealthItem, OperatorNote};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReportOwner;

impl harmos::Application for ReportOwner {
    type Lanes = ReportOwner;
    type State = ();
    type Transaction = harmos::state::Never;
    type Record = harmos::state::Never;
    type Streams = ();
    type Resources = ();
}

impl harmos::operations::Lane for ReportOwner {
    fn default_lane() -> Self {
        Self
    }
    fn name(&self) -> &'static str {
        "Report Storage"
    }
    fn capacity(&self) -> harmos::operations::Capacity {
        harmos::operations::Capacity::unbounded()
    }
}

pub async fn storage_runtime(manager: &Manager) -> harmos::Runtime<ReportOwner> {
    use harmos::operations::Lane;
    let runtime = harmos::Runtime::<ReportOwner>::builder((), ())
        .register(harmos::operations::LaneSet::declared([(ReportOwner.name(), ReportOwner.capacity())]))
        .runtime_with_storage(manager)
        .await
        .unwrap();
    let storage = harmos::builtin::storage_registration(Arc::new(manager.clone())).unwrap();
    runtime.operations.register(harmos::RegistrationExt::lane(storage, ReportOwner)).await.unwrap();
    runtime
}

pub async fn bytes(manager: &Manager, file: &FileReference) -> Vec<u8> {
    let bytes = dokime_runtime::storage::read_file_bytes(manager, file).await.unwrap();
    assert_eq!(bytes.len() as u64, file.length);
    assert_eq!(sha2::Sha256::digest(bytes.as_slice()).as_slice(), file.sha256.as_slice());
    bytes.as_slice().to_vec()
}

pub async fn opened_bytes(report: &mut dokime_runtime::OpenedReport) -> Vec<u8> {
    let mut bytes = Vec::new();
    while let Some(batch) = report.reader.next().await.unwrap() {
        assert_eq!(batch.offset(), bytes.len() as u64);
        assert!(batch.bytes().len() <= 8 * 1024 * 1024);
        bytes.extend_from_slice(batch.bytes());
    }
    report.reader.completion().unwrap();
    assert_eq!(bytes.len() as u64, report.file.length);
    assert_eq!(sha2::Sha256::digest(&bytes).as_slice(), report.file.sha256.as_slice());
    bytes
}

pub async fn payload(manager: &Manager, file: &FileReference) -> serde_json::Value {
    parse(&String::from_utf8(bytes(manager, file).await).unwrap())
}

pub fn parse(html: &str) -> serde_json::Value {
    let json = html.split_once(r#"<script id="hub-report-data" type="application/json">"#).unwrap().1.split_once("</script>").unwrap().0;
    serde_json::from_str(json).unwrap()
}

pub fn payload_text(html: &str) -> &str {
    html.split_once(r#"<script id="hub-report-data" type="application/json">"#).unwrap().1.split_once("</script>").unwrap().0.trim()
}

pub async fn render(input: &CycleReportInput) -> String {
    render_annotated(input, Vec::new(), Vec::new()).await
}

pub async fn render_annotated(input: &CycleReportInput, notes: Vec<OperatorNote>, health: Vec<HealthItem>) -> String {
    assert_eq!(input.cycle.test_executions.len(), 1);

    let root = tempfile::tempdir().unwrap();
    let primary = Arc::new(Backend::primary(root.path().join("primary")).await.unwrap());
    let settings = crate::service_support::budgets::story_settings();
    let recovery = Arc::new(harmos::storage::state::file::FileBackend::new(
        root.path().to_owned(),
        harmos::storage::StorageBackend::history(primary.as_ref()).clone(),
        dokime_runtime::config::storage_state_defaults(),
    ));
    let manager = Manager::new(StorageBackends { primary, state: recovery }, settings).unwrap();
    let owner = storage_runtime(&manager).await;

    // Settle The Execution Like A Final Capture
    let mut input = input.clone();
    let finished_at = input.generated_at.clone();
    let execution = &mut input.cycle.test_executions[0];
    execution.finished_at.get_or_insert(finished_at);
    let execution = execution.clone();
    let row = Execution::try_from(&execution).unwrap();
    let receipt = manager.write_object(&row).await.unwrap();

    // Freeze The Inspection Context And Declared Contracts
    let mut grouping = input.cycle.clone();
    grouping.test_executions.clear();
    let context = dokime_runtime::execution::context::ExecutionContext {
        appearance_policy: dokime_runtime::execution::context::captured::AppearancePolicy::capture(),
        scope: execution.scope.clone(),
        session_id: input.session_id.clone(),
        cycle: dokime_runtime::execution::context::ExecutionCycleContext {
            cycle: grouping,
            cases: vec![execution.snapshot.case().clone()],
            execution_order: vec![execution.test_execution_id.clone()],
        },
        execution_documents: BTreeMap::new(),
        prompts: Vec::new(),
        health,
        notes,
        traceability: input.traceability.clone(),
        appearance: serde_json::from_value(serde_json::json!({
            "app_name": "Fixture", "theme_family_locked": false,
            "mode_policy": "user_or_system", "mode_locked": false
        }))
        .unwrap(),
    };
    let mut record_contracts: BTreeMap<_, BTreeMap<_, _>> = BTreeMap::new();
    for record in &input.records {
        let mut contract = RecordContract::declared(record.record_type.as_str(), "Fixture Record");
        contract.schema_version = record.schema_version;
        record_contracts.entry(record.record_type.clone()).or_default().insert(record.schema_version, contract);
    }
    let mut telemetry_contracts: BTreeMap<_, BTreeMap<_, _>> = BTreeMap::new();
    for sample in &input.telemetry {
        let mut contract = TelemetryContract::declared(sample.stream.as_str(), "Fixture Stream");
        contract.schema_version = sample.schema_version;
        contract.storage_schema = Some(
            crate::service_support::contracts::fixture_schema(sample.stream.as_str()).expect("report fixture declares its row schema"),
        );
        telemetry_contracts.entry(sample.stream.clone()).or_default().insert(sample.schema_version, contract);
    }
    let captured = CapturedExecutionReportInput {
        report_defaults: Default::default(),
        final_report: true,
        owner_node_id: dokime_runtime::NodeId::from_seed("fixture.report.owner"),
        execution_id: execution.test_execution_id.clone(),
        lifecycle_id: "00000000-0000-4000-8000-000000000001".into(),
        execution: Some(ExecutionRevision {
            execution_id: execution.test_execution_id.clone(),
            position: receipt.position.clone(),
            version: receipt.version,
        }),
        evidence_through: receipt.position,
        telemetry_contracts,
        record_contracts,
        input,
        context,
    };

    // Prepare, Save, And Read Back The Stored Report
    let captured = harmos::storage::ReadBatch::new(captured);
    let revision = captured.execution.as_ref().unwrap().clone();
    let outputs = Vec::new();
    let mut prepared = dokime_runtime::report::Renderer.execution(captured, outputs, &manager.cursor_saved()).unwrap();
    let seal = prepared.save_dependencies(&manager, &execution.finished_at.clone().unwrap(), &revision).await.unwrap();
    let html = dokime_runtime::storage::read_file_bytes(&manager, &seal.report).await.unwrap();
    let html = String::from_utf8(html.as_slice().to_vec()).unwrap();
    drop(prepared);
    owner.stop().await.unwrap();
    html
}
