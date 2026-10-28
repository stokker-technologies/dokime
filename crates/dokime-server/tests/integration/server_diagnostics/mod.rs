/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Diagnostic collection, log streaming and trace journal watches.

/* === Modules === */

mod diagnostics;
mod trace;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[tokio::test]
async fn server_diagnostics() {
    Box::pin(diagnostics::collected_archive_material()).await;
    Box::pin(diagnostics::collected_tail_preserves_selected_bytes()).await;
    Box::pin(diagnostics::log_stream_file()).await;
    Box::pin(diagnostics::quiet_log_watch_outlives_the_idle_deadline()).await;
    Box::pin(diagnostics::level_filter_stream()).await;
    Box::pin(diagnostics::missing_file_live()).await;
    Box::pin(trace::trace_watch_replays()).await;
    Box::pin(trace::trace_watch_follows()).await;
    trace::journal::capture_scrubs_secrets();
    trace::journal::capture_filters_independently();
}
