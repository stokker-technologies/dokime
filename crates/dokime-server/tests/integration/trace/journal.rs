/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use tracing_subscriber::prelude::*;

use dokime_server::trace::{Capture, Journal, Limits, Scope, WATCH_SPAN, scrubbed};

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
        let span = tracing::info_span!("execution", cycle_id = "cycle-a", execution_id = tracing::field::Empty);
        let _entered = span.enter();
        span.record("execution_id", "execution-a");
        tracing::info!(target: "sidecar", plugin_id = "plugin_a", api_token = "must-not-leak", credentials = "must-not-leak", line = "hello Bearer must-not-leak", timestamp = "2026-10-13T00:00:00Z");
        span.record("execution_id", "execution-b");
        tracing::warn!(message = "changed execution", details = %"x".repeat(40_000));
        let watch = tracing::info_span!(WATCH_SPAN);
        let _watch = watch.enter();
        tracing::info!("reader traffic");
    });
    let page = journal.read(&Scope::Instance, 0, |_| true, 10, 65536).unwrap();
    assert_eq!(page.events.len(), 2);
    let first = &page.events[0].1;
    assert_eq!(first.cycle.as_deref(), Some("cycle-a"));
    assert_eq!(first.execution.as_deref(), Some("execution-a"));
    assert_eq!(first.plugin.as_deref(), Some("plugin_a"));
    assert_eq!(first.message, "hello Bearer <redacted>");
    assert_eq!(first.time.unix_timestamp(), 1791849600);
    assert_eq!(first.fields, [("api_token".into(), "<redacted>".into()), ("credentials".into(), "<redacted>".into())]);
    let second = &page.events[1].1;
    assert_eq!(second.execution.as_deref(), Some("execution-b"));
    assert!(second.fields[0].1.len() <= 1024);
    assert!(!format!("{first:?}").contains("must-not-leak"));
}

// Capture Has Its Own Filter And Does Not Follow The Terminal Filter.
#[test]
fn capture_filters_independently() {
    let filtered = Journal::new(Limits::default());
    let subscriber = tracing_subscriber::registry().with(dokime_server::trace::layer(filtered.clone()));
    tracing::subscriber::with_default(subscriber, || {
        tracing::trace!(target: "sidecar", "plugin trace");
        tracing::debug!(target: "dokime_server", "host detail");
        tracing::info!(target: "h2", "transport traffic");
        tracing::debug!(target: "unrelated", "unselected detail");
    });
    let captured = filtered.read(&Scope::Instance, 0, |_| true, 10, 65536).unwrap();
    assert_eq!(captured.events.len(), 2);
    assert_eq!(captured.events[0].1.level, tracing::Level::TRACE);
    assert_eq!(captured.events[1].1.level, tracing::Level::DEBUG);
}
