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
mod session;

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
use artifacts::DiscardedArtifact;
use dokime_domain::model::execution::RecordScope;
use dokime_domain::model::execution::result::CycleView;
use dokime_domain::model::execution::result::ExecutionStatus;
use dokime_domain::model::execution::result::TestExecution;
use dokime_domain::model::identity::Identifier;
use dokime_domain::model::identity::PromptId;
use dokime_domain::model::records::publication::RecordPublishRequest;
use dokime_domain::model::values::Value;
use session::Session;

/* === Constants === */
const WATCH_INTERVAL: Duration = Duration::from_millis(100);
/// How long a cancelled run may take to answer before the host settles the
/// execution as stopped without it. Stop stays cooperative inside this budget.
const STOP_SETTLEMENT: Duration = Duration::from_secs(30);
/// Encoded sample bytes one replay signal carries before the next begins.
const REPLAY_SIGNAL_BYTES: usize = crate::transport::MAX_ARTIFACT_CHUNK_BYTES;

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

#[derive(Default)]
struct StreamLog {
    failed: bool,
}

struct ProgressCleanup {
    registry: Arc<crate::domains::execution::progress::ProgressRegistry>,
    execution: dokime_domain::model::identity::TestExecutionId,
}

/* === Enums === */
#[derive(Clone, Copy, Debug)]
enum DropReason {
    Abandoned,
    Incomplete,
}

/* === Traits === */

/* === Implementations === */
impl ExecutionFixture for TemplateRunExecutor {
    fn run_cycle<'a>(
        &'a self, context: ExecutionFixtureContext, cycle: &'a CycleView, target: &'a ExecutionProviderTarget,
    ) -> ExecutionFixtureFuture<'a> {
        Box::pin(async move { run_sidecar_cycle(context, cycle, target).await })
    }
}

impl Drop for ProgressCleanup {
    fn drop(&mut self) {
        self.registry.clear(&self.execution);
    }
}

impl Target {
    pub(crate) fn of(operations: OperationManager, template: &str) -> Self {
        Self { operations, run: format!("{template}.run"), signal: format!("{template}.signal") }
    }

    async fn run(&self, request: RunExecution) -> Result<SidecarExecutionOutcome, ProviderExecutionFailure> {
        let mut context = harmos::operations::CallContext::default();
        context
            .insert_portable(ExecutionScope::execution(request.execution_token.clone()))
            .map_err(|error| failure("execution_run", error.to_string()))?;
        let bytes = self
            .operations
            .with_context(context)
            .call_encoded(&self.run, request.encode_to_vec())
            .await
            .map_err(|error| sidecar_failure("execution_run", error))?;
        SidecarExecutionOutcome::decode(bytes.as_slice())
            .map_err(|error| failure("execution_run", format!("run `{}` answered malformed protobuf: {error}", self.run)))
    }

    pub(crate) async fn signal(&self, signal: ExecutionSignal) -> Result<SignalReply, ProviderExecutionFailure> {
        let bytes = self
            .operations
            .call_encoded(&self.signal, signal.encode_to_vec())
            .await
            .map_err(|error| sidecar_failure("execution_signal", error))?;
        SignalReply::decode(bytes.as_slice())
            .map_err(|error| failure("execution_signal", format!("signal `{}` answered malformed protobuf: {error}", self.signal)))
    }
}

impl PromptBridge {
    fn new(declared: Vec<crate::prompts::PromptDefinition>) -> Self {
        Self { outstanding: BTreeMap::new(), declared }
    }

    fn admit_replay(&self, prompt: &PromptRequest) -> Result<Option<[u8; 32]>, ProviderExecutionFailure> {
        use sha2::Digest;
        let encoded = serde_json::to_vec(prompt).map_err(|error| failure("sidecar_prompt", error.to_string()))?;
        let fingerprint: [u8; 32] = sha2::Sha256::digest(encoded).into();
        if let Some(previous) = self.outstanding.get(&prompt.prompt_id) {
            if previous != &fingerprint {
                return Err(failure("sidecar_prompt", "replayed question identity has different values or definition"));
            }
            return Ok(None);
        }
        Ok(Some(fingerprint))
    }

    async fn request(
        &mut self, context: &ExecutionFixtureContext, scope: &RecordScope, encoded: &str,
    ) -> Result<PromptId, ProviderExecutionFailure> {
        let mut prompt: PromptRequest = serde_json::from_str(encoded)
            .map_err(|error| failure("sidecar_prompt", format!("the run sent a malformed operator prompt: {error}")))?;
        admit_prompt(&self.declared, &prompt)?;
        // Scope is the host's to assign, exactly as it is for samples and
        // records: a run must not be able to raise a prompt against another
        // artifact's execution.
        prompt.scope = scope.clone();
        let prompt_id = prompt.prompt_id.clone();
        let Some(fingerprint) = self.admit_replay(&prompt)? else {
            return Ok(prompt_id);
        };
        context
            .request_prompt(prompt)
            .await
            .map_err(|rejection| failure("sidecar_prompt", format!("the runtime refused an operator prompt: {rejection:?}")))?;
        self.outstanding.insert(prompt_id.clone(), fingerprint);
        Ok(prompt_id)
    }

    async fn settle_on_cancel(&mut self, context: &ExecutionFixtureContext) {
        for prompt_id in self.outstanding.keys().cloned().collect::<Vec<_>>() {
            // Answered, or already settled by the stop that ended its owner.
            let runtime = &context.runtime;
            if runtime.prompt_response(&prompt_id).await.is_some() || runtime.prompt_settled_at(&prompt_id).await.is_some() {
                self.outstanding.remove(&prompt_id);
                continue;
            }
            // Not accepted: nobody agreed to anything, the run was stopped.
            let settled = context
                .runtime
                .submit_prompt_response(prompt_id.clone(), "cancel".to_owned(), "runtime".to_owned(), BTreeMap::new(), false)
                .await;
            self.outstanding.remove(&prompt_id);
            if let Err(detail) = settled
                && !matches!(detail.code(), Some("prompt_already_answered" | "prompt_settled"))
            {
                tracing::warn!(
                    %prompt_id,
                    detail = ?detail,
                    "could not settle an operator prompt on cancellation"
                );
            }
        }
    }
}

impl Finalization {
    pub(crate) async fn pending(&self) -> bool {
        self.active.lock().await.is_some()
    }

    async fn register(&self, cleanup: ExecutionCleanup) {
        let mut active = self.active.lock().await;
        assert!(active.is_none(), "previous execution cleanup remains owned");
        *active = Some(cleanup);
    }

    async fn cancelling(&self, deadline: tokio::time::Instant) {
        if let Some(cleanup) = self.active.lock().await.as_mut() {
            cleanup.stop_deadline = Some(deadline);
        }
    }

    async fn settled(&self) {
        if let Some(cleanup) = self.active.lock().await.as_mut() {
            cleanup.settled = true;
        }
    }

    pub(crate) async fn finish(&self, context: &ExecutionFixtureContext) -> Result<(), ProviderExecutionFailure> {
        let mut active = self.active.lock().await;
        if let Some(cleanup) = active.as_mut() {
            cleanup.finish(context).await?;
        }
        *active = None;
        Ok(())
    }
}

impl ExecutionCleanup {
    async fn finish(&mut self, context: &ExecutionFixtureContext) -> Result<(), ProviderExecutionFailure> {
        if !self.settled {
            self.session.prompts.lock().await.settle_on_cancel(context).await;
            let deadline = *self.stop_deadline.get_or_insert_with(|| tokio::time::Instant::now() + STOP_SETTLEMENT);
            let signal = self.session.target.signal(ExecutionSignal {
                execution_token: self.execution.test_execution_id.to_string(),
                cancel: true,
                ..ExecutionSignal::default()
            });
            match tokio::time::timeout_at(deadline, signal).await {
                Ok(Ok(_)) => {}
                Ok(Err(error)) => {
                    tracing::debug!(detail = %error.message, "an unsettled run did not take its cancellation");
                }
                Err(_) => {
                    tracing::debug!("an unsettled run exhausted its cancellation budget");
                }
            }
            self.settled = true;
        }
        let publication_started = Instant::now();
        tracing::debug!(execution_id = %self.execution.test_execution_id, "execution publication flush started");
        context.runtime.flush_plugin_publications().await.map_err(|error| failure("sidecar_telemetry", error.to_string()))?;
        tracing::debug!(execution_id = %self.execution.test_execution_id,
            elapsed_ms = publication_started.elapsed().as_millis() as u64, "execution publication flush settled");
        close_record_subscriptions(context, &self.execution).await?;
        self.session.prompts.lock().await.settle_on_cancel(context).await;
        let capture_started = Instant::now();
        tracing::debug!(execution_id = %self.execution.test_execution_id, "execution capture release started");
        self.session.step_leases.release_all(context).await?;
        self.capture.release(context).await?;
        tracing::debug!(execution_id = %self.execution.test_execution_id,
            elapsed_ms = capture_started.elapsed().as_millis() as u64, "execution capture release settled");
        Ok(())
    }
}

impl StepLeaseBridge {
    async fn request(
        &self, context: &ExecutionFixtureContext, execution: &TestExecution, source: &str, request: crate::telemetry::LeaseRequest,
    ) -> crate::transport::LeaseResolution {
        let answer = context.runtime.lease_streams(sampling_scope(execution), std::slice::from_ref(&request.stream), source).await;
        let (granted, message) = match answer {
            Ok(leases) => match leases.first() {
                Some(lease_id) => {
                    self.held.lock().await.insert(request.lease_token.clone(), lease_id.clone());
                    (true, String::new())
                }
                // A grant with no hold behind it would be the worst of both: a
                // step that believes it is capturing and a stream nobody
                // started.
                None => (false, format!("the host took no hold on {}", request.stream)),
            },
            Err(error) => (false, error.to_string()),
        };
        crate::transport::LeaseResolution { granted, message }
    }

    async fn release(&self, context: &ExecutionFixtureContext, released: &[String]) -> Result<(), ProviderExecutionFailure> {
        if released.is_empty() {
            return Ok(());
        }
        let held = self.held.lock().await;
        let leases = released.iter().filter_map(|token| held.get(token).map(|lease| (token.clone(), lease.clone()))).collect::<Vec<_>>();
        drop(held);
        self.release_owned(context, leases).await
    }

    async fn release_all(&self, context: &ExecutionFixtureContext) -> Result<(), ProviderExecutionFailure> {
        let leases = self.held.lock().await.iter().map(|(token, lease)| (token.clone(), lease.clone())).collect::<Vec<_>>();
        self.release_owned(context, leases).await
    }

    async fn release_owned(
        &self, context: &ExecutionFixtureContext, leases: Vec<(String, String)>,
    ) -> Result<(), ProviderExecutionFailure> {
        if leases.is_empty() {
            return Ok(());
        }
        let lease_ids = leases.iter().map(|(_, lease)| lease.clone()).collect();
        context.runtime.release_stream_leases(lease_ids).await.map_err(|error| failure("sidecar_leases", error.to_string()))?;
        let mut held = self.held.lock().await;
        for (token, lease) in leases {
            if held.get(&token) == Some(&lease) {
                held.remove(&token);
            }
        }
        Ok(())
    }
}

impl PhaseTracker {
    pub(super) async fn observe(
        &mut self, context: &ExecutionFixtureContext, scope: &RecordScope, phase: &str, execution: &TestExecution, publish: bool,
    ) -> Result<(), ProviderExecutionFailure> {
        if phase.is_empty() || self.current.as_ref().is_some_and(|(identifier, _)| identifier.as_str() == phase) {
            return Ok(());
        }

        let identifier = Identifier::new(phase)
            .map_err(|error| failure("sidecar_phase", format!("sidecar returned invalid phase identifier `{phase}`: {error}")))?;
        let label = execution
            .snapshot
            .template()
            .and_then(|template| template.phases.iter().find(|step| step.metadata.identifier.as_str() == phase))
            .map(|step| step.metadata.label.clone())
            .unwrap_or_else(|| phase_label(phase));
        if publish {
            update_phase(context, scope, identifier.clone(), label.clone(), "running").await?;
        }
        if let Some((previous_identifier, previous_label)) = self.current.take() {
            if publish {
                update_phase(context, scope, previous_identifier, previous_label, "passed").await?;
            }
            if let Some(previous) = self.phases.last_mut() {
                previous.status = "passed".to_owned();
            }
        }
        self.phases.push(ProviderPhaseOutcome { identifier: phase.to_owned(), label: label.clone(), status: "running".to_owned() });
        self.current = Some((identifier, label));
        Ok(())
    }

    pub(super) fn finish(&mut self, status: &str) {
        self.current = None;
        if let Some(current) = self.phases.last_mut() {
            current.status = status.to_owned();
        }
    }
}

impl StreamLog {
    pub(super) async fn append(
        &mut self, context: &ExecutionFixtureContext, execution: &TestExecution, workflow: &str, source: &str,
        entries: &[crate::execution::log::Entry],
    ) {
        if entries.is_empty() {
            return;
        }
        use crate::execution::log::Level;
        for entry in entries {
            macro_rules! mirror {
                ($level:expr) => {
                    tracing::event!(target: "sidecar", $level,
                        plugin_id = source,
                        execution_id = %execution.test_execution_id,
                        cycle_id = %execution.test_cycle_id,
                        line = %entry.message,
                        timestamp = %entry.timestamp,
                        "sidecar execution log"
                    )
                };
            }
            match Level::try_from(entry.level) {
                Ok(Level::Trace) => mirror!(tracing::Level::TRACE),
                Ok(Level::Debug) => mirror!(tracing::Level::DEBUG),
                Ok(Level::Info) => mirror!(tracing::Level::INFO),
                Ok(Level::Warn) => mirror!(tracing::Level::WARN),
                Ok(Level::Error) => mirror!(tracing::Level::ERROR),
                Err(_) => tracing::warn!(
                    plugin_id = source,
                    execution_id = %execution.test_execution_id,
                    cycle_id = %execution.test_cycle_id,
                    level = entry.level,
                    line = %entry.message,
                    "sidecar execution diagnostic has unknown severity"
                ),
            }
        }
        if self.failed {
            return;
        }
        let path = format!("{workflow}/stream.log");
        let mut buffer = Vec::with_capacity(64 * 1024);
        for entry in entries {
            let level = match Level::try_from(entry.level) {
                Ok(Level::Trace) => "TRACE".to_owned(),
                Ok(Level::Debug) => "DEBUG".to_owned(),
                Ok(Level::Info) => "INFO".to_owned(),
                Ok(Level::Warn) => "WARN".to_owned(),
                Ok(Level::Error) => "ERROR".to_owned(),
                Err(_) => format!("UNKNOWN({})", entry.level),
            };
            // Prefix every physical line, including blank messages, so multiline
            // diagnostics retain severity when read as plain text.
            for line in entry.message.split('\n') {
                let timestamp = if entry.timestamp.is_empty() { crate::execution::log::timestamp() } else { entry.timestamp.clone() };
                let formatted = format!("{timestamp} [{level}] {line}\n");
                let bytes = formatted.as_bytes();
                let mut remaining = bytes;
                while !remaining.is_empty() {
                    let count = remaining.len().min(64 * 1024 - buffer.len());
                    buffer.extend_from_slice(&remaining[..count]);
                    remaining = &remaining[count..];
                    if buffer.len() == 64 * 1024 {
                        if !self.persist(context, execution, source, &path, &buffer).await {
                            return;
                        }
                        buffer.clear();
                    }
                }
            }
        }
        if !buffer.is_empty() {
            self.persist(context, execution, source, &path, &buffer).await;
        }
    }

    async fn persist(
        &mut self, context: &ExecutionFixtureContext, execution: &TestExecution, source: &str, path: &str, bytes: &[u8],
    ) -> bool {
        let storage = &context.runtime.execution_storage;
        let persisted = async {
            storage.reserve_output(&execution.test_execution_id, path).await?;
            storage.append_output(&execution.test_execution_id, path, bytes).await
        }
        .await;
        if let Err(error) = persisted {
            self.failed = true;
            tracing::warn!(plugin_id = source, %error, "failed to persist sidecar stream log");
            false
        } else {
            true
        }
    }
}

impl HostCapture {
    async fn start(context: &ExecutionFixtureContext, execution: &TestExecution, streams: &[Identifier], source: &str) -> Self {
        if streams.is_empty() {
            return Self::default();
        }
        match context.runtime.lease_streams(sampling_scope(execution), streams, source).await {
            Ok(leases) => Self { leases },
            Err(error) => {
                tracing::warn!(
                    plugin_id = source,
                    streams = streams
                        .iter()
                        .map(Identifier::to_string)
                        .collect::<Vec<_>>()
                        .join(", "),
                    %error,
                    "declared streams could not be leased"
                );
                Self::default()
            }
        }
    }

    async fn release(&mut self, context: &ExecutionFixtureContext) -> Result<(), ProviderExecutionFailure> {
        context.runtime.release_stream_leases(self.leases.clone()).await.map_err(|error| failure("sidecar_leases", error.to_string()))?;
        self.leases.clear();
        Ok(())
    }
}

impl DropReason {
    fn trace(self) -> &'static str {
        match self {
            Self::Abandoned => "dropped an execution artifact its producer abandoned",
            Self::Incomplete => "discarded an execution artifact that never completed",
        }
    }

    fn entry(self, name: &str, received: u64) -> String {
        match self {
            Self::Abandoned => format!("dropped execution artifact `{name}`: the producer aborted the transfer after {received} bytes"),
            Self::Incomplete => {
                format!("discarded incomplete execution artifact `{name}`: {received} bytes arrived before the execution ended")
            }
        }
    }
}

/* === Public Functions === */

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
        .map(|(name, parameter)| (name.clone(), parameter.value.clone()))
        .collect::<BTreeMap<String, Value>>();
    let secrets = match context.runtime.installed_plugins().get(source) {
        Some(artifact) => context
            .runtime
            .resolve_secrets(&artifact.settings, &parameters)
            .await
            .map_err(|error| failure("sidecar_start", error.to_string()))?,
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
        parameters_json: serde_json::to_string(&parameters)
            .map_err(|error| failure("sidecar_start", format!("failed to encode resolved case parameters: {error}")))?,
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
    let record_subscription_after = context.runtime.state_manager.applied().0;
    let mut phase = PhaseTracker::default();
    phase.observe(context, &execution.scope, "initialization", execution, true).await?;
    context
        .capture_phase(execution.scope.clone(), dokime_domain::model::execution::CapturePhase::Initialization)
        .await
        .map_err(|error| failure("sidecar_capture_phase", error.to_string()))?;
    let record_demand = match context.runtime.hold_record_demand(&declared.records) {
        Ok(holds) => holds,
        Err(error) => {
            close_record_subscriptions(context, execution).await?;
            return Err(failure("sidecar_records", error.to_string()));
        }
    };
    let mut capture = HostCapture::start(context, execution, &declared.samples, source).await;
    if context.is_cancelled() {
        capture.release(context).await?;
        close_record_subscriptions(context, execution).await?;
        return Err(failure("cancel", "provider invocation cancelled before the run started"));
    }

    let target = Target::of(operations, case.metadata.identifier.as_str());
    let session = Arc::new(Session::new(session::Opened {
        context: context.clone(),
        execution: execution.clone(),
        workflow: workflow.to_owned(),
        source: source.to_owned(),
        target: target.clone(),
        prompts: PromptBridge::new(declared.prompts),
        record_subscription_after,
        phase,
    }));
    context
        .finalization
        .register(ExecutionCleanup {
            capture,
            _publication_admission: publication_admission,
            _record_demand: record_demand,
            execution: execution.clone(),
            session: Arc::clone(&session),
            _progress: ProgressCleanup { registry: Arc::clone(&context.runtime.progress), execution: execution.test_execution_id.clone() },
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
    let answered = loop {
        tokio::select! {
            answered = &mut run => break Some(answered),
            () = tokio::time::sleep(WATCH_INTERVAL) => {}
        }
        if stop_deadline.is_some_and(|deadline| tokio::time::Instant::now() >= deadline) {
            break None;
        }
        if !cancelled
            && (session.refused().await
                || cancellation_requested(context, cycle).await
                || context.execution_stop_requested(&execution.test_execution_id))
        {
            cancelled = true;
            let deadline = tokio::time::Instant::now() + STOP_SETTLEMENT;
            stop_deadline = Some(deadline);
            context.finalization.cancelling(deadline).await;
            let cancel = async {
                // Cancel the pending operator questions before signalling the run.
                session.cancel().await;
                if let Err(error) = target
                    .signal(ExecutionSignal { execution_token: execution_token.clone(), cancel: true, ..ExecutionSignal::default() })
                    .await
                {
                    session.refuse(error).await;
                }
            };
            if tokio::time::timeout_at(deadline, cancel).await.is_err() {
                break None;
            }
        }
        let reconcile = commands::reconcile(context, &target, execution, false, cancelled);
        let reconciled = match stop_deadline {
            Some(deadline) => match tokio::time::timeout_at(deadline, reconcile).await {
                Ok(result) => result,
                Err(_) => break None,
            },
            None => reconcile.await,
        };
        if let Err(error) = reconciled {
            session.refuse(error).await;
        }
    };
    context.runtime.execution_sessions.remove(&execution_token);
    // A run can wind down before the watch loop sees its stop, for instance
    // when the stop settled the question it was waiting on. It still ended
    // because it was stopped.
    if !cancelled && (cancellation_requested(context, cycle).await || context.execution_stop_requested(&execution.test_execution_id)) {
        cancelled = true;
    }
    let Some(answered) = answered else {
        // The run never answered its cancellation. The host settles it as
        // stopped with the evidence captured so far; cleanup signals the run
        // once more and releases everything the execution held.
        tracing::warn!(execution_id = %execution.test_execution_id,
            budget_s = STOP_SETTLEMENT.as_secs(), "cancelled run did not settle; host settled it as stopped");
        let message =
            format!("the run did not answer its stop within {} seconds; the host settled it as stopped", STOP_SETTLEMENT.as_secs());
        session.narrate_refusal(&failure("stop_settlement", message.clone())).await;
        // A late run may close its own step first; cleanup still owns release.
        if let Err(error) = close_stopped_attempt(context, execution).await {
            tracing::warn!(execution_id = %execution.test_execution_id,
                detail = %error.message, "host could not close the stopped step attempt");
        }
        context.finalization.finish(context).await?;
        return Ok(CaseResult {
            execution: execution_outcome(case, execution, source, "cancelled", Vec::new(), Some(message), Vec::new(), None),
            cancelled: true,
        });
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
    let mut stop_after_failure = false;
    for execution in runnable_executions(cycle) {
        let case = execution.snapshot.case();
        if stop_after_failure {
            executions.push(execution_outcome(
                case,
                execution,
                &target.source,
                "skipped",
                Vec::new(),
                Some("not dispatched because an earlier case failed".to_owned()),
                Vec::new(),
                None,
            ));
            continue;
        }
        if cancellation_requested(&context, cycle).await {
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
        stop_after_failure = !cycle.continue_on_failure && result.execution.status != "passed" && !stopped_alone;
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

pub(crate) fn with_failed_stage(mut phases: Vec<ProviderPhaseOutcome>, failed_in: Option<&str>) -> Vec<ProviderPhaseOutcome> {
    let Some(stage) = failed_in else {
        return phases;
    };
    if phases.iter().any(|phase| phase.identifier == stage) {
        return phases;
    }
    phases.push(ProviderPhaseOutcome { identifier: stage.to_owned(), label: phase_label(stage), status: "failed".to_owned() });
    phases
}

fn admit_prompt(declared: &[crate::prompts::PromptDefinition], prompt: &PromptRequest) -> Result<(), ProviderExecutionFailure> {
    if !declared.iter().any(|definition| definition.prompt_type == prompt.prompt_type) {
        return Err(failure("sidecar_prompt", format!("operator question {} is not declared by this template", prompt.prompt_type)));
    }
    Ok(())
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

async fn report_dropped(
    context: &ExecutionFixtureContext, execution: &TestExecution, workflow: &str, source: &str, stream_log: &mut StreamLog,
    dropped: &[DiscardedArtifact], reason: DropReason,
) {
    if dropped.is_empty() {
        return;
    }
    let diagnostics = dropped
        .iter()
        .map(|artifact| {
            tracing::warn!(
                plugin_id = source,
                transfer_id = artifact.transfer_id,
                name = artifact.name,
                received_bytes = artifact.received,
                "{}",
                reason.trace()
            );
            crate::execution::log::Entry::new(crate::execution::log::Level::Warn, reason.entry(&artifact.name, artifact.received))
        })
        .collect::<Vec<_>>();
    stream_log.append(context, execution, workflow, source, &diagnostics).await;
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
    let step = Identifier::new(&reply.phase)
        .map_err(|error| failure("sidecar_progress", format!("sidecar progress has no valid current step: {error}")))?;
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
        context
            .runtime
            .publish_sidecar_record(record, source)
            .await
            .map_err(|error| failure("sidecar_record", format!("sidecar `{source}` record was rejected: {error}")))?;
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
            serde_json::from_str::<dokime_domain::model::traceability::TraceabilitySection>(encoded)
                .map_err(|error| failure("sidecar_traceability", format!("sidecar `{source}` traceability section is invalid: {error}")))
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
        .map_err(|error| failure("sidecar_traceability", format!("sidecar `{source}` traceability was rejected: {}", error)))
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
            serde_json::from_str::<crate::document::DocumentAddition>(encoded)
                .map_err(|error| failure("sidecar_document", format!("sidecar `{source}` document addition is invalid: {error}")))
        })
        .collect::<Result<Vec<_>, ProviderExecutionFailure>>()?;

    context
        .add_document_parts(scope.clone(), additions)
        .await
        .map_err(|error| failure("sidecar_document", format!("sidecar `{source}` document addition was rejected: {error}")))
}

fn decode_records(reply: &EvidenceBatch, scope: &RecordScope) -> Result<Vec<RecordPublishRequest>, ProviderExecutionFailure> {
    let mut records = Vec::with_capacity(reply.records.len());
    for encoded in &reply.records {
        let mut value: serde_json::Value = serde_json::from_str(encoded)
            .map_err(|error| failure("sidecar_record", format!("sidecar returned malformed record JSON: {error}")))?;
        ensure_nullable_field(&mut value, "scope");
        let mut record: RecordPublishRequest = serde_json::from_value(value)
            .map_err(|error| failure("sidecar_record", format!("sidecar returned an invalid record publish request: {error}")))?;
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
        .commit(crate::transactions::CloseRecordSubscriptions { test_cycle_id, test_execution_id: execution.test_execution_id.clone() })
        .await
        .map_err(|error| failure("sidecar_records", error.to_string()))?;
    Ok(())
}

async fn update_phase(
    context: &ExecutionFixtureContext, scope: &RecordScope, identifier: Identifier, label: String, status: &str,
) -> Result<(), ProviderExecutionFailure> {
    context
        .update_execution_phase(scope.clone(), identifier, label, status.to_owned())
        .await
        .map_err(|error| failure("sidecar_phase", format!("sidecar phase update was rejected: {error}")))
}

async fn cancellation_requested(context: &ExecutionFixtureContext, cycle: &CycleView) -> bool {
    context.is_cancelled()
        || matches!(
            context.cycle_lifecycle(&cycle.test_cycle_id).await,
            Some(
                dokime_domain::model::execution::result::CycleStatus::Stopping
                    | dokime_domain::model::execution::result::CycleStatus::Stopped
            )
        )
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

/// Ends the step a host-settled stop left open as cancelled by the operator.
///
/// Interrupted is reserved for recovery that never resumed the run; an
/// explicit stop the host settled keeps its stop outcome.
async fn close_stopped_attempt(context: &ExecutionFixtureContext, execution: &TestExecution) -> Result<(), ProviderExecutionFailure> {
    let execution_id = execution.test_execution_id.clone();
    let active = context
        .runtime
        .state_manager
        .state()
        .read(move |state| state.executions.get(&execution_id).and_then(|execution| execution.active_step_attempt.clone()))
        .await
        .map_err(|error| failure("stop_settlement", error.to_string()))?
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
    context.record_step_attempt(execution.scope.clone(), attempt).await.map_err(|error| failure("stop_settlement", error.to_string()))
}

/// Hands the run every captured sample after its stream's `replayed`
/// position, advancing that position past each signal the run has received.
async fn replay_sample_inputs(
    context: &ExecutionFixtureContext, target: &Target, execution: &TestExecution, replayed: &mut BTreeMap<harmos::Identifier, u64>,
) -> Result<(), ProviderExecutionFailure> {
    // The run's own evidence is fixed once its publications are flushed. Each
    // stream is one query against that cut, read through its own pages. The
    // positions outlive this call: a settling page the run sends again after
    // its call ran out of time resumes each stream instead of starting over.
    let storage = context.runtime.storage();
    let cut = storage.flush().await.map_err(|error| failure("sidecar_samples", error.to_string()))?;
    let mut at = storage.cursor_saved();
    at.sequence = cut.sequence;
    let contracts = context.runtime.state_read().telemetry_contracts();
    for contract in contracts.iter().filter(|contract| !contract.foreign) {
        let position = replayed.entry(contract.stream.clone()).or_default();
        replay_stream(context, target, execution, &at, contract, position).await?;
    }
    Ok(())
}

// One Stream's Samples In Storage Order, Read Through A Single Query.
async fn replay_stream(
    context: &ExecutionFixtureContext, target: &Target, execution: &TestExecution, at: &harmos::storage::StorageCursor,
    contract: &crate::telemetry::TelemetryContract, replayed: &mut u64,
) -> Result<(), ProviderExecutionFailure> {
    use harmos::storage::query::{Query, QueryOutput, RowSlice, Source, StreamSelection};
    let samples_failure = |error: harmos::Error| failure("sidecar_samples", error.to_string());
    let scope =
        harmos::Identifier::new(execution.test_execution_id.as_str()).map_err(|error| failure("sidecar_samples", error.to_string()))?;
    let scope_json = serde_json::to_string(&execution.scope).map_err(|error| failure("sidecar_samples", error.to_string()))?;
    let codec = crate::storage::telemetry::codec(contract).map_err(|error| samples_failure(replay_codec_error(error)))?;
    let mut reader = context
        .runtime
        .storage()
        .query(Query {
            at: at.clone(),
            source: Source::Stream(StreamSelection {
                schema: codec.schema().clone(),
                scope: Some(scope),
                item: None,
                stream: Some(contract.stream.clone()),
                source_start: None,
                source_end: None,
                after: *replayed,
            }),
            transforms: vec![],
            slice: RowSlice::default(),
        })
        .await
        .map_err(samples_failure)?;
    let owner = &execution.scope;
    let mut samples = Vec::new();
    let mut bytes = 0;
    let mut through = *replayed;
    while let Some(output) = reader.next().await.map_err(samples_failure)? {
        let QueryOutput::Batch(batch) = output else {
            continue;
        };
        let raw = batch.as_record_batch();
        let positions = raw
            .column_by_name("_storage")
            .and_then(|column| column.as_any().downcast_ref::<arrow_array::StructArray>())
            .and_then(|metadata| metadata.column_by_name("position"))
            .and_then(|column| column.as_any().downcast_ref::<arrow_array::UInt64Array>())
            .ok_or_else(|| failure("sidecar_samples", "storage positions are missing"))?;
        let entries =
            crate::storage::telemetry::decode_batch(&codec, &batch).map_err(|error| samples_failure(replay_codec_error(error)))?;
        for (index, entry) in entries.into_iter().enumerate() {
            let captured =
                entry.sample.scope.as_ref().ok_or_else(|| failure("sidecar_samples", "captured sample has no execution scope"))?;
            if captured.session_id != owner.session_id
                || captured.test_cycle_id != owner.test_cycle_id
                || captured.test_case_id != owner.test_case_id
                || captured.test_execution_id != owner.test_execution_id
                || captured.template_id != owner.template_id
                || captured.suite_id != owner.suite_id
            {
                return Err(failure("sidecar_samples", "captured sample differs from its execution owner"));
            }
            let encoded = serde_json::to_string(&entry.sample).map_err(|error| failure("sidecar_samples", error.to_string()))?;
            if !samples.is_empty() && bytes + encoded.len() > REPLAY_SIGNAL_BYTES {
                deliver_replayed(target, execution, &scope_json, std::mem::take(&mut samples)).await?;
                *replayed = through;
                bytes = 0;
            }
            bytes += encoded.len();
            samples.push(encoded);
            through = positions.value(index);
        }
    }
    reader.completion().map_err(samples_failure)?;
    deliver_replayed(target, execution, &scope_json, samples).await?;
    *replayed = through;
    Ok(())
}

fn replay_codec_error(source: crate::storage::Error) -> harmos::Error {
    match source {
        crate::storage::Error::Runtime(source) => source,
        source => harmos::Error::backend("decode typed sample replay", harmos::BackendFailure::Corrupt, source),
    }
}

async fn deliver_replayed(
    target: &Target, execution: &TestExecution, scope_json: &str, samples: Vec<String>,
) -> Result<(), ProviderExecutionFailure> {
    if samples.is_empty() {
        return Ok(());
    }
    target
        .signal(ExecutionSignal {
            execution_token: execution.test_execution_id.to_string(),
            samples: Some(crate::transport::DeliverSamplesRequest { scope_json: scope_json.to_owned(), samples, gaps: Vec::new() }),
            ..ExecutionSignal::default()
        })
        .await?;
    Ok(())
}

/* === Tests === */
