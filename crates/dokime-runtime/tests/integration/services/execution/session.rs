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
use dokime_runtime::execution::{Caller, ExecutionHost, Runnable, SampleOutput, StepAttemptStart, TerminationReason, TestStepAttempt};
use dokime_runtime::execution::log::{Entry, Level};
use dokime_runtime::transport::{EvidenceBatch, ExecutionSignal, RunExecutionRequest, SidecarCriterionOutcome, SidecarCriterionStatus, SidecarExecutionOutcome, SignalReply, TransportFuture};
use dokime_runtime::{InstalledPlugin, Refusal};
use dokime_runtime::runtime::EmbeddedProvider;

/* === Constants === */
const SOURCE: &str = "fixture.sessions";
const TEMPLATE: &str = "fixture.sessions.cancel";
const PHASE: &str = "fixture.sessions.stream";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct Probe {
    mode: Settlement,
    entered: tokio::sync::Notify,
    cancelled: tokio::sync::Notify,
    released: tokio::sync::watch::Sender<bool>,
    cancellation: tokio::sync::watch::Sender<bool>,
    token: Mutex<String>,
}

struct Provider(Arc<Probe>);

/* === Enums === */
#[derive(Clone, Copy, PartialEq)]
enum Settlement {
    Complete,
    Deferred,
    Refused,
    Ignored,
}

/* === Traits === */

/* === Implementations === */
impl Probe {
    fn new(mode: Settlement) -> Self {
        Self {
            mode,
            entered: tokio::sync::Notify::new(),
            cancelled: tokio::sync::Notify::new(),
            released: tokio::sync::watch::Sender::new(false),
            cancellation: tokio::sync::watch::Sender::new(false),
            token: Mutex::new(String::new()),
        }
    }
}

impl Runnable for Probe {
    fn template_identifier(&self) -> &str { TEMPLATE }

    fn run(&self, request: RunExecutionRequest, host: Arc<dyn ExecutionHost>, _: Option<SampleOutput>, _: Caller) -> TransportFuture<'_, Result<SidecarExecutionOutcome, Refusal>> {
        Box::pin(async move {
            *self.token.lock().unwrap() = request.execution_token;
            let start = StepAttemptStart {
                attempt_id: dokime_runtime::core::StepAttemptId::new(),
                step_id: dokime_runtime::core::StepId::from_seed(PHASE),
                started_at: Timestamp::now_utc(),
            };
            host.push(EvidenceBatch {
                cursor: 1,
                phase: PHASE.into(),
                step_begins: vec![serde_json::to_string(&start).unwrap()],
                ..Default::default()
            }).await.map_err(Refusal::of)?;
            self.entered.notify_one();
            self.cancellation.subscribe().wait_for(|cancelled| *cancelled).await.unwrap();
            if matches!(self.mode, Settlement::Deferred | Settlement::Ignored) {
                self.released.subscribe().wait_for(|released| *released).await.unwrap();
            }
            if self.mode == Settlement::Ignored {
                return Err(Refusal::of("run released after the host settled it"));
            }
            let ended = TestStepAttempt {
                attempt_id: start.attempt_id,
                step_id: start.step_id,
                started_at: start.started_at,
                finished_at: Some(Timestamp::now_utc()),
                termination: TerminationReason::Cancelled,
                detail: None,
            };
            let refused = self.mode == Settlement::Refused;
            host.push(EvidenceBatch {
                cursor: 2,
                step_attempts: vec![serde_json::to_string(&ended).unwrap()],
                diagnostics: if refused { Vec::new() } else { cancellation_log() },
                cleanup_settled: !refused,
                ..Default::default()
            }).await.map_err(Refusal::of)?;
            if refused { return Err(Refusal::of("run refused during cancellation")); }
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

    fn signal(&self, signal: ExecutionSignal) -> Result<SignalReply, Refusal> {
        if signal.cancel {
            if *self.token.lock().unwrap() != signal.execution_token { return Err(Refusal::of("cancellation token does not own execution")); }
            self.cancellation.send_replace(true);
            self.cancelled.notify_one();
        }
        Ok(SignalReply::default())
    }

    fn take_local_samples(&self, _: &str) -> Result<Vec<dokime_runtime::telemetry::TelemetrySample>, Refusal> { Ok(Vec::new()) }
    fn interrupt(&self) { self.cancellation.send_replace(true); self.released.send_replace(true); }
    fn shutdown(&self) -> TransportFuture<'_, ()> { self.interrupt(); Box::pin(async {}) }
}

#[async_trait]
impl EmbeddedProvider for Provider {
    fn catalog(&self) -> InstalledPlugin {
        let mut suite = fixture_suite(&["cancelled"]);
        suite.metadata = metadata(SOURCE, "Session Fixture");
        let template = &mut suite.test_case_templates[0];
        template.metadata = metadata(TEMPLATE, "Cancellation");
        template.phases[0].metadata = metadata(PHASE, "Stream");
        template.phases[0].step_id = dokime_runtime::core::StepId::from_seed(PHASE);
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
    fn streams(&self) -> Vec<StreamDeclaration> { Vec::new() }
    async fn start(&self) -> dokime_runtime::runtime::Result<Vec<Arc<dyn Runnable>>> { Ok(vec![self.0.clone()]) }
    async fn stop(&self) -> dokime_runtime::runtime::Result<()> { self.0.interrupt(); Ok(()) }
}

/* === Public Functions === */

/* === Private Functions === */
fn cancellation_log() -> Vec<Entry> {
    vec![
        Entry::new(Level::Trace, "trace detail"),
        Entry::new(Level::Debug, "debug detail"),
        Entry::new(Level::Info, "fixture cancellation settled"),
        Entry::new(Level::Warn, "warning\ncontinued"),
        Entry::new(Level::Error, "error detail"),
        Entry::new(Level::Info, ""),
        Entry { level: 99, message: "unknown detail".into(), timestamp: dokime_runtime::execution::log::timestamp() },
    ]
}

fn without_timestamps(log: &str) -> String {
    log.lines().map(|line| {
        let (timestamp, message) = line.split_once(' ').unwrap();
        assert_eq!(timestamp.len(), 24);
        assert!(timestamp.ends_with('Z'));
        assert!(Timestamp::new(timestamp).unix_millis().is_some());
        format!("{message}\n")
    }).collect()
}

/* === Tests === */
// The Host Preserves Cancellation Verdicts And All Diagnostic Levels From Its Provider.
#[tokio::test]
async fn cancellation_preserves_verdict_and_log() {
    for mode in [Settlement::Complete, Settlement::Deferred, Settlement::Refused] {
        let root = tempdir().unwrap();
        let probe = Arc::new(Probe::new(mode));
        let provider = Provider(probe.clone());
        let (mut owner, handle) = started(config(root.path()).register(Arc::new(provider))).await;
        let suite = handle.test_suites().await.into_iter().find(|suite| suite.source == SOURCE).unwrap().suite;
        let cycle_id = start_cases(&handle, metadata("session.cancel", "Cancellation"), materialize_cases(SOURCE, &suite), false).await;
        let receipt = handle.run_cycle_cases(cycle_id.clone(), CommandContext::default()).await;
        assert_eq!(receipt.status, CommandStatus::Accepted);
        probe.entered.notified().await;
        let stopped = {
            let handle = handle.clone();
            let cycle_id = cycle_id.clone();
            tokio::spawn(async move { handle.stop_cycle(cycle_id, CommandContext::default()).await.unwrap() })
        };
        probe.cancelled.notified().await;
        if mode == Settlement::Deferred {
            assert_ne!(cycle_snapshot(&handle, &cycle_id).await.lifecycle, CycleStatus::Stopped, "the cycle waits for the provider's outcome");
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
            assert_eq!(criteria[0].status, dokime_runtime::execution::CriterionStatus::Indeterminate);
            assert_eq!(criteria[0].message.as_deref(), Some("cancelled before deciding"));
        }
        let files = handle.execution_files(&execution.test_execution_id).await.unwrap();
        let log = files.iter().find(|file| file.relative_path.ends_with("stream.log")).unwrap();
        let content = handle.execution_file_content(&execution.test_execution_id, log.relative_path.clone(), 0, 4096).await.unwrap().unwrap();
        let log = without_timestamps(&String::from_utf8(content.bytes).unwrap());
        if mode == Settlement::Refused {
            assert!(log.starts_with("[ERROR] "), "{log}");
            assert!(log.contains("run refused during cancellation"), "{log}");
        } else {
            assert_eq!(log, "[TRACE] trace detail\n[DEBUG] debug detail\n[INFO] fixture cancellation settled\n[WARN] warning\n[WARN] continued\n[ERROR] error detail\n[INFO] \n[UNKNOWN(99)] unknown detail\n");
        }
        drop(handle);
        owner.shutdown().await.unwrap();
    }
}

// Thirty Seconds Of Host Settlement Policy Elapse On The Controlled Clock.
#[tokio::test]
async fn ignored_run_keeps_execution_identity() {
    let root = tempdir().unwrap();
    let probe = Arc::new(Probe::new(Settlement::Ignored));
    let (mut owner, handle) = started(config(root.path()).register(Arc::new(Provider(probe.clone())))).await;
    let suite = handle.test_suites().await.into_iter().find(|suite| suite.source == SOURCE).unwrap().suite;
    let cycle_id = start_cases(&handle, metadata("session.ignored", "Ignored Stop"), materialize_cases(SOURCE, &suite), false).await;
    let receipt = handle.run_cycle_cases(cycle_id.clone(), CommandContext::default()).await;
    assert_eq!(receipt.status, CommandStatus::Accepted);
    probe.entered.notified().await;
    let original = cycle_snapshot(&handle, &cycle_id).await.test_executions.remove(0);

    tokio::time::pause();
    let stopping = {
        let handle = handle.clone();
        let cycle_id = cycle_id.clone();
        tokio::spawn(async move { handle.stop_cycle(cycle_id, CommandContext::default()).await.unwrap() })
    };
    probe.cancelled.notified().await;
    tokio::time::advance(Duration::from_secs(31)).await;
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
    let log = files.iter().find(|file| file.relative_path.ends_with("stream.log")).unwrap();
    let log = handle.execution_file_content(&execution.test_execution_id, log.relative_path.clone(), 0, 4096).await.unwrap().unwrap();
    assert!(String::from_utf8(log.bytes).unwrap().contains("the run did not answer its stop within 30 seconds"));
    probe.released.send_replace(true);
    drop(handle);
    owner.shutdown().await.unwrap();
}
