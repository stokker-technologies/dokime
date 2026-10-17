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
use harmos::{
    MemorySize, Protobuf,
    operations::{Context, Operation, Response, StreamOutput},
};
use tonic::Status;
use tracing::Instrument;
// Local Imports
use crate::trace::{Event, Journal, Scope, WATCH_SPAN};
use dokime_runtime::RuntimeHandle;
use dokime_domain::operations::backend as wire;
use dokime_domain::operations::common::Timestamp;

/* === Constants === */
const DEFAULT_EVENTS: usize = 256;
const MAX_EVENTS: usize = 1024;
const DEFAULT_BYTES: usize = 256 * 1024;
const MIN_BYTES: usize = 32 * 1024;
const BATCH_OVERHEAD: usize = 256;

/* === Macros === */
/* === Type Aliases === */
/* === Structs === */
struct Resources {
    runtime: RuntimeHandle,
    maximum: usize,
    journal: Arc<Journal>,
}
struct Watch {
    resources: Arc<Resources>,
}

/* === Enums === */
/* === Traits === */
/* === Implementations === */
impl Operation for Watch {
    type Declaration = wire::WatchTrace;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = StreamOutput<wire::TraceBatch>;

    async fn initialize<'ctx>(
        resources: Arc<Resources>,
        _: Self::Context<'ctx>,
    ) -> Result<Self, Status> {
        Ok(Self { resources })
    }

    async fn run<'a>(
        &mut self,
        _: Self::Context<'a>,
        request: &mut wire::WatchTraceRequest,
        output: &mut Self::Endpoints,
    ) -> Result<Response<()>, Status> {
        self.follow(request, output)
            .instrument(tracing::info_span!(WATCH_SPAN))
            .await?;
        Ok(().into())
    }
}

impl Watch {
    async fn follow(
        &self,
        request: &wire::WatchTraceRequest,
        output: &mut StreamOutput<wire::TraceBatch>,
    ) -> Result<(), Status> {
        let epoch = self.resources.runtime.process_epoch().to_string();
        if request
            .epoch
            .as_deref()
            .is_some_and(|expected| expected != epoch)
            || (request.after_sequence != 0 && request.epoch.is_none())
        {
            return Err(Status::out_of_range(
                "trace cursor belongs to another backend epoch; restart replay",
            ));
        }
        let journal = &self.resources.journal;
        if !journal.installed() {
            return Err(Status::unavailable(
                "backend tracing capture is not installed",
            ));
        }
        let scope = scope(request.scope.as_ref())?;
        let min_level = request.min_level as i32;
        let events = if request.max_events == 0 {
            DEFAULT_EVENTS
        } else {
            request.max_events as usize
        }
        .min(MAX_EVENTS);
        let maximum = self.resources.maximum;
        let bytes = if request.max_bytes == 0 {
            DEFAULT_BYTES
        } else {
            request.max_bytes as usize
        }
        .max(MIN_BYTES)
        .min(maximum);
        if bytes <= BATCH_OVERHEAD {
            return Err(Status::resource_exhausted(
                "trace message budget is too small",
            ));
        }
        // Subscribe Before Reading So Capture Cannot Race The Follow Wait.
        let mut changes = journal.subscribe();
        let through = if request.follow {
            u64::MAX
        } else {
            journal.tail(&scope)
        };
        let mut after = request.after_sequence;
        let mut dropped = 0_u64;
        let mut initial = true;
        loop {
            changes.borrow_and_update();
            let page = journal
                .read_until(
                    &scope,
                    after,
                    |event| level(event.level) as i32 >= min_level,
                    events,
                    // Account Conservatively For Cloned Wire Rows And Vector Capacity.
                    bytes.saturating_sub(BATCH_OVERHEAD) / 4,
                    through,
                )
                .map_err(|_| {
                    Status::out_of_range(
                        "trace cursor is unavailable; restart replay with after_sequence zero",
                    )
                })?;
            dropped = dropped.saturating_add(page.missed);
            let advanced = page.next > after;
            after = page.next;
            if initial || advanced || page.missed > 0 {
                let batch = wire::TraceBatch {
                    epoch: epoch.clone(),
                    events: page
                        .events
                        .into_iter()
                        .map(|(sequence, event)| event_wire(sequence, &event))
                        .collect(),
                    dropped,
                    next_sequence: after,
                };
                if batch.encoded_len() > bytes || batch.memory_size() > bytes {
                    return Err(Status::resource_exhausted(
                        "trace batch exceeds configured budget",
                    ));
                }
                let response = batch.into();
                output.send_owned(response).await.map_err(failure)?;
                initial = false;
            }
            if !request.follow && after >= through {
                return Ok(());
            }
            if advanced {
                tokio::task::yield_now().await;
                continue;
            }
            changes
                .changed()
                .await
                .map_err(|_| Status::unavailable("trace journal closed"))?;
        }
    }
}

/* === Public Functions === */
pub(super) fn register(runtime: &RuntimeHandle, maximum: usize) -> Result<(), super::Error> {
    runtime.operations().register_wire_operation::<Watch>(
        "dokime",
        dokime_runtime::app::Lanes::Open,
        Arc::new(Resources {
            runtime: runtime.clone(),
            maximum,
            journal: Journal::global().clone(),
        }),
    )?;
    Ok(())
}

/* === Private Functions === */
fn scope(scope: Option<&wire::TraceScope>) -> Result<Scope, Status> {
    match scope {
        None | Some(wire::TraceScope::Instance) => Ok(Scope::Instance),
        Some(wire::TraceScope::Cycle(id)) => Ok(Scope::Cycle(scope_id(id)?)),
        Some(wire::TraceScope::Execution(id)) => Ok(Scope::Execution(scope_id(id)?)),
    }
}

fn scope_id(id: &str) -> Result<String, Status> {
    let parsed = uuid::Uuid::parse_str(id)
        .map_err(|_| Status::invalid_argument("trace scope requires a canonical UUID"))?;
    let canonical = parsed.to_string();
    if canonical != id {
        return Err(Status::invalid_argument(
            "trace scope requires a canonical UUID",
        ));
    }
    Ok(canonical)
}

fn level(level: tracing::Level) -> wire::LogLevel {
    match level {
        tracing::Level::TRACE => wire::LogLevel::Trace,
        tracing::Level::DEBUG => wire::LogLevel::Debug,
        tracing::Level::INFO => wire::LogLevel::Info,
        tracing::Level::WARN => wire::LogLevel::Warn,
        tracing::Level::ERROR => wire::LogLevel::Error,
    }
}

fn event_wire(sequence: u64, event: &Event) -> wire::TraceEvent {
    wire::TraceEvent {
        sequence,
        time: Some(Timestamp {
            seconds: event.time.unix_timestamp(),
            nanos: event.time.nanosecond() as i32,
        }),
        level: level(event.level),
        target: event.target.clone(),
        message: event.message.clone(),
        plugin_id: event.plugin.clone(),
        cycle_id: event.cycle.clone(),
        execution_id: event.execution.clone(),
        fields: event
            .fields
            .iter()
            .map(|(key, value)| wire::LogFrameFieldsEntry {
                key: key.clone(),
                value: value.clone(),
            })
            .collect(),
    }
}

fn failure(source: harmos::Error) -> Status {
    let status = if matches!(source, harmos::Error::ResourceExhausted { .. }) {
        Status::resource_exhausted("trace delivery admission failed")
    } else {
        Status::internal("trace delivery failed")
    };
    crate::grpc::error::detailed(status, &source.into())
}

/* === Tests === */
