/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

mod artifacts;
mod commands;
pub(crate) mod host;
mod lifecycle;
mod logs;
mod replay;
mod session;
mod supervision;

/* === Re-Exports === */

pub(crate) use host::Sessions;

/* === Imports === */

// Standard Library Imports
use std::{
    collections::BTreeMap,
    sync::Arc,
    time::{Duration, Instant},
};

// Third Party Imports
use harmos::Message;
use harmos::operations::OperationManager;

// Local Imports
use super::provider::{ExecutionFixture, ExecutionFixtureContext, ExecutionFixtureFuture};
use crate::definition::{CriterionDefinition, TestCase};
use crate::domains::{ExecutionProviderTarget, SidecarExecutionBinding};
use crate::execution::ExecutionScope;
use crate::plugin::Diagnostic;
use crate::prompts::PromptRequest;
use crate::telemetry::SamplingScope;
use crate::transport::{
    EvidenceBatch, ExecutionSignal, ProviderCriterionOutcome, ProviderCycleOutcome, ProviderExecutionFailure, ProviderExecutionOutcome,
    ProviderPhaseOutcome, ProviderStatusOutcome, RunExecution, SidecarCriterionOutcome, SidecarCriterionStatus, SidecarExecutionOutcome,
    SignalReply,
};
use dokime_domain::model::execution::RecordScope;
use dokime_domain::model::execution::result::CyclePhase;
use dokime_domain::model::execution::result::CycleView;
use dokime_domain::model::execution::result::ExecutionStatus;
use dokime_domain::model::execution::result::TestExecution;
use dokime_domain::model::identity::Identifier;
use dokime_domain::model::identity::PromptId;
use dokime_domain::model::records::publication::RecordPublishRequest;
use dokime_domain::model::values::Value;
use logs::{DropReason, StreamLog, report_dropped};
use replay::replay_sample_inputs;
use session::Session;
use supervision::{COMMAND_POLL, Concern, Subject, Supervision, Wake};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct TemplateRunExecutor;

#[derive(Clone)]
pub(crate) struct Target {
    operations: OperationManager,
    run: String,
    signal: String,
}

#[derive(Default)]
struct PhaseTracker {
    current: Option<(Identifier, String)>,
    phases: Vec<ProviderPhaseOutcome>,
}

#[derive(Default)]
struct PromptBridge {
    outstanding: BTreeMap<PromptId, [u8; 32]>,
    declared: Vec<crate::prompts::PromptDefinition>,
}

#[derive(Default)]
struct TemplateDeclarations {
    prompts: Vec<crate::prompts::PromptDefinition>,
    samples: Vec<Identifier>,
    records: Vec<Identifier>,
    criteria: Vec<CriterionDefinition>,
    streams: Vec<Identifier>,
}

#[derive(Default)]
struct HostCapture {
    leases: Vec<String>,
}

struct ExecutionCleanup {
    _publication_admission: crate::runtime::ExecutionPublicationAdmission,
    _record_demand: Vec<harmos::sidecar::DemandLease>,
    execution: TestExecution,
    session: Arc<Session>,
    _progress: ProgressCleanup,
    settled: bool,
    stop_deadline: Option<tokio::time::Instant>,
    capture: HostCapture,
}

#[derive(Clone, Default)]
pub(crate) struct Finalization {
    active: Arc<tokio::sync::Mutex<Option<ExecutionCleanup>>>,
}

#[derive(Clone, Default)]
struct StepLeaseBridge {
    held: Arc<tokio::sync::Mutex<BTreeMap<String, String>>>,
}

struct CaseResult {
    execution: ProviderExecutionOutcome,
    cancelled: bool,
}

struct ProgressCleanup {
    registry: Arc<crate::domains::execution::progress::ProgressRegistry>,
    execution: dokime_domain::model::identity::TestExecutionId,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl ExecutionFixture for TemplateRunExecutor {
    fn run_cycle<'a>(
        &'a self, context: ExecutionFixtureContext, cycle: &'a CycleView, target: &'a ExecutionProviderTarget,
    ) -> ExecutionFixtureFuture<'a> {
        Box::pin(async move { run_sidecar_cycle(context, cycle, target).await })
    }
}

impl Target {
    pub(crate) fn of(operations: OperationManager, template: &str) -> Self {
        Self {
            operations,
            run: format!("{template}.run"),
            signal: format!("{template}.signal"),
        }
    }

    async fn run(&self, request: RunExecution) -> Result<SidecarExecutionOutcome, ProviderExecutionFailure> {
        let mut context = harmos::operations::CallContext::default();
        let token = request.execution_token.clone();
        let scope = ExecutionScope::execution(token);
        context.insert_portable(scope).map_err(|error| {
            let message = error.to_string();
            failure("execution_run", message)
        })?;
        let operations = self.operations.with_context(context);
        let encoded = request.encode_to_vec();
        let bytes = operations.call_encoded(&self.run, encoded).await.map_err(|error| sidecar_failure("execution_run", error))?;
        SidecarExecutionOutcome::decode(bytes.as_slice()).map_err(|error| {
            let message = format!("run `{}` answered malformed protobuf: {error}", self.run);
            failure("execution_run", message)
        })
    }

    pub(crate) async fn signal(&self, signal: ExecutionSignal) -> Result<SignalReply, ProviderExecutionFailure> {
        let encoded = signal.encode_to_vec();
        let bytes =
            self.operations.call_encoded(&self.signal, encoded).await.map_err(|error| sidecar_failure("execution_signal", error))?;
        SignalReply::decode(bytes.as_slice()).map_err(|error| {
            let message = format!("signal `{}` answered malformed protobuf: {error}", self.signal);
            failure("execution_signal", message)
        })
    }
}

impl PromptBridge {
    fn new(declared: Vec<crate::prompts::PromptDefinition>) -> Self {
        Self { outstanding: BTreeMap::new(), declared }
    }

    fn admit_replay(&self, prompt: &PromptRequest) -> Result<Option<[u8; 32]>, ProviderExecutionFailure> {
        use sha2::Digest;
        let encoded = serde_json::to_vec(prompt).map_err(|error| {
            let message = error.to_string();
            failure("sidecar_prompt", message)
        })?;
        let fingerprint: [u8; 32] = sha2::Sha256::digest(encoded).into();
        let Some(previous) = self.outstanding.get(&prompt.prompt_id) else {
            return Ok(Some(fingerprint));
        };
        if previous != &fingerprint {
            return Err(failure("sidecar_prompt", "replayed question identity has different values or definition"));
        }
        Ok(None)
    }

    async fn request(
        &mut self, context: &ExecutionFixtureContext, scope: &RecordScope, encoded: &str,
    ) -> Result<PromptId, ProviderExecutionFailure> {
        let mut prompt: PromptRequest = serde_json::from_str(encoded).map_err(|error| {
            let message = format!("the run sent a malformed operator prompt: {error}");
            failure("sidecar_prompt", message)
        })?;
        admit_prompt(&self.declared, &prompt)?;
        // Scope is the host's to assign, exactly as it is for samples and
        // records: a run must not be able to raise a prompt against another
        // artifact's execution.
        prompt.scope = scope.clone();
        let prompt_id = prompt.prompt_id.clone();
        let Some(fingerprint) = self.admit_replay(&prompt)? else {
            return Ok(prompt_id);
        };
        context.request_prompt(prompt).await.map_err(|rejection| {
            let message = format!("the runtime refused an operator prompt: {rejection:?}");
            failure("sidecar_prompt", message)
        })?;
        self.outstanding.insert(prompt_id.clone(), fingerprint);
        Ok(prompt_id)
    }

    async fn settle_on_cancel(&mut self, context: &ExecutionFixtureContext) {
        for prompt_id in self.outstanding.keys().cloned().collect::<Vec<_>>() {
            self.settle_prompt(context, &prompt_id).await;
        }
    }

    async fn settle_prompt(&mut self, context: &ExecutionFixtureContext, prompt_id: &PromptId) {
        // Answered, Or Already Settled By The Stop That Ended Its Owner.
        let runtime = &context.runtime;
        if runtime.prompt_response(prompt_id).await.is_some() || runtime.prompt_settled_at(prompt_id).await.is_some() {
            self.outstanding.remove(prompt_id);
            return;
        }
        // Not Accepted: Nobody Agreed To Anything; The Run Was Stopped.
        let settled =
            runtime.submit_prompt_response(prompt_id.clone(), "cancel".to_owned(), "runtime".to_owned(), BTreeMap::new(), false).await;
        self.outstanding.remove(prompt_id);
        if let Err(detail) = settled
            && !matches!(detail.code(), Some("prompt_already_answered" | "prompt_settled"))
        {
            tracing::warn!(%prompt_id, detail = ?detail, "could not settle an operator prompt on cancellation");
        }
    }
}

impl PhaseTracker {
    pub(super) async fn observe(
        &mut self, context: &ExecutionFixtureContext, scope: &RecordScope, phase: &str, execution: &TestExecution, publish: bool,
    ) -> Result<(), ProviderExecutionFailure> {
        if phase.is_empty() {
            return Ok(());
        }
        let current = self.current.as_ref().is_some_and(|(identifier, _)| {
            let current = identifier.as_str();
            current == phase
        });
        if current {
            return Ok(());
        }

        let identifier = Identifier::new(phase).map_err(|error| {
            let message = format!("sidecar returned invalid phase identifier `{phase}`: {error}");
            failure("sidecar_phase", message)
        })?;
        let template = execution.snapshot.template();
        let step = template.and_then(|template| {
            template.phases.iter().find(|step| {
                let identifier = step.metadata.identifier.as_str();
                identifier == phase
            })
        });
        let label = match step {
            Some(step) => step.metadata.label.clone(),
            None => phase_label(phase),
        };
        if publish {
            update_phase(context, scope, identifier.clone(), label.clone(), "running").await?;
        }
        self.complete_previous(context, scope, publish).await?;
        self.phases.push(ProviderPhaseOutcome {
            identifier: phase.to_owned(),
            label: label.clone(),
            status: "running".to_owned(),
        });
        self.current = Some((identifier, label));
        Ok(())
    }

    async fn complete_previous(
        &mut self, context: &ExecutionFixtureContext, scope: &RecordScope, publish: bool,
    ) -> Result<(), ProviderExecutionFailure> {
        let Some((identifier, label)) = self.current.take() else {
            return Ok(());
        };
        if publish {
            update_phase(context, scope, identifier, label, "passed").await?;
        }
        if let Some(previous) = self.phases.last_mut() {
            previous.status = "passed".to_owned();
        }
        Ok(())
    }

    pub(super) fn finish(&mut self, status: &str) {
        self.current = None;
        if let Some(current) = self.phases.last_mut() {
            current.status = status.to_owned();
        }
    }
}

/* === Public Functions === */

pub(crate) fn with_failed_stage(mut phases: Vec<ProviderPhaseOutcome>, failed_in: Option<&str>) -> Vec<ProviderPhaseOutcome> {
    let Some(stage) = failed_in else {
        return phases;
    };
    if phases.iter().any(|phase| phase.identifier == stage) {
        return phases;
    }
    phases.push(ProviderPhaseOutcome {
        identifier: stage.to_owned(),
        label: phase_label(stage),
        status: "failed".to_owned(),
    });
    phases
}

/* === Private Functions === */

#[allow(clippy::too_many_arguments)]
async fn execute_case(
    context: &ExecutionFixtureContext, cycle: &CycleView, case: &TestCase, execution: &TestExecution, workflow: &str, source: &str,
    operations: OperationManager,
) -> Result<CaseResult, ProviderExecutionFailure> {
    if context.finalization.pending().await {
        return Err(failure("sidecar_cleanup", "previous execution cleanup remains unresolved"));
    }
    let execution_token = execution.test_execution_id.to_string();
    let parameters = execution
        .snapshot
        .parameters()
        .iter()
        .map(|(name, parameter)| {
            let name = name.clone();
            let value = parameter.value.clone();
            (name, value)
        })
        .collect::<BTreeMap<String, Value>>();
    let secrets = match context.runtime.state_read().installed_plugins.get(source) {
        Some(artifact) => {
            let settings = context.runtime.settings_snapshot(artifact).await.map_err(|error| {
                let message = error.to_string();
                failure("sidecar_start", message)
            })?;
            let secrets = context.runtime.resolve_secrets(&artifact.settings, &settings).await.map_err(|error| {
                let message = error.to_string();
                failure("sidecar_start", message)
            })?;
            check_secret_bindings(&artifact.settings, &secrets)?;
            secrets
        }
        None => BTreeMap::new(),
    };
    let request = RunExecution {
        mode: execution.mode.as_str().into(),
        execution_token: execution_token.clone(),
        test_cycle_id: cycle.test_cycle_id.to_string(),
        test_execution_id: execution.test_execution_id.to_string(),
        template_identifier: case.metadata.identifier.to_string(),
        workflow: workflow.to_owned(),
        secrets_json: serde_json::to_string(&secrets)
            .map_err(|_| failure("sidecar_start", "failed to encode execution secret bindings"))?,
        parameters_json: serde_json::to_string(&parameters).map_err(|error| {
            let message = format!("failed to encode resolved case parameters: {error}");
            failure("sidecar_start", message)
        })?,
    };
    let declared = template_declarations(context, case, source).await;
    let publication_scope = execution
        .scope
        .clone()
        .with_cycle(cycle.test_cycle_id.clone())
        .with_test_case(execution.test_case_id.clone())
        .with_test_execution(execution.test_execution_id.clone());
    let publication_admission = context
        .runtime
        .native_executions
        .admit(source, &execution_token, publication_scope, declared.streams.clone())
        .map_err(|error| failure("sidecar_start", error))?;
    let mut phase = PhaseTracker::default();
    phase.observe(context, &execution.scope, "initialization", execution, true).await?;
    context.capture_phase(execution.scope.clone(), dokime_domain::model::execution::CapturePhase::Initialization).await.map_err(
        |error| {
            let message = error.to_string();
            failure("sidecar_capture_phase", message)
        },
    )?;
    let record_demand = match context.runtime.hold_record_demand(&declared.records) {
        Ok(holds) => holds,
        Err(error) => {
            close_record_subscriptions(context, execution).await?;
            let message = error.to_string();
            return Err(failure("sidecar_records", message));
        }
    };
    let mut capture = HostCapture::start(context, execution, &declared.samples, source).await;
    if context.is_cancelled() {
        capture.release(context).await?;
        close_record_subscriptions(context, execution).await?;
        return Err(failure("cancel", "provider invocation cancelled before the run started"));
    }

    let target = Target::of(operations, case.metadata.identifier.as_str());
    let opened = session::Opened {
        context: context.clone(),
        execution: execution.clone(),
        workflow: workflow.to_owned(),
        source: source.to_owned(),
        target: target.clone(),
        prompts: PromptBridge::new(declared.prompts),
        phase,
    };
    let session = Arc::new(Session::new(opened));
    context
        .finalization
        .register(ExecutionCleanup {
            capture,
            _publication_admission: publication_admission,
            _record_demand: record_demand,
            execution: execution.clone(),
            session: Arc::clone(&session),
            _progress: ProgressCleanup {
                registry: Arc::clone(&context.runtime.progress),
                execution: execution.test_execution_id.clone(),
            },
            settled: false,
            stop_deadline: None,
        })
        .await;
    context.runtime.execution_sessions.insert(&execution_token, Arc::clone(&session));

    let provider_started = Instant::now();
    tracing::debug!(execution_id = %execution.test_execution_id, cycle_id = %cycle.test_cycle_id, "execution provider run started");
    let run = target.run(request);
    tokio::pin!(run);
    let mut cancelled = false;
    let mut stop_deadline = None;
    let subject = Subject {
        cycle: &cycle.test_cycle_id,
        request: context.request_id.as_ref(),
    };
    let mut changes = context.runtime.state_watch.subscribe();
    let mut supervision = Supervision::new(Concern::of(changes.borrow_and_update().state(), &subject));
    let mut pending_command = commands::Pending::default();
    let mut session_changes = session.watch_changes();
    let cancellation = context.cancelled();
    tokio::pin!(cancellation);
    let mut initial = true;
    let answered = loop {
        let actionable = initial || (!cancelled && stop_requested(context, &session, supervision.concern()).await);
        let wake = tokio::select! {
            biased;
            answered = &mut run => break Some(answered),
            () = std::future::ready(()), if actionable => Wake::Poll,
            // Commits Are Filtered On The Committed Payload, So An Unrelated Commit Neither Wakes Nor Queries.
            result = changes.wait_for(|snapshot| supervision.concerns(Concern::of(snapshot.state(), &subject))) => match result {
                Ok(snapshot) => Wake::Commit(Concern::of(snapshot.state(), &subject)),
                Err(_) => break None,
            },
            result = session_changes.changed() => match result {
                Ok(()) => Wake::Session { command_admitted: session.take_command_admitted() },
                Err(_) => break None,
            },
            () = &mut cancellation, if !cancelled => Wake::Poll,
            () = tokio::time::sleep(COMMAND_POLL), if supervision.command_open() => Wake::Poll,
            () = async {
                match stop_deadline {
                    Some(deadline) => tokio::time::sleep_until(deadline).await,
                    None => std::future::pending().await,
                }
            } => break None,
        };
        if matches!(wake, Wake::Session { command_admitted: true }) {
            pending_command.invalidate();
        }
        supervision.wake(wake);
        initial = false;
        let expired = stop_deadline.is_some_and(|deadline| {
            let now = tokio::time::Instant::now();
            now >= deadline
        });
        if expired {
            break None;
        }
        let stopping = !cancelled && stop_requested(context, &session, supervision.concern()).await;
        if stopping {
            cancelled = true;
            let deadline = tokio::time::Instant::now() + Duration::from_millis(dokime_domain::STOP_SETTLEMENT_SECONDS * 1000);
            stop_deadline = Some(deadline);
            context.finalization.cancelling(deadline).await;
            let cancel = async {
                // Cancel the pending operator questions before signalling the run.
                session.cancel().await;
                let signal = ExecutionSignal {
                    execution_token: execution_token.clone(),
                    cancel: true,
                    ..ExecutionSignal::default()
                };
                let reply = target.signal(signal).await;
                if let Err(error) = reply {
                    session.refuse(error).await;
                }
            };
            if tokio::time::timeout_at(deadline, cancel).await.is_err() {
                break None;
            }
        }
        if !supervision.command_open() {
            continue;
        }
        let reconcile = commands::reconcile(context, &target, execution, &mut pending_command, false, cancelled);
        let reconciled = match stop_deadline {
            Some(deadline) => match tokio::time::timeout_at(deadline, reconcile).await {
                Ok(result) => result,
                Err(_) => break None,
            },
            None => reconcile.await,
        };
        match reconciled {
            Ok(open) => supervision.reconciled(open),
            Err(error) => session.refuse(error).await,
        }
    };
    context.runtime.execution_sessions.remove(&execution_token);
    // A run can wind down before the watch loop sees its stop, for instance
    // when the stop settled the question it was waiting on. It still ended
    // because it was stopped.
    if !cancelled && (cancellation_requested(context, cycle) || context.execution_stop_requested(&execution.test_execution_id)) {
        cancelled = true;
    }
    let Some(answered) = answered else {
        // The run never answered its cancellation. The host settles it as
        // stopped with the evidence captured so far; cleanup signals the run
        // once more and releases everything the execution held.
        tracing::warn!(execution_id = %execution.test_execution_id,
            budget_s = Duration::from_millis(dokime_domain::STOP_SETTLEMENT_SECONDS * 1000).as_secs(), "cancelled run did not settle; host settled it as stopped");
        let message = format!(
            "the run did not answer its stop within {} seconds; the host settled it as stopped",
            Duration::from_millis(dokime_domain::STOP_SETTLEMENT_SECONDS * 1000).as_secs()
        );
        let refused = failure("stop_settlement", message.clone());
        session.narrate_refusal(&refused).await;
        drop(refused);
        // A late run may close its own step first; cleanup still owns release.
        if let Err(error) = close_stopped_attempt(context, execution).await {
            tracing::warn!(execution_id = %execution.test_execution_id,
                detail = %error.message, "host could not close the stopped step attempt");
        }
        context.finalization.finish(context).await?;
        let outcome = execution_outcome(case, execution, source, "cancelled", Vec::new(), Some(message), Vec::new(), None);
        return Ok(CaseResult { execution: outcome, cancelled: true });
    };
    tracing::debug!(execution_id = %execution.test_execution_id, succeeded = answered.is_ok(),
        elapsed_ms = provider_started.elapsed().as_millis() as u64, "execution provider run settled");
    let concluded = match answered {
        Ok(outcome) => {
            context.finalization.settled().await;
            session.conclude(case, outcome, cancelled, &declared.criteria).await
        }
        Err(error) => Err(error),
    };
    // Whichever failure ends the case is the one its log says, so a run that
    // refused on its own is narrated the same way as a page the host refused.
    let result = match (concluded, session.take_refusal().await) {
        (Ok(_), Some(refusal)) | (Err(_), Some(refusal)) => {
            session.narrate_refusal(&refusal).await;
            Err(refusal)
        }
        (Err(failure), None) => {
            session.narrate_refusal(&failure).await;
            Err(failure)
        }
        (result, None) => result,
    };
    context.finalization.finish(context).await?;
    result
}

async fn run_sidecar_cycle(
    context: ExecutionFixtureContext, cycle: &CycleView, target: &ExecutionProviderTarget,
) -> Result<ProviderCycleOutcome, ProviderExecutionFailure> {
    let SidecarExecutionBinding { workflow } = &target.binding;
    let operations = context.runtime.operations.clone();

    let mut executions = Vec::new();
    let mut failures = Vec::new();
    let mut cancelled = false;
    for execution in runnable_executions(cycle) {
        let case = execution.snapshot.case();
        if cancellation_requested(&context, cycle) {
            cancelled = true;
            executions.push(execution_outcome(
                case,
                execution,
                &target.source,
                "skipped",
                Vec::new(),
                Some("sidecar execution cancelled before dispatch".to_owned()),
                Vec::new(),
                None,
            ));
            continue;
        }
        if context.execution_stop_requested(&execution.test_execution_id) {
            executions.push(execution_outcome(
                case,
                execution,
                &target.source,
                "cancelled",
                Vec::new(),
                Some("execution stopped by the operator before dispatch".to_owned()),
                Vec::new(),
                None,
            ));
            continue;
        }
        let result = execute_case(&context, cycle, case, execution, workflow, &target.source, operations.clone()).await;
        // An operator stop of this one execution is not a cycle stop and not a
        // failure: the siblings behind it still run, even when the stopped run
        // unwound through a refusal rather than an outcome.
        let stopped_alone = context.execution_stop_requested(&execution.test_execution_id);
        let result = match result {
            Ok(result) => result,
            Err(error) if stopped_alone => CaseResult {
                execution: execution_outcome(
                    case,
                    execution,
                    &target.source,
                    "cancelled",
                    Vec::new(),
                    Some(format!("execution stopped by the operator: {}", error.message)),
                    Vec::new(),
                    None,
                ),
                cancelled: true,
            },
            Err(error) => return Err(error),
        };
        cancelled |= result.cancelled && !stopped_alone;
        failures.extend(result.execution.failures.iter().cloned());
        executions.push(result.execution);
    }

    let status = if cancelled {
        "cancelled"
    } else if failures.is_empty() && executions.iter().all(|item| item.status == "passed") {
        "passed"
    } else {
        "failed"
    };
    Ok(ProviderCycleOutcome {
        status: status.to_owned(),
        dependency_imports: Vec::new(),
        provider: ProviderStatusOutcome {
            status: "ready".to_owned(),
            domain_registration_count: 1,
            test_suite_count: 1,
            test_suite_identifiers: vec![workflow.clone()],
            error: None,
        },
        executions,
        failures,
    })
}

fn admit_prompt(declared: &[crate::prompts::PromptDefinition], prompt: &PromptRequest) -> Result<(), ProviderExecutionFailure> {
    if !declared.iter().any(|definition| definition.prompt_type == prompt.prompt_type) {
        return Err(failure("sidecar_prompt", format!("operator question {} is not declared by this template", prompt.prompt_type)));
    }
    Ok(())
}

fn check_secret_bindings(
    catalog: &crate::settings::Settings, secrets: &BTreeMap<String, Option<String>>,
) -> Result<(), ProviderExecutionFailure> {
    let mut definitions = catalog.secrets();
    let missing = definitions.find(|definition| {
        if !definition.required {
            return false;
        }
        let value = secrets.get(&definition.id);
        value.is_none_or(Option::is_none)
    });
    if let Some(definition) = missing {
        let message = format!("required secret setting {:?} is missing", definition.id);
        return Err(failure("sidecar_start", message));
    }
    Ok(())
}

async fn stop_requested(context: &ExecutionFixtureContext, session: &Session, concern: Concern) -> bool {
    session.refused().await || context.is_cancelled() || concern.halts()
}

async fn template_declarations(context: &ExecutionFixtureContext, case: &TestCase, source: &str) -> TemplateDeclarations {
    for snapshot in context.runtime.test_suites().await {
        if snapshot.source != source {
            continue;
        }
        for template in &snapshot.suite.test_case_templates {
            if template.test_case_template_id == case.test_case_template_id {
                return TemplateDeclarations {
                    prompts: template.prompt_definitions.clone(),
                    records: template.subscriptions.iter().filter_map(crate::definition::Subscription::record).cloned().collect(),
                    samples: template.subscriptions.iter().filter_map(crate::definition::Subscription::stream).cloned().collect(),
                    criteria: template.criteria.clone(),
                    streams: snapshot
                        .telemetry_contracts
                        .iter()
                        .filter(|contract| !contract.foreign && contract.template_id.as_ref() == Some(&template.test_case_template_id))
                        .map(|contract| contract.stream.clone())
                        .collect(),
                };
            }
        }
    }
    TemplateDeclarations::default()
}

fn sampling_scope(execution: &TestExecution) -> SamplingScope {
    SamplingScope {
        test_cycle_id: execution.scope.test_cycle_id.as_ref().map(ToString::to_string),
        test_case_id: execution.scope.test_case_id.as_ref().map(ToString::to_string),
        test_execution_id: Some(execution.test_execution_id.to_string()),
    }
}

fn validate_cursor(requested: u64, reply: &EvidenceBatch) -> Result<(), ProviderExecutionFailure> {
    if reply.cursor < requested {
        return Err(failure("sidecar_drain", format!("sidecar drain cursor regressed from {requested} to {}", reply.cursor)));
    }
    let has_output = !reply.phase.is_empty()
        || !reply.step_begins.is_empty()
        || !reply.step_attempts.is_empty()
        || !reply.capture_phases.is_empty()
        || !reply.traceability.is_empty()
        || !reply.documents.is_empty()
        || !reply.records.is_empty()
        || reply.outcome.is_some()
        || !reply.diagnostics.is_empty()
        || !reply.prompt_requests.is_empty()
        || !reply.lease_requests.is_empty()
        || !reply.lease_releases.is_empty()
        || !reply.artifact_chunks.is_empty()
        || reply.progress.is_some();
    if has_output && reply.cursor == requested {
        return Err(failure("sidecar_drain", format!("sidecar returned output without advancing drain cursor {requested}")));
    }
    Ok(())
}

fn publish_progress(
    context: &ExecutionFixtureContext, execution: &TestExecution, reply: &EvidenceBatch,
) -> Result<(), ProviderExecutionFailure> {
    let Some(progress) = reply.progress.clone() else {
        return Ok(());
    };
    if progress.phase.trim().is_empty() {
        return Err(failure("sidecar_progress", "sidecar progress phase must not be empty"));
    }
    if progress.completed > progress.total {
        return Err(failure(
            "sidecar_progress",
            format!("sidecar progress completed {} exceeds total {}", progress.completed, progress.total),
        ));
    }
    let step = Identifier::new(&reply.phase).map_err(|error| {
        let message = format!("sidecar progress has no valid current step: {error}");
        failure("sidecar_progress", message)
    })?;
    let declared =
        execution.snapshot.template().is_some_and(|template| template.phases.iter().any(|phase| phase.metadata.identifier == step));
    if !declared {
        return Err(failure("sidecar_progress", format!("sidecar progress names undeclared current step `{step}`")));
    }
    context.runtime.progress.update(crate::domains::execution::progress::LiveExecutionProgress {
        test_execution_id: execution.test_execution_id.clone(),
        step,
        progress,
    });
    Ok(())
}

async fn publish_records(
    context: &ExecutionFixtureContext, scope: &RecordScope, source: &str, reply: &EvidenceBatch,
) -> Result<(), ProviderExecutionFailure> {
    let records = decode_records(reply, scope)?;
    for record in records {
        context.runtime.publish_sidecar_record(record, source).await.map_err(|error| {
            let message = format!("sidecar `{source}` record was rejected: {error}");
            failure("sidecar_record", message)
        })?;
    }
    Ok(())
}

async fn publish_traceability(
    context: &ExecutionFixtureContext, scope: &RecordScope, source: &str, reply: &EvidenceBatch,
) -> Result<(), ProviderExecutionFailure> {
    if reply.traceability.is_empty() {
        return Ok(());
    }
    let sections = reply
        .traceability
        .iter()
        .map(|encoded| {
            serde_json::from_str::<dokime_domain::model::traceability::TraceabilitySection>(encoded).map_err(|error| {
                let message = format!("sidecar `{source}` traceability section is invalid: {error}");
                failure("sidecar_traceability", message)
            })
        })
        .collect::<Result<Vec<_>, ProviderExecutionFailure>>()?;

    context
        .runtime
        .commit_execution_traceability(crate::traceability_runtime::phase::RecordExecutionTraceability {
            scope: scope.clone(),
            sections,
            occurred_at: crate::model::time::now_utc(),
        })
        .await
        .map_err(|error| {
            let message = format!("sidecar `{source}` traceability was rejected: {}", error);
            failure("sidecar_traceability", message)
        })
}

async fn publish_document_parts(
    context: &ExecutionFixtureContext, scope: &RecordScope, source: &str, reply: &EvidenceBatch,
) -> Result<(), ProviderExecutionFailure> {
    if reply.documents.is_empty() {
        return Ok(());
    }
    let additions = reply
        .documents
        .iter()
        .map(|encoded| {
            serde_json::from_str::<crate::document::DocumentAddition>(encoded).map_err(|error| {
                let message = format!("sidecar `{source}` document addition is invalid: {error}");
                failure("sidecar_document", message)
            })
        })
        .collect::<Result<Vec<_>, ProviderExecutionFailure>>()?;

    context.add_document_parts(scope.clone(), additions).await.map_err(|error| {
        let message = format!("sidecar `{source}` document addition was rejected: {error}");
        failure("sidecar_document", message)
    })
}

fn decode_records(reply: &EvidenceBatch, scope: &RecordScope) -> Result<Vec<RecordPublishRequest>, ProviderExecutionFailure> {
    let mut records = Vec::with_capacity(reply.records.len());
    for encoded in &reply.records {
        let mut value: serde_json::Value = serde_json::from_str(encoded).map_err(|error| {
            let message = format!("sidecar returned malformed record JSON: {error}");
            failure("sidecar_record", message)
        })?;
        ensure_nullable_field(&mut value, "scope");
        let mut record: RecordPublishRequest = serde_json::from_value(value).map_err(|error| {
            let message = format!("sidecar returned an invalid record publish request: {error}");
            failure("sidecar_record", message)
        })?;
        record.scope = Some(scope.clone());
        records.push(record);
    }
    Ok(records)
}

fn ensure_nullable_field(value: &mut serde_json::Value, field: &str) {
    if let Some(object) = value.as_object_mut() {
        object.entry(field.to_owned()).or_insert(serde_json::Value::Null);
    }
}

async fn close_record_subscriptions(context: &ExecutionFixtureContext, execution: &TestExecution) -> Result<(), ProviderExecutionFailure> {
    let test_cycle_id = execution.test_cycle_id.clone();
    context
        .runtime
        .state_manager
        .commit(crate::transactions::CloseRecordSubscriptions {
            test_cycle_id,
            test_execution_id: execution.test_execution_id.clone(),
        })
        .await
        .map_err(|error| {
            let message = error.to_string();
            failure("sidecar_records", message)
        })?;
    Ok(())
}

async fn update_phase(
    context: &ExecutionFixtureContext, scope: &RecordScope, identifier: Identifier, label: String, status: &str,
) -> Result<(), ProviderExecutionFailure> {
    context.update_execution_phase(scope.clone(), identifier, label, status.to_owned()).await.map_err(|error| {
        let message = format!("sidecar phase update was rejected: {error}");
        failure("sidecar_phase", message)
    })
}

// An Execution Stop Reaches The Provider Through Its Request; Only Closing The Cycle Stops The Whole Run.
fn cancellation_requested(context: &ExecutionFixtureContext, cycle: &CycleView) -> bool {
    context.is_cancelled()
        || context.runtime.state_read().cycles.get(&cycle.test_cycle_id).is_none_or(|cycle| cycle.phase != CyclePhase::Active)
}

fn runnable_executions(cycle: &CycleView) -> impl Iterator<Item = &TestExecution> {
    cycle.test_executions.iter().filter(|execution| matches!(execution.status, ExecutionStatus::Pending | ExecutionStatus::Running))
}

fn criterion_outcomes(criteria: &[SidecarCriterionOutcome], declared: &[CriterionDefinition]) -> Vec<ProviderCriterionOutcome> {
    criteria
        .iter()
        .map(|criterion| ProviderCriterionOutcome {
            identifier: criterion.identifier.clone(),
            label: criterion.label.clone(),
            status: match SidecarCriterionStatus::try_from(criterion.status) {
                Ok(SidecarCriterionStatus::Passed) => "passed",
                Ok(SidecarCriterionStatus::Failed) => "failed",
                Ok(SidecarCriterionStatus::Indeterminate) | Ok(SidecarCriterionStatus::Unspecified) | Err(_) => "indeterminate",
            }
            .to_owned(),
            // A sidecar reports what it decided, not what it was asked for.
            // The expectation belongs to the declaration the host installed,
            // so the host supplies it rather than trusting a copy on the wire.
            expected: declared
                .iter()
                .find(|declaration| declaration.metadata.identifier.as_str() == criterion.identifier)
                .and_then(|declaration| declaration.expected.clone()),
            required: Some(true),
            message: criterion.message.clone(),
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn execution_outcome(
    case: &TestCase, execution: &TestExecution, source: &str, status: &str, phases: Vec<ProviderPhaseOutcome>, message: Option<String>,
    criteria: Vec<ProviderCriterionOutcome>, detail: Option<&Diagnostic>,
) -> ProviderExecutionOutcome {
    let failures = if status == "failed" {
        let mut failure = ProviderExecutionFailure {
            stage: "sidecar_execution".to_owned(),
            template_identifier: Some(case.metadata.identifier.to_string()),
            module: Some(source.to_owned()),
            message: message.clone().unwrap_or_else(|| "sidecar execution failed".to_owned()),
            detail: None,
        };
        // `detailed` is what redacts, so the failure that gets written down
        // never names the machine the plugin happened to run on.
        if let Some(diagnostic) = detail {
            failure = failure.detailed(diagnostic);
        }
        vec![failure]
    } else {
        Vec::new()
    };
    ProviderExecutionOutcome {
        case_id: case.test_case_id.to_string(),
        test_execution_id: execution.test_execution_id.to_string(),
        template_identifier: case.metadata.identifier.to_string(),
        label: case.metadata.label.clone(),
        module: source.to_owned(),
        status: status.to_owned(),
        phases,
        records: Vec::new(),
        telemetry: Vec::new(),
        criteria,
        failures,
    }
}

fn failure_detail(outcome: &SidecarExecutionOutcome, source: &str) -> Option<Diagnostic> {
    let encoded = outcome.detail_json.as_deref()?;
    match serde_json::from_str::<Diagnostic>(encoded) {
        Ok(diagnostic) => {
            tracing::error!("{diagnostic}");
            Some(diagnostic)
        }
        Err(error) => {
            tracing::warn!(
                source,
                detail = %error,
                "sidecar execution failure detail could not be decoded"
            );
            None
        }
    }
}

fn phase_label(phase: &str) -> String {
    let mut label = phase
        .split(['.', '_', '-'])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut characters = part.chars();
            match characters.next() {
                Some(first) => first.to_uppercase().collect::<String>() + characters.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ");
    if label.is_empty() {
        label = "Sidecar Execution".to_owned();
    }
    label
}

fn sidecar_failure(stage: &'static str, error: harmos::Error) -> ProviderExecutionFailure {
    use std::error::Error;

    let mut message = error.to_string();
    let mut cause = error.source();
    while let Some(source) = cause {
        message.push_str(": ");
        message.push_str(&source.to_string());
        cause = source.source();
    }
    failure(stage, message)
}

fn failure(stage: impl Into<String>, message: impl Into<String>) -> ProviderExecutionFailure {
    ProviderExecutionFailure::new(stage, message)
}

async fn close_stopped_attempt(context: &ExecutionFixtureContext, execution: &TestExecution) -> Result<(), ProviderExecutionFailure> {
    let execution_id = execution.test_execution_id.clone();
    let active = context
        .runtime
        .state_manager
        .state()
        .read(move |state| state.executions.get(&execution_id).and_then(|execution| execution.active_step_attempt.clone()))
        .await
        .map_err(|error| {
            let message = error.to_string();
            failure("stop_settlement", message)
        })?
        .value;
    let Some(start) = active else {
        return Ok(());
    };
    let attempt = dokime_domain::model::execution::result::TestStepAttempt {
        attempt_id: start.attempt_id,
        step_id: start.step_id,
        started_at: start.started_at,
        finished_at: Some(crate::model::time::now_utc()),
        termination: dokime_domain::model::execution::termination::TerminationReason::Cancelled,
        detail: None,
    };
    context.record_step_attempt(execution.scope.clone(), attempt).await.map_err(|error| {
        let message = error.to_string();
        failure("stop_settlement", message)
    })
}

/* === Tests === */
