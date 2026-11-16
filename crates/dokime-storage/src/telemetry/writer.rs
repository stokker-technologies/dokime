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
use arrow_array::{Array, Int64Array, RecordBatch, StructArray, UInt64Array};
use arrow_select::concat::concat_batches;
use prost::Message;
use prost_reflect::DescriptorPool;
use rusqlite::{Connection, params};
use tokio::time::Instant;

// Local Imports
use super::{Pending, Telemetry, catalog, codec::Codec};
use crate::{Error, Execution, Result, Sample, Store, ipc};
use harmos_protocol::model::Declarations;

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
        let frozen = Declarations::decode(definition.catalog.as_slice())?;
        let declaration =
            frozen.streams.iter().find(|entry| entry.id == stream).ok_or(Error::Data("declare the stream in the frozen catalog"))?;
        let pool = DescriptorPool::from_file_descriptor_set(frozen.types.ok_or(Error::Data("include frozen stream descriptors"))?)
            .map_err(|_| Error::Data("include valid frozen stream descriptors"))?;
        let descriptor = pool.get_message_by_name(&declaration.row).ok_or(Error::Data("include the frozen row descriptor"))?;
        let codec = Arc::new(Codec::new(descriptor)?);
        self.codecs.insert(key, codec.clone());
        Ok(codec)
    }
    /// Whether Accepted Rows Reached the Pending Byte Pressure or One Stream's Flush Row Count.
    fn flush_due(&self) -> bool {
        self.pending.values().map(|pending| pending.bytes).sum::<usize>() >= self.settings.pending_bytes.get()
            || self.pending.values().any(|pending| pending.rows >= self.settings.flush_rows.get())
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
            if !prepared.contains_key(&sample.stream) {
                require_stream(&connection, execution, &sample.stream)?;
                let codec = telemetry.codec(&connection, execution, &sample.stream)?;
                prepared.insert(sample.stream.clone(), (codec, Vec::new()));
            }
            let mut sample = sample.clone();
            sample.position = position;
            positions.push(position);
            prepared.get_mut(&sample.stream).unwrap().1.push(sample);
            position = position
                .checked_add(1)
                .filter(|p| *p <= i64::MAX as u64)
                .ok_or(Error::Data("start a fresh execution before positions overflow"))?;
        }
        // Validate Every Row Before Admitting Any Part Of The Call.
        let batches =
            prepared.into_iter().map(|(stream, (codec, rows))| Ok((stream, codec.batch(&rows)?, codec))).collect::<Result<Vec<_>>>()?;
        accept(&connection, execution, position - 1)?;
        for (stream, batch, codec) in batches {
            admit(&mut telemetry, execution, stream, codec, batch);
        }
        let flush = telemetry.flush_due();
        drop(telemetry);
        drop(connection);
        self.admitted(flush)?;
        Ok(positions)
    }
    /// Admit One Stored Interval Batch Row as Consecutive Samples, Columns Straight Into Arrow.
    ///
    /// `payload` is the host's stored `harmos.batch.<row>` encoding; `sequence` numbers its first
    /// sample. Every sample takes one position and the time `start + index * period`. Returns the
    /// number of samples admitted.
    pub fn sample_interval(&self, execution: &str, stream: &str, payload: &[u8], sequence: u64) -> Result<u64> {
        let connection = self.lock()?;
        crate::store::require_open(&connection, execution)?;
        let mut telemetry = self.telemetry.lock().map_err(|_| Error::Data("restart after the telemetry writer lock failed"))?;
        if telemetry.closed {
            return Err(Error::Data("open a storage session before publishing samples"));
        }
        require_stream(&connection, execution, stream)?;
        let codec = telemetry.codec(&connection, execution, stream)?;
        let interval = codec.interval(payload)?;
        let count = interval.count as u64;
        let first = crate::store::next_position(&connection, execution)?;
        let last = first
            .checked_add(count - 1)
            .filter(|last| *last <= i64::MAX as u64)
            .ok_or(Error::Data("start a fresh execution before positions overflow"))?;
        sequence.checked_add(count).ok_or(Error::Data("restart the stream before its sequence overflows"))?;
        let batch = codec.assemble(first, sequence, &interval)?;
        accept(&connection, execution, last)?;
        admit(&mut telemetry, execution, stream.to_owned(), codec, batch);
        let flush = telemetry.flush_due();
        drop(telemetry);
        drop(connection);
        self.admitted(flush)?;
        Ok(count)
    }
    fn admitted(&self, flush: bool) -> Result<()> {
        if flush {
            self.flush_samples()?;
        }
        self.start_sample_flush()?;
        self.changed.send_modify(|revision| *revision += 1);
        Ok(())
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
                let samples = match pending.batches.as_slice() {
                    [batch] => batch.clone(),
                    batches => concat_batches(&codec.schema, batches)?,
                };
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
                for offset in (0..samples.num_rows()).step_by(rows_per_batch) {
                    let batch = samples.slice(offset, rows_per_batch.min(samples.num_rows() - offset));
                    let (first, last, start, end) = span(&batch)?;
                    let index = writer.append(&batch)?;
                    let checkpoint = writer.checkpoint();
                    super::aggregates::append(&transaction, &id, index.offset, &batch, previews)?;
                    transaction.execute(
                        "INSERT INTO telemetry_batches VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
                        params![
                            id,
                            index.offset as i64,
                            index.metadata_bytes,
                            index.body_bytes as i64,
                            index.rows as i64,
                            index.sha256.as_slice(),
                            start,
                            end,
                            first as i64,
                            last as i64
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

fn require_stream(connection: &Connection, execution: &str, stream: &str) -> Result<()> {
    let closed: i32 =
        connection
            .query_row("SELECT closed FROM streams WHERE execution=?1 AND stream=?2", params![execution, stream], |row| row.get(0))?;
    if closed != 0 {
        return Err(Error::Data("open the execution stream before publishing samples"));
    }
    Ok(())
}

fn accept(connection: &Connection, execution: &str, through: u64) -> Result<()> {
    connection.execute(
        "INSERT INTO temp.accepted VALUES(?1,?2) ON CONFLICT(execution) DO UPDATE SET position=excluded.position",
        params![execution, through as i64],
    )?;
    Ok(())
}

fn admit(telemetry: &mut Telemetry, execution: &str, stream: String, codec: Arc<Codec>, batch: RecordBatch) {
    let pending = telemetry.pending.entry((execution.into(), stream)).or_insert_with(|| Pending {
        codec,
        started: Instant::now(),
        batches: Vec::new(),
        rows: 0,
        bytes: 0,
    });
    pending.rows += batch.num_rows();
    // Count the Bytes the Rows Use, Not Buffer Capacity, So Small Device Admissions Keep the Byte Bound Meaningful.
    pending.bytes += batch.columns().iter().map(|column| column.to_data().get_slice_memory_size().unwrap_or_default()).sum::<usize>();
    pending.batches.push(batch);
}

/// The First and Last Positions and the Earliest and Latest Source Times of a Stored Batch.
pub(super) fn span(batch: &RecordBatch) -> Result<(u64, u64, i64, i64)> {
    let storage = batch.column(0).as_any().downcast_ref::<StructArray>().ok_or(Error::Corrupt("storage metadata differs"))?;
    let positions = storage
        .column_by_name("position")
        .and_then(|column| column.as_any().downcast_ref::<UInt64Array>())
        .ok_or(Error::Corrupt("position type differs"))?;
    let times = storage
        .column_by_name("source_ns")
        .and_then(|column| column.as_any().downcast_ref::<Int64Array>())
        .ok_or(Error::Corrupt("time type differs"))?;
    let (Some(first), Some(last)) = (positions.values().first(), positions.values().last()) else {
        return Err(Error::Data("flush a nonempty sample batch"));
    };
    let start = times.values().iter().copied().min().unwrap();
    let end = times.values().iter().copied().max().unwrap();
    Ok((*first, *last, start, end))
}

/* === Tests === */
