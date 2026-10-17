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
use std::sync::Arc;
// Third Party Imports
use futures_util::FutureExt;
use harmos::operations::{Context, Operation, OperationDeclaration, OperationPolicy, RestartPolicy, RetryPolicy};
use serde::{Deserialize, Serialize};
// Local Imports
use crate::app::Lanes;
use crate::domains::ExecutionProviderTarget;
use crate::transport::{ProviderCycleOutcome, ProviderExecutionFailure};
use crate::{
    execution_runtime::provider::{ExecutionFixtureContext, ExecutionFixtureFuture, ProviderExecutorRegistry},
    handle::RuntimeHandle,
};
use dokime_domain::model::execution::result::ExecutionStatus;
use dokime_domain::model::identity::TestCycleId;
use dokime_domain::model::identity::TestExecutionId;

/* === Constants === */
const PROVIDER_POLICY: OperationPolicy = OperationPolicy {
    timeout: std::time::Duration::from_secs(86_400),
    attempt_timeout: std::time::Duration::from_secs(86_400),
    retries: RetryPolicy::Never,
    restart: RestartPolicy::Never,
};

/* === Macros === */

/* === Type Aliases === */
type ProviderOutcome = std::result::Result<ProviderCycleOutcome, ProviderExecutionFailure>;

/* === Structs === */
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct ProviderCycleRequest {
    pub(crate) request_id: String,
    pub(crate) test_cycle_id: String,
    pub(crate) target: serde_json::Value,
}

pub(crate) struct ProviderTask {
    pub(crate) source: String,
    pub(crate) cycle: TestCycleId,
    pub(crate) task: tokio::task::JoinHandle<()>,
    started: Arc<std::sync::atomic::AtomicU8>,
    cancel: tokio::sync::watch::Sender<bool>,
}

pub(crate) struct PublicationFailure {
    request_id: dokime_domain::model::identity::CommandId,
    source: Arc<crate::runtime::Error>,
}

// The portable identity of one provider cycle run; the process-local
// invocation travels as the operation's endpoints and carries its outcome.
#[harmos::message]
#[harmos::operation(response = (), id = "dokime.execution.provider.run", version = 1, policy = PROVIDER_POLICY)]
#[derive(Clone, PartialEq)]
struct RunProviderCycle {
    request_id: String,
}

#[derive(Clone)]
struct ProviderInvocation {
    job: ProviderCycleRequest,
    runtime: RuntimeHandle,
    after: harmos::state::Position,
    started: Arc<std::sync::atomic::AtomicU8>,
    finalization: crate::execution_runtime::run::Finalization,
    outcome: Arc<std::sync::Mutex<Option<ProviderOutcome>>>,
}

struct ProviderState {
    invocation: Option<ProviderInvocation>,
    work: ProviderWork,
}

struct ProviderWork {
    future: Option<ExecutionFixtureFuture<'static>>,
    cancellation: tokio::sync::watch::Sender<bool>,
    cancelled: tokio::sync::watch::Receiver<bool>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl ProviderWork {
    fn new() -> Self {
        let (cancellation, cancelled) = tokio::sync::watch::channel(false);
        Self { future: None, cancellation, cancelled }
    }

    async fn run(&mut self, future: ExecutionFixtureFuture<'static>) -> Result<ProviderCycleOutcome, ProviderExecutionFailure> {
        self.future = Some(Box::pin(async move {
            match std::panic::AssertUnwindSafe(future).catch_unwind().await {
                Ok(outcome) => outcome,
                Err(_) => Err(ProviderExecutionFailure::new("panic", "provider work panicked")),
            }
        }));
        let outcome = self.future.as_mut().expect("provider work is retained").await;
        self.future = None;
        outcome
    }

    async fn finalize(&mut self) -> Option<ProviderExecutionFailure> {
        self.cancellation.send_replace(true);
        let outcome = match self.future.as_mut() {
            Some(future) => future.await.err(),
            None => None,
        };
        self.future = None;
        outcome
    }
}

impl RuntimeHandle {
    pub(crate) fn provider_completion_result(&self) -> crate::runtime::Result<()> {
        let failure = self.provider_completion_failure.lock().expect("provider completion failure lock poisoned");
        if let Some(failure) = &*failure {
            return Err(crate::runtime::Error::ProviderCompletion {
                request_id: failure.request_id.clone(),
                source: failure.source.clone(),
            });
        }
        Ok(())
    }
}

// The invocation is process-local state, never a streamed endpoint.
impl harmos::operations::EndpointSet for ProviderInvocation {
    fn item_types() -> (Option<std::any::TypeId>, Option<std::any::TypeId>) {
        (None, None)
    }
}

impl Operation for ProviderState {
    type Context<'a> = Context<'a>;
    type Declaration = RunProviderCycle;
    type Resources = ();
    type Endpoints = ProviderInvocation;
    type Error = crate::runtime::Error;

    async fn initialize<'ctx>(_: Arc<Self::Resources>, _: Self::Context<'ctx>) -> Result<Self, Self::Error> {
        Ok(ProviderState { invocation: None, work: ProviderWork::new() })
    }

    async fn run<'ctx>(
        &mut self, _context: Self::Context<'ctx>, _: &mut RunProviderCycle, endpoints: &mut ProviderInvocation,
    ) -> Result<harmos::operations::Response<()>, Self::Error> {
        let invocation = self.invocation.get_or_insert_with(|| endpoints.clone());
        let slot = Arc::clone(&invocation.outcome);
        if invocation.started.compare_exchange(0, 1, std::sync::atomic::Ordering::AcqRel, std::sync::atomic::Ordering::Acquire).is_err() {
            *slot.lock().expect("provider outcome lock poisoned") =
                Some(Err(ProviderExecutionFailure::new("cancel", "plugin shutdown before execution started")));
            return Ok(().into());
        }

        let runtime = invocation.runtime.clone();
        let providers = Arc::clone(&runtime.providers);
        let job = invocation.job.clone();
        let after = invocation.after;
        let cancelled = self.work.cancelled.clone();
        let finalization = invocation.finalization.clone();
        let outcome = self
            .work
            .run(Box::pin(async move { run_provider_cycle(runtime, &providers, job, after, cancelled, finalization).await }))
            .await;
        *slot.lock().expect("provider outcome lock poisoned") = Some(outcome);
        Ok(().into())
    }

    async fn cleanup(&mut self) -> Result<(), Self::Error> {
        let Some(invocation) = &self.invocation else {
            return Ok(());
        };
        let settlement = std::time::Instant::now();
        tracing::debug!(request_id = %invocation.job.request_id, phase = "provider_settlement", "provider work settlement started");
        if let Some(error) = self.work.finalize().await {
            tracing::warn!(
                request_id = %invocation.job.request_id,
                stage = %error.stage,
                message = %error.message,
                "cancelled provider work settled with an execution failure"
            );
        }
        tracing::debug!(request_id = %invocation.job.request_id, phase = "provider_settlement", elapsed_ms = %settlement.elapsed().as_millis(), "provider work settlement finished");
        invocation
            .finalization
            .finish(&ExecutionFixtureContext::new(invocation.runtime.clone()))
            .await
            .map_err(|error| crate::runtime::Error::Operation(format!("sidecar cleanup unresolved: {}", error.message)))
    }
}

impl ProviderTask {
    pub(crate) fn cancel_queued(&self) {
        if self.started.compare_exchange(0, 2, std::sync::atomic::Ordering::AcqRel, std::sync::atomic::Ordering::Acquire).is_ok() {
            self.cancel.send_replace(true);
        }
    }
}

impl ProviderCycleRequest {
    pub(crate) fn submit(self, runtime: RuntimeHandle, after: harmos::state::Position) {
        let registry = runtime.provider_tasks.clone();
        let mut tasks = registry.lock().unwrap();
        // Admission already committed this request and retains its capacity
        // permit until registration. Shutdown waits for that permit before
        // draining this registry, so even a now-interrupted request must keep
        // its monitor and publish a terminal result.
        tasks.retain(|task| !task.task.is_finished());
        let target: ExecutionProviderTarget = serde_json::from_value(self.target.clone()).expect("provider target");
        let source = target.source;
        let cycle = TestCycleId::from_uuid(&self.test_cycle_id).expect("cycle id");
        let started = Arc::new(std::sync::atomic::AtomicU8::new(0));
        let (cancel, mut cancelled) = tokio::sync::watch::channel(false);
        let finalization = crate::execution_runtime::run::Finalization::default();
        let settlement = Arc::new(std::sync::Mutex::new(None));
        let invocation = runtime.operations.submit_endpoints(
            RunProviderCycle { request_id: self.request_id.clone() },
            ProviderInvocation {
                job: self.clone(),
                runtime: runtime.clone(),
                after,
                started: Arc::clone(&started),
                finalization: finalization.clone(),
                outcome: Arc::clone(&settlement),
            },
        );
        let task = tokio::spawn(async move {
            let mut handle = match invocation {
                Ok(handle) => handle,
                Err(error) => {
                    finish_interrupted_provider_job(&runtime, &self, error).await;
                    return;
                }
            };

            let mut changes = runtime.state_watch.subscribe();
            let mut shutdown = runtime.shutdown_requested.subscribe();
            let monitoring = std::time::Instant::now();
            tracing::debug!(request_id = %self.request_id, cycle_id = %self.test_cycle_id, phase = "provider_monitor", "provider monitor awaiting original invocation settlement");
            let outcome = tokio::select! {
                outcome = &mut handle => outcome,
                _ = async { let _ = cancelled.wait_for(|cancelled| *cancelled).await; } => {
                    handle.cancel();
                    handle.await
                }
                _ = async { let _ = shutdown.wait_for(|requested| *requested).await; } => {
                    handle.cancel();
                    handle.await
                }
                _ = async {
                    while !changes.borrow_and_update().state().shutting_down {
                        if changes.changed().await.is_err() { break; }
                    }
                } => {
                    handle.cancel();
                    handle.await
                }
            };
            tracing::debug!(request_id = %self.request_id, cycle_id = %self.test_cycle_id, phase = "provider_monitor", elapsed_ms = %monitoring.elapsed().as_millis(), transport_ok = outcome.is_ok(), "provider monitor observed settled invocation");
            match outcome {
                Ok(_) => {
                    let settled = settlement
                        .lock()
                        .expect("provider outcome lock poisoned")
                        .take()
                        .unwrap_or_else(|| Err(ProviderExecutionFailure::new("cancel", "provider run settled without an outcome")));
                    let cycle = TestCycleId::from_uuid(&self.test_cycle_id).expect("journaled cycle id is valid");
                    let started = started_execution_ids_for(&runtime, &cycle);
                    submit_finish_provider_cycle(&runtime, &self, settled, started).await;
                }
                Err(error) if finalization.pending().await => {
                    tracing::error!(%error, "provider cleanup is unresolved; terminal publication remains blocked");
                    runtime.provider_completion_failure.lock().expect("provider completion failure lock poisoned").get_or_insert_with(
                        || PublicationFailure {
                            request_id: dokime_domain::model::identity::CommandId::from_uuid(&self.request_id)
                                .expect("journaled request ID is valid"),
                            source: Arc::new(crate::runtime::Error::Operation(format!("provider cleanup remains unresolved: {error}"))),
                        },
                    );
                }
                Err(error) => finish_interrupted_provider_job(&runtime, &self, error).await,
            }
        });
        tasks.push(ProviderTask { source, cycle, task, started, cancel });
    }
}

/* === Public Functions === */
pub(crate) fn register(operations: &harmos::operations::OperationManager) -> harmos::Result<()> {
    operations.register_operation::<ProviderState>("dokime.execution", Lanes::Execution, Arc::new(()))
}

/* === Private Functions === */
async fn finish_interrupted_provider_job(runtime: &RuntimeHandle, job: &ProviderCycleRequest, outcome: harmos::Error) {
    let stage = interruption_stage(&outcome, runtime.is_shutting_down());
    let error = ProviderExecutionFailure::new(stage, format!("provider cycle run did not complete: {outcome}"));
    let test_cycle_id = TestCycleId::from_uuid(job.test_cycle_id.as_str()).expect("journaled provider-cycle test cycle id is valid");
    let started_execution_ids = started_execution_ids_for(runtime, &test_cycle_id);
    submit_finish_provider_cycle(runtime, job, Err(error), started_execution_ids).await;
}

fn interruption_stage(outcome: &harmos::Error, shutting_down: bool) -> &'static str {
    match outcome {
        harmos::Error::OperationRefused(harmos::Refusal::TimedOut) => "timeout",
        harmos::Error::OperationRefused(harmos::Refusal::Cancelled) => "cancel",
        harmos::Error::OperationPanicked { .. } => "panic",
        harmos::Error::OperationRefused(harmos::Refusal::Unavailable) if !shutting_down => "panic",
        harmos::Error::OperationRefused(harmos::Refusal::Unavailable) | harmos::Error::Closing { .. } => "interrupt",
        harmos::Error::OperationDeclaration { .. } | harmos::Error::OperationLaneCycle { .. } | harmos::Error::NotRegistered { .. } => {
            "bind"
        }
        _ => "operation",
    }
}

#[tracing::instrument(
    level = "info",
    name = "runtime.cycle",
    skip_all,
    fields(cycle_id = %job.test_cycle_id),
)]
async fn run_provider_cycle(
    runtime: RuntimeHandle, providers: &ProviderExecutorRegistry, job: ProviderCycleRequest, after: harmos::state::Position,
    cancelled: tokio::sync::watch::Receiver<bool>, finalization: crate::execution_runtime::run::Finalization,
) -> std::result::Result<ProviderCycleOutcome, ProviderExecutionFailure> {
    let test_cycle_id = TestCycleId::from_uuid(job.test_cycle_id.as_str()).expect("journaled provider-cycle test cycle id is valid");
    let target: ExecutionProviderTarget = serde_json::from_value(job.target.clone()).expect("journaled provider-cycle target deserializes");
    let request_id = dokime_domain::model::identity::CommandId::from_uuid(job.request_id.clone()).expect("journaled request ID");
    let context = ExecutionFixtureContext::new(runtime.clone())
        .with_request(request_id.clone())
        .with_cancellation(cancelled)
        .with_finalization(finalization);
    prepare_attempts(&context, &request_id, after).await?;
    let mut changes = runtime.state_watch.subscribe();
    loop {
        let summary = runtime.state_read().run_requests[&request_id].summary.clone();
        if summary.stop_requested || context.is_cancelled() {
            return Err(ProviderExecutionFailure::new("cancel", "request stopped before dispatch"));
        }
        if summary.paused {
            tokio::select! {
                _ = context.cancelled() => continue,
                result = changes.changed() => {
                    result.map_err(|_| ProviderExecutionFailure::new("cancel", "runtime ended"))?;
                }
            }
            continue;
        }
        let at = crate::model::time::now_utc();
        match runtime.commit(crate::transactions::DispatchProvider { request_id: request_id.clone(), at }).await {
            Ok(_) => break,
            Err(_) if runtime.state_read().run_requests[&request_id].summary.paused => {
                continue;
            }
            Err(error) => return Err(ProviderExecutionFailure::new("dispatch", error.to_string())),
        }
    }
    let summary = runtime.state_read().run_requests[&request_id].summary.clone();
    runtime
        .request_storage
        .update(&request_id, |receipt| receipt.summary = summary)
        .await
        .map_err(|error| ProviderExecutionFailure::new("request_checkpoint", error.to_string()))?;
    let cycle = runtime
        .state_read()
        .cycle_view(&test_cycle_id)
        .ok_or_else(|| ProviderExecutionFailure::new("bind", "cycle disappeared before run"))?;
    let executor = providers.get(&target).ok_or_else(|| ProviderExecutionFailure::new("bind", "provider executor is unavailable"))?;
    // Terminal publication belongs to the monitor after this cancellable job
    // and its children settle, so shutdown cannot start a competing finish.
    let running = std::time::Instant::now();
    tracing::debug!(request_id = %request_id, cycle_id = %test_cycle_id, phase = "provider_work", "provider work started");
    let outcome = executor.run_cycle(context, &cycle, &target).await;
    tracing::debug!(request_id = %request_id, cycle_id = %test_cycle_id, phase = "provider_work", elapsed_ms = %running.elapsed().as_millis(), succeeded = outcome.is_ok(), "provider work returned");
    outcome
}

async fn prepare_attempts(
    context: &ExecutionFixtureContext, request_id: &dokime_domain::model::identity::CommandId, _after: harmos::state::Position,
) -> Result<(), ProviderExecutionFailure> {
    let runtime = &context.runtime;
    let mut changes = runtime.state_watch.subscribe();
    loop {
        if context.is_cancelled() {
            return Err(ProviderExecutionFailure::new("cancel", "provider invocation cancelled"));
        }
        let (summary, next) = {
            let state = runtime.state_read();
            let request =
                state.run_requests.get(request_id).ok_or_else(|| ProviderExecutionFailure::new("prepare", "request disappeared"))?;
            if state.shutting_down || request.summary.stop_requested {
                return Err(ProviderExecutionFailure::new("cancel", "request stopped before provider dispatch"));
            }
            let next = request
                .pending
                .front()
                .map(|id| {
                    state
                        .executions
                        .get(id)
                        .map(|execution| (id.clone(), execution.test_case_id.clone()))
                        .ok_or_else(|| ProviderExecutionFailure::new("prepare", "queued execution disappeared"))
                })
                .transpose()?;
            (request.summary.clone(), next)
        };
        if summary.paused {
            tokio::select! {
                _ = context.cancelled() => continue,
                result = changes.changed() => {
                    result.map_err(|_| ProviderExecutionFailure::new("cancel", "runtime state stream ended"))?;
                }
            }
            continue;
        }
        let Some((execution_id, case_id)) = next else {
            return Ok(());
        };
        let ordinal = summary.prepared;
        runtime
            .request_storage
            .update(request_id, |receipt| {
                receipt.summary = summary.clone();
                receipt.claimed = ordinal + 1;
                receipt.last_claim =
                    Some(crate::runtime::storage::requests::AttemptClaim { ordinal, case_id, execution_id: execution_id.clone() });
            })
            .await
            .map_err(|error| ProviderExecutionFailure::new("request_checkpoint", error.to_string()))?;
        let result = runtime
            .commit(crate::transactions::PrepareAttempt { request_id: request_id.clone(), ordinal, at: crate::model::time::now_utc() })
            .await;
        if let Err(error) = result {
            let state = runtime.state_read();
            if state.run_requests.get(request_id).is_some_and(|request| request.summary.paused && !request.summary.stop_requested) {
                continue;
            }
            return Err(ProviderExecutionFailure::new("prepare", error.to_string()));
        }
        let cycle = runtime.state_read().cycle_view(&summary.cycle_id).expect("request cycle exists");
        let execution = cycle
            .test_executions
            .iter()
            .find(|execution| execution.test_execution_id == execution_id)
            .ok_or_else(|| ProviderExecutionFailure::new("prepare", "prepared execution disappeared"))?;

        runtime
            .execution_storage
            .prepare_execution(&cycle, execution)
            .await
            .map_err(|error| ProviderExecutionFailure::new("storage", error.to_string()))?;
        let summary = runtime.state_read().run_requests[request_id].summary.clone();
        runtime
            .request_storage
            .update(request_id, |receipt| receipt.summary = summary)
            .await
            .map_err(|error| ProviderExecutionFailure::new("request_checkpoint", error.to_string()))?;
    }
}

fn started_execution_ids_for(runtime: &RuntimeHandle, test_cycle_id: &TestCycleId) -> Vec<TestExecutionId> {
    let state = runtime.state_read();
    state
        .cycles
        .get(test_cycle_id)
        .map(|cycle| {
            state
                .executions_for_cycle(cycle)
                .filter(|execution| matches!(execution.status, ExecutionStatus::Pending | ExecutionStatus::Running))
                .map(|execution| execution.test_execution_id.clone())
                .collect()
        })
        .unwrap_or_default()
}

async fn submit_finish_provider_cycle(
    runtime: &RuntimeHandle, job: &ProviderCycleRequest, mut outcome: std::result::Result<ProviderCycleOutcome, ProviderExecutionFailure>,
    started_execution_ids: Vec<TestExecutionId>,
) {
    let test_cycle_id = TestCycleId::from_uuid(job.test_cycle_id.as_str()).expect("journaled provider-cycle test cycle id is valid");
    let target: ExecutionProviderTarget = serde_json::from_value(job.target.clone()).expect("journaled provider-cycle target deserializes");
    let mut output_error = None;
    for execution_id in &started_execution_ids {
        let sealing = std::time::Instant::now();
        tracing::debug!(request_id = %job.request_id, cycle_id = %test_cycle_id, %execution_id, phase = "output_sealing", "original execution outputs sealing started");
        let sealed = runtime.finish_execution_outputs(execution_id).await;
        tracing::debug!(request_id = %job.request_id, cycle_id = %test_cycle_id, %execution_id, phase = "output_sealing", elapsed_ms = %sealing.elapsed().as_millis(), succeeded = sealed.is_ok(), "original execution outputs sealing finished");
        if let Err(error) = sealed {
            output_error.get_or_insert_with(|| ProviderExecutionFailure::new("artifact", error.to_string()));
        }
    }
    if let Some(error) = output_error {
        match &mut outcome {
            Ok(outcome) => {
                outcome.status = "failed".into();
                outcome.failures.push(error);
            }
            Err(_) => { /* Preserve the original provider failure. */ }
        }
    }
    // A failure the host minted itself never crossed the sidecar's log, so this
    // event is the only place a developer's console learns why a step ended.
    if let Err(failure) = &outcome {
        tracing::error!(
            stage = %failure.stage,
            cycle_id = %test_cycle_id,
            "execution failed on the host side: {}",
            failure.message
        );
    }
    let finish = crate::execution_runtime::provider_finish::Finish {
        test_cycle_id,
        target,
        started_execution_ids,
        outcome,
        finished_at: crate::model::time::now_utc(),
    };
    let publishing = std::time::Instant::now();
    tracing::debug!(request_id = %job.request_id, cycle_id = %job.test_cycle_id, phase = "terminal_publication", "provider monitor publishing terminal state");
    let published = runtime.finish_provider_cycle(finish).await;
    tracing::debug!(request_id = %job.request_id, cycle_id = %job.test_cycle_id, phase = "terminal_publication", elapsed_ms = %publishing.elapsed().as_millis(), succeeded = published.is_ok(), "provider monitor terminal publication finished");
    if let Err(error) = published {
        tracing::error!(?error, storage_read_bytes = runtime.storage().status().read_bytes, "failed to record terminal provider state");
        runtime.provider_completion_failure.lock().expect("provider completion failure lock poisoned").get_or_insert_with(|| {
            PublicationFailure {
                request_id: dokime_domain::model::identity::CommandId::from_uuid(&job.request_id).expect("journaled request ID is valid"),
                source: Arc::new(error),
            }
        });
    }
}

/* === Tests === */
