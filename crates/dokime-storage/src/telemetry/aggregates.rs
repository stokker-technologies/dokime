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
use std::{collections::BTreeMap, num::NonZeroU128};

// Third Party Imports
use arrow_array::{RecordBatch, StructArray};
use arrow_schema::DataType;
use harmos_storage::query::scalar::ExactNumber;
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};

// Local Imports
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
    pub min: ScalarValue,
    pub max: ScalarValue,
    pub first: ScalarValue,
    pub last: ScalarValue,
    pub total: Option<ExactNumber>,
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
            min: ScalarValue::Missing,
            max: ScalarValue::Missing,
            first: ScalarValue::Missing,
            last: ScalarValue::Missing,
            total: Some(ExactNumber::new(false, 0, NonZeroU128::MIN)),
        }
    }
    fn push(&mut self, value: ScalarValue) -> Result<()> {
        let Some(number) = scalar::number(&value).filter(|v| v.is_finite()) else {
            return Ok(());
        };
        if self.count == 0 {
            self.min = value.clone();
            self.max = value.clone();
            self.first = value.clone();
        }
        self.count += 1;
        let delta = number - self.mean;
        self.mean += delta / self.count as f64;
        self.m2 += delta * (number - self.mean);
        self.squares += number * number;
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
        let count = self.count + other.count;
        let delta = other.mean - self.mean;
        self.m2 += other.m2 + delta * delta * self.count as f64 * other.count as f64 / count as f64;
        self.mean += delta * other.count as f64 / count as f64;
        self.squares += other.squares;
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
    // Eight Radix Passes Retain Only Requested Ranks And 256 Counters Per Prefix.
    // Integer Order And Interpolation Stay Exact Until The Protocol's f64 Boundary.
    pub(crate) fn ranks(&self, window: NumericWindow<'_>, count: u64, percentiles: &[u8]) -> Result<BTreeMap<u8, ScalarValue>> {
        let ranks = percentiles
            .iter()
            .flat_map(|p| {
                let n = (count.saturating_sub(1) as u128) * u128::from(*p);
                [(n / 100) as u64, n.div_ceil(100) as u64]
            })
            .collect::<std::collections::BTreeSet<_>>();
        let mut selected = ranks.into_iter().map(|rank| (rank, (0u64, rank))).collect::<BTreeMap<_, _>>();
        let mut kind = None;
        for shift in (0..8).rev().map(|n| n * 8) {
            let mut groups = selected.values().map(|(prefix, _)| (*prefix, [0u64; 256])).collect::<BTreeMap<_, _>>();
            self.evaluated_batches(window, &[window.field.into()], |batch| {
                let column = scalar::column(&batch, window.field)?;
                for row in 0..batch.num_rows() {
                    let value = column.value(row)?;
                    let key = match value {
                        ScalarValue::Signed(v) => {
                            kind = Some(0);
                            (v as u64) ^ (1 << 63)
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
                        _ => continue,
                    };
                    let prefix = if shift == 56 { 0 } else { key & (!0u64 << (shift + 8)) };
                    if let Some(histogram) = groups.get_mut(&prefix) {
                        histogram[((key >> shift) & 255) as usize] += 1;
                    }
                }
                Ok(true)
            })?;
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
}

/* === Public Functions === */

/* === Private Functions === */

pub(super) fn append(
    connection: &Connection, id: &str, offset: u64, batch: &RecordBatch, first: u64, last: u64, start: i64, end: i64,
    settings: super::config::PreviewSettings,
) -> Result<()> {
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
fn scalar_fields(fields: &arrow_schema::Fields, prefix: &str, output: &mut Vec<String>) {
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
