/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Integration Benchmarks: Runtime Capture And Durable Plot Visibility

/* === Modules === */
mod capture;
mod plots;

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports

// Third Party Imports
use criterion::{criterion_group, criterion_main};

// Local Imports

/* === Constants === */

/* === Macros === */
criterion_group!(benches, capture::benchmark, plots::benchmark);
criterion_main!(benches);

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
