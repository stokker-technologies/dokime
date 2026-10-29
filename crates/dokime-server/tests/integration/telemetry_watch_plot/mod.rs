/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Telemetry ingest, query, plot watches and plot payloads over the served surface.

mod plots;
mod telemetry;

use super::*;

#[tokio::test]
async fn telemetry_watch_plot() {
    Box::pin(telemetry::telemetry_pages_window()).await;
    Box::pin(plots::sample_watch_fixed_frontier_and_resume()).await;
    Box::pin(plots::plot_points_admission()).await;
    Box::pin(plots::late_samples_keep_time_order()).await;
    Box::pin(plots::sealed_window_all()).await;
    Box::pin(plots::saved_frontier_refreshes_affected_plots()).await;
}
