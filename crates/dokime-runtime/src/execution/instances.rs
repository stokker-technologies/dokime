/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// What a step submits through `ctx.operation(request).start()`: one handle over an instance the
// host runs or one this process serves, and the step's ledger of every
// instance still running. A handle dropped unsettled cancels its instance; a
// detached one, or one kept past its step, is cancelled by the step-end sweep
// with a `test.step.handle_cancelled` warning.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

// Third Party Imports
use harmos::Message;
use harmos::operations::instance::{Id, Outcome, Phase, Status};
use harmos::operations::{Instance, OperationDeclaration, OperationHost, Response};
use harmos::sidecar::{CallError, Supervisor};
use tokio::sync::watch;
use tokio::task::JoinHandle;

// Local Imports
use crate::host::StepHandleCancelled;
use dokime_operations::process::RunProcess;

/* === Constants === */

// The JSON-RPC server error code of a refusal raised on the step side.
const STEP_REFUSAL: i64 = -32_000;

// Local instance ids count down from the top, apart from the host's.
const LOCAL_IDS: u64 = u64::MAX;

/* === Macros === */

/* === Type Aliases === */

type HostInstance<O> = <Supervisor as OperationHost>::Instance<O>;
type Sweeping<'a> = Pin<Box<dyn Future<Output = Option<StepHandleCancelled>> + Send + 'a>>;

/* === Structs === */

// The running execution as a child process and a warning name it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ExecutionIdentity {
    pub execution_id: String,
    pub cycle_id: String,
    pub template_id: String,
    pub plugin_id: String,
}

// The step a stage runs, while it runs.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StepIdentity {
    pub step: String,
    pub step_id: String,
    pub attempt: String,
}

// The live instances of one execution's current step.
#[derive(Clone)]
pub(crate) struct Handles {
    ledger: Arc<Mutex<Ledger>>,
    settlement: Duration,
}

#[derive(Default)]
struct Ledger {
    step: Option<StepIdentity>,
    live: Vec<Arc<dyn Sweep>>,
}

#[must_use = "dropping an unsettled operation instance requests its cancellation"]
pub struct StepInstance<O: OperationDeclaration> {
    id: Id,
    slot: Arc<Slot<O>>,
    released: bool,
}

struct Slot<O: OperationDeclaration> {
    settlement: Duration,
    running: tokio::sync::Mutex<Option<Running<O>>>,
    local_cancel: Option<(watch::Sender<bool>, tokio::task::AbortHandle)>,
    kind: &'static str,
    name: String,
}

// An instance this process serves: its task, and the cancellation it heeds.
pub(crate) struct LocalRun {
    cancel: watch::Sender<bool>,
    task: Option<JoinHandle<Result<Vec<u8>, CallError>>>,
    id: Id,
    attempt: Arc<AtomicUsize>,
    outcome: Arc<Mutex<Option<Outcome>>>,
}

struct LocalAttempt {
    journal: Option<Arc<dyn crate::transport::Journal>>,
    operation: &'static str,
    instance: u64,
    stage: String,
    attempt: usize,
    outcome: Outcome,
    observed: Arc<Mutex<Option<Outcome>>>,
}

/* === Enums === */

enum Running<O: OperationDeclaration> {
    Host(HostInstance<O>),
    Local(LocalRun),
}

/* === Traits === */

trait Sweep: Send + Sync {
    fn sweep(&self, step: String) -> Sweeping<'_>;
}

/* === Implementations === */

impl Handles {
    pub(crate) fn new() -> Self {
        Self {
            ledger: Arc::new(Mutex::new(Ledger::default())),
            settlement: Duration::from_secs(30),
        }
    }

    pub(crate) fn begin(&self, step: StepIdentity) {
        self.locked().step = Some(step);
    }

    pub(crate) fn step(&self) -> Option<StepIdentity> {
        self.locked().step.clone()
    }

    fn track(&self, slot: Arc<dyn Sweep>) {
        self.locked().live.push(slot);
    }

    // Cancels everything the step left running, one warning each, and ends
    // the step's ledger.
    pub(crate) async fn sweep(&self) -> Vec<StepHandleCancelled> {
        let (step, live) = {
            let mut ledger = self.locked();
            let step = ledger.step.take().map(|step| step.step).unwrap_or_default();
            (step, std::mem::take(&mut ledger.live))
        };
        let mut cancelled = Vec::new();
        for slot in live {
            if let Some(warning) = slot.sweep(step.clone()).await {
                cancelled.push(warning);
            }
        }
        cancelled
    }

    fn locked(&self) -> std::sync::MutexGuard<'_, Ledger> {
        self.ledger.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl LocalRun {
    pub(crate) fn spawn<O: OperationDeclaration, F>(
        mut serve: impl FnMut(watch::Receiver<bool>) -> F + Send + 'static, cooperative: bool, retry: Option<harmos::RetryPolicy>,
        journal: Option<Arc<dyn crate::transport::Journal>>, stage: String,
    ) -> Result<Self, CallError>
    where
        F: Future<Output = Result<Vec<u8>, String>> + Send + 'static,
    {
        use harmos::operations::FailureKind;
        let mut policy = O::POLICY;
        if let Some(retry) = retry {
            policy.retries = retry;
        }
        policy.validate().map_err(|error| refused(error.to_string()))?;
        static NEXT: AtomicU64 = AtomicU64::new(LOCAL_IDS);
        let id = Id::new(NEXT.fetch_sub(1, Ordering::Relaxed));
        let (cancel, mut cancelled) = watch::channel(false);
        let attempt = Arc::new(AtomicUsize::new(0));
        let outcome = Arc::new(Mutex::new(None));
        let progress = attempt.clone();
        let observed = outcome.clone();
        let deadline = tokio::time::Instant::now() + policy.timeout;
        let task = tokio::spawn(async move {
            let mut attempt = 0;
            loop {
                if *cancelled.borrow() {
                    return Err(refused("operation cancelled"));
                }
                let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
                if remaining.is_zero() {
                    return Err(refused("operation deadline elapsed"));
                }
                progress.store(attempt, Ordering::Relaxed);
                let mut evidence = LocalAttempt {
                    journal: journal.clone(),
                    operation: O::ID,
                    instance: id.value(),
                    stage: stage.clone(),
                    attempt,
                    outcome: Outcome::Cancelled,
                    observed: observed.clone(),
                };
                let (interrupt, signal) = watch::channel(false);
                let mut body = Box::pin(serve(signal));
                let answer = tokio::select! {
                    biased;
                    _ = cancelled.changed() => None,
                    answer = tokio::time::timeout(policy.attempt_timeout.min(remaining), &mut body) => Some(answer),
                };
                let (error, classification) = match answer {
                    Some(Ok(Ok(bytes))) => {
                        evidence.outcome = Outcome::Succeeded;
                        return Ok(bytes);
                    }
                    Some(Ok(Err(message))) => {
                        evidence.outcome = Outcome::Failed;
                        (refused(message), FailureKind::Permanent)
                    }
                    interruption => {
                        interrupt.send_replace(true);
                        let cleanup = if cooperative {
                            tokio::time::timeout(
                                policy.cancellation_timeout.min(deadline.saturating_duration_since(tokio::time::Instant::now())),
                                &mut body,
                            )
                            .await
                            .ok()
                        } else {
                            None
                        };
                        if interruption.is_none() {
                            evidence.outcome = Outcome::Cancelled;
                            return match cleanup {
                                Some(answer) => answer.map_err(refused),
                                None => Err(refused("operation cancelled")),
                            };
                        }
                        evidence.outcome = Outcome::TimedOut;
                        let failure =
                            harmos::Failure::new(FailureKind::Ambiguous, "operation attempt deadline elapsed").in_operation(O::ID);
                        (CallError::Refused { code: -32_607, failure }, FailureKind::Ambiguous)
                    }
                };
                drop(body);
                drop(evidence);
                let Some(delay) = policy.retries.retry_after(attempt, classification, O::REPLAY, id.value()) else {
                    return Err(error);
                };
                tokio::select! {
                    biased;
                    _ = cancelled.changed() => return Err(refused("operation cancelled")),
                    waited = tokio::time::timeout_at(deadline, tokio::time::sleep(delay)) => {
                        if waited.is_err() { return Err(error); }
                    }
                }
                attempt += 1;
            }
        });
        Ok(Self {
            task: Some(task),
            cancel,
            id,
            attempt,
            outcome,
        })
    }

    fn request_cancel(&self) {
        self.cancel.send_replace(true);
    }

    fn status(&self) -> Status {
        let finished = self.task.as_ref().is_none_or(JoinHandle::is_finished);
        Status {
            phase: if finished { Phase::Settled } else { Phase::Running },
            outcome: if finished { *self.outcome.lock().unwrap_or_else(PoisonError::into_inner) } else { None },
            cancellation_requested: *self.cancel.borrow(),
            attempt: u32::try_from(self.attempt.load(Ordering::Relaxed)).unwrap_or(u32::MAX),
        }
    }

    async fn join(&mut self, method: &'static str) -> Result<Vec<u8>, CallError> {
        let answer = self.task.as_mut().expect("owned local task").await;
        self.task.take();
        match answer {
            Ok(answer) => answer.map_err(|error| match error {
                CallError::Refused { code, failure } => CallError::Refused {
                    code,
                    failure: failure.in_operation(method),
                },
                error => error,
            }),
            Err(error) if error.is_cancelled() => Err(refused(format!("`{method}` was cancelled"))),
            Err(error) => Err(refused(format!("`{method}` panicked: {error}"))),
        }
    }
}

impl Drop for LocalRun {
    fn drop(&mut self) {
        if self.task.is_some() {
            self.request_cancel();
        }
    }
}

impl Drop for LocalAttempt {
    fn drop(&mut self) {
        if std::thread::panicking() {
            self.outcome = Outcome::Failed;
        }
        *self.observed.lock().unwrap_or_else(PoisonError::into_inner) = Some(self.outcome);
        if let Some(journal) = &self.journal {
            let entry = super::log::operation_attempt(self.operation, self.instance, None, &self.stage, self.attempt, self.outcome);
            let level = super::log::Level::try_from(entry.level).expect("operation attempt has a known log level");
            journal.log(level, entry.message);
        }
    }
}

impl<O: OperationDeclaration> StepInstance<O> {
    pub(crate) fn host(handles: &Handles, instance: HostInstance<O>, request: &[u8]) -> Self {
        let id = instance.id();
        Self::tracked(handles, id, Running::Host(instance), request)
    }

    pub(crate) fn local(handles: &Handles, run: LocalRun, request: &[u8]) -> Self {
        let id = run.id;
        Self::tracked(handles, id, Running::Local(run), request)
    }

    fn tracked(handles: &Handles, id: Id, running: Running<O>, request: &[u8]) -> Self {
        let (kind, name) = label::<O>(request);
        let local_cancel = match &running {
            Running::Local(run) => Some((run.cancel.clone(), run.task.as_ref().expect("owned local task").abort_handle())),
            Running::Host(_) => None,
        };
        let slot = Arc::new(Slot {
            settlement: handles.settlement,
            running: tokio::sync::Mutex::new(Some(running)),
            local_cancel,
            kind,
            name,
        });
        handles.track(Arc::clone(&slot) as Arc<dyn Sweep>);
        Self { id, slot, released: false }
    }

    pub fn id(&self) -> Id {
        self.id
    }

    pub async fn status(&self) -> Result<Status, crate::Error> {
        match self.slot.running.lock().await.as_ref() {
            Some(Running::Host(instance)) => instance.status().await.map_err(crate::Error::from),
            Some(Running::Local(run)) => Ok(run.status()),
            None => Ok(swept()),
        }
    }

    pub async fn request_cancel(&self) -> Result<(), crate::Error> {
        match self.slot.running.lock().await.as_ref() {
            Some(Running::Host(instance)) => instance.request_cancel().await.map_err(crate::Error::from),
            Some(Running::Local(run)) => {
                run.request_cancel();
                Ok(())
            }
            None => Ok(()),
        }
    }

    pub async fn join(mut self) -> Result<Response<O::Response>, crate::Error> {
        let slot = self.slot.clone();
        let mut running = slot.running.lock().await;
        if let Some(Running::Local(run)) = running.as_mut() {
            // Keep The Local Task In The Stage Ledger While Its Join Is Borrowed.
            let answer = run.join(O::ID).await;
            running.take();
            self.released = true;
            let response = O::Response::decode(&answer?).map_err(|error| CallError::Malformed {
                method: O::ID,
                message: error.to_string(),
            })?;
            return Ok(Response::new(response));
        }
        let instance = running.take();
        drop(running);
        self.released = true;
        match instance {
            Some(Running::Host(instance)) => instance.join().await.map_err(crate::Error::from),
            Some(Running::Local(_)) => unreachable!("local tasks are joined in their stage slot"),
            None => Err(refused(format!("`{}` was cancelled when its step ended", O::ID)).into()),
        }
    }

    pub async fn cancel(self) -> Result<Response<O::Response>, crate::Error> {
        self.request_cancel().await?;
        self.join().await
    }

    // Keeps the instance running without its handle until the step ends; the
    // step-end sweep then cancels it with a warning.
    pub fn detach(mut self) -> Id {
        self.released = true;
        self.id
    }
}

impl<O: OperationDeclaration> Drop for StepInstance<O> {
    fn drop(&mut self) {
        if self.released {
            return;
        }
        if let Some((cancel, _)) = &self.slot.local_cancel {
            cancel.send_replace(true);
            return;
        }
        // The sweep holding the slot is already settling it.
        let Ok(mut running) = self.slot.running.try_lock() else {
            return;
        };
        match running.as_ref() {
            // The Host's Handle Requests Cancellation When It Drops Unsettled.
            Some(Running::Host(_)) => {
                running.take();
            }
            Some(Running::Local(run)) => run.request_cancel(),
            None => {}
        }
    }
}

impl<O: OperationDeclaration> Instance<O> for StepInstance<O> {
    type Error = crate::Error;

    fn id(&self) -> Id {
        self.id
    }

    fn status(&self) -> impl Future<Output = Result<Status, crate::Error>> + Send {
        StepInstance::status(self)
    }

    fn request_cancel(&self) -> impl Future<Output = Result<(), crate::Error>> + Send {
        StepInstance::request_cancel(self)
    }

    fn join(self) -> impl Future<Output = Result<Response<O::Response>, crate::Error>> + Send {
        StepInstance::join(self)
    }

    fn detach(self) -> Id {
        StepInstance::detach(self)
    }
}

impl<O: OperationDeclaration> Sweep for Slot<O> {
    fn sweep(&self, step: String) -> Sweeping<'_> {
        Box::pin(async move {
            let local_unsettled = self.local_cancel.as_ref().is_some_and(|(_, task)| !task.is_finished());
            if local_unsettled && let Some((cancel, _)) = &self.local_cancel {
                cancel.send_replace(true);
            }
            let Some(running) = self.running.lock().await.take() else {
                return local_unsettled.then(|| StepHandleCancelled {
                    step,
                    kind: self.kind.to_owned(),
                    name: self.name.clone(),
                });
            };
            let settled = match running {
                Running::Host(instance) => {
                    if instance.status().await.is_ok_and(|status| status.is_settled()) {
                        // Settled on its own; dropping the handle releases it.
                        return None;
                    }
                    tokio::time::timeout(self.settlement, instance.cancel()).await.is_ok()
                }
                Running::Local(mut run) => {
                    if !local_unsettled {
                        return None;
                    }
                    run.request_cancel();
                    tokio::time::timeout(self.settlement, run.join(O::ID)).await.is_ok()
                }
            };
            if !settled {
                tracing::warn!(
                    kind = self.kind,
                    name = %self.name,
                    "an instance cancelled at step end did not settle in time"
                );
            }
            Some(StepHandleCancelled {
                step,
                kind: self.kind.to_owned(),
                name: self.name.clone(),
            })
        })
    }
}

/* === Public Functions === */

pub(crate) fn refused(message: impl Into<String>) -> CallError {
    CallError::Refused {
        code: STEP_REFUSAL,
        failure: refusal_failure(message),
    }
}

pub(crate) fn refusal_failure(message: impl Into<String>) -> harmos::Failure {
    harmos::Failure::new(harmos::operations::FailureKind::Permanent, message)
}

/* === Private Functions === */

fn swept() -> Status {
    Status {
        phase: Phase::Settled,
        outcome: Some(Outcome::Cancelled),
        cancellation_requested: true,
        attempt: 1,
    }
}

// What a warning calls the instance: the program a process runs, the
// artifact a watch fills, or the operation itself.
fn label<O: OperationDeclaration>(request: &[u8]) -> (&'static str, String) {
    if O::ID == <RunProcess as harmos::operations::OperationDeclaration>::ID
        && let Ok(process) = RunProcess::decode(request)
    {
        return ("process", process.program);
    }
    ("operation", O::ID.to_owned())
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::Journal;
    use std::sync::atomic::AtomicBool;

    #[derive(Default)]
    struct Evidence {
        entries: Mutex<Vec<serde_json::Value>>,
        levels: Mutex<Vec<super::super::log::Level>>,
    }

    impl Journal for Evidence {
        fn log(&self, level: super::super::log::Level, message: String) {
            self.entries.lock().unwrap().push(serde_json::from_str(&message).unwrap());
            self.levels.lock().unwrap().push(level);
        }
        fn record(&self, _: dokime_domain::model::records::publication::RecordPublishRequest) {}
    }

    #[crate::harmos::operation(harmos_path = crate::harmos, id = "invocation.local", response = u64,
        replay = crate::harmos::operations::ReplaySafety::Idempotent,
        policy = crate::harmos::OperationPolicy {
            timeout: Duration::from_secs(5), attempt_timeout: Duration::from_millis(10),
            cancellation_timeout: Duration::from_millis(100), retries: crate::harmos::RetryPolicy::Never,
        })]
    struct Read {}

    #[crate::harmos::operation(harmos_path = crate::harmos, id = "invocation.unknown", response = u64, policy = Read::POLICY)]
    struct Unknown {}

    #[tokio::test(start_paused = true)]
    async fn local_retry_retains_defaults_deadlines_and_attempt_evidence() {
        let evidence = Arc::new(Evidence::default());
        let mut default =
            LocalRun::spawn::<Read, _>(|_| std::future::pending(), false, None, Some(evidence.clone()), "step/read".into()).unwrap();
        assert!(default.join(Read::ID).await.is_err());
        let calls = Arc::new(AtomicUsize::new(0));
        let invoked = calls.clone();
        let configured = harmos::RetryPolicy::OnTransient {
            max_retries: 1,
            backoff: harmos::Backoff::Fixed {
                delay: Duration::from_millis(1),
                jitter: harmos::Jitter::None,
            },
        };
        let mut retrying = LocalRun::spawn::<Read, _>(
            move |_| {
                let first = invoked.fetch_add(1, Ordering::SeqCst) == 0;
                Box::pin(async move { if first { std::future::pending().await } else { Ok(42_u64.encode()) } })
            },
            false,
            Some(configured),
            Some(evidence.clone()),
            "step/read".into(),
        )
        .unwrap();
        assert_eq!(u64::decode(&retrying.join(Read::ID).await.unwrap()).unwrap(), 42);
        assert_eq!(calls.load(Ordering::SeqCst), 2);
        let mut later =
            LocalRun::spawn::<Read, _>(|_| std::future::pending(), false, None, Some(evidence.clone()), "step/read".into()).unwrap();
        assert!(later.join(Read::ID).await.is_err());
        let entries = evidence.entries.lock().unwrap();
        let attempts =
            entries.iter().map(|entry| (entry["attempt"].as_u64().unwrap(), entry["outcome"].as_str().unwrap())).collect::<Vec<_>>();
        assert_eq!(attempts, [(0, "timed_out"), (0, "timed_out"), (1, "succeeded"), (0, "timed_out")]);
        use super::super::log::Level;
        assert_eq!(*evidence.levels.lock().unwrap(), [Level::Warn, Level::Warn, Level::Info, Level::Warn]);
        assert_eq!(entries[1]["instance"], entries[2]["instance"]);
        assert_ne!(entries[0]["instance"], entries[1]["instance"]);
        assert!(entries.iter().all(|entry| entry["operation"] == Read::ID && entry["stage"] == "step/read"));
    }

    #[tokio::test(start_paused = true)]
    async fn abandoned_local_join_keeps_cancellation_in_the_stage_ledger() {
        let evidence = Arc::new(Evidence::default());
        let started = Arc::new(tokio::sync::Notify::new());
        let completed = Arc::new(AtomicBool::new(false));
        let entered = started.clone();
        let settled = completed.clone();
        let run = LocalRun::spawn::<Read, _>(
            move |mut cancelled| {
                let entered = entered.clone();
                let settled = settled.clone();
                async move {
                    entered.notify_one();
                    cancelled.changed().await.unwrap();
                    settled.store(true, Ordering::SeqCst);
                    Ok(0_u64.encode())
                }
            },
            true,
            None,
            Some(evidence.clone()),
            "step/wait".into(),
        )
        .unwrap();
        let handles = Handles::new();
        let instance = StepInstance::<Read>::local(&handles, run, &Read {}.encode());
        let joining = tokio::spawn(instance.join());
        started.notified().await;
        joining.abort();
        assert!(joining.await.unwrap_err().is_cancelled());
        handles.sweep().await;
        assert!(completed.load(Ordering::SeqCst));
        assert!(handles.sweep().await.is_empty());
        let entries = evidence.entries.lock().unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0]["outcome"], "cancelled");
        assert_eq!(*evidence.levels.lock().unwrap(), [super::super::log::Level::Debug]);
    }

    #[tokio::test(start_paused = true)]
    async fn stage_sweep_cancels_a_concurrent_local_join() {
        let evidence = Arc::new(Evidence::default());
        let started = Arc::new(tokio::sync::Notify::new());
        let completed = Arc::new(AtomicBool::new(false));
        let entered = started.clone();
        let settled = completed.clone();
        let run = LocalRun::spawn::<Read, _>(
            move |mut cancelled| {
                let entered = entered.clone();
                let settled = settled.clone();
                async move {
                    entered.notify_one();
                    cancelled.changed().await.unwrap();
                    settled.store(true, Ordering::SeqCst);
                    Ok(0_u64.encode())
                }
            },
            true,
            None,
            Some(evidence.clone()),
            "step/join".into(),
        )
        .unwrap();
        let handles = Handles::new();
        let instance = StepInstance::<Read>::local(&handles, run, &Read {}.encode());
        let joining = tokio::spawn(instance.join());
        started.notified().await;
        let warnings = handles.sweep().await;
        assert_eq!(*joining.await.unwrap().unwrap(), 0);
        assert!(completed.load(Ordering::SeqCst));
        assert_eq!(warnings.len(), 1);
        assert!(handles.sweep().await.is_empty());
        let entries = evidence.entries.lock().unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0]["outcome"], "cancelled");
    }

    #[tokio::test]
    async fn unsafe_local_override_refuses_before_the_harness_action() {
        let calls = Arc::new(AtomicUsize::new(0));
        let invoked = calls.clone();
        let answer: super::super::calls::Answering = Arc::new(move |_| {
            invoked.fetch_add(1, Ordering::SeqCst);
            Box::pin(async { Ok(7_u64.encode()) })
        });
        let caller = super::super::Caller::local([(Unknown::ID.to_owned(), answer)].into());
        let host = caller.step_host("step/unknown", None);
        let configured = harmos::RetryPolicy::OnTransient {
            max_retries: 1,
            backoff: harmos::Backoff::Fixed {
                delay: Duration::from_millis(1),
                jitter: harmos::Jitter::None,
            },
        };
        assert!(host.operation(Unknown {}).retry(configured).await.is_err());
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        assert_eq!(*host.operation(Unknown {}).await.unwrap(), 7);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }
}
