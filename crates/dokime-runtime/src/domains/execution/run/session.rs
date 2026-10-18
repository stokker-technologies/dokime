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
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
// Third Party Imports
// Local Imports
use super::artifacts::ArtifactBridge;
use super::{
    CaseResult, DropReason, ExecutionFixtureContext, PhaseTracker, PromptBridge, StepLeaseBridge, StreamLog, Target,
    close_record_subscriptions, commands, criterion_outcomes, execution_outcome, failure, failure_detail, publish_document_parts,
    publish_progress, publish_records, publish_traceability, report_dropped, validate_cursor, with_failed_stage,
};
use crate::definition::{CriterionDefinition, TestCase};
use crate::transport::{
    EvidenceBatch, EvidenceReceipt, ExecutionSignal, LeaseAsk, LeaseResolution, OperatorAnswer, OperatorQuestion, ProviderExecutionFailure,
    SidecarExecutionOutcome, StepAdmission,
};
use dokime_domain::model::execution::result::TestExecution;

/* === Constants === */
const ANSWER_POLL: Duration = Duration::from_millis(100);

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub(super) struct Opened {
    pub context: ExecutionFixtureContext,
    pub execution: TestExecution,
    pub workflow: String,
    pub source: String,
    pub target: Target,
    pub prompts: PromptBridge,
    pub record_subscription_after: u64,
    pub phase: PhaseTracker,
}

pub(crate) struct Session {
    context: ExecutionFixtureContext,
    execution: TestExecution,
    workflow: String,
    source: String,
    pub(super) target: Target,
    pub(super) prompts: tokio::sync::Mutex<PromptBridge>,
    pub(super) step_leases: StepLeaseBridge,
    record_subscription_after: u64,
    pages: tokio::sync::Mutex<Pages>,
    replayed: tokio::sync::Mutex<bool>,
    cancelling: AtomicBool,
    refusal: tokio::sync::Mutex<Option<ProviderExecutionFailure>>,
}

struct Pages {
    cursor: u64,
    settled: bool,
    replayed: BTreeMap<harmos::Identifier, u64>,
    stream_log: StreamLog,
    artifacts: ArtifactBridge,
    phase: PhaseTracker,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Session {
    pub(super) fn new(opened: Opened) -> Self {
        let spill = opened.context.runtime.config().storage_root.clone();
        Self {
            context: opened.context,
            execution: opened.execution,
            workflow: opened.workflow,
            source: opened.source,
            target: opened.target,
            prompts: tokio::sync::Mutex::new(opened.prompts),
            step_leases: StepLeaseBridge::default(),
            record_subscription_after: opened.record_subscription_after,
            pages: tokio::sync::Mutex::new(Pages {
                cursor: 0,
                settled: false,
                replayed: BTreeMap::new(),
                stream_log: StreamLog::default(),
                artifacts: ArtifactBridge::new(spill),
                phase: opened.phase,
            }),
            replayed: tokio::sync::Mutex::new(false),
            cancelling: AtomicBool::new(false),
            refusal: tokio::sync::Mutex::new(None),
        }
    }

    pub(crate) fn source(&self) -> &str {
        &self.source
    }

    pub(crate) async fn accept(&self, batch: EvidenceBatch) -> EvidenceReceipt {
        self.replay_once().await;
        let mut pages = self.pages.lock().await;
        let settling = batch.record_subscriptions_settling && !pages.settled;
        if batch.cursor <= pages.cursor && !settling {
            // The run repeated a page whose receipt it never saw.
            return EvidenceReceipt { admitted: Vec::new(), settled: pages.settled };
        }
        let begins = batch.step_begins.clone();
        // Once one page was refused the case is over: nothing later is
        // admitted as evidence, every step it asks to begin is refused, and
        // the run is released to wind down.
        let admitted = if self.refused().await {
            Err(failure("sidecar_evidence", "the case already ended on a refused page"))
        } else {
            self.admit(&mut pages, batch).await
        };
        match admitted {
            Ok(receipt) => receipt,
            Err(refusal) => {
                let message = refusal.message.clone();
                if !self.refused().await {
                    self.release_run();
                }
                self.refuse(refusal).await;
                EvidenceReceipt {
                    admitted: begins
                        .iter()
                        .filter_map(|encoded| {
                            serde_json::from_str::<dokime_domain::model::execution::result::StepAttemptStart>(encoded).ok()
                        })
                        .map(|start| StepAdmission { attempt_id: start.attempt_id.to_string(), accepted: false, message: message.clone() })
                        .collect(),
                    settled: false,
                }
            }
        }
    }

    pub(crate) async fn ask(&self, question: OperatorQuestion) -> Result<OperatorAnswer, String> {
        let prompt_id = self
            .prompts
            .lock()
            .await
            .request(&self.context, &self.execution.scope, &question.prompt_json)
            .await
            .map_err(|refusal| refusal.message)?;
        loop {
            if let Some(response) = self.context.runtime.prompt_response(&prompt_id).await {
                self.prompts.lock().await.outstanding.remove(&prompt_id);
                return serde_json::to_string(&response)
                    .map(|response_json| OperatorAnswer { response_json })
                    .map_err(|error| error.to_string());
            }
            // A stop settles the question in its own transaction, before the
            // watch loop cancels the run: the run is being stopped, so the
            // question ends as a cancellation rather than a refused answer.
            let settled = self.context.runtime.prompt_settled_at(&prompt_id).await;
            if settled.is_some() {
                self.prompts.lock().await.outstanding.remove(&prompt_id);
            }
            if self.cancelling.load(Ordering::Acquire) || settled.is_some() {
                return Err("the execution was stopped while the question was outstanding".into());
            }
            tokio::time::sleep(ANSWER_POLL).await;
        }
    }

    pub(crate) async fn lease(&self, ask: LeaseAsk) -> LeaseResolution {
        match serde_json::from_str::<crate::telemetry::LeaseRequest>(&ask.lease_json) {
            Ok(request) => self.step_leases.request(&self.context, &self.execution, &self.source, request).await,
            Err(error) => LeaseResolution { granted: false, message: format!("the lease request could not be read: {error}") },
        }
    }

    pub(super) async fn cancel(&self) {
        self.cancelling.store(true, Ordering::Release);
        self.prompts.lock().await.settle_on_cancel(&self.context).await;
    }

    pub(super) async fn refused(&self) -> bool {
        self.refusal.lock().await.is_some()
    }

    fn release_run(&self) {
        let target = self.target.clone();
        let execution_token = self.execution.test_execution_id.to_string();
        tokio::spawn(async move {
            let cancelled = target.signal(ExecutionSignal { execution_token, cancel: true, ..ExecutionSignal::default() }).await;
            if let Err(error) = cancelled {
                tracing::debug!(detail = %error.message, "a refused run did not take its cancellation");
            }
        });
    }

    pub(super) async fn refuse(&self, refusal: ProviderExecutionFailure) {
        let mut kept = self.refusal.lock().await;
        if kept.is_none() {
            *kept = Some(refusal);
        }
    }

    pub(super) async fn take_refusal(&self) -> Option<ProviderExecutionFailure> {
        self.refusal.lock().await.take()
    }

    pub(super) async fn narrate_refusal(&self, refusal: &ProviderExecutionFailure) {
        let mut pages = self.pages.lock().await;
        let stage = pages.phase.current.as_ref().map(|(id, _)| id.to_string()).unwrap_or_else(|| "initialization".to_owned());
        pages
            .stream_log
            .append(
                &self.context,
                &self.execution,
                &self.workflow,
                &self.source,
                &[crate::execution::log::Entry::new(
                    crate::execution::log::Level::Error,
                    format!("execution failed during {stage} ({}): {}", refusal.stage, refusal.message),
                )],
            )
            .await;
    }

    pub(super) async fn conclude(
        &self, case: &TestCase, outcome: SidecarExecutionOutcome, cancelled: bool, criteria: &[CriterionDefinition],
    ) -> Result<CaseResult, ProviderExecutionFailure> {
        let reconciled = commands::reconcile(&self.context, &self.target, &self.execution, true, cancelled).await;
        match reconciled {
            Err(error) if !cancelled => return Err(error),
            Err(error) => tracing::warn!(detail = %error.message,
                "cancelled command receipt did not reconcile"),
            Ok(()) => {}
        }
        let mut pages = self.pages.lock().await;
        let pages = &mut *pages;
        let discarded = pages.artifacts.finish().await?;
        report_dropped(
            &self.context,
            &self.execution,
            &self.workflow,
            &self.source,
            &mut pages.stream_log,
            &discarded,
            DropReason::Incomplete,
        )
        .await;
        let decided = criterion_outcomes(&outcome.criteria, criteria);
        if cancelled {
            // A cancelled run still has a verdict to give: its criteria are
            // indeterminate, and that is exactly the state worth recording.
            let status = if outcome.passed { "passed" } else { "cancelled" };
            pages.phase.finish(status);
            return Ok(CaseResult {
                execution: execution_outcome(
                    case,
                    &self.execution,
                    &self.source,
                    status,
                    std::mem::take(&mut pages.phase.phases),
                    outcome.message.or_else(|| Some("execution cancelled by the host".to_owned())),
                    decided,
                    None,
                ),
                cancelled: !outcome.passed,
            });
        }
        let status = if outcome.passed { "passed" } else { "failed" };
        let detail = (!outcome.passed).then(|| failure_detail(&outcome, &self.source)).flatten();
        pages.phase.finish(status);
        let phases = with_failed_stage(std::mem::take(&mut pages.phase.phases), outcome.failed_in.as_deref());
        Ok(CaseResult {
            execution: execution_outcome(case, &self.execution, &self.source, status, phases, outcome.message, decided, detail.as_ref()),
            cancelled: false,
        })
    }

    async fn admit(&self, pages: &mut Pages, mut batch: EvidenceBatch) -> Result<EvidenceReceipt, ProviderExecutionFailure> {
        let context = &self.context;
        let execution = &self.execution;
        let cancelling = self.cancelling.load(Ordering::Acquire);
        validate_cursor(pages.cursor, &batch)?;
        // Rows the run published reach the host on their own calls, ahead of
        // this page; they are stamped with the phase current when they are
        // ingested. A page that moves the phase therefore waits for every row
        // already accepted, so a row is never filed under the phase after the
        // one that produced it.
        if !batch.capture_phases.is_empty() || !batch.step_begins.is_empty() || !batch.step_attempts.is_empty() {
            context.runtime.flush_plugin_publications().await.map_err(|error| failure("sidecar_samples", error.to_string()))?;
        }
        pages.stream_log.append(context, execution, &self.workflow, &self.source, &batch.diagnostics).await;
        let abandoned = pages.artifacts.accept(context, &execution.scope, std::mem::take(&mut batch.artifact_chunks)).await?;
        report_dropped(context, execution, &self.workflow, &self.source, &mut pages.stream_log, &abandoned, DropReason::Abandoned).await;
        // A cycle that is stopping admits no live phase transactions; the
        // final observations are kept for the provider completion fold. A stop
        // can land before this session learns of it, so a phase update the
        // stopped cycle refused is the stop, not a failure of the run.
        let observed =
            pages.phase.observe(context, &execution.scope, &batch.phase, execution, !cancelling && !context.is_cancelled()).await;
        if let Err(refusal) = observed
            && !(context.is_cancelled() || self.cancelling.load(Ordering::Acquire))
        {
            return Err(refusal);
        }
        let admitted = self.admit_steps(&batch).await?;
        if !cancelling {
            publish_progress(context, execution, &batch)?;
        }
        if let Err(refusal) = publish_records(context, &execution.scope, &self.source, &batch).await {
            if !cancelling {
                return Err(refusal);
            }
            pages
                .stream_log
                .append(
                    context,
                    execution,
                    &self.workflow,
                    &self.source,
                    &[crate::execution::log::Entry::new(crate::execution::log::Level::Error, refusal.message)],
                )
                .await;
        }
        publish_traceability(context, &execution.scope, &self.source, &batch).await?;
        publish_document_parts(context, &execution.scope, &self.source, &batch).await?;
        {
            let mut prompts = self.prompts.lock().await;
            for encoded in &batch.prompt_requests {
                prompts.request(context, &execution.scope, encoded).await?;
            }
        }
        self.step_leases.release(context, &batch.lease_releases).await?;
        if batch.record_subscriptions_settling && !pages.settled {
            self.settle_records(Some(&mut pages.replayed)).await?;
            pages.settled = true;
        }
        pages.cursor = pages.cursor.max(batch.cursor);
        Ok(EvidenceReceipt { admitted, settled: pages.settled })
    }

    async fn admit_steps(&self, batch: &EvidenceBatch) -> Result<Vec<StepAdmission>, ProviderExecutionFailure> {
        let context = &self.context;
        let execution = &self.execution;
        for encoded in &batch.capture_phases {
            let phase = serde_json::from_str(encoded)
                .map_err(|error| failure("sidecar_capture_phase", format!("invalid capture phase: {error}")))?;
            context
                .capture_phase(execution.scope.clone(), phase)
                .await
                .map_err(|error| failure("sidecar_capture_phase", error.to_string()))?;
        }
        let mut admitted = Vec::new();
        for encoded in &batch.step_begins {
            let start: dokime_domain::model::execution::result::StepAttemptStart = serde_json::from_str(encoded)
                .map_err(|error| failure("sidecar_step_admission", format!("invalid step admission: {error}")))?;
            context.runtime.wait_if_paused(&execution.test_cycle_id).await;
            let result = context.begin_step_attempt(execution.scope.clone(), start.clone()).await;
            admitted.push(StepAdmission {
                attempt_id: start.attempt_id.to_string(),
                accepted: result.is_ok(),
                message: result.as_ref().err().map(ToString::to_string).unwrap_or_default(),
            });
        }
        for encoded in &batch.step_attempts {
            let attempt =
                serde_json::from_str(encoded).map_err(|error| failure("sidecar_step_attempt", format!("invalid step attempt: {error}")))?;
            context
                .record_step_attempt(execution.scope.clone(), attempt)
                .await
                .map_err(|error| failure("sidecar_step_attempt", error.to_string()))?;
        }
        Ok(admitted)
    }

    async fn replay_once(&self) {
        let mut replayed = self.replayed.lock().await;
        if *replayed {
            return;
        }
        *replayed = true;
        if let Err(refusal) = self.settle_records(None).await {
            self.refuse(refusal).await;
        }
    }

    async fn settle_records(&self, replayed: Option<&mut BTreeMap<harmos::Identifier, u64>>) -> Result<(), ProviderExecutionFailure> {
        let diagnostic_started = std::time::Instant::now();
        let close = replayed.is_some();
        let context = &self.context;
        let execution = &self.execution;
        eprintln!("settlement diagnostic {}: host begins, close={close}", execution.test_execution_id);
        context.runtime.flush_plugin_publications().await.map_err(|error| failure("sidecar_records", error.to_string()))?;
        eprintln!("settlement diagnostic {}: publications flushed at {:?}", execution.test_execution_id, diagnostic_started.elapsed());
        let cycle_id = execution.test_cycle_id.clone();
        let execution_id = execution.test_execution_id.clone();
        if close {
            close_record_subscriptions(context, execution).await?;
            eprintln!("settlement diagnostic {}: subscriptions closed at {:?}", execution.test_execution_id, diagnostic_started.elapsed());
        }
        let ids = context
            .runtime
            .state_manager
            .state()
            .read(move |state| {
                state
                    .executions
                    .get(&execution_id)
                    .filter(|execution| execution.test_cycle_id == cycle_id)
                    .map(|execution| execution.records.subscribed_records().clone())
            })
            .await
            .map_err(|error| failure("sidecar_records", error.to_string()))?
            .value
            .ok_or_else(|| failure("sidecar_records", "record subscription execution disappeared"))?;
        let through = context.runtime.state_manager.applied().0;
        context
            .runtime
            .record_source
            .wait_through(&context.runtime.state_manager, through)
            .await
            .map_err(|error| failure("sidecar_records", error.to_string()))?;
        eprintln!("settlement diagnostic {}: record source caught up at {:?}", execution.test_execution_id, diagnostic_started.elapsed());
        let mut remaining = ids;
        if !remaining.is_empty() {
            let mut reader = context
                .runtime
                .record_source
                .history(&context.runtime.state_manager, 0, through)
                .await
                .map_err(|error| failure("sidecar_records", error.to_string()))?;
            loop {
                let record = reader.next().await.map_err(|error| failure("sidecar_records", error.to_string()))?;
                let Some(record) = record else {
                    break;
                };
                if record.sequence <= self.record_subscription_after {
                    continue;
                }
                let Some(captured) = remaining.remove(&record.record_id) else {
                    continue;
                };
                let record_json = serde_json::to_string(&record.record).map_err(|error| failure("sidecar_records", error.to_string()))?;
                self.target
                    .signal(ExecutionSignal {
                        execution_token: execution.test_execution_id.to_string(),
                        record: Some(crate::transport::DeliverRecordRequest {
                            record_json,
                            captured_scopes: vec![serde_json::to_string(&captured).expect("captured scope serializes")],
                        }),
                        ..ExecutionSignal::default()
                    })
                    .await?;
                if remaining.is_empty() {
                    break;
                }
            }
            if !remaining.is_empty() {
                return Err(failure("sidecar_records", "captured subscription records are missing from the state history"));
            }
        }
        if let Some(replayed) = replayed {
            super::replay_sample_inputs(context, &self.target, execution, replayed).await?;
        }
        eprintln!("settlement diagnostic {}: host settled at {:?}", execution.test_execution_id, diagnostic_started.elapsed());
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
