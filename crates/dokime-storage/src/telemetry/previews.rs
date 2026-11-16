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
use arrow_buffer::NullBuffer;
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};

// Local Imports
use super::aggregates::{self, Values};
use super::config::PreviewSettings;
use crate::{Error, NumericBin, NumericWindow, Store};

/* === Constants === */

pub(crate) const SCHEMA: &str = "CREATE TABLE telemetry_previews(file TEXT NOT NULL REFERENCES telemetry_files(id),field TEXT NOT NULL,through INTEGER NOT NULL,start_ns INTEGER NOT NULL,end_ns INTEGER NOT NULL,encoding INTEGER NOT NULL CHECK(encoding=1),payload BLOB NOT NULL,PRIMARY KEY(file,field));";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct SavedPreview {
    start_ns: i64,
    end_ns: i64,
    payload: Option<Vec<u8>>,
}

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

/// The Time, Position and Validity of One Chunk's Rows.
struct Rows<'a> {
    times: &'a [i64],
    positions: &'a [u64],
    nulls: Option<&'a NullBuffer>,
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
    pub(super) fn empty(settings: PreviewSettings) -> Self {
        Self {
            origin: 0,
            width: settings.bucket_interval.as_nanos() as i128,
            buckets: BTreeMap::new(),
        }
    }

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

impl Rows<'_> {
    fn extend<T: Copy>(&self, envelope: &mut Envelope, values: &[T], number: impl Fn(T) -> f64, maximum: usize) -> Result<(), Error> {
        let point = |row: usize| {
            let value = number(values[row]);
            (self.nulls.is_none_or(|nulls| nulls.is_valid(row)) && value.is_finite()).then(|| Point {
                time: self.times[row] as i128,
                position: self.positions[row],
                value,
            })
        };
        let mut row = 0;
        while row < values.len() {
            let Some(opening) = point(row) else {
                row += 1;
                continue;
            };
            row += 1;
            envelope.push(opening, maximum)?;
            // The Opening Row May Have Coarsened the Grid, So Its Bucket Is Found Again Under the Current One.
            let key = (opening.time - envelope.origin).div_euclid(envelope.width);
            let Some((start, end)) = key
                .checked_mul(envelope.width)
                .and_then(|offset| envelope.origin.checked_add(offset))
                .and_then(|start| Some((start, start.checked_add(envelope.width)?)))
            else {
                continue;
            };
            let bucket = envelope.buckets.get_mut(&key).ok_or(Error::Corrupt("preview bucket is absent after its push"))?;
            while row < values.len() {
                let Some(point) = point(row) else {
                    row += 1;
                    continue;
                };
                if point.time < start || point.time >= end {
                    break;
                }
                bucket.merge(Bucket::new(point))?;
                row += 1;
            }
        }
        Ok(())
    }
}

impl Store {
    fn saved_preview(&self, window: NumericWindow<'_>, with_payload: bool) -> Result<Option<SavedPreview>, Error> {
        if self.has_projection(window)? {
            return Ok(None);
        }
        let connection = self.lock()?;
        let selected:Option<(u64,SavedPreview)>=connection.query_row("SELECT p.through,p.start_ns,p.end_ns,CASE WHEN ?4 THEN p.payload END FROM telemetry_previews p JOIN telemetry_files f ON f.id=p.file WHERE f.execution=?1 AND f.stream=?2 AND p.field=?3",params![window.execution,window.stream,window.field,with_payload],|row|Ok((row.get::<_,i64>(0)? as u64,SavedPreview {start_ns:row.get(1)?,end_ns:row.get(2)?,payload:row.get(3)?}))).optional()?;
        let Some((through, preview)) = selected else {
            return Ok(None);
        };
        if window.after != 0 || window.through < through || window.start_ns > preview.start_ns || window.end_ns < preview.end_ns {
            return Ok(None);
        }
        let accepted = self.accepted_position(&connection, window.execution)?;
        let saved = crate::store::saved_position(&connection, window.execution)?;
        if window.through > saved && accepted > saved {
            return Ok(None);
        }
        Ok(Some(preview))
    }
    pub(crate) fn preview_bounds(&self, window: NumericWindow<'_>) -> Result<Option<(i64, i64)>, Error> {
        Ok(self.saved_preview(window, false)?.map(|preview| (preview.start_ns, preview.end_ns)))
    }
    pub(crate) fn preview_bins(&self, window: NumericWindow<'_>) -> Result<Option<Vec<NumericBin>>, Error> {
        let Some(preview) = self.saved_preview(window, true)? else {
            return Ok(None);
        };
        let mut envelope: Envelope = postcard::from_bytes(&preview.payload.ok_or(Error::Corrupt("preview payload is absent"))?)?;
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

/// Every Saved Preview of One Stream File, by Field.
pub(super) fn load(connection: &Connection, id: &str) -> Result<BTreeMap<String, Envelope>, Error> {
    let mut query = connection.prepare_cached("SELECT field,payload FROM telemetry_previews WHERE file=?1")?;
    query
        .query_map([id], |row| Ok((row.get::<_, String>(0)?, row.get::<_, Vec<u8>>(1)?)))?
        .map(|row| {
            let (field, bytes) = row?;
            Ok((field, postcard::from_bytes::<Envelope>(&bytes)?))
        })
        .collect()
}

/// Save One Field's Preview Through the Last Position of the Latest Chunk That Gave It a Value.
pub(super) fn save(connection: &Connection, id: &str, field: &str, through: u64, envelope: &Envelope) -> Result<(), Error> {
    let start = envelope.buckets.first_key_value().ok_or(Error::Data("preview needs a numeric row"))?.1.first.time as i64;
    let end = envelope.buckets.last_key_value().ok_or(Error::Data("preview needs a numeric row"))?.1.last.time as i64;
    connection.prepare_cached("INSERT INTO telemetry_previews VALUES(?1,?2,?3,?4,?5,1,?6) ON CONFLICT(file,field) DO UPDATE SET through=excluded.through,start_ns=excluded.start_ns,end_ns=excluded.end_ns,encoding=1,payload=excluded.payload")?.execute(params![id,field,through as i64,start,end,postcard::to_allocvec(envelope)?])?;
    Ok(())
}

/// Push One Chunk's Finite Values Into an Envelope in Row Order.
///
/// Each run of rows inside one bucket's time bounds merges straight into that bucket, so the
/// bucket key is divided out once per run rather than once per row. A row outside the bounds,
/// including an earlier time, takes the ordinary push, which inserts and coarsens as before.
pub(super) fn extend(envelope: &mut Envelope, batch: &RecordBatch, field: &str, maximum: usize) -> Result<(), Error> {
    let column = aggregates::numeric(batch, field)?;
    let metadata = batch.column(0).as_any().downcast_ref::<StructArray>().ok_or(Error::Corrupt("storage metadata differs"))?;
    let times = metadata
        .column_by_name("source_ns")
        .and_then(|column| column.as_any().downcast_ref::<Int64Array>())
        .ok_or(Error::Corrupt("time type differs"))?;
    let positions = metadata
        .column_by_name("position")
        .and_then(|column| column.as_any().downcast_ref::<UInt64Array>())
        .ok_or(Error::Corrupt("position type differs"))?;
    let rows = Rows {
        times: times.values(),
        positions: positions.values(),
        nulls: column.nulls.as_ref(),
    };
    match column.values {
        Values::Signed(values) => rows.extend(envelope, values, |value| value as f64, maximum),
        Values::Unsigned(values) => rows.extend(envelope, values, |value| value as f64, maximum),
        Values::Float(values) => rows.extend(envelope, values, |value| value, maximum),
    }
}

/* === Private Functions === */

/* === Tests === */
