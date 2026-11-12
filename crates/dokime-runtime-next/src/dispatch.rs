/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Execution Host

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{collections::HashMap, sync::Arc};

// Third Party Imports
use harmos_protocol::v1 as h;
use prost_reflect::ReflectMessage;
use tokio::{sync::mpsc, task::JoinSet};
use tokio_stream::wrappers::ReceiverStream;
use tokio_util::sync::CancellationToken;

// Local Imports
use crate::{Error, PlacementSettings, Result, SchedulerClock, plugin::Entry};
use dokime_operations_next::Resources;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(crate) struct Dispatch {
    pub entry: Arc<Entry>,
    pub resources: Option<Resources>,
    pub cancellation: CancellationToken,
    pub settings: PlacementSettings,
    pub clock: Arc<dyn SchedulerClock>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Dispatch {
    pub async fn invoke<Q: ReflectMessage + Default, R: ReflectMessage + Default>(&self, operation: &str, request: Q) -> Result<R> {
        let contract = self
            .entry
            .description
            .catalog
            .as_ref()
            .and_then(|catalog| catalog.operations.iter().find(|contract| contract.id == operation))
            .ok_or_else(|| Error::Invalid(format!("Reconnect a Plugin Serving {operation}")))?;
        if contract.request != Q::default().descriptor().full_name() || contract.response != R::default().descriptor().full_name() {
            return Err(Error::Invalid(format!("Use the Declared Request and Response for {operation}")));
        }
        let policy = contract.policy.unwrap_or_default();
        let started_ns = self.clock.now_ns();
        let total_ns = duration_ns(policy.timeout.as_ref())?.and_then(|duration| started_ns.checked_add(duration));
        let payload = request.encode_to_vec();
        let mut attempt = 0;
        loop {
            attempt += 1;
            let attempt_ns = duration_ns(policy.attempt_timeout.as_ref())?.and_then(|duration| self.clock.now_ns().checked_add(duration));
            let deadline = total_ns.into_iter().chain(attempt_ns).min();
            let result = tokio::select! {
                biased;
                _ = reached(&self.clock, deadline) => Err(Error::Invalid(format!("Operation {operation} Reached Its Deadline; Inspect Evidence Before Retrying"))),
                result = self.attempt(operation, payload.clone(), attempt) => result,
            };
            let retryable = matches!(&result, Err(Error::Plugin(failure)) if failure.classification == h::Classification::Transient as i32)
                || matches!(&result, Err(Error::Status(status)) if status.code() == tonic::Code::Unavailable);
            if !retryable || attempt >= policy.attempts.max(1) {
                return result.and_then(|bytes| R::decode(bytes.as_slice()).map_err(Into::into));
            }
            let mut client = self.entry.client.clone();
            let description = client
                .describe(h::DescribeRequest {
                    revision: 1,
                    host: "dokime".into(),
                    host_version: env!("CARGO_PKG_VERSION").into(),
                })
                .await?
                .into_inner();
            if description.started_at != self.entry.description.started_at {
                return Err(Error::Invalid("Plugin Process Changed; Inspect the Interrupted Execution and Start a New Cycle".into()));
            }
            if let Some(backoff) = duration_ns(policy.backoff.as_ref())? {
                if let Some(resources) = &self.resources {
                    resources.clock.sleep(backoff).await?;
                } else {
                    self.clock.wait_until(self.clock.now_ns() + backoff).await?;
                }
            }
        }
    }
    async fn attempt(&self, operation: &str, payload: Vec<u8>, attempt: u32) -> Result<Vec<u8>> {
        let (send, receive) = mpsc::channel(self.settings.stream_window.get());
        send.send(h::HostFrame {
            frame: Some(h::host_frame::Frame::Request(h::Request {
                operation: operation.into(),
                payload,
                invocation: uuid::Uuid::now_v7().to_string(),
                attempt,
                scope: self.resources.as_ref().map_or_else(String::new, |resources| resources.execution.clone()),
            })),
        })
        .await
        .map_err(|_| Error::Invalid("Reconnect the Plugin Invocation".into()))?;
        let mut output = self.entry.client.clone().invoke(ReceiverStream::new(receive)).await?.into_inner();
        let mut children = JoinSet::new();
        let mut pending = HashMap::new();
        loop {
            tokio::select! {
                biased;
                _ = self.cancellation.cancelled() => {
                    let frame = h::HostFrame {
                        frame: Some(h::host_frame::Frame::Cancel(h::Cancel {
                            reason: "Operator Cancelled the Execution".into(),
                        })),
                    };
                    send.send(frame).await.map_err(|_| Error::Invalid("Reconnect the Cancelled Plugin Invocation".into()))?;
                    return Err(Error::Cancelled);
                }
                child = children.join_next(), if !children.is_empty() => {
                    let (call, result): (u64, dokime_operations_next::Result<Vec<u8>>) = match child {
                        Some(Ok(value)) => value,
                        Some(Err(error)) if error.is_cancelled() => continue,
                        Some(Err(error)) => return Err(Error::Join(error)),
                        None => continue,
                    };
                    pending.remove(&call);
                    let result = match result {
                        Ok(payload) => h::child_response::Result::Payload(payload),
                        Err(error) => h::child_response::Result::Failure(h::Failure {
                            classification: h::Classification::Permanent as i32,
                            code: "host_operation_failed".into(),
                            message: error.to_string(),
                            action: "Inspect the Execution Evidence and Correct the Host Call".into(),
                            ..Default::default()
                        }),
                    };
                    let frame = h::HostFrame {
                        frame: Some(h::host_frame::Frame::Child(h::ChildResponse { call, result: Some(result) })),
                    };
                    send.send(frame).await.map_err(|_| Error::Invalid("Reconnect the Plugin Before Returning Host Evidence".into()))?;
                }
                frame = output.message() => {
                    let frame = frame?.and_then(|frame| frame.frame).ok_or_else(|| {
                        Error::Invalid("Plugin Closed Before Settling the Operation; Inspect Interrupted Evidence".into())
                    })?;
                    match frame {
                        h::plugin_frame::Frame::Child(request) => {
                            let mut resources = self.resources.clone().ok_or_else(|| {
                                Error::Invalid("Keep Materialization Pure; Remove Its Host Calls".into())
                            })?;
                            resources.operation = operation.into();
                            resources.attempt = attempt;
                            if pending.contains_key(&request.call) {
                                return Err(Error::Invalid("Give Each Child Call a Unique Identifier".into()));
                            }
                            let call = request.call;
                            let handle = children.spawn(async move {
                                (call, dokime_operations_next::dispatch(&resources, &request).await)
                            });
                            pending.insert(call, handle);
                        }
                        h::plugin_frame::Frame::ChildCancel(cancel) => {
                            if let Some(handle) = pending.remove(&cancel.call) { handle.abort(); }
                        }
                        h::plugin_frame::Frame::Response(response) => return match response.result {
                            Some(h::response::Result::Payload(payload)) => Ok(payload),
                            Some(h::response::Result::Failure(failure)) => Err(Error::Plugin(Box::new(failure))),
                            None => Err(Error::Invalid("Return a Result from the Plugin Operation".into())),
                        },
                    }
                }
            }
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn duration_ns(duration: Option<&prost_types::Duration>) -> Result<Option<i64>> {
    duration
        .map(|duration| {
            duration
                .seconds
                .checked_mul(1_000_000_000)
                .and_then(|value| value.checked_add(duration.nanos as i64))
                .filter(|value| *value >= 0)
                .ok_or_else(|| Error::Invalid("Set a Nonnegative Operation Policy Duration Within Range".into()))
        })
        .transpose()
}
async fn reached(clock: &Arc<dyn SchedulerClock>, at_ns: Option<i64>) {
    match at_ns {
        Some(at_ns) => clock.reached(at_ns).await,
        None => std::future::pending().await,
    }
}

/* === Tests === */
