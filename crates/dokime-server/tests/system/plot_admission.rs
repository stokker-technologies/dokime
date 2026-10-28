/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Plot Admission At Full Scale: 40,000 Saved Rows Under The Default Message Budget;
// Integration Keeps The 4,000-Row Guard Of The Same Path.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::support::plots;

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
async fn plot_admission() {
    plots::admission(plots::Admission {
        rows: 40_000,
        batch: 2_000,
        message_bytes: None,
        window: 10_000,
        oversized: 100_000,
    })
    .await;
}
