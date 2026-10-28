/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// File watch operations: sources polled during an execution, each matched
// file sealed as one of its artifacts.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::Arc;

// Third Party Imports
use dokime::harmos::operations::StreamOutput;
use dokime::harmos::{Context, RegistrationExt};
use tonic::Status;

// Local Imports
use super::content::{content_failure, metadata};
use dokime::domain::model::diagnostics::admission::AdmissionFailure;
use dokime::operations::files as contract;
use dokime::operations::hosting::{Host, Serve, Served};
use dokime::{
    RuntimeHandle,
    domains::files::{CAPPED, FileWatch as Watch, INVALID, UNREACHABLE, UNSUPPORTED, UNTRUSTED, WatchEvent},
    runtime::Error,
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct Resources {
    runtime: RuntimeHandle,
}

// Seals What An Abandoned Stream Captured: A Cancelled Or Failed Call Drops The Future.
struct Sealing {
    watch: Option<Watch>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Serve<contract::FileWatch> for Resources {
    fn serve<'a>(&'a self, request: contract::FileWatch, ctx: Context<'a, contract::FileWatch>) -> Served<'a, ()> {
        Box::pin(async move {
            let revoked = crate::grpc::access::revocation(ctx.call());
            crate::grpc::access::during(revoked, async {
                // A Step's Call Carries Its Execution; A Listener Call Names It
                let scoped = ctx.call().value::<dokime::execution::ExecutionScope>().ok().flatten().map(|scope| scope.execution);
                let watch = self.runtime.open_file_watch(&request, scoped.as_deref()).await.map_err(failure)?;
                let mut guard = Sealing { watch: Some(watch) };
                let watch = guard.watch.as_mut().expect("file watch was just opened");

                // Poll Until Complete, Bounded, Or Sealed By The Execution;
                // Cancellation Seals Through `Sealing`
                let mut events = Vec::new();
                loop {
                    let polled = watch.poll(&mut events).await;
                    if let Err(source) = polled {
                        let sealed = watch.seal(&mut events).await;
                        let _ = send_events(ctx.streams(), &mut events).await;
                        guard.watch = None;
                        if let Err(seal) = sealed {
                            tracing::warn!(error = %seal, "failed file watch did not seal every file");
                        }
                        return Err(failure(source));
                    }
                    send_events(ctx.streams(), &mut events).await?;
                    if watch.is_complete() {
                        break;
                    }
                    let remaining = watch.remaining();
                    if remaining.is_some_and(|remaining| remaining.is_zero()) {
                        break;
                    }
                    let pause = remaining.map_or(watch.poll_interval(), |remaining| remaining.min(watch.poll_interval()));
                    tokio::select! {
                        () = watch.seal_requested() => break,
                        () = tokio::time::sleep(pause) => {}
                    }
                }

                let sealed = watch.seal(&mut events).await;
                send_events(ctx.streams(), &mut events).await?;
                sealed.map_err(failure)?;
                let completed = dokime::domain::model::files::wire::FileWatchFrame {
                    payload: Some(dokime::domain::model::files::wire::FileWatchFramePayload::Completed(
                        dokime::domain::model::files::wire::FileWatchCompleted {
                            artifacts: watch.sealed().iter().map(metadata).collect::<Result<Vec<_>, _>>()?,
                        },
                    )),
                };
                guard.watch = None;
                ctx.streams().send_owned((completed).into()).await.map_err(crate::grpc::chunked::output_failure)?;
                Ok(().into())
            })
            .await
        })
    }
}

impl Drop for Sealing {
    fn drop(&mut self) {
        let Some(mut watch) = self.watch.take() else {
            return;
        };
        let Ok(handle) = tokio::runtime::Handle::try_current() else {
            return;
        };
        handle.spawn(async move {
            let mut events = Vec::new();
            match watch.seal(&mut events).await {
                Ok(()) => tracing::info!(artifacts = watch.sealed().len(), "cancelled file watch sealed"),
                Err(error) => tracing::warn!(error = %error, "cancelled file watch did not seal every file"),
            }
        });
    }
}

/* === Public Functions === */

pub(super) async fn register(runtime: &RuntimeHandle) -> Result<(), crate::grpc::Error> {
    let served = Arc::new(Resources { runtime: runtime.clone() });
    let mut dependencies = dokime::harmos::Dependencies::new();
    dependencies.insert(Host::<contract::FileWatch>::new(served))?;
    let batch = dokime::harmos::RegistrationBatch::new(dependencies).operation::<contract::FileWatch>();
    runtime.operations().register(batch.lane(dokime::app::Lanes::Open)).await?;
    Ok(())
}

/* === Private Functions === */

async fn send_events(
    output: &StreamOutput<dokime::domain::model::files::wire::FileWatchFrame>, events: &mut Vec<WatchEvent>,
) -> Result<(), Status> {
    for event in events.drain(..) {
        let frame = frame(event)?;
        output.send_owned((frame).into()).await.map_err(crate::grpc::chunked::output_failure)?;
    }
    Ok(())
}

fn frame(event: WatchEvent) -> Result<dokime::domain::model::files::wire::FileWatchFrame, Status> {
    let payload = match event {
        WatchEvent::Armed { baseline } => {
            dokime::domain::model::files::wire::FileWatchFramePayload::Armed(dokime::domain::model::files::wire::FileWatchArmed {
                baseline,
            })
        }
        WatchEvent::Progress { path, offset, phase } => {
            dokime::domain::model::files::wire::FileWatchFramePayload::Progress(dokime::domain::model::files::wire::FileWatchProgress {
                path,
                offset,
                phase,
            })
        }
        WatchEvent::Gap { path, offset, reason } => {
            dokime::domain::model::files::wire::FileWatchFramePayload::Gap(dokime::domain::model::files::wire::FileWatchGap {
                path,
                offset,
                reason,
            })
        }
        WatchEvent::Sealed { path, record } => dokime::domain::model::files::wire::FileWatchFramePayload::Sealed(Box::new(
            dokime::domain::model::files::wire::FileWatchSealed { path, artifact: Some(metadata(&record)?) },
        )),
    };
    Ok(dokime::domain::model::files::wire::FileWatchFrame { payload: Some(payload) })
}

fn failure(error: Error) -> Status {
    let code = error.code().unwrap_or_default();
    match &error {
        Error::Admission(AdmissionFailure::Validation { message, .. }) if code == INVALID => Status::invalid_argument(message.clone()),
        Error::Admission(AdmissionFailure::Overload { message, .. }) if code == CAPPED => Status::resource_exhausted(message.clone()),
        Error::Admission(AdmissionFailure::Conflict { message, .. }) if code == UNTRUSTED || code == UNSUPPORTED => {
            Status::failed_precondition(message.clone())
        }
        Error::Admission(AdmissionFailure::Unavailable { message, .. }) if code == UNREACHABLE => Status::unavailable(message.clone()),
        Error::Admission(AdmissionFailure::Conflict { message, .. }) => Status::failed_precondition(message.clone()),
        Error::Admission(AdmissionFailure::NotFound { message, .. }) => Status::not_found(message.clone()),
        _ => content_failure(error),
    }
}

/* === Tests === */
