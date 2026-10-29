/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Execution Read Contention At Full Scale: Fifteen Plot Followers Over 2,000 Saved Rows;
// `integration/domains/execution/contention.rs` Guards The Path With Three Over 200.

use crate::service_support::sampling::{Contention, contended_reads};

#[tokio::test]
async fn execution_read_contention() {
    contended_reads(Contention { plots: 15, rows: 2_000, rounds: 2 }).await;
}
