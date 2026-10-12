/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::{env, hint::black_box, time::Duration};
// Third Party Imports
use criterion::{Criterion, criterion_group, criterion_main};
// Local Imports
use dokime_host::{Host, HostTelemetryStream};

/* === Constants === */

/* === Macros === */
criterion_group!(
    benches,
    bench_native_collectors,
    bench_fixture_collectors,
    bench_expensive_wifi_channel_occupancy
);

criterion_main!(benches);

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub fn bench_native_collectors(criterion: &mut Criterion) {
    let host = Host::default();
    let mut group = criterion.benchmark_group("host_native_collectors");
    group.sample_size(10);
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(3));

    group.bench_function("system", |bencher| {
        bencher.iter(|| {
            black_box(
                host.collect_stream(HostTelemetryStream::System)
                    .expect("system collection should complete"),
            )
        });
    });
    group.bench_function("network", |bencher| {
        bencher.iter(|| {
            black_box(
                host.collect_stream(HostTelemetryStream::Network)
                    .expect("network collection should complete"),
            )
        });
    });
    group.bench_function("wifi", |bencher| {
        bencher.iter(|| {
            black_box(
                host.collect_stream(HostTelemetryStream::Wifi)
                    .expect("wifi collection should complete"),
            )
        });
    });
    group.bench_function("bluetooth", |bencher| {
        bencher.iter(|| {
            black_box(
                host.collect_stream(HostTelemetryStream::Bluetooth)
                    .expect("bluetooth collection should complete"),
            )
        });
    });

    group.finish();
}

pub fn bench_fixture_collectors(criterion: &mut Criterion) {
    let host = Host::fixture();
    let mut group = criterion.benchmark_group("host_fixture_collectors");
    group.sample_size(10);

    for stream in dokime_host::HOST_STREAMS {
        group.bench_function(stream.as_str(), |bencher| {
            bencher.iter(|| {
                black_box(
                    host.collect_stream(*stream)
                        .expect("fixture collection should complete"),
                )
            });
        });
    }

    group.finish();
}

pub fn bench_expensive_wifi_channel_occupancy(criterion: &mut Criterion) {
    if env::var("DOKIME_HOST_BENCH_EXPENSIVE").as_deref() != Ok("1") {
        return;
    }

    let host = Host::default();
    let mut group = criterion.benchmark_group("host_expensive_collectors");
    group.sample_size(10);
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(45));

    group.bench_function("wifi_channel_occupancy", |bencher| {
        bencher.iter(|| {
            black_box(
                host.collect_stream(HostTelemetryStream::WifiChannelOccupancy)
                    .expect("wifi stream occupancy collection should complete"),
            )
        });
    });

    group.finish();
}

/* === Private Functions === */

/* === Tests === */
