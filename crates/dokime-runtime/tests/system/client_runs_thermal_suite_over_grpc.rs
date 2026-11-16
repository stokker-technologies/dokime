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

use dokime::client::EvidenceSamplesListOptions;

use dokime::client::EvidenceRecordsListOptions;

// Standard Library Imports

// Third Party Imports

// Local Imports
use dokime::Client;
use dokime::EvidenceRecordScope;
use dokime::EvidenceScope;
use dokime::MapValue;
use dokime::Value;
use dokime::client::ExecutionState;
use dokime::client::ExecutionsProgressWatchOptions;
use dokime::client::FrozenSchemaCache;
use dokime::client::PluginPlacement;
use dokime::client::PluginRemote;
use dokime::client::PluginValue;
use dokime::client::ReportSubject;
use dokime::client::ReportsCreateOptions;
use dokime::client::ResultStatus;
use dokime::record_scope::Kind as RecordScopeKind;
use dokime_protocol::host::evidence::sample_row::Kind as SampleRowKind;
use dokime_protocol::host::execution_outcome::Attribution as ExecutionOutcomeAttribution;
use dokime_protocol::host::report_subject::Kind as ReportSubjectKind;
use dokime_protocol::model::value::Kind as ValueKind;
use dokime_runtime::PluginProcess;
use dokime_storage::Report;
use dokime_storage::Store;
use harmos_protocol::host::plugin_placement::Kind as PluginPlacementKind;
use prost::Message;
use prost_reflect::DescriptorPool;
use prost_reflect::DynamicMessage;
use prost_reflect::Value as ProstReflectValue;
use std::collections::HashMap;
use std::error::Error;
use std::ffi::OsString;
use std::path::Path;
use std::process::Stdio;
use tokio::io::AsyncBufReadExt;
use tokio::io::BufReader;
use tokio::process::Child;
use tokio::process::Command;
use tonic::transport::Endpoint;

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
    pub(super) async fn start_with_environment(root: &Path, environment: &[(&str, OsString)]) -> Result<Self, Box<dyn Error>> {
        Self::start_with_clock(root, environment, true).await
    }
    pub(super) async fn start_with_clock(root: &Path, environment: &[(&str, OsString)], controlled: bool) -> Result<Self, Box<dyn Error>> {
        let mut command = Command::from(dokime_runtime::process::command(super::support::host_fixture()?));
        command
            .envs(environment.iter().cloned())
            .args(["--credential-namespace", super::support::namespace().as_str(), "--endpoint", "127.0.0.1:0", "--storage"])
            .arg(root);
        if controlled {
            command.arg("--controlled-clock");
        }
        let mut child = command.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::inherit()).kill_on_drop(true).spawn()?;
        let mut lines = BufReader::new(child.stdout.take().unwrap()).lines();
        let url = lines.next_line().await?.ok_or("Host Exited Before Announcing Its Listener")?;
        assert!(url.starts_with("http://127.0.0.1:"));
        Ok(Self { child, url })
    }
    pub(super) async fn kill(mut self) -> Result<(), Box<dyn Error>> {
        self.child.kill().await?;
        assert!(!self.child.wait().await?.success());
        Ok(())
    }
    pub(super) async fn close(mut self) -> Result<(), Box<dyn Error>> {
        drop(self.child.stdin.take());
        assert!(self.child.wait().await?.success());
        Ok(())
    }
}

#[cfg(windows)]
impl Drop for HostProcess {
    fn drop(&mut self) {
        if self.child.try_wait().is_ok_and(|status| status.is_some()) {
            return;
        }
        let Some(identifier) = self.child.id() else { return };
        // Failure and Unwind Must Retire the Managed Children Before the Host.
        let result = dokime_runtime::process::command("taskkill.exe").args(["/PID", &identifier.to_string(), "/T", "/F"]).output();
        let failure = match result {
            Ok(output) if output.status.success() || self.child.try_wait().is_ok_and(|status| status.is_some()) => return,
            Ok(output) => format!("{}: {}", output.status, String::from_utf8_lossy(&output.stderr)),
            Err(source) => source.to_string(),
        };
        let message = format!("Stop Story Host Process Tree {identifier}: {failure}");
        if std::thread::panicking() {
            eprintln!("{message}");
        } else {
            panic!("{message}");
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[test]
fn client_runs_thermal_suite_over_grpc() -> Result<(), Box<dyn Error>> {
    super::support::run(Box::pin(async {
        tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
        // Start a Real Host Process and the Shared Thermal Plugin
        let root = super::support::tempdir()?;
        let credential_root = super::support::tempdir()?;
        let credentials = super::support::CredentialStore::start(credential_root.path()).await?;
        let settings = super::support::settings();
        let executable = super::support::thermal_fixture()?;
        let mut plugin = PluginProcess::start(&executable, true, &settings.placement).await?;
        let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
        let channel = Endpoint::from_shared(process.url.clone())?.connect().await?;
        let client = Client::from_channel(channel, super::support::client_settings())?;
        let status = client.status.get().await?;
        let identity = status.status.as_ref().unwrap().identity.as_ref().unwrap().clone();
        assert!(status.status.as_ref().unwrap().available);
        client.evidence.health_events.list(Default::default()).await?;
        let catalog = client.operations.list().await?;
        assert!(catalog.iter().any(|operation| operation.contract.as_ref().is_some_and(|contract| contract.id == "harmos.host.v1.Status/Get")));
        let entry = client
            .plugins
            .create(
                "thermal",
                PluginPlacement {
                    kind: Some(PluginPlacementKind::Remote(PluginRemote { url: plugin.endpoint().into() })),
                },
                PluginValue { enabled: true },
            )
            .await?
            .entry
            .unwrap();
        let entry = super::support::plugin_state(&client.plugins, entry, harmos_protocol::host::PluginState::Ready).await?;
        assert_eq!(client.plugins.list().await?.entries, vec![entry]);
        let suites = client.suites.list(String::new()).await?;
        let suite_id = suites.values[0].metadata.as_ref().unwrap().identifier.clone();
        let suite = client.suites.get(suite_id.clone(), String::new(), 0).await?.value.unwrap();
        assert_eq!(suite.templates[0].metadata.as_ref().unwrap().identifier, "thermal.cycle");
        let cases = super::support::materialize(
            &client,
            suite_id,
            "thermal.cycle",
            HashMap::from([("limit".into(), vec![Value { kind: Some(ValueKind::Number(85.0)) }])]),
        )
        .await?;
        assert_eq!(cases.len(), 1);
        let isolation = cases[0].clone();
        let cycle = super::support::admit(&client, cases).await?;
        let execution_id = cycle[0].clone();
        let mut progress = client.executions.progress.watch(cycle[0].clone()).await?;
        let mut phases = Vec::new();
        while let Some(frame) = progress.next().await? {
            phases.push(frame.phase.clone());
            if frame.phase == "prompt" {
                break;
            }
        }
        assert!(phases.iter().any(|phase| phase == "prompt"));
        let pending = client.prompts.list(execution_id.clone(), true).await?;
        let prompt_id = pending.prompts[0].prompt.as_ref().unwrap().prompt_id.clone();
        assert_eq!(client.prompts.get(prompt_id.clone()).await?.prompt.unwrap().prompt.unwrap().prompt_id, prompt_id);
        client.prompts.answers.create(prompt_id, "yes", MapValue::default(), true).await?;
        while let Some(frame) = progress.next().await? {
            phases.push(frame.phase);
        }
        assert!(phases.iter().any(|phase| phase == "finished"));
        let reply = client.executions.get(execution_id.clone()).await?;
        eprintln!("Execution Get Reply: {} Bytes", reply.encoded_len());
        let detail = reply.execution.unwrap();
        assert_eq!(detail.summary.as_ref().unwrap().state, ExecutionState::Sealed as i32);
        assert!(!detail.seal.is_empty());
        assert!(
            matches!(detail.outcome.as_ref().unwrap().attribution.as_ref(),Some(ExecutionOutcomeAttribution::Evaluated(outcome)) if outcome.status==ResultStatus::Passed as i32)
        );
        let page = client.executions.list(String::new(), Default::default(), Default::default(), Default::default()).await?;
        assert_eq!(page.executions[0].summary.as_ref(), detail.summary.as_ref());
        let records = client
            .evidence
            .records
            .list_with(EvidenceRecordScope {
                    kind: Some(RecordScopeKind::Execution(execution_id.clone())),
                }, String::new(), EvidenceRecordsListOptions { versions: Some(Vec::new()), between: Some(Default::default()), step: Some(String::new()), selection: Some(Default::default()), ..Default::default() })
            .await?;
        assert!(!records.records.is_empty());
        let mut schema_cache = FrozenSchemaCache::default();
        let frozen = schema_cache.get(&client, &detail).await?;
        let pool = DescriptorPool::from_file_descriptor_set((*frozen).clone())?;
        let door = DynamicMessage::decode(pool.get_message_by_name("thermal.Door").unwrap(), records.records[0].payload.as_slice())?;
        assert_eq!(door.get_field_by_name("closed").unwrap().as_ref(), &ProstReflectValue::Bool(true));
        let samples = client
            .evidence
            .samples
            .list_with(EvidenceScope {
                    execution_id: Some(execution_id.clone()),
                    ..Default::default()
                }, vec!["thermal.temperature".into()], EvidenceSamplesListOptions { versions: Some(Vec::new()), fields: Some(Vec::new()), sources: Some(Vec::new()), signals: Some(Vec::new()), between: Some(Default::default()), bucket: Some(Default::default()), latest: Some(false), ..Default::default() })
            .await?;
        let raw = samples
            .samples
            .iter()
            .map(|row| match row.kind.as_ref().unwrap() {
                SampleRowKind::Raw(sample) => sample,
                _ => panic!("Expected Raw Samples"),
            })
            .collect::<Vec<_>>();
        assert_eq!(raw.len(), 2);
        assert_eq!(raw.iter().map(|sample| sample.timestamp.unwrap().seconds).collect::<Vec<_>>(), vec![1, 2]);
        assert!(raw.iter().all(|sample| !sample.payload.is_empty()));
        assert!(raw.iter().all(|sample| sample.values.as_ref().unwrap().values["celsius"].kind == Some(ValueKind::Number(84.0))));
        // Ingestion Settles Before the Seal: the First Statistics Read Counts Every Sample.
        let sealed = EvidenceScope {
            execution_id: Some(execution_id.clone()),
            ..Default::default()
        };
        let statistics = client.evidence.statistics(sealed, "thermal.temperature", "celsius").await?.statistics.unwrap();
        assert_eq!(statistics.count, raw.len() as u64);

        // Prepared Report Metadata Identifies Its Exact Frozen Bytes Without Publishing.
        let reports = Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
            .reports
            .clone();
        let prepared = reports
            .create_with(
                ReportSubject {
                    kind: Some(ReportSubjectKind::Execution(execution_id.clone())),
                },
                String::new(),
                ReportsCreateOptions {
                    request_id: Some("thermal-preview".into()),
                    ..Default::default()
                },
            )
            .await?;
        let file = prepared.file.unwrap();
        let store = Store::open(root.path())?;
        let saved: Report = store.administration(&file.metadata.unwrap().id)?.unwrap();
        use sha2::Digest;
        assert_eq!(saved.html.len() as u64, file.length);
        assert_eq!(sha2::Sha256::digest(&saved.html).as_slice(), file.sha256);
        assert!(std::str::from_utf8(&saved.html)?.contains(&execution_id));
        assert!(store.reports(&execution_id)?.is_empty());

        // Concurrent Executions Retain Their Own Questions and Record Identities.
        let isolated = super::support::admit(&client, vec![isolation.clone(), isolation]).await?;
        assert_eq!(isolated.len(), 2);
        assert_ne!(isolated[0], isolated[1]);
        let mut watches = Vec::new();
        for (id, closed) in isolated.iter().zip([true, false]) {
            let mut progress =
                Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
                    .executions
                    .progress
                    .clone()
                    .watch_with(id.clone(), ExecutionsProgressWatchOptions { at: None, ..Default::default() })
                    .await?;
            while let Some(frame) = progress.next().await? {
                if frame.phase == "prompt" {
                    break;
                }
            }
            let running = client.executions.get(id.clone()).await?.execution.unwrap();
            assert!(running.controls.unwrap().can_cancel);
            let pending = client.prompts.list(id.clone(), true).await?;
            let prompt = pending.prompts[0].prompt.as_ref().unwrap().prompt_id.clone();
            client.prompts.answers.create(prompt, if closed { "yes" } else { "no" }, MapValue::default(), true).await?;
            watches.push(progress);
        }
        for mut progress in watches {
            let mut terminal = false;
            while let Some(frame) = progress.next().await? {
                terminal |= frame.finished;
            }
            assert!(terminal);
        }
        let mut record_ids = Vec::new();
        for (id, closed) in isolated.iter().zip([true, false]) {
            let records = client
                .evidence
                .records
                .list_with(EvidenceRecordScope {
                        kind: Some(RecordScopeKind::Execution(id.clone())),
                    }, String::new(), EvidenceRecordsListOptions { versions: Some(Vec::new()), between: Some(Default::default()), step: Some(String::new()), selection: Some(Default::default()), ..Default::default() })
                .await?;
            assert_eq!(records.records.len(), 1);
            let record = &records.records[0];
            assert_eq!(record.scope.as_ref().unwrap().execution_id.as_deref(), Some(id.as_str()));
            let door = DynamicMessage::decode(pool.get_message_by_name("thermal.Door").unwrap(), record.payload.as_slice())?;
            assert_eq!(door.get_field_by_name("closed").unwrap().as_ref(), &ProstReflectValue::Bool(closed));
            record_ids.push(record.record_id.clone());
            let detail = client.executions.get(id.clone()).await?.execution.unwrap();
            let controls = detail.controls.unwrap();
            assert!(controls.can_rerun && controls.can_export && controls.can_inspect);
            assert!(!controls.can_start && !controls.can_cancel);
            assert_eq!(detail.cycle_counts.unwrap().evaluated_count, 2);
        }
        assert_ne!(record_ids[0], record_ids[1]);
        drop(progress);
        process.close().await?;
        let restarted = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
        let channel = Endpoint::from_shared(restarted.url.clone())?.connect().await?;
        let client = Client::from_channel(channel, super::support::client_settings())?;
        assert_eq!(client.executions.get(execution_id.clone()).await?.execution.unwrap(), detail);
        assert_eq!(
            client
                .evidence
                .records
                .list_with(EvidenceRecordScope {
                        kind: Some(RecordScopeKind::Execution(execution_id.clone()))
                    }, String::new(), EvidenceRecordsListOptions { versions: Some(Vec::new()), between: Some(Default::default()), step: Some(String::new()), selection: Some(Default::default()), ..Default::default() })
                .await?
                .records,
            records.records
        );
        assert_eq!(
            client
                .evidence
                .samples
                .list_with(EvidenceScope {
                        execution_id: Some(execution_id),
                        ..Default::default()
                    }, vec!["thermal.temperature".into()], EvidenceSamplesListOptions { versions: Some(Vec::new()), fields: Some(Vec::new()), sources: Some(Vec::new()), signals: Some(Vec::new()), between: Some(Default::default()), bucket: Some(Default::default()), latest: Some(false), ..Default::default() })
                .await?
                .samples,
            samples.samples
        );
        let restored = client.status.get().await?;
        assert_eq!(restored.status.unwrap().identity.unwrap().host_id, identity.host_id);
        assert_ne!(restored.resolved_at.unwrap().history.unwrap().epoch, status.resolved_at.unwrap().history.unwrap().epoch);
        restarted.close().await?;
        plugin.close().await?;
        Ok(())
    }).await?
    }))
}
