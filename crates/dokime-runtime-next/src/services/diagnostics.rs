/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Client gRPC Services

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex},
};

// Third Party Imports
use tokio::sync::broadcast;
use tracing::{
    Event, Subscriber,
    field::{Field, Visit},
};
use tracing_subscriber::{Layer, layer::Context as LayerContext, registry::LookupSpan};

// Local Imports
use crate::SchedulerClock;
use dokime_protocol::v1;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone)]
pub struct Diagnostics {
    inner: Arc<Mutex<Tail>>,
    changes: broadcast::Sender<()>,
    clock: Arc<dyn SchedulerClock>,
    items: usize,
    bytes: usize,
}
struct Tail {
    sequence: u64,
    events: VecDeque<(v1::TraceEvent, usize)>,
    bytes: usize,
    dropped: HashMap<String, u64>,
}
#[derive(Default)]
struct Fields {
    values: HashMap<String, String>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Diagnostics {
    pub fn new(clock: Arc<dyn SchedulerClock>, items: std::num::NonZeroUsize, bytes: std::num::NonZeroUsize) -> Self {
        let (changes, _) = broadcast::channel(items.get());
        Self {
            inner: Arc::new(Mutex::new(Tail {
                sequence: 0,
                events: VecDeque::new(),
                bytes: 0,
                dropped: HashMap::new(),
            })),
            changes,
            clock,
            items: items.get(),
            bytes: bytes.get(),
        }
    }
    pub(super) fn snapshot(&self, scope: Option<&v1::TraceScope>) -> (Vec<v1::TraceEvent>, u64, u64) {
        let tail = self.inner.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        let key = match scope.and_then(|scope| scope.kind.as_ref()) {
            Some(v1::trace_scope::Kind::Cycle(id)) => format!("cycle:{id}"),
            Some(v1::trace_scope::Kind::Execution(id)) => format!("execution:{id}"),
            _ => "instance".into(),
        };
        (tail.events.iter().map(|(event, _)| event.clone()).collect(), tail.sequence, tail.dropped.get(&key).copied().unwrap_or_default())
    }
    pub(super) fn changes(&self) -> broadcast::Receiver<()> {
        self.changes.subscribe()
    }
}
impl Visit for Fields {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        self.values.insert(field.name().into(), format!("{value:?}"));
    }
    fn record_str(&mut self, field: &Field, value: &str) {
        self.values.insert(field.name().into(), value.into());
    }
}
impl<S> Layer<S> for Diagnostics
where
    S: Subscriber + for<'lookup> LookupSpan<'lookup>,
{
    fn on_event(&self, event: &Event<'_>, context: LayerContext<'_, S>) {
        let mut fields = Fields::default();
        if let Some(scope) = context.event_scope(event) {
            for span in scope.from_root() {
                if let Some(values) = span.extensions().get::<Fields>() {
                    fields.values.extend(values.values.clone());
                }
            }
        }
        event.record(&mut fields);
        let message = fields.values.remove("message").unwrap_or_default();
        for (name, value) in &mut fields.values {
            let folded: String = name.chars().filter(|character| character.is_alphanumeric()).flat_map(char::to_lowercase).collect();
            if ["secret", "password", "passphrase", "token", "apikey", "privatekey", "accesskey", "authorization", "cookie"]
                .iter()
                .any(|fragment| folded.contains(fragment))
            {
                *value = "<redacted>".into();
            }
        }
        let now = self.clock.now_ns();
        let mut tail = self.inner.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        tail.sequence += 1;
        let sequence = tail.sequence;
        let mut output = v1::TraceEvent {
            sequence,
            time: Some(prost_types::Timestamp {
                seconds: now.div_euclid(1_000_000_000),
                nanos: now.rem_euclid(1_000_000_000) as i32,
            }),
            level: match *event.metadata().level() {
                tracing::Level::ERROR => v1::LogLevel::Error,
                tracing::Level::WARN => v1::LogLevel::Warn,
                tracing::Level::INFO => v1::LogLevel::Info,
                tracing::Level::DEBUG => v1::LogLevel::Debug,
                tracing::Level::TRACE => v1::LogLevel::Trace,
            } as i32,
            target: event.metadata().target().into(),
            message,
            plugin_id: fields.values.get("plugin").cloned(),
            cycle_id: fields.values.get("cycle").cloned(),
            execution_id: fields.values.get("execution").cloned(),
            fields: fields.values,
        };
        // One Event Cannot Consume the Entire Configured Tail.
        use prost::Message;
        let mut bytes = output.encoded_len();
        if bytes > self.bytes {
            output.message = "Diagnostic Event Exceeded Its Byte Budget; Reduce the Logged Fields".into();
            output.fields.clear();
            bytes = output.encoded_len();
        }
        tail.events.push_back((output, bytes));
        tail.bytes += bytes;
        while tail.events.len() > self.items || tail.bytes > self.bytes {
            let Some((evicted, bytes)) = tail.events.pop_front() else {
                break;
            };
            tail.bytes -= bytes;
            *tail.dropped.entry("instance".into()).or_default() += 1;
            if let Some(id) = evicted.cycle_id {
                *tail.dropped.entry(format!("cycle:{id}")).or_default() += 1;
            }
            if let Some(id) = evicted.execution_id {
                *tail.dropped.entry(format!("execution:{id}")).or_default() += 1;
            }
        }
        drop(tail);
        let _ = self.changes.send(());
    }
    fn on_new_span(&self, attributes: &tracing::span::Attributes<'_>, id: &tracing::Id, context: LayerContext<'_, S>) {
        if let Some(span) = context.span(id) {
            let mut fields = Fields::default();
            attributes.record(&mut fields);
            span.extensions_mut().insert(fields);
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;
    use prost::Message;
    use std::num::NonZeroUsize;
    use tracing_subscriber::prelude::*;

    #[test]
    fn tail_accounts_for_evicted_bytes_and_scope_counts() {
        let diagnostics =
            Diagnostics::new(Arc::new(crate::ControlledClock::new(0)), NonZeroUsize::new(3).unwrap(), NonZeroUsize::new(1024).unwrap());
        let subscriber = tracing_subscriber::registry().with(diagnostics.clone());
        tracing::subscriber::with_default(subscriber, || {
            for value in 0..8 {
                tracing::info!(execution = "fixture", value, payload = "x".repeat(400));
            }
        });
        let scope = v1::TraceScope {
            kind: Some(v1::trace_scope::Kind::Execution("fixture".into())),
        };
        let (events, sequence, dropped) = diagnostics.snapshot(Some(&scope));
        assert_eq!(sequence, 8);
        assert_eq!(events.last().unwrap().fields["value"], "7");
        assert_eq!(events.len() as u64 + dropped, sequence);
        let bytes = events.iter().map(Message::encoded_len).sum::<usize>();
        assert!(bytes <= diagnostics.bytes);
        assert_eq!(diagnostics.inner.lock().unwrap().bytes, bytes);
    }
}
