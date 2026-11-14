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
use arrow_array::{RecordBatch, StructArray};
use arrow_schema::DataType;
use arrow_schema::Fields;
use harmos_storage::query::scalar::ExactNumber;
use num_bigint::BigInt;
use num_traits::ToPrimitive;
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};

// Local Imports
use super::config::PreviewSettings;
use crate::{
    Error, NumericWindow, Result, Sample, Store,
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

/* === Enums === */

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
impl Store {
    pub(crate) fn moments(&self, window: NumericWindow<'_>) -> Result<Moments> {
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

pub(super) fn append(
    connection: &Connection, id: &str, offset: u64, batch: &RecordBatch, samples: &[Sample], settings: PreviewSettings,
) -> Result<()> {
    let first = samples.first().ok_or(Error::Data("flush a nonempty sample batch"))?.position;
    let last = samples.last().unwrap().position;
    let start = samples.iter().map(|s| s.source_ns).min().unwrap();
    let end = samples.iter().map(|s| s.source_ns).max().unwrap();
    let values = batch.column(1).as_any().downcast_ref::<StructArray>().ok_or(Error::Corrupt("summary values differ"))?;
    let mut fields = Vec::new();
    scalar_fields(values.fields(), "", &mut fields);
    for field in fields {
        let mut moments = Moments::empty();
        let column = scalar::column(batch, &field)?;
        for row in 0..batch.num_rows() {
            moments.push(column.value(row)?)?;
        }
        if moments.count == 0 {
            continue;
        }
        super::previews::append(connection, id, &field, batch, last, settings)?;
        let bytes = serde_json::to_vec(&moments)?;
        connection.execute(
            "INSERT INTO telemetry_checkpoints VALUES(?1,?2,?3,?4,?5,?6,?7,?8)",
            params![id, offset as i64, field, first as i64, last as i64, start, end, bytes],
        )?;
        let previous: Option<Vec<u8>> = connection
            .query_row("SELECT moments FROM telemetry_summaries WHERE file=?1 AND field=?2", params![id, field], |row| row.get(0))
            .optional()?;
        let mut full = previous.map(|bytes| serde_json::from_slice::<Moments>(&bytes)).transpose()?.unwrap_or_else(Moments::empty);
        full.merge(&moments)?;
        connection.execute(
            "INSERT INTO telemetry_summaries VALUES(?1,?2,?3) ON CONFLICT(file,field) DO UPDATE SET moments=excluded.moments",
            params![id, field, serde_json::to_vec(&full)?],
        )?;
    }
    Ok(())
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
