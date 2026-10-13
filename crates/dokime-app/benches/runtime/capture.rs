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
use std::{
    collections::BTreeMap,
    fs,
    num::NonZeroUsize,
    path::Path,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

// Third Party Imports
use criterion::{BenchmarkId, Criterion, Throughput};
use harmos::{
    Identifier,
    state::Transaction,
    storage::{
        ReadKind, ReadLimits, ReadSelection, ReadView, StorageBackend, StorageBatch, StorageCursor,
        StoragePart, StorageSchema, StoreHistory, StoredRecord,
    },
};
use serde_json::{Value, json};
use tracing::{
    Subscriber,
    span::{Attributes, Id},
};
use tracing_subscriber::{Layer, layer::Context, prelude::*, registry::LookupSpan};

// Local Imports
use dokime::storage::sqlite::{Backend, Limits};
use dokime_app::{
    Runtime,
    records::{ProductCalibrationDetails, ProductCalibrationKind, ProductCalibrationPhase},
    state::RuntimeState,
    transactions::PublishRecord,
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Default)]
struct AdmissionTimings(Arc<Mutex<BTreeMap<&'static str, Vec<Duration>>>>);

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl<S> Layer<S> for AdmissionTimings
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_new_span(&self, attributes: &Attributes<'_>, id: &Id, context: Context<'_, S>) {
        if attributes.metadata().target() == "dokime_app::record_admission" {
            context
                .span(id)
                .unwrap()
                .extensions_mut()
                .insert(Instant::now());
        }
    }

    fn on_close(&self, id: Id, context: Context<'_, S>) {
        let span = context.span(&id).unwrap();
        if span.metadata().target() == "dokime_app::record_admission" {
            let elapsed = span.extensions().get::<Instant>().unwrap().elapsed();
            let mut totals = self.0.lock().unwrap();
            totals
                .entry(span.metadata().name())
                .or_default()
                .push(elapsed);
        }
    }
}

/* === Public Functions === */
pub fn benchmark(criterion: &mut Criterion) {
    let timings = AdmissionTimings::default();
    let layer = timings
        .clone()
        .with_filter(tracing_subscriber::filter::filter_fn(|metadata| {
            metadata.target() == "dokime_app::record_admission"
        }));
    // A process-wide subscriber also observes the journal's worker task. Runs
    // are sequential and shut their runtime down before the next iteration.
    tracing::subscriber::set_global_default(tracing_subscriber::registry().with(layer)).unwrap();
    let executor = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .unwrap();
    let mut group = criterion.benchmark_group("record_pipeline");

    for count in [500_u64, 1_000, 2_000] {
        group.throughput(Throughput::Elements(count));
        group.bench_with_input(
            BenchmarkId::new("transaction_fold", count),
            &count,
            |bencher, count| {
                bencher.iter_custom(|iterations| {
                    (0..iterations).map(|_| transaction_fold(*count)).sum()
                });
            },
        );
        group.bench_with_input(
            BenchmarkId::new("runtime", count),
            &count,
            |bencher, count| {
                bencher.iter_custom(|iterations| {
                    (0..iterations)
                        .map(|_| executor.block_on(runtime(*count, &timings)))
                        .sum::<Duration>()
                });
            },
        );
    }
    for batch_size in [32_usize, 2_000] {
        for count in [500_u64, 1_000, 2_000] {
            group.throughput(Throughput::Elements(count));
            group.bench_with_input(
                BenchmarkId::new(format!("direct_sqlite_batch_{batch_size}"), count),
                &count,
                |bencher, count| {
                    bencher.iter_custom(|iterations| {
                        (0..iterations)
                            .map(|_| executor.block_on(direct_sqlite(*count, batch_size)))
                            .sum::<Duration>()
                    });
                },
            );
        }
    }
    group.finish();
}

/* === Private Functions === */
fn transaction_fold(count: u64) -> Duration {
    let mut state = RuntimeState::default();
    state.install_builtin_domains().unwrap();
    let transactions = (0..count)
        .map(|_| {
            PublishRecord::host(
                ProductCalibrationDetails::new(
                    "benchmark-charger",
                    ProductCalibrationKind::Field,
                    ProductCalibrationPhase::Completed,
                )
                .into_draft("benchmark.charger")
                .into(),
                dokime::core::Timestamp::now_utc(),
            )
        })
        .collect::<Vec<_>>();
    let mut checked = Duration::ZERO;
    let mut prepared = Duration::ZERO;
    let mut applied = Duration::ZERO;
    let started = Instant::now();

    for transaction in transactions {
        let at = Instant::now();
        transaction.check(&state).unwrap();
        checked += at.elapsed();

        let mut records = harmos::state::Records::default();
        let at = Instant::now();
        transaction.records(&state, &mut records);
        prepared += at.elapsed();
        std::hint::black_box(&records);

        let at = Instant::now();
        transaction.apply(&mut state);
        applied += at.elapsed();
    }

    let elapsed = started.elapsed();
    assert_eq!(state.record_count, count);
    assert_eq!(state.records.record_visibility.len() as u64, count);
    println!(
        "{}",
        json!({
            "workload": "product_calibration_transaction_fold",
            "records": count,
            "elapsed_ms": elapsed.as_secs_f64() * 1000.0,
            "check_ms": checked.as_secs_f64() * 1000.0,
            "records_ms": prepared.as_secs_f64() * 1000.0,
            "apply_ms": applied.as_secs_f64() * 1000.0,
        })
    );
    elapsed
}

async fn runtime(count: u64, timings: &AdmissionTimings) -> Duration {
    let root = tempfile::tempdir().unwrap();
    let config = super::config(root.path());
    let primary = config.storage_root.clone();
    let mut owner = Runtime::start(config).await.unwrap();
    let handle = owner.handle();
    handle.storage().flush().await.unwrap();
    let initial_files = storage_files(&primary);
    let drafts = (0..count)
        .map(|_| {
            ProductCalibrationDetails::new(
                "benchmark-charger",
                ProductCalibrationKind::Field,
                ProductCalibrationPhase::Completed,
            )
            .into_draft("benchmark.charger")
        })
        .collect::<Vec<_>>();
    let request_json_bytes = serde_json::to_vec(&drafts[0]).unwrap().len();
    timings.0.lock().unwrap().clear();

    let started = Instant::now();
    for draft in drafts {
        handle.publish_product_record(draft).await.unwrap();
    }
    let admitted = started.elapsed();
    let stages = timings
        .0
        .lock()
        .unwrap()
        .iter()
        .map(|(stage, durations)| {
            assert_eq!(durations.len() as u64, count);
            let elapsed = durations.iter().sum::<Duration>();
            let mut ordered = durations.iter().copied().enumerate().collect::<Vec<_>>();
            ordered.sort_by_key(|(_, duration)| *duration);
            let slowest = ordered.iter().rev().take(5).map(|(index, duration)| {
                json!({"call": index + 1, "elapsed_ms": duration.as_secs_f64() * 1000.0})
            }).collect::<Vec<_>>();
            (
                *stage,
                json!({
                    "calls": durations.len(),
                    "elapsed_ms": elapsed.as_secs_f64() * 1000.0,
                    "median_ms": ordered[ordered.len() / 2].1.as_secs_f64() * 1000.0,
                    "p99_ms": ordered[(ordered.len() - 1) * 99 / 100].1.as_secs_f64() * 1000.0,
                    "slowest": slowest,
                }),
            )
        })
        .collect::<BTreeMap<_, _>>();
    assert_eq!(stages.len(), 3);
    let pending = handle.storage().status();
    let reader = handle.captured_records();
    let frontier = reader.frontier();
    let captured_at = Instant::now();
    assert!(reader.wait_after(frontier - 1).await.unwrap().unwrap() >= frontier);
    let captured = captured_at.elapsed();
    let flush_at = Instant::now();
    handle.storage().flush().await.unwrap();
    let flushed = flush_at.elapsed();

    // Start a fresh reader only after the explicit save barrier. This measures
    // native saved replay rather than a read of the manager's unsaved buffer.
    let reader = handle.captured_records();
    let replay_at = Instant::now();
    let mut after = 0;
    let mut received = 0;
    while let Some(record) = reader.next(after, frontier).await.unwrap() {
        after = record.sequence;
        received += 1;
    }
    assert_eq!(received, count);
    let replayed = replay_at.elapsed();
    let final_files = storage_files(&primary);
    drop(reader);
    drop(handle);
    let shutdown_at = Instant::now();
    owner.shutdown().await.unwrap();
    let shutdown = shutdown_at.elapsed();

    eprintln!(
        "{}",
        json!({
            "workload": "captured_product_calibration",
            "records": count,
            "record_schemas": 1,
            "request_json_bytes_per_record": request_json_bytes,
            "tokio_workers": 2,
            "store": "fresh_local_sqlite",
            "admission_ms": admitted.as_secs_f64() * 1000.0,
            "admission_stages": stages,
            "capture_settlement_ms": captured.as_secs_f64() * 1000.0,
            "flush_ms": flushed.as_secs_f64() * 1000.0,
            "saved_replay_ms": replayed.as_secs_f64() * 1000.0,
            "shutdown_ms": shutdown.as_secs_f64() * 1000.0,
            "pending_after_admission": {
                "batches": pending.pending_batches,
                "bytes": pending.pending_bytes,
                "accepted": pending.accepted.sequence,
                "saved": pending.saved.sequence,
            },
            "initial_files": initial_files,
            "saved_files": final_files,
        })
    );
    admitted + captured + flushed + replayed
}

async fn direct_sqlite(count: u64, batch_size: usize) -> Duration {
    let root = tempfile::tempdir().unwrap();
    let history = StoreHistory {
        store: Identifier::new("bench").unwrap(),
        epoch: Identifier::new("run").unwrap(),
    };
    let position = |sequence| StorageCursor {
        history: history.clone(),
        sequence,
    };
    let schema = dokime::storage::records::schema();
    let backend = Backend::<()>::replica(
        root.path().to_path_buf(),
        Limits::default(),
        history.clone(),
    )
    .await
    .unwrap();
    backend.open().await.unwrap();
    backend
        .write(
            StorageBatch::new(
                &[StoragePart::Schemas(&[StorageSchema::new(
                    position(1),
                    schema.clone(),
                    harmos::schema::graph::Schema::from_declaration(
                        <dokime::storage::records::RecordRow as harmos::schema::Described>::DEFINITION,
                        &harmos::config::SchemaSettings::default(),
                    )
                    .unwrap(),
                )])],
                (16 * 1024 * 1024).try_into().unwrap(),
            )
            .unwrap(),
        )
        .await
        .unwrap();
    let mut encoding = Duration::ZERO;
    let records = (2..count + 2)
        .map(|sequence| {
            let draft = ProductCalibrationDetails::new(
                "benchmark-charger",
                ProductCalibrationKind::Field,
                ProductCalibrationPhase::Completed,
            )
            .into_draft("benchmark.charger");
            let mut record = dokime::records::Record::new(
                dokime::core::Timestamp::new("2026-10-04T00:00:00.000000000Z"),
                draft.record_type,
                1,
                draft.source,
                draft.severity,
            );
            record.record_id = dokime::core::RecordId::from_seed(format!("record-{sequence}"));
            record.message = draft.message;
            record.details = draft.details;
            record.scope = draft.scope;
            let record = dokime::records::RecordEntry::from_record(sequence, record);
            let encode_at = Instant::now();
            let payload = dokime::storage::records::encode(&record, 1024 * 1024).unwrap();
            encoding += encode_at.elapsed();
            StoredRecord {
                position: position(sequence),
                schema: schema.clone(),
                scope: Some(Identifier::new("benchmark.process").unwrap()),
                item: Identifier::new(record.record_id.to_string()).unwrap(),
                source: None,
                source_time: None,
                payload,
            }
        })
        .collect::<Vec<_>>();
    let payload_bytes: usize = records.iter().map(|record| record.payload.len()).sum();
    let initial_files = storage_files(root.path());

    let write_at = Instant::now();
    for batch in records.chunks(batch_size) {
        backend
            .write(
                StorageBatch::new(
                    &[StoragePart::Records(batch)],
                    (16 * 1024 * 1024).try_into().unwrap(),
                )
                .unwrap(),
            )
            .await
            .unwrap();
    }
    let written = write_at.elapsed();
    let saved = backend.cursor_saved().await.unwrap();
    let replay_at = Instant::now();
    let view = ReadView::new(
        Identifier::new("benchmark.view").unwrap(),
        saved.clone(),
        saved.clone(),
        backend.read_view(&saved).await.unwrap(),
    )
    .unwrap();
    let selection = ReadSelection::new(ReadKind::Records);
    let limits = ReadLimits {
        rows: NonZeroUsize::new(256).unwrap(),
        bytes: NonZeroUsize::new(1024 * 1024).unwrap(),
    };
    let mut cursor = None;
    let mut received = 0;
    let mut pages = 0;
    loop {
        let page = backend
            .read_records(&view, &selection, cursor.as_ref(), limits)
            .await
            .unwrap();
        pages += 1;
        for record in page.entries {
            assert_eq!(record, records[received]);
            received += 1;
        }
        cursor = page.next;
        if cursor.is_none() {
            break;
        }
    }
    assert_eq!(received, records.len());
    backend.release_view(view.snapshot()).await.unwrap();
    let replayed = replay_at.elapsed();
    let final_files = storage_files(root.path());
    backend.close().await.unwrap();

    eprintln!(
        "{}",
        json!({
            "workload": "direct_sqlite_product_calibration",
            "records": count,
            "record_schemas": 1,
            "payload_bytes": payload_bytes,
            "encoding_ms": encoding.as_secs_f64() * 1000.0,
            "schema": schema,
            "durable_write_calls": records.chunks(batch_size).len(),
            "read_records_calls": pages,
            "batch_size": batch_size,
            "durable_write_ms": written.as_secs_f64() * 1000.0,
            "saved_replay_ms": replayed.as_secs_f64() * 1000.0,
            "initial_files": initial_files,
            "saved_files": final_files,
        })
    );
    written + replayed
}

fn storage_files(root: &Path) -> Value {
    let mut files = serde_json::Map::new();
    for name in ["local.sqlite", "local.sqlite-wal", "local.sqlite-shm"] {
        let path = root.join(name);
        match fs::metadata(path) {
            Ok(metadata) => {
                files.insert(name.into(), json!({"bytes": metadata.len()}));
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => panic!("inspect {name}: {error}"),
        }
    }
    Value::Object(files)
}

/* === Tests === */
