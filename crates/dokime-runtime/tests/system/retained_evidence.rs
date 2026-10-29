/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Shared Bounded-Evidence Proof At The Lane-Owned Volume

#[tokio::test]
async fn retained_evidence() {
    dokime_runtime::testing::report_points(100_101).await;
    dokime_runtime::testing::capture(7, 1440);
    dokime_runtime::testing::records(7 * 1440);
    dokime_runtime::testing::stream(7 * 1440);
    dokime_runtime::testing::telemetry(7, 1440).await;
    dokime_runtime::testing::logs(7, 147).await;
}
