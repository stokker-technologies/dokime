/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Arrow Telemetry Catalog And Native Reads

/* === Modules === */

pub(crate) mod aggregates;
mod catalog;
mod flush;
mod inventory;
pub(crate) mod previews;
mod reader;
#[cfg(test)]
mod tests;
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

/* === Macros === */

/* === Type Aliases === */

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
        scan.metrics = Some(self.0.read_metrics.clone());
        let owner = self.0.clone();
        self.read(move |connection| {
            Ok((|| {
                let tail = owner.telemetry_tail.lock().unwrap().entries.clone();
                let head = saved(connection)?;
                head.history.check(&at.history)?;
                let mut rows = reader::numeric(connection, &root, &scan, &keys, &scope)?;
                for (position, (_, entry)) in tail {
                    if position <= head.sequence
                        || position <= page.after
                        || position > at.sequence
                        || !scope.matches(entry.sample.scope.as_ref())
                        || !reader::selected(&entry, &selection)?
                    {
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

    pub(crate) async fn telemetry_files(&self, execution: String) -> Result<Vec<StreamFile>, Error> {
        let root = self.0.readers.database.parent().ok_or(Error::Invalid("storage root is absent"))?.to_owned();
        self.read(move |connection| Ok(inventory::files(connection, &root, &execution))).await?
    }

    pub(crate) async fn register_telemetry_contract(
        &self, contract: dokime_domain::model::telemetry::TelemetryContract, codec: Arc<TypedCodec>,
    ) -> Result<(), Error> {
        let key = serde_json::to_string(codec.schema())?;
        let encoded = serde_json::to_string(&contract)?;
        let owner = self.0.clone();
        self.call(move |connection| {
            Ok((|| {
                owner.telemetry.lock().unwrap().register(&contract, codec)?;
                connection.execute(
                    "INSERT INTO telemetry_contracts VALUES (?1,?2) ON CONFLICT(schema_key) DO UPDATE SET contract=excluded.contract",
                    params![key, encoded],
                )?;
                Ok(())
            })())
        })
        .await?
    }

    pub(super) async fn arrow_write(&self, batch: StorageBatch<StoredRow>) -> Result<SavedData, Error> {
        let owner = self.0.clone();
        self.call(move |connection| Ok(owner.telemetry.lock().unwrap().write(connection, batch))).await?
    }

    pub(crate) async fn finalize_telemetry(&self, execution: String) -> Result<(), Error> {
        let owner = self.0.clone();
        self.call(move |connection| Ok(owner.telemetry.lock().unwrap().finish(connection, Some(&execution)))).await?
    }

    pub(crate) async fn telemetry_read(
        &self, at: StorageCursor, selection: StreamSelection, page: ScanPage,
    ) -> Result<Vec<(u64, TelemetryEntry)>, Error> {
        let root = self.0.readers.database.parent().ok_or(Error::Invalid("storage root is absent"))?.to_owned();
        let mut scan = selection_scan(at.sequence, &selection, page)?;
        scan.metrics = Some(self.0.read_metrics.clone());
        let owner = self.0.clone();
        self.read(move |connection| {
            Ok((|| {
                // Capture The Tail Before Establishing SQLite's Snapshot; A Concurrent Commit
                // Then Appears In The Catalog, The Tail, Or Both And Is Deduplicated By Position.
                let tail = owner.telemetry_tail.lock().unwrap().entries.clone();
                let head = saved(connection)?;
                head.history.check(&at.history)?;
                let mut rows = reader::entries(connection, &root, &scan)?;
                for (position, (_, entry)) in tail {
                    if position <= head.sequence || position <= page.after || position > at.sequence {
                        continue;
                    }
                    if reader::selected(&entry, &selection)? {
                        rows.push((position, entry));
                    }
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
        let mut scan = Scan::new("stream", at.sequence, 0, page).filter("scope", execution.clone()).filter("stream", stream.clone());
        scan.metrics = Some(self.0.read_metrics.clone());
        let root = self.0.readers.database.parent().ok_or(Error::Invalid("storage root is absent"))?.to_owned();
        let owner = self.0.clone();
        self.read(move |connection| {
            Ok((|| {
                let tail = owner.telemetry_tail.lock().unwrap().entries.clone();
                let head = saved(connection)?;
                head.history.check(&at.history)?;
                let mut rows = reader::entries(connection, &root, &scan)?;
                for (position, (_, entry)) in tail {
                    if position <= head.sequence
                        || position <= after
                        || position > at.sequence
                        || !execution.as_deref().is_none_or(|id| {
                            entry
                                .sample
                                .scope
                                .as_ref()
                                .and_then(|scope| scope.test_execution_id.as_ref())
                                .is_some_and(|owner| owner.as_str() == id)
                        })
                        || !stream.as_deref().is_none_or(|stream| entry.sample.stream.as_str() == stream)
                    {
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
        let root = self.0.readers.database.parent().ok_or(Error::Invalid("storage root is absent"))?.to_owned();
        self.call(move |connection| Ok((|| {
            let previous: Option<String> = connection.query_row("SELECT position FROM storage_sample_receipts WHERE execution=?1 AND stream=?2 AND sample_id=?3 AND position<=?4",
                params![execution,stream,sample_id,position(through)], |row| row.get(0)).optional()?;
            if let Some(previous) = previous { return sequence(&previous); }
            let page = ScanPage { after: 0, rows: NonZeroUsize::new(1).unwrap(), bytes: NonZeroUsize::MAX };
            let scan = Scan::new("stream",through,0,page).filter("scope",Some(execution.clone())).filter("stream",Some(stream.clone())).filter("item",Some(sample_id.clone()));
            let rows = reader::entries(connection,&root,&scan)?;
            let (selected,_) = rows.first().ok_or(Error::Corrupt("acknowledged sample is absent from saved execution evidence"))?;
            let transaction = connection.transaction()?;
            transaction.execute("INSERT INTO storage_sample_receipts VALUES (?1,?2,?3,?4) ON CONFLICT(execution,stream) DO UPDATE SET sample_id=excluded.sample_id,position=excluded.position",
                params![execution,stream,sample_id,position(*selected)])?;
            super::super::commit(transaction)?;
            Ok(*selected)
        })())).await?
    }

    pub(crate) async fn latest_telemetry_time(
        &self, at: StorageCursor, schema: SchemaKey, terms: Vec<(&'static str, Value)>,
        selection: crate::evidence_runtime::TelemetrySelection, _codec: TypedCodec,
    ) -> Result<Option<std::time::SystemTime>, Error> {
        let root = self.0.readers.database.parent().ok_or(Error::Invalid("storage root is absent"))?.to_owned();
        let owner = self.0.clone();
        self.read(move |connection| {
            Ok((|| {
                let tail = owner.telemetry_tail.lock().unwrap().entries.clone();
                let head = saved(connection)?;
                head.history.check(&at.history)?;
                let context = reader::Context {
                    root: &root,
                    metrics: &owner.read_metrics,
                };
                let mut latest = reader::latest(connection, context, &at, &schema, &terms, &selection)?;
                for (position, (_, entry)) in tail {
                    if position <= head.sequence
                        || position > at.sequence
                        || reader::key(&entry)? != schema
                        || !reader::matches(&entry, &terms, &selection)?
                    {
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
                let tail = owner.telemetry_tail.lock().unwrap().entries.clone();
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
                let mut selected = selection.clone();
                selected.start = None;
                let pending = tail
                    .into_iter()
                    .filter(|(position, _)| *position > head.sequence && *position <= at.sequence)
                    .map(|(position, (_, entry))| (position, entry));
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
        .filter("scope", selection.scope.as_ref().map(ToString::to_string))
        .filter("item", selection.item.as_ref().map(ToString::to_string))
        .filter("stream", selection.stream.as_ref().map(ToString::to_string));
    scan.source_window = selection.source_start.zip(selection.source_end);
    scan.source_bounds = (selection.source_start, selection.source_end);
    Ok(scan)
}

/* === Tests === */
