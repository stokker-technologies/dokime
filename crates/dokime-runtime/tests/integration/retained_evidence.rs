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
    dokime_runtime::testing::report_points(128).await;
    dokime_runtime::testing::capture(2, 120);
    dokime_runtime::testing::records(2 * 120);
    dokime_runtime::testing::stream(2 * 120);
    dokime_runtime::testing::telemetry(2, 120).await;
    dokime_runtime::testing::logs(2, 2).await;
}
