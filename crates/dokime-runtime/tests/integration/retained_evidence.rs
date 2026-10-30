/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Shared Bounded-Evidence Proof At The Lane-Owned Volume

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[tokio::test]
async fn joined_capture_drains() {
    dokime_runtime::testing::joined_drains().await;
}

#[tokio::test]
async fn early_log_stop() {
    dokime_runtime::testing::early_log_stop().await;
}

#[tokio::test]
async fn retained_evidence() {
    dokime_runtime::testing::capture(2, 120);
    dokime_runtime::testing::records(2 * 120);
    dokime_runtime::testing::stream(2 * 120);
    dokime_runtime::testing::telemetry(2, 120).await;
    dokime_runtime::testing::logs(2, 2).await;
}
