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
async fn retained_evidence() {
    dokime_runtime::testing::capture(7, 1440);
    dokime_runtime::testing::records(7 * 1440);
    dokime_runtime::testing::stream(7 * 1440);
    dokime_runtime::testing::window(7, 1440).await;
    dokime_runtime::testing::follow(7, 1440).await;
    dokime_runtime::testing::logs(7, 147).await;
}
