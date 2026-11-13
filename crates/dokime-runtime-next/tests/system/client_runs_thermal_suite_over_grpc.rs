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
        let status=client.status.get(tonic::Request::new(harmos_protocol::v1::StatusGetRequest {})).await?.into_inner();let identity=status.status.as_ref().unwrap().identity.as_ref().unwrap().clone();assert!(status.status.as_ref().unwrap().available);client.health.list().await?;
        let catalog=client.catalog_entries.list(tonic::Request::new(harmos_protocol::v1::CatalogEntriesListRequest {kind:harmos_protocol::v1::CatalogKind::Operations as i32,..Default::default()})).await?.into_inner();assert!(catalog.operations.iter().any(|operation|operation.contract.as_ref().is_some_and(|contract|contract.id=="harmos.v1.Status/Get")));
        let entry=client.plugins.add(plugin.endpoint()).await?;assert_eq!(client.plugins.list().await?.plugins,vec![entry]);let suites=client.suites.list().await?;let suite_id=suites.values[0].metadata.as_ref().unwrap().identifier.clone();let suite=client.suites.get(suite_id.clone()).await?.value.unwrap();assert_eq!(suite.templates[0].metadata.as_ref().unwrap().identifier,"thermal.cycle");
        let cases=client.templates.materialize(suite_id,"thermal.cycle",std::collections::HashMap::from([("limit".into(),vec![v1::Value {kind:Some(v1::value::Kind::Number(85.0))}])])).await?;assert_eq!(cases.len(),1);let isolation=cases[0].clone();
        let cycle=client.cycles.start(cases).await?;let execution_id=cycle.executions[0].clone();let mut progress=cycle.watch();let mut phases=Vec::new();while let Some(frame)=progress.next().await?{phases.push(frame.phase.clone());if frame.phase=="prompt"{break;}}
        assert!(phases.iter().any(|phase|phase=="prompt"));let pending=client.prompts.pending(execution_id.clone()).await?;let prompt_id=pending.prompts[0].prompt.as_ref().unwrap().prompt_id.clone();assert_eq!(client.prompts.get(prompt_id.clone()).await?.prompt.unwrap().prompt.unwrap().prompt_id,prompt_id);client.prompts.respond(prompt_id,"yes",true).await?;while let Some(frame)=progress.next().await?{phases.push(frame.phase);}assert!(phases.iter().any(|phase|phase=="finished"));
        let detail=client.executions.get(execution_id.clone()).await?.execution.unwrap();assert_eq!(detail.summary.as_ref().unwrap().state,v1::ExecutionState::Sealed as i32);assert!(!detail.seal.is_empty());assert!(matches!(detail.outcome.as_ref().unwrap().attribution.as_ref(),Some(v1::execution_outcome::Attribution::Evaluated(outcome)) if outcome.status==v1::ResultStatus::Passed as i32));
        let page=client.executions.list().await?;assert_eq!(page.executions[0].summary.as_ref(),detail.summary.as_ref());
        let records=client.records.list(execution_id.clone()).await?;assert!(!records.records.is_empty());let frozen=detail.schemas.as_ref().unwrap().catalog.as_ref().unwrap();let pool=prost_reflect::DescriptorPool::from_file_descriptor_set(frozen.types.clone().unwrap())?;
        let door=prost_reflect::DynamicMessage::decode(pool.get_message_by_name("thermal.v1.Door").unwrap(),records.records[0].payload.as_slice())?;assert_eq!(door.get_field_by_name("closed").unwrap().as_ref(),&prost_reflect::Value::Bool(true));
        let samples=client.samples.list(execution_id.clone()).await?;let raw=samples.samples.iter().map(|row|match row.kind.as_ref().unwrap(){v1::sample_row::Kind::Raw(sample)=>sample,_=>panic!("Expected Raw Samples")}).collect::<Vec<_>>();assert_eq!(raw.len(),3);assert_eq!(raw.iter().map(|sample|sample.timestamp.unwrap().seconds).collect::<Vec<_>>(),vec![0,1,2]);assert!(raw.iter().all(|sample|!sample.payload.is_empty()));assert!(raw.iter().all(|sample|sample.values.as_ref().unwrap().values["celsius"].kind==Some(v1::value::Kind::Number(84.0))));

        // Prepared Report Metadata Identifies Its Exact Frozen Bytes Without Publishing.
        let mut reports=v1::reports_client::ReportsClient::connect(process.url.clone()).await?;let prepared=reports.create(v1::ReportsCreateRequest {subject:Some(v1::ReportSubject {kind:Some(v1::report_subject::Kind::Execution(execution_id.clone()))}),request_id:"thermal-preview".into(),..Default::default()}).await?.into_inner();let file=prepared.file.unwrap();let store=dokime_storage::Store::open(root.path())?;let saved:dokime_storage::Report=store.administration(&file.metadata.unwrap().id)?.unwrap();use sha2::Digest;assert_eq!(saved.html.len() as u64,file.length);assert_eq!(sha2::Sha256::digest(&saved.html).as_slice(),file.sha256);assert!(std::str::from_utf8(&saved.html)?.contains(&execution_id));assert!(store.reports(&execution_id)?.is_empty());

        // Concurrent Executions Retain Their Own Questions and Record Identities.
        let isolated=client.cycles.start(vec![isolation.clone(),isolation]).await?;assert_eq!(isolated.executions.len(),2);assert_ne!(isolated.executions[0],isolated.executions[1]);let mut watches=Vec::new();
        for (id,closed) in isolated.executions.iter().zip([true,false]) {
         let mut progress=v1::executions::progress_client::ProgressClient::connect(process.url.clone()).await?.watch(v1::executions::ProgressWatchRequest {name:id.clone(),at:None}).await?.into_inner();while let Some(frame)=progress.message().await?{if frame.phase=="prompt"{break;}}
         let running=client.executions.get(id.clone()).await?.execution.unwrap();assert!(running.controls.unwrap().can_cancel);let pending=client.prompts.pending(id.clone()).await?;let prompt=pending.prompts[0].prompt.as_ref().unwrap().prompt_id.clone();client.prompts.respond(prompt,if closed{"yes"}else{"no"},true).await?;watches.push(progress);
        }
        for mut progress in watches {let mut terminal=false;while let Some(frame)=progress.message().await?{terminal|=frame.finished;}assert!(terminal);}
        let mut record_ids=Vec::new();for (id,closed) in isolated.executions.iter().zip([true,false]) {let records=client.records.list(id.clone()).await?;assert_eq!(records.records.len(),1);let record=&records.records[0];assert_eq!(record.scope.as_ref().unwrap().execution_id.as_deref(),Some(id.as_str()));let door=prost_reflect::DynamicMessage::decode(pool.get_message_by_name("thermal.v1.Door").unwrap(),record.payload.as_slice())?;assert_eq!(door.get_field_by_name("closed").unwrap().as_ref(),&prost_reflect::Value::Bool(closed));record_ids.push(record.record_id.clone());let detail=client.executions.get(id.clone()).await?.execution.unwrap();let controls=detail.controls.unwrap();assert!(controls.can_rerun && controls.can_export && controls.can_inspect);assert!(!controls.can_start && !controls.can_cancel);assert_eq!(detail.cycle_counts.unwrap().evaluated_count,2);}assert_ne!(record_ids[0],record_ids[1]);
        drop(progress);client.close();process.close().await?;
        let restarted=HostProcess::start_with_environment(root.path(),&credentials.environment).await?;let channel=tonic::transport::Endpoint::from_shared(restarted.url.clone())?.connect().await?;let client=dokime::client::HostClient::new(channel,super::support::client_settings())?;
        assert_eq!(client.executions.get(execution_id.clone()).await?.execution.unwrap(),detail);assert_eq!(client.records.list(execution_id.clone()).await?.records,records.records);assert_eq!(client.samples.list(execution_id).await?.samples,samples.samples);let restored=client.status.get(tonic::Request::new(harmos_protocol::v1::StatusGetRequest {})).await?.into_inner();assert_eq!(restored.status.unwrap().identity.unwrap().host_id,identity.host_id);assert_ne!(restored.resolved_at.unwrap().history.unwrap().epoch,status.resolved_at.unwrap().history.unwrap().epoch);client.close();restarted.close().await?;plugin.close().await?;
        Ok(())
    }).await?
}
