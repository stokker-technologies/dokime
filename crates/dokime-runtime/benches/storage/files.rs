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
use std::time::{Duration, Instant};

// Third Party Imports
use criterion::{BenchmarkId, Criterion, Throughput};
use harmos::Identifier;
use harmos::storage::FileRead;
use harmos::storage::{ByteRange, FileMetadata, FileReference, FileSource, StorageManager};
use sha2::{Digest, Sha256};

// Local Imports
use super::{fixture::Fixture, waveforms};

/* === Constants === */

const FILE_BYTES: usize = 30 * 1024 * 1024;
const CHUNK_BYTES: usize = 1024 * 1024;
const PRODUCERS: usize = 4;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn run(criterion: &mut Criterion, executor: &tokio::runtime::Runtime) {
    let mut group = criterion.benchmark_group("storage/files");
    group.sample_size(10);
    group.sampling_mode(criterion::SamplingMode::Flat);
    let payload = waveforms::binary(CHUNK_BYTES);
    let full_digest = expected_digest(&payload, FILE_BYTES);
    let distinct_digest = expected_digest(&payload, FILE_BYTES / PRODUCERS);
    group.throughput(Throughput::Bytes(FILE_BYTES as u64));
    for action in ["publish", "read", "distinct-serial", "distinct-concurrent", "distinct-serial-read", "distinct-concurrent-read"] {
        let producers = if action.starts_with("distinct") { PRODUCERS } else { 1 };
        let reading = action.ends_with("read");
        let concurrent = action.starts_with("distinct-concurrent");
        group.bench_function(BenchmarkId::new(action, FILE_BYTES), |bencher| {
            bencher.iter_custom(|iterations| {
                executor.block_on(async {
                    let mut elapsed = Duration::ZERO;
                    let mut resources = super::metrics::Observed::default();
                    let mut configuration = None;
                    for _ in 0..iterations {
                        let fixture = Fixture::new().await;
                        configuration = Some(fixture.settings.clone());
                        let count = producers;
                        let length = FILE_BYTES / count;
                        let probe = super::metrics::Probe::start(fixture.storage.clone());
                        let started = Instant::now();
                        let mut references = Vec::new();
                        if concurrent {
                            let producers = (0..count).map(|id| publish(&fixture.storage, id, length));
                            references = futures_util::future::join_all(producers).await;
                        } else {
                            for id in 0..count {
                                references.push(publish(&fixture.storage, id, length).await);
                            }
                        }
                        fixture.storage.flush().await.unwrap();
                        let publication = started.elapsed();
                        let published = probe.finish();
                        let probe = super::metrics::Probe::start(fixture.storage.clone());
                        let expected = if count == 1 { full_digest } else { distinct_digest };
                        let maximum = fixture.settings.response.batch_bytes.get();
                        let read = if concurrent {
                            verify_concurrent(&fixture.storage, references, length, expected, maximum).await
                        } else {
                            let mut elapsed = Duration::ZERO;
                            for reference in references {
                                elapsed += verify(&fixture.storage, reference, length, expected, maximum).await;
                            }
                            elapsed
                        };
                        let downloaded = probe.finish();
                        resources.merge(if reading { downloaded } else { published });
                        elapsed += if reading { read } else { publication };
                        assert_eq!(fixture.storage.status().pending_bytes, 0);
                        fixture.finish().await;
                    }
                    resources.report(&format!("files/{action}"), FILE_BYTES, producers, configuration.as_ref());
                    elapsed
                })
            });
        });
    }
    group.finish();
}
/* === Private Functions === */

async fn publish(storage: &StorageManager, id: usize, length: usize) -> FileReference {
    let metadata = FileMetadata {
        id: Identifier::new(format!("binary-{id}")).unwrap(),
        scope: None,
        media_type: "application/octet-stream".into(),
        expected_length: Some(length as u64),
        expected_sha256: None,
    };
    let mut file = storage.produce_file(metadata).await.unwrap();
    for offset in (0..length).step_by(CHUNK_BYTES) {
        file.write_all(waveforms::binary(CHUNK_BYTES.min(length - offset))).await.unwrap();
    }
    file.finish().await.unwrap()
}

fn expected_digest(chunk: &[u8], length: usize) -> [u8; 32] {
    let mut digest = Sha256::new();
    for offset in (0..length).step_by(CHUNK_BYTES) {
        digest.update(&chunk[..CHUNK_BYTES.min(length - offset)]);
    }
    digest.finalize().into()
}

async fn verify(
    storage: &StorageManager, reference: FileReference, length: usize, expected: [u8; 32], maximum_batch_bytes: usize,
) -> Duration {
    assert_eq!(reference.length, length as u64);
    assert_eq!(reference.sha256, expected);
    let request = FileRead { source: FileSource::Stored(reference), range: ByteRange { offset: 0, limit: None } };
    let started = Instant::now();
    let mut reader = storage.read_file(request).await.unwrap();
    let mut elapsed = started.elapsed();

    let mut received = 0;
    let mut digest = Sha256::new();
    loop {
        let started = Instant::now();
        let batch = reader.next().await.unwrap();
        elapsed += started.elapsed();
        let Some(batch) = batch else { break };
        assert!(batch.bytes().len() <= maximum_batch_bytes);
        assert_eq!(batch.offset(), received as u64);
        received += batch.bytes().len();
        digest.update(batch.bytes());
    }
    assert_eq!(received, length);
    assert_eq!(<[u8; 32]>::from(digest.finalize()), expected);
    elapsed
}

async fn verify_concurrent(
    storage: &StorageManager, references: Vec<FileReference>, length: usize, expected: [u8; 32], maximum_batch_bytes: usize,
) -> Duration {
    for reference in &references {
        assert_eq!(reference.length, length as u64);
        assert_eq!(reference.sha256, expected);
    }
    let requests = references
        .into_iter()
        .map(|reference| FileRead { source: FileSource::Stored(reference), range: ByteRange { offset: 0, limit: None } })
        .collect::<Vec<_>>();
    let count = requests.len();
    let mut received = vec![0usize; count];
    let mut digests = vec![Sha256::new(); count];
    let mut complete = vec![false; count];
    let started = Instant::now();
    let readers = futures_util::future::join_all(requests.into_iter().map(|request| storage.read_file(request))).await;
    let mut elapsed = started.elapsed();
    let mut readers = readers.into_iter().map(Result::unwrap).collect::<Vec<_>>();
    while complete.iter().any(|done| !done) {
        // Measure One Parallel I/O Round; Validate Its Retained Pages Afterwards.
        let started = Instant::now();
        let pages = futures_util::future::join_all(
            readers.iter_mut().zip(&complete).map(|(reader, done)| async move { if *done { None } else { reader.next().await.unwrap() } }),
        )
        .await;
        elapsed += started.elapsed();
        for (index, page) in pages.into_iter().enumerate() {
            let Some(page) = page else {
                complete[index] = true;
                continue;
            };
            assert_eq!(page.offset(), received[index] as u64);
            assert!(page.bytes().len() <= maximum_batch_bytes);
            received[index] += page.bytes().len();
            digests[index].update(page.bytes());
        }
    }
    for (received, digest) in received.into_iter().zip(digests) {
        assert_eq!(received, length);
        assert_eq!(<[u8; 32]>::from(digest.finalize()), expected);
    }
    elapsed
}
/* === Tests === */
