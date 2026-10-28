/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use serde::Deserialize;
use serde::Serialize;

use super::*;

// Pages the backend's live tracing output for one scope, then optionally follows
// it. Read access, like the hub log it complements.
#[harmos::operation(resources = [crate::hosting::Host<WatchTrace>], response = (), output = TraceBatch, id = "dokime.backend.watch_trace",
        policy = FOLLOW, visibility = harmos_domain::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Backend", method = "WatchTrace"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct WatchTrace {
    // Required when resuming a nonzero sequence; obtained from the prior batch.
    #[row(description = "Required when resuming a nonzero sequence; obtained from the prior batch.")]
    pub epoch: Option<String>,

    // Absent means Instance: every captured event of this backend process.
    #[row(description = "Absent means Instance: every captured event of this backend process.")]
    pub scope: Option<TraceScope>,

    // Events below this level are never delivered. Unspecified means TRACE.
    #[row(description = "Events below this level are never delivered. Unspecified means TRACE.")]
    pub min_level: LogLevel,

    // Resume after this scope sequence. Zero starts at the oldest retained event;
    // a sequence this backend process never issued is refused as out of range.
    #[row(
        description = "Resume after this scope sequence. Zero starts at the oldest retained event; a sequence this backend process never issued is refused as out of range."
    )]
    pub after_sequence: u64,

    // Events per batch. Zero uses 256; larger values are clamped to 1024.
    #[row(description = "Events per batch. Zero uses 256; larger values are clamped to 1024.")]
    pub max_events: u32,

    // Encoded bytes per batch. Zero uses 256 KiB; values are clamped between
    // 32 KiB and the backend's message budget.
    #[row(description = "Encoded bytes per batch. Zero uses 256 KiB; values are clamped between 32 KiB and the backend's message budget.")]
    pub max_bytes: u32,

    // False ends the stream once the events retained at admission are
    // delivered; true keeps following until the caller cancels.
    #[row(
        description = "False ends the stream once the events retained at admission are delivered; true keeps following until the caller cancels."
    )]
    pub follow: bool,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(WatchTrace);
}
