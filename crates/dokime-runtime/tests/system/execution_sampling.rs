/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Execution Sampling At Full Scale: 20,000-Row Cases, An 8,192-Row Whole Capture Across
// 512 KiB Pages And Six Paged Cases; `acceptance/services/execution/sampling.rs` Guards Each Path.

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
    cases: 6,
    rows: 20_000,
    spectrum_rows: 8_192,
    page_bytes: 512 * 1024,
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

#[dokime_tracing::trace_test]
#[tokio::test]
async fn paged_evidence_settles() {
    sampling::paged_evidence_settles(SIZE).await;
}

#[dokime_tracing::trace_test]
#[tokio::test]
async fn large_case_seals() {
    sampling::large_case_seals(SIZE).await;
}

#[dokime_tracing::trace_test]
#[tokio::test]
async fn whole_capture_seals() {
    sampling::whole_capture_seals(SIZE).await;
}

#[dokime_tracing::trace_test]
#[tokio::test]
async fn running_report_follows() {
    sampling::running_report_follows(SIZE).await;
}

#[dokime_tracing::trace_test]
#[tokio::test]
async fn report_shows_progress() {
    sampling::report_shows_progress(SIZE).await;
}
