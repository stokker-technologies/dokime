/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Durable Adaptive Time Envelopes

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::BTreeMap;

// Third Party Imports
use arrow_array::{Int64Array, RecordBatch, StructArray, UInt64Array};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};

// Local Imports
use super::config::PreviewSettings;
use crate::{Error, NumericBin, NumericWindow, Store, scalar};

/* === Constants === */

pub(crate) const SCHEMA: &str = "CREATE TABLE telemetry_previews(file TEXT NOT NULL REFERENCES telemetry_files(id),field TEXT NOT NULL,through INTEGER NOT NULL,payload BLOB NOT NULL,PRIMARY KEY(file,field));";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct Point {
    pub(crate) time: i128,
    pub(crate) position: u64,
    pub(crate) value: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Bucket {
    pub(crate) first: Point,
    pub(crate) last: Point,
    pub(crate) min: Point,
    pub(crate) max: Point,
    pub(crate) count: u64,
    pub(crate) mean: f64,
    pub(crate) squares: f64,
    pub(crate) scale: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Envelope {
    pub(crate) origin: i128,
    pub(crate) width: i128,
    pub(crate) buckets: BTreeMap<i128, Bucket>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Bucket {
    fn new(point: Point) -> Self {
        Self {
            first: point,
            last: point,
            min: point,
            max: point,
            count: 1,
            mean: point.value,
            squares: if point.value == 0.0 { 0.0 } else { 1.0 },
            scale: point.value.abs(),
        }
    }

    fn merge(&mut self, other: Self) -> Result<(), Error> {
        let order = |point: Point| (point.time, point.position);
        if order(other.first) < order(self.first) {
            self.first = other.first;
        }
        if order(other.last) > order(self.last) {
            self.last = other.last;
        }
        if other.min.value < self.min.value {
            self.min = other.min;
        }
        if other.max.value > self.max.value {
            self.max = other.max;
        }
        let total = self.count + other.count;
        let scale = self.scale.max(other.scale);
        let a = if scale == 0.0 { 0.0 } else { self.scale / scale };
        let b = if scale == 0.0 { 0.0 } else { other.scale / scale };
        let left = if scale == 0.0 { 0.0 } else { self.mean / scale };
        let right = if scale == 0.0 { 0.0 } else { other.mean / scale };
        self.mean = (left + (right - left) * other.count as f64 / total as f64).clamp(-1.0, 1.0) * scale;
        self.squares = self.squares * a * a + other.squares * b * b;
        self.scale = scale;
        self.count = self.count.checked_add(other.count).ok_or(Error::Data("preview count overflow"))?;
        Ok(())
    }
}

impl Envelope {
    pub(crate) fn fit(&mut self, maximum: usize) -> Result<(), Error> {
        while self.buckets.len() > maximum {
            self.coarsen()?;
        }
        Ok(())
    }

    fn coarsen(&mut self) -> Result<(), Error> {
        let first = self.buckets.first_key_value().map(|(key, _)| *key).unwrap_or_default();
        self.origin = self
            .origin
            .checked_add(first.checked_mul(self.width).ok_or(Error::Data("preview origin overflow"))?)
            .ok_or(Error::Data("preview origin overflow"))?;
        self.width = self.width.checked_mul(2).ok_or(Error::Data("preview time range overflow"))?;
        for (key, bucket) in std::mem::take(&mut self.buckets) {
            match self.buckets.entry((key - first).div_euclid(2)) {
                std::collections::btree_map::Entry::Vacant(entry) => {
                    entry.insert(bucket);
                }
                std::collections::btree_map::Entry::Occupied(mut entry) => entry.get_mut().merge(bucket)?,
            }
        }
        Ok(())
    }

    pub(crate) fn push(&mut self, point: Point, maximum: usize) -> Result<(), Error> {
        if !point.value.is_finite() {
            return Err(Error::Data("preview value is not finite"));
        }
        let key = (point.time - self.origin).div_euclid(self.width);
        match self.buckets.entry(key) {
            std::collections::btree_map::Entry::Vacant(entry) => {
                entry.insert(Bucket::new(point));
            }
            std::collections::btree_map::Entry::Occupied(mut entry) => entry.get_mut().merge(Bucket::new(point))?,
        }
        self.fit(maximum)
    }
}

impl Store {
    pub(crate) fn preview_bins(&self, window: NumericWindow<'_>) -> Result<Option<Vec<NumericBin>>, Error> {
        if self.has_projection(window)? {
            return Ok(None);
        }
        let connection = self.lock()?;
        let selected:Option<(u64,Vec<u8>)>=connection.query_row("SELECT p.through,p.payload FROM telemetry_previews p JOIN telemetry_files f ON f.id=p.file WHERE f.execution=?1 AND f.stream=?2 AND p.field=?3",params![window.execution,window.stream,window.field],|row|Ok((row.get::<_,i64>(0)? as u64,row.get(1)?))).optional()?;
        let Some((through, bytes)) = selected else {
            return Ok(None);
        };
        if window.after != 0 || window.through < through {
            return Ok(None);
        }
        let accepted = crate::store::next_position(&connection, window.execution)?.saturating_sub(1);
        let saved = crate::store::saved_position(&connection, window.execution)?;
        if window.through > saved && accepted > saved {
            return Ok(None);
        }
        let mut envelope: Envelope = serde_json::from_slice(&bytes)?;
        if envelope.buckets.values().any(|b| b.first.time < i128::from(window.start_ns) || b.last.time > i128::from(window.end_ns)) {
            return Ok(None);
        }
        envelope.fit(window.bins.get())?;
        Ok(Some(
            envelope
                .buckets
                .values()
                .map(|b| NumericBin {
                    start_ns: b.first.time as i64,
                    end_ns: b.last.time as i64,
                    count: b.count,
                    first_ns: b.first.time as i64,
                    last_ns: b.last.time as i64,
                    min_ns: b.min.time as i64,
                    max_ns: b.max.time as i64,
                    min: b.min.value,
                    max: b.max.value,
                    mean: b.mean,
                    scaled_squares: b.squares,
                    scale: b.scale,
                    first: b.first.value,
                    last: b.last.value,
                })
                .collect(),
        ))
    }
    pub(crate) fn has_projection(&self, window: NumericWindow<'_>) -> Result<bool, Error> {
        let plan = self.projection.lock().map_err(|_| Error::Data("restart after the projection lock failed"))?;
        Ok(plan.as_ref().is_some_and(|p| {
            p.cut.execution.identifier == window.execution
                && p.stream == window.stream
                && (!p.filters.is_empty() || p.fields.contains_key(window.field) || p.group.is_some())
        }))
    }
}

/* === Public Functions === */

pub(super) fn append(
    connection: &Connection, id: &str, field: &str, batch: &RecordBatch, through: u64, settings: PreviewSettings,
) -> Result<(), Error> {
    let previous: Option<Vec<u8>> = connection
        .query_row("SELECT payload FROM telemetry_previews WHERE file=?1 AND field=?2", params![id, field], |row| row.get(0))
        .optional()?;
    let mut envelope = previous.map(|bytes| serde_json::from_slice::<Envelope>(&bytes)).transpose()?.unwrap_or(Envelope {
        origin: 0,
        width: settings.bucket_interval.as_nanos() as i128,
        buckets: BTreeMap::new(),
    });
    let column = scalar::column(batch, field)?;
    let metadata = batch.column(0).as_any().downcast_ref::<StructArray>().unwrap();
    let times = metadata.column_by_name("source_ns").unwrap().as_any().downcast_ref::<Int64Array>().unwrap();
    let positions = metadata.column_by_name("position").unwrap().as_any().downcast_ref::<UInt64Array>().unwrap();
    for row in 0..batch.num_rows() {
        if let Some(value) = scalar::number(&column.value(row)?).filter(|v| v.is_finite()) {
            envelope.push(
                Point {
                    time: times.value(row) as i128,
                    position: positions.value(row),
                    value,
                },
                settings.bucket_count.get(),
            )?;
        }
    }
    connection.execute("INSERT INTO telemetry_previews VALUES(?1,?2,?3,?4) ON CONFLICT(file,field) DO UPDATE SET through=excluded.through,payload=excluded.payload",params![id,field,through as i64,serde_json::to_vec(&envelope)?])?;
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
