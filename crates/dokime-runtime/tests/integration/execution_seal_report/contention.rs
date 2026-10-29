/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use crate::service_support::sampling::{Contention, contended_reads};

// The 3-Follower Guard Of The Contended Read Path; `system/execution_read_contention.rs` Runs Fifteen Over 2,000 Rows.
pub(crate) async fn contended_reads_interactive() {
    contended_reads(Contention { plots: 3, rows: 200, rounds: 2 }).await;
}
