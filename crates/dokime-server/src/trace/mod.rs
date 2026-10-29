/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// The Backend's Live Tracing Output, Held For The Console
//
// One capture layer in the process subscriber feeds one bounded journal; the
// `Backend.WatchTrace` operation pages it per scope. The capture filter is its
// own, independent of `RUST_LOG`: Dokime and sidecar output at debug and below
// is always held, while transport crates stay out so a reader's own gRPC
// traffic never becomes the trace it reads.

mod journal;
mod layer;

pub use journal::{Event, Journal, Limits, Page, Scope, Stale};
pub use layer::{Capture, WATCH_SPAN, scrubbed};

use std::sync::Arc;

use tracing::Subscriber;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::{EnvFilter, Layer};

const CAPTURE_DIRECTIVES: &str = "info,dokime_runtime=debug,dokime_server=debug,\
dokime_host=debug,sidecar=trace,harmos=info,h2=off,hyper=off,hyper_util=off,tonic=off,\
tower=off,tower_http=off,rustls=off";

pub fn layer<S>(journal: Arc<Journal>) -> Box<dyn Layer<S> + Send + Sync>
where
    S: Subscriber + for<'a> LookupSpan<'a> + Send + Sync,
{
    Capture::new(journal).with_filter(EnvFilter::new(CAPTURE_DIRECTIVES)).boxed()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing_subscriber::prelude::*;

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
        let subscriber = tracing_subscriber::registry().with(super::layer(filtered.clone()));
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
}
