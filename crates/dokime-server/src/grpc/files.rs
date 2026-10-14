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
use std::{sync::Arc, time::Instant};

// Third Party Imports
use harmos::{
    MemorySize,
    operations::{Context, Operation, Response, StreamOutput},
};
use tonic::Status;

// Local Imports
use super::content::{content_failure, metadata};
use dokime_app::{
    Error, RuntimeHandle,
    capabilities::files::{
        CAPPED, FileWatch as Watch, INVALID, UNREACHABLE, UNSUPPORTED, UNTRUSTED, WatchEvent,
    },
};
use dokime_contract::operations::files as contract;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct Resources {
    runtime: RuntimeHandle,
}

struct FileWatch {
    resources: Arc<Resources>,
    watch: Option<Watch>,
}

/* === Enums === */
#[harmos::catalog]
enum Operations {
    FileWatch(FileWatch),
}

/* === Traits === */

/* === Implementations === */
impl Operation for FileWatch {
    type Declaration = contract::FileWatch;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = StreamOutput<contract::FileWatchFrame>;

    fn new(resources: Arc<Resources>) -> Result<Self, Status> {
        Ok(Self {
            resources,
            watch: None,
        })
    }

    async fn run<'a>(
        &mut self,
        context: Self::Context<'a>,
        request: &mut contract::FileWatchRequest,
        output: &mut Self::Endpoints,
    ) -> Result<Response<()>, Status> {
        // A Step's Call Carries Its Execution; A Listener Call Names It
        let scoped = context
            .context()
            .value::<dokime::execution::ExecutionScope>()
            .ok()
            .flatten()
            .map(|scope| scope.execution);
        let watch = self
            .resources
            .runtime
            .open_file_watch(request, scoped.as_deref())
            .await
            .map_err(failure)?;
        let watch = self.watch.insert(watch);
        let started = Instant::now();

        // Poll Until Complete, Bounded, Or Sealed By The Execution;
        // Cancellation Seals Through `stop`
        let mut events = Vec::new();
        loop {
            let polled = watch.poll(&mut events).await;
            if let Err(source) = polled {
                let sealed = watch.seal(&mut events).await;
                let _ = send_events(output, &mut events).await;
                self.watch = None;
                if let Err(seal) = sealed {
                    tracing::warn!(error = %seal, "failed file watch did not seal every file");
                }
                return Err(failure(source));
            }
            send_events(output, &mut events).await?;
            if watch.is_complete() {
                break;
            }
            let remaining = watch
                .deadline()
                .map(|deadline| deadline.saturating_sub(started.elapsed()));
            if remaining.is_some_and(|remaining| remaining.is_zero()) {
                break;
            }
            let pause = remaining.map_or(watch.poll_interval(), |remaining| {
                remaining.min(watch.poll_interval())
            });
            tokio::select! {
                () = watch.seal_requested() => break,
                () = tokio::time::sleep(pause) => {}
            }
        }

        // Seal, Then Announce Every Sealed Artifact
        let sealed = watch.seal(&mut events).await;
        send_events(output, &mut events).await?;
        sealed.map_err(failure)?;
        let completed = contract::FileWatchFrame {
            payload: Some(contract::FileWatchFramePayload::Completed(
                contract::FileWatchCompleted {
                    artifacts: watch
                        .sealed()
                        .iter()
                        .map(metadata)
                        .collect::<Result<Vec<_>, _>>()?,
                },
            )),
        };
        self.watch = None;
        output
            .send_with(completed.memory_size(), || Ok(completed))
            .await
            .map_err(stream_failure)?;
        Ok(().into())
    }

    // A cancelled or failed stream still seals what the watch captured.
    async fn stop(&mut self) -> Result<(), Status> {
        let Some(mut watch) = self.watch.take() else {
            return Ok(());
        };
        let mut events = Vec::new();
        watch.seal(&mut events).await.map_err(failure)?;
        tracing::info!(
            artifacts = watch.sealed().len(),
            "cancelled file watch sealed"
        );
        Ok(())
    }
}

/* === Public Functions === */
pub(super) fn register(runtime: &RuntimeHandle) -> Result<(), crate::grpc::Error> {
    let resources = Arc::new(Resources {
        runtime: runtime.clone(),
    });
    runtime.operations().register_wire_operation::<FileWatch>(
        "dokime",
        dokime_app::app::Lanes::Open,
        resources,
    )?;
    Ok(())
}

/* === Private Functions === */
async fn send_events(
    output: &mut StreamOutput<contract::FileWatchFrame>,
    events: &mut Vec<WatchEvent>,
) -> Result<(), Status> {
    for event in events.drain(..) {
        let frame = frame(event)?;
        output
            .send_with(frame.memory_size(), || Ok(frame))
            .await
            .map_err(stream_failure)?;
    }
    Ok(())
}

fn frame(event: WatchEvent) -> Result<contract::FileWatchFrame, Status> {
    let payload = match event {
        WatchEvent::Progress {
            path,
            offset,
            phase,
        } => contract::FileWatchFramePayload::Progress(contract::FileWatchProgress {
            path,
            offset,
            phase,
        }),
        WatchEvent::Gap {
            path,
            offset,
            reason,
        } => contract::FileWatchFramePayload::Gap(contract::FileWatchGap {
            path,
            offset,
            reason,
        }),
        WatchEvent::Sealed { path, record } => {
            contract::FileWatchFramePayload::Sealed(Box::new(contract::FileWatchSealed {
                path,
                artifact: Some(metadata(&record)?),
            }))
        }
    };
    Ok(contract::FileWatchFrame {
        payload: Some(payload),
    })
}

fn failure(error: Error) -> Status {
    let code = error.code().unwrap_or_default();
    match &error {
        Error::Validation { message, .. } if code == INVALID => {
            Status::invalid_argument(message.clone())
        }
        Error::Overload { message, .. } if code == CAPPED => {
            Status::resource_exhausted(message.clone())
        }
        Error::Conflict { message, .. } if code == UNTRUSTED || code == UNSUPPORTED => {
            Status::failed_precondition(message.clone())
        }
        Error::Unavailable { message, .. } if code == UNREACHABLE => {
            Status::unavailable(message.clone())
        }
        Error::Conflict { message, .. } => Status::failed_precondition(message.clone()),
        Error::NotFound { message, .. } => Status::not_found(message.clone()),
        _ => content_failure(error),
    }
}

fn stream_failure(error: harmos::Error) -> Status {
    match error {
        harmos::Error::ResourceExhausted { .. } => Status::resource_exhausted(error.to_string()),
        harmos::Error::NotReady { .. } => Status::unavailable(error.to_string()),
        error => Status::internal(error.to_string()),
    }
}

/* === Tests === */
