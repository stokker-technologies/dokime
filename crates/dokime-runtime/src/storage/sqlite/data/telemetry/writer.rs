/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Durable Telemetry Batches And Execution File Lifetime

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{collections::BTreeMap, path::PathBuf, sync::Arc};

// Third Party Imports
use harmos::storage::{StorageBatch, StoredRow};
use rusqlite::{Connection, OptionalExtension, params};
use sha2::{Digest, Sha256};

// Local Imports
use super::{
    super::{SavedData, position, saved},
    catalog,
};
use crate::storage::{
    Error, Result, ipc,
    telemetry::{Admitted, TypedCodec},
};
use dokime_domain::model::telemetry::TelemetryContract;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(crate) struct Store {
    pub root: PathBuf,
    pub settings: super::super::super::Settings,
    open: BTreeMap<String, Stream>,
    contracts: BTreeMap<String, (Arc<TypedCodec>, arrow_schema::FieldRef)>,
    #[cfg(test)]
    fail_sync: bool,
}

struct Stream {
    contract: TelemetryContract,
    codec: Arc<TypedCodec>,
    schema: arrow_schema::SchemaRef,
    writer: ipc::Writer,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Store {
    pub fn new(root: PathBuf, settings: super::super::super::Settings) -> Self {
        Self {
            root,
            settings,
            open: BTreeMap::new(),
            contracts: BTreeMap::new(),
            #[cfg(test)]
            fail_sync: false,
        }
    }

    pub(super) fn register(&mut self, contract: &TelemetryContract, codec: Arc<TypedCodec>) -> Result<()> {
        let schema = ipc::contract_schema(contract, None)?;
        self.contracts.insert(serde_json::to_string(codec.schema())?, (codec, schema.fields()[1].clone()));
        Ok(())
    }

    pub(super) fn prepare_tail(&self, entry: &mut crate::storage::telemetry::TelemetryEntry) -> Result<()> {
        let key = serde_json::to_string(&super::reader::key(entry)?)?;
        let (_, field) = self.contracts.get(&key).ok_or(Error::Invalid("telemetry tail contract is not registered"))?;
        ipc::normalize_values(field, &mut entry.sample.values)
    }

    #[cfg(test)]
    pub(super) fn fail_next_sync(&mut self) {
        self.fail_sync = true;
    }

    pub(super) fn write(&mut self, connection: &mut Connection, batch: StorageBatch<StoredRow>) -> Result<SavedData> {
        let rows = batch.as_slice();
        let mut head = saved(connection)?;
        let mut groups: BTreeMap<String, Vec<(u64, Admitted)>> = BTreeMap::new();
        let mut admitted = Vec::new();
        // Plot Previews Fold The Normalized Admission; Their Checkpoints Commit With The Catalog Below.
        let mut plots = crate::plot::preview::index::Batch::default();
        for row in rows {
            head.history.check(&row.record.position.history)?;
            if row.record.position.sequence <= head.sequence {
                // Retries Compare Reconstructed Canonical Frames, Never A Second Payload Store.
                let prior = super::reader::positioned(connection, &self.root, &row.record.schema, row.record.position.sequence)?
                    .ok_or(Error::Conflict("telemetry retry position is absent"))?;
                if prior != *row {
                    return Err(Error::Conflict("telemetry retry differs"));
                }
                continue;
            }
            if head.sequence.checked_add(1) != Some(row.record.position.sequence) {
                return Err(Error::Data("telemetry write has a gap"));
            }
            let id = identity(row);
            if !self.open.contains_key(&id) {
                let stream = self.create(connection, &id, row)?;
                self.open.insert(id.clone(), stream);
            }
            let stream = self.open.get(&id).expect("telemetry writer was opened");
            if stream.codec.schema() != &row.record.schema {
                return Err(Error::Conflict("stream contract changed within its execution"));
            }
            let mut entry = stream.codec.decode_admission(row)?;
            ipc::normalize_values(stream.schema.field(1), &mut entry.entry.sample.values)?;
            plots.sample(connection, row, &entry.entry.sample)?;
            groups.entry(id).or_default().push((row.record.position.sequence, entry));
            admitted.push(row);
            head.sequence = row.record.position.sequence;
        }
        let checkpoints = groups.keys().map(|id| (id.clone(), self.open[id].writer.checkpoint())).collect::<Vec<_>>();
        let result = (|| {
            let transaction = connection.transaction()?;
            for (id, entries) in groups {
                let stream = self.open.get_mut(&id).expect("telemetry writer was opened");
                let first = &entries[0].1;
                let execution = first.sample.scope.as_ref().and_then(|scope| scope.test_execution_id.as_ref()).map(ToString::to_string);
                let path = format!("telemetry/{id}.arrow");
                let start = stream.writer.checkpoint();
                let schema_key = serde_json::to_string(stream.codec.schema())?;
                transaction.execute(
                    "INSERT OR IGNORE INTO files VALUES (?1,'telemetry',?2,?3,?4,'application/vnd.apache.arrow.file')",
                    params![id, path, catalog::integer(start.end)?, stream.writer.digest().as_slice()],
                )?;
                transaction.execute(
                    "INSERT OR IGNORE INTO streams VALUES (?1,?2,?3,?4,?5,?6,0,NULL,NULL,?7,?7,?8,?9,0,0)",
                    params![
                        id,
                        execution,
                        first.epoch.as_str(),
                        first.sample.stream.as_str(),
                        schema_key,
                        serde_json::to_string(&stream.contract)?,
                        position(0),
                        catalog::integer(start.end)?,
                        start.hash
                    ],
                )?;
                let finalized: bool = transaction.query_row("SELECT finalized FROM streams WHERE file_id=?1", [&id], |row| row.get(0))?;
                if finalized {
                    return Err(Error::Conflict("telemetry file is already finalized"));
                }
                let mut stored_rows: u64 =
                    transaction.query_row("SELECT rows FROM streams WHERE file_id=?1", [&id], |row| catalog::unsigned(row, 0))?;
                let batch_number: u64 = transaction
                    .query_row("SELECT count(*) FROM stream_batches WHERE file_id=?1", [&id], |row| catalog::unsigned(row, 0))?;
                for (batch_number, chunk) in (batch_number..).zip(entries.chunks(self.settings.telemetry_rows.get())) {
                    let batch = ipc::record_batch(stream.schema.clone(), chunk)?;
                    #[cfg(test)]
                    if std::mem::take(&mut self.fail_sync) {
                        stream.writer.fail_next_sync();
                    }
                    let index = stream.writer.append(&batch)?;
                    let times = chunk
                        .iter()
                        .map(|(_, entry)| catalog::micros(entry.sample.timestamp.to_datetime()?.into()))
                        .collect::<Result<Vec<_>>>()?;
                    let first_at = *times.iter().min().ok_or(Error::Invalid("empty telemetry batch"))?;
                    let last_at = *times.iter().max().unwrap();
                    let first_position = position(chunk.first().unwrap().0);
                    let last_position = position(chunk.last().unwrap().0);
                    transaction.execute(
                        "INSERT INTO stream_batches VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",
                        params![
                            id,
                            catalog::integer(batch_number)?,
                            catalog::integer(index.offset)?,
                            index.metadata_bytes,
                            catalog::integer(index.body_bytes)?,
                            catalog::integer(stored_rows)?,
                            catalog::integer(index.rows)?,
                            first_at,
                            last_at,
                            first_position,
                            last_position,
                            index.sha256.as_slice()
                        ],
                    )?;
                    transaction.execute(
                        "UPDATE streams SET rows=rows+?2, first_at=CASE WHEN first_at IS NULL THEN ?3 ELSE min(first_at,?3) END,
                        last_at=CASE WHEN last_at IS NULL THEN ?4 ELSE max(last_at,?4) END,
                        first_position=CASE WHEN rows=0 THEN ?5 ELSE first_position END,last_position=?6 WHERE file_id=?1",
                        params![id, catalog::integer(index.rows)?, first_at, last_at, first_position, last_position],
                    )?;
                    let mut scopes: BTreeMap<String, (i64, i64)> = BTreeMap::new();
                    for ((_, entry), time) in chunk.iter().zip(times) {
                        let scope = serde_json::to_string(&entry.sample.scope.as_ref().map(crate::storage::records::ScopeRow::from))?;
                        scopes
                            .entry(scope)
                            .and_modify(|range| {
                                range.0 = range.0.min(time);
                                range.1 = range.1.max(time);
                            })
                            .or_insert((time, time));
                    }
                    for (scope, (first, last)) in scopes {
                        transaction.execute(
                            "INSERT INTO stream_batch_scopes VALUES (?1,?2,?3,?4,?5)",
                            params![id, catalog::integer(batch_number)?, scope, first, last],
                        )?;
                    }
                    let mut captures = BTreeMap::<String, Vec<(usize, usize)>>::new();
                    for (row, (_, entry)) in chunk.iter().enumerate() {
                        for scope in entry.sample.scope.iter().chain(&entry.sample.captured_scopes) {
                            if scope.test_execution_id.is_none() && scope.test_cycle_id.is_none() {
                                continue;
                            }
                            let ranges = captures.entry(serde_json::to_string(scope)?).or_default();
                            if let Some((first, count)) = ranges.last_mut()
                                && *first + *count == row
                            {
                                *count += 1;
                            } else {
                                ranges.push((row, 1));
                            }
                        }
                    }
                    for (encoded, ranges) in captures {
                        let scope: dokime_domain::model::execution::RecordScope = serde_json::from_str(&encoded)?;
                        for (first, count) in ranges {
                            transaction.execute(
                                "INSERT INTO stream_captures VALUES (?1,?2,?3,?4,?5,?6,?7)",
                                params![
                                    scope.test_execution_id.as_ref().map(|id| id.as_str()),
                                    scope.test_cycle_id.as_ref().map(|id| id.as_str()),
                                    id,
                                    catalog::integer(batch_number)?,
                                    first as i64,
                                    count as i64,
                                    encoded
                                ],
                            )?;
                        }
                    }
                    summaries(&transaction, &id, chunk)?;
                    stored_rows += index.rows;
                }
                let checkpoint = stream.writer.checkpoint();
                transaction.execute(
                    "UPDATE streams SET durable_end=?2,hash_state=?3 WHERE file_id=?1",
                    params![id, catalog::integer(checkpoint.end)?, checkpoint.hash],
                )?;
                transaction.execute(
                    "UPDATE files SET size_bytes=?2,sha256=?3 WHERE id=?1",
                    params![id, catalog::integer(checkpoint.end)?, stream.writer.digest().as_slice()],
                )?;
            }
            // Report Aggregates, Preview Envelopes And Plot Previews Share The Catalog Commit That Publishes Their Batches.
            let mut aggregates = super::aggregates::Batch::default();
            let mut previews = super::previews::Batch::new(self.settings.previews);
            for row in &admitted {
                aggregates.push(&transaction, row, &mut previews)?;
                for scope in crate::storage::telemetry::capture_scopes(&row.record.payload)? {
                    let execution = scope.test_execution_id.as_ref().ok_or(Error::Invalid("sample capture has no execution"))?;
                    let mut captured = (*row).clone();
                    captured.record.scope = Some(harmos::Identifier::new(execution.as_str())?);
                    aggregates.push(&transaction, &captured, &mut previews)?;
                }
            }
            aggregates.save(&transaction)?;
            previews.save(&transaction)?;
            plots.save(&transaction)?;
            transaction.execute("UPDATE storage_data_head SET saved=?1 WHERE singleton=1", [position(head.sequence)])?;
            super::super::super::commit(transaction)?;
            Ok(head)
        })();
        if result.is_err() {
            for (id, checkpoint) in checkpoints {
                self.open.get_mut(&id).expect("telemetry writer exists").writer.rollback(&checkpoint)?;
            }
        }
        result
    }

    fn create(&self, connection: &mut Connection, id: &str, row: &StoredRow) -> Result<Stream> {
        let schema = catalog::composed(connection, &row.record.schema)?;
        let key = serde_json::to_string(&row.record.schema)?;
        let contract: Option<String> =
            connection.query_row("SELECT contract FROM telemetry_contracts WHERE schema_key=?1", [&key], |row| row.get(0)).optional()?;
        let contract = match contract {
            Some(contract) => serde_json::from_str(&contract)?,
            None => {
                let field = schema.schema.definitions[schema.schema.root]
                    .iter()
                    .find(|field| field.name == "values")
                    .ok_or(Error::Invalid("telemetry schema has no values"))?;
                let harmos::schema::graph::Kind::Message { definition } = field.kind else {
                    return Err(Error::Invalid("telemetry values are not a message"));
                };
                let graph = payload_graph(&schema.schema, definition)?;
                let mut contract = TelemetryContract::declared(row.stream.as_str(), row.stream.as_str()).with_storage_schema(graph);
                contract.schema_version = row.record.schema.version.get();
                contract
            }
        };
        let execution = row.record.scope.as_ref().map(harmos::Identifier::as_str);
        let arrow = ipc::contract_schema(&contract, execution)?;
        std::fs::create_dir_all(self.root.join("telemetry"))?;
        let path = self.root.join(format!("telemetry/{id}.arrow"));
        let previous: Option<(u64, Vec<u8>, bool, String)> = connection
            .query_row("SELECT durable_end,hash_state,sealed,contract FROM streams WHERE file_id=?1", [id], |row| {
                Ok((catalog::unsigned(row, 0)?, row.get(1)?, row.get(2)?, row.get(3)?))
            })
            .optional()?;
        let writer = if let Some((end, hash, sealed, previous_contract)) = previous {
            if sealed {
                return Err(Error::Conflict("execution telemetry is sealed"));
            }
            if serde_json::from_str::<TelemetryContract>(&previous_contract)? != contract {
                return Err(Error::Conflict("stream contract changed within its execution"));
            }
            let indices = super::reader::indices(connection, id)?;
            let checkpoint = ipc::Checkpoint { end, hash, batches: indices.len() };
            // Commit Recovery Intent Before Removing Only The Metadata Footer.
            let transaction = connection.transaction()?;
            transaction.execute("UPDATE streams SET finalized=0 WHERE file_id=?1", [id])?;
            transaction.execute(
                "UPDATE files SET size_bytes=?2,sha256=?3 WHERE id=?1",
                params![id, catalog::integer(checkpoint.end)?, checkpoint.digest()?.as_slice()],
            )?;
            super::super::super::commit(transaction)?;
            ipc::Writer::resume(&path, arrow.clone(), &indices, &checkpoint)?
        } else {
            let cataloged: bool = connection.query_row("SELECT EXISTS(SELECT 1 FROM files WHERE id=?1)", [id], |row| row.get(0))?;
            if cataloged {
                return Err(Error::Corrupt("telemetry file has no stream catalog"));
            }
            if path.try_exists()? { ipc::Writer::recover_empty(&path, arrow.clone())? } else { ipc::Writer::create(&path, arrow.clone())? }
        };
        let codec = match self.contracts.get(&key) {
            Some((codec, _)) => codec.clone(),
            None => {
                Arc::new(TypedCodec::from_stored(row.record.schema.clone(), schema.schema, &harmos::config::SchemaSettings::default())?)
            }
        };
        Ok(Stream { contract, schema: arrow, codec, writer })
    }

    pub(crate) fn finish(&mut self, connection: &mut Connection, execution: Option<&str>) -> Result<()> {
        let ids = self
            .open
            .iter()
            .filter(|(_, stream)| execution.is_none_or(|id| stream.schema.metadata().get("execution_id").is_some_and(|value| value == id)))
            .map(|(id, _)| id.clone())
            .collect::<Vec<_>>();
        for id in ids {
            let stream = self.open.get_mut(&id).expect("telemetry writer exists");
            let (size, digest) = stream.writer.finish()?;
            let transaction = connection.transaction()?;
            transaction
                .execute("UPDATE files SET size_bytes=?2,sha256=?3 WHERE id=?1", params![id, catalog::integer(size)?, digest.as_slice()])?;
            transaction.execute("UPDATE streams SET finalized=1 WHERE file_id=?1", [&id])?;
            super::super::super::commit(transaction)?;
            // Close Before Removing Ownership; Windows Never Renames Or Deletes An Open File.
            self.open.remove(&id);
        }
        if let Some(execution) = execution {
            connection.execute("UPDATE streams SET sealed=1 WHERE execution=?1", [execution])?;
        }
        Ok(())
    }

    pub(crate) fn recover(&mut self, connection: &mut Connection) -> Result<()> {
        let ids = {
            let mut statement = connection.prepare("SELECT file_id,contract,durable_end,hash_state FROM streams WHERE finalized=0")?;
            statement
                .query_map([], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, catalog::unsigned(row, 2)?, row.get::<_, Vec<u8>>(3)?))
                })?
                .collect::<rusqlite::Result<Vec<_>>>()?
        };
        for (id, contract, end, hash) in ids {
            if self.open.contains_key(&id) {
                continue;
            }
            let contract: TelemetryContract = serde_json::from_str(&contract)?;
            let execution: Option<String> =
                connection.query_row("SELECT execution FROM streams WHERE file_id=?1", [&id], |row| row.get(0))?;
            let schema = ipc::contract_schema(&contract, execution.as_deref())?;
            let indices = super::reader::indices(connection, &id)?;
            let checkpoint = ipc::Checkpoint { end, hash, batches: indices.len() };
            let (size, hash) = ipc::recover(&self.root.join(format!("telemetry/{id}.arrow")), schema, &indices, &checkpoint)?;
            let transaction = connection.transaction()?;
            transaction
                .execute("UPDATE files SET size_bytes=?2,sha256=?3 WHERE id=?1", params![id, catalog::integer(size)?, hash.as_slice()])?;
            transaction.execute("UPDATE streams SET finalized=1 WHERE file_id=?1", [&id])?;
            super::super::super::commit(transaction)?;
        }
        Ok(())
    }
}

/* === Public Functions === */

pub(super) fn identity(row: &StoredRow) -> String {
    let owner = row
        .record
        .scope
        .as_ref()
        .map(ToString::to_string)
        .unwrap_or_else(|| format!("session:{}", row.record.source.as_ref().map(harmos::Identifier::as_str).unwrap_or_default()));
    let digest = Sha256::digest(format!("{owner}|{}", row.stream));
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

/* === Private Functions === */

fn payload_graph(graph: &harmos::schema::graph::Schema, root: usize) -> Result<harmos::schema::graph::Schema> {
    use harmos::schema::graph::{Kind, Schema};
    let mut indices = BTreeMap::from([(root, 0)]);
    let mut pending = vec![root];
    let mut definitions = Vec::new();
    let mut next = 0;
    while let Some(index) = pending.get(next).copied() {
        let mut fields = graph.definitions.get(index).ok_or(Error::Invalid("telemetry definition is absent"))?.clone();
        for field in &mut fields {
            if let Kind::Message { definition } = &mut field.kind {
                let mapped = *indices.entry(*definition).or_insert_with(|| {
                    let mapped = pending.len();
                    pending.push(*definition);
                    mapped
                });
                *definition = mapped;
            }
        }
        definitions.push(fields);
        next += 1;
    }
    Ok(Schema { root: 0, definitions })
}

fn summaries(connection: &Connection, file: &str, entries: &[(u64, Admitted)]) -> Result<()> {
    let mut summaries: BTreeMap<&str, (u64, f64, f64, f64)> = BTreeMap::new();
    for (_, entry) in entries {
        for (signal, value) in &entry.sample.values {
            let Some(value) = dokime_domain::model::values::numeric_value(value).filter(|value| value.is_finite()) else {
                continue;
            };
            let summary = summaries.entry(signal).or_insert((0, value, value, 0.0));
            summary.0 += 1;
            summary.1 = summary.1.min(value);
            summary.2 = summary.2.max(value);
            summary.3 += value;
        }
    }
    let mut statement = connection.prepare_cached(
        "INSERT INTO stream_summaries VALUES (?1,?2,?3,?4,?5,?6)
        ON CONFLICT(file_id,signal) DO UPDATE SET count=count+excluded.count,minimum=min(minimum,excluded.minimum),
        maximum=max(maximum,excluded.maximum),sum=sum+excluded.sum",
    )?;
    for (signal, (count, min, max, sum)) in summaries {
        statement.execute(params![file, signal, catalog::integer(count)?, min, max, sum])?;
    }
    Ok(())
}

/* === Tests === */
