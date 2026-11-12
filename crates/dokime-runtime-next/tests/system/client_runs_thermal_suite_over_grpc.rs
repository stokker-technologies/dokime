/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Client Runs a Thermal Suite over gRPC

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::process::Stdio;

// Third Party Imports
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::{Child, Command},
};

// Local Imports
use dokime::v1;
use dokime_runtime_next::PluginProcess;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(super) struct HostProcess {
    child: Child,
    pub(super) url: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl HostProcess {
    pub(super) async fn start(root: &std::path::Path) -> Result<Self, Box<dyn std::error::Error>> {
        let mut child = Command::from(dokime_runtime_next::process::command(super::support::host_fixture()?))
            .args(["--endpoint", "127.0.0.1:0", "--storage"])
            .arg(root)
            .arg("--controlled-clock")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .kill_on_drop(true)
            .spawn()?;
        let mut lines = BufReader::new(child.stdout.take().unwrap()).lines();
        let url = lines.next_line().await?.ok_or("Host Exited Before Announcing Its Listener")?;
        assert!(url.starts_with("http://127.0.0.1:"));
        Ok(Self { child, url })
    }
    pub(super) async fn close(mut self) -> Result<(), Box<dyn std::error::Error>> {
        drop(self.child.stdin.take());
        assert!(self.child.wait().await?.success());
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[tokio::test]
async fn client_runs_thermal_suite_over_grpc() -> Result<(), Box<dyn std::error::Error>> {
    tokio::time::timeout(std::time::Duration::from_secs(60), async {
        // Start a Real Host Process and the Shared Thermal Plugin
        let root = tempfile::tempdir()?;
        let settings = super::support::settings();
        let executable = super::support::thermal_fixture()?;
        let mut plugin = PluginProcess::start(&executable, true, &settings.placement).await?;
        let process = HostProcess::start(root.path()).await?;
        let channel = tonic::transport::Endpoint::from_shared(process.url.clone())?.connect().await?;
        let mut plugins = v1::plugins_client::PluginsClient::new(channel.clone());
        let mut catalog = v1::catalog_client::CatalogClient::new(channel.clone());
        let mut planning = v1::planning_client::PlanningClient::new(channel.clone());
        let mut execution = v1::execution_client::ExecutionClient::new(channel.clone());
        let mut observation = v1::observation_client::ObservationClient::new(channel.clone());
        let mut prompts = v1::prompts_client::PromptsClient::new(channel.clone());
        let mut query = v1::query_client::QueryClient::new(channel.clone());
        let mut evidence = v1::evidence_client::EvidenceClient::new(channel.clone());
        let mut telemetry = v1::telemetry_client::TelemetryClient::new(channel.clone());
        let mut reports = v1::reports_client::ReportsClient::new(channel.clone());
        let mut backend = v1::backend_client::BackendClient::new(channel.clone());
        let mut health = v1::health_client::HealthClient::new(channel);
        let identity = backend.get_identity(v1::GetBackendIdentity {}).await?.into_inner();
        assert_eq!(backend.get_health(v1::GetBackendHealth {}).await?.into_inner().readiness, v1::BackendReadiness::Ready as i32);
        health.list_health(v1::HealthListHealthRequest::default()).await?;
        let availability = backend.get_availability(v1::GetBackendAvailability {}).await?.into_inner();
        assert!(availability.methods.iter().any(|method| method == "/dokime.v1.Plugins/Add"));

        // Add by URL and Read the Admitted Catalog Through the SDK's Generated Clients
        let entry = plugins.add(v1::AddRemotePlugin { url: plugin.endpoint().into() }).await?.into_inner();
        assert_eq!(plugins.list(v1::ListPlugins {}).await?.into_inner().plugins, vec![entry]);
        let suites = catalog.list_suites(v1::CatalogListSuitesRequest::default()).await?.into_inner();
        let suite_id = suites.suites[0].suite_id.clone();
        let suite = catalog.get_suite(v1::CatalogGetSuiteRequest { suite_id: suite_id.clone() }).await?.into_inner();
        assert_eq!(suite.templates[0].metadata.as_ref().unwrap().identifier, "thermal.cycle");

        // Materialize a Parameter Set and Start Its Frozen Origins
        let planned = planning.materialize_template(v1::PlanningMaterializeTemplateRequest {
            selection: Some(v1::RegisteredSelection {
                suite_id, template_id: "thermal.cycle".into(),
                parameters: vec![v1::RegisteredSelectionParametersEntry { key: "limit".into(), value: Some(v1::SelectionValues {
                    values: vec![v1::Value { kind: Some(v1::value::Kind::Number(85.0)) }]
                }) }], ..Default::default()
            })
        }).await?.into_inner();
        assert_eq!(planned.cases.len(), 1);
        let cycle = execution.create_cycle(v1::ExecutionCreateCycleRequest {
            origins: planned.cases.into_iter().map(|case| case.origin.unwrap()).collect(),
            continue_automatically: true, ..Default::default()
        }).await?.into_inner();
        let execution_id = cycle.execution_ids[0].clone();
        let mut progress = observation.watch_execution_progress(v1::ObservationWatchExecutionProgressRequest {
            test_execution_id: execution_id.clone()
        }).await?.into_inner();

        // Await the Door Prompt Event and Answer Through the Prompts Service
        let mut phases = Vec::new();
        while let Some(frame) = progress.message().await? {
            if let Some(v1::observation_watch_execution_progress_output::Value::Item(frame)) = frame.value {
                let phase = frame.progress.unwrap().phase;
                phases.push(phase.clone());
                if phase == "prompt" { break; }
            }
        }
        assert!(phases.iter().any(|phase| phase == "prompt"));
        let pending = prompts.list_pending(v1::PromptsListPendingRequest { execution_id: Some(execution_id.clone()), ..Default::default() }).await?.into_inner();
        let prompt_id = pending.prompts[0].prompt_id.clone();
        let prompt = prompts.get_prompt(v1::PromptsGetPromptRequest { prompt_id: prompt_id.clone() }).await?.into_inner();
        assert_eq!(prompt.prompt.unwrap().prompt_id, prompt_id);
        prompts.respond(v1::PromptsRespondRequest { prompt_id, action: "yes".into(), accepted: true, ..Default::default() }).await?;

        // Drain Progress Through Its Terminal Response and Successful Final Status
        let mut terminal = false;
        while let Some(frame) = progress.message().await? {
            match frame.value.unwrap() {
                v1::observation_watch_execution_progress_output::Value::Item(frame) => phases.push(frame.progress.unwrap().phase),
                v1::observation_watch_execution_progress_output::Value::Response(_) => terminal = true,
            }
        }
        assert!(terminal);
        assert!(phases.iter().any(|phase| phase == "finished"));

        // Read the Sealed Evidence, Acknowledged Records and Three Controlled Ticks
        let detail = execution.get_execution(v1::ExecutionGetExecutionRequest { execution_id: execution_id.clone() }).await?.into_inner();
        assert_eq!(detail.summary.as_ref().unwrap().state, v1::ExecutionState::Sealed as i32);
        assert!(!detail.seal.is_empty());
        assert!(matches!(detail.outcome.as_ref().unwrap().attribution.as_ref(), Some(v1::execution_outcome::Attribution::Evaluated(outcome)) if outcome.status == v1::ResultStatus::Passed as i32));
        let page = query.list_executions(v1::QueryListExecutionsRequest::default()).await?.into_inner();
        assert_eq!(page.rows[0], *detail.summary.as_ref().unwrap());
        let scope = v1::EvidenceScope { execution_id: Some(execution_id.clone()), ..Default::default() };
        let records = evidence.list_records(v1::EvidenceListRecordsRequest {
            selection: Some(v1::CapturedRecordSelection { scope: Some(scope.clone()), ..Default::default() }), ..Default::default()
        }).await?.into_inner();
        assert!(!records.records.is_empty());
        assert!(!records.records[0].payload.is_empty());
        let frozen = detail.schemas.as_ref().unwrap().catalog.as_ref().unwrap();
        let pool = prost_reflect::DescriptorPool::from_file_descriptor_set(frozen.types.clone().unwrap())?;
        let door = prost_reflect::DynamicMessage::decode(pool.get_message_by_name("thermal.v1.Door").unwrap(), records.records[0].payload.as_slice())?;
        assert_eq!(door.get_field_by_name("closed").unwrap().as_ref(), &prost_reflect::Value::Bool(true));
        let samples = telemetry.list_samples(v1::TelemetryListSamplesRequest { scope: Some(scope), ..Default::default() }).await?.into_inner();
        assert_eq!(samples.samples.len(), 3);
        assert_eq!(samples.samples.iter().map(|sample| sample.timestamp.unwrap().seconds).collect::<Vec<_>>(), vec![0, 1, 2]);
        assert!(samples.samples.iter().all(|sample| !sample.payload.is_empty()));
        assert!(samples.samples.iter().all(|sample| sample.values.as_ref().unwrap().values["celsius"].kind == Some(v1::value::Kind::Number(84.0))));

        // Download the Report Including Metadata, Ordered Bytes and Digest Completion
        let mut report = reports.download_execution_report(v1::ReportsDownloadExecutionReportRequest { execution_id: execution_id.clone() }).await?.into_inner();
        let mut bytes = Vec::new();
        let mut completion = None;
        let mut report_terminal = false;
        while let Some(frame) = report.message().await? {
            match frame.value.unwrap() {
                v1::reports_download_execution_report_output::Value::Item(frame) => match frame.payload.unwrap().kind.unwrap() {
                    v1::report_html_frame_payload::Kind::Report(_) => assert!(bytes.is_empty()),
                    v1::report_html_frame_payload::Kind::Chunk(chunk) => { assert_eq!(chunk.offset, bytes.len() as u64); bytes.extend(chunk.data); },
                    v1::report_html_frame_payload::Kind::Complete(value) => completion = Some(value),
                },
                v1::reports_download_execution_report_output::Value::Response(_) => report_terminal = true,
            }
        }
        assert!(report_terminal);
        let completion = completion.unwrap();
        assert_eq!(completion.size_bytes, bytes.len() as u64);
        use sha2::Digest;
        assert_eq!(completion.sha256, sha2::Sha256::digest(&bytes).iter().map(|byte| format!("{byte:02x}")).collect::<String>());
        assert!(String::from_utf8(bytes)?.contains(&execution_id));

        // Refuse Unsupported Operations by Name
        let refused = plugins.reload(v1::PluginsReloadRequest::default()).await.unwrap_err();
        assert_eq!(refused.code(), tonic::Code::Unimplemented);
        assert!(refused.message().contains("Plugins.Reload"));

        // Restart the Host and Read Exactly the Same Sealed Execution
        process.close().await?;
        let restarted = HostProcess::start(root.path()).await?;
        let mut reader = v1::execution_client::ExecutionClient::connect(restarted.url.clone()).await?;
        let restored = reader.get_execution(v1::ExecutionGetExecutionRequest { execution_id: execution_id.clone() }).await?.into_inner();
        assert_eq!(restored, detail);
        let mut evidence_reader = v1::evidence_client::EvidenceClient::connect(restarted.url.clone()).await?;
        let scope = v1::EvidenceScope { execution_id: Some(execution_id), ..Default::default() };
        let restored_records = evidence_reader.list_records(v1::EvidenceListRecordsRequest {
            selection: Some(v1::CapturedRecordSelection { scope: Some(scope.clone()), ..Default::default() }), ..Default::default()
        }).await?.into_inner();
        assert_eq!(restored_records, records);
        let mut telemetry_reader = v1::telemetry_client::TelemetryClient::connect(restarted.url.clone()).await?;
        let restored_samples = telemetry_reader.list_samples(v1::TelemetryListSamplesRequest { scope: Some(scope), ..Default::default() }).await?.into_inner();
        assert_eq!(restored_samples, samples);
        let mut backend = v1::backend_client::BackendClient::connect(restarted.url.clone()).await?;
        let restored_identity = backend.get_identity(v1::GetBackendIdentity {}).await?.into_inner();
        assert_eq!(restored_identity.backend_id, identity.backend_id);
        assert_ne!(restored_identity.epoch, identity.epoch);
        restarted.close().await?;
        plugin.close().await?;
        Ok(())
    }).await?
}
