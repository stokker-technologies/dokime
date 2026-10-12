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
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    path::Path,
    time::{Duration, Instant},
};
// Third Party Imports
use sha2::{Digest, Sha256};
use tempfile::{TempDir, tempdir};
use tonic::transport::{Channel, Endpoint};
// Local Imports
use dokime_app::capabilities::installed_plugins::load_brand;
use dokime_app::{Runtime, RuntimeCredentialConfig, app, config};
use dokime_protocol::v1::{self};
use dokime_server::grpc::{Config as ServerConfig, Server};

/* === Constants === */
const STARTUP_SUITE: &str = "mini.charger.startup";
const STARTUP_TEMPLATE: &str = "mini.charger.startup.sequence";
const STARTUP_SAMPLES: &str = "mini.charger.startup.samples";
const STARTUP_STREAM_LOG: &str = "mini.charger.startup/stream.log";
const STARTUP_SAMPLE_COUNT: u64 = 40;
const STEADY_SUITE: &str = "mini.charger.steady";
const STEADY_TEMPLATE: &str = "mini.charger.steady.measurement";
const STEADY_SAMPLES: &str = "mini.charger.steady.samples";
const STEADY_EVENTS: &str = "mini.charger.steady.events";

const CONNECTION_RECORD: &str = "tool.connection.acquired";

const HOST_FAST_STREAM: &str = "host.system";

const HOST_STREAMS: &[&str] = &[
    "host.system",
    "host.network",
    "host.wifi",
    "host.wifi.latency",
    "host.wifi.channel_occupancy",
    "host.bluetooth",
];
const STEADY_STREAM_LOG: &str = "mini.charger.steady/stream.log";
const STEADY_HOST_SYSTEM: &str = "host.system";
const STEADY_SAMPLE_COUNT: u64 = 240;
const STEADY_PROMPT: &str = "mini.charger.steady.check";
const STEADY_ACCEPTANCE: &str = "mini.charger.steady.acceptance";
const STEADY_SUMMARY: &str = "steady-state-summary.json";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct NativeHub {
    _root: TempDir,
    runtime: Runtime,
    server: Server,
    connection: Channel,
}

struct HomedRow {
    value: f64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl NativeHub {
    async fn start() -> Self {
        Self::start_with_brand(false).await
    }

    async fn start_with_brand(brand: bool) -> Self {
        if std::env::var_os("RUST_LOG").is_some() {
            let _ = tracing_subscriber::fmt()
                .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
                .with_test_writer()
                .try_init();
        }
        let root = tempdir().unwrap();
        let mut config = config::Config::local(root.path());
        // Keep settings, plugins and credentials hermetic to this test process.
        config.application_root = Some(root.path().join("application"));
        // The first-party bundles `mise run build:plugins:dev` exports are the
        // compositions these scenarios exercise; nothing loads them implicitly.
        let bundles = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/fixtures");
        config.explicit_plugins = ["mini-sidecar", "dokime-route-plugin"]
            .into_iter()
            .map(|package| bundles.join(package).join(format!("{package}.dokime")))
            .collect();
        config.credential_backend = RuntimeCredentialConfig {
            backend: dokime_app::core::CredentialBackendKind::Memory,
            local_file: None,
        };
        if brand {
            config.brand = Some(
                load_brand(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../../branding")).unwrap(),
            );
        }
        let runtime = app::launch(config).await.unwrap();
        let server = Server::start(runtime.handle(), ServerConfig::local())
            .await
            .unwrap();
        let connection = Endpoint::from_shared(server.local_endpoint())
            .unwrap()
            .connect()
            .await
            .unwrap();
        Self {
            _root: root,
            runtime,
            server,
            connection,
        }
    }

    async fn shutdown(mut self) {
        self.server.shutdown().await.unwrap();
        self.runtime.shutdown().await.unwrap();
    }
}

impl dokime_app::StreamSpec for HomedRow {
    type Row = Self;
    const ID: &'static str = STEADY_EVENTS;
    const VERSION: u32 = 1;
    const WINDOW: usize = 4096;
}

impl dokime_app::telemetry::TelemetryStream for HomedRow {
    fn contract() -> dokime_app::telemetry::TelemetryContract {
        dokime_app::telemetry::TelemetryContract::declared(
            <Self as dokime_app::StreamSpec>::ID,
            "Steady Events",
        )
        .columns(<Self as dokime_app::telemetry::TelemetryRow>::columns())
        .strict()
    }
}

impl dokime_app::telemetry::OwnedStream for HomedRow {}

impl dokime_app::telemetry::TelemetryRow for HomedRow {
    fn columns() -> Vec<dokime_app::telemetry::TelemetryColumn> {
        vec![dokime_app::telemetry::TelemetryColumn::declared(
            "value",
            "Value",
            dokime_app::core::ValueType::Number,
            dokime_app::telemetry::TelemetryRole::Value,
        )]
    }

    fn into_row(self) -> BTreeMap<String, dokime_app::core::Value> {
        BTreeMap::from([(
            "value".to_owned(),
            dokime_app::core::Value::Number(self.value),
        )])
    }

    fn from_row(
        values: &BTreeMap<String, dokime_app::core::Value>,
    ) -> Result<Self, dokime_app::telemetry::FieldError> {
        Ok(Self {
            value: dokime_app::telemetry::field(values, "value")?,
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn field<'a>(sample: &'a v1::TelemetrySample, name: &str) -> &'a str {
    match sample
        .values
        .as_ref()
        .and_then(|values| values.fields.get(name))
        .and_then(|value| value.kind.as_ref())
    {
        Some(v1::value::Kind::TextValue(text)) => text,
        other => panic!("sample field {name:?} is not text: {other:?}"),
    }
}

fn number(value: f64) -> v1::Value {
    v1::Value {
        kind: Some(v1::value::Kind::NumberValue(value)),
    }
}

fn text(value: &str) -> v1::Value {
    v1::Value {
        kind: Some(v1::value::Kind::TextValue(value.to_owned())),
    }
}

fn boolean(value: bool) -> v1::Value {
    v1::Value {
        kind: Some(v1::value::Kind::BoolValue(value)),
    }
}

fn selection(
    suite_id: &str,
    template_id: &str,
    parameters: impl IntoIterator<Item = (&'static str, v1::Value)>,
) -> v1::SelectionPlan {
    v1::SelectionPlan {
        selections: vec![v1::RegisteredSelection {
            suite_id: suite_id.to_owned(),
            template_id: template_id.to_owned(),
            parameters: parameters
                .into_iter()
                .map(|(name, value)| {
                    (
                        name.to_owned(),
                        v1::SelectionValues {
                            values: vec![value],
                        },
                    )
                })
                .collect::<HashMap<_, _>>(),
            files: HashMap::new(),
        }],
        max_cases: 0,
        case_ids: vec![],
    }
}

fn scope(cycle_id: &str, execution_id: &str) -> v1::EvidenceScope {
    v1::EvidenceScope {
        cycle_id: Some(cycle_id.to_owned()),
        execution_id: Some(execution_id.to_owned()),
        ..Default::default()
    }
}

async fn suite(
    catalog: &mut v1::catalog_client::CatalogClient<Channel>,
    suite_id: &str,
) -> v1::SuiteDefinition {
    catalog
        .get_suite(v1::GetSuiteRequest {
            suite_id: suite_id.to_owned(),
        })
        .await
        .unwrap()
        .into_inner()
}

async fn create_cycle(
    planning: &mut v1::planning_client::PlanningClient<Channel>,
    execution: &mut v1::execution_client::ExecutionClient<Channel>,
    plan: v1::SelectionPlan,
    identifier: &str,
) -> String {
    let preview = planning
        .preview_selection(v1::PreviewSelectionRequest {
            plan: Some(plan.clone()),
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(preview.cases.len(), 1);
    execution
        .create_cycle(v1::CreateCycleRequest {
            plan: Some(plan),
            metadata: Some(v1::Metadata {
                identifier: identifier.to_owned(),
                label: identifier.to_owned(),
                description: None,
                ..Default::default()
            }),
            ..Default::default()
        })
        .await
        .unwrap()
        .into_inner()
        .cycle_id
}

async fn run_cycle(execution: &mut v1::execution_client::ExecutionClient<Channel>, cycle_id: &str) {
    let reply = execution
        .run_cycle(v1::RunCycleRequest {
            cycle_id: cycle_id.to_owned(),
            ..Default::default()
        })
        .await
        .unwrap()
        .into_inner();
    assert!(matches!(
        reply.outcome,
        Some(v1::run_reply::Outcome::Accepted(_))
    ));
}

async fn wait_prompt(
    prompts: &mut v1::prompts_client::PromptsClient<Channel>,
    cycle_id: &str,
) -> v1::PendingPromptIdentity {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        let page = prompts
            .list_pending(v1::ListPendingPromptsRequest {
                cycle_id: Some(cycle_id.to_owned()),
                ..Default::default()
            })
            .await
            .unwrap()
            .into_inner();
        if let Some(prompt) = page.prompts.into_iter().next() {
            return prompt;
        }
        assert!(Instant::now() < deadline, "prompt did not appear in time");
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

async fn wait_completed(
    execution: &mut v1::execution_client::ExecutionClient<Channel>,
    cycle_id: &str,
) -> v1::CycleSummary {
    let deadline = Instant::now() + Duration::from_secs(40);
    loop {
        let cycle = execution
            .get_cycle(v1::GetCycleRequest {
                cycle_id: cycle_id.to_owned(),
            })
            .await
            .unwrap()
            .into_inner();
        if matches!(
            cycle.status(),
            v1::CycleStatus::Completed | v1::CycleStatus::Failed | v1::CycleStatus::Stopped
        ) {
            return cycle;
        }
        if Instant::now() >= deadline {
            let executions = execution
                .list_executions(v1::ListExecutionsRequest {
                    cycle_id: cycle_id.to_owned(),
                    ..Default::default()
                })
                .await;
            panic!("cycle did not complete in time: {cycle:?}; executions: {executions:?}");
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

async fn execution_id(
    execution: &mut v1::execution_client::ExecutionClient<Channel>,
    cycle_id: &str,
) -> String {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let page = execution
            .list_executions(v1::ListExecutionsRequest {
                cycle_id: cycle_id.to_owned(),
                ..Default::default()
            })
            .await
            .unwrap()
            .into_inner();
        if let Some(item) = page.executions.into_iter().next() {
            return item.execution_id;
        }
        assert!(
            Instant::now() < deadline,
            "execution did not appear in time"
        );
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

async fn read_file(
    content: &mut v1::content_client::ContentClient<Channel>,
    execution_id: &str,
    relative_path: &str,
) -> v1::ExecutionFileWindow {
    content
        .read_execution_file(v1::ReadExecutionFileRequest {
            execution_id: execution_id.to_owned(),
            relative_path: relative_path.to_owned(),
            offset: 0,
            max_bytes: 8192,
        })
        .await
        .unwrap()
        .into_inner()
}

async fn list_file(
    content: &mut v1::content_client::ContentClient<Channel>,
    execution_id: &str,
    relative_path: &str,
) -> v1::ExecutionFileMetadata {
    let page = content
        .list_execution_files(v1::ListExecutionFilesRequest {
            execution_id: execution_id.to_owned(),
            ..Default::default()
        })
        .await
        .unwrap()
        .into_inner();
    page.files
        .into_iter()
        .find(|file| file.relative_path == relative_path)
        .unwrap_or_else(|| panic!("execution file {relative_path} was not present"))
}

fn value_object(fields: impl IntoIterator<Item = (&'static str, v1::Value)>) -> v1::ValueObject {
    v1::ValueObject {
        fields: fields
            .into_iter()
            .map(|(key, value)| (key.to_owned(), value))
            .collect(),
    }
}

async fn charger_telemetry_query_uses_native_prompt_log_telemetry() {
    let hub = NativeHub::start().await;
    let mut catalog = v1::catalog_client::CatalogClient::new(hub.connection.clone());
    let startup = suite(&mut catalog, STARTUP_SUITE).await;
    let template = startup
        .templates
        .iter()
        .find(|template| {
            template
                .metadata
                .as_ref()
                .is_some_and(|metadata| metadata.identifier == STARTUP_TEMPLATE)
        })
        .unwrap_or_else(|| panic!("startup template is registered: {:?}", startup.templates));
    let mut planning = v1::planning_client::PlanningClient::new(hub.connection.clone());
    let plan = selection(
        STARTUP_SUITE,
        &template.template_id,
        [
            ("duration_s", number(STARTUP_SAMPLE_COUNT as f64 * 0.1)),
            ("frequency_hz", number(1.0)),
            ("sample_interval_s", number(0.1)),
            ("realtime", boolean(true)),
        ],
    );
    let mut execution = v1::execution_client::ExecutionClient::new(hub.connection.clone());
    let cycle_id = create_cycle(
        &mut planning,
        &mut execution,
        plan,
        "native.charger.startup",
    )
    .await;
    run_cycle(&mut execution, &cycle_id).await;

    let mut prompts = v1::prompts_client::PromptsClient::new(hub.connection.clone());
    let pending = wait_prompt(&mut prompts, &cycle_id).await;
    assert_eq!(
        pending
            .definition
            .as_ref()
            .expect("prompt definition")
            .prompt_type,
        "mini.charger.startup.check"
    );
    let prompt_scope = pending.scope.expect("prompt scope");
    let execution_id = prompt_scope.execution_id.clone().expect("execution scope");

    let mut content = v1::content_client::ContentClient::new(hub.connection.clone());
    let running_log = tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let file = list_file(&mut content, &execution_id, STARTUP_STREAM_LOG).await;
            if file.state() == v1::ExecutionFileState::Streaming && file.size_bytes > 0 {
                let window = read_file(&mut content, &execution_id, STARTUP_STREAM_LOG).await;
                let text = String::from_utf8_lossy(&window.data).into_owned();
                if text.contains("record=started") && text.contains("record=prompt_requested") {
                    break text;
                }
            }
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
    })
    .await
    .expect("startup stream log is readable while the prompt is pending");
    assert!(running_log.contains("record=started"));
    assert!(running_log.contains("record=prompt_requested"));

    prompts
        .respond(v1::RespondToPromptRequest {
            prompt_id: pending.prompt_id,
            action: "submit".into(),
            values: Some(value_object([
                ("operator_note", text("native-test-confirmed")),
                ("accepted", boolean(true)),
            ])),
            accepted: true,
            ..Default::default()
        })
        .await
        .unwrap();

    let cycle = wait_completed(&mut execution, &cycle_id).await;
    assert_eq!(cycle.status(), v1::CycleStatus::Completed);
    let mut telemetry = v1::telemetry_client::TelemetryClient::new(hub.connection.clone());
    let telemetry = telemetry
        .list_samples(v1::ListTelemetrySamplesRequest {
            streams: vec![STARTUP_SAMPLES.into()],
            scope: Some(scope(&cycle_id, &execution_id)),
            limit: 100,
            ..Default::default()
        })
        .await
        .unwrap()
        .into_inner();
    assert!(telemetry.complete);
    assert_eq!(telemetry.samples.len() as u64, STARTUP_SAMPLE_COUNT);
    assert_eq!(
        telemetry
            .samples
            .iter()
            .map(|sample| sample.stream.as_str())
            .collect::<HashSet<_>>(),
        HashSet::from([STARTUP_SAMPLES])
    );

    let file = list_file(&mut content, &execution_id, STARTUP_STREAM_LOG).await;
    assert_eq!(file.kind(), v1::ExecutionFileKind::Text);
    assert_eq!(file.state(), v1::ExecutionFileState::Finalized);
    let final_log = read_file(&mut content, &execution_id, STARTUP_STREAM_LOG).await;
    let final_text = String::from_utf8_lossy(&final_log.data);
    assert!(final_text.contains("record=prompt_answered action=continue accepted=true"));
    assert!(final_text.contains("Charger resources released"));
    hub.shutdown().await;
}

async fn sidecar_execution_streams_evidence_through_native_grpc() {
    let hub = NativeHub::start().await;
    let mut catalog = v1::catalog_client::CatalogClient::new(hub.connection.clone());
    let steady = suite(&mut catalog, STEADY_SUITE).await;
    let template = steady
        .templates
        .iter()
        .find(|template| {
            template
                .metadata
                .as_ref()
                .is_some_and(|metadata| metadata.identifier == STEADY_TEMPLATE)
        })
        .expect("steady template is registered");
    let mut planning = v1::planning_client::PlanningClient::new(hub.connection.clone());
    let plan = selection(
        STEADY_SUITE,
        &template.template_id,
        [
            ("ac_voltage_v", number(230.0)),
            ("load_ohm", number(100.0)),
            ("frequency_hz", number(1.0)),
            ("duration_s", number(STEADY_SAMPLE_COUNT as f64 * 0.1)),
            ("sample_interval_s", number(0.1)),
        ],
    );
    let mut execution = v1::execution_client::ExecutionClient::new(hub.connection.clone());
    let cycle_id = create_cycle(&mut planning, &mut execution, plan, "native.charger.steady").await;
    run_cycle(&mut execution, &cycle_id).await;
    let execution_id = execution_id(&mut execution, &cycle_id).await;
    let execution_scope = scope(&cycle_id, &execution_id);

    // The sidecar observer runs independently of the prompt step. A non-empty
    // execution-scoped page proves that evidence is visible while the run is
    // still live, before the prompt is answered or the report is finalized.
    let mut telemetry = v1::telemetry_client::TelemetryClient::new(hub.connection.clone());
    let live_deadline = Instant::now() + Duration::from_secs(10);
    let live_samples = loop {
        let page = telemetry
            .list_samples(v1::ListTelemetrySamplesRequest {
                streams: vec![STEADY_SAMPLES.into()],
                scope: Some(execution_scope.clone()),
                // Headroom, not a count: the stream measures at its declared
                // cadence for as long as the prompt is pending, so this page is
                // sized to hold everything rather than to predict how much.
                limit: 5_000,
                ..Default::default()
            })
            .await
            .unwrap()
            .into_inner();
        if !page.samples.is_empty() {
            break page;
        }
        assert!(
            Instant::now() < live_deadline,
            "sidecar telemetry was not visible while execution was running"
        );
        tokio::time::sleep(Duration::from_millis(25)).await;
    };
    assert!(live_samples.samples.len() < STEADY_SAMPLE_COUNT as usize);

    let mut prompts = v1::prompts_client::PromptsClient::new(hub.connection.clone());
    let pending = wait_prompt(&mut prompts, &cycle_id).await;
    let definition = pending.definition.as_ref().expect("prompt definition");
    assert_eq!(definition.prompt_type, STEADY_PROMPT);
    let prompt_scope = pending.scope.as_ref().expect("prompt scope");
    assert_eq!(prompt_scope.cycle_id.as_deref(), Some(cycle_id.as_str()));
    assert_eq!(
        prompt_scope.execution_id.as_deref(),
        Some(execution_id.as_str())
    );

    // Hold the prompt until two host ticks are observable. Charger sample
    // time can advance faster than the host collector's wall-clock cadence.
    let host_deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let page = telemetry
            .list_samples(v1::ListTelemetrySamplesRequest {
                streams: vec![STEADY_HOST_SYSTEM.into()],
                scope: Some(execution_scope.clone()),
                limit: 10,
                ..Default::default()
            })
            .await
            .unwrap()
            .into_inner();
        if page.samples.len() >= 2 {
            break;
        }
        assert!(
            Instant::now() < host_deadline,
            "declared host telemetry did not recur while prompt was pending"
        );
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    prompts
        .respond(v1::RespondToPromptRequest {
            prompt_id: pending.prompt_id,
            action: "submit".into(),
            values: Some(value_object([
                ("operator_note", text("embedded-sidecar-confirmed")),
                ("accepted", boolean(true)),
            ])),
            accepted: true,
            ..Default::default()
        })
        .await
        .unwrap();

    let cycle = wait_completed(&mut execution, &cycle_id).await;
    assert_eq!(cycle.status(), v1::CycleStatus::Completed);
    let summary = execution
        .list_executions(v1::ListExecutionsRequest {
            cycle_id: cycle_id.clone(),
            ..Default::default()
        })
        .await
        .unwrap()
        .into_inner()
        .executions
        .into_iter()
        .next()
        .expect("steady execution summary");
    assert_eq!(summary.execution_id, execution_id);
    assert_eq!(summary.status(), v1::ExecutionStatus::Passed);

    let steps = execution
        .list_execution_steps(v1::ListExecutionStepsRequest {
            execution_id: execution_id.clone(),
            ..Default::default()
        })
        .await
        .unwrap()
        .into_inner()
        .steps;
    let mut criteria = Vec::new();
    for step in steps {
        criteria.extend(
            execution
                .list_step_criteria(v1::ListStepEvidenceRequest {
                    execution_id: execution_id.clone(),
                    step_id: step.step_id,
                    ..Default::default()
                })
                .await
                .unwrap()
                .into_inner()
                .criteria,
        );
    }
    let acceptance = criteria
        .iter()
        .find(|criterion| {
            criterion
                .metadata
                .as_ref()
                .is_some_and(|metadata| metadata.identifier == STEADY_ACCEPTANCE)
        })
        .expect("steady acceptance criterion is present");
    assert_eq!(acceptance.status(), v1::CriterionStatus::Passed);
    assert_eq!(
        acceptance
            .observed
            .as_ref()
            .and_then(|value| value.kind.as_ref()),
        Some(&v1::value::Kind::TextValue("passed".into()))
    );

    let mut content = v1::content_client::ContentClient::new(hub.connection.clone());
    let stream_log = list_file(&mut content, &execution_id, STEADY_STREAM_LOG).await;
    assert_eq!(stream_log.kind(), v1::ExecutionFileKind::Text);
    assert_eq!(stream_log.state(), v1::ExecutionFileState::Finalized);
    let log = read_file(&mut content, &execution_id, STEADY_STREAM_LOG).await;
    let log = String::from_utf8_lossy(&log.data);
    assert!(log.contains("record=sidecar_started"));
    assert!(log.contains("record=sidecar_prompt_answered action=continue accepted=true"));
    assert!(log.contains("Charger resources released"));

    // What the template stated about the DUT before its first step, read back
    // through the typed query an operator's Traceability card reads.
    let mut observation = v1::observation_client::ObservationClient::new(hub.connection.clone());
    let traceability = observation
        .get_traceability(v1::GetTraceabilityRequest {
            scope: Some(execution_scope.clone()),
            ..Default::default()
        })
        .await
        .unwrap()
        .into_inner();
    let dut = traceability
        .snapshots
        .iter()
        .find(|snapshot| {
            snapshot
                .metadata
                .as_ref()
                .is_some_and(|metadata| metadata.identifier == "dut")
        })
        .expect("the steady template states the device it energized");
    assert_eq!(
        dut.metadata
            .as_ref()
            .map(|metadata| metadata.label.as_str()),
        Some("Device Under Test")
    );
    let supply = dut
        .entries
        .iter()
        .find(|entry| {
            entry
                .metadata
                .as_ref()
                .is_some_and(|metadata| metadata.identifier == "nominal_voltage_rms_v")
        })
        .expect("the stated section carries the DUT supply it read");
    assert_eq!(supply.unit.as_deref(), Some("V"));
    assert_eq!(
        supply.source.as_deref(),
        Some("traceability.template.device")
    );

    let final_samples = telemetry
        .list_samples(v1::ListTelemetrySamplesRequest {
            streams: vec![STEADY_SAMPLES.into()],
            scope: Some(execution_scope.clone()),
            // Headroom, not a count: the stream measures at its declared
            // cadence for as long as the prompt is pending, so this page is
            // sized to hold everything rather than to predict how much.
            limit: 5_000,
            ..Default::default()
        })
        .await
        .unwrap()
        .into_inner();
    assert!(final_samples.complete);
    assert!(final_samples.samples.len() >= STEADY_SAMPLE_COUNT as usize);
    assert!(final_samples.samples.iter().all(|sample| {
        sample.stream == STEADY_SAMPLES
            && sample.scope.as_ref().is_some_and(|sample_scope| {
                sample_scope.cycle_id.as_deref() == Some(cycle_id.as_str())
                    && sample_scope.execution_id.as_deref() == Some(execution_id.as_str())
            })
    }));

    // The stream the measuring step pushes into: declared by its row type, with
    // nothing measuring it, and written by hand around the wait on the observer.
    let events = telemetry
        .list_samples(v1::ListTelemetrySamplesRequest {
            streams: vec![STEADY_EVENTS.into()],
            scope: Some(execution_scope.clone()),
            limit: 100,
            ..Default::default()
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(
        events
            .samples
            .iter()
            .map(|sample| field(sample, "event"))
            .collect::<Vec<_>>(),
        ["hold_started", "hold_reached"],
        "a step pushes into a stream no observer measures"
    );
    assert!(
        events.samples.iter().all(|sample| {
            sample
                .scope
                .as_ref()
                .is_some_and(|scope| scope.execution_id.as_deref() == Some(execution_id.as_str()))
        }),
        "a pushed row is scoped to the execution that pushed it"
    );

    // A template naming every host stream in `subscribes(...)` leases them all.
    // Due collectors start on the shared tick and publish independently.
    // Alignment is carried by the timestamp rather than by waiting, so a wifi
    // scan that takes seconds costs only its own stream's ticks.
    let mut host_instants: BTreeMap<&str, Vec<_>> = BTreeMap::new();
    for stream in HOST_STREAMS {
        let page = telemetry
            .list_samples(v1::ListTelemetrySamplesRequest {
                streams: vec![(*stream).into()],
                scope: Some(execution_scope.clone()),
                limit: 500,
                ..Default::default()
            })
            .await
            .unwrap()
            .into_inner();
        for sample in page.samples {
            host_instants
                .entry(*stream)
                .or_default()
                .push(sample.timestamp.expect("a host row carries its instant"));
        }
    }
    assert!(
        host_instants.len() >= 2,
        "a declared host request captures more than one stream: {:?}",
        host_instants.keys().collect::<Vec<_>>()
    );
    // CPU observations recur while other streams collect independently.
    let ticks = host_instants
        .get(HOST_FAST_STREAM)
        .map(|instants| instants.iter().cloned().collect::<HashSet<_>>())
        .unwrap_or_default();
    assert!(
        ticks.len() >= 2,
        "`{HOST_FAST_STREAM}` produces recurring observations: {} rows",
        host_instants
            .get(HOST_FAST_STREAM)
            .map(Vec::len)
            .unwrap_or_default()
    );
    // Each collector has its own cadence and can miss ticks while busy,
    // including host.system. Host capture also starts before the first step
    // reports its execution start. Check the admitted cycle's capture window;
    // controlled observer tests assert exact cross-stream tick alignment.
    let started = cycle.started_at.as_ref().expect("cycle started");
    let finished = cycle.updated_at.as_ref().expect("cycle completed");
    // The lake stores microseconds while lifecycle timestamps retain nanoseconds.
    let micros = |at: &prost_types::Timestamp| {
        i128::from(at.seconds) * 1_000_000 + i128::from(at.nanos) / 1_000
    };
    for (stream, instants) in &host_instants {
        for instant in instants {
            assert!(
                (micros(started)..=micros(finished)).contains(&micros(instant)),
                "`{stream}` row at {instant:?} is outside {started:?}..={finished:?}"
            );
        }
    }

    let mut documents = v1::documents_client::DocumentsClient::new(hub.connection.clone());
    let mut stream = documents
        .capture_current_document(v1::CaptureCurrentDocumentRequest {
            scope: Some(v1::DocumentScope {
                cycle_id: cycle_id.clone(),
                execution_id: execution_id.clone(),
            }),
            document_id: "default".into(),
        })
        .await
        .unwrap()
        .into_inner();
    let mut plot_found = false;
    let mut document_complete = false;
    while let Some(frame) = stream.message().await.unwrap() {
        match frame.frame {
            Some(v1::document_frame::Frame::Card(card)) => {
                if let Some(v1::document_card::Body::Owner(reference)) = card.body
                    && reference.plot_id.as_deref()
                        == Some("mini.charger.steady.measurement.execution.signals")
                {
                    let plot = reference.plot.expect("plot definition");
                    let request = plot.request.expect("plot request");
                    assert_eq!(request.stream, STEADY_SAMPLES);
                    // A time series draws against the row's automatic timestamp
                    // rather than an authored x expression — the wire carries no
                    // x expression for it, and a reader defaults the absent x to
                    // the "timestamp" field itself.
                    assert!(
                        request.x_expression.is_none(),
                        "a time_series plot's x is the automatic timestamp, not an authored expression"
                    );
                    assert!(request.y_expressions.iter().any(|expression| {
                        matches!(
                            expression.source,
                            Some(v1::data_expression::Source::Telemetry(
                                v1::TelemetryDataSource { ref stream, ref field }
                            )) if stream == STEADY_SAMPLES && field == "voltage_v"
                        )
                    }));
                    plot_found = true;
                }
            }
            Some(v1::document_frame::Frame::Completion(_)) => document_complete = true,
            _ => {}
        }
    }
    assert!(
        plot_found,
        "steady telemetry document exposes its signal plot"
    );
    assert!(document_complete, "document stream has a completion frame");

    let mut evidence = v1::evidence_client::EvidenceClient::new(hub.connection.clone());
    let records = evidence
        .list_records(v1::ListRecordsRequest {
            selection: Some(v1::CapturedRecordSelection {
                scope: Some(execution_scope.clone()),
                ..Default::default()
            }),
            ..Default::default()
        })
        .await
        .unwrap()
        .into_inner()
        .records;
    for record_type in [
        "mini.charger.steady.started",
        "mini.charger.steady.completed",
    ] {
        let record = records
            .iter()
            .find(|record| record.record_type == record_type)
            .unwrap_or_else(|| panic!("record {record_type} was not journaled"));
        let record_scope = record.scope.as_ref().expect("record scope");
        assert_eq!(record_scope.cycle_id.as_deref(), Some(cycle_id.as_str()));
        assert_eq!(
            record_scope.execution_id.as_deref(),
            Some(execution_id.as_str())
        );
    }
    let artifact_created = records
        .iter()
        .find(|record| record.record_type == "artifact.created")
        .expect("sidecar artifact creation is journaled");
    let artifact_scope = artifact_created.scope.as_ref().expect("artifact scope");
    assert_eq!(artifact_scope.cycle_id.as_deref(), Some(cycle_id.as_str()));
    assert_eq!(
        artifact_scope.execution_id.as_deref(),
        Some(execution_id.as_str())
    );
    let artifact_id = artifact_created
        .related_artifact_id
        .clone()
        .expect("artifact.created carries its artifact ID");
    let details = artifact_created.details.as_ref().expect("artifact details");
    assert_eq!(
        details.fields["artifact_id"].kind,
        Some(v1::value::Kind::TextValue(artifact_id.clone()))
    );
    assert_eq!(
        details.fields["filename"].kind,
        Some(v1::value::Kind::TextValue(STEADY_SUMMARY.into()))
    );

    let mut artifact_stream = content
        .download_artifact(v1::DownloadArtifactRequest { artifact_id })
        .await
        .unwrap()
        .into_inner();
    let Some(v1::artifact_frame::Payload::Metadata(metadata)) =
        artifact_stream.message().await.unwrap().unwrap().payload
    else {
        panic!("an artifact download opens with its metadata");
    };
    assert_eq!(metadata.filename, STEADY_SUMMARY);
    assert_eq!(metadata.content_type.as_deref(), Some("application/json"));
    assert_eq!(metadata.state(), v1::ArtifactState::Complete);
    let mut artifact_bytes = Vec::new();
    let mut artifact_complete = false;
    while let Some(frame) = artifact_stream.message().await.unwrap() {
        match frame.payload.unwrap() {
            v1::artifact_frame::Payload::Chunk(chunk) => artifact_bytes.extend(chunk.data),
            v1::artifact_frame::Payload::Complete(completion) => {
                assert_eq!(completion.size_bytes, metadata.size_bytes);
                artifact_complete = true;
            }
            v1::artifact_frame::Payload::Metadata(_) => panic!("artifact metadata repeated"),
        }
    }
    assert!(artifact_complete);
    let summary: serde_json::Value = serde_json::from_slice(&artifact_bytes).unwrap();
    assert_eq!(summary["accepted"], true);
    assert_eq!(summary["prompt_action"], "continue");
    assert_eq!(summary["supply"]["voltage_rms_v"], 230.0);
    assert_eq!(summary["supply"]["load_ohm"], 100.0);
    assert!(
        summary["sample_count"]
            .as_u64()
            .is_some_and(|count| count >= STEADY_SAMPLE_COUNT)
    );

    hub.shutdown().await;
}

async fn fixtures_expose_installed_settings_branding_and_diagnostics() {
    let expected_theme_label =
        load_brand(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../../branding"))
            .unwrap()
            .theme
            .expect("repository brand theme")
            .metadata
            .label;
    let hub = NativeHub::start_with_brand(true).await;

    let mut plugins = v1::plugins_client::PluginsClient::new(hub.connection.clone());
    let installed = plugins
        .list_admitted_plugins(v1::ListAdmittedPluginsRequest::default())
        .await
        .unwrap()
        .into_inner();
    assert_eq!(installed.total_count, 3);
    let artifact_ids = installed
        .plugins
        .iter()
        .map(|artifact| artifact.plugin_id.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        artifact_ids,
        std::collections::BTreeSet::from(["host", "route-fixture", "mini-sidecar",])
    );
    assert!(installed.plugins.iter().all(|artifact| artifact.enabled));

    let mut settings = v1::settings_client::SettingsClient::new(hub.connection.clone());
    let frame = settings
        .get_settings(v1::GetSettingsRequest::default())
        .await
        .unwrap()
        .into_inner();
    for section_id in ["identity", "route-fixture"] {
        assert!(
            frame
                .sections
                .iter()
                .any(|section| section.section_id == section_id),
            "settings section {section_id} is missing"
        );
    }
    // One card per plugin, identified the way the artifact is: a plugin declares
    // no card, it declares the readiness route the host's card shows.
    let artifact_id = "route-fixture";
    let card_id = artifact_id;
    let card = frame
        .cards
        .iter()
        .find(|card| card.plugin_id == artifact_id && card.card_id == card_id)
        .unwrap_or_else(|| panic!("settings card {card_id} is missing"));
    assert_eq!(card.state(), v1::SettingsCardState::Disconnected);
    assert!(card.body.as_ref().is_some_and(|body| {
        body.blocks.iter().any(|block| {
            matches!(
                block.kind,
                Some(v1::host_card_block::Kind::Status(v1::HostCardStatus {
                    ref label,
                    tone,
                    ..
                })) if label == "Disconnected" && tone == v1::HostCardTone::Warning as i32
            )
        })
    }));

    settings
        .update_plugin_credential(v1::UpdatePluginCredentialRequest {
            plugin_id: "route-fixture".into(),
            credential_id: "token".into(),
            secret: "native-fixture-token".into(),
        })
        .await
        .unwrap();

    let mut appearance = v1::appearance_client::AppearanceClient::new(hub.connection.clone());
    let appearance = appearance
        .get_appearance(v1::GetAppearanceRequest::default())
        .await
        .unwrap()
        .into_inner();
    assert_eq!(appearance.app_name, "Dokime");
    assert_eq!(appearance.source_plugin_id.as_deref(), Some("brand"));
    assert_eq!(appearance.active_theme.as_deref(), Some("brand.theme"));
    assert_eq!(appearance.themes.len(), 1);
    assert_eq!(appearance.themes[0].theme_id, "brand.theme");
    assert_eq!(appearance.themes[0].label, expected_theme_label);
    let logo = appearance.logo.expect("brand logo");
    assert_eq!(logo.alt.as_deref(), Some("Dokime"));
    let logo_metadata = logo.asset.expect("brand logo metadata");
    assert_eq!(logo_metadata.content_type, "image/svg+xml");
    let logo_reference = logo_metadata.reference.expect("brand logo reference");
    assert_eq!(logo_reference.plugin_id, "brand");
    assert_eq!(logo_reference.asset_id, "brand.logo");
    assert!(!logo_reference.sha256.is_empty());

    let mut content = v1::content_client::ContentClient::new(hub.connection.clone());
    let mut logo_stream = content
        .download_appearance_asset(v1::DownloadAppearanceAssetRequest {
            reference: Some(logo_reference.clone()),
        })
        .await
        .unwrap()
        .into_inner();
    assert!(matches!(
        logo_stream.message().await.unwrap().unwrap().payload,
        Some(v1::appearance_asset_frame::Payload::Metadata(_))
    ));
    let mut logo_bytes = Vec::new();
    let mut logo_complete = false;
    while let Some(frame) = logo_stream.message().await.unwrap() {
        match frame.payload.unwrap() {
            v1::appearance_asset_frame::Payload::Chunk(chunk) => logo_bytes.extend(chunk.data),
            v1::appearance_asset_frame::Payload::Complete(completion) => {
                assert_eq!(completion.size_bytes, logo_metadata.size_bytes);
                assert_eq!(completion.sha256, logo_reference.sha256);
                logo_complete = true;
            }
            v1::appearance_asset_frame::Payload::Metadata(_) => {
                panic!("appearance metadata repeated")
            }
        }
    }
    assert!(logo_complete);
    assert_eq!(logo_bytes.len() as u64, logo_metadata.size_bytes);
    assert_eq!(
        Sha256::digest(&logo_bytes)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>(),
        logo_reference.sha256
    );

    let mut backend = v1::backend_client::BackendClient::new(hub.connection.clone());
    let diagnostics = backend
        .get_diagnostics(v1::GetDiagnosticsRequest::default())
        .await
        .unwrap()
        .into_inner();
    let runtime = diagnostics.runtime.expect("runtime diagnostics");
    assert_eq!(
        runtime.startup_doctor.expect("startup doctor").status(),
        v1::DiagnosticCheckStatus::Ok
    );
    assert!(runtime.path_conventions.iter().any(|path| {
        path.key() == v1::DiagnosticPathKey::HubLog && path.absolute_path.is_some()
    }));
    assert_eq!(
        runtime
            .counts
            .expect("diagnostic counts")
            .installed_plugin_errors,
        0
    );

    hub.shutdown().await;
}

async fn plot_watch_pushes_live_deltas_near_ten_hz() {
    let hub = NativeHub::start().await;
    let mut catalog = v1::catalog_client::CatalogClient::new(hub.connection.clone());
    let startup = suite(&mut catalog, STARTUP_SUITE).await;
    let template = startup
        .templates
        .iter()
        .find(|template| {
            template
                .metadata
                .as_ref()
                .is_some_and(|metadata| metadata.identifier == STARTUP_TEMPLATE)
        })
        .expect("startup template is registered");
    let mut planning = v1::planning_client::PlanningClient::new(hub.connection.clone());
    let plan = selection(
        STARTUP_SUITE,
        &template.template_id,
        [
            ("duration_s", number(12.0)),
            ("frequency_hz", number(1.0)),
            ("sample_interval_s", number(0.1)),
            ("realtime", boolean(true)),
        ],
    );
    let mut execution = v1::execution_client::ExecutionClient::new(hub.connection.clone());
    let cycle_id = create_cycle(
        &mut planning,
        &mut execution,
        plan,
        "native.charger.cadence",
    )
    .await;
    run_cycle(&mut execution, &cycle_id).await;
    let execution_id = execution_id(&mut execution, &cycle_id).await;

    let prompt_channel = hub.connection.clone();
    let prompt_cycle = cycle_id.clone();
    let prompt_task = tokio::spawn(async move {
        let mut prompts = v1::prompts_client::PromptsClient::new(prompt_channel);
        let pending = wait_prompt(&mut prompts, &prompt_cycle).await;
        prompts
            .respond(v1::RespondToPromptRequest {
                prompt_id: pending.prompt_id,
                action: "submit".into(),
                values: Some(value_object([
                    ("operator_note", text("native-cadence-confirmed")),
                    ("accepted", boolean(true)),
                ])),
                accepted: true,
                ..Default::default()
            })
            .await
            .unwrap();
    });

    let mut plots = v1::plots_client::PlotsClient::new(hub.connection.clone());
    let mut stream = plots
        .watch(v1::PlotWatchRequest {
            requests: vec![v1::PlotRequest {
                plot_id: Some("mini.charger.primary".into()),
                plot_type: v1::PlotType::TimeSeries as i32,
                stream: STARTUP_SAMPLES.into(),
                y_keys: vec!["voltage_v".into()],
                x_key: Some("timestamp".into()),
                scope: Some(scope(&cycle_id, &execution_id)),
                ..Default::default()
            }],
        })
        .await
        .unwrap()
        .into_inner();

    let mut delta_times = Vec::new();
    let deadline = Instant::now() + Duration::from_secs(12);
    while delta_times.len() < 16 && Instant::now() < deadline {
        let frame = tokio::time::timeout(Duration::from_secs(2), stream.message())
            .await
            .expect("plot watch remains responsive")
            .unwrap();
        let Some(frame) = frame else {
            break;
        };
        if matches!(
            frame.kind(),
            v1::PlotStreamKind::Append | v1::PlotStreamKind::Replace
        ) {
            delta_times.push(Instant::now());
        }
    }
    drop(stream);
    prompt_task.await.unwrap();
    let cycle = wait_completed(&mut execution, &cycle_id).await;
    assert_eq!(cycle.status(), v1::CycleStatus::Completed);

    assert!(
        delta_times.len() >= 8,
        "expected a live native plot delta stream; captured only {} pushes",
        delta_times.len()
    );
    let mut gaps = delta_times
        .windows(2)
        .map(|pair| pair[1].duration_since(pair[0]).as_secs_f64() * 1000.0)
        .collect::<Vec<_>>();
    gaps.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median = gaps[gaps.len() / 2];
    eprintln!(
        "native plot cadence over {} deltas: median={median:.1}ms",
        delta_times.len()
    );
    assert!(
        median < 220.0,
        "median native plot push gap {median:.1}ms regressed toward the old repull floor"
    );

    hub.shutdown().await;
}

fn connection(resource: &str) -> dokime_app::records::RecordPublishRequest {
    let mut record = dokime_app::records::RecordPublishRequest::new(
        dokime_app::core::Identifier::new(CONNECTION_RECORD).expect("a builtin record type"),
        dokime_app::records::RecordSeverity::Info,
    );
    record.message = Some("a unit reported itself on the bench".to_owned());
    record.details.extend([
        ("resource_id".to_owned(), resource.into()),
        ("resource_kind".to_owned(), "device".into()),
        ("connection_state".to_owned(), "connected".into()),
    ]);

    record
}

async fn awaiting_startup(
    hub: &NativeHub,
    identifier: &str,
    budget_s: f64,
    connect: Option<&str>,
) -> (String, v1::CycleSummary) {
    let mut catalog = v1::catalog_client::CatalogClient::new(hub.connection.clone());
    let startup = suite(&mut catalog, STARTUP_SUITE).await;
    let template = startup
        .templates
        .iter()
        .find(|template| {
            template
                .metadata
                .as_ref()
                .is_some_and(|metadata| metadata.identifier == STARTUP_TEMPLATE)
        })
        .expect("the startup template is registered");
    let mut planning = v1::planning_client::PlanningClient::new(hub.connection.clone());
    let plan = selection(
        STARTUP_SUITE,
        &template.template_id,
        [
            ("duration_s", number(0.4)),
            ("sample_interval_s", number(0.1)),
            ("realtime", boolean(false)),
            ("await_connection", boolean(true)),
            ("connection_budget_s", number(budget_s)),
        ],
    );
    let mut execution = v1::execution_client::ExecutionClient::new(hub.connection.clone());
    let cycle_id = create_cycle(&mut planning, &mut execution, plan, identifier).await;
    run_cycle(&mut execution, &cycle_id).await;
    let execution_id = execution_id(&mut execution, &cycle_id).await;

    // The step logs what the bench already knew before it starts waiting, so
    // seeing that line is what says this execution is live and listening.
    let mut content = v1::content_client::ContentClient::new(hub.connection.clone());
    let observed = tokio::time::timeout(Duration::from_secs(20), async {
        loop {
            let present = content
                .list_execution_files(v1::ListExecutionFilesRequest {
                    execution_id: execution_id.clone(),
                    ..Default::default()
                })
                .await
                .unwrap()
                .into_inner()
                .files
                .into_iter()
                .any(|file| file.relative_path == STARTUP_STREAM_LOG && file.size_bytes > 0);
            if present {
                let window = read_file(&mut content, &execution_id, STARTUP_STREAM_LOG).await;
                let text = String::from_utf8_lossy(&window.data).into_owned();
                if let Some(line) = text
                    .lines()
                    .find(|line| line.contains("record=bench_connections"))
                {
                    break line.to_owned();
                }
            }
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
    })
    .await
    .expect("the waiting step reports what the bench already knew");

    if let Some(resource) = connect {
        hub.runtime
            .handle()
            .publish_record(connection(resource))
            .await
            .expect("the host publishes a builtin connection record");
        let mut prompts = v1::prompts_client::PromptsClient::new(hub.connection.clone());
        let pending = wait_prompt(&mut prompts, &cycle_id).await;
        prompts
            .respond(v1::RespondToPromptRequest {
                prompt_id: pending.prompt_id,
                action: "submit".into(),
                values: Some(value_object([
                    ("operator_note", text("native-record-await")),
                    ("accepted", boolean(true)),
                ])),
                accepted: true,
                ..Default::default()
            })
            .await
            .unwrap();
    }
    let cycle = wait_completed(&mut execution, &cycle_id).await;
    let window = read_file(&mut content, &execution_id, STARTUP_STREAM_LOG).await;
    let mut log = String::from_utf8_lossy(&window.data).into_owned();
    log.push('\n');
    log.push_str(&observed);

    (log, cycle)
}

async fn step_awaits_record_listener_hears_one_from_another_execution() {
    let hub = NativeHub::start().await;

    // A step that waits proceeds when another party publishes what it waits for.
    let (first, cycle) = awaiting_startup(&hub, "native.records.await", 30.0, Some("unit-a")).await;
    assert_eq!(cycle.status(), v1::CycleStatus::Completed, "{first}");
    assert!(
        first.contains("record=bench_connections count=0"),
        "the bench had been told about nothing before the first run: {first}"
    );
    assert!(
        first.contains("record=connected resource=unit-a"),
        "the waiting step read the record the host published: {first}"
    );

    // The listener saw that same record without belonging to that execution, so
    // the next run starts already knowing about the unit.
    let (second, cycle) =
        awaiting_startup(&hub, "native.records.listener", 30.0, Some("unit-b")).await;
    assert_eq!(cycle.status(), v1::CycleStatus::Completed);
    assert!(
        second.contains("record=bench_connections count=1"),
        "a plugin listener fired for a record published during another execution: {second}"
    );
    assert!(
        second.contains("record=connected resource=unit-b"),
        "{second}"
    );

    // And a record nobody publishes ends the step on its own budget.
    let (third, cycle) = awaiting_startup(&hub, "native.records.timeout", 0.25, None).await;
    assert_eq!(cycle.status(), v1::CycleStatus::Failed);
    assert!(
        !third.contains("record=connected"),
        "no unit reported itself, so nothing was read: {third}"
    );
    // A deadline is the standard stream adapter's `Elapsed`, which says only
    // that one elapsed. The SDK owns the wait and reports its elapsed deadline
    // through the single crate-level error boundary.
    assert!(
        third.contains("the deadline elapsed before the stream produced an item"),
        "the failure names the elapsed SDK deadline: {third}"
    );

    hub.shutdown().await;
}

async fn name_without_media_type_refused_row_lands_on_own_stream() {
    let sink = std::sync::Arc::new(dokime_app::transport::recording::RecordingSink::new());
    let artifacts = dokime_app::artifacts::Artifacts::new("acceptance", sink.clone());

    // An artifact name is the one statement the file makes about its content,
    // and there is no typed override left to make that statement a lie.
    let refused = artifacts
        .write("x", b"bytes")
        .expect_err("a name that does not say what the content is is refused");
    // `dokime::Error` is not a name this crate resolves (it depends on
    // `dokime` only through `dokime_app`'s re-exports), so the `io::Error`
    // beneath the refusal is reached through `source()` instead of a match.
    let io_error = std::error::Error::source(&refused)
        .and_then(|source| source.downcast_ref::<std::io::Error>())
        .expect("an artifact refusal keeps the io::Error as its source");
    assert_eq!(io_error.kind(), std::io::ErrorKind::InvalidInput);
    assert!(
        refused
            .to_string()
            .contains("does not say what the content is"),
        "{refused}"
    );
    assert!(artifacts.create("evidence/trace").is_err());
    artifacts
        .write("x.bin", b"bytes")
        .expect("opaque bytes are a statement a name can make");
    assert_eq!(sink.chunks().len(), 1, "a refused name sends no piece");

    // A row type that declared its stream is published on that one and nowhere
    // else — the contract the host validates it against is that stream's, and
    // the call has no room left to name another. Driven through a step's own
    // grant, which is where an author meets the rule.
    let mut transport = dokime_app::transport::recording::RecordingTransport::new();
    let parameters = BTreeMap::new();
    let settings = dokime::plugin::Configuration::default();
    let environment: dokime_app::execution::Environment<'_, (), ()> =
        dokime_app::execution::Environment::new(
            &(),
            &parameters,
            &settings,
            artifacts,
            dokime_app::telemetry::Streams::new(sink.clone()),
            dokime_app::records::Deliveries::new(sink.clone()),
        );
    let step = environment.step(&mut transport);

    step.stream::<HomedRow>()
        .open()
        .push(HomedRow { value: 1.5 })
        .await;
    assert_eq!(sink.rows(STEADY_EVENTS).len(), 1);
    assert!(
        sink.rows(STARTUP_SAMPLES).is_empty(),
        "a declared row type has one home and lands on that one"
    );
}

/* === Tests === */

#[tokio::test]
async fn native_grpc_contract() {
    charger_telemetry_query_uses_native_prompt_log_telemetry().await;
    sidecar_execution_streams_evidence_through_native_grpc().await;
    fixtures_expose_installed_settings_branding_and_diagnostics().await;
    plot_watch_pushes_live_deltas_near_ten_hz().await;
    step_awaits_record_listener_hears_one_from_another_execution().await;
    name_without_media_type_refused_row_lands_on_own_stream().await;
}
