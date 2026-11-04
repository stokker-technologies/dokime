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
    arrivals: Mutex<Vec<(String, String)>>,
    settling: std::sync::atomic::AtomicBool,
    settlement_signals: std::sync::atomic::AtomicUsize,
    record_ordinal: std::sync::atomic::AtomicU64,
    // The Live Replay Story Reads Host Rows Through Its Runtime And Clears It Before Shutdown.
    runtime: Mutex<Option<RuntimeHandle>>,
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
    // The Run Stays Active Until Released, Then Passes.
    Held,
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
            arrivals: Mutex::new(Vec::new()),
            settling: std::sync::atomic::AtomicBool::new(false),
            settlement_signals: std::sync::atomic::AtomicUsize::new(0),
            record_ordinal: std::sync::atomic::AtomicU64::new(0),
            runtime: Mutex::new(None),
        }
    }

    fn receive(&self, encoded: &[String]) {
        let mut arrivals = self.arrivals.lock().unwrap();
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
    fn receipts(&self) -> Vec<dokime_runtime::transport::protocol::SampleReceipt> {
        let arrivals = self.arrivals.lock().unwrap();
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
            if matches!(self.mode, Settlement::LiveReplay | Settlement::Held) {
                if self.mode == Settlement::Held {
                    self.released.subscribe().wait_for(|released| *released).await.unwrap();
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
            if self.mode == Settlement::LiveReplay {
                let settling = self.settling.load(std::sync::atomic::Ordering::SeqCst);
                match &signal.samples {
                    Some(delivery) => self.receive(&delivery.samples),
                    // The Second Settlement Signal Asks For The Receipt After The Replay Cut; Collection Keeps Ticking.
                    None if settling && self.settlement_signals.fetch_add(1, std::sync::atomic::Ordering::SeqCst) == 1 => {
                        let runtime = self.runtime.lock().unwrap().clone().expect("the live replay story binds its runtime");
                        let from = host_rows(&runtime).await;
                        until_telemetry(&runtime, || async { (host_rows(&runtime).await >= from + 3).then_some(()) }).await;
                    }
                    None => {}
                }
                return Ok(SignalReply {
                    samples: self.receipts(),
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
            Ok(SignalReply { record_ordinal, ..Default::default() })
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
        if matches!(self.0.mode, Settlement::Failure | Settlement::LiveReplay) {
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

/* === Private Functions === */
async fn save_scan_interval(runtime: &RuntimeHandle, seconds: u32) -> PluginRestart {
    let host = identifier(HOST);
    let mut values = runtime.plugin_settings(&host).await.unwrap().values;
    values.insert(SCAN_INTERVAL.into(), dokime_runtime::Value::from(seconds));
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
    let Some(file) = files.iter().find(|file| file.relative_path.ends_with("stream.000000.log")) else { return String::new() };
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

/* === Tests === */
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
        let log = files.iter().find(|file| file.relative_path.ends_with("stream.000000.log")).unwrap();
        let content =
            handle.execution_file_content(&execution.test_execution_id, log.relative_path.clone(), 0, 4096).await.unwrap().unwrap();
        let log_path = log.relative_path.clone();
        let checkpoint = handle.execution_saved_checkpoint(execution.test_execution_id.clone()).await.unwrap();
        let log = without_timestamps(std::str::from_utf8(&content.bytes).unwrap());
        if mode == Settlement::Refused {
            assert!(log.starts_with("[ERROR] "), "{log}");
            assert!(log.contains("run refused during cancellation"), "{log}");
        } else {
            assert_eq!(
                log,
                "[TRACE] trace detail\n[DEBUG] debug detail\n[INFO] fixture cancellation settled\n[WARN] warning\n[WARN] continued\n[ERROR] error detail\n[INFO] \n[UNKNOWN(99)] unknown detail\n"
            );
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
    let log = files.iter().find(|file| file.relative_path.ends_with("stream.000000.log")).unwrap();
    let log = handle.execution_file_content(&execution.test_execution_id, log.relative_path.clone(), 0, 4096).await.unwrap().unwrap();
    assert!(std::str::from_utf8(&log.bytes).unwrap().contains(&format!(
        "the run did not answer its stop within {} seconds",
        dokime_runtime::operations::STOP_SETTLEMENT.timeout.as_secs()
    )));
    probe.released.send_replace(true);
    drop(handle);
    owner.shutdown().await.unwrap();
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
    let log = files.iter().find(|file| file.relative_path.ends_with("stream.000000.log")).unwrap();
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
    let log = files.iter().find(|file| file.relative_path.ends_with("stream.000000.log")).unwrap();
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
    if let Some(file) = files.iter().find(|file| file.relative_path.ends_with("stream.000000.log")) {
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

// A Saved Change Restarts The Idle Host Plugin With It; Saving It Again Restarts Nothing.
pub(crate) async fn saved_change_restarts() {
    let root = tempdir().unwrap();
    let (mut owner, handle) = started(config(root.path()).with_plugins([crate::service_support::bundles::host_plugin()])).await;
    let (generation, prepared, _) = prepared_scan_interval(&owner).await;

    assert_eq!(save_scan_interval(&handle, 47).await, PluginRestart::Restarting);
    handle.plugin_restart_settled(&identifier(HOST)).await;
    let (restarted, applied, saved) = prepared_scan_interval(&owner).await;
    assert_ne!(restarted, generation, "the save started a new generation");
    assert_ne!(applied, prepared);
    assert_eq!(applied, saved, "the new generation prepared with the saved value");

    assert_eq!(save_scan_interval(&handle, 47).await, PluginRestart::Idle);
    assert_eq!(prepared_scan_interval(&owner).await.0, restarted, "an unchanged save keeps the running generation");
    drop(handle);
    owner.shutdown().await.unwrap();
}

// A Save While An Execution Is Active Waits For It: The Host Publishes Telemetry Any Run May Capture.
// The Restart Follows Once, With The Latest Value.
pub(crate) async fn saved_change_waits_for_execution() {
    let root = tempdir().unwrap();
    let probe = Arc::new(Probe::new(Settlement::Held));
    let configuration =
        config(root.path()).with_plugins([crate::service_support::bundles::host_plugin()]).register(Arc::new(Provider(probe.clone())));
    let (mut owner, handle) = started(configuration).await;
    *probe.runtime.lock().unwrap() = Some(handle.clone());
    let (generation, prepared, _) = prepared_scan_interval(&owner).await;
    let suite = handle.test_suites().await.into_iter().find(|suite| suite.source == SOURCE).unwrap().suite;
    let cycle_id = start_cases(&handle, metadata("session.restart", "Pending Restart"), materialize_cases(SOURCE, &suite)).await;
    let receipt = handle.start_next_execution(cycle_id.clone(), CommandContext::default()).await;
    assert_eq!(receipt.status, CommandStatus::Accepted);
    probe.entered.notified().await;

    assert_eq!(save_scan_interval(&handle, 47).await, PluginRestart::Pending);
    assert_eq!(save_scan_interval(&handle, 48).await, PluginRestart::Pending);
    assert_eq!(handle.plugin_settings(&identifier(HOST)).await.unwrap().restart, PluginRestart::Pending);
    assert_eq!(prepared_scan_interval(&owner).await.0, generation, "the running execution keeps its host generation");

    probe.released.send_replace(true);
    wait_for_run_settled(&handle, &receipt.command_id).await;
    handle.plugin_restart_settled(&identifier(HOST)).await;
    let cycle = cycle_snapshot(&handle, &cycle_id).await;
    let execution = &cycle.test_executions[0];
    let log = execution_log(&handle, &execution.test_execution_id).await;
    assert_eq!(execution.status, ExecutionStatus::Passed, "the restart never touched the execution: {log}");
    let (restarted, applied, saved) = prepared_scan_interval(&owner).await;
    assert_ne!(restarted, generation);
    assert_ne!(applied, prepared);
    assert_eq!(applied, saved, "the restart prepared with the latest saved value");
    probe.runtime.lock().unwrap().take();
    drop(handle);
    owner.shutdown().await.unwrap();
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
    let second = installed(&handle, crate::service_support::bundles::host_plugin_version("99.0.0")).await;
    assert!(second.active, "{:?}", second.installation.last_boot);
    assert!(!installation(&handle, first.installation.install_id).await.active, "the replaced installation no longer runs");
    let replaced = running_generation(&owner).await.expect("the new version runs");
    assert_ne!(replaced, started);

    // A Retry Starts Only This Plugin Again.
    let retried = handle.retry_plugin(second.installation.install_id).await.unwrap();
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
    let removed = handle.remove_plugin(second.installation.install_id).await.unwrap().expect("removal is recorded until the next start");
    let removed = settled(&handle, &removed).await;
    assert!(!removed.active);
    assert!(running_generation(&owner).await.is_none() && !admits_host(&handle).await);
    drop(handle);
    owner.shutdown().await.unwrap();
}

// A Replacement While An Execution Captures The Program's Telemetry Waits For It, Then Applies.
pub(crate) async fn replacement_waits_for_execution() {
    let root = tempdir().unwrap();
    let probe = Arc::new(Probe::new(Settlement::Held));
    let (mut owner, handle) = started(config(root.path()).register(Arc::new(Provider(probe.clone())))).await;
    *probe.runtime.lock().unwrap() = Some(handle.clone());
    let first = installed(&handle, host_bundle()).await;
    let generation = running_generation(&owner).await.expect("the installed program runs");
    let suite = handle.test_suites().await.into_iter().find(|suite| suite.source == SOURCE).unwrap().suite;
    let cycle_id = start_cases(&handle, metadata("session.replace", "Pending Replacement"), materialize_cases(SOURCE, &suite)).await;
    let receipt = handle.start_next_execution(cycle_id.clone(), CommandContext::default()).await;
    assert_eq!(receipt.status, CommandStatus::Accepted);
    probe.entered.notified().await;

    let replacement = handle.install_plugin(crate::service_support::bundles::host_plugin_version("99.0.0"), |_| Ok(())).await.unwrap();
    let pending = installation(&handle, replacement.installation.install_id).await;
    assert_eq!(pending.restart, PluginRestart::Pending);
    assert!(!pending.active);
    assert_eq!(handle.plugin_settings(&identifier(HOST)).await.unwrap().restart, PluginRestart::Pending);
    assert!(installation(&handle, first.installation.install_id).await.active, "the previous version keeps running");
    assert_eq!(running_generation(&owner).await, Some(generation), "the running execution keeps its generation");

    probe.released.send_replace(true);
    wait_for_run_settled(&handle, &receipt.command_id).await;
    let applied = settled(&handle, &replacement).await;
    let cycle = cycle_snapshot(&handle, &cycle_id).await;
    let execution = &cycle.test_executions[0];
    let log = execution_log(&handle, &execution.test_execution_id).await;
    assert_eq!(execution.status, ExecutionStatus::Passed, "the replacement never touched the execution: {log}");
    assert!(applied.active && applied.restart == PluginRestart::Idle);
    assert_ne!(running_generation(&owner).await.expect("the new version runs"), generation);
    probe.runtime.lock().unwrap().take();
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
    let installed: Vec<_> = handle.plugin_store().installed().await.unwrap();
    assert_eq!(installed.len(), 1, "a restart runs the kept installation alone");
    drop(handle);
    owner.shutdown().await.unwrap();
}

// A Refused Restart Keeps The Running Generation; Retry Starts One Plugin Again Once The Cause Is Fixed.
pub(crate) async fn retry_restarts_one_plugin() {
    let root = tempdir().unwrap();
    let (mut owner, handle) = started(config(root.path())).await;

    // A Restart That Cannot Read Its Settings Is Refused While The Program Keeps Running.
    let host = installed(&handle, host_bundle()).await;
    let install = host.installation.install_id;
    let generation = running_generation(&owner).await.expect("the installed program runs");
    save_unresolvable(&handle, install, true).await;
    handle.retry_plugin(install).await.unwrap();
    handle.plugin_restart_settled(&identifier(HOST)).await;
    let refused = installation(&handle, install).await;
    assert!(refused.active && !refused.boot_failed, "the previous generation still runs");
    assert!(matches!(refused.installation.last_boot, Some(PluginLastBoot::Failed { .. })));
    assert_eq!(running_generation(&owner).await, Some(generation));
    save_unresolvable(&handle, install, false).await;
    handle.retry_plugin(install).await.unwrap();
    handle.plugin_restart_settled(&identifier(HOST)).await;
    let retried = installation(&handle, install).await;
    assert!(retried.active && matches!(retried.installation.last_boot, Some(PluginLastBoot::Loaded { .. })));
    assert_ne!(running_generation(&owner).await.expect("the retried program runs"), generation);

    // A Plugin That Did Not Start Is Retried In Place: It Reads The Secret Stored Since.
    let failed = installed(&handle, secret_bundle("secret-fixture", "secret_fixture")).await;
    assert!(failed.boot_failed && !failed.active);
    let Some(PluginLastBoot::Failed { failure, .. }) = &failed.installation.last_boot else { panic!("the failed start is recorded") };
    assert_eq!(failure.missing_secret.as_deref(), Some("token"));
    handle
        .operations()
        .call::<dokime_runtime::operations::secrets::Put>(dokime_runtime::operations::secrets::Put {
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
    let retried = handle.retry_plugin(failed.installation.install_id).await.unwrap();
    let retried = settled(&handle, &retried).await;
    let Some(PluginLastBoot::Failed { failure, .. }) = &retried.installation.last_boot else { panic!("the inert program cannot start") };
    assert_eq!(failure.missing_secret, None, "the retry prepared with the stored secret");
    assert!(installation(&handle, install).await.active, "retrying one plugin leaves the others running");
    drop(handle);
    owner.shutdown().await.unwrap();
}
