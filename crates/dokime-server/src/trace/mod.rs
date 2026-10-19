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

/* === Modules === */
mod journal;
mod layer;

/* === Re-Exports === */
pub use journal::{Event, Journal, Limits, Page, Scope, Stale};
pub use layer::{Capture, WATCH_SPAN, scrubbed};

/* === Imports === */
// Standard Library Imports
use std::sync::Arc;
// Third Party Imports
use tracing::Subscriber;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::{EnvFilter, Layer};
// Local Imports

/* === Constants === */
const CAPTURE_DIRECTIVES: &str = "info,dokime_runtime=debug,dokime_server=debug,\
dokime_host=debug,sidecar=trace,harmos=info,h2=off,hyper=off,hyper_util=off,tonic=off,\
tower=off,tower_http=off,rustls=off";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub fn layer<S>(journal: Arc<Journal>) -> Box<dyn Layer<S> + Send + Sync>
where
    S: Subscriber + for<'a> LookupSpan<'a> + Send + Sync,
{
    Capture::new(journal).with_filter(EnvFilter::new(CAPTURE_DIRECTIVES)).boxed()
}

/* === Private Functions === */

/* === Tests === */
