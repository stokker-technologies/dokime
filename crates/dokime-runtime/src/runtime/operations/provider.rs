/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::collections::HashMap;
use std::sync::Arc;

use futures_util::FutureExt;
use futures_util::future::BoxFuture;
use harmos::{Context, RegistrationExt};
use serde::{Deserialize, Serialize};

use crate::app::Lanes;
use crate::domains::ExecutionProviderTarget;
use crate::operations::execution::RunProviderCycle;
use crate::operations::hosting::{Host, Serve, Served};
use crate::transport::{ProviderCycleOutcome, ProviderExecutionFailure};
use crate::{
    execution_runtime::provider::{ExecutionFixtureContext, ProviderExecutorRegistry},
    handle::RuntimeHandle,
};
use dokime_domain::model::execution::result::ExecutionStatus;
use dokime_domain::model::identity::TestCycleId;
use dokime_domain::model::identity::TestExecutionId;

type ProviderOutcome = std::result::Result<ProviderCycleOutcome, ProviderExecutionFailure>;

// The Process-Local Work Of One Invocation, Started Once By Its First Attempt.
type ProviderWork = Box<dyn FnOnce(tokio::sync::watch::Receiver<bool>) -> BoxFuture<'static, ProviderOutcome> + Send>;

// Concludes An Invocation's Provider Resources Exactly Once After Its Work Settled.
type ProviderFinish = Box<dyn FnOnce() -> BoxFuture<'static, Result<(), ProviderExecutionFailure>> + Send>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct ProviderCycleRequest {
    pub(crate) request_id: String,
    pub(crate) test_cycle_id: String,
    pub(crate) target: serde_json::Value,
}

pub(crate) struct ProviderTask {
    pub(crate) request_id: dokime_domain::model::identity::CommandId,
    pub(crate) settled: tokio_util::sync::CancellationToken,
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

// Process-Local Provider Invocations Keyed By Request ID. `RunProviderCycle`
// Carries Only The Portable Request ID; Its Attempt Starts The Registered Work
// As A Task This Registry Owns, And Whoever Settles The Entry (Its Monitor, Or
// Runtime Shutdown For An Orphan) Awaits That Task And Finishes It Once.
#[derive(Default)]
pub(crate) struct ProviderInvocations {
    entries: std::sync::Mutex<HashMap<String, ProviderInvocation>>,
}

struct ProviderInvocation {
    work: Option<ProviderWork>,
    task: Option<tokio::task::JoinHandle<ProviderOutcome>>,
    cancellation: tokio::sync::watch::Sender<bool>,
    concluded: tokio_util::sync::CancellationToken,
    finish: Option<ProviderFinish>,
}

// What Settling One Invocation Observed.
struct ProviderSettlement {
    // `None` When No Attempt Ever Started The Work.
    outcome: Option<ProviderOutcome>,
    finished: Result<(), ProviderExecutionFailure>,
}

// Signals The Work To Cancel When An Attempt Is Dropped Before It Concluded.
struct CancelOnDrop(Option<tokio::sync::watch::Sender<bool>>);

impl ProviderInvocations {
    // Registers An Invocation Before Its Operation Is Submitted.
    fn open(&self, request_id: &str, work: ProviderWork, finish: ProviderFinish) {
        let (cancellation, _) = tokio::sync::watch::channel(false);
        let invocation = ProviderInvocation {
            work: Some(work),
            task: None,
            cancellation,
            concluded: tokio_util::sync::CancellationToken::new(),
            finish: Some(finish),
        };
        let replaced = self.entries.lock().expect("provider invocation registry is healthy").insert(request_id.to_owned(), invocation);
        debug_assert!(replaced.is_none(), "one provider invocation per request");
    }

    // Serves One Attempt: Starts The Work Once And Awaits Its Conclusion. A
    // Dropped Attempt (Deadline Or Cancellation) Signals The Work To Cancel.
    async fn attend(&self, request_id: &str) -> Result<(), tonic::Status> {
        let (concluded, cancellation) = {
            let mut entries = self.entries.lock().expect("provider invocation registry is healthy");
            let invocation = entries
                .get_mut(request_id)
                .ok_or_else(|| tonic::Status::failed_precondition(format!("provider invocation `{request_id}` is not open")))?;
            if let Some(work) = invocation.work.take() {
                let work = work(invocation.cancellation.subscribe());
                let concluded = invocation.concluded.clone();
                invocation.task = Some(tokio::spawn(async move {
                    // Conclude On Every Exit, Including Panic Or Runtime Teardown.
                    let _concluded = concluded.drop_guard();
                    match std::panic::AssertUnwindSafe(work).catch_unwind().await {
                        Ok(outcome) => outcome,
                        Err(_) => Err(ProviderExecutionFailure::new("panic", "provider work panicked")),
                    }
                }));
            }
            (invocation.concluded.clone(), invocation.cancellation.clone())
        };
        let cancel_on_drop = CancelOnDrop(Some(cancellation));
        concluded.cancelled().await;
        cancel_on_drop.disarm();
        Ok(())
    }

    // Removes The Invocation, Cancels Work No Attempt Awaits Any Longer, Awaits
    // Its Task And Finishes It. Returns `None` When Another Owner Settled It.
    async fn settle(&self, request_id: &str) -> Option<ProviderSettlement> {
        let mut invocation = self.entries.lock().expect("provider invocation registry is healthy").remove(request_id)?;
        invocation.cancellation.send_replace(true);
        let outcome = match invocation.task.take() {
            Some(task) => Some(match task.await {
                Ok(outcome) => outcome,
                Err(error) if error.is_panic() => Err(ProviderExecutionFailure::new("panic", "provider work panicked")),
                Err(_) => Err(ProviderExecutionFailure::new("interrupt", "provider work was aborted")),
            }),
            None => None,
        };
        let finished = match invocation.finish.take() {
            Some(finish) => finish().await,
            None => Ok(()),
        };
        Some(ProviderSettlement { outcome, finished })
    }

    // Settles Every Outstanding Invocation. Runtime Shutdown Calls This After
    // Its Monitors Drained, So No Provider Work Outlives The Runtime.
    pub(crate) async fn settle_all(&self) {
        loop {
            let request_ids: Vec<String> = self.entries.lock().expect("provider invocation registry is healthy").keys().cloned().collect();
            if request_ids.is_empty() {
                return;
            }
            for request_id in request_ids {
                if let Some(settlement) = self.settle(&request_id).await {
                    tracing::warn!(
                        request_id = %request_id,
                        started = settlement.outcome.is_some(),
                        finished = settlement.finished.is_ok(),
                        "shutdown settled an orphaned provider invocation"
                    );
                }
            }
        }
    }
}

impl Serve<RunProviderCycle> for ProviderInvocations {
    fn serve<'a>(&'a self, request: RunProviderCycle, _: Context<'a, RunProviderCycle>) -> Served<'a, ()> {
        Box::pin(async move {
            self.attend(&request.request_id).await?;
            Ok(().into())
        })
    }
}

impl CancelOnDrop {
    fn disarm(mut self) {
        self.0 = None;
    }
}

impl Drop for CancelOnDrop {
    fn drop(&mut self) {
        if let Some(cancellation) = self.0.take() {
            cancellation.send_replace(true);
        }
    }
}

impl RuntimeHandle {
    pub(crate) async fn dispatch_next_request(&self) {
        let _admission = self.plugin_admission.lock().await;
        if self.is_shutting_down() {
            return;
        }
        let job = {
            let state = self.state_read();
            let Some(id) = state.run_queue.front() else { return };
            let request = &state.run_requests[id];
            if request.summary.phase != crate::transactions::provider::request::RunPhase::Queued {
                return;
            }
            let tasks = self.provider_tasks.lock().expect("provider task registry is healthy");
            if tasks.iter().any(|task| &task.request_id == id && !task.task.is_finished()) {
                return;
            }
            ProviderCycleRequest {
                request_id: id.to_string(),
                test_cycle_id: request.summary.cycle_id.to_string(),
                target: serde_json::to_value(&request.target).expect("provider target serializes"),
            }
        };
        job.submit(self.clone());
    }

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

impl ProviderTask {
    pub(crate) fn cancel_queued(&self) {
        if self.started.compare_exchange(0, 2, std::sync::atomic::Ordering::AcqRel, std::sync::atomic::Ordering::Acquire).is_ok() {
            self.cancel.send_replace(true);
        }
    }
}

impl ProviderCycleRequest {
    pub(crate) fn submit(self, runtime: RuntimeHandle) {
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
        let request_id = dokime_domain::model::identity::CommandId::from_uuid(&self.request_id).expect("request id");
        let started = Arc::new(std::sync::atomic::AtomicU8::new(0));
        let (cancel, mut cancelled) = tokio::sync::watch::channel(false);
        let finalization = crate::execution_runtime::run::Finalization::default();
        runtime.provider_invocations.open(
            &self.request_id,
            provider_work(runtime.clone(), self.clone(), Arc::clone(&started), finalization.clone()),
            provider_finish(runtime.clone(), finalization.clone()),
        );
        let invocation = runtime.operations.submit::<RunProviderCycle>(RunProviderCycle { request_id: self.request_id.clone() });
        let settled = tokio_util::sync::CancellationToken::new();
        let settlement_guard = settled.clone().drop_guard();
        let task = tokio::spawn(async move {
            // Signal Settlement On Every Exit, Including Cancellation Or Panic.
            let _settlement = settlement_guard;
            let outcome = match invocation {
                Ok(mut handle) => {
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
                    outcome.map(|_| ())
                }
                Err(error) => Err(error),
            };
            // The Work And Its Finalization Settle Here Exactly Once, Whatever The Attempt Did.
            let settlement = std::time::Instant::now();
            tracing::debug!(request_id = %self.request_id, phase = "provider_settlement", "provider work settlement started");
            let settled = runtime.provider_invocations.settle(&self.request_id).await;
            tracing::debug!(request_id = %self.request_id, phase = "provider_settlement", elapsed_ms = %settlement.elapsed().as_millis(), "provider work settlement finished");
            let settled = settled.and_then(|ProviderSettlement { outcome, finished }| {
                if let Err(error) = finished {
                    tracing::warn!(request_id = %self.request_id, stage = %error.stage, message = %error.message, "provider finalization is unresolved");
                }
                outcome
            });
            match (outcome, settled) {
                (Ok(()), settled) => {
                    let settled =
                        settled.unwrap_or_else(|| Err(ProviderExecutionFailure::new("cancel", "provider run settled without an outcome")));
                    let cycle = TestCycleId::from_uuid(&self.test_cycle_id).expect("journaled cycle id is valid");
                    let started = started_execution_ids_for(&runtime, &cycle);
                    submit_finish_provider_cycle(&runtime, &self, settled, started).await;
                }
                (Err(_), Some(settled)) => {
                    let cycle = TestCycleId::from_uuid(&self.test_cycle_id).expect("journaled cycle id is valid");
                    let started = started_execution_ids_for(&runtime, &cycle);
                    submit_finish_provider_cycle(&runtime, &self, settled, started).await;
                }
                (Err(error), None) if finalization.pending().await => {
                    tracing::error!(%error, "provider cleanup is unresolved; terminal publication remains blocked");
                    runtime.provider_completion_failure.lock().expect("provider completion failure lock poisoned").get_or_insert_with(
                        || PublicationFailure {
                            request_id: dokime_domain::model::identity::CommandId::from_uuid(&self.request_id)
                                .expect("journaled request ID is valid"),
                            source: Arc::new(crate::runtime::Error::Operation(format!("provider cleanup remains unresolved: {error}"))),
                        },
                    );
                }
                (Err(error), None) => finish_interrupted_provider_job(&runtime, &self, error).await,
            }
        });
        tasks.push(ProviderTask {
            request_id,
            settled,
            source,
            cycle,
            task,
            started,
            cancel,
        });
    }
}

pub(crate) async fn register(
    operations: &crate::app::OperationManager, invocations: Arc<ProviderInvocations>,
) -> crate::runtime::Result<()> {
    let mut dependencies = harmos::Dependencies::new();
    dependencies.insert(Host::<RunProviderCycle>::new(invocations))?;
    let batch = harmos::RegistrationBatch::new(dependencies).operation::<RunProviderCycle>();
    operations.register(batch.lane(Lanes::Execution)).await?;
    Ok(())
}

fn provider_work(
    runtime: RuntimeHandle, job: ProviderCycleRequest, started: Arc<std::sync::atomic::AtomicU8>,
    finalization: crate::execution_runtime::run::Finalization,
) -> ProviderWork {
    Box::new(move |cancelled: tokio::sync::watch::Receiver<bool>| -> BoxFuture<'static, ProviderOutcome> {
        Box::pin(async move {
            // A Queued Request Cancelled Before Its Attempt Began Never Starts Provider Work.
            if started.compare_exchange(0, 1, std::sync::atomic::Ordering::AcqRel, std::sync::atomic::Ordering::Acquire).is_err() {
                return Err(ProviderExecutionFailure::new("cancel", "plugin shutdown before execution started"));
            }
            let providers = Arc::clone(&runtime.providers);
            run_provider_cycle(runtime, &providers, job, cancelled, finalization).await
        })
    })
}

fn provider_finish(runtime: RuntimeHandle, finalization: crate::execution_runtime::run::Finalization) -> ProviderFinish {
    Box::new(move || -> BoxFuture<'static, Result<(), ProviderExecutionFailure>> {
        Box::pin(async move { finalization.finish(&ExecutionFixtureContext::new(runtime)).await })
    })
}

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
    runtime: RuntimeHandle, providers: &ProviderExecutorRegistry, job: ProviderCycleRequest, cancelled: tokio::sync::watch::Receiver<bool>,
    finalization: crate::execution_runtime::run::Finalization,
) -> std::result::Result<ProviderCycleOutcome, ProviderExecutionFailure> {
    let test_cycle_id = TestCycleId::from_uuid(job.test_cycle_id.as_str()).expect("journaled provider-cycle test cycle id is valid");
    let target: ExecutionProviderTarget = serde_json::from_value(job.target.clone()).expect("journaled provider-cycle target deserializes");
    let request_id = dokime_domain::model::identity::CommandId::from_uuid(job.request_id.clone()).expect("journaled request ID");
    let context = ExecutionFixtureContext::new(runtime.clone())
        .with_request(request_id.clone())
        .with_cancellation(cancelled)
        .with_finalization(finalization);
    prepare_attempts(&context, &request_id).await?;
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
    context: &ExecutionFixtureContext, request_id: &dokime_domain::model::identity::CommandId,
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
                receipt.last_claim = Some(crate::runtime::storage::requests::AttemptClaim {
                    ordinal,
                    case_id,
                    execution_id: execution_id.clone(),
                });
            })
            .await
            .map_err(|error| ProviderExecutionFailure::new("request_checkpoint", error.to_string()))?;
        let result = runtime
            .commit(crate::transactions::PrepareAttempt {
                request_id: request_id.clone(),
                ordinal,
                at: crate::model::time::now_utc(),
            })
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
        installed_plugin_errors: 100,
        request_id: dokime_domain::model::identity::CommandId::from_uuid(&job.request_id).expect("journaled request identity is valid"),
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
        tracing::error!(?error, "failed to record terminal provider state");
        runtime.provider_completion_failure.lock().expect("provider completion failure lock poisoned").get_or_insert_with(|| {
            PublicationFailure {
                request_id: dokime_domain::model::identity::CommandId::from_uuid(&job.request_id).expect("journaled request ID is valid"),
                source: Arc::new(error),
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    const BOUND: Duration = Duration::from_secs(10);

    fn counting_finish(finishes: &Arc<AtomicUsize>) -> ProviderFinish {
        let finishes = Arc::clone(finishes);
        Box::new(move || -> BoxFuture<'static, Result<(), ProviderExecutionFailure>> {
            Box::pin(async move {
                finishes.fetch_add(1, Ordering::SeqCst);
                Ok(())
            })
        })
    }

    // Work That Runs Until Its Attempt Cancels It, Reporting When It Started And When It Observed Cancellation.
    fn cancellable_work(started: tokio::sync::oneshot::Sender<()>, observed: tokio::sync::oneshot::Sender<()>) -> ProviderWork {
        Box::new(move |mut cancelled: tokio::sync::watch::Receiver<bool>| -> BoxFuture<'static, ProviderOutcome> {
            Box::pin(async move {
                let _ = started.send(());
                let _ = cancelled.wait_for(|cancelled| *cancelled).await;
                let _ = observed.send(());
                Err(ProviderExecutionFailure::new("cancel", "work observed cancellation"))
            })
        })
    }

    fn stage(settlement: &ProviderSettlement) -> &str {
        match settlement.outcome.as_ref().expect("work started") {
            Ok(_) => "ok",
            Err(failure) => failure.stage.as_str(),
        }
    }

    #[tokio::test]
    async fn provider_invocation_success_removes_entry_and_finalizes_once() {
        let invocations = ProviderInvocations::default();
        let finishes = Arc::new(AtomicUsize::new(0));
        invocations.open(
            "request",
            Box::new(|_: tokio::sync::watch::Receiver<bool>| -> BoxFuture<'static, ProviderOutcome> {
                Box::pin(async { Err(ProviderExecutionFailure::new("done", "work concluded")) })
            }),
            counting_finish(&finishes),
        );

        tokio::time::timeout(BOUND, invocations.attend("request")).await.expect("attempt concludes").expect("attempt succeeds");
        let settlement = invocations.settle("request").await.expect("monitor settles the invocation");

        assert_eq!(stage(&settlement), "done");
        assert!(settlement.finished.is_ok());
        assert_eq!(finishes.load(Ordering::SeqCst), 1);
        assert!(invocations.entries.lock().unwrap().is_empty());
        assert!(invocations.settle("request").await.is_none());
        assert_eq!(finishes.load(Ordering::SeqCst), 1);
        // A Late Attempt Cannot Restart Settled Work.
        assert!(invocations.attend("request").await.is_err());
    }

    #[tokio::test]
    async fn provider_invocation_dropped_attempt_cancels_work_and_finalizes_once() {
        let invocations = Arc::new(ProviderInvocations::default());
        let finishes = Arc::new(AtomicUsize::new(0));
        let (started, work_started) = tokio::sync::oneshot::channel();
        let (observed, work_cancelled) = tokio::sync::oneshot::channel();
        invocations.open("request", cancellable_work(started, observed), counting_finish(&finishes));

        let attempt = tokio::spawn({
            let invocations = Arc::clone(&invocations);
            async move { invocations.attend("request").await }
        });
        tokio::time::timeout(BOUND, work_started).await.expect("work starts").expect("work reports its start");
        // Deadline Or Cancellation Drops The Attempt Future.
        attempt.abort();
        assert!(attempt.await.expect_err("attempt was dropped").is_cancelled());
        tokio::time::timeout(BOUND, work_cancelled).await.expect("dropped attempt cancels work").expect("work observes cancellation");
        assert_eq!(finishes.load(Ordering::SeqCst), 0);

        let settlement = invocations.settle("request").await.expect("monitor settles the invocation");
        assert_eq!(stage(&settlement), "cancel");
        assert_eq!(finishes.load(Ordering::SeqCst), 1);
        assert!(invocations.entries.lock().unwrap().is_empty());
        assert!(invocations.settle("request").await.is_none());
        assert_eq!(finishes.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn provider_invocation_shutdown_settles_outstanding_entries() {
        let invocations = Arc::new(ProviderInvocations::default());
        let running_finishes = Arc::new(AtomicUsize::new(0));
        let idle_finishes = Arc::new(AtomicUsize::new(0));
        let (started, work_started) = tokio::sync::oneshot::channel();
        let (observed, work_cancelled) = tokio::sync::oneshot::channel();
        invocations.open("running", cancellable_work(started, observed), counting_finish(&running_finishes));
        invocations.open(
            "idle",
            Box::new(|_: tokio::sync::watch::Receiver<bool>| -> BoxFuture<'static, ProviderOutcome> {
                panic!("an unattended invocation never starts its work")
            }),
            counting_finish(&idle_finishes),
        );
        let attempt = tokio::spawn({
            let invocations = Arc::clone(&invocations);
            async move { invocations.attend("running").await }
        });
        tokio::time::timeout(BOUND, work_started).await.expect("work starts").expect("work reports its start");

        tokio::time::timeout(BOUND, invocations.settle_all()).await.expect("shutdown settles every invocation");

        assert!(invocations.entries.lock().unwrap().is_empty());
        assert_eq!(running_finishes.load(Ordering::SeqCst), 1);
        assert_eq!(idle_finishes.load(Ordering::SeqCst), 1);
        work_cancelled.await.expect("shutdown cancels running work");
        // The Attempt Still Waiting On The Work Concludes Once It Settled.
        tokio::time::timeout(BOUND, attempt).await.expect("attempt concludes").expect("attempt joins").expect("attempt succeeds");
    }
}
