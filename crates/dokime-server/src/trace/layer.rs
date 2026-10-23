/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// The Subscriber Layer That Feeds The Journal
//
// Scope ids come from the event's own fields first and then from the nearest
// enclosing span that recorded them. Sidecar output already arrives as tracing
// events: forwarded stderr under `sidecar::stderr` with `plugin`, and the
// execution log mirror under `sidecar` with `plugin_id`, `execution_id`,
// `line` and `timestamp`. Their `line` is the message and their `timestamp` the
// event time, so a plugin's line reads as the plugin wrote it.
//
// Secrets never enter the journal: a secret-looking field name keeps only a
// redaction marker, and bearer secrets are cut out of every text.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::Arc;

// Third Party Imports
use time::OffsetDateTime;
use tracing::field::{Field, Visit};
use tracing::span::{Attributes, Id, Record};
use tracing::{Event as TracingEvent, Subscriber};
use tracing_subscriber::layer::{Context, Layer};
use tracing_subscriber::registry::LookupSpan;

// Local Imports
use super::journal::{Event, Journal};
use crate::grpc::redaction;

/* === Constants === */

pub const WATCH_SPAN: &str = "dokime.trace.watch";
const MAX_MESSAGE_BYTES: usize = 8 * 1024;
const MAX_VALUE_BYTES: usize = 1024;
const MAX_NAME_BYTES: usize = 64;
const MAX_FIELDS: usize = 32;
const TRUNCATED: &str = "…";
const BEARER: &str = "bearer ";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct Capture {
    journal: Arc<Journal>,
}

#[derive(Clone, Debug, Default)]
struct SpanScope {
    plugin: Option<String>,
    cycle: Option<String>,
    execution: Option<String>,
}

#[derive(Default)]
struct Fields {
    message: Option<String>,
    line: Option<String>,
    time: Option<OffsetDateTime>,
    scope: SpanScope,
    fields: Vec<(String, String)>,
}

struct BoundedText(String);

/* === Enums === */

enum Role {
    Message,
    Line,
    Timestamp,
    Plugin,
    Cycle,
    Execution,
    Other,
}

/* === Traits === */

/* === Implementations === */

impl std::fmt::Write for BoundedText {
    fn write_str(&mut self, text: &str) -> std::fmt::Result {
        let remaining = MAX_MESSAGE_BYTES.saturating_sub(self.0.len());
        if text.len() <= remaining {
            self.0.push_str(text);
            return Ok(());
        }
        if remaining >= TRUNCATED.len() {
            self.0.push_str(&truncated(text, remaining));
        }
        Err(std::fmt::Error)
    }
}

impl Capture {
    pub fn new(journal: Arc<Journal>) -> Self {
        journal.mark_installed();
        Self { journal }
    }
}

impl<S> Layer<S> for Capture
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_new_span(&self, attributes: &Attributes<'_>, id: &Id, context: Context<'_, S>) {
        let Some(span) = context.span(id) else {
            return;
        };
        let mut fields = Fields::default();
        attributes.record(&mut fields);
        span.extensions_mut().insert(fields.scope);
    }

    fn on_record(&self, id: &Id, values: &Record<'_>, context: Context<'_, S>) {
        let Some(span) = context.span(id) else {
            return;
        };
        let mut fields = Fields::default();
        values.record(&mut fields);
        let mut extensions = span.extensions_mut();
        match extensions.get_mut::<SpanScope>() {
            Some(scope) => scope.update(fields.scope),
            None => extensions.insert(fields.scope),
        }
    }

    fn on_event(&self, event: &TracingEvent<'_>, context: Context<'_, S>) {
        // Read The Event's Own Fields
        let mut fields = Fields::default();
        event.record(&mut fields);

        // Fill Missing Scope Ids From The Enclosing Spans, Nearest First
        if let Some(spans) = context.event_scope(event) {
            for span in spans {
                if span.name() == WATCH_SPAN {
                    return;
                }
                if let Some(scope) = span.extensions().get::<SpanScope>() {
                    fields.scope.merge(scope.clone());
                }
            }
        }

        // Shape The Captured Event
        let metadata = event.metadata();
        let message = fields.line.or(fields.message).unwrap_or_default();
        self.journal.record(Event {
            time: fields.time.unwrap_or_else(OffsetDateTime::now_utc),
            level: *metadata.level(),
            target: truncated(&scrubbed(metadata.target()), MAX_VALUE_BYTES),
            message: truncated(&scrubbed(&message), MAX_MESSAGE_BYTES),
            plugin: fields.scope.plugin,
            cycle: fields.scope.cycle,
            execution: fields.scope.execution,
            fields: fields.fields,
        });
    }
}

impl SpanScope {
    fn update(&mut self, other: SpanScope) {
        if other.plugin.is_some() {
            self.plugin = other.plugin;
        }
        if other.cycle.is_some() {
            self.cycle = other.cycle;
        }
        if other.execution.is_some() {
            self.execution = other.execution;
        }
    }

    fn merge(&mut self, other: SpanScope) {
        self.plugin = self.plugin.take().or(other.plugin);
        self.cycle = self.cycle.take().or(other.cycle);
        self.execution = self.execution.take().or(other.execution);
    }
}

impl Fields {
    fn put(&mut self, field: &Field, value: String) {
        let name = field.name();
        match Role::of(name) {
            Role::Message => self.message = Some(value),
            Role::Line => self.line = Some(value),
            Role::Timestamp => match OffsetDateTime::parse(&value, &time::format_description::well_known::Rfc3339) {
                Ok(time) => self.time = Some(time),
                Err(_) => self.other(name, value),
            },
            Role::Plugin => self.scope.plugin = Some(truncated(&scrubbed(&value), MAX_VALUE_BYTES)),
            Role::Cycle => self.scope.cycle = Some(truncated(&scrubbed(&value), MAX_VALUE_BYTES)),
            Role::Execution => self.scope.execution = Some(truncated(&scrubbed(&value), MAX_VALUE_BYTES)),
            Role::Other => self.other(name, value),
        }
    }

    fn other(&mut self, name: &str, value: String) {
        if self.fields.len() >= MAX_FIELDS {
            return;
        }
        let value =
            if redaction::is_secret_name(name) { redaction::REDACTED.to_owned() } else { truncated(&scrubbed(&value), MAX_VALUE_BYTES) };
        self.fields.push((truncated(name, MAX_NAME_BYTES), value));
    }
}

impl Visit for Fields {
    fn record_str(&mut self, field: &Field, value: &str) {
        self.put(field, truncated(value, MAX_MESSAGE_BYTES));
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.put(field, value.to_string());
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        self.put(field, value.to_string());
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.put(field, value.to_string());
    }

    fn record_f64(&mut self, field: &Field, value: f64) {
        self.put(field, value.to_string());
    }

    fn record_error(&mut self, field: &Field, value: &(dyn std::error::Error + 'static)) {
        self.record_debug(field, &format_args!("{value}"));
    }

    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        use std::fmt::Write as _;
        let mut value_text = BoundedText(String::new());
        let _ = write!(value_text, "{value:?}");
        self.put(field, value_text.0);
    }
}

impl Role {
    fn of(name: &str) -> Self {
        match name {
            "message" => Self::Message,
            "line" => Self::Line,
            "timestamp" => Self::Timestamp,
            "plugin_id" | "plugin" | "artifact_id" => Self::Plugin,
            "cycle_id" | "test_cycle_id" => Self::Cycle,
            "execution_id" | "test_execution_id" => Self::Execution,
            _ => Self::Other,
        }
    }
}

/* === Public Functions === */

pub fn scrubbed(text: &str) -> String {
    let folded = text.to_ascii_lowercase();
    let mut result = String::with_capacity(text.len());
    let mut copied = 0;
    let mut search = 0;
    while let Some(found) = folded[search..].find(BEARER) {
        let start = search + found + BEARER.len();
        let end = text[start..]
            .find(|character: char| character.is_whitespace() || matches!(character, '"' | '\'' | ',' | ';'))
            .map_or(text.len(), |offset| start + offset);
        if end > start {
            result.push_str(&text[copied..start]);
            result.push_str(redaction::REDACTED);
            copied = end;
        }
        search = end.max(start);
    }
    result.push_str(&text[copied..]);
    redaction::text(&secret_assignments(&result))
}

/* === Private Functions === */

fn secret_assignments(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut copied = 0;
    for (delimiter, character) in text.char_indices() {
        if delimiter < copied || !matches!(character, ':' | '=') {
            continue;
        }
        let key_end = text[..delimiter].trim_end_matches(['\"', '\'']).len();
        let key_start = text[..key_end]
            .rfind(|character: char| !character.is_ascii_alphanumeric() && !matches!(character, '_' | '-'))
            .map_or(0, |position| position + text[position..].chars().next().map_or(0, char::len_utf8));
        if !redaction::is_secret_name(&text[key_start..key_end]) {
            continue;
        }
        let value = &text[delimiter + 1..];
        let trimmed = value.trim_start();
        let start = text.len() - trimmed.len();
        let (start, end) = match trimmed.chars().next() {
            Some(quote @ ('\"' | '\'')) => {
                let start = start + 1;
                let end = text[start..].find(quote).map_or(text.len(), |offset| start + offset);
                (start, end)
            }
            _ => {
                let end = trimmed
                    .find(|character: char| character.is_whitespace() || matches!(character, ',' | ';' | '&' | '}' | ']'))
                    .map_or(text.len(), |offset| start + offset);
                (start, end)
            }
        };
        if end > start {
            result.push_str(&text[copied..start]);
            result.push_str(redaction::REDACTED);
            copied = end;
        }
    }
    result.push_str(&text[copied..]);
    result
}

fn truncated(text: &str, limit: usize) -> String {
    if text.len() <= limit {
        return text.to_owned();
    }
    let mut end = limit.saturating_sub(TRUNCATED.len());
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}{TRUNCATED}", &text[..end])
}

/* === Tests === */
