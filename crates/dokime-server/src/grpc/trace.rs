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
use dokime::harmos::operations::StreamOutput;
use dokime::harmos::{Context, RegistrationExt};
use tonic::Status;
use tracing::Instrument;

// Local Imports
use crate::trace::{Event, Journal, Page, Scope, WATCH_SPAN};
use dokime::RuntimeHandle;
use dokime::domain::model::backend::wire::{LogFrameFieldsEntry, TraceBatch, TraceEvent};
use dokime::domain::model::identity::wire::Timestamp;
use dokime::operations::backend as wire;
use dokime::operations::hosting::{Host, Serve, Served};

/* === Constants === */

const WATCH_HEARTBEAT: std::time::Duration = std::time::Duration::from_secs(10);
const SERVER_DEFAULT_TRACE_BYTES: u64 = 262_144;
const SERVER_DEFAULT_TRACE_EVENTS: u64 = 256;
const SERVER_MAXIMUM_TRACE_EVENTS: u64 = 1_024;
const SERVER_MINIMUM_TRACE_BYTES: u64 = 32_768;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct Resources {
    runtime: RuntimeHandle,
    maximum: usize,
    journal: Arc<Journal>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Serve<wire::WatchTrace> for Resources {
    fn serve<'a>(&'a self, request: wire::WatchTrace, ctx: Context<'a, wire::WatchTrace>) -> Served<'a, ()> {
        Box::pin(async move {
            let call_context = ctx.call();
            let output = ctx.streams();
            let revoked = crate::grpc::access::revocation(call_context);
            crate::grpc::access::during(revoked, async {
                self.follow(&request, output).instrument(tracing::info_span!(WATCH_SPAN)).await?;
                Ok(().into())
            })
            .await
        })
    }
}

impl Resources {
    async fn follow(
        &self, request: &wire::WatchTrace, output: &StreamOutput<dokime::domain::model::backend::wire::TraceBatch>,
    ) -> Result<(), Status> {
        let epoch = self.runtime.process_epoch().to_string();
        if request.epoch.as_deref().is_some_and(|expected| expected != epoch) || (request.after_sequence != 0 && request.epoch.is_none()) {
            return Err(Status::out_of_range("trace cursor belongs to another backend epoch; restart replay"));
        }
        let journal = &self.journal;
        if !journal.installed() {
            return Err(Status::unavailable("backend tracing capture is not installed"));
        }
        let scope = scope(request.scope.as_ref())?;
        let min_level = request.min_level as i32;
        let events = if request.max_events == 0 { SERVER_DEFAULT_TRACE_EVENTS } else { u64::from(request.max_events) }
            .min(SERVER_MAXIMUM_TRACE_EVENTS);
        let events = usize::try_from(events).expect("validated trace event count");
        let maximum = self.maximum;
        let bytes =
            if request.max_bytes == 0 { SERVER_DEFAULT_TRACE_BYTES } else { u64::from(request.max_bytes) }.max(SERVER_MINIMUM_TRACE_BYTES);
        let bytes = usize::try_from(bytes).expect("validated trace byte count").min(maximum);
        // Subscribe Before Reading So Capture Cannot Race The Follow Wait.
        let mut changes = journal.subscribe();
        let through = if request.follow { u64::MAX } else { journal.tail(&scope) };
        let mut after = request.after_sequence;
        let mut dropped = 0_u64;
        let mut initial = true;
        loop {
            changes.borrow_and_update();
            let page = journal
                .read_until(&scope, after, |event| level(event.level) as i32 >= min_level, events, bytes, through)
                .map_err(|_| Status::out_of_range("trace cursor is unavailable; restart replay with after_sequence zero"))?;
            let missed = page.missed;
            dropped = dropped.saturating_add(missed);
            let batch = batch(&epoch, page, dropped, bytes)?;
            let advanced = batch.next_sequence > after;
            after = batch.next_sequence;
            if initial || advanced || missed > 0 {
                output.send_owned(batch.into()).await.map_err(crate::grpc::chunked::output_failure)?;
                initial = false;
            }
            if !request.follow && after >= through {
                return Ok(());
            }
            if advanced {
                tokio::task::yield_now().await;
                continue;
            }
            match tokio::time::timeout(WATCH_HEARTBEAT, changes.changed()).await {
                Ok(changed) => changed.map_err(|_| Status::unavailable("trace journal closed"))?,
                Err(_) => {
                    let keepalive = TraceBatch { epoch: epoch.clone(), events: Vec::new(), dropped, next_sequence: after };
                    output.send_owned(keepalive.into()).await.map_err(crate::grpc::chunked::output_failure)?;
                }
            }
        }
    }
}

/* === Public Functions === */

pub(super) async fn register(runtime: &RuntimeHandle, maximum: usize, journal: Arc<Journal>) -> Result<(), crate::grpc::Error> {
    let served = Arc::new(Resources {
        runtime: runtime.clone(),
        maximum,
        journal,
    });
    let mut dependencies = dokime::harmos::Dependencies::new();
    dependencies.insert(Host::<wire::WatchTrace>::new(served.clone()))?;
    let batch = dokime::harmos::RegistrationBatch::new(dependencies).operation::<wire::WatchTrace>();
    runtime.operations().register(batch.lane(dokime::app::Lanes::Open)).await?;
    Ok(())
}

/* === Private Functions === */

fn scope(scope: Option<&dokime::domain::model::backend::wire::TraceScope>) -> Result<Scope, Status> {
    match scope {
        None | Some(dokime::domain::model::backend::wire::TraceScope::Instance) => Ok(Scope::Instance),
        Some(dokime::domain::model::backend::wire::TraceScope::Cycle(id)) => Ok(Scope::Cycle(scope_id(id)?)),
        Some(dokime::domain::model::backend::wire::TraceScope::Execution(id)) => Ok(Scope::Execution(scope_id(id)?)),
    }
}

fn scope_id(id: &str) -> Result<String, Status> {
    let parsed = uuid::Uuid::parse_str(id).map_err(|_| Status::invalid_argument("trace scope requires a canonical UUID"))?;
    let canonical = parsed.to_string();
    if canonical != id {
        return Err(Status::invalid_argument("trace scope requires a canonical UUID"));
    }
    Ok(canonical)
}

fn level(level: tracing::Level) -> dokime::domain::model::backend::wire::LogLevel {
    match level {
        tracing::Level::TRACE => dokime::domain::model::backend::wire::LogLevel::Trace,
        tracing::Level::DEBUG => dokime::domain::model::backend::wire::LogLevel::Debug,
        tracing::Level::INFO => dokime::domain::model::backend::wire::LogLevel::Info,
        tracing::Level::WARN => dokime::domain::model::backend::wire::LogLevel::Warn,
        tracing::Level::ERROR => dokime::domain::model::backend::wire::LogLevel::Error,
    }
}

fn budget_failure() -> Status {
    Status::resource_exhausted("trace frame exceeds its allocation or wire limit")
}

fn batch(epoch: &str, page: Page, dropped: u64, maximum: usize) -> Result<TraceBatch, Status> {
    let count = page.events.len();
    let mut events = Vec::new();
    events.try_reserve_exact(count).map_err(|_| budget_failure())?;
    for (sequence, event) in page.events.iter().take(count) {
        events.push(event_wire(*sequence, event)?);
    }
    let next_sequence = if count == page.events.len() { page.next } else { events.last().expect("nonempty bounded prefix").sequence };
    let mut batch = TraceBatch {
        epoch: epoch.to_owned(),
        events,
        dropped,
        next_sequence,
    };
    while super::content::output_frame_len(&batch)? > maximum {
        batch.events.pop();
        batch.next_sequence = batch.events.last().ok_or_else(budget_failure)?.sequence;
    }
    Ok(batch)
}

fn event_wire(sequence: u64, event: &Event) -> Result<TraceEvent, Status> {
    let mut fields = Vec::new();
    fields.try_reserve_exact(event.fields.len()).map_err(|_| budget_failure())?;

    for (key, value) in &event.fields {
        fields.push(LogFrameFieldsEntry { key: key.clone(), value: value.clone() });
    }
    Ok(TraceEvent {
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
        fields,
    })
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    fn event(message: &str) -> Arc<Event> {
        Arc::new(Event {
            time: time::OffsetDateTime::UNIX_EPOCH,
            level: tracing::Level::INFO,
            target: "fixture".into(),
            message: message.into(),
            plugin: None,
            cycle: None,
            execution: None,
            fields: Vec::new(),
        })
    }

    #[test]
    fn split_batch_preserves_resume_position() {
        let event = event(&"x".repeat(1024));
        let one = TraceBatch {
            epoch: "epoch".into(),
            events: vec![event_wire(1, &event).unwrap()],
            dropped: 0,
            next_sequence: 1,
        };
        let maximum = super::super::content::output_frame_len(&one).unwrap();
        let page = Page {
            events: vec![(1, Arc::clone(&event)), (2, Arc::clone(&event))],
            missed: 0,
            next: 3,
        };
        let first = batch("epoch", page, 0, maximum).unwrap();
        assert_eq!(first.events.iter().map(|event| event.sequence).collect::<Vec<_>>(), [1]);
        assert_eq!(first.next_sequence, 1);
        assert!(super::super::content::output_frame_len(&first).unwrap() <= maximum);

        let page = Page {
            events: vec![(2, event)],
            missed: 0,
            next: 3,
        };
        let second = batch("epoch", page, 0, maximum).unwrap();
        assert_eq!(second.events.iter().map(|event| event.sequence).collect::<Vec<_>>(), [2]);
        assert_eq!(second.next_sequence, 3);
    }
}
