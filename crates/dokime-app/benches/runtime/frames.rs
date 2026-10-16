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
use std::{collections::BTreeMap, num::NonZeroUsize, sync::Arc, time::Duration};

// Third Party Imports
use criterion::{BenchmarkId, Criterion, Throughput};
use harmos::{
    Aggregate, Application, Identifier, Runtime,
    state::Never,
    storage::{
        QueryOptions, ReadKind, ReadSelection, RecordWrite, StorageBackends, StorageManager,
        StreamWrite,
        frame::{Expr, Frame, FrameQuery, GroupBy, ScalarField, ScalarPath},
        reduce::{Agg, ScalarKind},
    },
};

// Local Imports
use dokime::core::{
    EvidenceOrigin, RecordScope, SessionId, StepAttemptId, TelemetrySampleId, TestExecutionId,
    Timestamp, Value,
};
use dokime::storage::{
    sqlite::{Backend, Limits},
    telemetry,
};
use dokime::telemetry::TelemetrySample;

/* === Constants === */
const BATCH: u64 = 1_000;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct Bench;
struct Fixture {
    _root: tempfile::TempDir,
    storage: StorageManager<()>,
    runtime: Runtime<Bench>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Application for Bench {
    type State = ();
    type Transaction = Never;
    type Record = ();
    type Streams = ();
    type Resources = ();
}

impl Fixture {
    async fn new(rows: u64) -> Self {
        let root = tempfile::tempdir().unwrap();
        let data = Arc::new(
            Backend::primary(root.path().to_owned(), Limits::default())
                .await
                .unwrap(),
        );
        let settings = dokime_app::config::StorageSettings::default().harmos(
            Duration::from_secs(60),
            std::num::NonZeroUsize::new(8 * 1024 * 1024).unwrap(),
        );
        let recovery = Arc::new(harmos::storage::state::file::FileBackend::new(
            root.path().to_owned(),
            harmos::storage::StorageBackend::history(data.as_ref()).clone(),
            &settings,
        ));
        let storage = StorageManager::new(
            StorageBackends {
                primary: data,
                state: recovery,
                replicas: vec![],
            },
            settings,
        )
        .unwrap();
        let runtime = Runtime::<Bench>::builder((), ())
            .runtime_with_storage(&storage)
            .await
            .unwrap();
        let codec = codec();
        storage
            .write_schema(codec.schema(), codec.graph())
            .await
            .unwrap();
        let fixture = Self {
            _root: root,
            storage,
            runtime,
        };
        fixture.append(0, rows).await;
        fixture
    }

    async fn append(&self, from: u64, rows: u64) {
        let stream = id("meter");
        let schema = codec().schema().clone();
        let scope = id(TestExecutionId::from_seed("execution").as_str());
        for start in (from..from + rows).step_by(BATCH as usize) {
            let end = (start + BATCH).min(from + rows);
            let payloads = (start..end).map(sample).collect::<Vec<_>>();
            let items = (start..end)
                .map(|index| id(&format!("sample-{index}")))
                .collect::<Vec<_>>();
            let writes = payloads
                .iter()
                .zip(&items)
                .zip(start..end)
                .map(|((payload, item), index)| StreamWrite {
                    stream: &stream,
                    sequence: index,
                    record: RecordWrite {
                        schema: &schema,
                        scope: Some(&scope),
                        item,
                        source: None,
                        source_time: None,
                        payload,
                    },
                })
                .collect::<Vec<_>>();
            let mut offset = 0;
            while offset < writes.len() {
                let length = self.storage.stream_batch_len(&writes[offset..]).unwrap();
                self.storage
                    .write_stream(&writes[offset..offset + length])
                    .await
                    .unwrap();
                offset += length;
            }
        }
        self.storage.flush().await.unwrap();
    }

    async fn collect(&self, rows: u64, keyed: bool) -> harmos::Result<()> {
        let view = self
            .storage
            .view_with_limits(dokime_app::config::StorageSettings::default().interactive)
            .await?;
        let mut selection = ReadSelection::new(ReadKind::Stream);
        selection.schema = Some(codec().schema().clone());
        selection.stream = Some(id("meter"));
        let mut frame = Frame::new(NonZeroUsize::new(rows as usize).unwrap(), NonZeroUsize::MIN);
        let scan = frame.scan(
            selection,
            vec![
                field(&["values", "value"], ScalarKind::Float),
                field(
                    &["metadata", "scope", "test_execution_id"],
                    ScalarKind::Text,
                ),
                field(&["metadata", "timestamp"], ScalarKind::Datetime),
            ],
        );
        let group_by = if keyed {
            let shift = frame.shift(scan, vec![1], 2);
            GroupBy::new(scan).agg(Expr::diff(shift), Agg::Of(Aggregate::Mean))
        } else {
            GroupBy::new(scan).agg(Expr::Col(0), Agg::Of(Aggregate::Mean))
        };
        frame.group_by(group_by);
        let mut reader = self.storage.query_in(
            &view,
            FrameQuery {
                frame,
                options: QueryOptions::default(),
            },
        )?;
        reader.read().await.map(|_| ())
    }
}

/* === Public Functions === */
pub fn benchmark(criterion: &mut Criterion) {
    let executor = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .unwrap();
    let mut group = criterion.benchmark_group("storage_frames");
    group.sample_size(10);
    for rows in [1_000_u64, 5_000] {
        let fixture = executor.block_on(Fixture::new(rows));
        group.throughput(Throughput::Elements(rows));
        for (name, keyed) in [("capture_mean", false), ("keyed_elapsed", true)] {
            group.bench_with_input(BenchmarkId::new(name, rows), &rows, |bencher, rows| {
                bencher.iter(|| executor.block_on(fixture.collect(*rows, keyed)).unwrap());
            });
        }
        executor.block_on(fixture.runtime.stop()).unwrap();
    }
    group.finish();

    // Keyed Sorts Live In The Query's Bounded SQLite Allocation; Report The
    // Window At Which The Default Execution Memory First Refuses One.
    executor.block_on(async {
        let fixture = Fixture::new(0).await;
        let mut rows = 0;
        for step in std::iter::repeat_n(2_500_u64, 16) {
            fixture.append(rows, step).await;
            rows += step;
            if let Err(error) = fixture.collect(rows, true).await {
                eprintln!("storage_frames: keyed sort refused at {rows} rows: {error}");
                break;
            }
            eprintln!("storage_frames: keyed sort admitted {rows} rows");
        }
        fixture.runtime.stop().await.unwrap();
    });
}

/* === Private Functions === */
fn id(value: &str) -> Identifier {
    Identifier::new(value).unwrap()
}

fn field(names: &[&str], kind: ScalarKind) -> ScalarField {
    ScalarField {
        path: ScalarPath::Payload(names.iter().map(|name| id(name)).collect()),
        kind,
    }
}

#[harmos::message]
struct Meter {
    value: f64,
}

fn codec() -> telemetry::TypedCodec {
    telemetry::codec(
        &dokime::telemetry::TelemetryContract::declared("meter", "Meter").typed::<Meter>(),
    )
    .unwrap()
}

fn sample(index: u64) -> Vec<u8> {
    let timestamp = Timestamp::new(format!(
        "2026-10-10T12:{:02}:{:02}.{:03}Z",
        (index / 60_000) % 60,
        (index / 1_000) % 60,
        index % 1_000
    ));
    let sample = TelemetrySample {
        sample_id: TelemetrySampleId::from_seed(format!("sample-{index}")),
        stream: dokime::core::Identifier::new("meter").unwrap(),
        schema_version: 1,
        sequence: index,
        timestamp: timestamp.clone(),
        received_at: timestamp,
        source: "Bench".into(),
        scope: Some(RecordScope {
            test_execution_id: Some(TestExecutionId::from_seed("execution")),
            origin: EvidenceOrigin::Step {
                attempt_id: StepAttemptId::from_seed("attempt"),
            },
            ..RecordScope::default()
        }),
        values: BTreeMap::from([("value".into(), Value::Number(index as f64 * 0.5))]),
        summary: BTreeMap::new(),
        metadata: BTreeMap::new(),
        storage_payload: Some(harmos::Message::encode_to_vec(&Meter {
            value: index as f64 * 0.5,
        })),
    };
    codec()
        .encode(
            &telemetry::TelemetryEntry {
                epoch: SessionId::from_seed("producer"),
                ordinal: index,
                sample,
            },
            1024 * 1024,
        )
        .unwrap()
}

/* === Tests === */
