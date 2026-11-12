/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Deterministic Three-Day History At Eight Sample Rows Per Second.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Local Imports
use dokime::definition::{TestCase, TestCaseTemplate, TestSuite};
use dokime::domain::model::identity::{CaseId, CommandId, TemplateId};
use dokime::harmos::{Message, schema::Described};
use dokime::runtime::command::CommandContext;
use dokime::telemetry::{TelemetryBatch, TelemetryColumn, TelemetryContract, TelemetryRole, TelemetrySample};
use dokime::{Identifier, InstalledPlugin, Metadata, RuntimeConfig, Timestamp, ValueType};

/* === Constants === */

pub const ROWS: usize = 3 * 24 * 60 * 60 * 8;
pub const PLOTS: usize = 3;
pub const ORIGIN_S: f64 = 1_767_225_600.0;
const STREAM: &str = "bench.pipeline";
const OWNER: &str = "bench_pipeline";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[dokime::harmos::message(harmos_path = ::dokime::harmos)]
#[derive(Clone, PartialEq)]
struct Row {
    timestamp: String,
    y0: f64,
    y1: f64,
    y2: f64,
}

pub struct Hold {
    pub entered: tokio::sync::Notify,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl dokime::ExecutionFixture for Hold {
    fn run_cycle<'a>(
        &'a self, context: dokime::ExecutionFixtureContext, _cycle: &'a dokime::domain::model::execution::result::CycleView,
        _target: &'a dokime::domains::ExecutionProviderTarget,
    ) -> dokime::ExecutionFixtureFuture<'a> {
        Box::pin(async move {
            self.entered.notify_one();
            context.cancelled().await;
            Err(dokime::transport::ProviderExecutionFailure::new("cancel", "benchmark closed"))
        })
    }
}

/* === Public Functions === */

pub fn config(root: &std::path::Path) -> RuntimeConfig {
    let schema = dokime::harmos::schema::graph::Schema::from_declaration(Row::DEFINITION, &Default::default()).unwrap();
    let contract = TelemetryContract {
        stream: Identifier::new(STREAM).unwrap(),
        schema_version: dokime::harmos::domain::streams::revision(&schema, 1).unwrap(),
        metadata: Metadata::new(Identifier::new(STREAM).unwrap(), "Pipeline Samples", None),
        template_id: None,
        owner: None,
        storage_schema: Some(schema),
        columns: (0..PLOTS)
            .map(|column| {
                let key = format!("y{column}");
                TelemetryColumn::new(Metadata::new(Identifier::new(key).unwrap(), "Value", None), ValueType::Number, TelemetryRole::Value)
            })
            .collect(),
        default_x: Some("timestamp".into()),
        interval: None,
        allow_extra_fields: false,
        scalar: false,
        validation_mode: None,
        foreign: false,
    };
    let plugin = InstalledPlugin {
        source: OWNER.into(),
        version: "bench".into(),
        enabled: true,
        telemetry_contracts: vec![contract],
        ..Default::default()
    };
    RuntimeConfig::new(root, Metadata::new(Identifier::new("bench.pipeline").unwrap(), "Pipeline Bench", None))
        .with_installed_plugins([plugin])
}

pub async fn register_execution(runtime: &dokime::RuntimeHandle, hold: &Hold) -> dokime::TestCycleId {
    let metadata = Metadata::new(Identifier::new("bench.pipeline").unwrap(), "Pipeline Bench", None);
    let template_id = TemplateId::from_seed("pipeline-template");
    let suite = TestSuite {
        metadata: metadata.clone(),
        tags: Vec::new(),
        parameter_groups: Vec::new(),
        test_case_templates: vec![TestCaseTemplate {
            test_case_template_id: template_id.clone(),
            metadata: metadata.clone(),
            tags: Vec::new(),
            parameter_groups: Vec::new(),
            parameters: Vec::new(),
            files: Vec::new(),
            phases: Vec::new(),
            criteria: Vec::new(),
            prompt_definitions: Vec::new(),
            subscriptions: Vec::new(),
            document: None,
            attributes: Default::default(),
        }],
    };
    runtime
        .set_suites(
            "provider:bench_pipeline",
            dokime::definition::SuiteSet::new([dokime::domains::TestSuiteDefinition {
                suite_id: "bench.pipeline".into(),
                tags: Vec::new(),
                workflow: Some("run".into()),
                suite,
                record_contracts: Vec::new(),
                telemetry_contracts: Vec::new(),
            }]),
        )
        .await
        .unwrap();
    let case_id = CaseId::from_seed("pipeline-case");
    let case = TestCase {
        suite_id: "bench.pipeline".into(),
        test_case_id: case_id.clone(),
        test_case_template_id: template_id,
        metadata: metadata.clone(),
        suite: Some(metadata.clone()),
        parameters: Default::default(),
        generation_index: 0,
        label_number: 0,
    };
    runtime.register_cases(CommandId::from_seed("pipeline-registration"), vec![case]).await.unwrap();
    let cycle = runtime
        .create_cycle(dokime::transactions::CycleDefinition { metadata, continue_automatically: false }, CommandContext::default())
        .await
        .unwrap();
    runtime.append_executions(cycle.clone(), vec![(execution_id(), case_id)]).await.unwrap();
    let receipt = runtime
        .start_execution(execution_id(), dokime::domain::model::execution::ExecutionMode::Automatic, CommandContext::default())
        .await;
    assert_eq!(receipt.status, dokime::runtime::command::CommandStatus::Accepted, "{receipt:?}");
    hold.entered.notified().await;
    cycle
}

pub fn execution_id() -> dokime::TestExecutionId {
    dokime::TestExecutionId::from_seed("pipeline-execution")
}

pub fn batch(first: usize, count: usize, late: bool, publication: usize, cycle: &dokime::TestCycleId) -> TelemetryBatch {
    let stream = Identifier::new(STREAM).unwrap();
    let samples = (first..first + count)
        .map(|index| {
            let timestamp = Timestamp::from_datetime(
                time::OffsetDateTime::UNIX_EPOCH
                    + time::Duration::seconds(ORIGIN_S as i64)
                    + time::Duration::milliseconds(index as i64 * 125)
                    + time::Duration::milliseconds(if late { 500 } else { 0 }),
            );
            let values = [(index % 8) as f64, if late { 150.0 } else { (index % 776) as f64 / 8.0 }, ((index / 8) % 8) as f64];
            let row = Row {
                timestamp: timestamp.as_str().to_owned(),
                y0: values[0],
                y1: values[1],
                y2: values[2],
            };
            let mut sample = TelemetrySample::new(
                dokime::domain::model::identity::TelemetrySampleId::from_seed(format!("pipeline:{index}:{late}:{publication}")),
                stream.clone(),
                0,
                timestamp,
                OWNER,
            );
            for (column, value) in values.into_iter().enumerate() {
                sample.values.insert(format!("y{column}"), value.into());
            }
            sample.scope = Some(dokime::RecordScope {
                test_cycle_id: Some(cycle.clone()),
                test_execution_id: Some(execution_id()),
                ..Default::default()
            });
            sample.storage_payload = Some(row.encode());
            sample
        })
        .collect();
    TelemetryBatch::new(
        dokime::domain::model::identity::TelemetryBatchId::from_seed(format!("pipeline:{first}:{count}:{late}:{publication}")),
        stream,
        samples,
    )
}

/* === Private Functions === */

/* === Tests === */
