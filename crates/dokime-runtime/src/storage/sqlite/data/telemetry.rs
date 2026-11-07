/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Arrow Telemetry Catalog and Native Reads

/* === Modules === */

pub(crate) mod aggregates;
mod catalog;
mod flush;
mod inventory;
pub(crate) mod previews;
pub(crate) mod query;
mod reader;

#[cfg(feature = "native")]
pub(crate) mod report;
mod writer;

/* === Re-Exports === */

pub(super) use catalog::SCHEMA;
pub(super) use catalog::file as catalog_file;
pub(crate) use flush::Tail;
pub(crate) use inventory::StreamFile;
pub(in crate::storage::sqlite) use reader::CachedBatch;
pub(crate) use reader::NumericRow;
pub(crate) use writer::Store;

/* === Imports === */

// Standard Library Imports
use std::{num::NonZeroUsize, sync::Arc};

// Third Party Imports
use harmos::storage::{ScanPage, SchemaKey, StorageBatch, StorageCursor, StoredRow, query::StreamSelection};
use rusqlite::{OptionalExtension, params, types::Value};

// Local Imports
use super::{Entry, SavedData, Scan, State, position, saved, sequence};
use crate::storage::{
    Error,
    telemetry::{TelemetryEntry, TypedCodec},
};

/* === Constants === */

const SAMPLE_POSITION: &str = "
SELECT i.position FROM stream_samples i JOIN streams s ON s.file_id=i.file_id
WHERE i.sample_id=?1 AND s.stream=?2 AND i.position<=?3
AND EXISTS(SELECT 1 FROM stream_captures c
    WHERE c.execution=?4 AND c.file_id=i.file_id AND c.batch=i.batch
    AND c.first_row<=i.batch_row AND c.first_row+c.rows>i.batch_row)
ORDER BY i.position LIMIT 1";

/* === Macros === */

/* === Type Aliases === */

pub(in crate::storage::sqlite) type RegisteredContracts =
    std::sync::RwLock<std::collections::BTreeMap<String, (Arc<TypedCodec>, arrow_schema::FieldRef)>>;

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl State {
    pub(crate) async fn telemetry_numeric(
        &self, at: StorageCursor, selection: StreamSelection, page: ScanPage, keys: Vec<String>,
        scope: crate::evidence_runtime::ScopeSelection,
    ) -> Result<Vec<(u64, NumericRow)>, Error> {
        let root = self.0.readers.database.parent().ok_or(Error::Invalid("storage root is absent"))?.to_owned();
        let mut scan = selection_scan(at.sequence, &selection, page)?;
        scan = scan
            .filter("cycle", scope.test_cycle_id.as_ref().map(ToString::to_string))
            .filter("case", scope.test_case_id.as_ref().map(ToString::to_string))
            .filter("template", scope.template_id.as_ref().map(ToString::to_string))
            .filter("suite", scope.suite_id.as_ref().map(ToString::to_string));
        scan.metrics = Some(self.0.read_metrics.clone());
        let owner = self.0.clone();
        self.read(move |connection| {
            Ok((|| {
                let tail = owner.telemetry_tail.lock().unwrap().select(page.after, at.sequence, |entry| {
                    Ok(scope.matches_evidence(entry.sample.scope.as_ref(), &entry.sample.captured_scopes)
                        && reader::selected(entry, &selection)?)
                })?;
                let head = saved(connection)?;
                head.history.check(&at.history)?;
                let mut rows = reader::numeric(connection, &root, &scan, &keys, &scope)?;
                for (position, entry) in tail {
                    if position <= head.sequence {
                        continue;
                    }
                    rows.push((
                        position,
                        NumericRow {
                            timestamp: entry.sample.timestamp,
                            sequence: entry.sample.sequence,
                            values: keys
                                .iter()
                                .map(|key| entry.sample.values.get(key).and_then(dokime_domain::model::values::numeric_value))
                                .collect(),
                        },
                    ));
                }
                rows.sort_unstable_by_key(|(position, _)| *position);
                rows.truncate(page.rows.get());
                Ok(rows)
            })())
        })
        .await?
    }

    pub(crate) async fn telemetry_streams_between(
        &self, after: StorageCursor, at: StorageCursor,
    ) -> Result<std::collections::BTreeSet<String>, Error> {
        if !at.covers(&after)? {
            return Err(Error::Invalid("telemetry frontier moved backwards"));
        }
        let owner = self.0.clone();
        self.read(move |connection| {
            Ok((|| {
                let mut streams = owner.telemetry_tail.lock().unwrap().streams_between(after.sequence, at.sequence);
                saved(connection)?.history.check(&at.history)?;
                let mut query = connection.prepare(
                    "SELECT DISTINCT s.stream FROM streams s WHERE s.last_position>?1 AND
                     (s.last_position<=?2 OR EXISTS(SELECT 1 FROM stream_samples i
                      WHERE i.file_id=s.file_id AND i.position>?1 AND i.position<=?2))",
                )?;
                let names = query.query_map(params![position(after.sequence), position(at.sequence)], |row| row.get::<_, String>(0))?;
                for name in names {
                    streams.insert(name?);
                }
                Ok(streams)
            })())
        })
        .await?
    }

    pub(crate) async fn telemetry_schemas(
        &self, at: StorageCursor, streams: Vec<dokime_domain::model::identity::IdentifierPattern>,
    ) -> Result<Vec<(SchemaKey, Arc<harmos::schema::graph::Schema>)>, Error> {
        let metrics = self.0.read_metrics.clone();
        self.read(move |connection| {
            Ok((|| {
                saved(connection)?.history.check(&at.history)?;
                let mut query = connection.prepare(
                    "SELECT DISTINCT schema_key FROM storage_data_entries
                     WHERE kind='schema' AND schema_id LIKE 'dokime.telemetry.%' AND position<=?1 ORDER BY schema_key",
                )?;
                let mut keys = Vec::new();
                for key in query.query_map([position(at.sequence)], |row| row.get::<_, String>(0))? {
                    let key: SchemaKey = serde_json::from_str(&key?)?;
                    if crate::storage::telemetry::selected_schema(key.id.as_str(), &streams) {
                        let schema = catalog::composed_at(connection, &key, at.sequence)?;
                        metrics.data_scan_rows.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        super::super::DATA_SCAN_ROWS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        keys.push((key, schema.schema));
                    }
                }
                Ok(keys)
            })())
        })
        .await?
    }

    pub(crate) async fn telemetry_files(&self, execution: String, at: StorageCursor) -> Result<Vec<StreamFile>, Error> {
        let root = self.0.readers.database.parent().ok_or(Error::Invalid("storage root is absent"))?.to_owned();
        self.read(move |connection| Ok(inventory::files(connection, &root, &execution, &at))).await?
    }

    pub(crate) async fn register_telemetry_contract(
        &self, contract: dokime_domain::model::telemetry::TelemetryContract, codec: Arc<TypedCodec>,
    ) -> Result<(), Error> {
        let key = serde_json::to_string(codec.schema())?;
        let encoded = serde_json::to_string(&contract)?;
        let schema = crate::storage::ipc::contract_schema(&contract, None)?;
        let field = schema.fields()[1].clone();
        let owner = self.0.clone();
        self.call(move |connection| {
            Ok((|| {
                let previous: Option<String> = connection
                    .query_row("SELECT contract FROM telemetry_contracts WHERE schema_key=?1", [&key], |row| row.get(0))
                    .optional()?;
                if let Some(previous) = previous {
                    let previous: dokime_domain::model::telemetry::TelemetryContract = serde_json::from_str(&previous)?;
                    let same = match (&previous.storage_schema, &contract.storage_schema) {
                        (Some(previous), Some(current)) => crate::storage::telemetry::same_graph(previous, current)?,
                        (None, None) => true,
                        _ => false,
                    };
                    if !same {
                        return Err(Error::TelemetryIntegrity { stream: contract.stream.to_string() });
                    }
                }
                connection.execute(
                    "INSERT INTO telemetry_contracts VALUES (?1,?2) ON CONFLICT(schema_key) DO UPDATE SET contract=excluded.contract",
                    params![key, encoded],
                )?;
                owner.telemetry_contracts.write().unwrap().insert(key, (codec, field));
                Ok(())
            })())
        })
        .await?
    }

    pub(super) async fn arrow_write(&self, batch: StorageBatch<StoredRow>) -> Result<SavedData, Error> {
        let owner = self.0.clone();
        self.call(move |connection| {
            let waiting = std::time::Instant::now();
            let mut store = owner.telemetry.lock().unwrap();
            tracing::debug!(name: "ingest.lock.store.writer", elapsed_ns = waiting.elapsed().as_nanos() as u64);
            Ok(store.write(connection, batch, &owner.telemetry_contracts))
        })
        .await?
    }

    pub(crate) async fn finalize_telemetry(&self, execution: String) -> Result<(), Error> {
        let owner = self.0.clone();
        self.call(move |connection| Ok(owner.telemetry.lock().unwrap().finish(connection, Some(&execution)))).await?
    }

    pub(crate) async fn telemetry_read(
        &self, at: StorageCursor, selection: StreamSelection, page: ScanPage,
    ) -> Result<Vec<(u64, TelemetryEntry)>, Error> {
        self.telemetry_read_scoped(at, selection, page, Default::default()).await
    }

    pub(crate) async fn telemetry_read_scoped(
        &self, at: StorageCursor, selection: StreamSelection, page: ScanPage, scope: crate::evidence_runtime::ScopeSelection,
    ) -> Result<Vec<(u64, TelemetryEntry)>, Error> {
        let root = self.0.readers.database.parent().ok_or(Error::Invalid("storage root is absent"))?.to_owned();
        let mut scan = selection_scan(at.sequence, &selection, page)?;
        scan = scan
            .filter("cycle", scope.test_cycle_id.as_ref().map(ToString::to_string))
            .filter("case", scope.test_case_id.as_ref().map(ToString::to_string))
            .filter("template", scope.template_id.as_ref().map(ToString::to_string))
            .filter("suite", scope.suite_id.as_ref().map(ToString::to_string));
        scan.metrics = Some(self.0.read_metrics.clone());
        let owner = self.0.clone();
        self.read(move |connection| {
            Ok((|| {
                // Capture The Tail Before Establishing SQLite's Snapshot; A Concurrent Commit
                // Then Appears In The Catalog, The Tail, Or Both And Is Deduplicated By Position.
                let tail = owner.telemetry_tail.lock().unwrap().select(page.after, at.sequence, |entry| {
                    Ok(reader::selected(entry, &selection)?
                        && scope.matches_evidence(entry.sample.scope.as_ref(), &entry.sample.captured_scopes))
                })?;
                let head = saved(connection)?;
                head.history.check(&at.history)?;
                let mut rows = reader::entries(connection, &root, &scan)?;
                for (position, entry) in tail {
                    if position <= head.sequence {
                        continue;
                    }
                    rows.push((position, entry));
                }
                rows.sort_unstable_by_key(|(position, _)| *position);
                rows.truncate(page.rows.get());
                Ok(rows)
            })())
        })
        .await?
    }

    pub(super) async fn arrow_scan(&self, scan: Scan) -> Result<Vec<Entry>, Error> {
        let root = self.0.readers.database.parent().ok_or(Error::Invalid("storage root is absent"))?.to_owned();
        self.read(move |connection| Ok(reader::stored(connection, &root, &scan).map(|rows| rows.into_iter().map(Entry::Stream).collect())))
            .await?
    }

    pub(crate) async fn telemetry_page(
        &self, at: StorageCursor, after: u64, execution: Option<String>, stream: Option<String>,
    ) -> Result<Vec<(u64, TelemetryEntry)>, Error> {
        let page = ScanPage {
            after,
            rows: NonZeroUsize::new(512).unwrap(),
            bytes: NonZeroUsize::new(8 * 1024 * 1024).unwrap(),
        };
        let mut scan = Scan::new("stream", at.sequence, 0, page).filter("execution", execution.clone()).filter("stream", stream.clone());
        scan.metrics = Some(self.0.read_metrics.clone());
        let root = self.0.readers.database.parent().ok_or(Error::Invalid("storage root is absent"))?.to_owned();
        let owner = self.0.clone();
        self.read(move |connection| {
            Ok((|| {
                let tail = owner.telemetry_tail.lock().unwrap().select(after, at.sequence, |entry| {
                    Ok(execution.as_deref().is_none_or(|id| {
                        entry
                            .sample
                            .scope
                            .iter()
                            .chain(&entry.sample.captured_scopes)
                            .any(|scope| scope.test_execution_id.as_ref().is_some_and(|owner| owner.as_str() == id))
                    }) && stream.as_deref().is_none_or(|stream| entry.sample.stream.as_str() == stream))
                })?;
                let head = saved(connection)?;
                head.history.check(&at.history)?;
                let mut rows = reader::entries(connection, &root, &scan)?;
                for (position, entry) in tail {
                    if position <= head.sequence {
                        continue;
                    }
                    rows.push((position, entry));
                }
                rows.sort_unstable_by_key(|(position, _)| *position);
                rows.truncate(page.rows.get());
                Ok(rows)
            })())
        })
        .await?
    }

    pub(crate) async fn acknowledge_sample(
        &self, execution: String, stream: String, sample_id: String, through: u64,
    ) -> Result<u64, Error> {
        self.read(move |connection| {
            Ok((|| {
                let selected: Option<String> = connection
                    .prepare_cached(SAMPLE_POSITION)?
                    .query_row(params![sample_id, stream, position(through), execution], |row| row.get(0))
                    .optional()?;
                let selected = selected.ok_or(Error::Corrupt("acknowledged sample is absent from saved execution evidence"))?;
                sequence(&selected)
            })())
        })
        .await?
    }

    pub(crate) async fn latest_telemetry_time(
        &self, at: StorageCursor, schema: SchemaKey, terms: Vec<(&'static str, Value)>,
        selection: crate::evidence_runtime::TelemetrySelection, _codec: TypedCodec,
    ) -> Result<Option<std::time::SystemTime>, Error> {
        let root = self.0.readers.database.parent().ok_or(Error::Invalid("storage root is absent"))?.to_owned();
        let owner = self.0.clone();
        self.read(move |connection| {
            Ok((|| {
                let tail = owner
                    .telemetry_tail
                    .lock()
                    .unwrap()
                    .select(0, at.sequence, |entry| Ok(reader::key(entry)? == schema && reader::matches(entry, &terms, &selection)?))?;
                let head = saved(connection)?;
                head.history.check(&at.history)?;
                let context = reader::Context {
                    root: &root,
                    metrics: &owner.read_metrics,
                };
                let mut latest = reader::latest(connection, context, &at, &schema, &terms, &selection)?;
                for (position, entry) in tail {
                    if position <= head.sequence {
                        continue;
                    }
                    let time: std::time::SystemTime = entry.sample.timestamp.to_datetime()?.into();
                    latest = Some(latest.map_or(time, |latest| latest.max(time)));
                }
                Ok(latest)
            })())
        })
        .await?
    }

    pub(crate) async fn telemetry_predecessor_page(
        &self, at: StorageCursor, schema: SchemaKey, selection: crate::evidence_runtime::TelemetrySelection,
        terms: Vec<(&'static str, Value)>, start: std::time::SystemTime, after: String,
    ) -> Result<(String, Vec<(u64, TelemetryEntry)>), Error> {
        let root = self.0.readers.database.parent().ok_or(Error::Invalid("storage root is absent"))?.to_owned();
        let owner = self.0.clone();
        self.read(move |connection| {
            Ok((|| {
                let mut selected = selection.clone();
                selected.start = None;
                let tail = owner
                    .telemetry_tail
                    .lock()
                    .unwrap()
                    .select(0, at.sequence, |entry| Ok(reader::key(entry)? == schema && reader::matches(entry, &terms, &selected)?))?;
                let head = saved(connection)?;
                head.history.check(&at.history)?;
                let (_, rows) = reader::predecessors(
                    connection,
                    reader::Context {
                        root: &root,
                        metrics: &owner.read_metrics,
                    },
                    &at,
                    &schema,
                    &terms,
                    &selection,
                    (start, &after),
                )?;
                let mut groups = std::collections::BTreeMap::new();
                let pending = tail.into_iter().filter(|(position, _)| *position > head.sequence && *position <= at.sequence);
                for (position, entry) in rows.into_iter().chain(pending) {
                    if reader::key(&entry)? != schema || !reader::matches(&entry, &terms, &selected)? {
                        continue;
                    }
                    let time: std::time::SystemTime = entry.sample.timestamp.to_datetime()?.into();
                    let scope = serde_json::to_string(&entry.sample.scope.as_ref().map(crate::storage::records::ScopeRow::from))?;
                    if time >= start || scope <= after {
                        continue;
                    }
                    if groups.get(&scope).is_none_or(|(prior, prior_position, _): &(std::time::SystemTime, u64, TelemetryEntry)| {
                        (time, position) > (*prior, *prior_position)
                    }) {
                        groups.insert(scope, (time, position, entry));
                    }
                }
                let mut cursor = after;
                let rows = groups
                    .into_iter()
                    .take(256)
                    .map(|(scope, (_, position, entry))| {
                        cursor = scope;
                        (position, entry)
                    })
                    .collect();
                Ok((cursor, rows))
            })())
        })
        .await?
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn selection_scan(at: u64, selection: &StreamSelection, page: ScanPage) -> Result<Scan, Error> {
    let mut scan = Scan::new("stream", at, 0, page)
        .filter("schema_key", Some(serde_json::to_string(&selection.schema)?))
        .filter("execution", selection.scope.as_ref().map(ToString::to_string))
        .filter("item", selection.item.as_ref().map(ToString::to_string))
        .filter("stream", selection.stream.as_ref().map(ToString::to_string));
    scan.source_window = selection.source_start.zip(selection.source_end);
    scan.source_bounds = (selection.source_start, selection.source_end);
    Ok(scan)
}

/* === Tests === */

#[cfg(test)]
mod tests {
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
        local: State,
        storage: Manager,
        runtime: Runtime<App>,
        codec: Arc<telemetry::TypedCodec>,
        execution: TestExecutionId,
        root: tempfile::TempDir,
    }

    #[harmos::message]
    struct Reading {
        value: f64,
    }

    #[harmos::message]
    struct UnsignedReading {
        value: Option<u64>,
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
            sample.schema_version = self.codec.schema().version.get();
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

    #[cfg(feature = "native")]
    #[tokio::test]
    async fn time_series_views_share_field_state_and_preserve_exact_rows() {
        let fixture = Fixture::new(1024).await;
        let contract = crate::telemetry::TelemetryContract::declared("arrow.fixture", "Arrow").typed::<Reading>().column(
            crate::telemetry::TelemetryColumn::declared("value", "Value", crate::ValueType::Number, crate::telemetry::TelemetryRole::Value),
        );
        let request = crate::plot::PlotPayloadRequest {
            stream: "arrow.fixture".into(),
            y_keys: vec!["value".into()],
            scope: Some(RecordScope {
                test_execution_id: Some(fixture.execution.clone()),
                ..Default::default()
            }),
            ..Default::default()
        };
        let requests = (0..64)
            .map(|index| crate::plot::PlotPayloadRequest {
                plot_id: Some(format!("plot.{index}")),
                ..request.clone()
            })
            .collect();
        fixture
            .local
            .register_plot_previews(
                fixture.execution.to_string(),
                requests,
                vec![contract],
                crate::config::local_storage_defaults().previews.into(),
                Default::default(),
            )
            .await
            .unwrap();
        let points: Vec<_> = (1..=17).map(|index| (index, index as i64)).collect();
        let at = fixture.admit(&points).await;
        fixture.storage.flush_until(&at).await.unwrap();
        let execution = fixture.execution.to_string();
        let selected = execution.clone();
        let (plans, views, fields, shared): (i64, i64, i64, i64) = fixture
            .local
            .call(move |connection| {
                connection.query_row(
                    "SELECT (SELECT count(*) FROM report_plot_previews WHERE execution=?1),
                (SELECT count(*) FROM report_plot_bindings WHERE execution=?1),
                (SELECT count(*) FROM storage_report_previews WHERE execution=?1),
                (SELECT sum(shared) FROM report_plot_previews WHERE execution=?1)",
                    [selected],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
                )
            })
            .await
            .unwrap();
        assert_eq!((plans, views, fields, shared), (1, 64, 1, 1));
        let postings = fixture
            .local
            .read(|connection| {
                connection.query_row("SELECT count(*) FROM storage_value_postings WHERE kind='telemetry'", [], |row| row.get::<_, i64>(0))
            })
            .await
            .unwrap();
        assert_eq!(postings, 0, "plot fields must not create distinct postings");
        let checkpoints = fixture
            .local
            .read(move |connection| Ok(crate::plot::preview::index::checkpoints(connection, &execution, at.sequence)))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(checkpoints.len(), views as usize);
        let mut selected = request;
        selected.plot_id = Some("plot.31".into());
        let payload = checkpoints["plot.31"].payload(&selected, "", (None, None)).unwrap();
        assert_eq!(payload.series[0].points.len(), points.len());
        assert_eq!(
            payload.series[0].points.iter().map(|point| point.y.unwrap()).collect::<Vec<_>>(),
            points.iter().map(|(_, value)| *value as f64).collect::<Vec<_>>()
        );
        fixture.finish().await;
    }

    #[tokio::test]
    async fn distinct_fields_retain_only_registered_values_across_reopen() {
        let fixture = Fixture::new(2).await;
        fixture
            .local
            .register_distinct_fields(
                fixture.execution.to_string(),
                std::collections::BTreeMap::from([("arrow.fixture".into(), std::collections::BTreeSet::from(["value".into()]))]),
            )
            .await
            .unwrap();
        let points = [(1, 1), (2, 2), (3, 3)];
        let at = fixture.admit(&points).await;
        fixture.storage.flush_until(&at).await.unwrap();
        let root = fixture.stop().await;
        let fixture = Fixture::with_root(root, 2).await;
        let at = fixture.admit(&[(4, 4)]).await;
        fixture.storage.flush_until(&at).await.unwrap();
        let paths = fixture
            .local
            .read(|connection| {
                let mut statement = connection
                    .prepare("SELECT path,count(*) FROM storage_value_postings WHERE kind='telemetry' GROUP BY path ORDER BY path")?;
                statement.query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)))?.collect::<rusqlite::Result<Vec<_>>>()
            })
            .await
            .unwrap();
        assert_eq!(paths, vec![("[\"value\"]".into(), points.len() as i64 + 1)]);
        fixture.finish().await;
    }

    #[cfg(feature = "native")]
    #[tokio::test]
    async fn unindexed_medians_preserve_nullable_large_integers_at_saved_cuts() {
        let fixture = Fixture::new(8).await;
        let contract = crate::telemetry::TelemetryContract::declared("unsigned.fixture", "Unsigned").typed::<UnsignedReading>();
        let codec = Arc::new(telemetry::codec(&contract, &Default::default()).unwrap());
        fixture.storage.write_schema(codec.schema().clone(), codec.graph().clone()).await.unwrap();
        fixture.storage.flush().await.unwrap();
        fixture.local.register_telemetry_contract(contract, codec.clone()).await.unwrap();
        let values = [Some(9_007_199_254_740_993), None, Some(9_007_199_254_740_995), Some(u64::MAX)];
        let (rows, entries): (Vec<_>, Vec<_>) = values
            .into_iter()
            .enumerate()
            .map(|(index, value)| {
                let (mut input, mut entry) = fixture.row(index as u64, index as i64);
                entry.sample.stream = harmos::Identifier::new("unsigned.fixture").unwrap();
                entry.sample = entry.sample.typed(&UnsignedReading { value });
                entry.sample.schema_version = codec.schema().version.get();
                entry.sample.values.insert("value".into(), value.map(Cell::Unsigned).unwrap_or(Cell::Null));
                input.stream = entry.sample.stream.clone();
                input.record.schema = codec.schema().clone();
                input.record.payload = codec.encode(&entry, usize::MAX).unwrap();
                (input, entry)
            })
            .unzip();
        let at = fixture.storage.write_stream(rows).await.unwrap();
        fixture.local.stage_telemetry(fixture.storage.clone(), at.clone(), entries).await.unwrap();
        fixture.storage.flush_until(&at).await.unwrap();
        let query = crate::config::query_defaults().compose(&crate::config::storage_defaults(), &crate::config::schema_defaults()).unwrap();
        for (omitted, expected) in [(0, 9_007_199_254_740_995), (1, 9_007_199_254_740_994), (2, 9_007_199_254_740_993)] {
            let cut = StorageCursor {
                sequence: at.sequence - omitted,
                ..at.clone()
            };
            let actual = fixture
                .local
                .document_statistic(
                    fixture.execution.to_string(),
                    "unsigned.fixture".into(),
                    "value".into(),
                    cut,
                    harmos::Aggregate::Median,
                    crate::storage::sqlite::QueryWork::document(query.clone(), "fixture"),
                )
                .await
                .unwrap()
                .unwrap();
            assert_eq!(actual, Some(Cell::Unsigned(expected)));
        }
        fixture.finish().await;
    }

    #[cfg(feature = "native")]
    #[tokio::test]
    async fn percentiles_replay_beyond_retained_prefix() {
        use harmos::storage::query::{
            Accumulator,
            scalar::{Agg, ScalarValue},
        };
        let fixture = Fixture::new(65_536).await;
        let count = crate::config::local_storage_defaults().previews.exact_points.get() + 7;
        let points: Vec<_> = (0..count as u64).map(|index| (index, index as i64)).collect();
        let at = fixture.admit(&points).await;
        fixture.storage.flush_until(&at).await.unwrap();
        let query = crate::config::query_defaults().compose(&crate::config::storage_defaults(), &crate::config::schema_defaults()).unwrap();
        for omitted in [0, 3] {
            let cut = StorageCursor {
                sequence: at.sequence - omitted,
                ..at.clone()
            };
            let median = fixture
                .local
                .document_statistic(
                    fixture.execution.to_string(),
                    "arrow.fixture".into(),
                    "value".into(),
                    cut,
                    harmos::Aggregate::Median,
                    crate::storage::sqlite::QueryWork::document(query.clone(), "fixture"),
                )
                .await
                .unwrap()
                .unwrap();
            let expected = (count as u64 - omitted - 1) as f64 / 2.0;
            assert_eq!(median, Some(Cell::Number(expected)));
        }
        let evidence = fixture
            .local
            .report_evidence(vec![super::report::Request {
                execution: fixture.execution.to_string(),
                cycle: "cycle".into(),
                at,
                current: false,
                rows: 32,
                streams: Vec::new(),
                percentiles: std::collections::BTreeSet::from([("arrow.fixture".into(), "value".into(), 17)]),
            }])
            .await
            .unwrap();
        let summary = evidence[0].exact.iter().find(|summary| summary.field == "value").unwrap();
        for (name, method) in
            [("median", harmos::Aggregate::Median), ("p95", harmos::Aggregate::Percentile(95)), ("p17", harmos::Aggregate::Percentile(17))]
        {
            let mut reference = Accumulator::new(Agg::Of(method));
            for (index, _) in &points {
                reference.push(*index, &ScalarValue::Float(*index as f64)).unwrap();
            }
            let ScalarValue::Float(expected) = reference.finish(&mut || Ok(())).unwrap() else {
                panic!("numeric percentile");
            };
            let value = summary.statistics.iter().find(|(key, _)| key == name).unwrap().1;
            assert_eq!(value, expected);
        }
        fixture.finish().await;
    }

    #[tokio::test]
    async fn acknowledgement_work() {
        let fixture = Fixture::new(512).await;
        for start in (1..=14_400).step_by(512) {
            let points = (start..=(start + 511).min(14_400)).map(|index| (index, index as i64)).collect::<Vec<_>>();
            let through = fixture.admit(&points).await;
            fixture.storage.flush_until(&through).await.unwrap();
        }
        let through = fixture.storage.cursor_saved();
        let mut work = Vec::new();
        for chunks in [8, 16] {
            let before = fixture.local.read_counters().data_scan_rows;
            let started = std::time::Instant::now();
            for chunk in 1..=chunks {
                let index = chunk * 512;
                let acknowledged = fixture
                    .local
                    .acknowledge_sample(
                        fixture.execution.to_string(),
                        "arrow.fixture".into(),
                        TelemetrySampleId::from_seed(format!("arrow.sample.{index}")).to_string(),
                        through.sequence,
                    )
                    .await
                    .unwrap();
                assert_eq!(acknowledged, through.sequence - 14_400 + index);
            }
            let rows = fixture.local.read_counters().data_scan_rows - before;
            eprintln!("acknowledgement: 4h at 1Hz, {chunks} chunks, {rows} Arrow rows examined, {:?}", started.elapsed());
            work.push(rows);
        }
        assert_eq!(work, [0, 0], "identity lookup must never decode Arrow history");
        let execution = fixture.execution.to_string();
        let (scans, steps) = fixture
            .local
            .read(move |connection| {
                let mut statement = connection.prepare(SAMPLE_POSITION)?;
                for index in [512, 8192, 14_400] {
                    let sample = TelemetrySampleId::from_seed(format!("arrow.sample.{index}"));
                    let selected: String = statement
                        .query_row(params![sample.as_str(), "arrow.fixture", position(through.sequence), execution], |row| row.get(0))?;
                    assert_eq!(sequence(&selected).unwrap(), through.sequence - 14_400 + index);
                }
                Ok((statement.get_status(rusqlite::StatementStatus::FullscanStep), statement.get_status(rusqlite::StatementStatus::VmStep)))
            })
            .await
            .unwrap();
        assert_eq!(scans, 0);
        assert!(steps < 300, "identity lookup must bound SQLite work: {steps}");

        let through = fixture.storage.cursor_saved();
        let mut after = 0;
        let before = fixture.local.read_counters().data_scan_rows;
        let mut delivered = 0;
        loop {
            let mut selection = fixture.selection();
            selection.after = after;
            let entries = fixture
                .local
                .telemetry_read(
                    through.clone(),
                    selection,
                    ScanPage {
                        after,
                        rows: NonZeroUsize::new(512).unwrap(),
                        bytes: NonZeroUsize::MAX,
                    },
                )
                .await
                .unwrap();
            let Some((last_position, last)) = entries.last() else { break };
            let rows_before_ack = fixture.local.read_counters().data_scan_rows;
            after = fixture
                .local
                .acknowledge_sample(
                    fixture.execution.to_string(),
                    "arrow.fixture".into(),
                    last.sample.sample_id.to_string(),
                    through.sequence,
                )
                .await
                .unwrap();
            assert_eq!(after, *last_position);
            assert_eq!(fixture.local.read_counters().data_scan_rows, rows_before_ack);
            delivered += entries.len();
        }
        assert_eq!(delivered, 14_400);
        assert_eq!(fixture.local.read_counters().data_scan_rows - before, 14_400);
        fixture.local.finalize_telemetry(fixture.execution.to_string()).await.unwrap();
        let root = fixture.stop().await;
        let fixture = Fixture::with_root(root, 512).await;
        assert_eq!(
            fixture
                .local
                .acknowledge_sample(
                    fixture.execution.to_string(),
                    "arrow.fixture".into(),
                    TelemetrySampleId::from_seed("arrow.sample.14400").to_string(),
                    through.sequence
                )
                .await
                .unwrap(),
            through.sequence
        );
        fixture.finish().await;
    }

    #[tokio::test]
    async fn acknowledgement_scope() {
        let fixture = Fixture::new(512).await;
        let captured = TestExecutionId::from_seed("captured.execution");
        let mut rows = Vec::new();
        let mut entries = Vec::new();
        for index in 1..=3 {
            let (mut input, mut entry) = fixture.row(index, 4 - index as i64);
            if index != 2 {
                entry.sample.captured_scopes.push(RecordScope {
                    test_execution_id: Some(captured.clone()),
                    ..Default::default()
                });
            }
            if index == 3 {
                entry.sample.scope = None;
                input.record.scope = None;
            }
            input.record.payload = fixture.codec.encode(&entry, usize::MAX).unwrap();
            rows.push(input);
            entries.push(entry);
        }
        let through = fixture.storage.write_stream(rows).await.unwrap();
        fixture.local.stage_telemetry(fixture.storage.clone(), through.clone(), entries).await.unwrap();
        fixture.storage.flush_until(&through).await.unwrap();

        // Neither Identity Resolution Nor Snapshot Pinning Needs The Writer.
        let writer = fixture.local.0.live.lock().await;
        assert_eq!(fixture.local.data_saved().await.unwrap().sequence, through.sequence);
        for (execution, index, frontier, expected) in [
            (fixture.execution.to_string(), 1, through.sequence, Some(through.sequence - 2)),
            (captured.to_string(), 1, through.sequence, Some(through.sequence - 2)),
            (captured.to_string(), 3, through.sequence, Some(through.sequence)),
            (fixture.execution.to_string(), 3, through.sequence, None),
            (captured.to_string(), 2, through.sequence, None),
            (captured.to_string(), 3, through.sequence - 1, None),
            (TestExecutionId::from_seed("other.execution").to_string(), 1, through.sequence, None),
        ] {
            let result = fixture
                .local
                .acknowledge_sample(
                    execution,
                    "arrow.fixture".into(),
                    TelemetrySampleId::from_seed(format!("arrow.sample.{index}")).to_string(),
                    frontier,
                )
                .await;
            assert_eq!(result.ok(), expected);
        }
        assert!(
            fixture
                .local
                .acknowledge_sample(
                    captured.to_string(),
                    "other.stream".into(),
                    TelemetrySampleId::from_seed("arrow.sample.1").to_string(),
                    through.sequence
                )
                .await
                .is_err()
        );
        drop(writer);
        fixture.finish().await;
    }

    #[tokio::test]
    async fn read_failure_seals() {
        let fixture = Fixture::new(512).await;
        let through = fixture.admit(&[(1, 1)]).await;
        fixture.storage.flush_until(&through).await.unwrap();
        let owner = fixture.local.0.clone();
        fixture.local.call(move |connection| Ok(owner.telemetry.lock().unwrap().finish(connection, None))).await.unwrap().unwrap();
        let file: String =
            fixture.local.read(|connection| connection.query_row("SELECT file_id FROM streams", [], |row| row.get(0))).await.unwrap();
        let path = fixture.root.path().join(format!("telemetry/{file}.arrow"));
        let held = path.with_extension("held");
        std::fs::rename(&path, &held).unwrap();
        let mut reader = fixture
            .storage
            .read_stream(harmos::storage::query::StreamRead {
                at: through,
                selection: fixture.selection(),
            })
            .await
            .unwrap();
        assert!(matches!(
            reader.next().await,
            Err(harmos::storage::Error::Backend {
                kind: harmos::BackendFailure::Transient,
                ..
            })
        ));
        drop(reader);
        std::fs::rename(&held, &path).unwrap();
        assert!(fixture.storage.status().failure.is_none());
        let through = fixture.admit(&[(2, 2)]).await;
        fixture.storage.flush_until(&through).await.unwrap();
        fixture.local.finalize_telemetry(fixture.execution.to_string()).await.unwrap();
        let sealed = fixture
            .local
            .read(|connection| {
                connection.query_row("SELECT rows,sealed FROM streams", [], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, bool>(1)?)))
            })
            .await
            .unwrap();
        assert_eq!(sealed, (2, true));
        fixture.finish().await;
    }

    #[tokio::test]
    async fn refuses_unindexed_root() {
        let fixture = Fixture::new(512).await;
        let through = fixture.admit(&[(1, 1)]).await;
        fixture.storage.flush_until(&through).await.unwrap();
        let root = fixture.stop().await;
        let connection = rusqlite::Connection::open(root.path().join("local.sqlite")).unwrap();
        connection.execute_batch("DROP TABLE stream_samples").unwrap();
        drop(connection);
        assert!(matches!(State::open(root.path()).await, Err(Error::Incompatible(_))));
        let connection = rusqlite::Connection::open(root.path().join("local.sqlite")).unwrap();
        let retained: i64 = connection.query_row("SELECT rows FROM streams", [], |row| row.get(0)).unwrap();
        assert_eq!(retained, 1);
        let created: bool =
            connection.query_row("SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE name='stream_samples')", [], |row| row.get(0)).unwrap();
        assert!(!created);
    }

    #[tokio::test]
    async fn stage_saved_tail() {
        let fixture = Fixture::new(2).await;
        let second = fixture.admit(&[(1, 10), (2, 20)]).await;
        fixture.saved(&second).await;
        for task in std::mem::take(&mut *fixture.local.0.telemetry_tasks.lock().await) {
            task.await.unwrap().unwrap();
        }
        // Model A Durable Prefix Whose In-Memory Retirement Has Not Yet Run.
        {
            let mut tail = fixture.local.0.telemetry_tail.lock().unwrap();
            tail.insert(second.sequence - 1, fixture.row(1, 10).1);
            tail.insert(second.sequence, fixture.row(2, 20).1);
            tail.running = true;
        }
        let third = fixture.admit(&[(3, 30)]).await;
        assert_eq!(fixture.local.0.telemetry_tail.lock().unwrap().entries.keys().copied().collect::<Vec<_>>(), [third.sequence]);
        assert_eq!(fixture.storage.cursor_saved(), second);
        fixture.finish().await;
    }

    #[tokio::test]
    async fn configured_row_flush_preserves_saved_order_and_combines_tail() {
        let fixture = Fixture::new(2).await;
        let saved = fixture.storage.cursor_saved();
        let writer = fixture.local.0.live.lock().await;
        let local = fixture.local.clone();
        let (entered, ready) = tokio::sync::oneshot::channel();
        let (release, released) = tokio::sync::oneshot::channel();
        let held = tokio::task::spawn_blocking(move || {
            let _store = local.0.telemetry.lock().unwrap();
            entered.send(()).unwrap();
            released.blocking_recv().unwrap();
        });
        ready.await.unwrap();
        // Admission And Its Live Read Complete Even While Disk Writes Cannot Run.
        let first = fixture.admit(&[(1, 10)]).await;
        assert_eq!(fixture.storage.cursor_saved(), saved);
        assert!(fixture.read(&saved, fixture.selection()).await.is_empty());
        assert_eq!(fixture.read(&first, fixture.selection()).await.len(), 1);
        release.send(()).unwrap();
        held.await.unwrap();
        drop(writer);
        let second = fixture.admit(&[(2, 20)]).await;
        fixture.saved(&second).await;
        let third = fixture.admit(&[(3, 30)]).await;
        let rows = fixture.read(&third, fixture.selection()).await;
        assert_eq!(rows.iter().map(|(_, entry)| entry.ordinal).collect::<Vec<_>>(), [1, 2, 3]);
        assert_eq!(fixture.storage.cursor_saved(), second);
        let (rows, largest): (i64, i64) = fixture
            .local
            .call(|connection| {
                connection.query_row("SELECT sum(rows),max(rows) FROM stream_batches", [], |row| Ok((row.get(0)?, row.get(1)?)))
            })
            .await
            .unwrap();
        assert_eq!(rows, 2);
        assert!((1..=2).contains(&largest));
        fixture.finish().await;
    }

    #[tokio::test]
    async fn configured_time_flush_occurs_at_its_deadline() {
        // Advance Only The Flush Clock; Filesystem Work Keeps Its Real Backend Clock.
        let io = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();
        let fixture = io.spawn(Fixture::new(1024)).await.unwrap();
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
        io.shutdown_background();
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
        assert_eq!(
            numeric.iter().map(|(_, row)| row.values[0]).collect::<Vec<_>>(),
            [Some(1.0), Some(2.0), Some(3.0), Some(4.0), Some(5.0)]
        );
        // Reuse One Indexed Batch First As Numeric Columns, Then As Complete Envelopes.
        let page = ScanPage {
            after: numeric[1].0,
            rows: NonZeroUsize::MAX,
            bytes: NonZeroUsize::MAX,
        };
        let projected = fixture
            .local
            .telemetry_numeric(through.clone(), fixture.selection(), page, vec!["value".into()], Default::default())
            .await
            .unwrap();
        let entries = fixture.local.telemetry_read(through.clone(), fixture.selection(), page).await.unwrap();
        let expected = entries
            .iter()
            .map(|(position, entry)| {
                (
                    *position,
                    entry.sample.timestamp.clone(),
                    entry.sample.sequence,
                    entry.sample.values.get("value").and_then(dokime_domain::model::values::numeric_value),
                )
            })
            .collect::<Vec<_>>();
        let actual =
            projected.into_iter().map(|(position, row)| (position, row.timestamp, row.sequence, row.values[0])).collect::<Vec<_>>();
        assert_eq!(actual, expected);
        let contract = crate::telemetry::TelemetryContract::declared("arrow.fixture", "Arrow").typed::<Reading>().column(
            crate::telemetry::TelemetryColumn::declared("value", "Value", crate::ValueType::Number, crate::telemetry::TelemetryRole::Value),
        );
        let outputs = crate::telemetry_runtime::output::project_saved(
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
            let output = outputs.iter().find(|output| output.key().ends_with(&format!(".{suffix}"))).unwrap();
            assert_eq!(output.value, Cell::Number(expected));
        }
        // Outputs Flush The Admitted Tail And Read The Summaries Committed With Its Arrow Batches.
        assert_eq!(fixture.storage.cursor_saved(), through);
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
                connection
                    .execute_batch("CREATE TRIGGER fail_batch BEFORE INSERT ON stream_captures BEGIN SELECT RAISE(ABORT,'injected'); END")
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
                .acknowledge_sample(
                    fixture.execution.to_string(),
                    "arrow.fixture".into(),
                    TelemetrySampleId::from_seed("arrow.sample.1").to_string(),
                    through.sequence,
                )
                .await
                .unwrap(),
            through.sequence,
        );
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
                connection
                    .execute_batch("CREATE TRIGGER fail_batch BEFORE INSERT ON stream_batches BEGIN SELECT RAISE(ABORT,'injected'); END")
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
                connection
                    .execute_batch("CREATE TRIGGER fail_batch BEFORE INSERT ON stream_batches BEGIN SELECT RAISE(ABORT,'injected'); END")
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

    #[tokio::test]
    async fn query_cut_excludes_live_and_late_arrivals_across_restart() {
        use crate::storage::sqlite::snapshots::Scope;
        let fixture = Fixture::new(2).await;
        let admitted = fixture.admit(&[(1, 10), (2, 20)]).await;
        fixture.storage.flush_until(&admitted).await.unwrap();
        let cut = fixture
            .local
            .snapshot_capture(
                Scope::History,
                Some(admitted.clone()),
                None,
                dokime_domain::model::identity::wire::Timestamp { seconds: 100, nanos: 0 },
                b"{\"records\":[],\"telemetry\":[]}".to_vec(),
                None,
            )
            .await
            .unwrap()
            .unwrap();
        let later = fixture.admit(&[(3, 5), (4, 30)]).await;
        fixture.storage.flush_until(&later).await.unwrap();
        let original = fixture.read(&cut.at, fixture.selection()).await;
        assert_eq!(original.iter().map(|(_, row)| row.sample.sequence).collect::<Vec<_>>(), [1, 2]);
        assert_eq!(fixture.read(&later, fixture.selection()).await.len(), 4);
        let root = fixture.stop().await;
        let fixture = Fixture::with_root(root, 2).await;
        assert!(fixture.local.snapshot_available(cut.clone()).await.unwrap());
        assert_eq!(fixture.read(&cut.at, fixture.selection()).await.len(), 2);
        fixture.finish().await;
    }

    #[tokio::test]
    async fn stream_watermarks_cover_saved_tail_and_historical_frontiers() {
        let fixture = Fixture::new(1024).await;
        let initial = fixture.storage.cursor_saved();
        let pattern = vec![dokime_domain::model::identity::IdentifierPattern::new("arrow.fixture").unwrap()];
        let mut before_schema = initial.clone();
        before_schema.sequence = 0;
        assert!(fixture.local.telemetry_schemas(before_schema, pattern.clone()).await.unwrap().is_empty());
        assert_eq!(
            fixture.local.telemetry_schemas(initial.clone(), pattern).await.unwrap().into_iter().map(|(key, _)| key).collect::<Vec<_>>(),
            [fixture.codec.schema().clone()]
        );
        let first = fixture.admit(&[(1, 10)]).await;
        let second = fixture.admit(&[(2, 20)]).await;
        let streams = fixture.local.telemetry_streams_between(initial.clone(), first.clone()).await.unwrap();
        assert_eq!(streams, [fixture.row(1, 10).1.sample.stream.to_string()].into());
        fixture.storage.flush_until(&second).await.unwrap();
        assert_eq!(fixture.local.telemetry_streams_between(initial, first.clone()).await.unwrap(), streams);
        assert!(fixture.local.telemetry_streams_between(first.clone(), first).await.unwrap().is_empty());
        fixture.finish().await;
    }

    #[tokio::test]
    async fn native_inventory_uses_the_execution_numeric_summary() {
        let fixture = Fixture::new(1024).await;
        let at = fixture.admit(&[(1, 1), (2, 2), (3, 3)]).await;
        fixture.storage.flush_until(&at).await.unwrap();
        fixture.local.finalize_telemetry(fixture.execution.to_string()).await.unwrap();
        let files = fixture.local.telemetry_files(fixture.execution.to_string(), at).await.unwrap();
        assert_eq!(files.len(), 1);
        let summary = &files[0].summaries[0];
        assert_eq!(summary.signal, "value");
        assert_eq!((summary.count, summary.minimum, summary.maximum, summary.sum), (3, 1.0, 3.0, 6.0));
        let numeric = fixture.local.numeric_summaries(fixture.execution.to_string()).await.unwrap();
        assert_eq!(numeric.len(), 1);
        assert!(numeric[0].statistics.contains(&("sum".into(), summary.sum)));
        fixture.finish().await;
    }

    /* === Tests === */
}
