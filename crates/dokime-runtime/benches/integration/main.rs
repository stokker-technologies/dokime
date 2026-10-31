/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Integration Benchmarks: Runtime Capture And Durable Plot Visibility

#![recursion_limit = "256"]

/* === Modules === */
mod capture;
mod finalization;
mod plots;
mod telemetry;

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
criterion_group!(benches, finalization::benchmark, capture::benchmark, plots::benchmark, telemetry::benchmark);
criterion_main!(benches);

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
