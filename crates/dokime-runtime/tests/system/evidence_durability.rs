/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Evidence Durability Through Actual SQLite Fault And Retry Boundaries

#[tokio::test]
async fn evidence_durability() {
    crate::evidence_support::evidence_durability(7, 1440).await;
}
