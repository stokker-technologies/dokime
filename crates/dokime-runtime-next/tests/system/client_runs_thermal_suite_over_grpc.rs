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
        let client = dokime::client::HostClient::new(channel);
        let identity = client.backend.identity().await?;
        assert_eq!(client.backend.health().await?.readiness, v1::BackendReadiness::Ready as i32);
        client.health.list().await?;
        let availability = client.backend.availability().await?;
        assert!(availability.methods.iter().any(|method| method == "/dokime.v1.Plugins/Add"));

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

        // Refuse Unsupported Operations by Name
        let refused = client.plugins.reload(v1::PluginsReloadRequest::default()).await.unwrap_err();
        assert_eq!(refused.code(), tonic::Code::Unimplemented);
        assert!(refused.message().contains("Plugins.Reload"));

        // Restart the Host and Read Exactly the Same Sealed Execution
        drop(progress);
        drop(cycle);
        client.close();
        process.close().await?;
        let restarted = HostProcess::start(root.path()).await?;
        let channel = tonic::transport::Endpoint::from_shared(restarted.url.clone())?.connect().await?;
        let client = dokime::client::HostClient::new(channel);
        let restored = client.executions.get(execution_id.clone()).await?;
        assert_eq!(restored, detail);
        let restored_records = client.records.list(execution_id.clone()).await?;
        assert_eq!(restored_records, records);
        let restored_samples = client.samples.list(execution_id).await?;
        assert_eq!(restored_samples, samples);
        let restored_identity = client.backend.identity().await?;
        assert_eq!(restored_identity.backend_id, identity.backend_id);
        assert_ne!(restored_identity.epoch, identity.epoch);
        client.close();
        restarted.close().await?;
        plugin.close().await?;
        Ok(())
    }).await?
}
