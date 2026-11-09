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
use std::sync::{Arc, Mutex};

// Third Party Imports
use async_trait::async_trait;
use harmos::sidecar::StreamDeclaration;

// Local Imports
use crate::service_support::prelude::*;
use dokime_runtime::domains::installed_plugins::{PluginRestart, PluginSettingsUpdate};
use dokime_runtime::execution::log::{Entry, Level};
use dokime_runtime::harmos::plugin::PluginGeneration;
use dokime_runtime::plugins::{PluginEntry, PluginLastBoot, PluginState};
use dokime_runtime::runtime::EmbeddedProvider;
use dokime_runtime::transport::{
    EvidenceBatch, ExecutionSignal, RunExecution, SidecarCriterionOutcome, SidecarCriterionStatus, SidecarExecutionOutcome, SignalReply,
    TransportFuture,
};
use dokime_runtime::{InstalledPlugin, Refusal, Value};
use {
    dokime_domain::model::execution::result::StepAttemptStart, dokime_domain::model::execution::result::TestStepAttempt,
    dokime_domain::model::execution::termination::TerminationReason, dokime_runtime::execution::Caller,
    dokime_runtime::execution::ExecutionHost, dokime_runtime::execution::Runnable, dokime_runtime::execution::SampleOutput,
};

/* === Constants === */

const SOURCE: &str = "fixture_sessions";
const TEMPLATE: &str = "fixture.sessions.cancel";
const PHASE: &str = "fixture.sessions.stream";
const HOST: &str = "plugin_host";
const SCAN_INTERVAL: &str = "minimum_scan_interval_s";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct Probe {
    mode: Settlement,
    entered: tokio::sync::Notify,
    row_received: tokio::sync::Notify,
    cancelled: tokio::sync::Notify,
    released: tokio::sync::watch::Sender<bool>,
    cancellation: tokio::sync::watch::Sender<bool>,
    token: Mutex<String>,
    // Like A Sidecar Receiver: Rows In Arrival Order, And Whether Settlement Began.
    arrivals: Mutex<std::collections::BTreeMap<String, Vec<(String, String)>>>,
    settling: std::sync::atomic::AtomicBool,
    settlement_signals: std::sync::atomic::AtomicUsize,
    record_ordinal: std::sync::atomic::AtomicU64,
    // The Live Replay Story Reads Host Rows Through Its Runtime And Clears It Before Shutdown.
    runtime: Mutex<Option<RuntimeHandle>>,
    hosts: Mutex<std::collections::BTreeMap<String, Arc<dyn ExecutionHost>>>,
}

struct Provider(Arc<Probe>);

/* === Enums === */

#[derive(Clone, Copy, PartialEq)]
enum Settlement {
    Complete,
    Deferred,
    Refused,
    RefusedPage,
    AbandonedAdmission,
    LiveReplay,
    // Each Run Stays Active Until Released, Then Passes; The Next Run Waits For Its Own Release.
    Held,
    HeldStreams,
    Ignored,
    IgnoredSignal,
    Failure,
}

/* === Traits === */

/* === Implementations === */

impl Probe {
    fn new(mode: Settlement) -> Self {
        Self {
            mode,
            entered: tokio::sync::Notify::new(),
            row_received: tokio::sync::Notify::new(),
            cancelled: tokio::sync::Notify::new(),
            released: tokio::sync::watch::Sender::new(false),
            cancellation: tokio::sync::watch::Sender::new(false),
            token: Mutex::new(String::new()),
            arrivals: Mutex::new(std::collections::BTreeMap::new()),
            settling: std::sync::atomic::AtomicBool::new(false),
            settlement_signals: std::sync::atomic::AtomicUsize::new(0),
            record_ordinal: std::sync::atomic::AtomicU64::new(0),
            runtime: Mutex::new(None),
            hosts: Mutex::new(std::collections::BTreeMap::new()),
        }
    }

    fn receive(&self, token: &str, encoded: &[String]) {
        let mut executions = self.arrivals.lock().unwrap();
        let arrivals = executions.entry(token.to_owned()).or_default();
        for sample in encoded {
            let sample: dokime_runtime::telemetry::TelemetrySample = serde_json::from_str(sample).unwrap();
            let (stream, id) = (sample.stream.to_string(), sample.sample_id.to_string());
            if arrivals.iter().any(|(known, sample)| known == &stream && sample == &id) {
                continue;
            }
            arrivals.push((stream, id));
        }
        if !arrivals.is_empty() {
            self.row_received.notify_one();
        }
    }

    // The Receipt Names The Row That Arrived Last On Each Stream, As A Sidecar Receiver Does.
    fn receipts(&self, token: &str) -> Vec<dokime_runtime::transport::protocol::SampleReceipt> {
        let executions = self.arrivals.lock().unwrap();
        let Some(arrivals) = executions.get(token) else { return Vec::new() };
        let mut receipts: Vec<dokime_runtime::transport::protocol::SampleReceipt> = Vec::new();
        for (stream, sample_id) in arrivals.iter() {
            receipts.retain(|receipt| &receipt.stream != stream);
            receipts.push(dokime_runtime::transport::protocol::SampleReceipt {
                stream: stream.clone(),
                sample_id: sample_id.clone(),
            });
        }
        receipts
    }
}

impl Runnable for Probe {
    fn template_identifier(&self) -> &str {
        TEMPLATE
    }

    fn run(
        &self, request: RunExecution, host: Arc<dyn ExecutionHost>, _: Option<SampleOutput>, _: Caller,
    ) -> TransportFuture<'_, Result<SidecarExecutionOutcome, Refusal>> {
        Box::pin(async move {
            if self.mode == Settlement::Ignored {
                self.hosts.lock().unwrap().insert(request.execution_token.clone(), host.clone());
            }
            *self.token.lock().unwrap() = request.execution_token;
            let start = StepAttemptStart {
                attempt_id: dokime_runtime::model::identity::new_id::<dokime_runtime::StepAttemptId>(),
                step_id: dokime_runtime::StepId::from_seed(PHASE),
                started_at: dokime_runtime::model::time::now_utc(),
            };
            host.push(EvidenceBatch {
                cursor: 1,
                phase: PHASE.into(),
                step_begins: vec![serde_json::to_string(&start).unwrap()],
                ..Default::default()
            })
            .await
            .map_err(Refusal::of)?;
            self.entered.notify_one();
            if matches!(self.mode, Settlement::LiveReplay | Settlement::Held | Settlement::HeldStreams) {
                if matches!(self.mode, Settlement::Held | Settlement::HeldStreams) {
                    self.released.subscribe().wait_for(|released| *released).await.unwrap();
                    self.released.send_replace(false);
                } else {
                    self.row_received.notified().await;
                }
                let ended = TestStepAttempt {
                    attempt_id: start.attempt_id,
                    step_id: start.step_id,
                    started_at: start.started_at,
                    finished_at: Some(dokime_runtime::model::time::now_utc()),
                    termination: TerminationReason::Completed,
                    detail: None,
                };
                let finished = EvidenceBatch {
                    cursor: 2,
                    step_attempts: vec![serde_json::to_string(&ended).unwrap()],
                    ..Default::default()
                };
                host.push(finished).await.map_err(Refusal::of)?;
                self.settling.store(true, std::sync::atomic::Ordering::SeqCst);
                let settling = EvidenceBatch {
                    cursor: 3,
                    record_subscriptions_settling: true,
                    ..Default::default()
                };
                let receipt = host.push(settling).await.map_err(Refusal::of)?;
                assert!(receipt.settled, "the settling page settles the run's inputs");
                let passed = SidecarExecutionOutcome {
                    passed: true,
                    criteria: vec![SidecarCriterionOutcome {
                        identifier: "fixture.criterion".into(),
                        label: "Fixture Criterion".into(),
                        status: SidecarCriterionStatus::Passed as i32,
                        message: None,
                    }],
                    ..Default::default()
                };
                let concluded = EvidenceBatch {
                    cursor: 4,
                    cleanup_settled: true,
                    outcome: Some(passed.clone()),
                    ..Default::default()
                };
                host.push(concluded).await.map_err(Refusal::of)?;
                return Ok(passed);
            }
            if self.mode == Settlement::Failure {
                self.released.subscribe().wait_for(|released| *released).await.unwrap();
                return Err(Refusal::of("fixture run failure"));
            }
            self.cancellation.subscribe().wait_for(|cancelled| *cancelled).await.unwrap();
            if matches!(self.mode, Settlement::Deferred | Settlement::Ignored | Settlement::IgnoredSignal) {
                self.released.subscribe().wait_for(|released| *released).await.unwrap();
            }
            if matches!(self.mode, Settlement::Ignored | Settlement::IgnoredSignal) {
                return Err(Refusal::of("run released after the host settled it"));
            }
            let ended = TestStepAttempt {
                attempt_id: start.attempt_id,
                step_id: start.step_id,
                started_at: start.started_at,
                finished_at: Some(dokime_runtime::model::time::now_utc()),
                termination: TerminationReason::Cancelled,
                detail: None,
            };
            if self.mode == Settlement::AbandonedAdmission {
                // Like A Sidecar Run Cancelled During Step Admission, Cleanup Begins Without Closing The Attempt.
                let cleanup = dokime_domain::model::execution::CapturePhase::Cleanup;
                host.push(EvidenceBatch {
                    cursor: 2,
                    capture_phases: vec![serde_json::to_string(&cleanup).unwrap()],
                    ..Default::default()
                })
                .await
                .map_err(Refusal::of)?;
                host.push(EvidenceBatch {
                    cursor: 3,
                    diagnostics: cancellation_log(),
                    cleanup_settled: true,
                    ..Default::default()
                })
                .await
                .map_err(Refusal::of)?;
                return Ok(SidecarExecutionOutcome::default());
            }
            if self.mode == Settlement::RefusedPage {
                // Like A Sidecar Run, The Settling Page Waits For Its Input Receipt Before Concluding.
                let receipt = host
                    .push(EvidenceBatch {
                        cursor: 2,
                        step_attempts: vec!["not a step attempt".into()],
                        record_subscriptions_settling: true,
                        ..Default::default()
                    })
                    .await
                    .map_err(Refusal::of)?;
                if !receipt.settled {
                    std::future::pending::<()>().await;
                }
                host.push(EvidenceBatch {
                    cursor: 3,
                    cleanup_settled: true,
                    outcome: Some(SidecarExecutionOutcome::default()),
                    ..Default::default()
                })
                .await
                .map_err(Refusal::of)?;
                return Ok(SidecarExecutionOutcome::default());
            }
            let refused = self.mode == Settlement::Refused;
            host.push(EvidenceBatch {
                cursor: 2,
                step_attempts: vec![serde_json::to_string(&ended).unwrap()],
                diagnostics: if refused { Vec::new() } else { cancellation_log() },
                cleanup_settled: !refused,
                ..Default::default()
            })
            .await
            .map_err(Refusal::of)?;
            if refused {
                return Err(Refusal::of("run refused during cancellation"));
            }
            Ok(SidecarExecutionOutcome {
                passed: false,
                message: Some("cancelled".into()),
                criteria: vec![SidecarCriterionOutcome {
                    identifier: "fixture.criterion".into(),
                    label: "Fixture Criterion".into(),
                    status: SidecarCriterionStatus::Indeterminate as i32,
                    message: Some("cancelled before deciding".into()),
                }],
                detail_json: None,
                failed_in: None,
            })
        })
    }

    fn signal(&self, signal: ExecutionSignal) -> TransportFuture<'_, Result<SignalReply, Refusal>> {
        Box::pin(async move {
            if let Some(record) = &signal.record {
                self.record_ordinal.fetch_max(record.ordinal, std::sync::atomic::Ordering::SeqCst);
            }
            let record_ordinal = self.record_ordinal.load(std::sync::atomic::Ordering::SeqCst);
            if let Some(delivery) = &signal.samples {
                self.receive(&signal.execution_token, &delivery.samples);
            }
            if self.mode == Settlement::LiveReplay {
                let settling = self.settling.load(std::sync::atomic::Ordering::SeqCst);
                match &signal.samples {
                    // The Second Settlement Signal Asks For The Receipt After The Replay Cut; Collection Keeps Ticking.
                    None if settling && self.settlement_signals.fetch_add(1, std::sync::atomic::Ordering::SeqCst) == 1 => {
                        let runtime = self.runtime.lock().unwrap().clone().expect("the live replay story binds its runtime");
                        let from = host_rows(&runtime).await;
                        until_telemetry(&runtime, || async { (host_rows(&runtime).await >= from + 3).then_some(()) }).await;
                    }
                    _ => {}
                }
                return Ok(SignalReply {
                    samples: self.receipts(&signal.execution_token),
                    record_ordinal,
                    ..Default::default()
                });
            }
            if signal.cancel {
                if *self.token.lock().unwrap() != signal.execution_token {
                    return Err(Refusal::of("cancellation token does not own execution"));
                }
                self.cancellation.send_replace(true);
                self.cancelled.notify_one();
                if self.mode == Settlement::IgnoredSignal {
                    self.released.subscribe().wait_for(|released| *released).await.unwrap();
                }
            }
            Ok(SignalReply {
                samples: self.receipts(&signal.execution_token),
                record_ordinal,
                ..Default::default()
            })
        })
    }

    fn take_local_samples(&self, _: &str) -> Result<Vec<dokime_runtime::telemetry::TelemetrySample>, Refusal> {
        Ok(Vec::new())
    }
    fn interrupt(&self) {
        self.cancellation.send_replace(true);
        self.released.send_replace(true);
    }
    fn shutdown(&self) -> TransportFuture<'_, ()> {
        self.interrupt();
        Box::pin(async {})
    }
}

#[async_trait]
impl EmbeddedProvider for Provider {
    fn catalog(&self) -> InstalledPlugin {
        let mut suite = fixture_suite(&["cancelled"]);
        suite.metadata = metadata(SOURCE, "Session Fixture");
        let template = &mut suite.test_case_templates[0];
        template.metadata = metadata(TEMPLATE, "Cancellation");
        template.phases[0].metadata = metadata(PHASE, "Stream");
        template.phases[0].step_id = dokime_runtime::StepId::from_seed(PHASE);
        // The failing provider samples host telemetry, so its run holds a stream
        // lease the failure cleanup has to close.
        if matches!(self.0.mode, Settlement::Failure | Settlement::LiveReplay | Settlement::HeldStreams) {
            template.subscriptions = vec![dokime_runtime::definition::Subscription::Stream(identifier("host.system"))];
        }
        let mut definition = TestSuiteDefinition::new(SOURCE, suite).unwrap();
        definition.workflow = Some(SOURCE.into());
        InstalledPlugin {
            source: SOURCE.into(),
            version: "0.1.0".into(),
            enabled: true,
            suites: dokime_runtime::definition::SuiteSet::new([definition]),
            ..Default::default()
        }
    }
    fn streams(&self) -> Vec<StreamDeclaration> {
        Vec::new()
    }
    async fn start(&self) -> dokime_runtime::runtime::Result<Vec<Arc<dyn Runnable>>> {
        Ok(vec![self.0.clone()])
    }
    async fn stop(&self) -> dokime_runtime::runtime::Result<()> {
        self.0.interrupt();
        Ok(())
    }
}

/* === Public Functions === */

// The Host Preserves Cancellation Verdicts And All Diagnostic Levels From Its Provider.
pub(crate) async fn cancellation_retains_verdict() {
    for mode in [Settlement::Complete, Settlement::Deferred, Settlement::Refused] {
        let root = tempdir().unwrap();
        let probe = Arc::new(Probe::new(mode));
        let provider = Provider(probe.clone());
        let settings = config(root.path());

        let (mut owner, handle) = started(settings.register(Arc::new(provider))).await;
        let suite = handle.test_suites().await.into_iter().find(|suite| suite.source == SOURCE).unwrap().suite;
        let cycle_id = start_cases(&handle, metadata("session.cancel", "Cancellation"), materialize_cases(SOURCE, &suite)).await;
        let receipt = handle.start_next_execution(cycle_id.clone(), CommandContext::default()).await;
        assert_eq!(receipt.status, CommandStatus::Accepted);
        probe.entered.notified().await;
        let prepared = handle.run_request(&receipt.command_id).await.unwrap().unwrap();
        assert_eq!(prepared.summary.prepared, 1, "provider dispatch follows the saved attempt preparation");
        let stopped = {
            let handle = handle.clone();
            let cycle_id = cycle_id.clone();
            tokio::spawn(async move { handle.seal_cycle(cycle_id, CommandContext::default()).await.unwrap() })
        };
        probe.cancelled.notified().await;
        if mode == Settlement::Deferred {
            assert_ne!(
                cycle_snapshot(&handle, &cycle_id).await.lifecycle,
                CycleStatus::Stopped,
                "the cycle waits for the provider's outcome"
            );
            probe.released.send_replace(true);
        }
        stopped.await.unwrap();
        let cycle = wait_for_cycle_lifecycle(&handle, &cycle_id, CycleStatus::Stopped).await;
        assert_eq!(cycle.test_executions.len(), 1);
        let execution = &cycle.test_executions[0];
        assert_eq!(execution.status, ExecutionStatus::Cancelled);
        if mode != Settlement::Refused {
            let criteria = execution.steps.iter().flat_map(|step| &step.criteria).collect::<Vec<_>>();
            assert_eq!(criteria.len(), 1);
            assert_eq!(criteria[0].status, dokime_domain::model::execution::result::CriterionStatus::Indeterminate);
            assert_eq!(criteria[0].message.as_deref(), Some("cancelled before deciding"));
        }
        let files = handle.execution_files(&execution.test_execution_id).await.unwrap();
        let log = files.iter().find(|file| file.relative_path == "artifacts/execution.log").unwrap();
        let content =
            handle.execution_file_content(&execution.test_execution_id, log.relative_path.clone(), 0, 4096).await.unwrap().unwrap();
        let log_path = log.relative_path.clone();
        let checkpoint = handle.execution_saved_checkpoint(execution.test_execution_id.clone()).await.unwrap();
        let log = without_timestamps(std::str::from_utf8(&content.bytes).unwrap());
        let mut diagnostics = Vec::new();
        let mut attempts = Vec::new();
        let mut latencies = Vec::new();
        for line in log.lines() {
            let event = line.split_once("] ").and_then(|(_, message)| serde_json::from_str::<serde_json::Value>(message).ok());
            if let Some(event) = event.filter(|event| event["kind"] == "dokime.operations.latency" || event["kind"] == "operation_attempt")
            {
                if event["kind"] == "operation_attempt" {
                    attempts.push(event);
                } else {
                    latencies.push(event);
                }
            } else {
                diagnostics.push(line);
            }
        }
        let [attempt] = attempts.as_slice() else {
            panic!("one provider invocation retains one operation attempt: {attempts:?}");
        };
        assert_eq!(attempt["operation"], "dokime.template.run");
        assert_eq!(attempt["outcome"], if mode == Settlement::Refused { "failed" } else { "succeeded" });
        assert!(attempt["instance"].as_u64().is_some());
        assert!(attempt["generation"].as_u64().is_some());
        assert!(!attempt["started_at"].is_null());
        assert!(attempt["elapsed_ms"].is_number());
        let latency = latencies.iter().find(|record| record["stage"] == "execution").unwrap();
        assert_eq!(latency["attempts"], 1);
        assert_eq!(latency["slowest_instance"], attempt["instance"]);
        assert_eq!(latency["max_elapsed_ms"], attempt["elapsed_ms"]);
        if mode == Settlement::Refused {
            assert!(diagnostics[0].starts_with("[ERROR] "), "{log}");
            assert!(diagnostics[0].contains("run refused during cancellation"), "{log}");
        } else {
            let mut lines = diagnostics.into_iter();
            for entry in cancellation_log() {
                let level = Level::try_from(entry.level)
                    .map(|level| format!("{level:?}").to_uppercase())
                    .unwrap_or_else(|_| format!("UNKNOWN({})", entry.level));
                for message in entry.message.split('\n') {
                    let expected = format!("[{level}] {message}");
                    assert_eq!(lines.next(), Some(expected.as_str()));
                }
            }
            assert!(lines.next().is_none());
        }
        drop(handle);
        owner.shutdown().await.unwrap();
        drop(owner);

        let probe = Arc::new(Probe::new(mode));
        let provider = Provider(probe);
        let configuration = config(root.path()).register(Arc::new(provider));
        let (mut restarted, handle) = started(configuration).await;
        let saved = crate::service_support::native::open(&handle, &checkpoint).await;
        assert_eq!(saved.execution().status, ExecutionStatus::Cancelled);
        let restored = handle.run_request(&receipt.command_id).await.unwrap().unwrap();
        assert!(restored.summary.phase.terminal());
        assert_eq!(restored.summary.prepared, 1);
        let content = handle.execution_file_content(&execution.test_execution_id, log_path, 0, 4096).await.unwrap().unwrap();
        let restored_log = without_timestamps(std::str::from_utf8(&content.bytes).unwrap());
        assert_eq!(restored_log, log, "cancellation evidence survives a fresh host session");
        drop(saved);
        drop(handle);
        restarted.shutdown().await.unwrap();
    }
}

// Thirty Seconds Of Host Settlement Policy Elapse On The Controlled Clock.
pub(crate) async fn ignored_run_identity() {
    ignored_stop(Settlement::Ignored).await;
}

pub(crate) async fn ignored_signal_identity() {
    ignored_stop(Settlement::IgnoredSignal).await;
}

// A Refused Settling Page Still Closes The Run's Inputs, So It Winds Down Instead Of Waiting Out The Stop Budget.
pub(crate) async fn refused_settlement_winds_down() {
    let root = tempdir().unwrap();
    let probe = Arc::new(Probe::new(Settlement::RefusedPage));
    let (mut owner, handle) = started(config(root.path()).register(Arc::new(Provider(probe.clone())))).await;
    let suite = handle.test_suites().await.into_iter().find(|suite| suite.source == SOURCE).unwrap().suite;
    let cycle_id = start_cases(&handle, metadata("session.refused", "Refused Settlement"), materialize_cases(SOURCE, &suite)).await;
    let receipt = handle.start_next_execution(cycle_id.clone(), CommandContext::default()).await;
    assert_eq!(receipt.status, CommandStatus::Accepted);
    probe.entered.notified().await;
    handle.seal_cycle(cycle_id.clone(), CommandContext::default()).await.unwrap();
    wait_for_run_settled(&handle, &receipt.command_id).await;
    let cycle = wait_for_cycle_lifecycle(&handle, &cycle_id, CycleStatus::Stopped).await;
    let execution = &cycle.test_executions[0];
    let files = handle.execution_files(&execution.test_execution_id).await.unwrap();
    let log = files.iter().find(|file| file.relative_path == "artifacts/execution.log").unwrap();
    let log = handle.execution_file_content(&execution.test_execution_id, log.relative_path.clone(), 0, 4096).await.unwrap().unwrap();
    let log = std::str::from_utf8(&log.bytes).unwrap().to_owned();
    assert!(log.contains("invalid step attempt"), "{log}");
    assert!(!log.contains("did not answer its stop"), "the run settled itself rather than the host settling it: {log}");
    drop(handle);
    owner.shutdown().await.unwrap();
}

// A Stop That Lands While The Next Step Admission Is In Flight Closes The Attempt The Run Abandoned.
pub(crate) async fn abandoned_admission_closes() {
    let root = tempdir().unwrap();
    let probe = Arc::new(Probe::new(Settlement::AbandonedAdmission));
    let (mut owner, handle) = started(config(root.path()).register(Arc::new(Provider(probe.clone())))).await;
    let suite = handle.test_suites().await.into_iter().find(|suite| suite.source == SOURCE).unwrap().suite;
    let cycle_id = start_cases(&handle, metadata("session.abandoned", "Abandoned Admission"), materialize_cases(SOURCE, &suite)).await;
    let receipt = handle.start_next_execution(cycle_id.clone(), CommandContext::default()).await;
    assert_eq!(receipt.status, CommandStatus::Accepted);
    probe.entered.notified().await;
    handle.seal_cycle(cycle_id.clone(), CommandContext::default()).await.unwrap();
    wait_for_run_settled(&handle, &receipt.command_id).await;
    let cycle = wait_for_cycle_lifecycle(&handle, &cycle_id, CycleStatus::Stopped).await;
    let execution = &cycle.test_executions[0];
    assert_eq!(execution.status, ExecutionStatus::Cancelled);
    let files = handle.execution_files(&execution.test_execution_id).await.unwrap();
    let log = files.iter().find(|file| file.relative_path == "artifacts/execution.log").unwrap();
    let log = handle.execution_file_content(&execution.test_execution_id, log.relative_path.clone(), 0, 4096).await.unwrap().unwrap();
    let log = std::str::from_utf8(&log.bytes).unwrap().to_owned();
    assert!(log.contains("fixture cancellation settled"), "the run's cleanup page is admitted: {log}");
    assert!(!log.contains("did not answer its stop"), "the run settled itself rather than the host settling it: {log}");
    drop(handle);
    owner.shutdown().await.unwrap();
}

// Live Rows Keep Arriving While Settlement Replays: The Fence Keeps The Final Receipt Exact.
pub(crate) async fn live_replay_settles() {
    let root = tempdir().unwrap();
    let probe = Arc::new(Probe::new(Settlement::LiveReplay));
    let configuration =
        config(root.path()).with_plugins([crate::service_support::bundles::host_plugin()]).register(Arc::new(Provider(probe.clone())));
    let (mut owner, handle) = started(configuration).await;
    *probe.runtime.lock().unwrap() = Some(handle.clone());
    // A Viewer Keeps Host Collection Ticking After The Run's Own Measurement Closes.
    let viewer = dokime_runtime::telemetry::SamplingScope {
        test_cycle_id: None,
        test_case_id: None,
        test_execution_id: None,
    };
    let viewing = handle.lease_streams(viewer, &[identifier("host.system")], "viewer").await.unwrap();
    let suite = handle.test_suites().await.into_iter().find(|suite| suite.source == SOURCE).unwrap().suite;
    let cycle_id = start_cases(&handle, metadata("session.replay", "Live Replay"), materialize_cases(SOURCE, &suite)).await;
    let receipt = handle.start_next_execution(cycle_id.clone(), CommandContext::default()).await;
    assert_eq!(receipt.status, CommandStatus::Accepted);
    wait_for_run_settled(&handle, &receipt.command_id).await;

    let cycle = cycle_snapshot(&handle, &cycle_id).await;
    let execution = &cycle.test_executions[0];
    let files = handle.execution_files(&execution.test_execution_id).await.unwrap();
    let mut log = String::new();
    if let Some(file) = files.iter().find(|file| file.relative_path == "artifacts/execution.log") {
        let path = file.relative_path.clone();
        let content = handle.execution_file_content(&execution.test_execution_id, path, 0, 4096).await.unwrap().unwrap();
        log = String::from_utf8(content.bytes.to_vec()).unwrap();
    }
    assert!(!log.contains("did not acknowledge its final row"), "{log}");
    assert_eq!(execution.status, ExecutionStatus::Passed, "{log}");
    assert!(host_rows(&handle).await > 0, "host collection ran while the run held its lease");
    let captured = handle
        .collect_telemetry(&dokime_runtime::domains::evidence::TelemetrySelection {
            streams: vec![identifier("host.system").into()],
            scope: dokime_runtime::domains::evidence::ScopeSelection {
                test_execution_id: Some(execution.test_execution_id.clone()),
                ..Default::default()
            },
            ..Default::default()
        })
        .await
        .unwrap();
    assert!(!captured.is_empty(), "the observer stream is retained under the execution's lease without a nominal cadence");
    let leases = handle.stream_leases_for(&execution.scope).await;
    assert_eq!(leases.len(), 1);
    assert!(leases[0].closed_at.is_some());
    handle.release_stream_leases(viewing).await.unwrap();
    probe.runtime.lock().unwrap().take();
    drop(handle);
    owner.shutdown().await.unwrap();
}

pub(crate) async fn provider_failure_releases() {
    let root = tempdir().unwrap();
    let probe = Arc::new(Probe::new(Settlement::Failure));
    let configuration =
        config(root.path()).with_plugins([crate::service_support::bundles::host_plugin()]).register(Arc::new(Provider(probe.clone())));
    let (mut owner, handle) = started(configuration).await;
    let suite = handle.test_suites().await.into_iter().find(|suite| suite.source == SOURCE).unwrap().suite;
    let cycle_id = start_cases(&handle, metadata("session.failure", "Provider Failure"), materialize_cases(SOURCE, &suite)).await;
    let receipt = handle.start_next_execution(cycle_id.clone(), CommandContext::default()).await;
    assert_eq!(receipt.status, CommandStatus::Accepted);
    probe.entered.notified().await;
    let cycle = cycle_snapshot(&handle, &cycle_id).await;
    let scope = cycle.test_executions[0].scope.clone();
    let leases = handle.stream_leases_for(&scope).await;
    assert_eq!(leases.len(), 1);
    assert!(leases[0].closed_at.is_none());
    probe.released.send_replace(true);
    wait_for_run_settled(&handle, &receipt.command_id).await;
    let failed = cycle_snapshot(&handle, &cycle_id).await;
    assert_eq!(failed.lifecycle, CycleStatus::Ready, "a failed request leaves its Active cycle idle");
    assert_eq!(failed.result.as_ref().map(|result| result.status.clone()), Some(ResultStatus::Blocked));
    // The provider failed inside its open step, so the host interrupts it.
    assert_eq!(failed.test_executions[0].status, ExecutionStatus::Interrupted);
    assert!(*probe.cancellation.borrow(), "failure cleanup cancels the provider");
    let released = handle.stream_leases_for(&scope).await;
    assert_eq!(released.len(), 1);
    assert!(released[0].closed_at.is_some());
    assert!(
        handle
            .watch_state()
            .latest()
            .state()
            .installed_plugin_errors
            .iter()
            .any(|error| error.code.as_deref() == Some("provider_execution_run"))
    );
    drop(handle);
    owner.shutdown().await.unwrap();
}

// Saved Values Wait For Explicit Reload And Are Visible To Independent Readers.
pub(crate) async fn saved_change_waits_for_reload() {
    let root = tempdir().unwrap();
    let (mut owner, handle) = started(config(root.path()).with_plugins([crate::service_support::bundles::host_plugin()])).await;
    let (generation, prepared, _) = prepared_scan_interval(&owner).await;
    assert!(handle.plugin_settings(&identifier(HOST)).await.unwrap().unapplied.is_empty());

    let equivalent = Value::Integer(dokime_domain::model::values::numeric_value(&prepared).unwrap() as i64);
    handle
        .save_plugin_settings(
            &identifier(HOST),
            PluginSettingsUpdate {
                values: std::collections::BTreeMap::from([(SCAN_INTERVAL.into(), equivalent)]),
            },
        )
        .await
        .unwrap();
    assert!(handle.plugin_settings(&identifier(HOST)).await.unwrap().unapplied.is_empty());
    assert_eq!(save_scan_interval(&handle, 47).await, PluginRestart::Idle);
    let unchanged = prepared_scan_interval(&owner).await;
    assert_eq!(unchanged.0, generation);
    assert_eq!(unchanged.1, prepared);
    for reader in [handle.clone(), owner.handle()] {
        assert_eq!(reader.plugin_settings(&identifier(HOST)).await.unwrap().unapplied, [SCAN_INTERVAL]);
    }
    handle
        .save_plugin_settings(
            &identifier(HOST),
            PluginSettingsUpdate {
                values: std::collections::BTreeMap::from([(SCAN_INTERVAL.into(), prepared.clone())]),
            },
        )
        .await
        .unwrap();
    assert!(handle.plugin_settings(&identifier(HOST)).await.unwrap().unapplied.is_empty());
    assert_eq!(prepared_scan_interval(&owner).await.0, generation);
    save_scan_interval(&handle, 47).await;
    assert_eq!(handle.reload_plugin_program(&identifier(HOST)), PluginRestart::Restarting);
    handle.plugin_restart_settled(&identifier(HOST)).await;
    assert!(handle.plugin_settings(&identifier(HOST)).await.unwrap().unapplied.is_empty());
    let (restarted, applied, saved) = prepared_scan_interval(&owner).await;
    assert_ne!(restarted, generation, "reload started a new generation");
    assert_ne!(applied, prepared);
    assert_eq!(applied, saved, "the new generation prepared with the saved value");

    assert_eq!(save_scan_interval(&handle, 47).await, PluginRestart::Idle);
    assert_eq!(prepared_scan_interval(&owner).await.0, restarted, "an unchanged save keeps the running generation");
    drop(handle);
    owner.shutdown().await.unwrap();
}

// Reload Waits Across Active Cases And Applies The Latest Saved Values.
pub(crate) async fn reload_waits_for_cycle() {
    let root = tempdir().unwrap();
    let probe = Arc::new(Probe::new(Settlement::Held));
    let configuration =
        config(root.path()).with_plugins([crate::service_support::bundles::host_plugin()]).register(Arc::new(Provider(probe.clone())));
    let (mut owner, handle) = started(configuration).await;
    *probe.runtime.lock().unwrap() = Some(handle.clone());
    let (generation, prepared, _) = prepared_scan_interval(&owner).await;
    let cycle_id = held_cycle(&handle, &probe, metadata("session.restart", "Pending Restart")).await;

    assert_eq!(save_scan_interval(&handle, 47).await, PluginRestart::Idle);
    assert_eq!(handle.reload_plugin_program(&identifier(HOST)), PluginRestart::Pending);
    assert_eq!(save_scan_interval(&handle, 48).await, PluginRestart::Pending);
    assert_eq!(handle.plugin_settings(&identifier(HOST)).await.unwrap().restart, PluginRestart::Pending);
    next_case(&probe).await;
    assert_eq!(handle.plugin_settings(&identifier(HOST)).await.unwrap().restart, PluginRestart::Pending);
    assert_eq!(prepared_scan_interval(&owner).await.0, generation, "the cycle keeps its host generation between cases");

    probe.released.send_replace(true);
    let cycle = wait_for_cycle_outcome(&handle, &cycle_id, ResultStatus::Passed).await;
    handle.plugin_restart_settled(&identifier(HOST)).await;
    for execution in &cycle.test_executions {
        let log = execution_log(&handle, &execution.test_execution_id).await;
        assert_eq!(execution.status, ExecutionStatus::Passed, "the restart never touched the cycle: {log}");
    }
    let (restarted, applied, saved) = prepared_scan_interval(&owner).await;
    assert_ne!(restarted, generation);
    assert_ne!(applied, prepared);
    assert_eq!(applied, saved, "the restart prepared with the latest saved value");

    // The Idle Cycle Stays Open, Yet Nothing Is Active.
    assert_eq!(save_scan_interval(&handle, 49).await, PluginRestart::Idle);
    assert_eq!(handle.reload_plugin_program(&identifier(HOST)), PluginRestart::Restarting);
    handle.plugin_restart_settled(&identifier(HOST)).await;
    assert_ne!(prepared_scan_interval(&owner).await.0, restarted, "reload with no active cycle restarts at once");
    probe.runtime.lock().unwrap().take();
    drop(handle);
    owner.shutdown().await.unwrap();
}

// Installing, Replacing, Retrying, Disabling And Removing A Bundle Apply In The Running Backend.
pub(crate) async fn installs_apply_live() {
    let root = tempdir().unwrap();
    let (mut owner, handle) = started(config(root.path())).await;
    assert!(running_generation(&owner).await.is_none());

    let first = installed(&handle, host_bundle()).await;
    assert!(first.active && !first.boot_failed, "{:?}", first.installation.last_boot);
    assert_eq!(first.restart, PluginRestart::Idle);
    assert!(admits_host(&handle).await, "the installed catalog is admitted without a restart");
    let started = running_generation(&owner).await.expect("the installed program runs");

    // A Changed Version Becomes The Running Generation.
    let publishing = handle.plugin_store().root().join("publishing.dokime");
    std::fs::write(&publishing, b"an install has published its file before committing settings").unwrap();
    let second = installed(&handle, crate::service_support::bundles::host_plugin_version("99.0.0")).await;
    assert!(second.active, "{:?}", second.installation.last_boot);
    let store = handle.plugin_store();
    assert!(store.list().await.unwrap().iter().all(|entry| entry.installation.install_id != first.installation.install_id));
    assert!(!store.path(first.installation.install_id).exists());
    assert!(publishing.is_file(), "live apply never sweeps unreferenced files");
    let replaced = running_generation(&owner).await.expect("the new version runs");
    assert_ne!(replaced, started);

    // A Retry Starts Only This Plugin Again.
    let retried = handle.reload_installed_plugin(second.installation.install_id).await.unwrap();
    settled(&handle, &retried).await;
    handle.plugin_restart_settled(&identifier(HOST)).await;
    let restarted = running_generation(&owner).await.expect("the retried program runs");
    assert_ne!(restarted, replaced);

    // Disabling Stops It, Enabling Starts It, And Removing Stops It Again.
    let disabled = handle.set_plugin_enabled(second.installation.install_id, false).await.unwrap();
    let disabled = settled(&handle, &disabled).await;
    assert!(!disabled.active);
    assert!(running_generation(&owner).await.is_none() && !admits_host(&handle).await);
    let enabled = handle.set_plugin_enabled(second.installation.install_id, true).await.unwrap();
    assert!(settled(&handle, &enabled).await.active);
    assert!(running_generation(&owner).await.is_some() && admits_host(&handle).await);
    handle.remove_plugin(second.installation.install_id).await.unwrap();
    handle.plugin_bundle_settled(&second.metadata.as_ref().unwrap().identity.name).await;
    assert!(store.list().await.unwrap().is_empty());
    assert!(!store.path(second.installation.install_id).exists());
    assert!(running_generation(&owner).await.is_none() && !admits_host(&handle).await);
    drop(handle);
    owner.shutdown().await.unwrap();
}

// A Replacement While A Cycle Is Active Waits Across The Gap Between Its Cases, Then Applies.
pub(crate) async fn replacement_waits_for_cycle() {
    let root = tempdir().unwrap();
    let probe = Arc::new(Probe::new(Settlement::HeldStreams));
    let (mut owner, handle) = started(config(root.path())).await;
    *probe.runtime.lock().unwrap() = Some(handle.clone());
    let first = installed(&handle, host_bundle()).await;
    owner.register(Arc::new(Provider(probe.clone()))).await.unwrap();
    let generation = running_generation(&owner).await.expect("the installed program runs");
    let cycle_id = held_cycle(&handle, &probe, metadata("session.replace", "Pending Replacement")).await;

    let replacement = handle.install_plugin(crate::service_support::bundles::host_plugin_version("99.0.0"), |_| Ok(())).await.unwrap();
    let pending = installation(&handle, replacement.installation.install_id).await;
    assert_eq!(pending.restart, PluginRestart::Pending);
    assert!(!pending.active);
    assert_eq!(installation(&handle, first.installation.install_id).await.installation.state, PluginState::RemovalPending);
    assert!(handle.plugin_store().path(first.installation.install_id).is_file());
    assert_eq!(handle.plugin_settings(&identifier(HOST)).await.unwrap().restart, PluginRestart::Pending);
    next_case(&probe).await;
    assert_eq!(installation(&handle, replacement.installation.install_id).await.restart, PluginRestart::Pending);
    assert!(installation(&handle, first.installation.install_id).await.active, "the previous version keeps running");
    assert_eq!(running_generation(&owner).await, Some(generation), "the cycle keeps its generation between cases");

    #[cfg(feature = "testing")]
    let drain = {
        let cycle = cycle_snapshot(&handle, &cycle_id).await;
        dokime_runtime::testing::hold_stream_drain(&cycle.test_executions[1].test_execution_id)
    };
    probe.released.send_replace(true);
    #[cfg(feature = "testing")]
    {
        drain.entered().await;
        assert_eq!(installation(&handle, replacement.installation.install_id).await.restart, PluginRestart::Pending);
        assert_eq!(running_generation(&owner).await, Some(generation));
        let cycle = cycle_snapshot(&handle, &cycle_id).await;
        let leases = handle.stream_leases_for(&cycle.test_executions[1].scope).await;
        assert!(leases.iter().any(|lease| lease.closed_at.is_none()), "the plugin stream still belongs to this execution");
        drain.release();
    }
    let cycle = wait_for_cycle_outcome(&handle, &cycle_id, ResultStatus::Passed).await;
    let applied = settled(&handle, &replacement).await;
    for execution in &cycle.test_executions {
        let log = execution_log(&handle, &execution.test_execution_id).await;
        assert_eq!(execution.status, ExecutionStatus::Passed, "the replacement never touched the cycle: {log}");
    }
    assert!(applied.active && applied.restart == PluginRestart::Idle);
    let store = handle.plugin_store();
    assert!(store.list().await.unwrap().iter().all(|entry| entry.installation.install_id != first.installation.install_id));
    assert!(!store.path(first.installation.install_id).exists());
    assert_ne!(running_generation(&owner).await.expect("the new version runs"), generation);
    probe.runtime.lock().unwrap().take();
    drop(handle);
    owner.shutdown().await.unwrap();
}

// A Successful Replacement Retires Failed Boots, Including After Startup Recovery.
pub(crate) async fn failed_installation_is_replaced() {
    for restart in [false, true] {
        // Install A Package Whose Executable Cannot Start.
        let root = tempdir().unwrap();
        let (mut owner, mut handle) = started(config(root.path())).await;
        let unprepared = installed(&handle, secret_bundle("plugin-host", HOST)).await;
        assert!(unprepared.boot_failed && !unprepared.active);
        let mut failed = installed(&handle, crate::service_support::bundles::bundle("plugin-host", HOST, "2.0.0")).await;
        let failed_id = failed.installation.install_id;
        assert!(failed.boot_failed && !failed.active);
        assert!(matches!(failed.installation.last_boot, Some(PluginLastBoot::Failed { .. })));

        // Recover The Failed Installation Before Replacing It.
        if restart {
            drop(handle);
            owner.shutdown().await.unwrap();
            (owner, handle) = started(config(root.path())).await;
            assert!(installation(&handle, failed_id).await.boot_failed);
        }

        // Activate A Newer Bundle Of The Same Package Through The Live Install Path.
        let replacement = installed(&handle, crate::service_support::bundles::host_plugin_version("99.0.0")).await;
        let store = handle.plugin_store();
        let listed = store.list().await.unwrap();
        assert_eq!(listed.len(), 1);
        let active = installation(&handle, listed[0].installation.install_id).await;
        assert_eq!(active.installation.install_id, replacement.installation.install_id);
        assert!(active.active && !active.boot_failed);
        assert!(matches!(active.installation.last_boot, Some(PluginLastBoot::Loaded { .. })));
        assert!(!store.path(failed_id).exists());
        assert!(!store.path(unprepared.installation.install_id).exists());
        handle.plugin_process_state(&mut failed);
        assert!(!failed.boot_failed, "the discarded installation has no process boot failure");

        // Reopening Retains Only The Working Installation.
        drop(store);
        drop(handle);
        owner.shutdown().await.unwrap();
        let (mut owner, handle) = started(config(root.path())).await;
        assert_eq!(handle.plugin_store().list().await.unwrap().len(), 1);
        assert!(installation(&handle, replacement.installation.install_id).await.active);
        drop(handle);
        owner.shutdown().await.unwrap();
    }

    // Recover A Replacement Published To The Store Before Live Activation Ran.
    let root = tempdir().unwrap();
    let (mut owner, handle) = started(config(root.path())).await;
    let failed = installed(&handle, crate::service_support::bundles::bundle("plugin-host", HOST, "0.0.1")).await;
    assert!(failed.boot_failed);
    let store = handle.plugin_store();
    let replacement =
        store.install(&crate::service_support::bundles::host_plugin_version("99.0.0"), &[], &Default::default()).await.unwrap();
    assert_eq!(store.list().await.unwrap().len(), 2);
    assert!(store.path(failed.installation.install_id).is_file());
    drop(store);
    drop(handle);
    owner.shutdown().await.unwrap();
    let (mut owner, handle) = started(config(root.path())).await;
    assert_eq!(handle.plugin_store().list().await.unwrap().len(), 1);
    let active = installation(&handle, replacement.installation.install_id).await;
    assert!(active.active && !active.boot_failed);
    assert!(matches!(active.installation.last_boot, Some(PluginLastBoot::Loaded { .. })));
    assert!(!handle.plugin_store().path(failed.installation.install_id).exists());
    drop(handle);
    owner.shutdown().await.unwrap();
}

// Failed Or Refused Installations Can Be Removed Without Stopping A Working Version.
pub(crate) async fn failed_installations_can_be_removed() {
    let root = tempdir().unwrap();
    let (mut owner, handle) = started(config(root.path())).await;

    // Remove A Package That Never Started.
    let failed = installed(&handle, crate::service_support::bundles::bundle("equipment", "equipment", "1.0.0")).await;
    assert!(failed.boot_failed);
    handle.remove_plugin(failed.installation.install_id).await.unwrap();
    handle.plugin_bundle_settled("equipment").await;
    assert!(handle.plugin_store().list().await.unwrap().is_empty());
    assert!(!handle.plugin_store().path(failed.installation.install_id).exists());

    // Remove A Refused Replacement While The Previous Version Keeps Running.
    let active = installed(&handle, host_bundle()).await;
    let package = active.metadata.as_ref().unwrap().identity.name.clone();
    let generation = running_generation(&owner).await.unwrap();
    let refused = installed(&handle, secret_bundle(&package, HOST)).await;
    assert!(refused.boot_failed && refused.installation.state == PluginState::RemovalPending);
    handle.remove_plugin(refused.installation.install_id).await.unwrap();
    handle.plugin_bundle_settled(&package).await;
    assert_eq!(handle.plugin_store().list().await.unwrap().len(), 1);
    assert!(!handle.plugin_store().path(refused.installation.install_id).exists());
    assert!(installation(&handle, active.installation.install_id).await.active);
    assert_eq!(running_generation(&owner).await, Some(generation));

    // An Unreadable Failed Bundle Is Also Removable Without A Verified Package Name.
    let unreadable = installed(&handle, crate::service_support::bundles::bundle("equipment", "equipment", "1.0.0")).await;
    let store = handle.plugin_store();
    std::fs::write(store.path(unreadable.installation.install_id), b"invalid bundle").unwrap();
    handle.remove_plugin(unreadable.installation.install_id).await.unwrap();
    assert!(!store.path(unreadable.installation.install_id).exists());
    assert_eq!(store.list().await.unwrap().len(), 1);
    drop(store);
    drop(handle);
    owner.shutdown().await.unwrap();
}

// A Refused Replacement Keeps The Previous Installation Running And Installed.
pub(crate) async fn refused_replacement_keeps_running() {
    let root = tempdir().unwrap();
    let (mut owner, handle) = started(config(root.path())).await;
    let first = installed(&handle, host_bundle()).await;
    let package = first.metadata.as_ref().unwrap().identity.name.clone();
    let generation = running_generation(&owner).await.expect("the installed program runs");

    // Refused Before Anything Stops: The Running Generation Is Untouched.
    let unprepared = installed(&handle, secret_bundle(&package, HOST)).await;
    assert_eq!(unprepared.installation.state, PluginState::RemovalPending);
    assert!(unprepared.boot_failed && !unprepared.active);
    let Some(PluginLastBoot::Failed { failure, .. }) = &unprepared.installation.last_boot else { panic!("the refusal is recorded") };
    assert_eq!(failure.missing_secret.as_deref(), Some("token"));
    let kept = installation(&handle, first.installation.install_id).await;
    assert_eq!(kept.installation.state, PluginState::Installed);
    assert!(kept.active);
    assert_eq!(running_generation(&owner).await, Some(generation));

    // Refused When It Starts: The Previous Version Runs Again.
    let unstarted = installed(&handle, crate::service_support::bundles::bundle(&package, HOST, "3.0.0")).await;
    assert_eq!(unstarted.installation.state, PluginState::RemovalPending);
    assert!(unstarted.boot_failed && !unstarted.active);
    let kept = installation(&handle, first.installation.install_id).await;
    assert_eq!(kept.installation.state, PluginState::Installed);
    assert!(kept.active, "{:?}", kept.installation.last_boot);
    assert!(running_generation(&owner).await.is_some() && admits_host(&handle).await);
    let runnable: Vec<_> = handle.plugin_store().installed().await.unwrap();
    assert_eq!(runnable.len(), 1, "a restart runs the kept installation alone");
    // A Later Successful Apply Retires Both Refused Replacements And Their Diagnostics.
    let replacement = installed(&handle, crate::service_support::bundles::host_plugin_version("99.0.0")).await;
    assert!(replacement.active);
    assert_eq!(handle.plugin_store().list().await.unwrap().len(), 1);
    for mut refused in [unprepared, unstarted] {
        assert!(!handle.plugin_store().path(refused.installation.install_id).exists());
        handle.plugin_process_state(&mut refused);
        assert!(!refused.boot_failed);
    }
    drop(handle);
    owner.shutdown().await.unwrap();
}

// A Refused Restart Keeps The Running Generation; Retry Starts One Plugin Again Once The Cause Is Fixed.
// While A Cycle Is Active, Retry Starts A Plugin That Did Not Start At Once, And Restarting One That Runs Waits.
pub(crate) async fn reload_restarts_one_plugin() {
    let root = tempdir().unwrap();
    let probe = Arc::new(Probe::new(Settlement::Held));
    let (mut owner, handle) = started(config(root.path()).register(Arc::new(Provider(probe.clone())))).await;
    *probe.runtime.lock().unwrap() = Some(handle.clone());

    // A Restart That Cannot Read Its Settings Is Refused While The Program Keeps Running.
    let host = installed(&handle, host_bundle()).await;
    let install = host.installation.install_id;
    let generation = running_generation(&owner).await.expect("the installed program runs");
    save_unresolvable(&handle, install, true).await;
    handle.reload_installed_plugin(install).await.unwrap();
    handle.plugin_restart_settled(&identifier(HOST)).await;
    let refused = installation(&handle, install).await;
    assert!(refused.active && !refused.boot_failed, "the previous generation still runs");
    assert!(matches!(refused.installation.last_boot, Some(PluginLastBoot::Failed { .. })));
    assert_eq!(running_generation(&owner).await, Some(generation));
    save_unresolvable(&handle, install, false).await;
    handle.reload_installed_plugin(install).await.unwrap();
    handle.plugin_restart_settled(&identifier(HOST)).await;
    let retried = installation(&handle, install).await;
    assert!(retried.active && matches!(retried.installation.last_boot, Some(PluginLastBoot::Loaded { .. })));
    let generation = running_generation(&owner).await.expect("the retried program runs");

    // A Plugin That Did Not Start Is Retried In Place While A Cycle Runs: It Reads The Secret Stored Since.
    let failed = installed(&handle, secret_bundle("secret-fixture", "secret_fixture")).await;
    assert!(failed.boot_failed && !failed.active);
    let Some(PluginLastBoot::Failed { failure, .. }) = &failed.installation.last_boot else { panic!("the failed start is recorded") };
    assert_eq!(failure.missing_secret.as_deref(), Some("token"));
    let cycle_id = held_cycle(&handle, &probe, metadata("session.retry", "Retry During a Cycle")).await;
    handle
        .operations()
        .operation::<dokime_runtime::operations::secrets::Put>(dokime_runtime::operations::secrets::Put {
            metadata: dokime_domain::model::secrets::catalog::Metadata {
                identifier: "token".into(),
                label: "Token".into(),
                description: None,
            },
            value: Some("stored".into()),
            generation: 0,
        })
        .await
        .unwrap();
    let retried = handle.reload_installed_plugin(failed.installation.install_id).await.unwrap();
    assert_ne!(installation(&handle, retried.installation.install_id).await.restart, PluginRestart::Pending);
    let retried = settled(&handle, &retried).await;
    let Some(PluginLastBoot::Failed { failure, .. }) = &retried.installation.last_boot else { panic!("the inert program cannot start") };
    assert_eq!(failure.missing_secret, None, "the retry prepared with the stored secret");
    assert!(installation(&handle, install).await.active, "retrying one plugin leaves the others running");

    // Retrying A Running Plugin Restarts It Once The Cycle Ends.
    handle.reload_installed_plugin(install).await.unwrap();
    assert_eq!(handle.plugin_settings(&identifier(HOST)).await.unwrap().restart, PluginRestart::Pending);
    next_case(&probe).await;
    assert_eq!(running_generation(&owner).await, Some(generation), "the running plugin keeps its generation during the cycle");
    probe.released.send_replace(true);
    wait_for_cycle_outcome(&handle, &cycle_id, ResultStatus::Passed).await;
    handle.plugin_restart_settled(&identifier(HOST)).await;
    assert_ne!(running_generation(&owner).await.expect("the retried program runs"), generation);
    probe.runtime.lock().unwrap().take();
    drop(handle);
    owner.shutdown().await.unwrap();
}

/* === Private Functions === */

async fn save_scan_interval(runtime: &RuntimeHandle, seconds: u32) -> PluginRestart {
    let host = identifier(HOST);
    let values = std::collections::BTreeMap::from([(SCAN_INTERVAL.into(), dokime_runtime::Value::Number(f64::from(seconds)))]);
    runtime.save_plugin_settings(&host, PluginSettingsUpdate { values }).await.unwrap().restart
}

// What The Running Host Generation Was Prepared With, And What Is Saved.
async fn prepared_scan_interval(owner: &dokime_runtime::Runtime) -> (Option<PluginGeneration>, Value, Value) {
    let status = owner.fixture_plugin_status(HOST).await.expect("the host plugin is admitted");
    let definition = status.active_definition.expect("the host plugin runs a generation");
    let configuration: dokime_runtime::plugin::Configuration = serde_json::from_slice(&definition.configuration).unwrap();
    let saved = owner.handle().plugin_settings(&identifier(HOST)).await.unwrap().values[SCAN_INTERVAL].clone();
    (status.generation, configuration.settings[SCAN_INTERVAL].clone(), saved)
}

async fn execution_log(runtime: &RuntimeHandle, execution: &dokime_domain::model::identity::TestExecutionId) -> String {
    let files = runtime.execution_files(execution).await.unwrap();
    let Some(file) = files.iter().find(|file| file.relative_path == "artifacts/execution.log") else { return String::new() };
    let content = runtime.execution_file_content(execution, file.relative_path.clone(), 0, 4096).await.unwrap().unwrap();
    String::from_utf8_lossy(&content.bytes).into_owned()
}

async fn host_rows(runtime: &RuntimeHandle) -> usize {
    let selection = dokime_runtime::domains::evidence::TelemetrySelection {
        streams: vec![identifier("host.system").into()],
        ..Default::default()
    };
    runtime.collect_telemetry(&selection).await.unwrap().len()
}

fn cancellation_log() -> Vec<Entry> {
    vec![
        Entry::new(Level::Trace, "trace detail"),
        Entry::new(Level::Debug, "debug detail"),
        Entry::new(Level::Info, "fixture cancellation settled"),
        Entry::new(Level::Warn, "warning\ncontinued"),
        Entry::new(Level::Error, "error detail"),
        Entry::new(Level::Info, ""),
        Entry {
            level: 99,
            message: "unknown detail".into(),
            timestamp: dokime_runtime::execution::log::timestamp(),
        },
    ]
}

fn without_timestamps(log: &str) -> String {
    log.lines()
        .map(|line| {
            let (timestamp, message) = line.split_once(' ').unwrap();
            assert_eq!(timestamp.len(), 24);
            assert!(timestamp.ends_with('Z'));
            assert!(Timestamp::new(timestamp).unix_millis().is_some());
            format!("{message}\n")
        })
        .collect()
}

#[dokime_tracing::trace_test]
#[tokio::test]
async fn settled_run_fenced() {
    let root = tempdir().unwrap();
    let probe = Arc::new(Probe::new(Settlement::Ignored));
    let (mut owner, handle) = started(config(root.path()).register(Arc::new(Provider(probe.clone())))).await;
    let suite = handle.test_suites().await.into_iter().find(|suite| suite.source == SOURCE).unwrap().suite;
    let first = start_cases(&handle, metadata("session.old", "Old Run"), materialize_cases(SOURCE, &suite)).await;
    let receipt = handle.start_next_execution(first.clone(), CommandContext::default()).await;
    assert_eq!(receipt.status, CommandStatus::Accepted);
    probe.entered.notified().await;
    let old_token = probe.token.lock().unwrap().clone();
    let old_host = probe.hosts.lock().unwrap()[&old_token].clone();

    tokio::time::pause();
    let stop = {
        let handle = handle.clone();
        let first = first.clone();
        tokio::spawn(async move { handle.seal_cycle(first, CommandContext::default()).await.unwrap() })
    };
    probe.cancelled.notified().await;
    // Auto-Advance Only While Idle, So In-Flight Storage Keeps Its Own Deadlines.
    tokio::time::sleep(dokime_runtime::operations::STOP_SETTLEMENT.timeout + Duration::from_secs(1)).await;
    tokio::time::resume();
    stop.await.unwrap();
    wait_for_run_settled(&handle, &receipt.command_id).await;
    let closed_id = TestExecutionId::from_uuid(old_token.clone()).unwrap();
    let checkpoint = handle.execution_saved_checkpoint(closed_id.clone()).await.unwrap();
    let closed = crate::service_support::native::open(&handle, &checkpoint).await;
    assert_eq!(closed.execution().status, ExecutionStatus::Cancelled);
    drop(closed);
    probe.cancellation.send_replace(false);

    let following = start_cases(&handle, metadata("session.next", "Following Run"), materialize_cases(SOURCE, &suite)).await;
    let receipt = handle.start_next_execution(following.clone(), CommandContext::default()).await;
    assert_eq!(receipt.status, CommandStatus::Accepted);
    probe.entered.notified().await;
    let following_token = probe.token.lock().unwrap().clone();
    assert_ne!(following_token, old_token);
    let following_host = probe.hosts.lock().unwrap()[&following_token].clone();
    let late = EvidenceBatch {
        cursor: 2,
        phase: PHASE.into(),
        diagnostics: vec![Entry::new(Level::Info, "late old run")],
        ..Default::default()
    };
    assert!(
        following_host
            .push(EvidenceBatch {
                cursor: 2,
                diagnostics: vec![Entry::new(Level::Info, "following run remains active")],
                ..Default::default()
            })
            .await
            .is_ok()
    );
    let before = cycle_snapshot(&handle, &following).await;
    assert!(old_host.push(late).await.is_err());
    assert_eq!(cycle_snapshot(&handle, &following).await, before);
    assert_eq!(handle.execution_saved_checkpoint(closed_id).await.unwrap(), checkpoint);
    assert!(!execution_log(&handle, &before.test_executions[0].test_execution_id).await.contains("late old run"));
    probe.released.send_replace(true);
    probe.cancellation.send_replace(true);
    wait_for_run_settled(&handle, &receipt.command_id).await;
    probe.hosts.lock().unwrap().clear();
    owner.shutdown().await.unwrap();
}

async fn ignored_stop(mode: Settlement) {
    let root = tempdir().unwrap();
    let probe = Arc::new(Probe::new(mode));
    let (mut owner, handle) = started(config(root.path()).register(Arc::new(Provider(probe.clone())))).await;
    let suite = handle.test_suites().await.into_iter().find(|suite| suite.source == SOURCE).unwrap().suite;
    let cycle_id = start_cases(&handle, metadata("session.ignored", "Ignored Stop"), materialize_cases(SOURCE, &suite)).await;
    let receipt = handle.start_next_execution(cycle_id.clone(), CommandContext::default()).await;
    assert_eq!(receipt.status, CommandStatus::Accepted);
    probe.entered.notified().await;
    let original = cycle_snapshot(&handle, &cycle_id).await.test_executions.remove(0);

    tokio::time::pause();
    let stopping = {
        let handle = handle.clone();
        let cycle_id = cycle_id.clone();
        tokio::spawn(async move { handle.seal_cycle(cycle_id, CommandContext::default()).await.unwrap() })
    };
    probe.cancelled.notified().await;
    // The paused clock reaches the stop budget only while the runtime is idle: forcing it forward
    // with `advance` also expired storage deadlines of writes still in flight on a slow machine,
    // failing the journal capture and leaving the run unsettled.
    tokio::time::sleep(dokime_runtime::operations::STOP_SETTLEMENT.timeout + Duration::from_secs(1)).await;
    tokio::time::resume();
    stopping.await.unwrap();
    wait_for_run_settled(&handle, &receipt.command_id).await;
    let cycle = cycle_snapshot(&handle, &cycle_id).await;
    assert_eq!(cycle.lifecycle, CycleStatus::Stopped);
    assert_eq!(cycle.test_executions.len(), 1);
    let execution = &cycle.test_executions[0];
    assert_eq!(execution.test_execution_id, original.test_execution_id);
    assert_eq!(execution.scope, original.scope);
    assert_eq!(execution.snapshot, original.snapshot);
    assert_eq!(execution.steps[0].step_id, original.steps[0].step_id);
    assert_eq!(execution.status, ExecutionStatus::Cancelled);
    assert!(cycle.saved.is_some(), "host settlement seals the final cycle report");
    assert!(handle.pending_prompt_projection().await.is_empty());
    let files = handle.execution_files(&execution.test_execution_id).await.unwrap();
    let log = files.iter().find(|file| file.relative_path == "artifacts/execution.log").unwrap();
    let log = handle.execution_file_content(&execution.test_execution_id, log.relative_path.clone(), 0, 4096).await.unwrap().unwrap();
    assert!(std::str::from_utf8(&log.bytes).unwrap().contains(&format!(
        "the run did not answer its stop within {} seconds",
        dokime_runtime::operations::STOP_SETTLEMENT.timeout.as_secs()
    )));
    probe.released.send_replace(true);
    drop(handle);
    owner.shutdown().await.unwrap();
}

// A Cycle That Runs Its Case Twice, Continuing Automatically, Held In Its First Execution.
async fn held_cycle(runtime: &RuntimeHandle, probe: &Probe, metadata: Metadata) -> TestCycleId {
    let suite = runtime.test_suites().await.into_iter().find(|suite| suite.source == SOURCE).unwrap().suite;
    let case = materialize_cases(SOURCE, &suite).remove(0);
    let cycle_id = start_cases(runtime, metadata, vec![case.clone(), case]).await;
    let receipt = runtime.start_next_execution(cycle_id.clone(), CommandContext::default()).await;
    assert_eq!(receipt.status, CommandStatus::Accepted);
    probe.entered.notified().await;
    cycle_id
}

// Release The Held Execution; The Cycle Continues With Its Next Case, Which Holds In Turn.
async fn next_case(probe: &Probe) {
    probe.released.send_replace(true);
    probe.entered.notified().await;
}

// What The Store Says About One Installation, With What The Running Backend Does With It.
async fn installation(runtime: &RuntimeHandle, install: uuid::Uuid) -> PluginEntry {
    let mut entry = runtime.plugin_store().get(install).await.unwrap();
    runtime.plugin_process_state(&mut entry);
    entry
}

// Install `bytes` Into The Running Backend And Wait Until It Applied Them.
async fn installed(runtime: &RuntimeHandle, bytes: Vec<u8>) -> PluginEntry {
    let entry = runtime.install_plugin(bytes, |_| Ok(())).await.unwrap();
    runtime.plugin_bundle_settled(&entry.metadata.as_ref().unwrap().identity.name).await;
    installation(runtime, entry.installation.install_id).await
}

async fn settled(runtime: &RuntimeHandle, entry: &PluginEntry) -> PluginEntry {
    runtime.plugin_bundle_settled(&entry.metadata.as_ref().unwrap().identity.name).await;
    installation(runtime, entry.installation.install_id).await
}

// The Ready Generation Of The Host Program, If It Runs.
async fn running_generation(owner: &dokime_runtime::Runtime) -> Option<PluginGeneration> {
    let status = owner.fixture_plugin_status(HOST).await?;
    (status.state == dokime_runtime::harmos::plugin::PluginState::Ready).then_some(status.generation).flatten()
}

// Whether The Admitted Catalog Lists The Host Program.
async fn admits_host(runtime: &RuntimeHandle) -> bool {
    let view = runtime
        .query_runtime_view(RuntimeViewQuery {
            topic: RuntimeViewTopic::InstalledPlugins,
            scope: None,
            window: RuntimeWindow::Full,
            resolution: RuntimeResolution::Raw,
            cursor: None,
        })
        .await
        .unwrap();
    let RuntimeViewPayload::InstalledPlugins { frame } = view.payload else { panic!("unexpected payload") };
    frame.installed_plugins.iter().any(|plugin| plugin.plugin_id.as_str() == HOST)
}

fn host_bundle() -> Vec<u8> {
    std::fs::read(crate::service_support::bundles::host_plugin_bundle()).unwrap()
}

// A Bundle Whose Program Requires A Secret Nobody Stored.
fn secret_bundle(package: &str, owner: &str) -> Vec<u8> {
    use dokime_domain::model::settings::{SecretSetting, Settings, SettingsEntry};
    let token = SecretSetting {
        id: "token".into(),
        title: "Access Token".into(),
        description: None,
        required: true,
    };
    let settings = Settings {
        contributions: vec![SettingsEntry::Secret(token)],
    };
    crate::service_support::bundles::bundle_with_settings(package, owner, settings)
}

// Saved Values The Program's Catalog Refuses, So Reading Its Settings Fails.
async fn save_unresolvable(runtime: &RuntimeHandle, install: uuid::Uuid, refused: bool) {
    let values = refused.then(|| dokime_runtime::plugins::ProgramValues {
        values: [("undeclared".to_owned(), Value::Bool(true))].into_iter().collect(),
        saved: ["undeclared".to_owned()].into_iter().collect(),
    });
    runtime
        .settings()
        .edit(move |document| {
            let programs = &mut document.plugins.get_mut(&install).expect("the installation is stored").programs;
            match values {
                Some(values) => programs.insert(HOST.to_owned(), values),
                None => programs.remove(HOST),
            };
            Ok::<_, dokime_runtime::plugins::PluginError>(())
        })
        .await
        .unwrap();
}

/* === Tests === */
