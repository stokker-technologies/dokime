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
    num::{NonZeroU128, NonZeroUsize},
    sync::Arc,
    time::Duration,
};

// Third Party Imports
use harmos::{
    Aggregate, Application, Message, Runtime,
    state::Never,
    storage::{
        frame::*,
        reduce::{Agg, ExactNumber, ScalarKind, ScalarValue},
        *,
    },
};

// Local Imports
use super::storage::id;
use dokime::{
    core::{
        EvidenceOrigin, RecordId, RecordScope, SessionId, StepAttemptId, TelemetrySampleId,
        TestExecutionId, Timestamp, Value,
    },
    records::{RecordEntry, RecordPublishRequest, RecordSeverity},
    storage::{
        records,
        sqlite::{Backend, Limits, RecoveryLimits},
        telemetry,
    },
    telemetry::TelemetrySample,
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct Bench;
#[harmos::message]
#[derive(Clone, PartialEq)]
struct Measurement {
    value: u64,
}
struct Fixture {
    data: Arc<Backend<()>>,
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
    async fn new(root: &std::path::Path) -> Self {
        Self::with_limits(root, Limits::default()).await
    }

    async fn with_limits(root: &std::path::Path, limits: Limits) -> Self {
        let data = Arc::new(Backend::primary(root.to_owned(), limits).await.unwrap());
        let recovery = Arc::new(data.recovery(RecoveryLimits::default()));
        let settings = StorageSettings {
            flush_interval: Duration::from_secs(60),
            max_retries: 0,
            ..StorageSettings::default()
        };
        let storage = StorageManager::new(
            StorageBackends {
                primary: data.clone(),
                recovery,
                replicas: vec![],
            },
            settings,
        )
        .unwrap();
        let runtime = Runtime::<Bench>::builder((), ())
            .runtime_with_storage(&storage)
            .await
            .unwrap();
        storage
            .write_schema(
                &records::schema(),
                &super::storage::declared::<records::RecordRow>(),
            )
            .await
            .unwrap();
        Self {
            data,
            storage,
            runtime,
        }
    }

    async fn append(&self, index: u64, execution: &str, value: Option<Value>, raw_scope: &str) {
        let mut request = RecordPublishRequest::new(
            dokime::core::Identifier::new("fixture.fact").unwrap(),
            RecordSeverity::Info,
        );
        if let Some(value) = value {
            request.details.insert("value".into(), value);
        }
        request.scope = Some(RecordScope {
            test_execution_id: Some(TestExecutionId::from_seed(execution)),
            origin: EvidenceOrigin::Step {
                attempt_id: StepAttemptId::from_seed("fixture.attempt"),
            },
            ..RecordScope::default()
        });
        let record = request.into_record(
            RecordId::from_seed(format!("record-{index}")),
            1,
            Timestamp::new("2026-10-10T12:00:00.123456789Z"),
            "Fixture",
        );
        // Producer Sequences Restart; Physical Capture Order Remains Authoritative.
        let payload =
            records::encode(&RecordEntry::from_record(index % 2, record), 1024 * 1024).unwrap();
        self.storage
            .write_records(&[RecordWrite {
                schema: &records::schema(),
                scope: Some(&id(raw_scope)),
                item: &id(&format!("record-{index}")),
                source: None,
                source_time: None,
                payload: &payload,
            }])
            .await
            .unwrap();
    }

    async fn view(&self) -> StorageView {
        self.storage.flush().await.unwrap();
        self.storage
            .view_with_limits(StorageSettings::default().query_execution)
            .await
            .unwrap()
    }

    async fn sample(&self, index: u64, execution: &str, attempt: &str, value: Option<Value>) {
        let timestamp = Timestamp::new("2026-10-10T12:00:00.123456789Z");
        let sample = TelemetrySample {
            sample_id: TelemetrySampleId::from_seed(format!("sample-{index}")),
            stream: id("meter"),
            schema_version: 1,
            sequence: index % 2,
            timestamp: timestamp.clone(),
            received_at: timestamp,
            source: "Fixture".into(),
            scope: Some(RecordScope {
                test_execution_id: Some(TestExecutionId::from_seed(execution)),
                origin: EvidenceOrigin::Step {
                    attempt_id: StepAttemptId::from_seed(attempt),
                },
                ..RecordScope::default()
            }),
            values: value
                .map(|value| BTreeMap::from([("value".into(), value)]))
                .unwrap_or_default(),
            summary: BTreeMap::new(),
            metadata: BTreeMap::new(),
        };
        let entry = telemetry::TelemetryEntry {
            epoch: SessionId::from_seed(format!("producer-{}", index % 2)),
            ordinal: index,
            sample,
        };
        let payload = telemetry::encode(&entry, 1024 * 1024).unwrap();
        self.storage
            .write_stream(&[StreamWrite {
                stream: &id("meter"),
                sequence: index % 2,
                record: RecordWrite {
                    schema: &telemetry::schema(),
                    scope: Some(&id(&format!("epoch-{}", index % 2))),
                    item: &id(&format!("sample-{index}")),
                    source: None,
                    source_time: None,
                    payload: &payload,
                },
            }])
            .await
            .unwrap();
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn field(names: &[&str], kind: ScalarKind) -> ScalarField {
    ScalarField {
        path: ScalarPath::Payload(names.iter().map(|name| id(name)).collect()),
        kind,
    }
}
fn request() -> FrameQuery {
    let mut selection = ReadSelection::new(ReadKind::Records);
    selection.schema = Some(records::schema());
    let mut frame = Frame::new(
        NonZeroUsize::new(100).unwrap(),
        NonZeroUsize::new(16).unwrap(),
    );
    let scan = frame.scan(
        selection,
        vec![
            field(&["details", "value"], ScalarKind::Exact),
            field(&["scope", "test_execution_id"], ScalarKind::Text),
            field(&["timestamp"], ScalarKind::Datetime),
            ScalarField {
                path: ScalarPath::CaptureOrder,
                kind: ScalarKind::Unsigned,
            },
        ],
    );
    let group_by = [
        Agg::Of(Aggregate::Mean),
        Agg::Of(Aggregate::Median),
        Agg::Last,
        Agg::Len,
    ]
    .into_iter()
    .fold(
        GroupBy::new(scan).filter(Predicate::Compare {
            expr: Expr::Col(1),
            compare: Compare::Eq,
            value: ScalarValue::Text(TestExecutionId::from_seed("execution-a").as_str().into()),
        }),
        |group_by, agg| group_by.agg(Expr::Col(0), agg),
    );
    frame.group_by(group_by);
    FrameQuery {
        frame,
        options: QueryOptions::default(),
    }
}
fn correlated() -> FrameQuery {
    let mut query = request();
    let frame = &mut query.frame;
    let mut selection = ReadSelection::new(ReadKind::Stream);
    selection.schema = Some(telemetry::schema());
    selection.stream = Some(id("meter"));
    let samples = frame.scan(
        selection,
        vec![
            field(&["values", "value"], ScalarKind::Exact),
            field(&["scope", "test_execution_id"], ScalarKind::Text),
            field(&["timestamp"], ScalarKind::Datetime),
        ],
    );
    let join = frame.join_asof(AsOf {
        left: 0,
        right: samples,
        by: vec![(1, 1)],
        on: (2, 2),
        strategy: AsOfStrategy::Backward { present: 0 },
        right_filter: vec![],
    });
    frame.group_bys = vec![GroupBy::new(0).agg(Expr::AsOf { join, col: 0 }, Agg::Last)];
    query
}
async fn sorted_rows(fixture: &Fixture) -> i64 {
    fixture
        .data
        .control()
        .await
        .unwrap()
        .call(|connection| {
            connection.query_row("SELECT COUNT(*) FROM temp.storage_frame_rows", [], |row| {
                row.get(0)
            })
        })
        .await
        .unwrap()
}
fn exact(value: u64) -> ScalarValue {
    ScalarValue::Exact(ExactNumber::new(
        false,
        u128::from(value),
        NonZeroU128::new(1).unwrap(),
    ))
}

/* === Tests === */

pub(crate) async fn native_frames_result() {
    let root = tempfile::tempdir().unwrap();
    let fixture = Fixture::new(root.path()).await;
    fixture
        .append(
            1,
            "execution-a",
            Some(Value::Unsigned(u64::MAX - 2)),
            "epoch-a",
        )
        .await;
    fixture
        .append(2, "execution-b", Some(Value::Unsigned(0)), "epoch-a")
        .await;
    fixture
        .append(3, "execution-a", Some(Value::Unsigned(u64::MAX)), "epoch-b")
        .await;
    let view = fixture.view().await;
    let mut reader = fixture.storage.query_in(&view, request()).unwrap();
    fixture
        .append(4, "execution-a", Some(Value::Unsigned(0)), "epoch-c")
        .await;
    fixture.storage.flush().await.unwrap();
    let result = reader.read().await.unwrap();
    assert_eq!(
        result.outputs[0].groups[0].values,
        vec![
            ScalarValue::Unsigned(u64::MAX - 1),
            ScalarValue::Unsigned(u64::MAX - 1),
            exact(u64::MAX),
            ScalarValue::Unsigned(2)
        ]
    );
    for _ in 0..64 {
        let cached = reader.read().await.unwrap();
        assert!(std::ptr::eq(&*result, &*cached));
    }
    drop(result);
    drop(reader);
    drop(view);
    fixture.runtime.stop().await.unwrap();

    let reopened = Fixture::new(root.path()).await;
    let view = reopened.view().await;
    let mut reader = reopened.storage.query_in(&view, request()).unwrap();
    let result = reader.read().await.unwrap();
    assert_eq!(
        result.outputs[0].groups[0].values[3],
        ScalarValue::Unsigned(3)
    );
    drop(result);
    drop(reader);
    drop(view);
    reopened.runtime.stop().await.unwrap();
}

pub(crate) async fn missing_null_distinct() {
    let root = tempfile::tempdir().unwrap();
    let fixture = Fixture::new(root.path()).await;
    fixture
        .append(1, "execution-a", Some(Value::Unsigned(7)), "epoch-a")
        .await;
    fixture
        .append(2, "execution-a", Some(Value::Null), "epoch-a")
        .await;
    fixture.append(3, "execution-a", None, "epoch-a").await;
    let view = fixture.view().await;
    let mut query = request();
    let group_by = &mut query.frame.group_bys[0];
    group_by.keys = vec![1];
    group_by.groups = Some(vec![
        vec![ScalarValue::Text(
            TestExecutionId::from_seed("empty").as_str().into(),
        )],
        vec![ScalarValue::Text(
            TestExecutionId::from_seed("execution-a").as_str().into(),
        )],
    ]);
    group_by.aggs = [Agg::Last, Agg::Of(Aggregate::Count), Agg::Len]
        .into_iter()
        .map(|agg| (Expr::Col(0), agg))
        .collect();
    let mut reader = fixture.storage.query_in(&view, query).unwrap();
    let result = reader.read().await.unwrap();
    assert_eq!(
        result.outputs[0].groups[0].values,
        vec![
            ScalarValue::Missing,
            ScalarValue::Unsigned(0),
            ScalarValue::Unsigned(0)
        ]
    );
    assert_eq!(
        result.outputs[0].groups[1].values,
        vec![
            ScalarValue::Null,
            ScalarValue::Unsigned(1),
            ScalarValue::Unsigned(3)
        ]
    );
    drop(result);
    drop(reader);
    drop(view);
    fixture.runtime.stop().await.unwrap();
}

pub(crate) async fn frame_refusals_view() {
    let root = tempfile::tempdir().unwrap();
    let fixture = Fixture::new(root.path()).await;
    for index in 1..=3 {
        fixture
            .append(
                index,
                "execution-a",
                Some(Value::Unsigned(index)),
                "epoch-a",
            )
            .await;
    }
    let view = fixture.view().await;
    let mut limited = request();
    limited.frame.max_rows = NonZeroUsize::new(2).unwrap();
    let mut reader = fixture.storage.query_in(&view, limited).unwrap();
    assert!(reader.read().await.is_err());
    drop(reader);
    let mut invalid = request();
    invalid.frame.scans[0].columns[0] = field(&["error"], ScalarKind::Text);
    assert!(fixture.storage.query_in(&view, invalid).is_err());
    let mut beyond = request();
    beyond.frame.scans[0].selection.through = Some(view.saved().unwrap().sequence + 1);
    assert!(fixture.storage.query_in(&view, beyond).is_err());
    let mut reader = fixture.storage.query_in(&view, request()).unwrap();
    let result = reader.read().await.unwrap();
    assert_eq!(
        result.outputs[0].groups[0].values[3],
        ScalarValue::Unsigned(3)
    );
    drop(result);
    drop(reader);
    drop(view);
    fixture.runtime.stop().await.unwrap();
}

pub(crate) async fn cached_batch_detects() {
    let root = tempfile::tempdir().unwrap();
    let fixture = Fixture::new(root.path()).await;
    fixture
        .append(1, "execution-a", Some(Value::Unsigned(7)), "epoch-a")
        .await;
    let view = fixture.view().await;
    let mut reader = fixture.storage.query_in(&view, request()).unwrap();
    let result = reader.read().await.unwrap();
    // Deliberate Physical Fault Injection After The Completed Original Read.
    fixture
        .data
        .control()
        .await
        .unwrap()
        .call(|connection| {
            connection.execute(
                "UPDATE storage_data_entries SET digest=zeroblob(32) WHERE kind='record'",
                [],
            )
        })
        .await
        .unwrap();
    let cached = reader.read().await.unwrap();
    assert!(std::ptr::eq(&*result, &*cached));
    let mut fresh = fixture.storage.query_in(&view, request()).unwrap();
    assert!(fresh.read().await.is_err());
    drop(cached);
    drop(result);
    drop(fresh);
    drop(reader);
    drop(view);
    fixture.runtime.stop().await.unwrap();
}

pub(crate) async fn native_correlations_order() {
    let root = tempfile::tempdir().unwrap();
    let fixture = Fixture::new(root.path()).await;
    fixture
        .storage
        .write_schema(
            &telemetry::schema(),
            &super::storage::declared::<telemetry::TelemetryRow>(),
        )
        .await
        .unwrap();
    fixture
        .sample(
            1,
            "execution-a",
            "fixture.attempt",
            Some(Value::Unsigned(u64::MAX)),
        )
        .await;
    fixture
        .sample(2, "execution-a", "fixture.attempt", Some(Value::Null))
        .await;
    fixture
        .sample(3, "execution-a", "fixture.attempt", None)
        .await;
    fixture
        .sample(
            4,
            "execution-b",
            "fixture.attempt",
            Some(Value::Unsigned(99)),
        )
        .await;
    fixture
        .sample(
            5,
            "execution-a",
            "another.attempt",
            Some(Value::Unsigned(98)),
        )
        .await;
    fixture
        .append(10, "execution-a", Some(Value::Unsigned(9)), "epoch-event")
        .await;
    let view = fixture.view().await;
    let mut query = request();
    let frame = &mut query.frame;
    frame.scans[0]
        .columns
        .push(field(&["scope", "origin", "attempt_id"], ScalarKind::Text));
    let mut selection = ReadSelection::new(ReadKind::Stream);
    selection.schema = Some(telemetry::schema());
    selection.stream = Some(id("meter"));
    let samples = frame.scan(
        selection,
        vec![
            field(&["values", "value"], ScalarKind::Exact),
            field(&["scope", "test_execution_id"], ScalarKind::Text),
            field(&["timestamp"], ScalarKind::Datetime),
            ScalarField {
                path: ScalarPath::CaptureOrder,
                kind: ScalarKind::Unsigned,
            },
            field(&["scope", "origin", "attempt_id"], ScalarKind::Text),
        ],
    );
    let [backward, previous] = [
        AsOfStrategy::Backward { present: 0 },
        AsOfStrategy::Previous,
    ]
    .map(|strategy| {
        frame.join_asof(AsOf {
            left: 0,
            right: samples,
            by: vec![(1, 1), (4, 4)],
            on: (2, 2),
            strategy,
            right_filter: vec![],
        })
    });
    // A Group-By Reads One Join; Each Correlation Gets Its Own Output.
    frame.group_bys = vec![
        GroupBy::new(0).agg(
            Expr::AsOf {
                join: backward,
                col: 0,
            },
            Agg::Last,
        ),
        GroupBy::new(0)
            .agg(
                Expr::AsOf {
                    join: previous,
                    col: 0,
                },
                Agg::Last,
            )
            .agg(Expr::diff(previous), Agg::Last),
    ];
    let mut reader = fixture.storage.query_in(&view, query).unwrap();
    let result = reader.read().await.unwrap();
    assert_eq!(result.outputs[0].groups[0].values, vec![ScalarValue::Null]);
    assert_eq!(
        result.outputs[1].groups[0].values,
        vec![ScalarValue::Missing, ScalarValue::Unsigned(0)]
    );
    drop(result);
    drop(reader);
    drop(view);
    fixture.runtime.stop().await.unwrap();
}

pub(crate) async fn dropped_queued_view() {
    use std::future::Future;
    let root = tempfile::tempdir().unwrap();
    let fixture = Fixture::new(root.path()).await;
    fixture
        .append(1, "execution-a", Some(Value::Unsigned(7)), "epoch-a")
        .await;
    let view = fixture.view().await;
    let mut reader = fixture.storage.query_in(&view, request()).unwrap();
    let entered = Arc::new(tokio::sync::Notify::new());
    let signal = entered.clone();
    let (release, gate) = std::sync::mpsc::channel();
    let control = fixture.data.control().await.unwrap();
    let worker = tokio::spawn(async move {
        control
            .call(move |_| {
                signal.notify_one();
                gate.recv_timeout(Duration::from_secs(5))
                    .expect("SQLite queue gate was not released");
                Ok(())
            })
            .await
    });
    tokio::time::timeout(Duration::from_secs(5), entered.notified())
        .await
        .unwrap();
    {
        let read = reader.read();
        tokio::pin!(read);
        std::future::poll_fn(|context| {
            assert!(read.as_mut().poll(context).is_pending());
            std::task::Poll::Ready(())
        })
        .await;
    }
    release.send(()).unwrap();
    tokio::time::timeout(Duration::from_secs(5), worker)
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    let result = tokio::time::timeout(Duration::from_secs(5), reader.read())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        result.outputs[0].groups[0].values[3],
        ScalarValue::Unsigned(1)
    );
    drop(result);
    drop(reader);
    drop(view);
    fixture.runtime.stop().await.unwrap();
}

pub(crate) async fn filtered_record_pages_cut() {
    let root = tempfile::tempdir().unwrap();
    let fixture = Fixture::new(root.path()).await;
    for (index, owner, value, epoch) in [
        (0, "other", Some(Value::Unsigned(1)), "process-a"),
        (1, "selected", Some(Value::Unsigned(2)), "process-a"),
        (2, "selected", None, "process-a"),
        (3, "other", Some(Value::Unsigned(3)), "process-a"),
        (4, "selected", Some(Value::Unsigned(4)), "process-a"),
        (5, "selected", Some(Value::Unsigned(5)), "process-b"),
    ] {
        fixture.append(index, owner, value, epoch).await;
    }
    let view = fixture.view().await;
    fixture
        .append(6, "selected", Some(Value::Unsigned(6)), "process-a")
        .await;
    fixture.storage.flush().await.unwrap();

    // Nested Ownership And Process Epoch Are Independent Selectors.
    let owner = TestExecutionId::from_seed("selected");
    let filter = RecordFilter {
        columns: vec![
            field(&["scope", "test_execution_id"], ScalarKind::Text),
            field(&["details", "value"], ScalarKind::Unsigned),
            ScalarField {
                path: ScalarPath::Item,
                kind: ScalarKind::Text,
            },
        ],
        clauses: vec![
            RecordClause {
                predicates: vec![
                    Predicate::In {
                        expr: Expr::Col(0),
                        values: vec![ScalarValue::Text(owner.as_str().to_owned())],
                    },
                    Predicate::Exists {
                        expr: Expr::Col(1),
                        present: true,
                    },
                ],
            },
            RecordClause {
                predicates: vec![Predicate::In {
                    expr: Expr::Col(2),
                    values: vec![
                        ScalarValue::Text("record-0".into()),
                        ScalarValue::Text("record-1".into()),
                    ],
                }],
            },
        ],
    };
    let mut query = RecordQuery::new(records::schema(), |bytes, _| Ok(bytes.to_vec()));
    query.scope = Some(id("process-a"));
    query.filter = Some(filter);
    query.options.page = Some(ReadLimits {
        rows: NonZeroUsize::MIN,
        bytes: NonZeroUsize::new(1024 * 1024).unwrap(),
    });
    let mut reader = fixture.storage.query_in(&view, query).unwrap();
    let mut positions = Vec::new();
    for expected in [0, 1, 4] {
        let page = reader.next_page().await.unwrap().unwrap();
        assert_eq!(page.len(), 1);
        assert_eq!(page[0].item, id(&format!("record-{expected}")));
        positions.push(page[0].position.sequence);
    }
    assert!(positions.windows(2).all(|pair| pair[0] < pair[1]));
    assert!(reader.next_page().await.unwrap().is_none());
    drop(reader);
    drop(view);
    fixture.runtime.stop().await.unwrap();
}

pub(crate) async fn postcard_frames_cut() {
    let root = tempfile::tempdir().unwrap();
    let fixture = Fixture::new(root.path()).await;
    let schema = SchemaKey {
        payload_encoding: PayloadEncoding::PostcardV1,
        id: id("fixture.measurement"),
        version: std::num::NonZeroU32::MIN,
    };
    let start = fixture
        .storage
        .write_schema(&schema, &super::storage::declared::<Measurement>())
        .await
        .unwrap();
    let mut through = start.sequence;
    for (index, value) in [u64::MAX - 1, u64::MAX].into_iter().enumerate() {
        let payload = Measurement { value }.encode_to_vec();
        // Preserve Distinct Times At Windows SystemTime Precision.
        let source_nanos = 100 * (index as u64 + 1);
        let source_time = std::time::UNIX_EPOCH + Duration::from_nanos(source_nanos);
        assert_eq!(
            source_time
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            u128::from(source_nanos)
        );
        through = fixture
            .storage
            .write_records(&[RecordWrite {
                schema: &schema,
                scope: Some(&id("selected")),
                item: &id(&format!("value-{index}")),
                source: None,
                source_time: Some(source_time),
                payload: &payload,
            }])
            .await
            .unwrap()
            .sequence;
    }
    let outside = Measurement { value: 0 }.encode_to_vec();
    fixture
        .storage
        .write_records(&[RecordWrite {
            schema: &schema,
            scope: Some(&id("selected")),
            item: &id("outside"),
            source: None,
            source_time: None,
            payload: &outside,
        }])
        .await
        .unwrap();
    let view = fixture.view().await;
    let mut query = request();
    let source = &mut query.frame.scans[0];
    source.selection.schema = Some(schema);
    source.selection.scope = Some(id("selected"));
    source.selection.after = start.sequence;
    source.selection.through = Some(through);
    source.columns = vec![
        field(&["value"], ScalarKind::Unsigned),
        ScalarField {
            path: ScalarPath::SourceTime,
            kind: ScalarKind::Datetime,
        },
    ];
    let group_by = &mut query.frame.group_bys[0];
    group_by.filter.clear();
    group_by.aggs = vec![
        (Expr::Col(0), Agg::Of(Aggregate::Mean)),
        (Expr::Col(1), Agg::Last),
    ];
    let mut reader = fixture.storage.query_in(&view, query).unwrap();
    let result = reader.read().await.unwrap();
    let mean = ScalarValue::Exact(ExactNumber::new(
        false,
        2 * u128::from(u64::MAX) - 1,
        NonZeroU128::new(2).unwrap(),
    ));
    assert_eq!(
        result.outputs[0].groups[0].values,
        vec![mean, ScalarValue::Datetime(200)]
    );
    drop(result);
    drop(reader);
    drop(view);
    fixture.runtime.stop().await.unwrap();
}

pub(crate) async fn keyed_scans_writes() {
    let root = tempfile::tempdir().unwrap();
    let limits = Limits {
        memory_bytes: NonZeroUsize::new(4 * 1024 * 1024).unwrap(),
        ..Limits::default()
    };
    let fixture = Fixture::with_limits(root.path(), limits).await;
    fixture
        .storage
        .write_schema(
            &telemetry::schema(),
            &super::storage::declared::<telemetry::TelemetryRow>(),
        )
        .await
        .unwrap();
    for index in 1..=64 {
        fixture
            .sample(
                index,
                "execution-a",
                "fixture.attempt",
                Some(Value::Unsigned(index)),
            )
            .await;
    }
    fixture
        .append(100, "execution-a", Some(Value::Unsigned(1)), "epoch-a")
        .await;
    for round in 0..2u64 {
        let view = fixture.view().await;
        let mut reader = fixture.storage.query_in(&view, correlated()).unwrap();
        let result = reader.read().await.unwrap();
        assert_eq!(result.outputs[0].groups[0].values, vec![exact(64)]);
        // The Last Page Dropped Both Sorted Copies Before The View Released.
        assert_eq!(sorted_rows(&fixture).await, 0);
        drop(result);
        drop(reader);
        drop(view);
        fixture
            .append(
                101 + round,
                "execution-b",
                Some(Value::Unsigned(2)),
                "epoch-b",
            )
            .await;
    }
    fixture.runtime.stop().await.unwrap();
}

pub(crate) async fn frame_in_budget() {
    // One Row May Be Mid-Decode When Its Future Drops; No Further Row Is Read.
    const BUDGET: u64 = 1;
    tokio::time::timeout(Duration::from_secs(60), async {
        let root = tempfile::tempdir().unwrap();
        let fixture = Fixture::new(root.path()).await;
        fixture
            .storage
            .write_schema(
                &telemetry::schema(),
                &super::storage::declared::<telemetry::TelemetryRow>(),
            )
            .await
            .unwrap();
        let rows = 4096;
        for index in 1..=rows {
            fixture
                .sample(
                    index,
                    "execution-a",
                    "fixture.attempt",
                    Some(Value::Unsigned(index)),
                )
                .await;
        }
        fixture
            .append(rows + 1, "execution-a", Some(Value::Unsigned(1)), "epoch-a")
            .await;
        let view = fixture.view().await;
        let control = fixture.data.control().await.unwrap();
        let reader = fixture.storage.query_in(&view, correlated()).unwrap();
        let task = tokio::spawn(async move {
            let mut reader = reader;
            reader.read().await.map(|_| ())
        });
        while control.scanned_rows() == 0 {
            tokio::task::yield_now().await;
        }
        task.abort();
        assert!(task.await.unwrap_err().is_cancelled());
        let settled = control.scanned_rows();
        drop(view);
        // Evidence Admitted While The Scan Unwinds Is Durable At Stop.
        fixture
            .append(rows + 2, "execution-a", Some(Value::Unsigned(2)), "epoch-a")
            .await;
        fixture.runtime.stop().await.unwrap();
        assert!(control.scanned_rows() <= settled + BUDGET);
        assert!(control.scanned_rows() < rows + 1);

        let reopened = Fixture::new(root.path()).await;
        let view = reopened.view().await;
        let mut reader = reopened.storage.query_in(&view, request()).unwrap();
        let result = reader.read().await.unwrap();
        assert_eq!(
            result.outputs[0].groups[0].values[3],
            ScalarValue::Unsigned(2)
        );
        drop(result);
        drop(reader);
        drop(view);
        reopened.runtime.stop().await.unwrap();
    })
    .await
    .expect("shutdown waited on an abandoned frame");
}
