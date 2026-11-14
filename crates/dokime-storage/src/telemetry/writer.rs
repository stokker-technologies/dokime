/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Buffered Durable Arrow Writes

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{collections::BTreeMap, sync::Arc};

// Third Party Imports
use prost::Message;
use prost_reflect::DescriptorPool;
use rusqlite::{Connection, params};
use tokio::time::Instant;

// Local Imports
use super::{Pending, Telemetry, catalog, codec::Codec};
use crate::{Error, Execution, Result, Sample, Store, ipc};
use harmos_protocol::v1::Catalog;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Telemetry {
    pub(crate) fn codec(&mut self, connection: &Connection, execution: &str, stream: &str) -> Result<Arc<Codec>> {
        let key = (execution.to_owned(), stream.to_owned());
        if let Some(codec) = self.codecs.get(&key) {
            return Ok(codec.clone());
        }
        let bytes: Vec<u8> = connection.query_row("SELECT payload FROM executions WHERE id=?1", [execution], |row| row.get(0))?;
        let definition = Execution::decode(bytes.as_slice())?;
        let frozen = Catalog::decode(definition.catalog.as_slice())?;
        let declaration =
            frozen.streams.iter().find(|entry| entry.id == stream).ok_or(Error::Data("declare the stream in the frozen catalog"))?;
        let pool = DescriptorPool::from_file_descriptor_set(frozen.types.ok_or(Error::Data("include frozen stream descriptors"))?)
            .map_err(|_| Error::Data("include valid frozen stream descriptors"))?;
        let descriptor = pool.get_message_by_name(&declaration.row).ok_or(Error::Data("include the frozen row descriptor"))?;
        let codec = Arc::new(Codec::new(descriptor)?);
        self.codecs.insert(key, codec.clone());
        Ok(codec)
    }
}
impl Store {
    pub fn sample_batch(&self, execution: &str, samples: &[Sample]) -> Result<Vec<u64>> {
        if samples.is_empty() {
            return Ok(Vec::new());
        }
        let connection = self.lock()?;
        crate::store::require_open(&connection, execution)?;
        let mut telemetry = self.telemetry.lock().map_err(|_| Error::Data("restart after the telemetry writer lock failed"))?;
        if telemetry.closed {
            return Err(Error::Data("open a storage session before publishing samples"));
        }
        let mut prepared = BTreeMap::<String, (Arc<Codec>, Vec<Sample>)>::new();
        let mut position = crate::store::next_position(&connection, execution)?;
        let mut positions = Vec::with_capacity(samples.len());
        for sample in samples {
            let closed: i32 = connection.query_row(
                "SELECT closed FROM streams WHERE execution=?1 AND stream=?2",
                params![execution, sample.stream],
                |row| row.get(0),
            )?;
            if closed != 0 {
                return Err(Error::Data("open the execution stream before publishing samples"));
            }
            let codec = telemetry.codec(&connection, execution, &sample.stream)?;
            let mut sample = sample.clone();
            sample.position = position;
            // Validate Every Row Before Admitting Any Part Of The Call.
            prepared.entry(sample.stream.clone()).or_insert_with(|| (codec, Vec::new())).1.push(sample);
            positions.push(position);
            position = position
                .checked_add(1)
                .filter(|p| *p <= i64::MAX as u64)
                .ok_or(Error::Data("start a fresh execution before positions overflow"))?;
        }
        for (codec, rows) in prepared.values() {
            codec.batch(rows)?;
        }
        connection.execute(
            "INSERT INTO temp.accepted VALUES(?1,?2) ON CONFLICT(execution) DO UPDATE SET position=excluded.position",
            params![execution, position as i64 - 1],
        )?;
        for (stream, (codec, rows)) in prepared {
            let pending = telemetry.pending.entry((execution.into(), stream)).or_insert_with(|| Pending {
                codec,
                started: Instant::now(),
                samples: Vec::new(),
            });
            pending.samples.extend(rows);
        }
        let pressure =
            telemetry.pending.values().map(|pending| pending.samples.iter().map(|row| row.row.len() + 64).sum::<usize>()).sum::<usize>()
                >= telemetry.settings.pending_bytes.get();
        let full = telemetry.pending.values().any(|pending| pending.samples.len() >= telemetry.settings.flush_rows.get());
        drop(telemetry);
        drop(connection);
        if pressure || full {
            self.flush_samples()?;
        }
        self.start_sample_flush()?;
        self.changed.send_modify(|revision| *revision += 1);
        Ok(positions)
    }
    fn start_sample_flush(&self) -> Result<()> {
        let Ok(runtime) = tokio::runtime::Handle::try_current() else { return Ok(()) };
        let mut telemetry = self.telemetry.lock().map_err(|_| Error::Data("restart after the telemetry writer lock failed"))?;
        if telemetry.closed || telemetry.running || telemetry.pending.is_empty() {
            return Ok(());
        }
        telemetry.running = true;
        let store = self.clone();
        let worker=runtime.spawn(async move {
            let mut changes=store.changed.subscribe();
            loop {
                let deadline = {
                    let mut telemetry = match store.telemetry.lock() {
                        Ok(t) => t,
                        Err(_) => return,
                    };
                    if telemetry.closed{telemetry.running=false;return;}
                    match telemetry.pending.values().map(|pending| pending.started + telemetry.settings.flush_interval).min() {
                        Some(deadline) => deadline,
                        None => {
                            telemetry.running = false;
                            return;
                        }
                    }
                };
                tokio::select!{biased; _=tokio::time::sleep_until(deadline)=>{}, result=changes.changed()=>{if result.is_err(){return;}continue;}}
                let worker = store.clone();
                let result = tokio::task::spawn_blocking(move || worker.flush_samples()).await;
                if !matches!(result, Ok(Ok(()))) {
                    tracing::error!(?result, "Flush Arrow Samples Failed; Saved Position Was Not Acknowledged");
                    if let Ok(mut telemetry) = store.telemetry.lock() {
                        telemetry.running = false;
                    }
                    return;
                }
            }
        });
        telemetry.workers.retain(|worker| !worker.is_finished());
        telemetry.workers.push(worker);
        Ok(())
    }
    pub async fn shutdown_samples(&self) -> Result<()> {
        let (mut result, workers) = {
            let mut connection = self.lock()?;
            let result = self.flush_samples_locked(&mut connection);
            let mut telemetry = self.telemetry.lock().map_err(|_| Error::Data("restart after the telemetry shutdown lock failed"))?;
            telemetry.closed = true;
            telemetry.running = false;
            (result, std::mem::take(&mut telemetry.workers))
        };
        self.changed.send_modify(|revision| *revision += 1);
        for worker in workers {
            if let Err(error) = worker.await {
                tracing::error!(%error,"Join Arrow Flush Worker Failed");
                if result.is_ok() {
                    result = Err(error.into());
                }
            }
        }
        result
    }
    pub fn flush_samples(&self) -> Result<()> {
        let mut connection = self.lock()?;
        self.flush_samples_locked(&mut connection)
    }
    pub(crate) fn flush_samples_locked(&self, connection: &mut Connection) -> Result<()> {
        let mut telemetry = self.telemetry.lock().map_err(|_| Error::Data("restart after the telemetry writer lock failed"))?;
        if telemetry.closed && !telemetry.pending.is_empty() {
            return Err(Error::Data("reopen storage after its failed shutdown flush"));
        }
        if telemetry.pending.is_empty() {
            return Ok(());
        }
        let transaction = connection.transaction()?;
        let mut checkpoints = BTreeMap::new();
        let root = self.database.parent().ok_or(Error::Data("select a valid storage root"))?;
        let result = (|| -> Result<()> {
            let keys = telemetry.pending.keys().cloned().collect::<Vec<_>>();
            for (execution, stream) in keys {
                let pending = &telemetry.pending[&(execution.clone(), stream.clone())];
                let codec = pending.codec.clone();
                let samples = pending.samples.clone();
                let id = catalog::identifier(&execution, &stream, &codec.digest);
                let path = catalog::path(root, &id)?;
                if let std::collections::btree_map::Entry::Vacant(entry) = telemetry.writers.entry(id.clone()) {
                    let exists: bool =
                        transaction.query_row("SELECT EXISTS(SELECT 1 FROM telemetry_files WHERE id=?1)", [&id], |row| row.get(0))?;
                    let writer = if exists {
                        let checkpoint = catalog::checkpoint(&transaction, &id)?;
                        let indices = catalog::indices(&transaction, &id)?;
                        ipc::Writer::resume(&path, codec.schema.clone(), &indices, &checkpoint)?
                    } else if path.exists() {
                        ipc::Writer::recover_empty(&path, codec.schema.clone())?
                    } else {
                        ipc::Writer::create(&path, codec.schema.clone())?
                    };
                    entry.insert(writer);
                }
                let previews = telemetry.settings.previews;
                let rows_per_batch = telemetry.settings.flush_rows.get();
                let writer = telemetry.writers.get_mut(&id).unwrap();
                checkpoints.insert(id.clone(), writer.checkpoint());
                let header = writer.checkpoint();
                transaction.execute(
                    "INSERT OR IGNORE INTO telemetry_files VALUES(?1,?2,?3,?4,?5,?6,0,0,?5,?7)",
                    params![id, execution, stream, codec.digest, header.end as i64, header.hash, writer.digest().as_slice()],
                )?;
                for samples in samples.chunks(rows_per_batch) {
                    let batch = codec.batch(samples)?;
                    let index = writer.append(&batch)?;
                    let checkpoint = writer.checkpoint();
                    super::aggregates::append(&transaction, &id, index.offset, &batch, samples, previews)?;
                    transaction.execute(
                        "INSERT INTO telemetry_batches VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
                        params![
                            id,
                            index.offset as i64,
                            index.metadata_bytes,
                            index.body_bytes as i64,
                            index.rows as i64,
                            index.sha256.as_slice(),
                            samples.iter().map(|s| s.source_ns).min().unwrap(),
                            samples.iter().map(|s| s.source_ns).max().unwrap(),
                            samples.first().unwrap().position as i64,
                            samples.last().unwrap().position as i64
                        ],
                    )?;
                    transaction.execute(
                        "UPDATE telemetry_files SET end=?2,hash_state=?3,batches=?4,length=?2,sha256=?5 WHERE id=?1",
                        params![id, checkpoint.end as i64, checkpoint.hash, checkpoint.batches as i64, checkpoint.digest()?.as_slice()],
                    )?;
                }
            }
            Ok(())
        })();
        if let Err(error) = result {
            drop(transaction);
            for (id, checkpoint) in &checkpoints {
                telemetry.writers.get_mut(id).unwrap().rollback(checkpoint)?;
            }
            return Err(error);
        }
        if let Err(error) = transaction.commit() {
            for (id, checkpoint) in &checkpoints {
                telemetry.writers.get_mut(id).unwrap().rollback(checkpoint)?;
            }
            return Err(error.into());
        }
        telemetry.pending.clear();
        self.changed.send_modify(|revision| *revision += 1);
        Ok(())
    }
    pub(crate) fn finalize_samples(&self, connection: &mut Connection, execution: &str) -> Result<()> {
        self.flush_samples_locked(connection)?;
        let mut telemetry = self.telemetry.lock().map_err(|_| Error::Data("restart after the telemetry writer lock failed"))?;
        let transaction = connection.transaction()?;
        let mut query = transaction.prepare("SELECT id,stream FROM telemetry_files WHERE execution=?1 AND finalized=0 ORDER BY id")?;
        let files = query
            .query_map([execution], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        drop(query);
        let mut finished = Vec::new();
        let result = (|| -> Result<()> {
            for (id, stream) in &files {
                let codec = telemetry.codec(&transaction, execution, stream)?;
                let path = catalog::path(self.database.parent().unwrap(), id)?;
                let checkpoint = catalog::checkpoint(&transaction, id)?;
                let indices = catalog::indices(&transaction, id)?;
                let (length, digest) = if let Some(writer) = telemetry.writers.get_mut(id) {
                    writer.finish()?
                } else {
                    ipc::recover(&path, codec.schema.clone(), &indices, &checkpoint)?
                };
                finished.push((id.clone(), codec.schema.clone(), indices, checkpoint));
                transaction.execute(
                    "UPDATE telemetry_files SET finalized=1,length=?2,sha256=?3 WHERE id=?1",
                    params![id, length as i64, digest.as_slice()],
                )?;
            }
            transaction.execute("UPDATE streams SET closed=1 WHERE execution=?1", [execution])?;
            Ok(())
        })();
        let result = result.and_then(|()| Ok(transaction.commit()?));
        for (id, schema, indices, checkpoint) in finished {
            telemetry.writers.remove(&id);
            if result.is_err() {
                telemetry.writers.insert(
                    id.clone(),
                    ipc::Writer::resume(&catalog::path(self.database.parent().unwrap(), &id)?, schema, &indices, &checkpoint)?,
                );
            }
        }
        result
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
