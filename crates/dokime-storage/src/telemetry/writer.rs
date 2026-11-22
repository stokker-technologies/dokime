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
// Intake (open streams, accepted positions and pending rows) has its own short lock and never
// takes the connection or the flush lock, so admitting samples never waits for a sync; only
// bytes over the pending bound make the admitting caller flush. A flush copies the head of every
// stream's pending batches, prepares each stream file in parallel (concatenation, IPC encoding,
// moments and previews) and commits them together: it writes and syncs every file, then saves
// one catalog transaction, and only then drops the batches from intake. Claimed rows stay
// pending, and so visible to readers, until that transaction commits. Background rounds also
// sync without the connection: a round writes under it, leaves its batches unsynced, syncs
// duplicated file handles, then catalogs the batches if they are still its own; whoever holds
// the connection next first syncs and catalogs them itself. Evidence reserves its position
// before its barrier flush, which saves only the execution's samples before that position.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    collections::BTreeMap,
    sync::{
        Arc, Mutex, MutexGuard, PoisonError,
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
use super::{Intake, Pending, Telemetry, aggregates::Statistics, catalog, codec::Codec, config::StorageSettings};
use crate::{Error, Execution, Result, Sample, Store, ipc};
use harmos_protocol::model::Declarations;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

type StreamKey = (String, String);

/* === Structs === */

/// The Head of One Stream's Pending Batches, Copied From Intake.
struct Head {
    key: StreamKey,
    codec: Arc<Codec>,
    head: u64,
    batches: Vec<RecordBatch>,
}

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
}

/// A Writer's Batches and End After a Commit's Writes.
struct Written {
    indices: Vec<ipc::BatchIndex>,
    after: ipc::Checkpoint,
}

/// Prepared Files Written but Neither Synced nor Cataloged, With Each Writer's End Before Them.
pub(super) struct Unsynced {
    round: u64,
    prepared: Vec<Prepared>,
    files: BTreeMap<String, Written>,
    befores: Vec<(String, ipc::Checkpoint)>,
}

/// A Position Reserved for Evidence Behind a Barrier.
///
/// Dropped before the evidence that used it committed, it returns the position unless intake has
/// already allocated past it, so a failed or replayed request leaves no gap.
pub(crate) struct Reservation {
    store: Store,
    execution: String,
    position: u64,
    used: std::cell::Cell<bool>,
    committed: std::cell::Cell<bool>,
}

/// A Background Round Between Its Writes and Its Catalog Commit.
struct Round {
    id: u64,
    handles: Vec<ipc::SyncHandle>,
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
}

impl Intake {
    /// Whether Accepted Rows Exceed the Pending Byte Bound, Which Flushes on the Admitting Caller.
    fn pressure(&self) -> bool {
        self.pending.values().map(|pending| pending.bytes).sum::<usize>() > self.settings.pending_bytes.get()
    }

    /// Whether One Stream Reached Its Flush Row Count, Which Starts a Background Round at Once.
    fn due(&self) -> bool {
        self.pending.values().any(|pending| pending.rows >= self.settings.flush_rows.get())
    }

    /// The Codec of an Open Execution Stream; Only Open Streams Accept Samples.
    fn stream(&self, execution: &str, stream: &str) -> Result<Arc<Codec>> {
        self.streams
            .get(&(execution.to_owned(), stream.to_owned()))
            .cloned()
            .ok_or(Error::Data("open the execution stream before publishing samples"))
    }

    /// The First Free Position of an Execution With Open Streams.
    fn next(&self, execution: &str) -> Result<u64> {
        let accepted = self.accepted.get(execution).ok_or(Error::Data("open the execution stream before publishing samples"))?;
        accepted
            .checked_add(1)
            .filter(|next| *next <= i64::MAX as u64)
            .ok_or(Error::Data("start a fresh execution before positions overflow"))
    }

    /// Copy the Head of Every Stream's Pending Batches; Behind a Fence, Only Batches Before Its Position.
    fn claim(&self, fence: Option<(&str, u64)>) -> Result<Vec<Head>> {
        let mut claims = Vec::with_capacity(self.pending.len());
        for (key, pending) in &self.pending {
            let mut batches = pending.batches.as_slice();
            if let Some((_, position)) = fence.filter(|(execution, _)| *execution == key.0) {
                let before = batches
                    .iter()
                    .map(|batch| head(std::slice::from_ref(batch)))
                    .take_while(|first| first.as_ref().is_ok_and(|first| *first < position))
                    .count();
                batches = &batches[..before];
            }
            if batches.is_empty() {
                continue;
            }
            claims.push(Head {
                key: key.clone(),
                codec: pending.codec.clone(),
                head: head(batches)?,
                batches: batches.to_vec(),
            });
        }
        Ok(claims)
    }

    /// Whether a Prepared File's Claimed Batches Still Lead Its Stream's Pending Batches.
    fn current(&self, prepared: &Prepared) -> bool {
        self.pending
            .get(&prepared.key)
            .is_some_and(|pending| pending.batches.len() >= prepared.batches && head(&pending.batches).ok() == Some(prepared.head))
    }

    /// Drop Cataloged Batches From the Head of Their Streams' Pending Rows.
    fn release(&mut self, prepared: &[Prepared]) -> Result<()> {
        for prepared in prepared {
            let pending = self.pending.get_mut(&prepared.key).ok_or(Error::Data("restart after pending samples went missing"))?;
            for batch in pending.batches.drain(..prepared.batches) {
                pending.rows -= batch.num_rows();
                pending.bytes -= bytes(&batch);
            }
            if pending.batches.is_empty() {
                self.pending.remove(&prepared.key);
            } else {
                pending.started = Instant::now();
            }
        }
        Ok(())
    }
}

impl Reservation {
    /// The Reserved Position, for Evidence Inserted at It.
    pub(crate) fn take(&self) -> u64 {
        self.used.set(true);
        self.position
    }
    /// Keep the Position Once the Transaction Holding Its Evidence Committed.
    pub(crate) fn committed(&self) {
        self.committed.set(true);
    }
}

impl Drop for Reservation {
    fn drop(&mut self) {
        if self.used.get() && self.committed.get() {
            return;
        }
        if let Ok(mut intake) = self.store.intake()
            && let Some(accepted) = intake.accepted.get_mut(&self.execution)
            && *accepted == self.position
        {
            *accepted -= 1;
        }
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
            statistics.advance(&batch, span.1)?;
            chunks.push(Chunk {
                encoded: ipc::encode(&codec.schema, &batch)?,
                span,
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
    /// The Intake Lock; Holders Never Write, Sync or Query the Catalog.
    pub(crate) fn intake(&self) -> Result<MutexGuard<'_, Intake>> {
        self.intake.lock().map_err(|_| Error::Data("restart after the sample intake lock failed"))
    }
    /// Reopen Intake for the Streams a Previous Process Left Open, Continuing After Their Saved Positions.
    pub(crate) fn restore_intake(&self) -> Result<()> {
        let connection = self.lock()?;
        let mut statement = connection.prepare("SELECT execution,stream FROM streams WHERE closed=0 ORDER BY execution,stream")?;
        let open = statement
            .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        drop(statement);
        let mut telemetry = self.telemetry.lock().map_err(|_| Error::Data("restart after the telemetry writer lock failed"))?;
        let mut streams = Vec::with_capacity(open.len());
        let mut accepted = BTreeMap::new();
        for (execution, stream) in open {
            if !accepted.contains_key(&execution) {
                accepted.insert(execution.clone(), crate::store::saved_position(&connection, &execution)?);
            }
            // A Stream Whose Frozen Row No Longer Decodes Refuses Samples, as Before It Was Reopened.
            match telemetry.codec(&connection, &execution, &stream) {
                Ok(codec) => streams.push(((execution, stream), codec)),
                Err(error) => tracing::warn!(%error, %execution, %stream, "Reopen Stream Intake"),
            }
        }
        drop(telemetry);
        let mut intake = self.intake()?;
        intake.streams.extend(streams);
        intake.accepted.extend(accepted);
        Ok(())
    }
    /// Forget a Closed Execution's Position Counter; It Accepts No More Samples.
    pub(crate) fn retire_intake(&self, execution: &str) -> Result<()> {
        self.intake()?.accepted.remove(execution);
        Ok(())
    }
    /// Reserve an Execution's Next Position for Evidence; Intake Allocates From the Same Counter.
    pub(crate) fn reserve_position(&self, connection: &Connection, execution: &str) -> Result<u64> {
        let saved = crate::store::saved_position(connection, execution)?;
        let mut intake = self.intake()?;
        let next = saved.max(intake.accepted.get(execution).copied().unwrap_or(0)) + 1;
        if let Some(accepted) = intake.accepted.get_mut(execution) {
            *accepted = next;
        }
        Ok(next)
    }
    /// The Last Position Visible to This Store or Its Captured Read Session.
    pub(crate) fn accepted_position(&self, connection: &Connection, execution: &str) -> Result<u64> {
        let saved = crate::store::saved_position(connection, execution)?;
        if let Some(samples) = &self.live_samples {
            return Ok(saved.max(samples.accepted.get(execution).copied().unwrap_or(0)));
        }
        if self.cancelled.is_some() {
            return Ok(saved);
        }
        Ok(saved.max(self.intake()?.accepted.get(execution).copied().unwrap_or(0)))
    }
    /// Save Every Sample an Execution Accepted So Far and Return the Position of the Evidence After Them.
    ///
    /// The position is reserved first, so samples accepted while the flush syncs follow the evidence
    /// and stay pending; intake keeps admitting throughout. The caller holds the connection until it
    /// commits the evidence at the returned position.
    pub(crate) fn barrier(&self, connection: &mut Connection, execution: &str) -> Result<Reservation> {
        let reservation = Reservation {
            store: self.clone(),
            execution: execution.to_owned(),
            position: self.reserve_position(connection, execution)?,
            used: std::cell::Cell::new(false),
            committed: std::cell::Cell::new(false),
        };
        self.flush_locked(connection, Some((execution, reservation.position)), false)?;
        Ok(reservation)
    }
    /// Open Execution Streams, Creating and Cataloging Each Stream's Arrow File in Parallel.
    ///
    /// Each file and its header are synced before one transaction records the streams and their empty
    /// files. A crash before that commit leaves files without catalog entries, which hold no saved rows;
    /// opening the streams again recovers their verified headers instead of creating them.
    pub fn open_streams(&self, execution: &str, streams: &[String]) -> Result<()> {
        let mut connection = self.lock()?;
        crate::store::require_open(&connection, execution)?;
        let mut telemetry = self.telemetry.lock().map_err(|_| Error::Data("restart after the telemetry writer lock failed"))?;
        let root = self.database.parent().ok_or(Error::Data("select a valid storage root"))?;
        let mut files = Vec::with_capacity(streams.len());
        for stream in streams {
            let codec = telemetry.codec(&connection, execution, stream)?;
            let id = catalog::identifier(execution, stream, &codec.digest);
            // A Cataloged File Holds Saved Rows; Only a File Without an Entry May Be Recovered to Its Header.
            let cataloged: bool =
                connection.query_row("SELECT EXISTS(SELECT 1 FROM telemetry_files WHERE id=?1)", [&id], |row| row.get(0))?;
            if cataloged {
                return Err(Error::Data("open each execution stream once"));
            }
            files.push((stream.as_str(), catalog::path(root, &id)?, id, codec));
        }
        let opened = parallel(files, |(stream, path, id, codec)| {
            let writer = if path.exists() {
                ipc::Writer::recover_empty(&path, codec.schema.clone())
            } else {
                ipc::Writer::create(&path, codec.schema.clone())
            };
            (stream, id, codec, writer)
        });
        let mut writers = Vec::with_capacity(opened.len());
        for (stream, id, codec, writer) in opened {
            writers.push((stream, id, codec, writer?));
        }
        let transaction = connection.transaction()?;
        for (stream, id, codec, writer) in &writers {
            let header = writer.checkpoint();
            transaction.execute("INSERT INTO streams VALUES (?1,?2,0)", params![execution, stream])?;
            transaction.execute(
                "INSERT INTO telemetry_files VALUES(?1,?2,?3,?4,?5,?6,0,0,?5,?7)",
                params![id, execution, stream, codec.digest, header.end as i64, header.hash, header.digest()?.as_slice()],
            )?;
        }
        transaction.commit()?;
        let saved = crate::store::saved_position(&connection, execution)?;
        let mut intake = self.intake()?;
        let accepted = intake.accepted.entry(execution.to_owned()).or_insert(0);
        *accepted = (*accepted).max(saved);
        for (stream, id, codec, writer) in writers {
            intake.streams.insert((execution.to_owned(), stream.to_owned()), codec);
            telemetry.writers.insert(id, writer);
        }
        Ok(())
    }
    pub fn sample_batch(&self, execution: &str, samples: &[Sample]) -> Result<Vec<u64>> {
        if samples.is_empty() {
            return Ok(Vec::new());
        }
        let mut intake = self.intake()?;
        if intake.closed {
            return Err(Error::Data("open a storage session before publishing samples"));
        }
        let mut prepared = BTreeMap::<String, (Arc<Codec>, Vec<Sample>)>::new();
        let mut position = intake.next(execution)?;
        let mut positions = Vec::with_capacity(samples.len());
        for sample in samples {
            if !prepared.contains_key(&sample.stream) {
                let codec = intake.stream(execution, &sample.stream)?;
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
        intake.accepted.insert(execution.to_owned(), position - 1);
        for (stream, batch, codec) in batches {
            admit(&mut intake, execution, stream, codec, batch);
        }
        let (pressure, due) = (intake.pressure(), intake.due());
        drop(intake);
        self.admitted(pressure, due)?;
        Ok(positions)
    }
    /// Admit One Stored Interval Batch Row as Consecutive Samples, Columns Straight Into Arrow.
    ///
    /// `payload` is the host's stored `harmos.batch.<row>` encoding; `sequence` numbers its first
    /// sample. Every sample takes one position and the time `start + index * period`. Returns the
    /// number of samples admitted.
    pub fn sample_interval(&self, execution: &str, stream: &str, payload: &[u8], sequence: u64) -> Result<u64> {
        let mut intake = self.intake()?;
        if intake.closed {
            return Err(Error::Data("open a storage session before publishing samples"));
        }
        let codec = intake.stream(execution, stream)?;
        let interval = codec.interval(payload)?;
        let count = interval.count as u64;
        let first = intake.next(execution)?;
        let last = first
            .checked_add(count - 1)
            .filter(|last| *last <= i64::MAX as u64)
            .ok_or(Error::Data("start a fresh execution before positions overflow"))?;
        sequence.checked_add(count).ok_or(Error::Data("restart the stream before its sequence overflows"))?;
        let batch = codec.assemble(first, sequence, &interval)?;
        intake.accepted.insert(execution.to_owned(), last);
        admit(&mut intake, execution, stream.to_owned(), codec, batch);
        let (pressure, due) = (intake.pressure(), intake.due());
        drop(intake);
        self.admitted(pressure, due)?;
        Ok(count)
    }
    fn admitted(&self, pressure: bool, due: bool) -> Result<()> {
        // Bytes Over the Pending Bound Flush on the Admitting Caller, the Only Wait Intake Makes for a Sync.
        // Without a Runtime No Round Can Take a Due Flush.
        if pressure || (due && tokio::runtime::Handle::try_current().is_err()) {
            self.flush_samples()?;
        }
        self.start_sample_flush()?;
        self.changed.send_modify(|revision| *revision += 1);
        Ok(())
    }
    fn start_sample_flush(&self) -> Result<()> {
        let Ok(runtime) = tokio::runtime::Handle::try_current() else { return Ok(()) };
        let mut intake = self.intake()?;
        if intake.closed || intake.running || intake.pending.is_empty() {
            return Ok(());
        }
        intake.running = true;
        let store = self.clone();
        let worker = runtime.spawn(async move {
            let mut changes = store.changed.subscribe();
            loop {
                let deadline = {
                    let Ok(mut intake) = store.intake() else { return };
                    if intake.closed {
                        intake.running = false;
                        return;
                    }
                    let interval = intake.settings.flush_interval;
                    match intake.pending.values().map(|pending| pending.started + interval).min() {
                        Some(_) if intake.due() => Instant::now(),
                        Some(deadline) => deadline,
                        None => {
                            intake.running = false;
                            return;
                        }
                    }
                };
                tokio::select! {biased; _=tokio::time::sleep_until(deadline)=>{}, result=changes.changed()=>{if result.is_err(){return;}continue;}}
                let worker = store.clone();
                let result = tokio::task::spawn_blocking(move || worker.flush_round()).await;
                if !matches!(result, Ok(Ok(()))) {
                    tracing::error!(?result, "Flush Arrow Samples Failed; Saved Position Was Not Acknowledged");
                    if let Ok(mut intake) = store.intake() {
                        intake.running = false;
                    }
                    return;
                }
            }
        });
        intake.workers.retain(|worker| !worker.is_finished());
        intake.workers.push(worker);
        Ok(())
    }
    pub async fn shutdown_samples(&self) -> Result<()> {
        let (mut result, workers) = {
            let mut connection = self.lock()?;
            // Intake Closes Before the Final Flush, So No Row Is Accepted That It Would Not Save.
            let workers = {
                let mut intake = self.intake()?;
                intake.closed = true;
                intake.running = false;
                std::mem::take(&mut intake.workers)
            };
            (self.flush_locked(&mut connection, None, true), workers)
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
        self.flush_locked(connection, None, false)
    }
    /// Flush Pending Rows, Behind a Fence Only an Execution's Rows Before Its Position, Syncing Without Intake.
    ///
    /// After shutdown closed intake, rows still pending mean its final flush failed; only that
    /// `closing` flush may save them.
    fn flush_locked(&self, connection: &mut Connection, fence: Option<(&str, u64)>, closing: bool) -> Result<()> {
        let mut telemetry = self.telemetry.lock().map_err(|_| Error::Data("restart after the telemetry writer lock failed"))?;
        self.settle(connection, &mut telemetry)?;
        let claims = {
            let intake = self.intake()?;
            if intake.closed && !closing && !intake.pending.is_empty() {
                return Err(Error::Data("reopen storage after its failed shutdown flush"));
            }
            intake.claim(fence)?
        };
        if claims.is_empty() {
            return Ok(());
        }
        let prepared = prepare(load_statistics(connection, claims, &telemetry.settings)?, &telemetry.settings)?;
        let unsynced = self.write(connection, &mut telemetry, prepared)?;
        self.sync_and_catalog(connection, &mut telemetry, unsynced)
    }
    /// One Background Round: Claim Under the Locks, Prepare Without Them, Write Under Them, Sync Without Them.
    fn flush_round(&self) -> Result<()> {
        match self.begin_round()? {
            Some(round) => self.finish_round(round),
            None => Ok(()),
        }
    }
    /// Claim, Prepare and Write One Round, Leaving Its Batches Unsynced With Handles That Sync Them.
    fn begin_round(&self) -> Result<Option<Round>> {
        let (claims, settings) = {
            let mut connection = self.lock()?;
            let mut telemetry = self.telemetry.lock().map_err(|_| Error::Data("restart after the telemetry writer lock failed"))?;
            self.settle(&mut connection, &mut telemetry)?;
            let claims = {
                let intake = self.intake()?;
                if intake.closed {
                    return Ok(None);
                }
                intake.claim(None)?
            };
            if claims.is_empty() {
                return Ok(None);
            }
            (load_statistics(&connection, claims, &telemetry.settings)?, telemetry.settings)
        };
        let prepared = prepare(claims, &settings)?;
        let mut connection = self.lock()?;
        let mut telemetry = self.telemetry.lock().map_err(|_| Error::Data("restart after the telemetry writer lock failed"))?;
        self.settle(&mut connection, &mut telemetry)?;
        // A Synchronous Flush Since the Claim Saved These Rows Itself; Their Prepared Copies Are Dropped.
        let prepared = {
            let intake = self.intake()?;
            if intake.closed {
                return Ok(None);
            }
            prepared.into_iter().filter(|prepared| intake.current(prepared)).collect::<Vec<_>>()
        };
        if prepared.is_empty() {
            return Ok(None);
        }
        let mut unsynced = self.write(&connection, &mut telemetry, prepared)?;
        let mut handles = Vec::with_capacity(unsynced.files.len());
        for id in unsynced.files.keys() {
            let handle = match telemetry.writers.get_mut(id) {
                Some(writer) => writer.sync_handle(),
                None => Err(Error::Data("restart after an Arrow writer went missing")),
            };
            match handle {
                Ok(handle) => handles.push(handle),
                Err(error) => return Err(rollback(&mut telemetry, &unsynced.befores, error)),
            }
        }
        telemetry.rounds += 1;
        unsynced.round = telemetry.rounds;
        let id = unsynced.round;
        telemetry.unsynced = Some(unsynced);
        Ok(Some(Round { id, handles }))
    }
    /// Sync a Round's Files Without the Locks, Then Catalog Its Batches Unless Another Flush Already Did.
    fn finish_round(&self, round: Round) -> Result<()> {
        let synced = parallel(round.handles, ipc::SyncHandle::sync).into_iter().collect::<Result<()>>();
        let mut connection = self.lock()?;
        let mut telemetry = self.telemetry.lock().map_err(|_| Error::Data("restart after the telemetry writer lock failed"))?;
        let Some(unsynced) = telemetry.unsynced.take_if(|unsynced| unsynced.round == round.id) else {
            return Ok(());
        };
        match synced {
            Ok(()) => self.catalog(&mut connection, &mut telemetry, unsynced),
            Err(error) => Err(rollback(&mut telemetry, &unsynced.befores, error)),
        }
    }
    /// Sync and Catalog a Round's Unsynced Batches; Every Flush Holding the Locks Does This Before Claiming.
    fn settle(&self, connection: &mut Connection, telemetry: &mut Telemetry) -> Result<()> {
        match telemetry.unsynced.take() {
            Some(unsynced) => self.sync_and_catalog(connection, telemetry, unsynced),
            None => Ok(()),
        }
    }
    /// Write Prepared Files Without Syncing Them; a Failure Rolls Every Writer Back to Its Committed Prefix.
    fn write(&self, connection: &Connection, telemetry: &mut Telemetry, prepared: Vec<Prepared>) -> Result<Unsynced> {
        let root = self.database.parent().ok_or(Error::Data("select a valid storage root"))?;
        // Every Stream File Is Created and Cataloged When Its Stream Opens; After a Restart It Resumes Here.
        let mut openings = Vec::new();
        for prepared in prepared.iter().filter(|prepared| !telemetry.writers.contains_key(&prepared.id)) {
            let path = catalog::path(root, &prepared.id)?;
            let cataloged: bool =
                connection.query_row("SELECT EXISTS(SELECT 1 FROM telemetry_files WHERE id=?1)", [&prepared.id], |row| row.get(0))?;
            if !cataloged {
                return Err(Error::Data("start a fresh execution: this stream opened before streams created their files"));
            }
            let saved = (catalog::indices(connection, &prepared.id)?, catalog::checkpoint(connection, &prepared.id)?);
            openings.push((prepared.id.clone(), prepared.codec.schema.clone(), path, saved));
        }
        for (id, writer) in
            parallel(openings, |(id, schema, path, (indices, checkpoint))| (id, ipc::Writer::resume(&path, schema, &indices, &checkpoint)))
        {
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
                (prepared.id.clone(), before, indices.map(|indices| (indices, writer.checkpoint())))
            })
        };
        let mut befores = Vec::new();
        let mut files = BTreeMap::new();
        let mut failure = None;
        for (id, before, result) in written {
            match result {
                Ok((indices, after)) => {
                    files.insert(id.clone(), Written { indices, after });
                }
                Err(error) => {
                    failure.get_or_insert(error);
                }
            }
            befores.push((id, before));
        }
        if let Some(error) = failure {
            return Err(rollback(telemetry, &befores, error));
        }
        Ok(Unsynced { round: 0, prepared, files, befores })
    }
    /// Sync Written Files Under the Locks, Then Catalog Them.
    fn sync_and_catalog(&self, connection: &mut Connection, telemetry: &mut Telemetry, unsynced: Unsynced) -> Result<()> {
        let work =
            telemetry.writers.iter_mut().filter(|(id, _)| unsynced.files.contains_key(id.as_str())).map(|(_, writer)| writer).collect();
        if let Err(error) = parallel(work, |writer: &mut ipc::Writer| writer.sync()).into_iter().collect::<Result<()>>() {
            return Err(rollback(telemetry, &unsynced.befores, error));
        }
        self.catalog(connection, telemetry, unsynced)
    }
    /// Catalog Synced Files in One Transaction, Then Release Their Claimed Batches.
    ///
    /// Every file is synced before this transaction commits, so a saved position never names rows a
    /// crash could lose; a failure rolls every writer back to its committed prefix.
    fn catalog(&self, connection: &mut Connection, telemetry: &mut Telemetry, unsynced: Unsynced) -> Result<()> {
        let Unsynced { prepared, files, befores, .. } = unsynced;
        if prepared.is_empty() {
            return Ok(());
        }
        let transaction = match connection.transaction() {
            Ok(transaction) => transaction,
            Err(error) => return Err(rollback(telemetry, &befores, error.into())),
        };
        let result = (|| -> Result<()> {
            for prepared in &prepared {
                let file = files.get(&prepared.id).ok_or(Error::Data("restart after an Arrow writer went missing"))?;
                for (chunk, index) in prepared.chunks.iter().zip(&file.indices) {
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
            return Err(rollback(telemetry, &befores, error));
        }
        self.intake()?.release(&prepared)?;
        self.changed.send_modify(|revision| *revision += 1);
        Ok(())
    }
    /// Stop an Execution's Intake, Save Its Pending Rows and Close Its Stream Files.
    pub(crate) fn finalize_samples(&self, connection: &mut Connection, execution: &str) -> Result<()> {
        let closed = {
            let mut intake = self.intake()?;
            let keys = intake.streams.keys().filter(|(owner, _)| owner == execution).cloned().collect::<Vec<_>>();
            keys.into_iter().filter_map(|key| intake.streams.remove_entry(&key)).collect::<Vec<_>>()
        };
        let result = self.flush_samples_locked(connection).and_then(|()| self.close_files(connection, execution));
        if result.is_err() {
            // Streams That Stay Open in the Catalog Keep Accepting Samples.
            self.intake()?.streams.extend(closed);
        }
        result
    }
    fn close_files(&self, connection: &mut Connection, execution: &str) -> Result<()> {
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

/// Restore Each Writer to Its Committed Prefix; Returns the Error That Required It, or the Rollback's Own.
fn rollback(telemetry: &mut Telemetry, befores: &[(String, ipc::Checkpoint)], error: Error) -> Error {
    for (id, before) in befores {
        let restored = match telemetry.writers.get_mut(id) {
            Some(writer) => writer.rollback(before),
            None => Err(Error::Data("restart after an Arrow writer went missing")),
        };
        if let Err(failure) = restored {
            return failure;
        }
    }
    error
}

/// Load Each Claimed File's Saved Statistics From the Catalog.
fn load_statistics(connection: &Connection, claims: Vec<Head>, settings: &StorageSettings) -> Result<Vec<Claim>> {
    claims
        .into_iter()
        .map(
            |Head {
                 key: (execution, stream),
                 codec,
                 head,
                 batches,
             }| {
                let id = catalog::identifier(&execution, &stream, &codec.digest);
                Ok(Claim {
                    key: (execution, stream),
                    statistics: Statistics::load(connection, &id, settings.previews)?,
                    id,
                    codec,
                    head,
                    batches,
                })
            },
        )
        .collect()
}

fn admit(intake: &mut Intake, execution: &str, stream: String, codec: Arc<Codec>, batch: RecordBatch) {
    let pending = intake.pending.entry((execution.into(), stream)).or_insert_with(|| Pending {
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
    use crate::Evidence;
    use crate::telemetry::{config::storage_defaults, previews::Envelope};
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

    /// Every Admitted Row Is Saved Exactly Once and Counted by Its Stored Preview.
    fn require_saved_once(store: &Store, rounds: u64) -> Result<()> {
        let total = rounds * ROWS * STREAMS as u64;
        assert_eq!(store.cut("execution")?.frontier, total);
        let mut positions = store.read("execution")?.samples.iter().map(|sample| sample.position).collect::<Vec<_>>();
        positions.sort_unstable();
        assert_eq!(positions, (1..=total).collect::<Vec<_>>());
        let connection = store.lock()?;
        let mut files = connection
            .prepare("SELECT p.payload,(SELECT SUM(rows) FROM telemetry_batches b WHERE b.file=p.file) FROM telemetry_previews p")?;
        let files = files
            .query_map([], |row| Ok((row.get::<_, Vec<u8>>(0)?, row.get::<_, i64>(1)? as u64)))?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        assert_eq!(files.len(), STREAMS);
        for (preview, rows) in files {
            assert_eq!(rows, rounds * ROWS);
            assert_eq!(postcard::from_bytes::<Envelope>(&preview)?.buckets.values().map(|bucket| bucket.count).sum::<u64>(), rows);
        }
        Ok(())
    }

    #[test]
    fn streams_open_with_cataloged_files_and_recover_an_uncommitted_open() -> Result<()> {
        let root = tempfile::tempdir()?;
        let store = Store::open(root.path())?;
        store.create(&execution())?;
        // A Crash Between Creating a Stream File and Its Catalog Commit Leaves a Header Without an Entry.
        let codec = {
            let connection = store.lock()?;
            store.telemetry.lock().unwrap().codec(&connection, "execution", &stream(0))?
        };
        let path = catalog::path(root.path(), &catalog::identifier("execution", &stream(0), &codec.digest))?;
        std::fs::create_dir_all(path.parent().unwrap())?;
        drop(ipc::Writer::create(&path, codec.schema.clone())?);
        store.open_streams("execution", &(0..STREAMS).map(stream).collect::<Vec<_>>())?;
        let empty: i64 =
            store.lock()?.query_row("SELECT COUNT(*) FROM telemetry_files WHERE batches=0 AND finalized=0", [], |row| row.get(0))?;
        assert_eq!(empty as usize, STREAMS);
        store.sample_batch("execution", &samples(0))?;
        store.flush_samples()?;
        // Opening a Stream Again Is Refused Before It Touches Its Saved File.
        assert!(store.open_stream("execution", &stream(0)).is_err());
        drop(store);
        let store = Store::open(root.path())?;
        require_saved_once(&store, 1)?;
        store.close_streams("execution")?;
        for entry in std::fs::read_dir(root.path().join("telemetry"))? {
            let reader = arrow_ipc::reader::FileReader::try_new(std::fs::File::open(entry?.path())?, None)?;
            assert_eq!(reader.map(|batch| Ok(batch?.num_rows() as u64)).sum::<Result<u64>>()?, ROWS);
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

    /// A Store With the Fixture Execution's Streams Open.
    fn opened(settings: StorageSettings) -> Result<(tempfile::TempDir, Store)> {
        let root = tempfile::tempdir()?;
        let store = Store::open_with(root.path(), settings)?;
        store.create(&execution())?;
        store.open_streams("execution", &(0..STREAMS).map(stream).collect::<Vec<_>>())?;
        Ok((root, store))
    }

    /// Releases the Sync Gate Even When a Story's Assertion Fails, So Its Parked Sync Can Finish.
    struct Released;

    impl Drop for Released {
        fn drop(&mut self) {
            ipc::gate::release();
        }
    }

    /// Run `path` on Its Own Thread, Run `during` Once Its Batch Sync Is Parked, Then Let the Sync Finish.
    fn while_syncing<T: Send>(path: impl FnOnce() -> T + Send, during: impl FnOnce()) -> T {
        ipc::gate::hold();
        std::thread::scope(|scope| {
            let running = scope.spawn(path);
            {
                let _released = Released;
                ipc::gate::parked();
                during();
            }
            running.join().unwrap()
        })
    }

    /// Every Sample Position of an Execution, in Order.
    fn saved_positions(store: &Store, execution: &str) -> Result<Vec<u64>> {
        let mut positions = store.read(execution)?.samples.iter().map(|sample| sample.position).collect::<Vec<_>>();
        positions.sort_unstable();
        Ok(positions)
    }

    #[test]
    fn intake_takes_neither_the_connection_nor_the_flush_lock() -> Result<()> {
        let (_root, store) = opened(storage_defaults())?;
        // Seals, Closures and Catalog Commits Hold These Locks; Admission Proceeds Without Them.
        let connection = store.lock()?;
        let telemetry = store.telemetry.lock().unwrap();
        store.sample_batch("execution", &samples(0))?;
        drop(telemetry);
        drop(connection);
        store.flush_samples()?;
        require_saved_once(&store, 1)
    }

    #[test]
    fn intake_admits_while_a_synchronous_flush_syncs() -> Result<()> {
        let (_root, store) = opened(storage_defaults())?;
        store.sample_batch("execution", &samples(0))?;
        while_syncing(
            || store.flush_samples(),
            || {
                assert!(store.intake.try_lock().is_ok());
                store.sample_batch("execution", &samples(1)).unwrap();
            },
        )?;
        // The Flush Saved Only What It Claimed; Rows Admitted During Its Sync Stay Pending Until the Next.
        assert_eq!(store.cut("execution")?.frontier, ROWS * STREAMS as u64);
        store.flush_samples()?;
        require_saved_once(&store, 2)
    }

    #[test]
    fn evidence_follows_its_samples_while_intake_continues_during_its_sync() -> Result<()> {
        let (_root, store) = opened(storage_defaults())?;
        let round = ROWS * STREAMS as u64;
        store.sample_batch("execution", &samples(0))?;
        let evidence = Evidence {
            kind: "note".into(),
            ..Default::default()
        };
        let position = while_syncing(
            || store.append("execution", evidence),
            || {
                assert!(store.intake.try_lock().is_ok());
                store.sample_batch("execution", &samples(1)).unwrap();
            },
        )?;
        // The Barrier Reserved the Evidence Position First: Earlier Samples Are Saved, Later Ones Follow It.
        assert_eq!(position, round + 1);
        assert_eq!(store.cut("execution")?.frontier, position);
        assert_eq!(store.intake()?.pending.values().map(|pending| pending.rows).sum::<usize>(), round as usize);
        store.flush_samples()?;
        assert_eq!(saved_positions(&store, "execution")?, (1..=round).chain(round + 2..=2 * round + 1).collect::<Vec<_>>());
        Ok(())
    }

    #[test]
    fn finalization_stops_its_intake_while_other_executions_keep_admitting() -> Result<()> {
        let (_root, store) = opened(storage_defaults())?;
        let mut other = execution();
        other.identifier = "other".into();
        store.create(&other)?;
        store.open_streams("other", &(0..STREAMS).map(stream).collect::<Vec<_>>())?;
        store.sample_batch("execution", &samples(0))?;
        while_syncing(
            || store.close_streams("execution"),
            || {
                assert!(store.intake.try_lock().is_ok());
                store.sample_batch("other", &samples(0)).unwrap();
                assert!(store.sample_batch("execution", &samples(1)).is_err());
            },
        )?;
        assert!(store.streams_closed("execution")?);
        require_saved_once(&store, 1)?;
        store.close_streams("other")?;
        assert_eq!(saved_positions(&store, "other")?, (1..=ROWS * STREAMS as u64).collect::<Vec<_>>());
        Ok(())
    }

    #[test]
    fn shutdown_closes_intake_before_its_final_flush() -> Result<()> {
        let (_root, store) = opened(storage_defaults())?;
        store.sample_batch("execution", &samples(0))?;
        let runtime = tokio::runtime::Builder::new_current_thread().build()?;
        while_syncing(
            || runtime.block_on(store.shutdown_samples()),
            || {
                // Rows Arriving During the Final Flush's Sync Are Refused, Not Accepted and Left Unsaved.
                assert!(store.sample_batch("execution", &samples(1)).is_err());
            },
        )?;
        assert!(store.intake()?.pending.is_empty());
        require_saved_once(&store, 1)
    }

    #[test]
    fn byte_pressure_flushes_on_the_admitting_caller_without_the_intake_lock() -> Result<()> {
        let mut settings = storage_defaults();
        settings.pending_bytes = NonZeroUsize::new(1).unwrap();
        let (_root, store) = opened(settings)?;
        while_syncing(
            || store.sample_batch("execution", &samples(0)),
            || {
                // The Pressured Caller Waits for Its Own Flush; Intake State Stays Readable and Its Rows Stay Counted.
                let intake = store.intake.try_lock().unwrap();
                assert_eq!(intake.pending.values().map(|pending| pending.rows).sum::<usize>(), (ROWS * STREAMS as u64) as usize);
            },
        )?;
        require_saved_once(&store, 1)?;
        store.sample_batch("execution", &samples(1))?;
        require_saved_once(&store, 2)
    }

    #[test]
    fn a_round_syncs_without_the_locks_and_the_next_flush_settles_it() -> Result<()> {
        let root = tempfile::tempdir()?;
        let settings = storage_defaults();
        let store = Store::open_with(root.path(), settings)?;
        store.create(&execution())?;
        for index in 0..STREAMS {
            store.open_stream("execution", &stream(index))?;
        }
        // A Round Parked Between Its Writes and Its Sync Holds No Lock: Admission and Reads Proceed.
        store.sample_batch("execution", &samples(0))?;
        let round = store.begin_round()?.expect("pending rows start a round");
        store.sample_batch("execution", &samples(1))?;
        assert_eq!(store.cut("execution")?.frontier, 0);
        // A Synchronous Flush Syncs and Catalogs the Parked Batches Before Its Own; the Round Then Commits Nothing.
        store.flush_samples()?;
        store.finish_round(round)?;
        require_saved_once(&store, 2)?;
        // A Round Left Alone Catalogs Its Own Batches.
        store.sample_batch("execution", &samples(2))?;
        let round = store.begin_round()?.expect("pending rows start a round");
        store.finish_round(round)?;
        require_saved_once(&store, 3)?;
        // A Failed Sync Rolls the Round Back; Its Rows Stay Pending and the Next Flush Saves Them Once.
        store.sample_batch("execution", &samples(3))?;
        store.telemetry.lock().unwrap().writers.values_mut().next().unwrap().fail_next_sync();
        let round = store.begin_round()?.expect("pending rows start a round");
        assert!(store.finish_round(round).is_err());
        store.flush_samples()?;
        require_saved_once(&store, 4)?;
        // A Crash While a Round Is Parked Keeps the Saved Prefix, and Writing Resumes Past the Uncataloged Bytes.
        store.sample_batch("execution", &samples(4))?;
        let round = store.begin_round()?.expect("pending rows start a round");
        drop(round);
        drop(store);
        let store = Store::open_with(root.path(), settings)?;
        require_saved_once(&store, 4)?;
        store.sample_batch("execution", &samples(4))?;
        store.flush_samples()?;
        require_saved_once(&store, 5)
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
            load_statistics(&connection, store.intake()?.claim(None)?, &settings)?
        };
        let prepared = prepare(claims, &settings)?;
        store.flush_samples()?;
        store.sample_batch("execution", &samples(1))?;
        assert!(!prepared.iter().any(|prepared| store.intake().unwrap().current(prepared)));
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
