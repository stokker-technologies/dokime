/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Sampling Guards: Each Smoke-Provider Sampling Path At A Size Just Past Its Display
// Points And Page Limits; `system/execution_sampling.rs` Runs The 20,000-Row Sizes.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::service_support::sampling::{self, Sampling};

/* === Constants === */

const SIZE: Sampling = Sampling {
    paged_rows: 640,
    cases: 2,
    rows: 2_000,
    spectrum_rows: 1_024,
    page_bytes: 64 * 1024,
};

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

// Cases whose own stream spans many pages replay and seal independent of history.
#[dokime_tracing::trace_test]
#[tokio::test]
async fn paged_evidence_settles() {
    sampling::paged_evidence_settles(SIZE).await;
}

// One case larger than its display points seals its whole series.
#[dokime_tracing::trace_test]
#[tokio::test]
async fn large_case_seals() {
    sampling::large_case_seals(SIZE).await;
}

// Whole-capture transforms preserve both streams across small storage pages.
#[dokime_tracing::trace_test]
#[tokio::test]
async fn whole_capture_seals() {
    sampling::whole_capture_seals(SIZE).await;
}

// A running report follows the live plot settings and the sealed report keeps them.
#[dokime_tracing::trace_test]
#[tokio::test]
async fn running_report_follows() {
    sampling::running_report_follows(SIZE).await;
}

// Opening a running cycle's report shows on the state watch as one operation.
#[dokime_tracing::trace_test]
#[tokio::test]
async fn report_shows_progress() {
    sampling::report_shows_progress(SIZE).await;
}
