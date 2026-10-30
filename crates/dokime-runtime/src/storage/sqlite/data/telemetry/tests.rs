/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Arrow Catalog, Flush Policy And Live Window Proofs

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{num::NonZeroUsize, sync::Arc, time::Duration};

// Third Party Imports
use harmos::{
    Application, Runtime,
    storage::{RecordInput, StorageBackend, StorageBackends, StreamInput},
};

// Local Imports
use super::*;
use crate::storage::{Manager, telemetry};
use dokime_domain::model::{
    execution::RecordScope,
    identity::{SessionId, TelemetrySampleId, TestExecutionId},
    time::Timestamp,
    values::Value as Cell,
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct App;

#[derive(Default, serde::Deserialize)]
struct Config {
    #[serde(skip)]
    storage: Option<Manager>,
}

struct Fixture {
    root: tempfile::TempDir,
    local: State,
    storage: Manager,
    runtime: Runtime<App>,
    codec: Arc<telemetry::TypedCodec>,
    execution: TestExecutionId,
}

#[harmos::message]
struct Reading {
    value: f64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Application for App {
    type Lanes = crate::app::Lanes;
    type State = ();
    type Transaction = harmos::state::Never;
    type Record = harmos::state::Never;
    type Streams = ();
}

impl harmos::Lifecycle for App {
    type Config = Config;
    type Error = harmos::Error;
    type Context = harmos::Startup<Self>;
    async fn prepare(config: Config, setup: &mut harmos::Setup<Self>) -> harmos::Result<Self> {
        setup.settings().storage(config.storage.as_ref().unwrap());
        Ok(Self)
    }
    async fn initialize(&mut self, _: &Self::Context) -> harmos::Result<()> {
        Ok(())
    }
    async fn stop(&mut self, _: &Self::Context) -> harmos::Result<()> {
        Ok(())
    }
    async fn cleanup(&mut self) -> harmos::Result<()> {
        Ok(())
    }
}

impl Fixture {
    async fn new(rows: usize) -> Self {
        let root = tempfile::tempdir().unwrap();
        Self::with_root(root, rows).await
    }

    async fn with_root(root: tempfile::TempDir, rows: usize) -> Self {
        let settings = crate::storage::sqlite::Settings {
            telemetry_rows: NonZeroUsize::new(rows).unwrap(),
            telemetry_interval: Duration::from_secs(60),
            ..crate::config::local_storage_defaults()
        };
        let backend = Arc::new(crate::storage::Backend::primary_with(root.path().to_owned(), settings).await.unwrap());
        let local = backend.local_state();
        let state = Arc::new(harmos::storage::state::file::FileBackend::new(
            root.path().to_owned(),
            backend.history().clone(),
            crate::config::storage_state_defaults(),
        ));
        let storage = Manager::new(StorageBackends { primary: backend, state }, crate::config::storage_defaults()).unwrap();
        let runtime = Runtime::<App>::launch((), Config { storage: Some(storage.clone()) }).await.unwrap();
        let contract = crate::telemetry::TelemetryContract::declared("arrow.fixture", "Arrow").typed::<Reading>();
        let codec = Arc::new(telemetry::codec(&contract, &Default::default()).unwrap());
        storage.write_schema(codec.schema().clone(), codec.graph().clone()).await.unwrap();
        storage.flush().await.unwrap();
        local.register_telemetry_contract(contract, codec.clone()).await.unwrap();
        Self {
            root,
            local,
            storage,
            runtime,
            codec,
            execution: TestExecutionId::from_seed("arrow.execution"),
        }
    }

    fn row(&self, index: u64, seconds: i64) -> (StreamInput, TelemetryEntry) {
        let epoch = SessionId::from_seed("arrow.session");
        let timestamp = Timestamp::from_datetime(time::OffsetDateTime::UNIX_EPOCH + time::Duration::seconds(seconds));
        let mut sample = crate::telemetry::TelemetrySample::new(
            TelemetrySampleId::from_seed(format!("arrow.sample.{index}")),
            harmos::Identifier::new("arrow.fixture").unwrap(),
            index,
            timestamp,
            "fixture",
        )
        .typed(&Reading { value: index as f64 });
        sample.values.insert("value".into(), Cell::Number(index as f64));
        sample.scope = Some(RecordScope {
            test_execution_id: Some(self.execution.clone()),
            ..Default::default()
        });
        let entry = TelemetryEntry {
            epoch: epoch.clone(),
            ordinal: index,
            sample,
        };
        let payload = self.codec.encode(&entry, usize::MAX).unwrap();
        let input = StreamInput {
            stream: entry.sample.stream.clone(),
            sequence: index,
            record: RecordInput {
                schema: self.codec.schema().clone(),
                scope: Some(harmos::Identifier::new(self.execution.as_str()).unwrap()),
                item: harmos::Identifier::new(entry.sample.sample_id.as_str()).unwrap(),
                source: Some(harmos::Identifier::new(epoch.as_str()).unwrap()),
                source_time: Some(entry.sample.timestamp.to_datetime().unwrap().into()),
                payload,
            },
        };
        (input, entry)
    }

    async fn admit(&self, points: &[(u64, i64)]) -> StorageCursor {
        let (rows, entries): (Vec<_>, Vec<_>) = points.iter().map(|(index, time)| self.row(*index, *time)).unzip();
        let cursor = self.storage.write_stream(rows).await.unwrap();
        self.local.stage_telemetry(self.storage.clone(), cursor.clone(), entries).await.unwrap();
        cursor
    }

    fn selection(&self) -> StreamSelection {
        StreamSelection {
            schema: self.codec.schema().clone(),
            scope: Some(harmos::Identifier::new(self.execution.as_str()).unwrap()),
            item: None,
            stream: None,
            source_start: None,
            source_end: None,
            after: 0,
        }
    }

    async fn read(&self, at: &StorageCursor, source: StreamSelection) -> Vec<(u64, TelemetryEntry)> {
        self.local
            .telemetry_read(
                at.clone(),
                source,
                ScanPage {
                    after: 0,
                    rows: NonZeroUsize::new(1024).unwrap(),
                    bytes: NonZeroUsize::MAX,
                },
            )
            .await
            .unwrap()
    }

    async fn saved(&self, through: &StorageCursor) {
        let mut saved = self.storage.cursor_saved();
        while !saved.covers(through).unwrap() {
            saved = self.storage.wait_saved_after(&saved).await.unwrap();
        }
    }

    async fn stop(self) -> tempfile::TempDir {
        self.storage.flush().await.unwrap();
        self.runtime.shutdown().await.unwrap();
        self.storage.close().await.unwrap();
        self.root
    }

    async fn finish(self) {
        self.stop().await.close().unwrap();
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[tokio::test]
async fn configured_row_flush_preserves_saved_order_and_combines_tail() {
    let fixture = Fixture::new(2).await;
    let saved = fixture.storage.cursor_saved();
    let first = fixture.admit(&[(1, 10)]).await;
    assert_eq!(fixture.storage.cursor_saved(), saved);
    assert!(fixture.read(&saved, fixture.selection()).await.is_empty());
    assert_eq!(fixture.read(&first, fixture.selection()).await.len(), 1);
    let second = fixture.admit(&[(2, 20)]).await;
    fixture.saved(&second).await;
    let third = fixture.admit(&[(3, 30)]).await;
    let rows = fixture.read(&third, fixture.selection()).await;
    assert_eq!(rows.iter().map(|(_, entry)| entry.ordinal).collect::<Vec<_>>(), [1, 2, 3]);
    assert_eq!(fixture.storage.cursor_saved(), second);
    let (rows, largest): (i64, i64) = fixture
        .local
        .call(|connection| connection.query_row("SELECT sum(rows),max(rows) FROM stream_batches", [], |row| Ok((row.get(0)?, row.get(1)?))))
        .await
        .unwrap();
    assert_eq!(rows, 2);
    assert!((1..=2).contains(&largest));
    fixture.finish().await;
}

#[tokio::test]
async fn configured_time_flush_occurs_at_its_deadline() {
    let fixture = Fixture::new(1024).await;
    let before = fixture.storage.cursor_saved();
    let through = fixture.admit(&[(1, 10)]).await;
    tokio::time::pause();
    let started = fixture.local.0.telemetry_tail.lock().unwrap().entries.values().next().unwrap().0;
    let deadline = started + Duration::from_secs(60);
    let remaining = deadline.duration_since(tokio::time::Instant::now());
    tokio::time::advance(remaining - Duration::from_nanos(1)).await;
    assert_eq!(fixture.storage.cursor_saved(), before);
    tokio::time::advance(Duration::from_nanos(1)).await;
    // Real Filesystem Work Runs Under Its Normal Backend Deadline.
    tokio::time::resume();
    fixture.saved(&through).await;
    assert_eq!(fixture.read(&through, fixture.selection()).await.len(), 1);
    fixture.finish().await;
}

#[tokio::test]
async fn out_of_order_windows_include_only_matching_saved_and_pending_rows() {
    let fixture = Fixture::new(1024).await;
    fixture.admit(&[(1, 10), (2, 30)]).await;
    fixture.storage.flush().await.unwrap();
    fixture.admit(&[(3, 5), (4, 20)]).await;
    fixture.storage.flush().await.unwrap();
    let through = fixture.admit(&[(5, 25)]).await;
    let mut source = fixture.selection();
    source.source_start = Some(std::time::UNIX_EPOCH + Duration::from_secs(15));
    source.source_end = Some(std::time::UNIX_EPOCH + Duration::from_secs(25));
    let rows = fixture.read(&through, source).await;
    assert_eq!(rows.iter().map(|(_, entry)| entry.ordinal).collect::<Vec<_>>(), [4]);
    let mut source = fixture.selection();
    source.source_start = Some(std::time::UNIX_EPOCH + Duration::from_secs(25));
    assert_eq!(fixture.read(&through, source).await.iter().map(|(_, entry)| entry.ordinal).collect::<Vec<_>>(), [2, 5]);
    let numeric = fixture
        .local
        .telemetry_numeric(
            through.clone(),
            fixture.selection(),
            ScanPage {
                after: 0,
                rows: NonZeroUsize::MAX,
                bytes: NonZeroUsize::MAX,
            },
            vec!["value".into()],
            Default::default(),
        )
        .await
        .unwrap();
    assert_eq!(numeric.iter().map(|(_, row)| row.values[0]).collect::<Vec<_>>(), [Some(1.0), Some(2.0), Some(3.0), Some(4.0), Some(5.0)]);
    let contract = crate::telemetry::TelemetryContract::declared("arrow.fixture", "Arrow").typed::<Reading>().column(
        crate::telemetry::TelemetryColumn::declared("value", "Value", crate::ValueType::Number, crate::telemetry::TelemetryRole::Value),
    );
    let saved = fixture.storage.cursor_saved();
    let outputs = crate::telemetry_runtime::output::project(
        &fixture.storage,
        &fixture.local,
        &RecordScope {
            test_execution_id: Some(fixture.execution.clone()),
            ..Default::default()
        },
        &[contract],
    )
    .await
    .unwrap();
    for (suffix, expected) in [("count", 5.0), ("mean", 3.0), ("sum", 15.0)] {
        let output = outputs.iter().find(|output| output.key.ends_with(&format!(".{suffix}"))).unwrap();
        assert_eq!(output.value, Cell::Number(expected));
    }
    assert_eq!(fixture.storage.cursor_saved(), saved);
    fixture.finish().await;
}

#[tokio::test]
async fn predecessor_selection_merges_the_tail_before_its_boundary() {
    let fixture = Fixture::new(1024).await;
    fixture.admit(&[(1, 10), (2, 30)]).await;
    fixture.storage.flush().await.unwrap();
    let through = fixture.admit(&[(3, 20)]).await;
    let selected = crate::evidence_runtime::TelemetrySelection {
        scope: crate::evidence_runtime::ScopeSelection {
            test_execution_id: Some(fixture.execution.clone()),
            ..Default::default()
        },
        ..Default::default()
    };
    let (_, rows) = fixture
        .local
        .telemetry_predecessor_page(
            through,
            fixture.codec.schema().clone(),
            selected,
            Vec::new(),
            std::time::UNIX_EPOCH + Duration::from_secs(25),
            String::new(),
        )
        .await
        .unwrap();
    assert_eq!(rows.iter().map(|(_, entry)| entry.ordinal).collect::<Vec<_>>(), [3]);
    fixture.finish().await;
}

#[tokio::test]
async fn catalog_failure_never_acknowledges_flushed_bytes_and_retry_is_exact() {
    let fixture = Fixture::new(1024).await;
    fixture
        .local
        .call(|connection| {
            connection.execute_batch("CREATE TRIGGER fail_batch BEFORE INSERT ON stream_batches BEGIN SELECT RAISE(ABORT,'injected'); END")
        })
        .await
        .unwrap();
    let before = fixture.storage.cursor_saved();
    let through = fixture.admit(&[(1, 10)]).await;
    assert!(fixture.storage.flush_until(&through).await.is_err());
    assert_eq!(fixture.storage.cursor_saved(), before);
    assert_eq!(fixture.local.data_saved().await.unwrap().sequence, before.sequence);
    assert!(
        fixture
            .local
            .acknowledge_sample(
                fixture.execution.to_string(),
                "arrow.fixture".into(),
                TelemetrySampleId::from_seed("arrow.sample.1").to_string(),
                through.sequence
            )
            .await
            .is_err()
    );
    fixture.local.call(|connection| connection.execute_batch("DROP TRIGGER fail_batch")).await.unwrap();
    fixture.storage.retry_transient_persistence();
    fixture.storage.flush_until(&through).await.unwrap();
    assert_eq!(fixture.read(&through, fixture.selection()).await.len(), 1);
    assert_eq!(
        fixture
            .local
            .call(|connection| connection.query_row("SELECT count(*) FROM stream_batches", [], |row| row.get::<_, i64>(0)))
            .await
            .unwrap(),
        1
    );
    fixture.finish().await;
}

#[tokio::test]
async fn window_index_does_not_read_a_nonoverlapping_batch() {
    use std::io::{Seek, SeekFrom, Write};
    let fixture = Fixture::new(1024).await;
    fixture.admit(&[(1, 10), (2, 20)]).await;
    fixture.storage.flush().await.unwrap();
    let through = fixture.admit(&[(3, 100), (4, 110)]).await;
    fixture.storage.flush().await.unwrap();
    let (id, body): (String, i64) = fixture
        .local
        .call(|connection| {
            connection.query_row("SELECT file_id,offset FROM stream_batches WHERE batch=0", [], |row| Ok((row.get(0)?, row.get(1)?)))
        })
        .await
        .unwrap();
    let mut file = std::fs::OpenOptions::new().write(true).open(fixture.root.path().join(format!("telemetry/{id}.arrow"))).unwrap();
    file.seek(SeekFrom::Start(body as u64)).unwrap();
    file.write_all(&[0; 16]).unwrap();
    drop(file);
    let mut source = fixture.selection();
    source.source_start = Some(std::time::UNIX_EPOCH + Duration::from_secs(90));
    source.source_end = Some(std::time::UNIX_EPOCH + Duration::from_secs(120));
    let rows = fixture.read(&through, source).await;
    assert_eq!(rows.iter().map(|(_, entry)| entry.ordinal).collect::<Vec<_>>(), [3, 4]);
    assert!(
        fixture
            .local
            .telemetry_read(
                through,
                fixture.selection(),
                ScanPage {
                    after: 0,
                    rows: NonZeroUsize::MAX,
                    bytes: NonZeroUsize::MAX
                }
            )
            .await
            .is_err()
    );
    fixture.finish().await;
}

#[tokio::test]
async fn sync_failure_never_commits_catalog_or_saved_progress() {
    let fixture = Fixture::new(1024).await;
    fixture.local.0.telemetry.lock().unwrap().fail_next_sync();
    let before = fixture.storage.cursor_saved();
    let through = fixture.admit(&[(1, 10)]).await;
    assert!(fixture.storage.flush_until(&through).await.is_err());
    assert_eq!(fixture.storage.cursor_saved(), before);
    assert_eq!(fixture.local.data_saved().await.unwrap().sequence, before.sequence);
    assert_eq!(
        fixture
            .local
            .call(|connection| connection.query_row("SELECT count(*) FROM stream_batches", [], |row| row.get::<_, i64>(0)))
            .await
            .unwrap(),
        0
    );
    fixture.storage.retry_transient_persistence();
    fixture.storage.flush_until(&through).await.unwrap();
    assert_eq!(fixture.read(&through, fixture.selection()).await.len(), 1);
    fixture.finish().await;
}

#[tokio::test]
async fn reopen_recovers_the_cataloged_footer_without_rewriting_batches() {
    use std::io::Write;
    let fixture = Fixture::new(1024).await;
    let through = fixture.admit(&[(1, 10), (2, 20), (3, 30)]).await;
    fixture.storage.flush().await.unwrap();
    let selection = fixture.selection();
    let (id, end): (String, i64) = fixture
        .local
        .call(|connection| connection.query_row("SELECT file_id,durable_end FROM streams", [], |row| Ok((row.get(0)?, row.get(1)?))))
        .await
        .unwrap();
    let path = fixture.root.path().join(format!("telemetry/{id}.arrow"));
    let before = std::fs::read(&path).unwrap();
    let root = fixture.stop().await;
    let connection = rusqlite::Connection::open(root.path().join("local.sqlite")).unwrap();
    connection.execute("UPDATE streams SET finalized=0", []).unwrap();
    drop(connection);
    let mut file = std::fs::OpenOptions::new().write(true).open(&path).unwrap();
    file.set_len(end as u64).unwrap();
    use std::io::{Seek, SeekFrom};
    file.seek(SeekFrom::End(0)).unwrap();
    file.write_all(b"partial footer").unwrap();
    drop(file);
    let backend = crate::storage::Backend::primary(root.path().to_owned()).await.unwrap();
    assert_eq!(backend.open().await.unwrap(), through);
    let local = backend.local_state();
    let rows = local
        .telemetry_read(
            through,
            selection,
            ScanPage {
                after: 0,
                rows: NonZeroUsize::MAX,
                bytes: NonZeroUsize::MAX,
            },
        )
        .await
        .unwrap();
    assert_eq!(rows.len(), 3);
    let after = std::fs::read(&path).unwrap();
    assert_eq!(&after[..end as usize], &before[..end as usize]);
    let physical = arrow_ipc::reader::FileReader::try_new(std::fs::File::open(&path).unwrap(), None).unwrap();
    assert_eq!(physical.num_batches(), 1);
    drop(physical);
    let hash: Vec<u8> = local
        .call(|connection| connection.query_row("SELECT sha256 FROM files WHERE kind='telemetry'", [], |row| row.get(0)))
        .await
        .unwrap();
    use sha2::Digest as _;
    assert_eq!(hash.as_slice(), sha2::Sha256::digest(&after).as_slice());
    backend.close().await.unwrap();
    drop(local);
    drop(backend);
    root.close().unwrap();
}

#[tokio::test]
async fn reopened_execution_appends_to_the_same_file_until_explicitly_sealed() {
    let fixture = Fixture::new(1024).await;
    fixture.admit(&[(1, 10)]).await;
    let root = fixture.stop().await;
    let fixture = Fixture::with_root(root, 1024).await;
    fixture
        .local
        .call(|connection| {
            connection.execute_batch("CREATE TRIGGER fail_batch BEFORE INSERT ON stream_batches BEGIN SELECT RAISE(ABORT,'injected'); END")
        })
        .await
        .unwrap();
    let through = fixture.admit(&[(2, 20)]).await;
    assert!(fixture.storage.flush().await.is_err());
    let (path, size, digest): (String, i64, Vec<u8>) = fixture
        .local
        .call(|connection| {
            connection.query_row("SELECT path,size_bytes,sha256 FROM files WHERE kind='telemetry'", [], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            })
        })
        .await
        .unwrap();
    let bytes = std::fs::read(fixture.root.path().join(path)).unwrap();
    use sha2::Digest as _;
    assert_eq!(bytes.len() as i64, size);
    assert_eq!(sha2::Sha256::digest(&bytes).as_slice(), digest);
    fixture.local.call(|connection| connection.execute_batch("DROP TRIGGER fail_batch")).await.unwrap();
    fixture.storage.retry_transient_persistence();
    fixture.storage.flush().await.unwrap();
    assert_eq!(fixture.read(&through, fixture.selection()).await.iter().map(|(_, row)| row.ordinal).collect::<Vec<_>>(), [1, 2]);
    fixture.local.finalize_telemetry(fixture.execution.to_string()).await.unwrap();
    let (input, _) = fixture.row(3, 30);
    let row = StoredRow {
        stream: input.stream,
        sequence: input.sequence,
        record: harmos::storage::StoredRecord {
            position: StorageCursor {
                history: through.history,
                sequence: through.sequence + 1,
            },
            schema: input.record.schema,
            scope: input.record.scope,
            item: input.record.item,
            source: input.record.source,
            source_time: input.record.source_time,
            payload: input.record.payload,
        },
    };
    assert!(fixture.local.arrow_write(StorageBatch::new(vec![row])).await.is_err());
    let (files, rows): (i64, i64) = fixture
        .local
        .call(|connection| connection.query_row("SELECT count(*),sum(rows) FROM streams", [], |row| Ok((row.get(0)?, row.get(1)?))))
        .await
        .unwrap();
    assert_eq!((files, rows), (1, 2));
    fixture.finish().await;
}

#[tokio::test]
async fn incompatible_catalog_is_refused_without_modifying_existing_data() {
    let fixture = Fixture::new(1024).await;
    fixture.admit(&[(1, 10)]).await;
    let root = fixture.stop().await;
    let path = root.path().join("local.sqlite");
    let connection = rusqlite::Connection::open(&path).unwrap();
    connection.execute_batch("PRAGMA foreign_keys=OFF; DROP TABLE stream_batches;").unwrap();
    drop(connection);
    let before = std::fs::read(&path).unwrap();
    assert!(crate::storage::Backend::primary(root.path().to_owned()).await.is_err());
    assert_eq!(std::fs::read(&path).unwrap(), before);
    root.close().unwrap();
}

#[tokio::test]
async fn invalid_flush_intervals_are_rejected_before_creating_storage() {
    let root = tempfile::tempdir().unwrap();
    for interval in [Duration::ZERO, Duration::MAX] {
        let settings = crate::storage::sqlite::Settings {
            telemetry_interval: interval,
            ..crate::config::local_storage_defaults()
        };
        assert!(State::open_with(root.path(), settings).await.is_err());
        assert!(!root.path().join("local.sqlite").exists());
    }
}

#[tokio::test]
async fn close_reports_a_failed_flush_and_keeps_the_store_available_for_repair() {
    let fixture = Fixture::new(1).await;
    fixture
        .local
        .call(|connection| {
            connection.execute_batch("CREATE TRIGGER fail_batch BEFORE INSERT ON stream_batches BEGIN SELECT RAISE(ABORT,'injected'); END")
        })
        .await
        .unwrap();
    let before = fixture.storage.cursor_saved();
    let through = fixture.admit(&[(1, 10)]).await;
    assert!(fixture.local.close().await.is_err());
    assert_eq!(fixture.storage.cursor_saved(), before);
    assert_eq!(fixture.local.data_saved().await.unwrap().sequence, before.sequence);
    fixture.local.call(|connection| connection.execute_batch("DROP TRIGGER fail_batch")).await.unwrap();
    fixture.storage.retry_transient_persistence();
    fixture.storage.flush_until(&through).await.unwrap();
    fixture.finish().await;
}

#[tokio::test]
async fn execution_and_session_streams_have_distinct_files_and_never_become_artifacts() {
    let fixture = Fixture::new(1024).await;
    fixture.admit(&[(1, 10)]).await;
    let mut rows = Vec::new();
    let mut entries = Vec::new();
    for index in [2, 3] {
        let (mut input, mut entry) = fixture.row(index, 10);
        entry.epoch = SessionId::from_seed(format!("session.{index}"));
        entry.sample.scope = None;
        input.record.scope = None;
        input.record.source = Some(harmos::Identifier::new(entry.epoch.as_str()).unwrap());
        input.record.payload = fixture.codec.encode(&entry, usize::MAX).unwrap();
        rows.push(input);
        entries.push(entry);
    }
    let through = fixture.storage.write_stream(rows).await.unwrap();
    fixture.local.stage_telemetry(fixture.storage.clone(), through.clone(), entries).await.unwrap();
    fixture.storage.flush().await.unwrap();
    let (files,artifacts,sessions):(i64,i64,i64) = fixture.local.call(|connection|connection.query_row(
        "SELECT (SELECT count(*) FROM files WHERE kind='telemetry'),(SELECT count(*) FROM artifacts),(SELECT count(*) FROM streams WHERE execution IS NULL)",
        [],|row|Ok((row.get(0)?,row.get(1)?,row.get(2)?)))).await.unwrap();
    assert_eq!((files, artifacts, sessions), (3, 0, 2));
    let mut selection = fixture.selection();
    selection.scope = None;
    assert_eq!(fixture.read(&through, selection).await.len(), 3);
    fixture.finish().await;
}
