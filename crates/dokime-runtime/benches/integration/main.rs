/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Integration Benchmarks: Runtime Capture And Durable Plot Visibility

mod capture;
mod plots;

use criterion::{criterion_group, criterion_main};

criterion_group!(benches, capture::benchmark, plots::benchmark);
criterion_main!(benches);
