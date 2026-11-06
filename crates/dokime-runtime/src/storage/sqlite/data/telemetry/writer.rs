/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Durable Telemetry Batches and Execution File Lifetime

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
use dokime_domain::model::values::Value;

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

#[derive(Default)]
struct Context<'a> {
    through: u64,
    values: BTreeMap<&'a str, &'a Value>,
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
        let _span = tracing::debug_span!("ingest.normalize.tail").entered();
        ipc::normalize_values(field, &mut entry.sample.values)
    }

    #[cfg(test)]
    pub(super) fn fail_next_sync(&mut self) {
        self.fail_sync = true;
    }

    #[tracing::instrument(level = "debug", name = "ingest.writer", skip_all)]
    pub(super) fn write(&mut self, connection: &mut Connection, batch: StorageBatch<StoredRow>) -> Result<SavedData> {
        let rows = batch.as_slice();
        let mut head = saved(connection)?;
        let mut groups: BTreeMap<String, Vec<(u64, Admitted)>> = BTreeMap::new();
        // Plot Previews Fold The Normalized Admission; Their Checkpoints Commit With The Catalog Below.
        let mut plots = crate::plot::preview::index::Batch::default();
        let decode_span = tracing::debug_span!("ingest.decode").entered();
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
            {
                let _span = tracing::debug_span!("ingest.normalize.writer").entered();
                ipc::normalize_values(stream.schema.field(1), &mut entry.entry.sample.values)?;
            }
            plots.sample(connection, row, &entry.entry.sample)?;
            groups.entry(id).or_default().push((row.record.position.sequence, entry));
            head.sequence = row.record.position.sequence;
        }
        drop(decode_span);
        let checkpoints = groups.keys().map(|id| (id.clone(), self.open[id].writer.checkpoint())).collect::<Vec<_>>();
        let result = (|| {
            let transaction = connection.transaction()?;
            let mut aggregates = super::aggregates::Batch::default();
            let mut previews = super::previews::Batch::new(self.settings.previews);
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
                let last_batch = transaction
                    .query_row("SELECT batch FROM stream_batches WHERE file_id=?1 ORDER BY batch DESC LIMIT 1", [&id], |row| {
                        catalog::unsigned(row, 0)
                    })
                    .optional()?;
                let batch_number =
                    last_batch.map_or(Ok(0), |last| last.checked_add(1).ok_or(Error::Invalid("telemetry batch number overflows")))?;
                for (batch_number, chunk) in (batch_number..).zip(entries.chunks(self.settings.telemetry_rows.get())) {
                    let arrow_span = tracing::debug_span!("ingest.arrow").entered();
                    let stamps = chunk
                        .iter()
                        .map(|(_, entry)| Ok(entry.sample.timestamp.to_datetime()?.unix_timestamp_nanos()))
                        .collect::<Result<Vec<_>>>()?;
                    let times = stamps
                        .iter()
                        .map(|stamp| {
                            i64::try_from(stamp.div_euclid(1000)).map_err(|_| Error::Invalid("telemetry timestamp exceeds Arrow range"))
                        })
                        .collect::<Result<Vec<_>>>()?;
                    let batch = ipc::record_batch_at(stream.schema.clone(), chunk, &times)?;
                    #[cfg(test)]
                    if std::mem::take(&mut self.fail_sync) {
                        stream.writer.fail_next_sync();
                    }
                    let index = stream.writer.append(&batch)?;
                    drop(arrow_span);
                    let catalog_span = tracing::debug_span!("ingest.catalog").entered();
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
                    let identity_span = tracing::debug_span!("ingest.identities").entered();
                    identities(&transaction, &id, batch_number, chunk)?;
                    drop(identity_span);
                    let mut scopes: BTreeMap<String, (i64, i64)> = BTreeMap::new();
                    let mut current: Option<(&Option<dokime_domain::model::execution::RecordScope>, i64, i64)> = None;
                    for ((_, entry), time) in chunk.iter().zip(&times) {
                        match current.as_mut() {
                            Some((scope, first, last)) if *scope == &entry.sample.scope => {
                                *first = (*first).min(*time);
                                *last = (*last).max(*time);
                            }
                            _ => {
                                if let Some((scope, first, last)) = current.take() {
                                    scope_range(&mut scopes, scope, first, last)?;
                                }
                                current = Some((&entry.sample.scope, *time, *time));
                            }
                        }
                    }
                    if let Some((scope, first, last)) = current {
                        scope_range(&mut scopes, scope, first, last)?;
                    }
                    for (scope, (first, last)) in scopes {
                        transaction.execute(
                            "INSERT INTO stream_batch_scopes VALUES (?1,?2,?3,?4,?5)",
                            params![id, catalog::integer(batch_number)?, scope, first, last],
                        )?;
                    }
                    let mut captures = BTreeMap::<String, Vec<(usize, usize)>>::new();
                    let mut previous: Option<&crate::telemetry::TelemetrySample> = None;
                    let mut encoded_scopes = Vec::new();
                    for (row, (_, entry)) in chunk.iter().enumerate() {
                        let sample = &entry.sample;
                        if previous.is_none_or(|prior| prior.scope != sample.scope || prior.captured_scopes != sample.captured_scopes) {
                            encoded_scopes.clear();
                            for scope in sample.scope.iter().chain(&sample.captured_scopes) {
                                if scope.test_execution_id.is_some() || scope.test_cycle_id.is_some() {
                                    encoded_scopes.push(serde_json::to_string(scope)?);
                                }
                            }
                            previous = Some(sample);
                        }
                        for encoded in &encoded_scopes {
                            if !captures.contains_key(encoded) {
                                captures.insert(encoded.clone(), Vec::new());
                            }
                            let ranges = captures.get_mut(encoded).expect("capture scope loaded");
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
                    drop(catalog_span);
                    let mut contexts = BTreeMap::new();
                    for ((position, entry), time) in chunk.iter().zip(&stamps) {
                        let owners: std::collections::BTreeSet<_> = entry
                            .sample
                            .scope
                            .iter()
                            .chain(&entry.sample.captured_scopes)
                            .filter_map(|scope| scope.test_execution_id.as_ref())
                            .collect();
                        for execution in owners {
                            let context = contexts.entry(execution).or_insert_with(Context::default);
                            context.through = *position;
                            for (field, value) in &entry.sample.values {
                                if !matches!(value, Value::Null) {
                                    context.values.insert(field.as_str(), value);
                                }
                            }
                            let input = super::previews::Input {
                                execution: execution.as_str(),
                                stream: entry.sample.stream.as_str(),
                                position: *position,
                                time: *time,
                            };
                            aggregates.push(&transaction, input, &entry.sample, &mut previews)?;
                            let timestamp = Value::Datetime(
                                time::OffsetDateTime::from_unix_timestamp_nanos(*time)
                                    .map_err(|_| Error::Invalid("telemetry timestamp is outside supported range"))?,
                            );
                            let fields = entry
                                .sample
                                .values
                                .iter()
                                .map(|(name, value)| (name.as_str(), value))
                                .chain((!entry.sample.values.contains_key("timestamp")).then_some(("timestamp", &timestamp)));
                            let scope = entry
                                .sample
                                .captured_scopes
                                .iter()
                                .find(|scope| scope.test_execution_id.as_ref() == Some(execution))
                                .or(entry.sample.scope.as_ref())
                                .ok_or(Error::Invalid("captured sample scope is absent"))?;
                            super::super::distinct::insert(
                                &transaction,
                                super::super::distinct::Row {
                                    kind: "telemetry",
                                    source: entry.sample.stream.as_str(),
                                    position: *position,
                                    file: Some(&id),
                                    record: false,
                                    scope,
                                },
                                fields,
                            )?;
                        }
                    }
                    for (execution, context) in contexts {
                        previews.context(
                            &transaction,
                            execution.as_str(),
                            context.through,
                            first.sample.stream.as_str(),
                            context.values,
                        )?;
                    }
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
            aggregates.save(&transaction)?;
            let preview_span = tracing::debug_span!("ingest.fields.save").entered();
            previews.save(&transaction)?;
            drop(preview_span);
            let plot_span = tracing::debug_span!("ingest.plot.save").entered();
            plots.save(&transaction)?;
            drop(plot_span);
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

fn identities(connection: &Connection, file: &str, batch: u64, entries: &[(u64, Admitted)]) -> Result<()> {
    use rusqlite::types::Value;
    for (offset, rows) in entries.chunks(128).enumerate() {
        let mut sql = String::from("INSERT INTO stream_samples VALUES ");
        let mut values = vec![Value::Text(file.into()), Value::Integer(catalog::integer(batch)?)];
        for (row, (positioned, entry)) in rows.iter().enumerate() {
            if row > 0 {
                sql.push(',');
            }
            let parameter = 3 + row * 3;
            sql.push_str(&format!("(?{parameter},?{},?1,?2,?{})", parameter + 1, parameter + 2));
            values.push(Value::Text(entry.sample.sample_id.to_string()));
            values.push(Value::Text(position(*positioned)));
            values.push(Value::Integer((offset * 128 + row) as i64));
        }
        connection.prepare_cached(&sql)?.execute(rusqlite::params_from_iter(values))?;
    }
    Ok(())
}

fn scope_range(
    scopes: &mut BTreeMap<String, (i64, i64)>, scope: &Option<dokime_domain::model::execution::RecordScope>, first: i64, last: i64,
) -> Result<()> {
    let encoded = serde_json::to_string(&scope.as_ref().map(crate::storage::records::ScopeRow::from))?;
    scopes
        .entry(encoded)
        .and_modify(|range| {
            range.0 = range.0.min(first);
            range.1 = range.1.max(last);
        })
        .or_insert((first, last));
    Ok(())
}

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

/* === Tests === */
