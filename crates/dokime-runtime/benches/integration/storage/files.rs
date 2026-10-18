/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Storage Benchmark Story: Durable File Publication And Finished Reads

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::*;
use criterion::{BenchmarkId, SamplingMode, Throughput};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(super) fn benchmark(criterion: &mut Criterion) {
    let executor = executor();
    let lengths = [4_096usize, 1024 * 1024];

    let mut group = criterion.benchmark_group("files_publish_durably");
    // Every iteration opens a fresh store: equal iteration counts per sample.
    group.sampling_mode(SamplingMode::Flat);
    for length in lengths {
        let bytes = bytes(length);
        group.throughput(Throughput::Bytes(length as u64));
        group.bench_function(BenchmarkId::from_parameter(length), |bencher| {
            bencher.iter_custom(|iterations| {
                let mut elapsed = Duration::ZERO;
                for _ in 0..iterations {
                    let fixture = executor.block_on(Fixture::new(4));
                    let metadata = metadata(length);
                    let started = std::time::Instant::now();
                    let file = executor.block_on(fixture.storage.write_file(metadata, &bytes)).unwrap();
                    executor.block_on(fixture.storage.flush()).unwrap();
                    elapsed += started.elapsed();
                    assert_eq!(file.length, length as u64);
                    executor.block_on(fixture.runtime.stop()).unwrap();
                }
                elapsed
            });
        });
    }
    group.finish();

    let mut group = criterion.benchmark_group("files_read_finished");
    for length in lengths {
        let bytes = bytes(length);
        let fixture = executor.block_on(Fixture::new(4));
        let file = executor.block_on(fixture.storage.write_file(metadata(length), &bytes)).unwrap();
        executor.block_on(fixture.storage.flush()).unwrap();
        let saved = fixture.storage.cursor_saved();
        let view = executor.block_on(fixture.backend.open_snapshot(&saved)).unwrap();
        assert_eq!(executor.block_on(read(&fixture, &view, &file)), bytes);
        group.throughput(Throughput::Bytes(length as u64));
        group.bench_function(BenchmarkId::from_parameter(length), |bencher| {
            bencher.iter(|| std::hint::black_box(executor.block_on(read(&fixture, &view, &file))));
        });
        executor.block_on(fixture.backend.release_snapshot(&view)).unwrap();
        executor.block_on(fixture.runtime.stop()).unwrap();
    }
    group.finish();
}

/* === Private Functions === */
fn bytes(length: usize) -> Vec<u8> {
    (0..length).map(|index| (index % 251) as u8).collect()
}
fn metadata(length: usize) -> FileMetadata {
    FileMetadata {
        id: id("artifact"),
        scope: Some(id("execution")),
        media_type: "application/octet-stream".into(),
        expected_length: Some(length as u64),
        expected_sha256: None,
    }
}
async fn read(fixture: &Fixture, view: &BackendSnapshot, file: &FileReference) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(file.length as usize);
    while bytes.len() < file.length as usize {
        let batch = fixture.backend.read_file(view, file, bytes.len() as u64, nz(64 * 1024)).await.unwrap();
        assert!(!batch.is_empty());
        bytes.extend(batch);
    }
    bytes
}

/* === Tests === */
