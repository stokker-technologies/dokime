/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Telemetry Plot History At Full Scale: A 50,000-Row Plot And Seven Days Of Minute-Rate
// History; `integration/domains/evidence/plots.rs` Guards Both Paths At 5,000 Rows And One Day.

use crate::service_support::plots;

#[tokio::test]
async fn telemetry_plot_history() {
    plots::sample_plot(50_000, 400).await;
    plots::saved_windows(7).await;
}
