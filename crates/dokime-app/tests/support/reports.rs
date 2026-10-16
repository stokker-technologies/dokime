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
use std::{collections::BTreeMap, num::NonZeroUsize, sync::Arc, time::Duration};

// Third Party Imports
use harmos::storage::FileReference;
use sha2::Digest;

// Local Imports
use dokime::execution::saved::ExecutionRevision;
use dokime::execution::{HealthItem, OperatorNote};
use dokime::records::RecordContract;
use dokime::storage::sqlite::{Limits, StateLimits};
use dokime::storage::{Backend, Manager, objects::execution::Execution};
use dokime::telemetry::contract::TelemetryContract;
use dokime_app::report::{CycleReportInput, saved::CapturedExecutionReportInput};
use harmos::storage::StorageBackends;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct ReportOwner;

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl harmos::Application for ReportOwner {
    type State = ();
    type Transaction = harmos::state::Never;
    type Record = harmos::state::Never;
    type Streams = ();
    type Resources = ();
}

impl harmos::operations::Lane for ReportOwner {
    fn name(&self) -> &'static str {
        "Report Storage"
    }
    fn capacity(&self) -> harmos::operations::Capacity {
        harmos::operations::Capacity::unbounded()
    }
}

/* === Public Functions === */
pub fn register_storage(operations: &harmos::operations::OperationManager, manager: &Manager) {
    harmos::storage::operations::Operations::register(
        operations,
        "fixture.storage",
        ReportOwner,
        Arc::new(manager.clone()),
    )
    .unwrap();
}

pub async fn bytes(manager: &Manager, file: &FileReference) -> Vec<u8> {
    let view = manager
        .view_with_limits(crate::support::budgets::story_settings().finalization())
        .await
        .unwrap();
    let mut reader = view.read_file(file).await.unwrap();
    let mut bytes = Vec::new();
    while let Some(chunk) = reader
        .next_chunk(NonZeroUsize::new(64 * 1024).unwrap())
        .await
        .unwrap()
    {
        bytes.extend_from_slice(chunk.as_bytes());
    }
    assert_eq!(bytes.len() as u64, file.length);
    assert_eq!(
        sha2::Sha256::digest(&bytes).as_slice(),
        file.sha256.as_slice()
    );
    bytes
}

pub async fn payload(manager: &Manager, file: &FileReference) -> serde_json::Value {
    parse(&String::from_utf8(bytes(manager, file).await).unwrap())
}

pub fn parse(html: &str) -> serde_json::Value {
    let json = html
        .split_once(r#"<script id="hub-report-data" type="application/json">"#)
        .unwrap()
        .1
        .split_once("</script>")
        .unwrap()
        .0;
    serde_json::from_str(json).unwrap()
}

pub fn payload_text(html: &str) -> &str {
    html.split_once(r#"<script id="hub-report-data" type="application/json">"#)
        .unwrap()
        .1
        .split_once("</script>")
        .unwrap()
        .0
        .trim()
}

pub async fn render(input: &CycleReportInput) -> String {
    render_annotated(input, Vec::new(), Vec::new()).await
}

/// Renders one execution whose frozen context carries these notes and health items.
pub async fn render_annotated(
    input: &CycleReportInput,
    notes: Vec<OperatorNote>,
    health: Vec<HealthItem>,
) -> String {
    assert_eq!(input.cycle.test_executions.len(), 1);

    let root = tempfile::tempdir().unwrap();
    let primary = Arc::new(
        Backend::primary(root.path().join("primary"), Limits::default())
            .await
            .unwrap(),
    );
    let recovery = Arc::new(primary.state(StateLimits::default()));
    let manager = Manager::new(
        StorageBackends {
            primary,
            state: recovery,
            replicas: vec![],
        },
        crate::support::budgets::story_settings().harmos(
            Duration::from_secs(60),
            std::num::NonZeroUsize::new(8 * 1024 * 1024).unwrap(),
        ),
    )
    .unwrap();
    let owner = harmos::Runtime::<ReportOwner>::builder((), ())
        .runtime_with_storage(&manager)
        .await
        .unwrap();
    register_storage(&owner.operations, &manager);

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
    let context = dokime_app::execution::context::ExecutionContext {
        scope: execution.scope.clone(),
        session_id: input.session_id.clone(),
        cycle: dokime_app::execution::context::ExecutionCycleContext {
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
        record_contracts
            .entry(record.record_type.clone())
            .or_default()
            .insert(record.schema_version, contract);
    }
    let mut telemetry_contracts: BTreeMap<_, BTreeMap<_, _>> = BTreeMap::new();
    for sample in &input.telemetry {
        let mut contract = TelemetryContract::declared(sample.stream.as_str(), "Fixture Stream");
        contract.schema_version = sample.schema_version;
        contract.storage_schema = Some(
            crate::support::contracts::fixture_schema(sample.stream.as_str())
                .expect("report fixture declares its row schema"),
        );
        telemetry_contracts
            .entry(sample.stream.clone())
            .or_default()
            .insert(sample.schema_version, contract);
    }
    let captured = CapturedExecutionReportInput {
        final_report: true,
        owner_node_id: dokime::core::NodeId::from_seed("fixture.report.owner"),
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
    let captured = manager
        .produce(
            NonZeroUsize::new(harmos::MemorySize::memory_size(&captured)).unwrap(),
            |_| Ok(captured),
        )
        .unwrap()
        .try_into_value()
        .unwrap()
        .try_into_shared()
        .unwrap();
    let revision = captured.execution.as_ref().unwrap().clone();
    let outputs = manager
        .produce(
            NonZeroUsize::new(std::mem::size_of::<
                Vec<dokime::execution::saved::ExecutionOutput>,
            >())
            .unwrap(),
            |_| Ok(Vec::new()),
        )
        .unwrap()
        .try_into_value()
        .unwrap();
    let mut prepared = dokime_app::report::Renderer
        .execution(manager.clone(), captured, outputs)
        .unwrap();
    let seal = prepared
        .save_dependencies(
            &owner.operations,
            &execution.finished_at.clone().unwrap(),
            &revision,
        )
        .await
        .unwrap();
    let mut reader = manager.read_file(&seal.report).await.unwrap();
    let mut html = Vec::new();
    while let Some(chunk) = reader
        .next_chunk(NonZeroUsize::new(64 * 1024).unwrap())
        .await
        .unwrap()
    {
        html.extend_from_slice(chunk.as_bytes());
    }
    drop(reader);
    let html = String::from_utf8(html).unwrap();
    drop(prepared);
    owner.stop().await.unwrap();
    html
}

/* === Private Functions === */

/* === Tests === */
