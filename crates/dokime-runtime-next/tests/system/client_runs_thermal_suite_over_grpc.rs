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
    pub(super) async fn start_with_environment(
        root: &std::path::Path, environment: &[(&str, std::ffi::OsString)],
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut child = Command::from(dokime_runtime_next::process::command(super::support::host_fixture()?))
            .envs(environment.iter().cloned())
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
    pub(super) async fn kill(mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.child.kill().await?;
        assert!(!self.child.wait().await?.success());
        Ok(())
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
    tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
        // Start a Real Host Process and the Shared Thermal Plugin
        let root = super::support::tempdir()?;
        let credential_root = super::support::tempdir()?;
        let credentials = super::support::CredentialStore::start(credential_root.path()).await?;
        let settings = super::support::settings();
        let executable = super::support::thermal_fixture()?;
        let mut plugin = PluginProcess::start(&executable, true, &settings.placement).await?;
        let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
        let channel = tonic::transport::Endpoint::from_shared(process.url.clone())?.connect().await?;
        let client = dokime::client::HostClient::new(channel, super::support::client_settings())?;
        let status = client.status.get(tonic::Request::new(harmos_protocol::v1::StatusGetRequest {})).await?.into_inner();
        let identity = status.status.as_ref().unwrap().identity.as_ref().unwrap().clone();
        assert!(status.status.as_ref().unwrap().available);
        client.health.list().await?;
        let catalog = client.catalog_entries.list(tonic::Request::new(harmos_protocol::v1::CatalogEntriesListRequest {
            kind: harmos_protocol::v1::CatalogKind::Operations as i32,
            ..Default::default()
        })).await?.into_inner();
        assert!(catalog.operations.iter().any(|operation| operation.contract.as_ref().is_some_and(|contract| contract.id == "harmos.v1.Status/Get")));

        // Add by URL and Read the Admitted Catalog Through the SDK's Generated Clients
        let entry = client.plugins.add(plugin.endpoint()).await?;
        assert_eq!(client.plugins.list().await?.plugins, vec![entry]);
        let suites = client.suites.list().await?;
        let suite_id = suites.suites[0].suite_id.clone();
        let suite = client.suites.get(suite_id.clone()).await?;
        assert_eq!(suite.templates[0].metadata.as_ref().unwrap().identifier, "thermal.cycle");

        // Materialize a Parameter Set and Start Its Frozen Origins
        let cases = client.templates.materialize(suite_id, "thermal.cycle", std::collections::HashMap::from([
            ("limit".into(), vec![v1::Value { kind: Some(v1::value::Kind::Number(85.0)) }])
        ])).await?;
        assert_eq!(cases.len(), 1);
        let isolation_origins = cases.iter().map(|case| case.origin.clone().unwrap()).collect::<Vec<_>>();
        let cycle = client.cycles.start(cases).await?;
        let execution_id = cycle.executions[0].clone();
        let mut progress = cycle.watch();

        // Await the Door Prompt Event and Answer Through the Prompts Service
        let mut phases = Vec::new();
        while let Some(frame) = progress.next().await? {
            phases.push(frame.phase.clone());
            if frame.phase == "prompt" { break; }
        }
        assert!(phases.iter().any(|phase| phase == "prompt"));
        let pending = client.prompts.pending(execution_id.clone()).await?;
        let prompt_id = pending.prompts[0].prompt_id.clone();
        let prompt = client.prompts.get(prompt_id.clone()).await?;
        assert_eq!(prompt.prompt.unwrap().prompt_id, prompt_id);
        client.prompts.respond(prompt_id, "yes", true).await?;

        // Drain Progress Through Its Terminal Response and Successful Final Status
        while let Some(frame) = progress.next().await? { phases.push(frame.phase); }
        assert!(phases.iter().any(|phase| phase == "finished"));

        // Read the Sealed Evidence, Acknowledged Records and Three Controlled Ticks
        let detail = client.executions.get(execution_id.clone()).await?;
        assert_eq!(detail.summary.as_ref().unwrap().state, v1::ExecutionState::Sealed as i32);
        assert!(!detail.seal.is_empty());
        assert!(matches!(detail.outcome.as_ref().unwrap().attribution.as_ref(), Some(v1::execution_outcome::Attribution::Evaluated(outcome)) if outcome.status == v1::ResultStatus::Passed as i32));
        let page = client.executions.list().await?;
        assert_eq!(page.rows[0], *detail.summary.as_ref().unwrap());
        let records = client.records.list(execution_id.clone()).await?;
        assert!(!records.records.is_empty());
        assert!(!records.records[0].payload.is_empty());
        let frozen = detail.schemas.as_ref().unwrap().catalog.as_ref().unwrap();
        let pool = prost_reflect::DescriptorPool::from_file_descriptor_set(frozen.types.clone().unwrap())?;
        let door = prost_reflect::DynamicMessage::decode(pool.get_message_by_name("thermal.v1.Door").unwrap(), records.records[0].payload.as_slice())?;
        assert_eq!(door.get_field_by_name("closed").unwrap().as_ref(), &prost_reflect::Value::Bool(true));
        let samples = client.samples.list(execution_id.clone()).await?;
        assert_eq!(samples.samples.len(), 3);
        assert_eq!(samples.samples.iter().map(|sample| sample.timestamp.unwrap().seconds).collect::<Vec<_>>(), vec![0, 1, 2]);
        assert!(samples.samples.iter().all(|sample| !sample.payload.is_empty()));
        assert!(samples.samples.iter().all(|sample| sample.values.as_ref().unwrap().values["celsius"].kind == Some(v1::value::Kind::Number(84.0))));

        // Download the Report Including Metadata, Ordered Bytes and Digest Completion
        let mut report = client.reports.download(execution_id.clone()).await?;
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

        // Start Two Executions in One Cycle and Read Their Records Separately
        let channel = tonic::transport::Endpoint::from_shared(process.url.clone())?.connect().await?;
        let mut execution = v1::execution_client::ExecutionClient::new(channel.clone());
        let mut prompts = v1::prompts_client::PromptsClient::new(channel.clone());
        let mut evidence = v1::evidence_client::EvidenceClient::new(channel);
        let cycle = execution.create_cycle(v1::ExecutionCreateCycleRequest {
            origins: vec![isolation_origins[0].clone(), isolation_origins[0].clone()],
            continue_automatically: true, ..Default::default()
        }).await?.into_inner();
        assert_eq!(cycle.execution_ids.len(), 2);
        assert_ne!(cycle.execution_ids[0], cycle.execution_ids[1]);
        let mut watches = Vec::new();
        for (identifier, closed) in cycle.execution_ids.iter().zip([true, false]) {
            let mut progress = v1::executions::progress_client::ProgressClient::connect(process.url.clone()).await?.watch(v1::executions::ProgressWatchRequest { name: identifier.clone()
            , at: None }).await?.into_inner();
            let mut prompted = false;
            while let Some(frame) = progress.message().await? {
                if frame.phase == "prompt"
                { prompted = true; break; }
            }
            assert!(prompted);
            let running = execution.get_execution(v1::ExecutionGetExecutionRequest { execution_id: identifier.clone() }).await?.into_inner();
            let controls = running.controls.unwrap();
            assert!(controls.can_cancel && controls.can_inspect);
            assert!(!controls.can_start && !controls.can_rerun && !controls.can_export);
            let pending = prompts.list_pending(v1::PromptsListPendingRequest { execution_id: Some(identifier.clone()), ..Default::default() }).await?.into_inner();
            prompts.respond(v1::PromptsRespondRequest {
                prompt_id: pending.prompts[0].prompt_id.clone(), action: if closed { "yes" } else { "no" }.into(),
                accepted: true, ..Default::default()
            }).await?;
            watches.push(progress);
        }
        for mut progress in watches {
            let mut terminal = false;
            while let Some(frame) = progress.message().await? {
                if frame.finished { terminal = true; }
            }
            assert!(terminal);
        }
        let mut record_ids = Vec::new();
        for (identifier, closed) in cycle.execution_ids.iter().zip([true, false]) {
            let records = evidence.list_records(v1::EvidenceListRecordsRequest {
                selection: Some(v1::CapturedRecordSelection {
                    scope: Some(v1::EvidenceScope { execution_id: Some(identifier.clone()), ..Default::default() }),
                    ..Default::default()
                }), ..Default::default()
            }).await?.into_inner();
            assert_eq!(records.records.len(), 1);
            let record = &records.records[0];
            assert_eq!(record.scope.as_ref().unwrap().execution_id.as_deref(), Some(identifier.as_str()));
            let door = prost_reflect::DynamicMessage::decode(pool.get_message_by_name("thermal.v1.Door").unwrap(), record.payload.as_slice())?;
            assert_eq!(door.get_field_by_name("closed").unwrap().as_ref(), &prost_reflect::Value::Bool(closed));
            record_ids.push(record.record_id.clone());
            let sealed = execution.get_execution(v1::ExecutionGetExecutionRequest { execution_id: identifier.clone() }).await?.into_inner();
            let controls = sealed.controls.unwrap();
            assert!(controls.can_rerun && controls.can_export && controls.can_inspect);
            assert!(!controls.can_start && !controls.can_cancel);
            assert_eq!(sealed.cycle_counts.unwrap().evaluated_count, 2);
        }
        assert_ne!(record_ids[0], record_ids[1]);

        // Restart the Host and Read Exactly the Same Sealed Execution
        drop(progress);
        drop(cycle);
        client.close();
        process.close().await?;
        let restarted = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
        let channel = tonic::transport::Endpoint::from_shared(restarted.url.clone())?.connect().await?;
        let client = dokime::client::HostClient::new(channel, super::support::client_settings())?;
        let restored = client.executions.get(execution_id.clone()).await?;
        assert_eq!(restored, detail);
        let restored_records = client.records.list(execution_id.clone()).await?;
        assert_eq!(restored_records, records);
        let restored_samples = client.samples.list(execution_id).await?;
        assert_eq!(restored_samples, samples);
        let restored = client.status.get(tonic::Request::new(harmos_protocol::v1::StatusGetRequest {})).await?.into_inner();
        assert_eq!(restored.status.unwrap().identity.unwrap().host_id, identity.host_id);
        assert_ne!(restored.resolved_at.unwrap().history.unwrap().epoch, status.resolved_at.unwrap().history.unwrap().epoch);
        client.close();
        restarted.close().await?;
        plugin.close().await?;
        Ok(())
    }).await?
}
