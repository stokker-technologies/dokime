/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// The artifact directory and diagnostics publish, list and read back through the engine.

/* === Modules === */

pub(crate) mod diagnostics;
pub(crate) mod publication;
mod temporary_cleanup;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

#[dokime_tracing::trace_test]
#[tokio::test]
async fn engine_artifacts() {
    publication::importing_file_whole();
    publication::importing_missing_nothing();
    #[cfg(unix)]
    publication::import_fails_opened();
    diagnostics::one_redacted_diagnostic();
    diagnostics::redacted_diagnostic_directory();
}

#[dokime_tracing::trace_test]
#[tokio::test(start_paused = true)]
async fn engine_artifacts_paused() {
    publication::directory::file_sent_piece().await;
    publication::directory::file_shrank_had().await;
    publication::directory::rewrite_same_time().await;
    publication::directory::file_past_skipped().await;
    publication::directory::file_outgrows_opened().await;
    publication::directory::name_refuses_directory().await;
    publication::directory::sweep_finalizes_directory().await;
    publication::directory::one_name_refused().await;
    publication::directory::one_directory_cleanup().await;
    publication::directory::closed_backlog_tick().await;
}

/* === Tests === */
