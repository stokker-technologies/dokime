/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Integration Benchmarks: One Host Collector Read In Process

mod collectors;

use criterion::{criterion_group, criterion_main};

criterion_group!(benches, collectors::benchmark);
criterion_main!(benches);
