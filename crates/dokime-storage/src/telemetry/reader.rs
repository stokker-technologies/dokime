/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Projected Saved and Live Reads

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    collections::BTreeMap,
    fs::File,
    io::BufReader,
    num::NonZeroUsize,
    path::Path,
    result,
    sync::{Arc, Mutex, atomic::Ordering},
};

// Third Party Imports
use arrow_array::{BinaryArray, BooleanArray, Int64Array, RecordBatch, StructArray, UInt64Array};
use arrow_schema::{DataType, Field, Schema};
use arrow_select::filter::filter_record_batch;
use harmos_query::{Table, TableWriter};
use rusqlite::params;

// Local Imports
use super::catalog;
use crate::ipc::BatchIndex;
use crate::{Error, ReadCancellation, Result, Sample, SampleWindow, Store, ipc};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Copy)]
pub struct BatchSelection<'a> {
    pub execution: &'a str,
    pub stream: &'a str,
    pub after: u64,
    pub through: u64,
    pub start_ns: i64,
    pub end_ns: i64,
}

/// Immutable Source Blocks Belong to One Finite Read, Including Its Nested Plot Readers.
#[derive(Default)]
pub(crate) struct SampleReads {
    blocks: Mutex<BTreeMap<(String, u64), Arc<SampleBlock>>>,
}

#[derive(Default)]
struct SampleBlock {
    value: Mutex<Option<Table>>,
}

/// Pending Arrow Batches and Their Accepted Positions at One Read's Saved Snapshot.
pub(crate) struct LiveSamples {
    pub(crate) accepted: BTreeMap<String, u64>,
    batches: BTreeMap<(String, String), Vec<RecordBatch>>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl SampleReads {
    fn batch(
        &self, store: &Store, path: &Path, file: &mut Option<BufReader<File>>, index: &BatchIndex, projection: &ipc::Projection,
        maximum: usize,
    ) -> Result<RecordBatch> {
        store.check_read()?;
        let key = (path.to_string_lossy().into_owned(), index.offset);
        let block = {
            let mut blocks = self.blocks.lock().map_err(|_| Error::Data("start a new read after its source block lock failed"))?;
            blocks.entry(key).or_insert_with(|| Arc::new(SampleBlock::default())).clone()
        };
        let table = {
            let mut value = block.value.lock().map_err(|_| Error::Data("start a new read after its source block load failed"))?;
            store.check_read()?;
            if value.is_none() {
                let _span = tracing::debug_span!("source.block", samples = index.rows).entered();
                let buffer = ipc::read_query_buffer(opened(file, path)?, index, maximum)?;
                // Spill Retained Blocks So Earlier Blocks Do Not Accumulate Beyond the Query Working Budget.
                let schema = Arc::new(Schema::new(vec![Field::new("block", DataType::Binary, false)]));
                let batch = RecordBatch::try_new(schema.clone(), vec![Arc::new(BinaryArray::from_iter_values([buffer.as_slice()]))])?;
                let mut writer = TableWriter::new(schema)?;
                writer.push(&batch, &mut || {
                    store.check_read().map_err(|_| harmos_query::Error::InvalidInput {
                        field: "query",
                        reason: "start a new read after cancellation",
                    })
                })?;
                *value = Some(writer.finish()?);
            }
            value.as_ref().unwrap().clone()
        };
        let mut reader = table.reader()?;
        let batch = reader
            .next(&mut || {
                store.check_read().map_err(|_| harmos_query::Error::InvalidInput {
                    field: "query",
                    reason: "start a new read after cancellation",
                })
            })?
            .ok_or(Error::Corrupt("retained source block is absent"))?;
        let bytes = batch.column(0).as_any().downcast_ref::<BinaryArray>().ok_or(Error::Corrupt("retained source block layout differs"))?;
        let offsets = bytes.value_offsets();
        let buffer = bytes.values().slice_with_length(offsets[0] as usize, (offsets[1] - offsets[0]) as usize);
        ipc::decode_query_buffer(&buffer, index, projection, maximum)
    }
}

impl Store {
    /// Capture Saved and Pending Samples Together Without Flushing or Losing Read Cancellation.
    pub fn live_read_session(&self) -> Result<(Self, ReadCancellation)> {
        let _connection = self.lock()?;
        let (mut session, cancellation) = self.read_session()?;
        // Establish SQLite's Snapshot Before a Flush Can Catalog and Release Its Pending Batches.
        session.lock()?.query_row("SELECT COUNT(*) FROM executions", [], |_| Ok(()))?;
        let intake = self.intake()?;
        session.live_samples = Some(Arc::new(LiveSamples {
            accepted: intake.accepted.clone(),
            batches: intake.pending.iter().map(|(key, pending)| (key.clone(), pending.batches.clone())).collect(),
        }));
        Ok((session, cancellation))
    }

    pub(super) fn pending_batches(&self, execution: &str, stream: &str) -> Result<Vec<RecordBatch>> {
        let key = (execution.to_owned(), stream.to_owned());
        if let Some(samples) = &self.live_samples {
            return Ok(samples.batches.get(&key).cloned().unwrap_or_default());
        }
        if self.cancelled.is_some() {
            return Ok(Vec::new());
        }
        Ok(self.intake()?.pending.get(&key).map(|pending| pending.batches.clone()).unwrap_or_default())
    }

    /// Retain Original Source Blocks Through the Library's Bounded Spill Tables for This Request Only.
    pub fn reuse_sample_reads(&mut self) -> Result<()> {
        if self.cancelled.is_none() {
            return Err(Error::Data("start a finite read before retaining its sample blocks"));
        }
        self.sample_reads.get_or_insert_with(|| Arc::new(SampleReads::default()));
        Ok(())
    }

    /// Isolate Mutable Expression Bindings While Sharing This Request's Cut, Cancellation and Results.
    pub fn independent_projection(&self) -> Self {
        let mut reader = self.clone();
        reader.projection = Arc::new(Mutex::new(None));
        reader
    }

    pub fn sample_frontier(&self, execution: &str, stream: &str, after: u64, through: u64) -> Result<u64> {
        let connection = self.lock()?;
        let (saved,partial):(u64,Option<u64>)=connection.query_row("SELECT COALESCE(MAX(CASE WHEN b.last_position<=?4 THEN b.last_position END),0),MIN(CASE WHEN b.last_position>?4 THEN b.first_position END) FROM telemetry_batches b JOIN telemetry_files f ON f.id=b.file WHERE f.execution=?1 AND f.stream=?2 AND b.last_position>?3 AND b.first_position<=?4",params![execution,stream,after as i64,through.min(i64::MAX as u64) as i64],|row|Ok((row.get::<_,i64>(0)? as u64,row.get::<_,Option<i64>>(1)?.map(|v|v as u64))))?;
        let pending = self.pending_batches(execution, stream)?;
        let pending = pending
            .iter()
            .flat_map(|batch| positions(batch).values().iter().copied())
            .filter(|position| *position > after && *position <= through)
            .max()
            .unwrap_or(0);
        drop(connection);
        let mut frontier = saved.max(pending);
        if let Some(first) = partial {
            self.read_batches(
                BatchSelection {
                    execution,
                    stream,
                    after: after.max(first.saturating_sub(1)),
                    through,
                    start_ns: i64::MIN,
                    end_ns: i64::MAX,
                },
                &[],
                |batch| {
                    let metadata = batch.column(0).as_any().downcast_ref::<StructArray>().unwrap();
                    let positions = metadata.column_by_name("position").unwrap().as_any().downcast_ref::<UInt64Array>().unwrap();
                    frontier = frontier.max(positions.values().iter().copied().max().unwrap_or(0));
                    Ok(true)
                },
            )?;
        }
        Ok(frontier)
    }
    pub fn read_batches(
        &self, selection: BatchSelection<'_>, columns: &[String], mut read: impl FnMut(RecordBatch) -> Result<bool>,
    ) -> Result<()> {
        let span = tracing::debug_span!("source.read", stream = selection.stream, samples = tracing::field::Empty);
        let _entered = span.enter();
        let mut samples = 0;
        self.ordered_batches(selection, columns, false, |batch, _| {
            samples += batch.num_rows() as u64;
            read(batch)
        })?;
        span.record("samples", samples);
        Ok(())
    }
    pub(crate) fn ordered_batches(
        &self, selection: BatchSelection<'_>, columns: &[String], latest: bool, mut read: impl FnMut(RecordBatch, i64) -> Result<bool>,
    ) -> Result<()> {
        let connection = self.lock()?;
        let codec = self.codec(&connection, selection.execution, selection.stream)?;
        let id = catalog::identifier(selection.execution, selection.stream, &codec.digest);
        let order = if latest { "b.end_ns DESC,b.first_position DESC" } else { "b.first_position" };
        let mut query=connection.prepare(&format!("SELECT b.offset,b.metadata_bytes,b.body_bytes,b.rows,b.sha256,b.end_ns,b.first_position>?3 AND b.last_position<=?4 AND b.start_ns>=?6 AND b.end_ns<=?5 FROM telemetry_batches b JOIN telemetry_files f ON f.id=b.file WHERE b.file=?1 AND f.schema_hash=?2 AND b.last_position>?3 AND b.first_position<=?4 AND b.start_ns<=?5 AND b.end_ns>=?6 ORDER BY {order}"))?;
        let indices = query
            .query_map(
                params![
                    id,
                    codec.digest,
                    selection.after as i64,
                    selection.through.min(i64::MAX as u64) as i64,
                    selection.end_ns,
                    selection.start_ns
                ],
                |row| Ok((catalog::index(row)?, row.get::<_, i64>(5)?, row.get::<_, bool>(6)?)),
            )?
            .collect::<result::Result<Vec<_>, _>>()?;
        drop(query);
        let batches = self.pending_batches(selection.execution, selection.stream)?;
        let tail = batches.iter().map(|batch| tail(batch, &selection)).collect::<Result<Vec<_>>>()?;
        let maximum = self.settings.query_bytes.get();
        drop(connection);
        let keys = if columns.is_empty() {
            match codec.schema.field(1).data_type() {
                DataType::Struct(fields) => fields.iter().map(|f| f.name().clone()).collect(),
                _ => return Err(Error::Corrupt("values layout differs")),
            }
        } else {
            columns.iter().map(|key| key.split('.').next().unwrap().to_owned()).collect::<Vec<_>>()
        };
        for column in columns {
            let mut kind = codec.schema.field(1).data_type();
            for component in column.split('.') {
                let DataType::Struct(fields) = kind else {
                    return Err(Error::Data("select a scalar field path from the frozen stream"));
                };
                kind = fields
                    .iter()
                    .find(|f| f.name() == component)
                    .ok_or(Error::Data("declare every projected column in the frozen stream"))?
                    .data_type();
            }
        }
        let projection = ipc::Projection::new(codec.schema.clone(), &["position", "source_ns", "receipt_ns", "sequence"], &keys)?;
        let path = catalog::path(self.database.parent().unwrap(), &id)?;
        let mut pending = Vec::new();
        let rows = self.storage_settings().flush_rows.get();
        let chunks = tail
            .iter()
            .filter(|batch| batch.num_rows() > 0)
            .flat_map(|batch| {
                (0..batch.num_rows()).step_by(rows).map(move |offset| batch.slice(offset, rows.min(batch.num_rows() - offset)))
            })
            .collect::<Vec<_>>();
        for batch in chunks {
            let end_ns = super::writer::span(&batch)?.3;
            let values = batch.column(1).as_any().downcast_ref::<StructArray>().unwrap();
            let selected = values
                .fields()
                .iter()
                .enumerate()
                .filter(|(_, f)| keys.contains(f.name()))
                .map(|(i, f)| (f.clone(), values.column(i).clone()))
                .collect::<Vec<_>>();
            let values =
                if selected.is_empty() { StructArray::new_empty_fields(batch.num_rows(), None) } else { StructArray::from(selected) };
            let batch = RecordBatch::try_from_iter(vec![("_storage", batch.column(0).clone()), ("values", Arc::new(values) as _)])?;
            pending.push((batch, end_ns));
        }
        if latest {
            for (batch, _) in pending.drain(..) {
                self.check_read()?;
                if !read(batch, i64::MAX)? {
                    return Ok(());
                }
            }
        }
        let mut file = None;
        for (index, end_ns, inside) in indices {
            self.check_read()?;
            let batch = match &self.sample_reads {
                Some(reads) => reads.batch(self, &path, &mut file, &index, &projection, maximum)?,
                None => ipc::read_query_batch(opened(&mut file, &path)?, &index, &projection, maximum)?,
            };
            // A Chunk Wholly Inside the Selection Needs No Row Filter.
            if inside {
                if !read(batch, end_ns)? {
                    return Ok(());
                }
                continue;
            }
            let storage = batch.column(0).as_any().downcast_ref::<StructArray>().ok_or(Error::Corrupt("metadata layout differs"))?;
            let positions = storage.column_by_name("position").unwrap().as_any().downcast_ref::<UInt64Array>().unwrap();
            let times = storage.column_by_name("source_ns").unwrap().as_any().downcast_ref::<Int64Array>().unwrap();
            let keep = BooleanArray::from_iter((0..batch.num_rows()).map(|row| {
                Some(
                    positions.value(row) > selection.after
                        && positions.value(row) <= selection.through
                        && times.value(row) >= selection.start_ns
                        && times.value(row) <= selection.end_ns,
                )
            }));
            let batch = filter_record_batch(&batch, &keep)?;
            if batch.num_rows() > 0 && !read(batch, end_ns)? {
                return Ok(());
            }
        }
        for (batch, end_ns) in pending {
            self.check_read()?;
            if !read(batch, end_ns)? {
                break;
            }
        }
        Ok(())
    }
    pub(crate) fn check_read(&self) -> Result<()> {
        if self.cancelled.as_ref().is_some_and(|flag| flag.load(Ordering::Acquire)) {
            return Err(Error::Data("start a new read after cancelling the query"));
        }
        Ok(())
    }
    pub fn sample_window(&self, window: SampleWindow<'_>) -> Result<Vec<Sample>> {
        let names = if window.streams.is_empty() { self.stream_names(window.execution)? } else { window.streams.to_vec() };
        let mut samples = BTreeMap::new();
        for stream in names {
            let codec = self.codec(&*self.lock()?, window.execution, &stream)?;
            let after = window.after.iter().find(|(name, _)| name == &stream).map_or(0, |(_, after)| *after);
            self.read_batches(
                BatchSelection {
                    execution: window.execution,
                    stream: &stream,
                    after,
                    through: window.through,
                    start_ns: window.start_ns,
                    end_ns: window.end_ns,
                },
                window.fields,
                |batch| {
                    for row in 0..batch.num_rows() {
                        let sample = codec.sample(&stream, &batch, row)?;
                        samples.insert(sample.position, sample);
                        if samples.len() > window.limit.get() {
                            if window.latest {
                                samples.pop_first();
                            } else {
                                samples.pop_last();
                            }
                        }
                    }
                    Ok(window.latest || samples.len() < window.limit.get())
                },
            )?;
        }
        Ok(samples.into_values().collect())
    }
    pub fn live_sample_window(&self, mut window: SampleWindow<'_>) -> Result<Vec<Sample>> {
        window.through = self.accepted_frontier(window.execution)?;
        self.sample_window(window)
    }
    pub fn accepted_frontier(&self, execution: &str) -> Result<u64> {
        self.accepted_position(&*self.lock()?, execution)
    }
    pub(crate) fn all_samples(&self, execution: &str, through: u64) -> Result<Vec<Sample>> {
        self.sample_window(SampleWindow {
            execution,
            streams: &[],
            fields: &[],
            after: &[],
            through,
            start_ns: i64::MIN,
            end_ns: i64::MAX,
            latest: false,
            limit: NonZeroUsize::new(usize::MAX).unwrap(),
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */

/// A Selection's Stream File, Opened on Its First Chunk Read From Disk; Later Chunks Share the Handle.
fn opened<'a>(file: &'a mut Option<BufReader<File>>, path: &Path) -> Result<&'a mut BufReader<File>> {
    Ok(match file {
        Some(file) => file,
        None => file.insert(BufReader::new(File::open(path)?)),
    })
}

fn positions(batch: &RecordBatch) -> &UInt64Array {
    let storage = batch.column(0).as_any().downcast_ref::<StructArray>().expect("pending rows keep their storage layout");
    storage.column_by_name("position").unwrap().as_any().downcast_ref::<UInt64Array>().unwrap()
}

/// The Accepted Rows of One Pending Batch Inside a Selection's Positions and Source Times.
pub(super) fn tail(batch: &RecordBatch, selection: &BatchSelection<'_>) -> Result<RecordBatch> {
    let storage = batch.column(0).as_any().downcast_ref::<StructArray>().ok_or(Error::Corrupt("metadata layout differs"))?;
    let times = storage.column_by_name("source_ns").unwrap().as_any().downcast_ref::<Int64Array>().unwrap();
    let selected = positions(batch)
        .values()
        .iter()
        .zip(times.values().iter())
        .map(|(position, time)| {
            *position > selection.after && *position <= selection.through && (selection.start_ns..=selection.end_ns).contains(time)
        })
        .collect::<BooleanArray>();
    if selected.true_count() == batch.num_rows() {
        return Ok(batch.clone());
    }
    Ok(filter_record_batch(batch, &selected)?)
}

/* === Tests === */
