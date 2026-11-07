/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// The step's file operations. `CopyFiles` copies once and settles. `StartCapture`
// returns once its baseline is taken and leaves the capture to a runtime-owned
// task, so it outlives the step and any reply. That task seals on `StopCapture`,
// when the execution asks, or at shutdown, and keeps its outcome for every stop.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, OnceLock};

// Third Party Imports
use harmos::Context;
use tokio::sync::watch;
use tonic::Status;

// Local Imports
use super::{CaptureEvent, FileCapture};
use crate::app::Lanes;
use crate::operations::hosting::{Host, Serve, Served};
use crate::runtime::operations::reports::status;
use crate::{RuntimeHandle, runtime::Error};
use dokime_domain::model::files::wire::CaptureGap;
use dokime_domain::model::files::wire::CaptureStarted;
use dokime_domain::model::files::wire::CapturedArtifact;
use dokime_domain::model::files::wire::CapturedFiles;
use dokime_domain::model::identity::TestExecutionId;
use dokime_operations::files::CopyFiles;
use dokime_operations::files::StartCapture;
use dokime_operations::files::StopCapture;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

type Outcome = Option<Result<CapturedFiles, Status>>;

/* === Structs === */

// Registered before any plugin attaches, bound to the runtime once it exists.
pub struct Resources {
    runtime: OnceLock<RuntimeHandle>,
    captures: Mutex<BTreeMap<u64, Running>>,
}

// One capture of the registry, kept after sealing so a repeated stop can answer.
struct Running {
    execution: String,
    stop: watch::Sender<bool>,
    outcome: watch::Receiver<Outcome>,
}

// The host side of the operations.
struct Files(Arc<Resources>);

/* === Enums === */

#[harmos::catalog]
enum PreparedOperations {
    Operation0(CopyFiles),
    Operation1(StartCapture),
    Operation2(StopCapture),
}

/* === Traits === */

/* === Implementations === */

impl Resources {
    pub fn bind(&self, runtime: &RuntimeHandle) {
        let _ = self.runtime.set(runtime.clone());
    }

    fn runtime(&self) -> Result<&RuntimeHandle, Error> {
        self.runtime.get().ok_or_else(|| Error::unavailable(super::UNREACHABLE, "the runtime is still starting"))
    }

    // Forgets sealed captures of finished executions, then admits one more
    // running capture within the configured capacity.
    fn admit(&self, runtime: &RuntimeHandle) -> Result<(), Error> {
        let mut captures = self.captures.lock().expect("capture registry lock is healthy");
        {
            let state = runtime.state_read();
            captures.retain(|_, capture| {
                let sealed = capture.outcome.borrow().is_some();
                let finished = TestExecutionId::from_uuid(capture.execution.as_str())
                    .ok()
                    .and_then(|execution| state.executions.get(&execution))
                    .is_none_or(|view| view.finished_at.is_some());
                !(sealed && finished)
            });
        }
        let running = captures.values().filter(|capture| capture.outcome.borrow().is_none()).count();
        if running >= runtime.config().file_capture.maximum_captures {
            return Err(super::capped("the configured file capture capacity is full"));
        }
        Ok(())
    }

    async fn start(&self, request: StartCapture, ctx: &Context<'_, StartCapture>) -> Result<CaptureStarted, Error> {
        let execution = execution_of(ctx.call()).ok_or_else(|| super::invalid("a file capture requires an execution"))?;
        let runtime = self.runtime()?;
        self.admit(runtime)?;
        let mut capture = tokio::select! {
            result = runtime.open_file_capture(&request, &execution) => result?,
            () = ctx.cancelled() => return Err(cancelled()),
        };

        // Take The Baseline; A Refused First Pass Still Seals What It Gave
        let mut events = Vec::new();
        let first = tokio::select! {
            result = capture.poll(&mut events) => result,
            () = ctx.cancelled() => Err(cancelled()),
        };
        if let Err(source) = first {
            runtime.owned_work.spawn(async move {
                if let Err(error) = capture.seal(&mut Vec::new()).await {
                    tracing::warn!(%error, "file capture refused at its start did not seal");
                }
            });
            return Err(source);
        }
        let mut gaps = Vec::new();
        collect_gaps(&mut events, &mut gaps);

        // The Runtime Owns The Capture From Here
        let capture_id = uuid::Uuid::now_v7().as_u64_pair().1;
        let (stop, stopping) = watch::channel(false);
        let (finished, outcome) = watch::channel(None);
        let driven = runtime.clone();
        runtime.owned_work.spawn(async move {
            let answer = drive(driven, capture, stopping, gaps).await;
            finished.send_replace(Some(answer));
        });
        let running = Running { execution, stop, outcome };
        self.captures.lock().expect("capture registry lock is healthy").insert(capture_id, running);
        Ok(CaptureStarted { capture_id })
    }

    async fn stop(&self, request: &StopCapture, ctx: &Context<'_, StopCapture>) -> Result<CapturedFiles, Status> {
        let execution = execution_of(ctx.call()).ok_or_else(|| status(super::invalid("a file capture requires an execution")))?;
        let mut outcome = {
            let captures = self.captures.lock().expect("capture registry lock is healthy");
            let Some(capture) = captures.get(&request.capture_id) else {
                return Err(status(Error::not_found("file_capture_unknown", "no such file capture")));
            };
            if capture.execution != execution {
                return Err(status(super::invalid("the file capture belongs to another execution")));
            }
            capture.stop.send_replace(true);
            capture.outcome.clone()
        };

        // Abandoning This Wait Abandons Only The Reply; Sealing Continues
        tokio::select! {
            answer = outcome.wait_for(Option::is_some) => match answer {
                Ok(answer) => answer.clone().expect("the wait ends on an outcome"),
                Err(_) => Err(status(Error::unavailable(super::UNREACHABLE, "the file capture ended without an outcome"))),
            },
            () = ctx.cancelled() => Err(failure(cancelled())),
        }
    }

    async fn copy(&self, request: CopyFiles, ctx: &Context<'_, CopyFiles>) -> Result<CapturedFiles, Error> {
        let execution = execution_of(ctx.call()).ok_or_else(|| super::invalid("a file copy requires an execution"))?;
        let runtime = self.runtime()?;
        let mut capture = tokio::select! {
            result = runtime.open_file_copy(&request, &execution) => result?,
            () = ctx.cancelled() => return Err(cancelled()),
        };
        let mut events = Vec::new();
        let copied = tokio::select! {
            result = capture.poll(&mut events) => result,
            () = ctx.cancelled() => Err(cancelled()),
        };
        let mut gaps = Vec::new();
        collect_gaps(&mut events, &mut gaps);

        // The Seal Survives Cancellation Of The Reply
        let sealing = runtime.owned_work.spawn(async move {
            let sealed = capture.seal(&mut events).await;
            collect_gaps(&mut events, &mut gaps);
            sealed.map(|()| outcome_of(&capture, gaps))
        });
        let outcome = sealing.await.map_err(|error| Error::Operation(format!("file copy seal task: {error}")))?;
        copied?;
        outcome
    }
}

impl Serve<CopyFiles> for Files {
    fn serve<'a>(&'a self, request: CopyFiles, ctx: Context<'a, CopyFiles>) -> Served<'a, CapturedFiles> {
        Box::pin(async move { self.0.copy(request, &ctx).await.map(Into::into).map_err(failure) })
    }
}

impl Serve<StartCapture> for Files {
    fn serve<'a>(&'a self, request: StartCapture, ctx: Context<'a, StartCapture>) -> Served<'a, CaptureStarted> {
        Box::pin(async move { self.0.start(request, &ctx).await.map(Into::into).map_err(failure) })
    }
}

impl Serve<StopCapture> for Files {
    fn serve<'a>(&'a self, request: StopCapture, ctx: Context<'a, StopCapture>) -> Served<'a, CapturedFiles> {
        Box::pin(async move { self.0.stop(&request, &ctx).await.map(Into::into) })
    }
}

/* === Public Functions === */

// Registers the step built-ins and the Harmos instance controls before any
// plugin that requires them attaches.
pub fn prepare(setup: &mut harmos::Setup<crate::app::Dokime>) -> harmos::Result<Arc<Resources>> {
    setup.operations::<harmos::builtins::Lifecycle>(Lanes::Open);
    let resources = Arc::new(Resources {
        runtime: OnceLock::new(),
        captures: Mutex::new(BTreeMap::new()),
    });
    let files = Arc::new(Files(Arc::clone(&resources)));

    setup.resource::<Host<CopyFiles>>(Host::<CopyFiles>::new(files.clone()));
    setup.resource::<Host<StartCapture>>(Host::<StartCapture>::new(files.clone()));
    setup.resource::<Host<StopCapture>>(Host::<StopCapture>::new(files));

    setup.operations::<PreparedOperations>(Lanes::Open);
    Ok(resources)
}

/* === Private Functions === */

// Passes until something asks for the seal, then seals. A capture that fails
// records why, because nothing may ever stop it to ask.
async fn drive(
    runtime: RuntimeHandle, mut capture: FileCapture, mut stopping: watch::Receiver<bool>, mut gaps: Vec<CaptureGap>,
) -> Result<CapturedFiles, Status> {
    let mut events = Vec::new();
    let mut failure = None;
    let mut changes = runtime.watch_state();
    loop {
        let sealing = tokio::select! {
            _ = stopping.wait_for(|stopped| *stopped) => true,
            _ = changes.wait_until(|snapshot| snapshot.state().shutting_down) => true,
            asked = capture.wait_for_poll() => asked,
        };
        if sealing {
            break;
        }
        if let Err(source) = capture.poll(&mut events).await {
            failure = Some(source);
            break;
        }
        collect_gaps(&mut events, &mut gaps);
    }
    let sealed = capture.seal(&mut events).await;
    collect_gaps(&mut events, &mut gaps);
    if let Err(error) = &sealed {
        tracing::warn!(%error, "file capture did not seal every file");
    }
    if let Some(source) = failure.or(sealed.err()) {
        capture.record_failure(&source).await;
        return Err(status(source));
    }
    Ok(outcome_of(&capture, gaps))
}

fn outcome_of(capture: &FileCapture, gaps: Vec<CaptureGap>) -> CapturedFiles {
    let artifacts = capture
        .sealed()
        .iter()
        .map(|record| CapturedArtifact {
            artifact: record.filename.clone(),
            size_bytes: record.size_bytes,
        })
        .collect();
    CapturedFiles { artifacts, gaps }
}

fn execution_of(call: &harmos::operations::CallContext) -> Option<String> {
    call.value::<crate::execution::ExecutionScope>().ok().flatten().map(|scope| scope.execution)
}

fn cancelled() -> Error {
    Error::cancelled("file_capture_cancelled", "the file operation was cancelled")
}

fn failure(error: Error) -> Status {
    if error.code() == Some("file_capture_cancelled") {
        let failure = harmos::Failure::new(harmos::operations::FailureKind::Cancelled, error.to_string());
        let mut status = Status::cancelled(failure.to_string());
        status.set_source(Arc::new(failure));
        return status;
    }
    status(error)
}

fn collect_gaps(events: &mut Vec<CaptureEvent>, gaps: &mut Vec<CaptureGap>) {
    for event in events.drain(..) {
        if let CaptureEvent::Gap { path, offset, reason } = event {
            gaps.push(CaptureGap { path, offset, reason });
        }
    }
}

/* === Tests === */
