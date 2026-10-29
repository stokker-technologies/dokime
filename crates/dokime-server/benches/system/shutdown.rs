/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Shutdown Benchmarks: Idle Server Drain After Stdin Closes

use std::{path::Path, time::Duration};

use criterion::{BenchmarkId, Criterion, SamplingMode};

use super::server::{Server, plugin_bundle};
use dokime::client::{Client, Target};

const SETTLE: Duration = Duration::from_secs(2);

pub fn benchmark(criterion: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let plugin = plugin_bundle();
    let mut scenarios = vec![("empty", false, false, None), ("idle_client", true, false, None), ("idle_watch", true, true, None)];
    if let Some(plugin) = &plugin {
        scenarios.push(("idle_plugin", true, true, Some(plugin.as_path())));
    }

    let mut group = criterion.benchmark_group("server_shuts_down");
    // Every iteration starts a process: equal iteration counts per sample.
    group.sampling_mode(SamplingMode::Flat);
    for (name, connected, watching, plugin) in scenarios {
        group.bench_function(BenchmarkId::from_parameter(name), |bencher| {
            bencher.iter_custom(|iterations| (0..iterations).map(|_| idle_shutdown(&runtime, connected, watching, plugin)).sum());
        });
    }
    group.finish();
}

fn idle_shutdown(runtime: &tokio::runtime::Runtime, connected: bool, watching: bool, plugin: Option<&Path>) -> Duration {
    let server = Server::start(plugin);
    let target = || Target::local(server.endpoint()).unwrap();
    let (client, watch) = runtime.block_on(async {
        let client = if connected { Some(Client::connect(target(), None, None).await.unwrap()) } else { None };
        let watch = if watching {
            let observer = Client::connect(target(), None, None).await.unwrap();
            let stream = observer.stream(dokime::client::contracts::observation::WatchRuntime::default()).await.unwrap();
            Some((observer, stream))
        } else {
            None
        };
        tokio::time::sleep(SETTLE).await;
        (client, watch)
    });
    let elapsed = server.stop();
    drop(watch);
    drop(client);
    elapsed
}
