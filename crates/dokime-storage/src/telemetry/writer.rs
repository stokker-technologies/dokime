/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Buffered Durable Arrow Writes
//
// A flush claims the head of every stream's pending batches, prepares each stream file in
// parallel (concatenation, IPC encoding, moments and previews) and commits them together: it
// writes and syncs every file in parallel, then saves one catalog transaction. Claimed rows
// stay pending, and so visible to readers, until that transaction commits. Background rounds
// prepare without the store's locks; a synchronous flush prepares under them and supersedes a
// round in progress, whose claims it detects as stale and drops.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    collections::BTreeMap,
    sync::{
        Arc, Mutex, PoisonError,
        atomic::{AtomicUsize, Ordering},
    },
};

// Third Party Imports
use arrow_array::{Array, Int64Array, RecordBatch, StructArray, UInt64Array};
use arrow_select::concat::concat_batches;
use prost::Message;
use prost_reflect::DescriptorPool;
use rusqlite::{Connection, params};
use tokio::time::Instant;

// Local Imports
use super::{Pending, Telemetry, aggregates, aggregates::Statistics, catalog, codec::Codec, config::StorageSettings};
use crate::{Error, Execution, Result, Sample, Store, ipc};
use harmos_protocol::model::Declarations;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/// One Stream File's Share of a Flush: the Head of Its Pending Batches and Its Saved Statistics.
struct Claim {
    key: (String, String),
    id: String,
    codec: Arc<Codec>,
    batches: Vec<RecordBatch>,
    head: u64,
    statistics: Statistics,
}

/// One Stream File's Encoded Chunks and Catalog Values, Ready to Write and Commit.
struct Prepared {
    key: (String, String),
    id: String,
    codec: Arc<Codec>,
    batches: usize,
    head: u64,
    chunks: Vec<Chunk>,
    statistics: Statistics,
}

struct Chunk {
    encoded: ipc::Encoded,
    span: (u64, u64, i64, i64),
    checkpoints: Vec<(String, Vec<u8>)>,
}

/// A Writer's Durable Prefix Before a Commit's Writes, and Its Batches and End After Them.
struct Written {
    before: ipc::Checkpoint,
    indices: Vec<ipc::BatchIndex>,
    after: ipc::Checkpoint,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Telemetry {
    /// The Codec of One Execution Stream.
    ///
    /// A miss decodes the execution's frozen catalog once and fills every declared stream, and
    /// streams that share a row type share one codec.
    pub(crate) fn codec(&mut self, connection: &Connection, execution: &str, stream: &str) -> Result<Arc<Codec>> {
        let key = (execution.to_owned(), stream.to_owned());
        if let Some(codec) = self.codecs.get(&key) {
            return Ok(codec.clone());
        }
        let bytes: Vec<u8> = connection.query_row("SELECT payload FROM executions WHERE id=?1", [execution], |row| row.get(0))?;
        let definition = Execution::decode(bytes.as_slice())?;
        let mut frozen = Declarations::decode(definition.catalog.as_slice())?;
        let types = frozen.types.take().ok_or(Error::Data("include frozen stream descriptors"))?;
        let declaration =
            frozen.streams.iter().find(|entry| entry.id == stream).ok_or(Error::Data("declare the stream in the frozen catalog"))?;
        let pool = DescriptorPool::from_file_descriptor_set(types).map_err(|_| Error::Data("include valid frozen stream descriptors"))?;
        let descriptor = pool.get_message_by_name(&declaration.row).ok_or(Error::Data("include the frozen row descriptor"))?;
        let requested = Arc::new(Codec::new(descriptor)?);
        let mut rows = BTreeMap::from([(declaration.row.as_str(), requested.clone())]);
        for declaration in &frozen.streams {
            let codec = match rows.get(declaration.row.as_str()) {
                Some(codec) => codec.clone(),
                None => {
                    // A Sibling Stream's Invalid Row Fails Only When That Stream Is Used.
                    let Some(Ok(codec)) = pool.get_message_by_name(&declaration.row).map(Codec::new) else {
                        continue;
                    };
                    let codec = Arc::new(codec);
                    rows.insert(declaration.row.as_str(), codec.clone());
                    codec
                }
            };
            self.codecs.entry((execution.to_owned(), declaration.id.clone())).or_insert(codec);
        }
        self.codecs.insert(key, requested.clone());
        Ok(requested)
    }

    /// Whether Accepted Rows Reached the Pending Byte Bound, Which Flushes on the Admitting Caller.
    fn pressure(&self) -> bool {
        self.pending.values().map(|pending| pending.bytes).sum::<usize>() >= self.settings.pending_bytes.get()
    }

    /// Whether One Stream Reached Its Flush Row Count, Which Starts a Background Round at Once.
    fn due(&self) -> bool {
        self.pending.values().any(|pending| pending.rows >= self.settings.flush_rows.get())
    }

    /// Claim Every Stream's Pending Batches With Their File's Saved Statistics.
    fn claim(&self, connection: &Connection) -> Result<Vec<Claim>> {
        self.pending
            .iter()
            .map(|((execution, stream), pending)| {
                let id = catalog::identifier(execution, stream, &pending.codec.digest);
                Ok(Claim {
                    key: (execution.clone(), stream.clone()),
                    statistics: Statistics::load(connection, &id, self.settings.previews)?,
                    id,
                    codec: pending.codec.clone(),
                    head: head(&pending.batches)?,
                    batches: pending.batches.clone(),
                })
            })
            .collect()
    }

    /// Whether a Prepared File's Claimed Batches Still Lead Its Stream's Pending Batches.
    fn current(&self, prepared: &Prepared) -> bool {
        self.pending
            .get(&prepared.key)
            .is_some_and(|pending| pending.batches.len() >= prepared.batches && head(&pending.batches).ok() == Some(prepared.head))
    }
}

impl Claim {
    fn prepare(self, rows: usize) -> Result<Prepared> {
        let Self {
            key,
            id,
            codec,
            batches,
            head,
            mut statistics,
        } = self;
        let samples = match batches.as_slice() {
            [batch] => batch.clone(),
            batches => concat_batches(&codec.schema, batches)?,
        };
        let mut chunks = Vec::new();
        for offset in (0..samples.num_rows()).step_by(rows) {
            let batch = samples.slice(offset, rows.min(samples.num_rows() - offset));
            let span = span(&batch)?;
            let checkpoints = statistics.advance(&batch, span.1)?;
            chunks.push(Chunk {
                encoded: ipc::encode(&codec.schema, &batch)?,
                span,
                checkpoints,
            });
        }
        Ok(Prepared {
            key,
            id,
            codec,
            batches: batches.len(),
            head,
            chunks,
            statistics,
        })
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
        let (pressure, due) = (telemetry.pressure(), telemetry.due());
        drop(telemetry);
        drop(connection);
        self.admitted(pressure, due)?;
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
        let (pressure, due) = (telemetry.pressure(), telemetry.due());
        drop(telemetry);
        drop(connection);
        self.admitted(pressure, due)?;
        Ok(count)
    }
    fn admitted(&self, pressure: bool, due: bool) -> Result<()> {
        // Byte Pressure Bounds Memory, So It Flushes Here; Without a Runtime No Round Can Take a Due Flush.
        if pressure || (due && tokio::runtime::Handle::try_current().is_err()) {
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
        let worker = runtime.spawn(async move {
            let mut changes = store.changed.subscribe();
            loop {
                let deadline = {
                    let mut telemetry = match store.telemetry.lock() {
                        Ok(t) => t,
                        Err(_) => return,
                    };
                    if telemetry.closed {
                        telemetry.running = false;
                        return;
                    }
                    let interval = telemetry.settings.flush_interval;
                    match telemetry.pending.values().map(|pending| pending.started + interval).min() {
                        Some(_) if telemetry.due() => Instant::now(),
                        Some(deadline) => deadline,
                        None => {
                            telemetry.running = false;
                            return;
                        }
                    }
                };
                tokio::select! {biased; _=tokio::time::sleep_until(deadline)=>{}, result=changes.changed()=>{if result.is_err(){return;}continue;}}
                let worker = store.clone();
                let result = tokio::task::spawn_blocking(move || worker.flush_round()).await;
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
    /// Flush Every Pending Row Before Returning; Its Caller Holds the Store's Connection.
    pub(crate) fn flush_samples_locked(&self, connection: &mut Connection) -> Result<()> {
        let mut telemetry = self.telemetry.lock().map_err(|_| Error::Data("restart after the telemetry writer lock failed"))?;
        if telemetry.closed && !telemetry.pending.is_empty() {
            return Err(Error::Data("reopen storage after its failed shutdown flush"));
        }
        if telemetry.pending.is_empty() {
            return Ok(());
        }
        let prepared = prepare(telemetry.claim(connection)?, &telemetry.settings)?;
        self.commit(connection, &mut telemetry, prepared)
    }
    /// One Background Round: Claim Under the Locks, Prepare Without Them, Then Commit What Is Still Pending.
    fn flush_round(&self) -> Result<()> {
        let (claims, settings) = {
            let connection = self.lock()?;
            let telemetry = self.telemetry.lock().map_err(|_| Error::Data("restart after the telemetry writer lock failed"))?;
            if telemetry.closed || telemetry.pending.is_empty() {
                return Ok(());
            }
            (telemetry.claim(&connection)?, telemetry.settings)
        };
        let prepared = prepare(claims, &settings)?;
        let mut connection = self.lock()?;
        let mut telemetry = self.telemetry.lock().map_err(|_| Error::Data("restart after the telemetry writer lock failed"))?;
        if telemetry.closed {
            return Ok(());
        }
        // A Synchronous Flush Since the Claim Saved These Rows Itself; Their Prepared Copies Are Dropped.
        let prepared = prepared.into_iter().filter(|prepared| telemetry.current(prepared)).collect();
        self.commit(&mut connection, &mut telemetry, prepared)
    }
    /// Write, Sync and Catalog Prepared Files, Then Release Their Claimed Batches.
    ///
    /// Every file is synced before the one catalog transaction commits, so a saved position never
    /// names rows a crash could lose; a failure rolls every writer back to its committed prefix.
    fn commit(&self, connection: &mut Connection, telemetry: &mut Telemetry, prepared: Vec<Prepared>) -> Result<()> {
        if prepared.is_empty() {
            return Ok(());
        }
        let root = self.database.parent().ok_or(Error::Data("select a valid storage root"))?;
        let mut openings = Vec::new();
        for prepared in prepared.iter().filter(|prepared| !telemetry.writers.contains_key(&prepared.id)) {
            let exists: bool =
                connection.query_row("SELECT EXISTS(SELECT 1 FROM telemetry_files WHERE id=?1)", [&prepared.id], |row| row.get(0))?;
            let saved = if exists {
                Some((catalog::indices(connection, &prepared.id)?, catalog::checkpoint(connection, &prepared.id)?))
            } else {
                None
            };
            openings.push((prepared.id.clone(), prepared.codec.schema.clone(), catalog::path(root, &prepared.id)?, saved));
        }
        // New Files Sync Their Header and Directory Entry, So They Open in Parallel Too.
        for (id, writer) in parallel(openings, |(id, schema, path, saved)| {
            let writer = match saved {
                Some((indices, checkpoint)) => ipc::Writer::resume(&path, schema, &indices, &checkpoint),
                None if path.exists() => ipc::Writer::recover_empty(&path, schema),
                None => ipc::Writer::create(&path, schema),
            };
            (id, writer)
        }) {
            telemetry.writers.insert(id, writer?);
        }
        let written = {
            let mut files = prepared.iter().map(|prepared| (prepared.id.as_str(), prepared)).collect::<BTreeMap<_, _>>();
            let work = telemetry
                .writers
                .iter_mut()
                .filter_map(|(id, writer)| files.remove(id.as_str()).map(|prepared| (writer, prepared)))
                .collect::<Vec<_>>();
            parallel(work, |(writer, prepared)| {
                let before = writer.checkpoint();
                let indices = prepared.chunks.iter().map(|chunk| writer.write(&chunk.encoded)).collect::<Result<Vec<_>>>();
                let result = indices.and_then(|indices| {
                    writer.sync()?;
                    Ok((indices, writer.checkpoint()))
                });
                (prepared.id.clone(), before, result)
            })
        };
        let mut befores = Vec::new();
        let mut files = BTreeMap::new();
        let mut failure = None;
        for (id, before, result) in written {
            match result {
                Ok((indices, after)) => {
                    files.insert(id.clone(), Written { before: before.clone(), indices, after });
                }
                Err(error) => {
                    failure.get_or_insert(error);
                }
            }
            befores.push((id, before));
        }
        let rollback = |telemetry: &mut Telemetry, error: Error| -> Result<()> {
            for (id, before) in &befores {
                telemetry.writers.get_mut(id).ok_or(Error::Data("restart after an Arrow writer went missing"))?.rollback(before)?;
            }
            Err(error)
        };
        if let Some(error) = failure {
            return rollback(telemetry, error);
        }
        let transaction = match connection.transaction() {
            Ok(transaction) => transaction,
            Err(error) => return rollback(telemetry, error.into()),
        };
        let result = (|| -> Result<()> {
            for prepared in &prepared {
                let file = files.get(&prepared.id).ok_or(Error::Data("restart after an Arrow writer went missing"))?;
                transaction.prepare_cached("INSERT OR IGNORE INTO telemetry_files VALUES(?1,?2,?3,?4,?5,?6,0,0,?5,?7)")?.execute(
                    params![
                        prepared.id,
                        prepared.key.0,
                        prepared.key.1,
                        prepared.codec.digest,
                        file.before.end as i64,
                        file.before.hash,
                        file.before.digest()?.as_slice()
                    ],
                )?;
                for (chunk, index) in prepared.chunks.iter().zip(&file.indices) {
                    for (field, moments) in &chunk.checkpoints {
                        aggregates::checkpoint(&transaction, &prepared.id, index.offset, field, chunk.span, moments)?;
                    }
                    let (first, last, start, end) = chunk.span;
                    transaction.prepare_cached("INSERT INTO telemetry_batches VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)")?.execute(
                        params![
                            prepared.id,
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
                }
                transaction
                    .prepare_cached("UPDATE telemetry_files SET end=?2,hash_state=?3,batches=?4,length=?2,sha256=?5 WHERE id=?1")?
                    .execute(params![
                    prepared.id,
                    file.after.end as i64,
                    file.after.hash,
                    file.after.batches as i64,
                    file.after.digest()?.as_slice()
                ])?;
                prepared.statistics.save(&transaction, &prepared.id)?;
            }
            Ok(())
        })();
        if let Err(error) = result.and_then(|()| Ok(transaction.commit()?)) {
            return rollback(telemetry, error);
        }
        for prepared in &prepared {
            let pending = telemetry.pending.get_mut(&prepared.key).ok_or(Error::Data("restart after pending samples went missing"))?;
            for batch in pending.batches.drain(..prepared.batches) {
                pending.rows -= batch.num_rows();
                pending.bytes -= bytes(&batch);
            }
            if pending.batches.is_empty() {
                telemetry.pending.remove(&prepared.key);
            } else {
                pending.started = Instant::now();
            }
        }
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
    pending.bytes += bytes(&batch);
    pending.batches.push(batch);
}

/// The Bytes a Batch's Rows Use, Not Buffer Capacity, So Small Device Admissions Keep the Byte Bound Meaningful.
fn bytes(batch: &RecordBatch) -> usize {
    batch.columns().iter().map(|column| column.to_data().get_slice_memory_size().unwrap_or_default()).sum()
}

/// Prepare Every Claimed File in Parallel.
fn prepare(claims: Vec<Claim>, settings: &StorageSettings) -> Result<Vec<Prepared>> {
    let rows = settings.flush_rows.get();
    parallel(claims, |claim| claim.prepare(rows)).into_iter().collect()
}

/// Run `work` Over Every Item on up to the Machine's Parallelism, Returning Results in Item Order.
fn parallel<T: Send, R: Send>(items: Vec<T>, work: impl Fn(T) -> R + Sync) -> Vec<R> {
    let threads = std::thread::available_parallelism().map_or(1, |threads| threads.get()).min(items.len());
    if threads <= 1 {
        return items.into_iter().map(work).collect();
    }
    let next = AtomicUsize::new(0);
    let slots = items.into_iter().map(|item| Mutex::new((Some(item), None))).collect::<Vec<_>>();
    std::thread::scope(|scope| {
        for _ in 0..threads {
            scope.spawn(|| {
                while let Some(slot) = slots.get(next.fetch_add(1, Ordering::Relaxed)) {
                    let item = slot.lock().unwrap_or_else(PoisonError::into_inner).0.take();
                    if let Some(item) = item {
                        let result = work(item);
                        slot.lock().unwrap_or_else(PoisonError::into_inner).1 = Some(result);
                    }
                }
            });
        }
    });
    // A Panicking Worker Panics the Scope, So Every Slot Holds Its Result Here.
    slots.into_iter().map(|slot| slot.into_inner().unwrap_or_else(PoisonError::into_inner).1.expect("parallel work completed")).collect()
}

/// The First Position of a Stream's Pending Batches.
fn head(batches: &[RecordBatch]) -> Result<u64> {
    let batch = batches.first().ok_or(Error::Data("flush a nonempty sample batch"))?;
    let storage = batch.column(0).as_any().downcast_ref::<StructArray>().ok_or(Error::Corrupt("storage metadata differs"))?;
    let positions = storage
        .column_by_name("position")
        .and_then(|column| column.as_any().downcast_ref::<UInt64Array>())
        .ok_or(Error::Corrupt("position type differs"))?;
    positions.values().first().copied().ok_or(Error::Data("flush a nonempty sample batch"))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::telemetry::{aggregates::Moments, config::storage_defaults, previews::Envelope};
    use dokime_protocol::model::{Import, Origin, origin};
    use harmos_protocol::model::StreamDeclaration;
    use prost_types::{
        DescriptorProto, FieldDescriptorProto, FileDescriptorProto, FileDescriptorSet,
        field_descriptor_proto::{Label, Type},
    };
    use std::{io::Write, num::NonZeroUsize, sync::atomic::AtomicBool};

    const STREAMS: usize = 16;
    const ROWS: u64 = 37;
    const ROUNDS: u64 = 40;

    #[derive(Clone, PartialEq, prost::Message)]
    struct Row {
        #[prost(uint64, tag = "1")]
        value: u64,
    }

    fn stream(index: usize) -> String {
        format!("stream_{index:02}")
    }

    fn execution() -> Execution {
        let types = FileDescriptorSet {
            file: vec![FileDescriptorProto {
                name: Some("fixture.proto".into()),
                package: Some("fixture".into()),
                syntax: Some("proto3".into()),
                message_type: vec![DescriptorProto {
                    name: Some("Row".into()),
                    field: vec![FieldDescriptorProto {
                        name: Some("value".into()),
                        number: Some(1),
                        label: Some(Label::Optional as i32),
                        r#type: Some(Type::Uint64 as i32),
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        };
        Execution {
            identifier: "execution".into(),
            cycle: "cycle".into(),
            origin: Some(Origin {
                kind: Some(origin::Kind::Import(Import::default())),
            }),
            catalog: Declarations {
                types: Some(types),
                streams: (0..STREAMS)
                    .map(|index| StreamDeclaration {
                        id: stream(index),
                        row: "fixture.Row".into(),
                        ..Default::default()
                    })
                    .collect(),
                ..Default::default()
            }
            .encode_to_vec(),
            ..Default::default()
        }
    }

    /// One Admission Round: `ROWS` Rows per Stream, Interleaved, Each Value Naming Its Round and Row.
    fn samples(round: u64) -> Vec<Sample> {
        (0..ROWS * STREAMS as u64)
            .map(|index| Sample {
                stream: stream(index as usize % STREAMS),
                source_ns: (round * ROWS + index / STREAMS as u64) as i64 * 1_000_000,
                row: Row { value: round * 1_000 + index }.encode_to_vec(),
                ..Default::default()
            })
            .collect()
    }

    /// Every Admitted Row Is Saved Exactly Once, and Each File's Summary, Chunk Moments and Preview Count It Once.
    fn require_saved_once(store: &Store, rounds: u64) -> Result<()> {
        let total = rounds * ROWS * STREAMS as u64;
        assert_eq!(store.cut("execution")?.frontier, total);
        let mut positions = store.read("execution")?.samples.iter().map(|sample| sample.position).collect::<Vec<_>>();
        positions.sort_unstable();
        assert_eq!(positions, (1..=total).collect::<Vec<_>>());
        let connection = store.lock()?;
        let mut files = connection.prepare("SELECT s.file,s.moments,p.payload,(SELECT SUM(rows) FROM telemetry_batches b WHERE b.file=s.file) FROM telemetry_summaries s JOIN telemetry_previews p ON p.file=s.file AND p.field=s.field")?;
        let files = files
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, Vec<u8>>(1)?, row.get::<_, Vec<u8>>(2)?, row.get::<_, i64>(3)?))
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        assert_eq!(files.len(), STREAMS);
        for (id, summary, preview, rows) in files {
            let rows = rows as u64;
            assert_eq!(rows, rounds * ROWS);
            assert_eq!(serde_json::from_slice::<Moments>(&summary)?.count, rows);
            assert_eq!(postcard::from_bytes::<Envelope>(&preview)?.buckets.values().map(|bucket| bucket.count).sum::<u64>(), rows);
            let mut checkpoints = connection.prepare("SELECT moments FROM telemetry_checkpoints WHERE file=?1")?;
            let counted = checkpoints
                .query_map([&id], |row| row.get::<_, Vec<u8>>(0))?
                .map(|moments| Ok(serde_json::from_slice::<Moments>(&moments?)?.count))
                .sum::<Result<u64>>()?;
            assert_eq!(counted, rows);
        }
        Ok(())
    }

    #[tokio::test(start_paused = true)]
    async fn a_due_row_count_flushes_in_the_background_before_the_interval() -> Result<()> {
        let root = tempfile::tempdir()?;
        let mut settings = storage_defaults();
        settings.flush_rows = NonZeroUsize::new(16).unwrap();
        let store = Store::open_with(root.path(), settings)?;
        store.create(&execution())?;
        for index in 0..STREAMS {
            store.open_stream("execution", &stream(index))?;
        }
        let admitted = Instant::now();
        store.sample_batch("execution", &samples(0))?;
        // Admission Returns Before the Flush; the Background Round Saves the Rows Without Waiting for the Interval.
        assert_eq!(store.cut("execution")?.frontier, 0);
        let mut changes = store.subscribe();
        while store.cut("execution")?.frontier < ROWS * STREAMS as u64 {
            changes.changed().await.unwrap();
        }
        assert!(admitted.elapsed() < settings.flush_interval);
        store.shutdown_samples().await?;
        require_saved_once(&store, 1)
    }

    #[test]
    fn concurrent_per_stream_flushes_save_each_row_once_and_survive_a_crash() -> Result<()> {
        let root = tempfile::tempdir()?;
        let mut settings = storage_defaults();
        settings.flush_rows = NonZeroUsize::new(16).unwrap();
        let store = Store::open_with(root.path(), settings)?;
        store.create(&execution())?;
        for index in 0..STREAMS {
            store.open_stream("execution", &stream(index))?;
        }
        // A Round Whose Claims a Synchronous Flush Saved First Finds Them Stale and Commits Nothing.
        store.sample_batch("execution", &samples(0))?;
        let claims = {
            let connection = store.lock()?;
            store.telemetry.lock().unwrap().claim(&connection)?
        };
        let prepared = prepare(claims, &settings)?;
        store.flush_samples()?;
        store.sample_batch("execution", &samples(1))?;
        assert!(!prepared.iter().any(|prepared| store.telemetry.lock().unwrap().current(prepared)));
        // Background Rounds Flush All Stream Files in Parallel While Rows and Synchronous Flushes Keep Arriving.
        let done = AtomicBool::new(false);
        std::thread::scope(|scope| -> Result<()> {
            let rounds = scope.spawn(|| -> Result<()> {
                while !done.load(Ordering::SeqCst) {
                    store.flush_round()?;
                }
                Ok(())
            });
            for round in 2..ROUNDS {
                store.sample_batch("execution", &samples(round))?;
                if round % 9 == 0 {
                    store.flush_samples()?;
                }
            }
            done.store(true, Ordering::SeqCst);
            rounds.join().unwrap()
        })?;
        store.flush_round()?;
        store.flush_samples()?;
        require_saved_once(&store, ROUNDS)?;
        // A Crash During an Uncommitted Write Leaves Bytes Past Each File's Saved Prefix.
        drop(store);
        for entry in std::fs::read_dir(root.path().join("telemetry"))? {
            std::fs::OpenOptions::new().append(true).open(entry?.path())?.write_all(b"uncommitted partial batch")?;
        }
        let store = Store::open_with(root.path(), settings)?;
        require_saved_once(&store, ROUNDS)?;
        store.sample_batch("execution", &samples(ROUNDS))?;
        store.close_streams("execution")?;
        drop(store);
        let store = Store::open_with(root.path(), settings)?;
        require_saved_once(&store, ROUNDS + 1)?;
        for entry in std::fs::read_dir(root.path().join("telemetry"))? {
            let reader = arrow_ipc::reader::FileReader::try_new(std::fs::File::open(entry?.path())?, None)?;
            assert_eq!(reader.map(|batch| Ok(batch?.num_rows() as u64)).sum::<Result<u64>>()?, (ROUNDS + 1) * ROWS);
        }
        Ok(())
    }
}
