/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// System Benchmarks: The Built Server Process Under Steady Load

mod grpc;
mod server;
mod shutdown;
mod workflows;

use criterion::{criterion_group, criterion_main};

criterion_group!(benches, grpc::benchmark, shutdown::benchmark, workflows::benchmark);
criterion_main!(benches);
