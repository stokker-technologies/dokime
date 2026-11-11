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
pub(in crate::storage::sqlite) mod writer;

/* === Re-Exports === */

pub(super) use catalog::SCHEMA;
pub(super) use catalog::file as catalog_file;
pub(crate) use flush::Tail;
pub use inventory::StreamFile;
pub(in crate::storage::sqlite) use reader::CachedBatch;
pub use reader::NumericRow;
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

pub(in crate::storage::sqlite) const SAMPLE_POSITION: &str = "
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
        let owner = self.0.owner.clone();
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
        let owner = self.0.owner.clone();
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

    #[tracing::instrument(level = "debug", name = "plot.schemas", skip_all)]
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
        let owner = self.0.owner.clone();
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

    pub(in crate::storage::sqlite) async fn arrow_write(&self, batch: StorageBatch<StoredRow>) -> Result<SavedData, Error> {
        let bytes = batch
            .as_slice()
            .iter()
            .try_fold(0usize, |total, row| super::entry::stream_length(row, usize::MAX).map(|bytes| total.saturating_add(bytes)))?;
        let prepare_owner = self.0.owner.clone();
        let prepare_batch = batch.clone();
        self.call(move |connection| {
            Ok(prepare_owner.telemetry.lock().unwrap().prepare(connection, prepare_batch.as_slice(), &prepare_owner.telemetry_contracts))
        })
        .await??;
        let owner = self.0.owner.clone();
        self.grouped(batch.len(), bytes, move |connection| {
            let mut store = owner.telemetry.lock().unwrap();
            Ok(store.write(connection, batch, &owner.telemetry_contracts))
        })
        .await?
    }

    pub(crate) async fn finalize_telemetry(&self, execution: String) -> Result<(), Error> {
        let owner = self.0.owner.clone();
        self.grouped(1, execution.len(), move |connection| Ok(owner.telemetry.lock().unwrap().finish(connection, Some(&execution)))).await?
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
        let owner = self.0.owner.clone();
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
        let owner = self.0.owner.clone();
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
        let owner = self.0.owner.clone();
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
        let owner = self.0.owner.clone();
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
