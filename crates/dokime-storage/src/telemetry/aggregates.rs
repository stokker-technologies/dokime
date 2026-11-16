/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Durable Moments and Exact Query Statistics

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    collections::BTreeMap,
    io::{BufReader, BufWriter, Read, Seek, Write},
    num::NonZeroU128,
};

// Third Party Imports
use arrow_array::{Array, Float64Array, Int64Array, RecordBatch, StructArray, UInt64Array};
use arrow_buffer::NullBuffer;
use arrow_schema::DataType;
use arrow_schema::Fields;
use harmos_query::scalar::ExactNumber;
use num_bigint::BigInt;
use num_traits::ToPrimitive;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};

// Local Imports
use super::config::PreviewSettings;
use super::previews::{self, Envelope};
use crate::{
    Error, NumericWindow, Result, Store,
    scalar::{self, ScalarValue},
};

/* === Constants === */

pub(crate) const SCHEMA:&str="
CREATE TABLE telemetry_summaries(file TEXT NOT NULL REFERENCES telemetry_files(id),field TEXT NOT NULL,moments BLOB NOT NULL,PRIMARY KEY(file,field));
CREATE TABLE telemetry_checkpoints(file TEXT NOT NULL REFERENCES telemetry_files(id),offset INTEGER NOT NULL,field TEXT NOT NULL,first_position INTEGER NOT NULL,last_position INTEGER NOT NULL,start_ns INTEGER NOT NULL,end_ns INTEGER NOT NULL,moments BLOB NOT NULL,PRIMARY KEY(file,offset,field));
";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct Moments {
    pub count: u64,
    pub mean: f64,
    pub m2: f64,
    pub squares: f64,
    pub scale: f64,
    pub min: ScalarValue,
    pub max: ScalarValue,
    pub first: ScalarValue,
    pub last: ScalarValue,
    pub total: Option<ExactNumber>,
    integer_squares: Option<BigInt>,
}

/// One Numeric Value Column of a Stored Batch: Its Typed Values and the Rows That Are Null.
pub(super) struct Numeric<'a> {
    pub(super) values: Values<'a>,
    pub(super) nulls: Option<NullBuffer>,
}

/// A Stream File's Running Summaries and Previews, Advanced Chunk by Chunk Apart From the Catalog.
///
/// They load from the catalog when a flush claims the file and save in its commit, so chunks fold
/// in without catalog access and in the same order and arithmetic as a catalog round trip each.
pub(super) struct Statistics {
    settings: PreviewSettings,
    summaries: BTreeMap<String, Moments>,
    previews: BTreeMap<String, Envelope>,
    /// The Last Position of the Latest Chunk That Gave Each Field a Value.
    touched: BTreeMap<String, u64>,
}

/* === Enums === */

pub(super) enum Values<'a> {
    Signed(&'a [i64]),
    Unsigned(&'a [u64]),
    Float(&'a [f64]),
}

/* === Traits === */

/* === Implementations === */

impl Moments {
    fn empty() -> Self {
        Self {
            count: 0,
            mean: 0.0,
            m2: 0.0,
            squares: 0.0,
            scale: 0.0,
            min: ScalarValue::Missing,
            max: ScalarValue::Missing,
            first: ScalarValue::Missing,
            last: ScalarValue::Missing,
            total: Some(ExactNumber::new(false, 0, NonZeroU128::MIN)),
            integer_squares: Some(BigInt::from(0)),
        }
    }
    fn push(&mut self, value: ScalarValue) -> Result<()> {
        let integer = match value {
            ScalarValue::Signed(value) => Some(i128::from(value)),
            ScalarValue::Unsigned(value) => Some(i128::from(value)),
            _ => None,
        };
        if let Some(integer) = integer {
            if self.count > 0 && self.integer_squares.is_none() {
                return Err(Error::Data("keep each numeric column's type stable"));
            }
            if self.count == 0 {
                self.min = value.clone();
                self.max = value.clone();
                self.first = value.clone();
            }
            self.count += 1;
            if value.compare(&self.min).map_err(|e| Error::Invalid(e.to_string()))?.is_lt() {
                self.min = value.clone();
            }
            if value.compare(&self.max).map_err(|e| Error::Invalid(e.to_string()))?.is_gt() {
                self.max = value.clone();
            }
            self.last = value;
            let exact = ExactNumber::new(integer < 0, integer.unsigned_abs(), NonZeroU128::MIN);
            self.total = Some(self.total.unwrap().checked_add(exact).map_err(|e| Error::Invalid(e.to_string()))?);
            let integer = BigInt::from(integer);
            *self.integer_squares.as_mut().unwrap() += &integer * &integer;
            return Ok(());
        }
        let Some(number) = scalar::number(&value).filter(|v| v.is_finite()) else {
            return Ok(());
        };
        if self.count > 0 && self.integer_squares.is_some() {
            return Err(Error::Data("keep each numeric column's type stable"));
        }
        self.integer_squares = None;
        if self.count == 0 {
            self.min = value.clone();
            self.max = value.clone();
            self.first = value.clone();
        }
        let scale = self.scale.max(number.abs());
        let factor = if scale == 0.0 { 0.0 } else { self.scale / scale };
        let previous = if scale == 0.0 { 0.0 } else { self.mean / scale };
        let normalized = if scale == 0.0 { 0.0 } else { number / scale };
        self.m2 *= factor * factor;
        self.squares *= factor * factor;
        self.scale = scale;
        self.count += 1;
        let delta = normalized - previous;
        let mean = previous + delta / self.count as f64;
        self.mean = mean.clamp(-1.0, 1.0) * scale;
        self.m2 += delta * (normalized - mean);
        self.squares += normalized * normalized;
        if value.compare(&self.min).map_err(|e| Error::Invalid(e.to_string()))?.is_lt() {
            self.min = value.clone();
        }
        if value.compare(&self.max).map_err(|e| Error::Invalid(e.to_string()))?.is_gt() {
            self.max = value.clone();
        }
        self.last = value.clone();
        self.total = match (&value, self.total) {
            (ScalarValue::Signed(v), Some(total)) => Some(
                total
                    .checked_add(ExactNumber::new(*v < 0, v.unsigned_abs() as u128, NonZeroU128::MIN))
                    .map_err(|e| Error::Invalid(e.to_string()))?,
            ),
            (ScalarValue::Unsigned(v), Some(total)) => {
                Some(total.checked_add(ExactNumber::new(false, *v as u128, NonZeroU128::MIN)).map_err(|e| Error::Invalid(e.to_string()))?)
            }
            _ => None,
        };
        Ok(())
    }
    fn merge(&mut self, other: &Self) -> Result<()> {
        if other.count == 0 {
            return Ok(());
        }
        if self.count == 0 {
            *self = other.clone();
            return Ok(());
        }
        if self.integer_squares.is_some() != other.integer_squares.is_some() {
            return Err(Error::Data("keep each numeric column's type stable"));
        }
        if let (Some(left), Some(right)) = (&mut self.integer_squares, &other.integer_squares) {
            *left += right;
        }
        let count = self.count + other.count;
        let scale = self.scale.max(other.scale);
        let a = if scale == 0.0 { 0.0 } else { self.scale / scale };
        let b = if scale == 0.0 { 0.0 } else { other.scale / scale };
        let left = if scale == 0.0 { 0.0 } else { self.mean / scale };
        let right = if scale == 0.0 { 0.0 } else { other.mean / scale };
        let delta = right - left;
        self.m2 = self.m2 * a * a + other.m2 * b * b + delta * delta * self.count as f64 * other.count as f64 / count as f64;
        self.mean = (left + delta * other.count as f64 / count as f64).clamp(-1.0, 1.0) * scale;
        self.squares = self.squares * a * a + other.squares * b * b;
        self.scale = scale;
        if other.min.compare(&self.min).map_err(|e| Error::Invalid(e.to_string()))?.is_lt() {
            self.min = other.min.clone();
        }
        if other.max.compare(&self.max).map_err(|e| Error::Invalid(e.to_string()))?.is_gt() {
            self.max = other.max.clone();
        }
        self.count = count;
        self.last = other.last.clone();
        self.total = match (self.total, other.total) {
            (Some(a), Some(b)) => Some(a.checked_add(b).map_err(|e| Error::Invalid(e.to_string()))?),
            _ => None,
        };
        Ok(())
    }
    pub(crate) fn mean(&self) -> Result<f64> {
        if self.count == 0 {
            return Ok(0.0);
        }
        if let Some(total) = self.total {
            return Ok(
                scalar::number(&ScalarValue::Exact(total.divide(self.count as u128).map_err(|e| Error::Invalid(e.to_string()))?)).unwrap()
            );
        }
        Ok(self.mean)
    }
    pub(crate) fn stddev(&self) -> Result<f64> {
        if self.count == 0 {
            return Ok(0.0);
        }
        if let (Some(squares), Some(total)) = (&self.integer_squares, self.total) {
            let mut sum = BigInt::from(total.numerator);
            if total.negative {
                sum = -sum;
            }
            let numerator = squares * BigInt::from(self.count) - &sum * &sum;
            let numerator = numerator.to_f64().ok_or(Error::Corrupt("integer variance exceeds its position bound"))?;
            return Ok((numerator / self.count as f64 / self.count as f64).max(0.0).sqrt());
        }
        Ok((self.m2 / self.count as f64).clamp(0.0, 1.0).sqrt() * self.scale)
    }
    pub(crate) fn rms(&self) -> Result<f64> {
        if self.count == 0 {
            return Ok(0.0);
        }
        if let Some(squares) = &self.integer_squares {
            return Ok((squares.to_f64().ok_or(Error::Corrupt("integer squares exceed their position bound"))? / self.count as f64).sqrt());
        }
        Ok((self.squares / self.count as f64).clamp(0.0, 1.0).sqrt() * self.scale)
    }
}
impl Statistics {
    pub(super) fn load(connection: &Connection, id: &str, settings: PreviewSettings) -> Result<Self> {
        let mut query = connection.prepare_cached("SELECT field,moments FROM telemetry_summaries WHERE file=?1")?;
        let summaries = query
            .query_map([id], |row| Ok((row.get::<_, String>(0)?, row.get::<_, Vec<u8>>(1)?)))?
            .map(|row| {
                let (field, bytes) = row?;
                Ok((field, serde_json::from_slice::<Moments>(&bytes)?))
            })
            .collect::<Result<_>>()?;
        Ok(Self {
            settings,
            summaries,
            previews: previews::load(connection, id)?,
            touched: BTreeMap::new(),
        })
    }

    /// Fold One Chunk In and Return Each Valued Field's Chunk Moments for Its Checkpoint.
    pub(super) fn advance(&mut self, batch: &RecordBatch, last: u64) -> Result<Vec<(String, Vec<u8>)>> {
        let mut checkpoints = Vec::new();
        for field in numeric_fields(batch)? {
            let moments = column_moments(&numeric(batch, &field)?);
            if moments.count == 0 {
                continue;
            }
            let envelope = self.previews.entry(field.clone()).or_insert_with(|| Envelope::empty(self.settings));
            previews::extend(envelope, batch, &field, self.settings.bucket_count.get())?;
            self.summaries.entry(field.clone()).or_insert_with(Moments::empty).merge(&moments)?;
            self.touched.insert(field.clone(), last);
            checkpoints.push((field, serde_json::to_vec(&moments)?));
        }
        Ok(checkpoints)
    }

    /// Save the Summary and Preview of Every Field a Chunk Gave a Value.
    pub(super) fn save(&self, connection: &Connection, id: &str) -> Result<()> {
        for (field, through) in &self.touched {
            connection
                .prepare_cached(
                    "INSERT INTO telemetry_summaries VALUES(?1,?2,?3) ON CONFLICT(file,field) DO UPDATE SET moments=excluded.moments",
                )?
                .execute(params![id, field, serde_json::to_vec(&self.summaries[field])?])?;
            previews::save(connection, id, field, *through, &self.previews[field])?;
        }
        Ok(())
    }
}

impl Store {
    pub(crate) fn moments(&self, window: NumericWindow<'_>) -> Result<Moments> {
        let window = if (window.start_ns != i64::MIN || window.end_ns != i64::MAX) && self.preview_bounds(window)?.is_some() {
            NumericWindow {
                start_ns: i64::MIN,
                end_ns: i64::MAX,
                ..window
            }
        } else {
            window
        };
        let mut moments = Moments::empty();
        if !self.has_projection(window)? && window.after == 0 && window.start_ns == i64::MIN && window.end_ns == i64::MAX {
            let connection = self.lock()?;
            let entries = {
                let mut query=connection.prepare("SELECT s.moments,b.last_position FROM telemetry_summaries s JOIN telemetry_files f ON f.id=s.file JOIN (SELECT file,MAX(last_position) last_position FROM telemetry_batches GROUP BY file) b ON b.file=s.file WHERE f.execution=?1 AND f.stream=?2 AND s.field=?3")?;
                query
                    .query_map(params![window.execution, window.stream, window.field], |row| {
                        Ok((row.get::<_, Vec<u8>>(0)?, row.get::<_, i64>(1)? as u64))
                    })?
                    .collect::<std::result::Result<Vec<_>, _>>()?
            };
            if entries.iter().all(|(_, through)| *through <= window.through) {
                let mut through = 0;
                for (bytes, last) in entries {
                    moments.merge(&serde_json::from_slice::<Moments>(&bytes)?)?;
                    through = through.max(last);
                }
                drop(connection);
                let tail = NumericWindow { after: through, ..window };
                self.evaluated_batches(tail, &[window.field.into()], |batch| {
                    let column = scalar::column(&batch, window.field)?;
                    for row in 0..batch.num_rows() {
                        moments.push(column.value(row)?)?;
                    }
                    Ok(true)
                })?;
                return Ok(moments);
            }
        }
        self.evaluated_batches(window, &[window.field.into()], |batch| {
            let column = scalar::column(&batch, window.field)?;
            for row in 0..batch.num_rows() {
                moments.push(column.value(row)?)?;
            }
            Ok(true)
        })?;
        Ok(moments)
    }
    // Spill Projected Order Keys Once; Radix Passes Never Replay Stream Decoding.
    // Each Pass Retains Only Requested Ranks And 256 Counters Per Prefix.
    // Integer Order And Interpolation Stay Exact Through The Typed Protocol Value.
    pub(crate) fn ranks(&self, window: NumericWindow<'_>, percentiles: &[u8]) -> Result<BTreeMap<u8, ScalarValue>> {
        rank_values(
            |push| {
                self.evaluated_batches(window, &[window.field.into()], |batch| {
                    let column = scalar::column(&batch, window.field)?;
                    for row in 0..batch.num_rows() {
                        push(column.value(row)?)?;
                    }
                    Ok(true)
                })
            },
            percentiles,
        )
    }
}

/* === Public Functions === */

/* === Private Functions === */

pub(crate) fn rank_values(
    read: impl FnOnce(&mut dyn FnMut(ScalarValue) -> Result<()>) -> Result<()>, percentiles: &[u8],
) -> Result<BTreeMap<u8, ScalarValue>> {
    let mut count = 0u64;
    let mut kind = None;
    let mut keys = tempfile::tempfile()?;
    {
        let mut output = BufWriter::new(&mut keys);
        read(&mut |value| {
            let key = match value {
                ScalarValue::Signed(v) => {
                    kind = Some(0);
                    (v as u64) ^ (1 << 63)
                }
                ScalarValue::Bool(v) => {
                    kind = Some(1);
                    u64::from(v)
                }
                ScalarValue::Unsigned(v) => {
                    kind = Some(1);
                    v
                }
                ScalarValue::Float(v) if v.is_finite() => {
                    kind = Some(2);
                    let bits = v.to_bits();
                    if bits >> 63 != 0 { !bits } else { bits ^ (1 << 63) }
                }
                ScalarValue::Missing | ScalarValue::Null => return Ok(()),
                _ => return Err(Error::Data("Compute Exact Percentile: Select Scalar Integers or Finite Numbers")),
            };
            output.write_all(&key.to_le_bytes())?;
            count += 1;
            Ok(())
        })?;
        output.flush()?;
    }
    if count == 0 {
        return Ok(BTreeMap::new());
    }
    let ranks = percentiles
        .iter()
        .flat_map(|p| {
            let n = (count.saturating_sub(1) as u128) * u128::from(*p);
            [(n / 100) as u64, n.div_ceil(100) as u64]
        })
        .collect::<std::collections::BTreeSet<_>>();
    let mut selected = ranks.into_iter().map(|rank| (rank, (0u64, rank))).collect::<BTreeMap<_, _>>();
    for shift in (0..8).rev().map(|n| n * 8) {
        let mut groups = selected.values().map(|(prefix, _)| (*prefix, [0u64; 256])).collect::<BTreeMap<_, _>>();
        keys.rewind()?;
        let mut input = BufReader::new(&mut keys);
        for _ in 0..count {
            let mut bytes = [0; 8];
            input.read_exact(&mut bytes)?;
            let key = u64::from_le_bytes(bytes);
            let prefix = if shift == 56 { 0 } else { key & (!0u64 << (shift + 8)) };
            if let Some(histogram) = groups.get_mut(&prefix) {
                histogram[((key >> shift) & 255) as usize] += 1;
            }
        }
        for (prefix, rank) in selected.values_mut() {
            let histogram = &groups[prefix];
            let mut bin = 0;
            while bin < 255 && *rank >= histogram[bin] {
                *rank -= histogram[bin];
                bin += 1;
            }
            *prefix |= (bin as u64) << shift;
        }
    }
    let decode = |key: u64| match kind {
        Some(0) => ScalarValue::Signed((key ^ (1 << 63)) as i64),
        Some(1) => ScalarValue::Unsigned(key),
        _ => ScalarValue::Float(f64::from_bits(if key >> 63 != 0 { key ^ (1 << 63) } else { !key })),
    };
    percentiles
        .iter()
        .map(|p| {
            let n = (count.saturating_sub(1) as u128) * u128::from(*p);
            let lo = decode(selected[&((n / 100) as u64)].0);
            let hi = decode(selected[&(n.div_ceil(100) as u64)].0);
            let fraction = n % 100;
            let value = if fraction == 0 {
                lo
            } else if kind == Some(2) {
                let a = scalar::number(&lo).unwrap();
                let b = scalar::number(&hi).unwrap();
                ScalarValue::Float(a * (1.0 - fraction as f64 / 100.0) + b * (fraction as f64 / 100.0))
            } else {
                let exact = |v: ScalarValue| match v {
                    ScalarValue::Signed(v) => ExactNumber::new(v < 0, v.unsigned_abs() as u128, NonZeroU128::MIN),
                    ScalarValue::Unsigned(v) => ExactNumber::new(false, v as u128, NonZeroU128::MIN),
                    _ => unreachable!(),
                };
                let a = exact(lo);
                let b = exact(hi);
                let a = ExactNumber::new(a.negative, a.numerator * (100 - fraction), NonZeroU128::MIN);
                let b = ExactNumber::new(b.negative, b.numerator * fraction, NonZeroU128::MIN);
                ScalarValue::Exact(a.checked_add(b).and_then(|n| n.divide(100)).map_err(|e| Error::Invalid(e.to_string()))?)
            };
            Ok((*p, value))
        })
        .collect()
}

/// Save One Chunk's Moments for One Field, Keyed by the Chunk's File Offset.
pub(super) fn checkpoint(
    connection: &Connection, id: &str, offset: u64, field: &str, (first, last, start, end): (u64, u64, i64, i64), moments: &[u8],
) -> Result<()> {
    connection.prepare_cached("INSERT INTO telemetry_checkpoints VALUES(?1,?2,?3,?4,?5,?6,?7,?8)")?.execute(params![
        id,
        offset as i64,
        field,
        first as i64,
        last as i64,
        start,
        end,
        moments
    ])?;
    Ok(())
}
/// The Dotted Paths of a Stored Batch's Numeric Value Columns, in Schema Order.
pub(super) fn numeric_fields(batch: &RecordBatch) -> Result<Vec<String>> {
    let values = batch.column(1).as_any().downcast_ref::<StructArray>().ok_or(Error::Corrupt("summary values differ"))?;
    let mut fields = Vec::new();
    scalar_fields(values.fields(), "", &mut fields);
    Ok(fields)
}

/// Resolve a Numeric Value Column With the Validity of Every Enclosing Structure Folded In.
pub(super) fn numeric<'a>(batch: &'a RecordBatch, path: &str) -> Result<Numeric<'a>> {
    let mut array = batch.column_by_name("values").ok_or(Error::Corrupt("values column missing"))?;
    let mut nulls = array.nulls().cloned();
    for name in path.split('.') {
        let structure = array.as_any().downcast_ref::<StructArray>().ok_or(Error::Data("select a scalar field path"))?;
        array = structure.column_by_name(name).ok_or(Error::Data("declare the selected scalar column"))?;
        nulls = NullBuffer::union(nulls.as_ref(), array.nulls());
    }
    let any = array.as_any();
    let values = if let Some(array) = any.downcast_ref::<Int64Array>() {
        Values::Signed(array.values())
    } else if let Some(array) = any.downcast_ref::<UInt64Array>() {
        Values::Unsigned(array.values())
    } else if let Some(array) = any.downcast_ref::<Float64Array>() {
        Values::Float(array.values())
    } else {
        return Err(Error::Data("select a numeric value column"));
    };
    Ok(Numeric { values, nulls })
}

/// One Chunk's Moments, Computed Over the Column's Typed Values in Row Order.
///
/// Integer totals and squares are exact, so they are summed directly; the scaled float recurrence
/// runs the same operations in the same order as [`Moments::push`], so its results are identical.
pub(super) fn column_moments(column: &Numeric<'_>) -> Moments {
    let nulls = column.nulls.as_ref();
    match column.values {
        Values::Signed(values) => integer_moments(values, nulls, ScalarValue::Signed),
        Values::Unsigned(values) => integer_moments(values, nulls, ScalarValue::Unsigned),
        Values::Float(values) => float_moments(values, nulls),
    }
}

fn integer_moments<T: Copy + Ord + Into<i128>>(values: &[T], nulls: Option<&NullBuffer>, scalar: fn(T) -> ScalarValue) -> Moments {
    let mut count = 0u64;
    let mut bounds: Option<(T, T, T, T)> = None;
    let mut total = 0i128;
    let mut squares = 0u128;
    let mut spilled = BigInt::from(0);
    each(values, nulls, |value| {
        count += 1;
        bounds = Some(match bounds {
            None => (value, value, value, value),
            Some((min, max, first, _)) => (min.min(value), max.max(value), first, value),
        });
        let integer: i128 = value.into();
        total += integer;
        let square = integer.unsigned_abs() * integer.unsigned_abs();
        squares = squares.checked_add(square).unwrap_or_else(|| {
            spilled += squares;
            square
        });
    });
    let Some((min, max, first, last)) = bounds else {
        return Moments::empty();
    };
    spilled += squares;
    Moments {
        count,
        min: scalar(min),
        max: scalar(max),
        first: scalar(first),
        last: scalar(last),
        total: Some(ExactNumber::new(total < 0, total.unsigned_abs(), NonZeroU128::MIN)),
        integer_squares: Some(spilled),
        ..Moments::empty()
    }
}

fn float_moments(values: &[f64], nulls: Option<&NullBuffer>) -> Moments {
    let (mut count, mut mean, mut m2, mut squares, mut scale) = (0u64, 0.0f64, 0.0f64, 0.0f64, 0.0f64);
    let (mut min, mut max, mut first, mut last) = (0.0f64, 0.0f64, 0.0f64, 0.0f64);
    each(values, nulls, |number| {
        if !number.is_finite() {
            return;
        }
        if count == 0 {
            (min, max, first) = (number, number, number);
        }
        let next = scale.max(number.abs());
        let factor = if next == 0.0 { 0.0 } else { scale / next };
        let previous = if next == 0.0 { 0.0 } else { mean / next };
        let normalized = if next == 0.0 { 0.0 } else { number / next };
        m2 *= factor * factor;
        squares *= factor * factor;
        scale = next;
        count += 1;
        let delta = normalized - previous;
        let updated = previous + delta / count as f64;
        mean = updated.clamp(-1.0, 1.0) * scale;
        m2 += delta * (normalized - updated);
        squares += normalized * normalized;
        if number < min {
            min = number;
        }
        if number > max {
            max = number;
        }
        last = number;
    });
    if count == 0 {
        return Moments::empty();
    }
    Moments {
        count,
        mean,
        m2,
        squares,
        scale,
        min: ScalarValue::Float(min),
        max: ScalarValue::Float(max),
        first: ScalarValue::Float(first),
        last: ScalarValue::Float(last),
        total: None,
        integer_squares: None,
    }
}

/// Visit the Values of Valid Rows in Row Order.
fn each<T: Copy>(values: &[T], nulls: Option<&NullBuffer>, mut visit: impl FnMut(T)) {
    match nulls {
        None => values.iter().for_each(|value| visit(*value)),
        Some(nulls) => nulls.valid_indices().for_each(|row| visit(values[row])),
    }
}

fn scalar_fields(fields: &Fields, prefix: &str, output: &mut Vec<String>) {
    for field in fields {
        let name = if prefix.is_empty() { field.name().clone() } else { format!("{prefix}.{}", field.name()) };
        match field.data_type() {
            DataType::Struct(fields) => scalar_fields(fields, &name, output),
            DataType::Int64 | DataType::UInt64 | DataType::Float64 => output.push(name),
            _ => (),
        }
    }
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;
    use crate::telemetry::previews::{Envelope, Point, extend};
    use arrow_array::ArrayRef;
    use arrow_schema::Field;
    use std::sync::Arc;

    /// A Deterministic xorshift Sequence, So Every Run Compares the Same Inputs.
    struct Sequence(u64);

    impl Sequence {
        fn next(&mut self) -> u64 {
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 7;
            self.0 ^= self.0 << 17;
            self.0
        }
        fn pick<T: Copy>(&mut self, choices: &[T]) -> T {
            choices[(self.next() % choices.len() as u64) as usize]
        }
    }

    fn nulls(sequence: &mut Sequence, rows: usize, one_in: u64) -> NullBuffer {
        NullBuffer::from((0..rows).map(|_| !sequence.next().is_multiple_of(one_in)).collect::<Vec<_>>())
    }

    /// Extremes, Signed Zeros, Non-Finite Values, Nulls at Every Level and Times That Jump Back.
    fn batch(sequence: &mut Sequence, rows: usize) -> RecordBatch {
        let mut time = sequence.next() as i64 % 1_000_000;
        let times = (0..rows)
            .map(|_| {
                time += sequence.pick(&[0, 1, 7, 999, 1_000, 25_000, -3_000, 1_000_000]);
                time
            })
            .collect::<Vec<i64>>();
        let signed = (0..rows)
            .map(|_| match sequence.next() % 4 {
                0 => sequence.pick(&[i64::MIN, i64::MAX, 0, -1, 1]),
                _ => sequence.next() as i64 >> sequence.pick(&[0, 20, 50]),
            })
            .collect::<Vec<_>>();
        let unsigned = (0..rows)
            .map(|_| match sequence.next() % 4 {
                0 => sequence.pick(&[u64::MAX, 0, 1]),
                _ => sequence.next() >> sequence.pick(&[0, 30, 60]),
            })
            .collect::<Vec<_>>();
        let mut float = || {
            (0..rows)
                .map(|_| match sequence.next() % 6 {
                    0 => sequence.pick(&[f64::NAN, f64::INFINITY, f64::NEG_INFINITY, 0.0, -0.0, 1e300, -1e-300, f64::MIN_POSITIVE]),
                    _ => (sequence.next() as i64 as f64) * sequence.pick(&[1e-12, 1.0, 1e6]),
                })
                .collect::<Vec<f64>>()
        };
        let (plain, deep) = (float(), float());
        let nullable = |name: &str, kind: DataType| Arc::new(Field::new(name, kind, true));
        let inner = StructArray::try_new(
            vec![nullable("deep", DataType::Float64)].into(),
            vec![Arc::new(Float64Array::new(deep.into(), Some(nulls(sequence, rows, 5)))) as ArrayRef],
            Some(nulls(sequence, rows, 4)),
        )
        .unwrap();
        let values = StructArray::try_new(
            vec![
                nullable("signed", DataType::Int64),
                nullable("unsigned", DataType::UInt64),
                nullable("float", DataType::Float64),
                nullable("nested", inner.data_type().clone()),
            ]
            .into(),
            vec![
                Arc::new(Int64Array::new(signed.into(), Some(nulls(sequence, rows, 7)))) as ArrayRef,
                Arc::new(UInt64Array::new(unsigned.into(), None)),
                Arc::new(Float64Array::new(plain.into(), Some(nulls(sequence, rows, 9)))),
                Arc::new(inner),
            ],
            Some(nulls(sequence, rows, 11)),
        )
        .unwrap();
        let storage = StructArray::try_new(
            vec![Arc::new(Field::new("position", DataType::UInt64, false)), Arc::new(Field::new("source_ns", DataType::Int64, false))]
                .into(),
            vec![Arc::new(UInt64Array::from_iter_values(1..=rows as u64)) as ArrayRef, Arc::new(Int64Array::from(times))],
            None,
        )
        .unwrap();
        RecordBatch::try_from_iter([("_storage", Arc::new(storage) as ArrayRef), ("values", Arc::new(values) as ArrayRef)]).unwrap()
    }

    #[test]
    fn column_kernels_match_the_row_order_reference_bit_for_bit() {
        let mut sequence = Sequence(0x9E37_79B9_7F4A_7C15);
        for (rows, width, maximum) in [(1, 1, 256), (300, 1_000, 4), (4_096, 1, 16), (2_000, 1_000_000, 256), (10_000, 7, 3)] {
            let batch = batch(&mut sequence, rows);
            for field in numeric_fields(&batch).unwrap() {
                let mut expected = Moments::empty();
                let reference = scalar::column(&batch, &field).unwrap();
                for row in 0..rows {
                    expected.push(reference.value(row).unwrap()).unwrap();
                }
                let actual = column_moments(&numeric(&batch, &field).unwrap());
                assert_eq!(serde_json::to_vec(&actual).unwrap(), serde_json::to_vec(&expected).unwrap(), "{field} moments");
                // Chunks Extend One Envelope, the Second Starting From the First One's Coarsened Grid.
                let empty = Envelope {
                    origin: 0,
                    width,
                    buckets: BTreeMap::new(),
                };
                let (mut expected, mut actual) = (empty.clone(), empty);
                for chunk in [batch.slice(0, rows / 2), batch.slice(rows / 2, rows - rows / 2)] {
                    let reference = scalar::column(&chunk, &field).unwrap();
                    let storage = chunk.column(0).as_any().downcast_ref::<StructArray>().unwrap();
                    let times = storage.column(1).as_any().downcast_ref::<Int64Array>().unwrap();
                    let positions = storage.column(0).as_any().downcast_ref::<UInt64Array>().unwrap();
                    for row in 0..chunk.num_rows() {
                        if let Some(value) = scalar::number(&reference.value(row).unwrap()).filter(|value| value.is_finite()) {
                            let point = Point {
                                time: times.value(row) as i128,
                                position: positions.value(row),
                                value,
                            };
                            expected.push(point, maximum).unwrap();
                        }
                    }
                    extend(&mut actual, &chunk, &field, maximum).unwrap();
                    assert_eq!(postcard::to_allocvec(&actual).unwrap(), postcard::to_allocvec(&expected).unwrap(), "{field} preview");
                }
            }
        }
    }
}
