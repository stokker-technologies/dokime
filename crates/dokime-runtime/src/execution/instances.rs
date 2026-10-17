/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// What a step submits through `ctx.host()`: one handle over an instance the
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
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

// Third Party Imports
use harmos::Message;
use harmos::operations::instance::{Id, Outcome, Phase, Status};
use harmos::operations::{Callable, Host, Instance, Response};
use harmos::sidecar::{CallError, Supervisor};
use tokio::sync::watch;
use tokio::task::JoinHandle;

// Local Imports
use crate::host::StepHandleCancelled;
use dokime_domain::model::files::wire::FileWatchSourcePayload;
use dokime_domain::operations::files::WatchFile;
use dokime_domain::operations::process::RunProcess;

/* === Constants === */
// The JSON-RPC server error code of a refusal raised on the step side.
const STEP_REFUSAL: i64 = -32_000;

// How long the sweep waits for one cancelled instance to settle.
const SETTLE: Duration = Duration::from_secs(30);

// Local instance ids count down from the top, apart from the host's.
const LOCAL_IDS: u64 = u64::MAX;

/* === Macros === */

/* === Type Aliases === */
type HostInstance<O> = <Supervisor as Host>::Instance<O>;

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
#[derive(Clone, Default)]
pub(crate) struct Handles(Arc<Mutex<Ledger>>);

#[derive(Default)]
struct Ledger {
    step: Option<StepIdentity>,
    live: Vec<Arc<dyn Sweep>>,
}

#[must_use = "dropping an unsettled operation instance requests its cancellation"]
pub struct StepInstance<O: Callable> {
    id: Id,
    slot: Arc<Slot<O>>,
    released: bool,
}

struct Slot<O: Callable> {
    running: tokio::sync::Mutex<Option<Running<O>>>,
    kind: &'static str,
    name: String,
}

// An instance this process serves: its task, and the cancellation it heeds.
pub(crate) struct LocalRun {
    cancel: watch::Sender<bool>,
    task: JoinHandle<Result<Vec<u8>, String>>,
    // A served built-in settles itself when cancelled; a harness answer that
    // never looks at the signal is aborted instead.
    cooperative: bool,
}

/* === Enums === */
enum Running<O: Callable> {
    Host(HostInstance<O>),
    Local(LocalRun),
}

/* === Traits === */
trait Sweep: Send + Sync {
    fn sweep(&self, step: String) -> Sweeping<'_>;
}

/* === Implementations === */
impl Handles {
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
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl LocalRun {
    pub(crate) fn spawn<F>(serve: impl FnOnce(watch::Receiver<bool>) -> F, cooperative: bool) -> Self
    where
        F: Future<Output = Result<Vec<u8>, String>> + Send + 'static,
    {
        let (cancel, cancelled) = watch::channel(false);
        Self { task: tokio::spawn(serve(cancelled)), cancel, cooperative }
    }

    fn request_cancel(&self) {
        self.cancel.send_replace(true);
        if !self.cooperative {
            self.task.abort();
        }
    }

    fn status(&self) -> Status {
        let finished = self.task.is_finished();
        Status {
            phase: if finished { Phase::Settled } else { Phase::Running },
            outcome: None,
            cancellation_requested: *self.cancel.borrow(),
            attempt: 1,
        }
    }

    async fn join(self, method: &'static str) -> Result<Vec<u8>, CallError> {
        match self.task.await {
            Ok(Ok(bytes)) => Ok(bytes),
            Ok(Err(message)) => Err(refused(message)),
            Err(error) if error.is_cancelled() => Err(refused(format!("`{method}` was cancelled"))),
            Err(error) => Err(refused(format!("`{method}` panicked: {error}"))),
        }
    }
}

impl<O: Callable> StepInstance<O> {
    pub(crate) fn host(handles: &Handles, instance: HostInstance<O>, request: &[u8]) -> Self {
        let id = instance.id();
        Self::tracked(handles, id, Running::Host(instance), request)
    }

    pub(crate) fn local(handles: &Handles, run: LocalRun, request: &[u8]) -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(LOCAL_IDS);
        let id = Id::new(NEXT.fetch_sub(1, Ordering::Relaxed));
        Self::tracked(handles, id, Running::Local(run), request)
    }

    fn tracked(handles: &Handles, id: Id, running: Running<O>, request: &[u8]) -> Self {
        let (kind, name) = label::<O>(request);
        let slot = Arc::new(Slot { running: tokio::sync::Mutex::new(Some(running)), kind, name });
        handles.track(Arc::clone(&slot) as Arc<dyn Sweep>);
        Self { id, slot, released: false }
    }

    pub fn id(&self) -> Id {
        self.id
    }

    pub async fn status(&self) -> Result<Status, CallError> {
        match self.slot.running.lock().await.as_ref() {
            Some(Running::Host(instance)) => instance.status().await,
            Some(Running::Local(run)) => Ok(run.status()),
            None => Ok(swept()),
        }
    }

    pub async fn cancel(&self) -> Result<(), CallError> {
        match self.slot.running.lock().await.as_ref() {
            Some(Running::Host(instance)) => instance.cancel().await,
            Some(Running::Local(run)) => {
                run.request_cancel();
                Ok(())
            }
            None => Ok(()),
        }
    }

    pub async fn join(mut self) -> Result<Response<O::Response>, CallError> {
        self.released = true;
        let running = self.slot.running.lock().await.take();
        match running {
            Some(Running::Host(instance)) => instance.join().await,
            Some(Running::Local(run)) => {
                let bytes = run.join(O::ID).await?;
                let response = O::Response::decode(bytes.as_slice())
                    .map_err(|error| CallError::Malformed { method: O::ID, message: error.to_string() })?;
                Ok(Response::new(response))
            }
            None => Err(refused(format!("`{}` was cancelled when its step ended", O::ID))),
        }
    }

    pub async fn cancel_and_join(self) -> Result<Response<O::Response>, CallError> {
        self.cancel().await?;
        self.join().await
    }

    // Keeps the instance running without its handle until the step ends; the
    // step-end sweep then cancels it with a warning.
    pub fn detach(mut self) -> Id {
        self.released = true;
        self.id
    }
}

impl<O: Callable> Drop for StepInstance<O> {
    fn drop(&mut self) {
        if self.released {
            return;
        }
        // The sweep holding the slot is already settling it.
        let Ok(mut running) = self.slot.running.try_lock() else {
            return;
        };
        match running.take() {
            // The host's handle requests cancellation when it drops unsettled.
            Some(Running::Host(instance)) => drop(instance),
            Some(Running::Local(run)) => run.request_cancel(),
            None => {}
        }
    }
}

impl<O: Callable> Instance<O> for StepInstance<O> {
    type Error = CallError;

    fn id(&self) -> Id {
        self.id
    }

    fn status(&self) -> impl Future<Output = Result<Status, CallError>> + Send {
        StepInstance::status(self)
    }

    fn cancel(&self) -> impl Future<Output = Result<(), CallError>> + Send {
        StepInstance::cancel(self)
    }

    fn join(self) -> impl Future<Output = Result<Response<O::Response>, CallError>> + Send {
        StepInstance::join(self)
    }

    fn detach(self) -> Id {
        StepInstance::detach(self)
    }
}

impl<O: Callable> Sweep for Slot<O> {
    fn sweep(&self, step: String) -> Sweeping<'_> {
        Box::pin(async move {
            let running = self.running.lock().await.take()?;
            let settled = match running {
                Running::Host(instance) => {
                    if instance.status().await.is_ok_and(|status| status.is_settled()) {
                        // Settled on its own; dropping the handle releases it.
                        return None;
                    }
                    tokio::time::timeout(SETTLE, instance.cancel_and_join()).await.is_ok()
                }
                Running::Local(run) => {
                    if run.task.is_finished() {
                        return None;
                    }
                    run.request_cancel();
                    tokio::time::timeout(SETTLE, run.join(O::ID)).await.is_ok()
                }
            };
            if !settled {
                tracing::warn!(
                    kind = self.kind,
                    name = %self.name,
                    "an instance cancelled at step end did not settle in time"
                );
            }
            Some(StepHandleCancelled { step, kind: self.kind.to_owned(), name: self.name.clone() })
        })
    }
}

/* === Public Functions === */
pub(crate) fn refused(message: impl Into<String>) -> CallError {
    CallError::Refused { code: STEP_REFUSAL, failure: refusal_failure(message) }
}

pub(crate) fn refusal_failure(message: impl Into<String>) -> harmos::Failure {
    harmos::Failure::new(harmos::operations::FailureKind::Permanent, message)
}

/* === Private Functions === */
fn swept() -> Status {
    Status { phase: Phase::Settled, outcome: Some(Outcome::Cancelled), cancellation_requested: true, attempt: 1 }
}

// What a warning calls the instance: the program a process runs, the
// artifact a watch fills, or the operation itself.
fn label<O: Callable>(request: &[u8]) -> (&'static str, String) {
    if O::ID == <RunProcess as harmos::operations::OperationDeclaration>::ID
        && let Ok(process) = RunProcess::decode(request)
    {
        return ("process", process.program);
    }
    if O::ID == <WatchFile as harmos::operations::OperationDeclaration>::ID
        && let Ok(watch) = WatchFile::decode(request)
    {
        let source = match watch.source.and_then(|source| source.payload) {
            Some(FileWatchSourcePayload::Local(local)) => local.root,
            Some(FileWatchSourcePayload::Ftp(ftp)) => ftp.url,
            None => String::new(),
        };
        return ("watch", watch.artifact.unwrap_or(source));
    }
    ("operation", O::ID.to_owned())
}

/* === Tests === */
