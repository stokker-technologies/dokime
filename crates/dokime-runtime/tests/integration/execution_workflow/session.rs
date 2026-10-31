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
use dokime_runtime::execution::log::{Entry, Level};
use dokime_runtime::runtime::EmbeddedProvider;
use dokime_runtime::transport::{
    EvidenceBatch, ExecutionSignal, RunExecution, SidecarCriterionOutcome, SidecarCriterionStatus, SidecarExecutionOutcome, SignalReply,
    TransportFuture,
};
use dokime_runtime::{InstalledPlugin, Refusal};
use {
    dokime_domain::model::execution::result::StepAttemptStart, dokime_domain::model::execution::result::TestStepAttempt,
    dokime_domain::model::execution::termination::TerminationReason, dokime_runtime::execution::Caller,
    dokime_runtime::execution::ExecutionHost, dokime_runtime::execution::Runnable, dokime_runtime::execution::SampleOutput,
};

/* === Constants === */
const SOURCE: &str = "fixture_sessions";
const TEMPLATE: &str = "fixture.sessions.cancel";
const PHASE: &str = "fixture.sessions.stream";

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
            if self.mode == Settlement::LiveReplay {
                self.row_received.notified().await;
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
