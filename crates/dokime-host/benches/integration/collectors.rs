/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Collector Benchmarks: Native And Fixture Stream Reads

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::hint::black_box;

// Third Party Imports
use criterion::{BenchmarkId, Criterion};

// Local Imports
use dokime_host::{HOST_STREAMS, Host, HostTelemetryStream};

/* === Constants === */
const NATIVE_STREAMS: [(&str, HostTelemetryStream); 4] = [
    ("collector_reads_system", HostTelemetryStream::System),
    ("collector_reads_network", HostTelemetryStream::Network),
    ("collector_reads_wifi", HostTelemetryStream::Wifi),
    ("collector_reads_bluetooth", HostTelemetryStream::Bluetooth),
];

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub fn benchmark(criterion: &mut Criterion) {
    let host = Host::default();
    for (name, stream) in NATIVE_STREAMS {
        criterion.bench_function(name, |bencher| {
            bencher.iter(|| {
                black_box(
                    host.collect_stream(stream)
                        .expect("native collection should complete"),
                )
            });
        });
    }

    let fixture = Host::fixture();
    let mut group = criterion.benchmark_group("fixture_collects_streams");
    for stream in HOST_STREAMS {
        group.bench_function(BenchmarkId::from_parameter(stream.as_str()), |bencher| {
            bencher.iter(|| {
                black_box(
                    fixture
                        .collect_stream(*stream)
                        .expect("fixture collection should complete"),
                )
            });
        });
    }
    group.finish();
}

/* === Private Functions === */

/* === Tests === */
