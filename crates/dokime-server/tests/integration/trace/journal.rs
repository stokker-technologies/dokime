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
// Third Party Imports
use time::OffsetDateTime;
use tracing_subscriber::prelude::*;
// Local Imports
use dokime_server::trace::{Capture, Event, Journal, Limits, Scope, WATCH_SPAN, scrubbed};

/* === Constants === */
/* === Macros === */
/* === Type Aliases === */
/* === Structs === */
/* === Enums === */
/* === Traits === */
/* === Implementations === */
/* === Public Functions === */
/* === Private Functions === */
fn event(cycle: Option<&str>, execution: Option<&str>, message: &str) -> Event {
    Event {
        time: OffsetDateTime::now_utc(),
        level: tracing::Level::INFO,
        target: "fixture".into(),
        message: message.into(),
        plugin: None,
        cycle: cycle.map(str::to_owned),
        execution: execution.map(str::to_owned),
        fields: Vec::new(),
    }
}

/* === Tests === */
#[test]
fn journal_bounds_retention() {
    let journal = Journal::new(Limits {
        instance_events: 2,
        scope_events: 2,
        cycles: 1,
        executions: 1,
        ..Limits::default()
    });
    let scope = Scope::Cycle("cycle-a".into());
    journal.record(event(Some("cycle-a"), Some("execution-a"), "one"));
    journal.record(event(None, Some("execution-a"), "two"));
    journal.record(event(None, Some("execution-a"), "three"));
    let page = journal.read(&scope, 0, |_| true, 2, 4096).unwrap();
    assert_eq!(page.missed, 1);
    assert_eq!(page.next, 3);
    assert_eq!(
        page.events
            .iter()
            .map(|(_, event)| event.message.as_str())
            .collect::<Vec<_>>(),
        ["two", "three"]
    );
    assert_eq!(page.events[0].1.cycle.as_deref(), Some("cycle-a"));
    let selected = journal.read(&scope, 1, |_| false, 2, 4096).unwrap();
    assert!(selected.events.is_empty());
    assert_eq!(selected.next, 3);
    let snapshot = journal.read_until(&scope, 0, |_| true, 2, 4096, 2).unwrap();
    assert_eq!(snapshot.events.len(), 1);
    assert_eq!(snapshot.next, 2);
    assert!(journal.read(&scope, 4, |_| true, 2, 4096).is_err());
    journal.record(event(Some("cycle-b"), None, "other"));
    assert!(journal.read(&scope, 3, |_| true, 2, 4096).is_err());
    journal.record(event(Some("cycle-a"), None, "recreated"));
    assert!(journal.read(&scope, 3, |_| true, 2, 4096).is_err());
    assert_eq!(
        journal.read(&scope, 0, |_| true, 2, 4096).unwrap().events[0].0,
        5
    );

    let bounded = Journal::new(Limits {
        instance_bytes: 128,
        scope_bytes: 128,
        ..Limits::default()
    });
    bounded.record(event(Some("cycle-a"), None, &"large".repeat(100)));
    let page = bounded.read(&Scope::Instance, 0, |_| true, 1, 128).unwrap();
    assert!(page.events.is_empty());
    assert_eq!(page.missed, 1);
}

#[test]
fn capture_scrubs_secrets() {
    assert_eq!(scrubbed("épassword=hidden"), "épassword=<redacted>");
    assert_eq!(
        scrubbed("password=hidden api_key=hidden \"token\":\"hidden\""),
        "password=<redacted> api_key=<redacted> \"token\":\"<redacted>\""
    );
    let journal = Journal::new(Limits::default());
    let subscriber = tracing_subscriber::registry().with(Capture::new(journal.clone()));
    tracing::subscriber::with_default(subscriber, || {
        let span = tracing::info_span!(
            "execution",
            cycle_id = "cycle-a",
            execution_id = tracing::field::Empty
        );
        let _entered = span.enter();
        span.record("execution_id", "execution-a");
        tracing::info!(target: "sidecar", plugin_id = "plugin-a", api_token = "must-not-leak", line = "hello Bearer must-not-leak", timestamp = "2026-10-13T00:00:00Z");
        span.record("execution_id", "execution-b");
        tracing::warn!(message = "changed execution", details = %"x".repeat(40_000));
        let watch = tracing::info_span!(WATCH_SPAN);
        let _watch = watch.enter();
        tracing::info!("reader traffic");
    });
    let page = journal
        .read(&Scope::Instance, 0, |_| true, 10, 65536)
        .unwrap();
    assert_eq!(page.events.len(), 2);
    let first = &page.events[0].1;
    assert_eq!(first.cycle.as_deref(), Some("cycle-a"));
    assert_eq!(first.execution.as_deref(), Some("execution-a"));
    assert_eq!(first.plugin.as_deref(), Some("plugin-a"));
    assert_eq!(first.message, "hello Bearer <redacted>");
    assert_eq!(first.time.unix_timestamp(), 1791849600);
    assert_eq!(first.fields, [("api_token".into(), "<redacted>".into())]);
    let second = &page.events[1].1;
    assert_eq!(second.execution.as_deref(), Some("execution-b"));
    assert!(second.fields[0].1.len() <= 1024);
    assert!(!format!("{first:?}").contains("must-not-leak"));
}

// Capture Has Its Own Filter And Does Not Follow The Terminal Filter.
#[test]
fn capture_filters_independently() {
    let filtered = Journal::new(Limits::default());
    let subscriber =
        tracing_subscriber::registry().with(dokime_server::trace::layer(filtered.clone()));
    tracing::subscriber::with_default(subscriber, || {
        tracing::trace!(target: "sidecar", "plugin trace");
        tracing::debug!(target: "dokime_server", "host detail");
        tracing::info!(target: "h2", "transport traffic");
        tracing::debug!(target: "unrelated", "unselected detail");
    });
    let captured = filtered
        .read(&Scope::Instance, 0, |_| true, 10, 65536)
        .unwrap();
    assert_eq!(captured.events.len(), 2);
    assert_eq!(captured.events[0].1.level, tracing::Level::TRACE);
    assert_eq!(captured.events[1].1.level, tracing::Level::DEBUG);
}
