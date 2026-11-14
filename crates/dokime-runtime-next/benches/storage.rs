/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Arrow Storage and Telemetry Measurements

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    hint::black_box,
    num::NonZeroUsize,
    sync::Arc,
    time::{Duration, Instant},
};

// Third Party Imports
use criterion::{BenchmarkId, Criterion, SamplingMode, Throughput};
use prost::Message;
use prost_types::{
    DescriptorProto, FieldDescriptorProto, FileDescriptorProto, FileDescriptorSet,
    field_descriptor_proto::{Label, Type},
};
use tempfile::TempDir;

// Local Imports
use dokime_protocol::{builders::Plot, v1};
use dokime_runtime_next::{ControlledClock, Host, HostSettings, PlacementSettings, RenderSettings};
use dokime_storage::{Execution, NumericWindow, Sample, Store, storage_defaults};

/* === Constants === */

const DAYS: u64 = 3;
const PLOTS: usize = 30;
const DOCUMENT_ROWS: [usize; 3] = [307_200, 1_536_000, 3_072_000];
const QUERY_ROWS: usize = 1_000_000;
const WRITE_ROWS: usize = 1_024;
const CLOSE_ROWS: usize = 65_536;
const EXPORT_ROWS: usize = 16_384;
const INGEST_ROWS: usize = 1_000_000;
const INGEST_BATCH: usize = 4_096;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, PartialEq, Message)]
struct Row {
    #[prost(double, tag = "1")]
    value: f64,

    #[prost(string, tag = "2")]
    timestamp: String,
}
#[derive(Clone, PartialEq, Message)]
struct IngestRow {
    #[prost(uint32, tag = "1")]
    phase: u32,
    #[prost(uint64, tag = "2")]
    ordinal: u64,
    #[prost(double, tag = "3")]
    value: f64,
}
struct Fixture {
    store: Store,
    root: TempDir,
}
struct DocumentFixture {
    host: Host,
    document: v1::Document,
    _fixture: Fixture,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Fixture {
    fn new() -> Self {
        let root = tempfile::tempdir().unwrap();
        let store = Store::open(root.path()).unwrap();
        store.create(&definition()).unwrap();
        store.open_stream("execution", "readings").unwrap();
        Self { store, root }
    }
    fn populated(count: usize) -> Self {
        let fixture = Self::new();
        for first in (0..count).step_by(WRITE_ROWS) {
            fixture.store.sample_batch("execution", &samples(first, (count - first).min(WRITE_ROWS))).unwrap();
        }
        fixture.store.flush_samples().unwrap();
        fixture
    }
    fn ingestion() -> Self {
        let root = tempfile::tempdir().unwrap();
        let store = Store::open(root.path()).unwrap();
        let mut execution = definition();
        let mut catalog = harmos_protocol::v1::Catalog::decode(execution.catalog.as_slice()).unwrap();
        catalog.types.as_mut().unwrap().file[0].message_type[0].field =
            [("phase", Type::Uint32), ("ordinal", Type::Uint64), ("value", Type::Double)]
                .into_iter()
                .enumerate()
                .map(|(index, (name, kind))| FieldDescriptorProto {
                    name: Some(name.into()),
                    number: Some(index as i32 + 1),
                    label: Some(Label::Optional as i32),
                    r#type: Some(kind as i32),
                    ..Default::default()
                })
                .collect();
        catalog.streams[0].id = "samples".into();
        execution.catalog = catalog.encode_to_vec();
        store.create(&execution).unwrap();
        store.open_stream("execution", "samples").unwrap();
        Self { store, root }
    }
    fn seal(&self) {
        self.store.close_streams("execution").unwrap();
        self.store.close("execution", v1::Termination::Completed, v1::ExecutionOutcome::default()).unwrap();
        self.store.seal("execution").unwrap();
    }
}
impl DocumentFixture {
    fn new(rows: usize, plots: usize, median: bool) -> Self {
        let fixture = Fixture::new();
        let mut execution = definition();
        let mut catalog = harmos_protocol::v1::Catalog::decode(execution.catalog.as_slice()).unwrap();
        let base = catalog.streams.remove(0);
        catalog.streams = (0..plots)
            .map(|i| {
                let mut stream = base.clone();
                stream.id = format!("signal.{i}");
                stream
            })
            .collect();
        execution.identifier = "document".into();
        execution.catalog = catalog.encode_to_vec();
        fixture.store.create(&execution).unwrap();
        let per_plot = rows / plots;
        for plot in 0..plots {
            let stream = format!("signal.{plot}");
            fixture.store.open_stream("document", &stream).unwrap();
            for first in (0..per_plot).step_by(2000) {
                let mut batch = samples(first, (per_plot - first).min(2000));
                for (offset, sample) in batch.iter_mut().enumerate() {
                    let index = first + offset;
                    sample.stream = stream.clone();
                    sample.source_ns =
                        1_788_220_800_000_000_000 + ((index as u128 * DAYS as u128 * 86_400_000_000_000) / (per_plot - 1) as u128) as i64;
                    sample.row = Row {
                        value: waveform(index + plot),
                        timestamp: timestamp(sample.source_ns),
                    }
                    .encode_to_vec();
                }
                fixture.store.sample_batch("document", &batch).unwrap();
            }
        }
        fixture.store.flush_samples().unwrap();
        let host = Host::open(
            fixture.root.path(),
            HostSettings {
                clock: Arc::new(ControlledClock::new(0)),
                storage: storage_defaults(),
                placement: PlacementSettings {
                    connect_timeout: Duration::from_secs(30),
                    reconnect_delay: Duration::from_millis(25),
                    stop_grace: Duration::from_secs(30),
                    stream_window: NonZeroUsize::new(8).unwrap(),
                    message_bytes: NonZeroUsize::new(4 * 1024 * 1024).unwrap(),
                },
            },
        )
        .unwrap();
        let mut cards = (0..plots)
            .map(|i| v1::Card {
                body: Some(v1::card::Body::Plot(Plot::time_series(v1::FieldReference::new(format!("signal.{i}")).field("value")).message)),
                ..Default::default()
            })
            .collect::<Vec<_>>();
        let kinds =
            [v1::Aggregate::Count, v1::Aggregate::Min, v1::Aggregate::Max, v1::Aggregate::Mean, v1::Aggregate::Stddev, v1::Aggregate::Rms]
                .into_iter()
                .chain(median.then_some(v1::Aggregate::Median));
        let tiles = kinds
            .map(|kind| v1::SummaryTile {
                metadata: Some(v1::Metadata {
                    identifier: format!("{kind:?}"),
                    label: format!("{kind:?}"),
                    ..Default::default()
                }),
                value: Some(aggregate("signal.0", kind)),
                ..Default::default()
            })
            .collect();
        cards.insert(
            0,
            v1::Card {
                body: Some(v1::card::Body::Summary(v1::Summary { tiles })),
                ..Default::default()
            },
        );
        let document = v1::Document {
            sections: vec![v1::Section { cards, ..Default::default() }],
            ..Default::default()
        };
        Self { host, document, _fixture: fixture }
    }
    fn query(&self) {
        let snapshot = self.host.snapshot("document").unwrap();
        let document = self
            .host
            .render_document(
                &snapshot,
                self.document.clone(),
                RenderSettings {
                    page_size: NonZeroUsize::new(256).unwrap(),
                    display_points: NonZeroUsize::new(1500).unwrap(),
                },
            )
            .unwrap();
        black_box(document);
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn definition() -> Execution {
    let fields = [("value", Type::Double), ("timestamp", Type::String)]
        .into_iter()
        .enumerate()
        .map(|(index, (name, kind))| FieldDescriptorProto {
            name: Some(name.into()),
            number: Some(index as i32 + 1),
            label: Some(Label::Optional as i32),
            r#type: Some(kind as i32),
            ..Default::default()
        })
        .collect();
    let types = FileDescriptorSet {
        file: vec![FileDescriptorProto {
            name: Some("benchmark.proto".into()),
            package: Some("benchmark".into()),
            syntax: Some("proto3".into()),
            message_type: vec![DescriptorProto {
                name: Some("Row".into()),
                field: fields,
                ..Default::default()
            }],
            ..Default::default()
        }],
    };
    Execution {
        identifier: "execution".into(),
        cycle: "cycle".into(),
        origin: Some(v1::Origin {
            kind: Some(v1::origin::Kind::Import(v1::Import::default())),
        }),
        catalog: harmos_protocol::v1::Catalog {
            types: Some(types),
            streams: vec![harmos_protocol::v1::StreamDeclaration {
                id: "readings".into(),
                row: "benchmark.Row".into(),
                ..Default::default()
            }],
            ..Default::default()
        }
        .encode_to_vec(),
        ..Default::default()
    }
}
fn timestamp(at: i64) -> String {
    prost_types::Timestamp {
        seconds: at.div_euclid(1_000_000_000),
        nanos: at.rem_euclid(1_000_000_000) as i32,
    }
    .to_string()
}
fn waveform(index: usize) -> f64 {
    let index = index % 192;
    let ordinal = index / 3;
    let phase = index % 3;
    12.0 * (std::f64::consts::TAU * ordinal as f64 / 64.0 - std::f64::consts::TAU * phase as f64 / 3.0).sin()
}
fn samples(first: usize, count: usize) -> Vec<Sample> {
    (0..count)
        .map(|i| {
            let source_ns = 1_788_220_800_000_000_000 + (first + i) as i64 * 1_000_000;
            Sample {
                sequence: (first + i) as u64,
                source_ns,
                receipt_ns: source_ns,
                stream: "readings".into(),
                row: Row {
                    value: (i % WRITE_ROWS) as f64,
                    timestamp: timestamp(source_ns),
                }
                .encode_to_vec(),
                ..Default::default()
            }
        })
        .collect()
}
fn window(count: u64) -> NumericWindow<'static> {
    NumericWindow {
        execution: "execution",
        stream: "readings",
        field: "value",
        after: 0,
        through: count,
        start_ns: 1_788_220_800_000_000_000 - 86_400_000_000_000,
        end_ns: 1_788_220_800_000_000_000 + count as i64 * 1_000_000,
        bins: NonZeroUsize::new(1500).unwrap(),
    }
}
fn aggregate(stream: &str, kind: v1::Aggregate) -> v1::DataExpression {
    v1::DataExpression {
        kind: Some(v1::data_expression::Kind::Aggregate(Box::new(v1::AggregateExpression {
            kind: kind as i32,
            input: Some(Box::new(v1::DataExpression {
                kind: Some(v1::data_expression::Kind::Stream(v1::FieldReference {
                    identifier: stream.into(),
                    field: "value".into(),
                })),
                ..Default::default()
            })),
            ..Default::default()
        }))),
        ..Default::default()
    }
}
fn main() {
    let executor = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    let _entered = executor.enter();
    let mut criterion = Criterion::default().configure_from_args();
    let mut group = criterion.benchmark_group("storage");
    group.sample_size(10);
    group.sampling_mode(SamplingMode::Flat);
    let fixture = Fixture::new();
    let mut offset = 0;
    group.bench_function("durable_write/1024", |b| {
        b.iter_custom(|iterations| {
            let mut elapsed = Duration::ZERO;
            for _ in 0..iterations {
                let rows = samples(offset, WRITE_ROWS);
                offset += WRITE_ROWS;
                let started = Instant::now();
                fixture.store.sample_batch("execution", black_box(&rows)).unwrap();
                fixture.store.flush_samples().unwrap();
                elapsed += started.elapsed();
            }
            elapsed
        })
    });
    executor.block_on(fixture.store.shutdown_samples()).unwrap();
    let mut query = None;
    group.bench_function("window_query/1000000", |b| {
        let query = query.get_or_insert_with(|| Fixture::populated(QUERY_ROWS));
        b.iter(|| {
            let (column, _, count) = query.store.time_series_window(window(QUERY_ROWS as u64)).unwrap();
            assert_eq!(count, QUERY_ROWS as u64);
            assert!(column.y.len() <= 1500);
            black_box(column);
        })
    });
    group.bench_function("finalize/65536", |b| {
        b.iter_custom(|iterations| {
            let mut elapsed = Duration::ZERO;
            for _ in 0..iterations {
                let fixture = Fixture::populated(CLOSE_ROWS);
                let started = Instant::now();
                fixture.store.close_streams("execution").unwrap();
                executor.block_on(fixture.store.shutdown_samples()).unwrap();
                elapsed += started.elapsed();
            }
            elapsed
        })
    });
    let export = Fixture::populated(EXPORT_ROWS);
    export.seal();
    group.bench_function("export/16384", |b| {
        b.iter(|| {
            let mut bytes = Vec::new();
            export.store.write_execution_archive("execution", &mut bytes).unwrap();
            black_box(bytes);
        })
    });
    group.throughput(Throughput::Elements(INGEST_ROWS as u64));
    for durable in [false, true] {
        group.bench_function(BenchmarkId::new(if durable { "ingest_durable" } else { "ingest_accepted" }, INGEST_ROWS), |b| {
            b.iter_custom(|iterations| {
                let mut elapsed = Duration::ZERO;
                for _ in 0..iterations {
                    let fixture = Fixture::ingestion();
                    let started = Instant::now();
                    for first in (0..INGEST_ROWS).step_by(INGEST_BATCH) {
                        let samples = (first..INGEST_ROWS.min(first + INGEST_BATCH))
                            .map(|sequence| Sample {
                                sequence: sequence as u64,
                                source_ns: sequence as i64 * 1000,
                                receipt_ns: sequence as i64 * 1000,
                                stream: "samples".into(),
                                row: IngestRow {
                                    phase: 0,
                                    ordinal: sequence as u64,
                                    value: sequence as f64 * 0.000001,
                                }
                                .encode_to_vec(),
                                ..Default::default()
                            })
                            .collect::<Vec<_>>();
                        fixture.store.sample_batch("execution", &samples).unwrap();
                    }
                    if durable {
                        fixture.store.flush_samples().unwrap();
                    }
                    elapsed += started.elapsed();
                    executor.block_on(fixture.store.shutdown_samples()).unwrap();
                    let mut rows = 0;
                    fixture
                        .store
                        .read_batches(
                            dokime_storage::BatchSelection {
                                execution: "execution",
                                stream: "samples",
                                after: 0,
                                through: INGEST_ROWS as u64,
                                start_ns: i64::MIN,
                                end_ns: i64::MAX,
                            },
                            &["phase".into(), "ordinal".into(), "value".into()],
                            |batch| {
                                rows += batch.num_rows();
                                Ok(true)
                            },
                        )
                        .unwrap();
                    assert_eq!(rows, INGEST_ROWS);
                }
                elapsed
            })
        });
    }
    if let Some(query) = query {
        executor.block_on(query.store.shutdown_samples()).unwrap();
    }
    executor.block_on(export.store.shutdown_samples()).unwrap();
    group.finish();
    let mut group = criterion.benchmark_group("telemetry");
    group.sample_size(10);
    group.sampling_mode(SamplingMode::Flat);
    let mut median = None;
    group.bench_function("exact_median/1000000", |b| {
        let median = median.get_or_insert_with(|| DocumentFixture::new(QUERY_ROWS, 1, true));
        b.iter(|| median.query());
    });
    if let Some(median) = median {
        executor.block_on(median.host.shutdown()).unwrap();
        executor.block_on(median._fixture.store.shutdown_samples()).unwrap();
    }
    for rows in DOCUMENT_ROWS {
        let mut document = None;
        group.bench_function(BenchmarkId::new("three_day_document_30_plots", rows), |b| {
            let document = document.get_or_insert_with(|| DocumentFixture::new(rows, PLOTS, false));
            b.iter(|| document.query());
        });
        if let Some(document) = document {
            executor.block_on(document.host.shutdown()).unwrap();
            executor.block_on(document._fixture.store.shutdown_samples()).unwrap();
        }
    }
    group.finish();
    criterion.final_summary();
}

/* === Tests === */
