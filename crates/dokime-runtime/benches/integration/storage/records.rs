/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Storage Benchmark Story: Durable Record Writes And Paged Replay On SQLite

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::time::Instant;

// Third Party Imports
use criterion::{BenchmarkId, SamplingMode, Throughput};
use harmos::storage::{ScanPage, StorageBatch, StoragePart, StorageSchema, StoredRecord, query::RecordSelection};

// Local Imports
use super::*;
use dokime_domain::model;
use {
    model::records::product::ProductCalibrationDetails, model::records::product::ProductCalibrationKind,
    model::records::product::ProductCalibrationPhase,
};

/* === Constants === */
const COUNTS: [u64; 3] = [500, 1_000, 2_000];
const BATCH_BYTES: usize = 16 * 1024 * 1024;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct Records {
    _root: tempfile::TempDir,
    backend: Backend<()>,
    schema: SchemaKey,
    records: Vec<StoredRecord>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Records {
    async fn new(count: u64) -> Self {
        let root = tempfile::tempdir().unwrap();
        let backend = Backend::<()>::primary(root.path().to_path_buf()).await.unwrap();
        backend.open().await.unwrap();
        let history = backend.history().clone();
        let position = |sequence| StorageCursor { history: history.clone(), sequence };
        let schema = dokime_runtime::storage::records::schema();
        backend
            .write(
                StorageBatch::new(
                    &[StoragePart::Schemas(&[StorageSchema::new(
                        position(1),
                        schema.clone(),
                        harmos::schema::graph::Schema::from_declaration(
                            <dokime_runtime::storage::records::RecordRow as harmos::schema::Described>::DEFINITION,
                            &harmos::config::SchemaSettings::default(),
                        )
                        .unwrap(),
                    )])],
                    nz(BATCH_BYTES),
                )
                .unwrap(),
            )
            .await
            .unwrap();
        let records = (2..count + 2)
            .map(|sequence| {
                let draft =
                    ProductCalibrationDetails::new("benchmark-charger", ProductCalibrationKind::Field, ProductCalibrationPhase::Completed)
                        .into_draft("benchmark.charger");
                let mut record = model::records::evidence::Record::new(
                    dokime_runtime::RecordId::from_seed(format!("record-{sequence}")),
                    dokime_runtime::Timestamp::new("2026-10-04T00:00:00.000000000Z"),
                    draft.record_type,
                    1,
                    draft.source,
                    draft.severity,
                );
                record.message = draft.message;
                record.details = draft.details;
                record.scope = draft.scope;
                let record = model::records::evidence::RecordEntry::from_record(sequence, record);
                StoredRecord {
                    position: position(sequence),
                    schema: schema.clone(),
                    scope: Some(id("benchmark.process")),
                    item: id(&record.record_id.to_string()),
                    source: None,
                    source_time: None,
                    payload: dokime_runtime::storage::records::encode(&record, 1024 * 1024).unwrap(),
                }
            })
            .collect();
        Self { _root: root, backend, schema, records }
    }

    async fn write(&self, batch_size: usize) {
        for batch in self.records.chunks(batch_size) {
            self.backend.write(StorageBatch::new(&[StoragePart::Records(batch)], nz(BATCH_BYTES)).unwrap()).await.unwrap();
        }
    }

    async fn replay(&self) -> usize {
        let saved = self.backend.cursor_saved().await.unwrap();
        let view = self.backend.open_snapshot(&saved).await.unwrap();
        let selection = RecordSelection { schema: self.schema.clone(), scope: None, item: None, source_start: None, source_end: None };
        let mut after = 0;
        let mut received = 0;
        loop {
            let page =
                self.backend.read_records(&view, &selection, ScanPage { after, rows: nz(256), bytes: nz(1024 * 1024) }).await.unwrap();
            let Some(last) = page.last() else {
                break;
            };
            after = last.position.sequence;
            received += page.len();
        }
        self.backend.release_snapshot(&view).await.unwrap();
        received
    }

    async fn close(self) {
        self.backend.close().await.unwrap();
    }
}

/* === Public Functions === */
pub(super) fn benchmark(criterion: &mut Criterion) {
    let executor = executor();

    let mut group = criterion.benchmark_group("backend_writes_records");
    // Every iteration opens a fresh database: equal iteration counts per sample.
    group.sampling_mode(SamplingMode::Flat);
    for batch_size in [32_usize, 2_000] {
        for count in COUNTS {
            group.throughput(Throughput::Elements(count));
            group.bench_function(BenchmarkId::new(format!("batch_{batch_size}"), count), |bencher| {
                bencher.iter_custom(|iterations| {
                    executor.block_on(async {
                        let mut elapsed = Duration::ZERO;
                        for _ in 0..iterations {
                            let records = Records::new(count).await;
                            let started = Instant::now();
                            records.write(batch_size).await;
                            elapsed += started.elapsed();
                            records.close().await;
                        }
                        elapsed
                    })
                });
            });
        }
    }
    group.finish();

    let mut group = criterion.benchmark_group("backend_replays_records");
    for count in COUNTS {
        let records = executor.block_on(async {
            let records = Records::new(count).await;
            records.write(2_000).await;
            records
        });
        assert_eq!(executor.block_on(records.replay()), records.records.len());
        group.throughput(Throughput::Elements(count));
        group.bench_function(BenchmarkId::from_parameter(count), |bencher| {
            bencher.iter(|| std::hint::black_box(executor.block_on(records.replay())));
        });
        executor.block_on(records.close());
    }
    group.finish();
}

/* === Private Functions === */

/* === Tests === */
