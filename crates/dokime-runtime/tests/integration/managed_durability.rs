/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Compact Checkpoints Through Publication And Reopen Fault Boundaries

#[tokio::test]
async fn managed_durability() {
    crate::managed_support::managed_durability(7, 4).await;
}
