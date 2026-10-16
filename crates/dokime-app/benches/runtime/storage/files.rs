/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Storage Benchmark Story

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::*;
use criterion::{BenchmarkId, Throughput};

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
    let mut group = criterion.benchmark_group("storage_files");
    group.sample_size(10);
    for length in [4_096usize, 1024 * 1024] {
        let bytes = (0..length)
            .map(|index| (index % 251) as u8)
            .collect::<Vec<_>>();
        group.throughput(Throughput::Bytes(length as u64));
        group.bench_function(BenchmarkId::new("publish_durable", length), |bencher| {
            bencher.iter_custom(|iterations| {
                let mut elapsed = Duration::ZERO;
                for _ in 0..iterations {
                    let fixture = executor.block_on(Fixture::new(4));
                    let metadata = metadata(length);
                    let started = std::time::Instant::now();
                    let file = executor
                        .block_on(fixture.storage.write_file(metadata, &bytes))
                        .unwrap();
                    executor.block_on(fixture.storage.flush()).unwrap();
                    elapsed += started.elapsed();
                    assert_eq!(file.length, length as u64);
                    executor.block_on(fixture.runtime.stop()).unwrap();
                }
                elapsed
            });
        });
        let fixture = executor.block_on(Fixture::new(4));
        let file = executor
            .block_on(fixture.storage.write_file(metadata(length), &bytes))
            .unwrap();
        executor.block_on(fixture.storage.flush()).unwrap();
        let saved = fixture.storage.cursor_saved();
        let snapshot = executor
            .block_on(fixture.backend.read_view(&saved))
            .unwrap();
        let view = ReadView::new(id("file-benchmark"), saved.clone(), saved, snapshot).unwrap();
        let expected = executor.block_on(read(&fixture, &view, &file));
        assert_eq!(expected, bytes);
        group.bench_function(BenchmarkId::new("read_finished", length), |bencher| {
            bencher.iter(|| {
                let result = executor.block_on(read(&fixture, &view, &file));
                std::hint::black_box(result);
            });
        });
        executor
            .block_on(fixture.backend.release_view(view.snapshot()))
            .unwrap();
        executor.block_on(fixture.runtime.stop()).unwrap();
    }
    group.finish();
}

/* === Private Functions === */
fn metadata(length: usize) -> FileMetadata {
    FileMetadata {
        id: id("artifact"),
        scope: Some(id("execution")),
        media_type: "application/octet-stream".into(),
        expected_length: Some(length as u64),
        expected_sha256: None,
    }
}
async fn read(fixture: &Fixture, view: &ReadView, file: &FileReference) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(file.length as usize);
    while bytes.len() < file.length as usize {
        let batch = fixture
            .backend
            .read_file(view, file, bytes.len() as u64, nz(64 * 1024))
            .await
            .unwrap();
        assert!(!batch.is_empty());
        bytes.extend(batch);
    }
    bytes
}

/* === Tests === */
