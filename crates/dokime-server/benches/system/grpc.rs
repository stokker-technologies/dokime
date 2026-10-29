/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// gRPC Benchmarks: Concurrent Observation Requests Against A Running Server

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, Throughput};
use futures_util::future::try_join_all;

use super::server::Server;
use dokime::client::{Client, Target, contracts::observation::GetRuntimeShell};

const PAGE_ITEMS: u32 = 50;

pub fn benchmark(criterion: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let server = Server::start(None);
    let client = runtime.block_on(Client::connect(Target::local(server.endpoint()).unwrap(), None, None)).unwrap();
    let request = GetRuntimeShell { limit: PAGE_ITEMS, ..Default::default() };

    let mut group = criterion.benchmark_group("grpc_serves_shell");
    for concurrency in [1_usize, 64] {
        group.throughput(Throughput::Elements(concurrency as u64));
        group.bench_function(BenchmarkId::from_parameter(concurrency), |bencher| {
            bencher.iter(|| black_box(runtime.block_on(try_join_all((0..concurrency).map(|_| client.call(request.clone())))).unwrap()));
        });
    }
    group.finish();

    drop(client);
    server.stop();
}
