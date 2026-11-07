/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Integration Benchmarks: Runtime Capture and Document Queries

#![recursion_limit = "256"]

/* === Modules === */
mod capture;
mod documents;
mod finalization;
mod heavy_paths;
#[path = "../storage/metrics.rs"]
#[allow(dead_code)]
mod metrics;
mod plots;
mod telemetry;
#[path = "../storage/waveforms.rs"]
#[allow(dead_code)]
mod waveforms;

#[path = "../../tests/support/services/mod.rs"]
#[allow(dead_code)]
pub mod service_support;

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports

// Third Party Imports
use criterion::{criterion_group, criterion_main};

// Local Imports

/* === Constants === */

/* === Macros === */
criterion_group!(
    benches,
    documents::benchmark,
    heavy_paths::benchmark,
    finalization::benchmark,
    capture::benchmark,
    plots::benchmark,
    telemetry::benchmark
);
criterion_main!(benches);

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
