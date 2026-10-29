/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Telemetry Plot History At Full Scale: A 50,000-Row Plot And Seven Days Of Minute-Rate
// History; `integration/domains/telemetry/plots.rs` Guards Both Paths At 4,000 Rows And One Day.

use crate::service_support::plots;

#[tokio::test]
async fn telemetry_plot_history() {
    plots::telemetry_history(50_000, 400, 7).await;
}
