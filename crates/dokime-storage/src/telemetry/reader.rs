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
use std::{collections::BTreeMap, num::NonZeroUsize};

// Third Party Imports
use arrow_array::{BooleanArray, Int64Array, RecordBatch, StructArray, UInt64Array};
use arrow_schema::DataType;
use arrow_select::filter::filter_record_batch;
use rusqlite::params;

// Local Imports
use super::catalog;
use crate::{Error, Result, Sample, SampleWindow, Store, ipc};

use std::sync::atomic::Ordering;

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

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Store {
    pub fn sample_frontier(&self, execution: &str, stream: &str, after: u64, through: u64) -> Result<u64> {
        let connection = self.lock()?;
        let (saved,partial):(u64,Option<u64>)=connection.query_row("SELECT COALESCE(MAX(CASE WHEN b.last_position<=?4 THEN b.last_position END),0),MIN(CASE WHEN b.last_position>?4 THEN b.first_position END) FROM telemetry_batches b JOIN telemetry_files f ON f.id=b.file WHERE f.execution=?1 AND f.stream=?2 AND b.last_position>?3 AND b.first_position<=?4",params![execution,stream,after as i64,through.min(i64::MAX as u64) as i64],|row|Ok((row.get::<_,i64>(0)? as u64,row.get::<_,Option<i64>>(1)?.map(|v|v as u64))))?;
        let telemetry = self.telemetry.lock().map_err(|_| Error::Data("restart after the telemetry reader lock failed"))?;
        let pending = if self.cancelled.is_none() {
            telemetry
                .pending
                .get(&(execution.into(), stream.into()))
                .and_then(|p| p.samples.iter().filter(|s| s.position > after && s.position <= through).map(|s| s.position).max())
                .unwrap_or(0)
        } else {
            0
        };
        drop(telemetry);
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
        self.ordered_batches(selection, columns, false, |batch, _| read(batch))
    }
    pub(crate) fn ordered_batches(
        &self, selection: BatchSelection<'_>, columns: &[String], latest: bool, mut read: impl FnMut(RecordBatch, i64) -> Result<bool>,
    ) -> Result<()> {
        let connection = self.lock()?;
        let mut telemetry = self.telemetry.lock().map_err(|_| Error::Data("restart after the telemetry reader lock failed"))?;
        let codec = telemetry.codec(&connection, selection.execution, selection.stream)?;
        let id = catalog::identifier(selection.execution, selection.stream, &codec.digest);
        let order = if latest { "b.end_ns DESC,b.first_position DESC" } else { "b.first_position" };
        let mut query=connection.prepare(&format!("SELECT b.offset,b.metadata_bytes,b.body_bytes,b.rows,b.sha256,b.end_ns FROM telemetry_batches b JOIN telemetry_files f ON f.id=b.file WHERE b.file=?1 AND f.schema_hash=?2 AND b.last_position>?3 AND b.first_position<=?4 AND b.start_ns<=?5 AND b.end_ns>=?6 ORDER BY {order}"))?;
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
                |row| Ok((catalog::index(row)?, row.get::<_, i64>(5)?)),
            )?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        drop(query);
        let tail = if self.cancelled.is_none() {
            telemetry
                .pending
                .get(&(selection.execution.into(), selection.stream.into()))
                .map(|pending| {
                    pending
                        .samples
                        .iter()
                        .filter(|sample| {
                            sample.position > selection.after
                                && sample.position <= selection.through
                                && sample.source_ns >= selection.start_ns
                                && sample.source_ns <= selection.end_ns
                        })
                        .cloned()
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default()
        } else {
            Vec::new()
        };
        let maximum = telemetry.settings.query_bytes.get();
        drop(telemetry);
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
        for rows in tail.chunks(self.storage_settings()?.flush_rows.get()) {
            let batch = codec.batch(rows)?;
            let values = batch.column(1).as_any().downcast_ref::<StructArray>().unwrap();
            let selected = values
                .fields()
                .iter()
                .enumerate()
                .filter(|(_, f)| keys.contains(f.name()))
                .map(|(i, f)| (f.clone(), values.column(i).clone()))
                .collect::<Vec<_>>();
            let values = if selected.is_empty() { StructArray::new_empty_fields(rows.len(), None) } else { StructArray::from(selected) };
            let batch =
                RecordBatch::try_from_iter(vec![("_storage", batch.column(0).clone()), ("values", std::sync::Arc::new(values) as _)])?;
            pending.push((batch, rows.iter().map(|row| row.source_ns).max().unwrap()));
        }
        if latest {
            for (batch, _) in pending.drain(..) {
                self.check_read()?;
                if !read(batch, i64::MAX)? {
                    return Ok(());
                }
            }
        }
        for (index, end_ns) in indices {
            self.check_read()?;
            let batch = ipc::read_query_batch(&path, &index, &projection, maximum)?;
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
            let codec = {
                let connection = self.lock()?;
                self.telemetry.lock().map_err(|_| Error::Data("restart after the telemetry reader lock failed"))?.codec(
                    &connection,
                    window.execution,
                    &stream,
                )?
            };
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
                &[],
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
        Ok(crate::store::next_position(&*self.lock()?, execution)?.saturating_sub(1))
    }
    pub(crate) fn all_samples(&self, execution: &str, through: u64) -> Result<Vec<Sample>> {
        self.sample_window(SampleWindow {
            execution,
            streams: &[],
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

/* === Tests === */
