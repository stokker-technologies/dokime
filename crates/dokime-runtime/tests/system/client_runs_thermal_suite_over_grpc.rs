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

#[cfg(target_os = "windows")]
mod windows {
    use std::thread;

    use dokime_runtime::process;

    use super::HostProcess;

    impl Drop for HostProcess {
        fn drop(&mut self) {
            if self.child.try_wait().is_ok_and(|status| status.is_some()) {
                return;
            }
            let Some(identifier) = self.child.id() else { return };
            // Failure and Unwind Must Retire the Managed Children Before the Host.
            let result = process::command("taskkill.exe").args(["/PID", &identifier.to_string(), "/T", "/F"]).output();
            let failure = match result {
                Ok(output) if output.status.success() || self.child.try_wait().is_ok_and(|status| status.is_some()) => return,
                Ok(output) => format!("{}: {}", output.status, String::from_utf8_lossy(&output.stderr)),
                Err(source) => source.to_string(),
            };
            let message = format!("Stop Story Host Process Tree {identifier}: {failure}");
            if thread::panicking() {
                eprintln!("{message}");
            } else {
                panic!("{message}");
            }
        }
    }
}

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::HashMap;
use std::error::Error;
use std::ffi::OsString;
use std::path::Path;
use std::process::Stdio;

// Third Party Imports
use dokime_protocol::host::plugin_placement;
use prost_reflect::DescriptorPool;
use prost_reflect::DynamicMessage;
use prost_types::Timestamp;
use sha2::Digest;
use sha2::Sha256;
use tokio::io::AsyncBufReadExt;
use tokio::io::BufReader;
use tokio::process::Child;
use tokio::process::Command;
use tokio::time;
use tonic::transport::Endpoint;

// Local Imports
use super::support::CredentialStore;
use dokime::EvidenceScope;
use dokime::MapValue;
use dokime::RecordScope;
use dokime::Value;
use dokime::model::Window;
use dokime::record_scope;
use dokime_client_internal::Client;
use dokime_client_internal::EvidenceRecordsListOptions;
use dokime_client_internal::EvidenceSamplesListOptions;
use dokime_client_internal::EvidenceStatisticsOptions;
use dokime_client_internal::ExecutionState;
use dokime_client_internal::ExecutionsProgressWatchOptions;
use dokime_client_internal::FilesReadOptions;
use dokime_client_internal::FrozenSchemaCache;
use dokime_client_internal::ReportState;
use dokime_client_internal::ReportSubject;
use dokime_client_internal::ReportsCreateOptions;
use dokime_client_internal::ResultStatus;
use dokime_client_internal::host::PluginPlacement;
use dokime_client_internal::host::PluginRemote;
use dokime_client_internal::host::PluginValue;
use dokime_protocol::host::PluginState;
use dokime_protocol::host::evidence::sample_row;
use dokime_protocol::host::execution_outcome;
use dokime_protocol::host::report_subject;
use dokime_protocol::model::value;
use dokime_runtime::PluginProcess;
use dokime_runtime::process;

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
        let mut command = Command::from(process::command(super::support::host_fixture()?));
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
    pub(super) async fn close_after_failure(mut self) -> Result<(), Box<dyn Error>> {
        drop(self.child.stdin.take());
        assert!(!self.child.wait().await?.success());
        Ok(())
    }
    pub(super) async fn close(mut self) -> Result<(), Box<dyn Error>> {
        drop(self.child.stdin.take());
        assert!(self.child.wait().await?.success());
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[test]
fn client_runs_thermal_suite_over_grpc() -> Result<(), Box<dyn Error>> {
    super::support::run(Box::pin(async {
        time::timeout(super::support::SYSTEM_HANG_GUARD, async {
            // Start a Real Host Process and the Shared Thermal Plugin
            let root = super::support::tempdir()?;
            let credential_root = super::support::tempdir()?;
            let credentials = CredentialStore::start(credential_root.path()).await?;
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
            assert!(
                catalog
                    .iter()
                    .any(|operation| operation.contract.as_ref().is_some_and(|contract| contract.id == "dokime.host.v1.Status/Get"))
            );
            let entry = client
                .plugins
                .create(
                    "thermal",
                    PluginPlacement {
                        kind: Some(plugin_placement::Kind::Remote(PluginRemote { url: plugin.endpoint().into() })),
                    },
                    PluginValue { enabled: true },
                )
                .await?
                .entry
                .unwrap();
            let entry = super::support::plugin_state(&client.plugins, entry, PluginState::Ready).await?;
            assert_eq!(client.plugins.list().await?.entries, vec![entry]);
            let suites = client.suites.list(String::new()).await?;
            let suite_id = suites.values[0].metadata.as_ref().unwrap().identifier.clone();
            let suite = client.suites.get(suite_id.clone(), String::new(), 0).await?.value.unwrap();
            assert_eq!(suite.templates[0].metadata.as_ref().unwrap().identifier, "thermal.thermal.cycle");
            let cases = super::support::materialize(
                &client,
                suite_id,
                "thermal.thermal.cycle",
                HashMap::from([("limit".into(), vec![Value { kind: Some(value::Kind::Number(85.0)) }])]),
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
            client.prompts.answers.create(prompt_id, "0", MapValue::default(), true).await?;
            let mut terminal = None;
            while let Some(frame) = progress.next().await? {
                terminal = Some(frame);
            }
            let terminal = terminal.expect("Watch Thermal Progress: Return the Final Sealed Frame");
            assert!(terminal.sealed);
            super::support::report_frozen(&client, &execution_id).await?;
            let reply = client.executions.get(execution_id.clone()).await?;
            let detail = reply.execution.unwrap();
            assert_eq!(terminal.outcome, detail.outcome);
            assert_eq!(detail.summary.as_ref().unwrap().state, ExecutionState::Sealed as i32);
            assert!(!detail.seal.is_empty());
            let result_status = match detail.outcome.as_ref().and_then(|outcome| outcome.attribution.as_ref()) {
                Some(execution_outcome::Attribution::Evaluated(outcome)) => ResultStatus::try_from(outcome.status).ok(),
                _ => None,
            };
            if result_status != Some(ResultStatus::Passed) {
                let steps = client.executions.steps(execution_id.clone()).await;
                let scope = EvidenceScope {
                    execution_id: Some(execution_id.clone()),
                    ..Default::default()
                };
                let hold = client
                    .evidence
                    .statistics_with(
                        "thermal.temperature",
                        "celsius",
                        EvidenceStatisticsOptions {
                            scope: Some(scope.clone()),
                            step: Some("thermal.thermal.cycle.hold".into()),
                            ..Default::default()
                        },
                    )
                    .await;
                let execution = client
                    .evidence
                    .statistics_with(
                        "thermal.temperature",
                        "celsius",
                        EvidenceStatisticsOptions { scope: Some(scope), ..Default::default() },
                    )
                    .await;
                let records = client
                    .evidence
                    .records
                    .list_with(
                        "thermal.door",
                        EvidenceRecordsListOptions {
                            scope: Some(RecordScope {
                                kind: Some(record_scope::Kind::Execution(execution_id.clone())),
                            }),
                            step: Some("thermal.thermal.cycle.hold".into()),
                            ..Default::default()
                        },
                    )
                    .await;
                assert_eq!(
                    result_status,
                    Some(ResultStatus::Passed),
                    "Outcome: {:?}; Steps: {steps:?}; Criterion within_limit: Hold {hold:?}; Execution {execution:?}; Door {records:?}",
                    detail.outcome
                );
            }
            let page = client.executions.list(String::new(), Default::default(), Default::default(), Default::default()).await?;
            assert_eq!(page.executions[0].summary.as_ref(), detail.summary.as_ref());
            let records = client
                .evidence
                .records
                .list_with(
                    String::new(),
                    EvidenceRecordsListOptions {
                        scope: Some(RecordScope {
                            kind: Some(record_scope::Kind::Execution(execution_id.clone())),
                        }),
                        versions: Some(Vec::new()),
                        between: Some(Default::default()),
                        step: Some(String::new()),
                        selection: Some(Default::default()),
                        ..Default::default()
                    },
                )
                .await?;
            assert!(!records.records.is_empty());
            let mut schema_cache = FrozenSchemaCache::default();
            let frozen = schema_cache.get(&client, &detail).await?;
            let pool = DescriptorPool::from_file_descriptor_set((*frozen).clone())?;
            let door = DynamicMessage::decode(pool.get_message_by_name("thermal.Door").unwrap(), records.records[0].payload.as_slice())?;
            assert_eq!(door.get_field_by_name("closed").unwrap().as_ref(), &prost_reflect::Value::Bool(true));
            let samples = client
                .evidence
                .samples
                .list_with(
                    EvidenceScope {
                        execution_id: Some(execution_id.clone()),
                        ..Default::default()
                    },
                    vec!["thermal.temperature".into()],
                    EvidenceSamplesListOptions {
                        versions: Some(Vec::new()),
                        fields: Some(Vec::new()),
                        sources: Some(Vec::new()),
                        signals: Some(Vec::new()),
                        between: Some(Default::default()),
                        bucket: Some(Default::default()),
                        latest: Some(false),
                        ..Default::default()
                    },
                )
                .await?;
            let raw = samples
                .samples
                .iter()
                .map(|row| match row.kind.as_ref().unwrap() {
                    sample_row::Kind::Raw(sample) => sample,
                    _ => panic!("Expected Raw Samples"),
                })
                .collect::<Vec<_>>();
            // Select the First Two Scheduled Samples Independently of Later Emissions.
            let selected = client
                .evidence
                .samples
                .list_with(
                    EvidenceScope {
                        execution_id: Some(execution_id.clone()),
                        ..Default::default()
                    },
                    vec!["thermal.temperature".into()],
                    EvidenceSamplesListOptions {
                        between: Some(Window {
                            start: Some(Timestamp { seconds: 1, nanos: 0 }),
                            end: Some(Timestamp { seconds: 2, nanos: 0 }),
                        }),
                        ..Default::default()
                    },
                )
                .await?;
            let times = selected
                .samples
                .iter()
                .map(|row| match row.kind.as_ref().unwrap() {
                    sample_row::Kind::Raw(sample) => sample.timestamp.map(|at| (at.seconds, at.nanos)),
                    _ => panic!("Expected Raw Samples"),
                })
                .collect::<Vec<_>>();
            assert_eq!(times, [Some((1, 0)), Some((2, 0))], "Sample the Hold at 1 s and 2 s: {raw:?}");
            assert!(raw.iter().all(|sample| !sample.payload.is_empty()));
            let celsius = raw.iter().map(|sample| sample.values.as_ref().unwrap().values["celsius"].kind.clone()).collect::<Vec<_>>();
            let expected = super::support::thermal::temperatures(raw.len()).into_iter().map(|value| Some(value::Kind::Number(value)));
            assert_eq!(celsius, expected.collect::<Vec<_>>());
            // Ingestion Settles Before the Seal: the First Statistics Read Counts Every Sample.
            let sealed = EvidenceScope {
                execution_id: Some(execution_id.clone()),
                ..Default::default()
            };
            let statistics = client
                .evidence
                .statistics_with(
                    "thermal.temperature",
                    "celsius",
                    EvidenceStatisticsOptions {
                        scope: Some(sealed),
                        ..Default::default()
                    },
                )
                .await?
                .statistics
                .unwrap();
            assert_eq!(statistics.count, raw.len() as u64);

            // Download Delivers the Report Frozen at Seal, Byte for Byte.
            let reports =
                Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
                    .reports
                    .clone();
            let prepared = reports
                .create_with(
                    ReportSubject {
                        kind: Some(report_subject::Kind::Execution(execution_id.clone())),
                    },
                    String::new(),
                    ReportsCreateOptions {
                        request_id: Some("thermal-preview".into()),
                        ..Default::default()
                    },
                )
                .await?;
            let file = prepared.file.unwrap();
            let selected = client.files.get(file.clone()).await?;
            let mut html = Vec::new();
            while html.len() < file.length as usize {
                let page = client
                    .files
                    .read_with(
                        file.clone(),
                        html.len() as u64,
                        selected.chunk_bytes,
                        FilesReadOptions {
                            at: selected.resolved_at.clone(),
                            ..Default::default()
                        },
                    )
                    .await?;
                assert_eq!(page.resolved_at, selected.resolved_at);
                assert!(!page.data.is_empty());
                html.extend(page.data);
            }
            assert_eq!(html.len() as u64, file.length);
            assert_eq!(Sha256::digest(&html).as_slice(), file.sha256);
            assert_eq!(
                client.executions.get(execution_id.clone()).await?.execution.unwrap().report.unwrap().state,
                ReportState::Ready as i32
            );

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
                client.prompts.answers.create(prompt, if closed { "0" } else { "1" }, MapValue::default(), true).await?;
                watches.push(progress);
            }
            for mut progress in watches {
                let mut terminal = false;
                while let Some(frame) = progress.next().await? {
                    terminal |= frame.sealed;
                }
                assert!(terminal);
            }
            let mut record_ids = Vec::new();
            for (id, closed) in isolated.iter().zip([true, false]) {
                let records = client
                    .evidence
                    .records
                    .list_with(
                        String::new(),
                        EvidenceRecordsListOptions {
                            scope: Some(RecordScope {
                                kind: Some(record_scope::Kind::Execution(id.clone())),
                            }),
                            versions: Some(Vec::new()),
                            between: Some(Default::default()),
                            step: Some(String::new()),
                            selection: Some(Default::default()),
                            ..Default::default()
                        },
                    )
                    .await?;
                assert_eq!(records.records.len(), 1);
                let record = &records.records[0];
                assert_eq!(record.scope.as_ref().unwrap().execution_id.as_deref(), Some(id.as_str()));
                let door = DynamicMessage::decode(pool.get_message_by_name("thermal.Door").unwrap(), record.payload.as_slice())?;
                assert_eq!(door.get_field_by_name("closed").unwrap().as_ref(), &prost_reflect::Value::Bool(closed));
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
                    .list_with(
                        String::new(),
                        EvidenceRecordsListOptions {
                            scope: Some(RecordScope {
                                kind: Some(record_scope::Kind::Execution(execution_id.clone()))
                            }),
                            versions: Some(Vec::new()),
                            between: Some(Default::default()),
                            step: Some(String::new()),
                            selection: Some(Default::default()),
                            ..Default::default()
                        }
                    )
                    .await?
                    .records,
                records.records
            );
            assert_eq!(
                client
                    .evidence
                    .samples
                    .list_with(
                        EvidenceScope {
                            execution_id: Some(execution_id),
                            ..Default::default()
                        },
                        vec!["thermal.temperature".into()],
                        EvidenceSamplesListOptions {
                            versions: Some(Vec::new()),
                            fields: Some(Vec::new()),
                            sources: Some(Vec::new()),
                            signals: Some(Vec::new()),
                            between: Some(Default::default()),
                            bucket: Some(Default::default()),
                            latest: Some(false),
                            ..Default::default()
                        }
                    )
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
        })
        .await?
    }))
}
