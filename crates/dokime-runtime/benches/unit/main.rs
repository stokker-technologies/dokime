/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Unit Benchmarks: One Kernel, No I/O

mod evidence;
mod transactions;

use criterion::{criterion_group, criterion_main};

criterion_group!(benches, evidence::benchmark, transactions::benchmark);
criterion_main!(benches);
