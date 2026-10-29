/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// The artifact directory and diagnostics publish, list and read back through the engine.

pub(crate) mod artifacts;
pub(crate) mod diagnostics;

#[tokio::test]
async fn engine_artifacts() {
    artifacts::importing_file_whole();
    artifacts::importing_missing_nothing();
    #[cfg(unix)]
    artifacts::import_fails_opened();
    diagnostics::one_redacted_diagnostic();
    diagnostics::redacted_diagnostic_directory();
}

#[tokio::test(start_paused = true)]
async fn engine_artifacts_paused() {
    artifacts::directory::file_sent_piece().await;
    artifacts::directory::file_shrank_had().await;
    artifacts::directory::rewrite_same_time().await;
    artifacts::directory::file_past_skipped().await;
    artifacts::directory::file_outgrows_opened().await;
    artifacts::directory::name_refuses_directory().await;
    artifacts::directory::sweep_finalizes_directory().await;
    artifacts::directory::one_name_refused().await;
    artifacts::directory::one_directory_cleanup().await;
    artifacts::directory::closed_backlog_tick().await;
}
