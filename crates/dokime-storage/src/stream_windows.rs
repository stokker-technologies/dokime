/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Indexed Stream Windows

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::num::NonZeroUsize;

// Third Party Imports
use prost_reflect::{DynamicMessage, ReflectMessage, Value};

// Local Imports
use crate::{Error, Result, Store};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct SampleWindow<'a> {
    pub execution: &'a str,
    pub streams: &'a [String],
    pub after: &'a [(String, u64)],
    pub through: u64,
    pub start_ns: i64,
    pub end_ns: i64,
    pub latest: bool,
    pub limit: NonZeroUsize,
}

#[derive(Clone, Copy)]
pub struct NumericWindow<'a> {
    pub execution: &'a str,
    pub stream: &'a str,
    pub field: &'a str,
    pub after: u64,
    pub through: u64,
    pub start_ns: i64,
    pub end_ns: i64,
    pub bins: NonZeroUsize,
}

#[derive(Clone, Debug)]
pub struct NumericBin {
    pub start_ns: i64,
    pub end_ns: i64,
    pub count: u64,
    pub first_ns: i64,
    pub last_ns: i64,
    pub min_ns: i64,
    pub max_ns: i64,
    pub min: f64,
    pub max: f64,
    pub mean: f64,
    pub sum_squares: f64,
    pub first: f64,
    pub last: f64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Store {
    pub fn stream_names(&self, execution: &str) -> Result<Vec<String>> {
        let connection = self.lock()?;
        let mut statement = connection.prepare("SELECT stream FROM streams WHERE execution=?1 ORDER BY stream")?;
        Ok(statement.query_map([execution], |row| row.get(0))?.collect::<std::result::Result<_, _>>()?)
    }

    pub fn numeric_bounds(&self, execution: &str, stream: &str, field: &str, through: u64) -> Result<Option<(i64, i64)>> {
        let mut bounds = None::<(i64, i64)>;
        self.numeric_values(
            NumericWindow {
                execution,
                stream,
                field,
                through,
                after: 0,
                start_ns: i64::MIN,
                end_ns: i64::MAX,
                bins: NonZeroUsize::MIN,
            },
            |_, time, _| {
                bounds = Some(bounds.map_or((time, time), |(start, end)| (start.min(time), end.max(time))));
                Ok(())
            },
        )?;
        Ok(bounds)
    }

    pub(crate) fn numeric_values(&self, window: NumericWindow<'_>, mut read: impl FnMut(u64, i64, f64) -> Result<()>) -> Result<()> {
        self.evaluated_batches(window, &[window.field.into()], |batch| {
            use arrow_array::{Int64Array, StructArray, UInt64Array};
            let metadata = batch.column(0).as_any().downcast_ref::<StructArray>().ok_or(Error::Corrupt("sample metadata differs"))?;
            let positions = metadata.column_by_name("position").unwrap().as_any().downcast_ref::<UInt64Array>().unwrap();
            let times = metadata.column_by_name("source_ns").unwrap().as_any().downcast_ref::<Int64Array>().unwrap();
            let column = crate::scalar::column(&batch, window.field)?;
            for row in 0..batch.num_rows() {
                if let Some(value) = crate::scalar::number(&column.value(row)?).filter(|value| value.is_finite()) {
                    read(positions.value(row), times.value(row), value)?;
                }
            }
            Ok(true)
        })
    }

    pub fn numeric_bins(&self, window: NumericWindow<'_>) -> Result<Vec<NumericBin>> {
        if window.end_ns < window.start_ns {
            return Err(Error::Invalid("Read Plot Window: Choose Increasing Bounds".into()));
        }
        if let Some(bins) = self.preview_bins(window)? {
            return Ok(bins);
        }
        let span = (i128::from(window.end_ns) - i128::from(window.start_ns) + 1).max(1);
        let width = ((span + window.bins.get() as i128 - 1) / window.bins.get() as i128).max(1);
        let mut bins = std::collections::BTreeMap::<usize, (NumericBin, i64, i64)>::new();
        self.numeric_values(window, |position, source_ns, value| {
            let position = position as i64;
            let index = ((i128::from(source_ns) - i128::from(window.start_ns)) / width) as usize;
            let (bin, first_position, last_position) = bins.entry(index).or_insert_with(|| {
                (
                    NumericBin {
                        start_ns: source_ns,
                        end_ns: source_ns,
                        count: 0,
                        min: value,
                        max: value,
                        mean: 0.0,
                        sum_squares: 0.0,
                        first: value,
                        last: value,
                        first_ns: source_ns,
                        last_ns: source_ns,
                        min_ns: source_ns,
                        max_ns: source_ns,
                    },
                    position,
                    position,
                )
            });
            // Each Retained Input Row Is Examined Once; Only Requested Buckets Survive.
            bin.count += 1;
            bin.mean += (value - bin.mean) / bin.count as f64;
            bin.sum_squares += value * value;
            bin.start_ns = bin.start_ns.min(source_ns);
            bin.end_ns = bin.end_ns.max(source_ns);
            if value < bin.min {
                bin.min = value;
                bin.min_ns = source_ns;
            }
            if value > bin.max {
                bin.max = value;
                bin.max_ns = source_ns;
            }
            if position < *first_position {
                *first_position = position;
                bin.first = value;
                bin.first_ns = source_ns;
            }
            if position > *last_position {
                *last_position = position;
                bin.last = value;
                bin.last_ns = source_ns;
            }
            Ok(())
        })?;
        Ok(bins.into_values().map(|(bin, _, _)| bin).collect())
    }
}

/* === Public Functions === */

/* === Private Functions === */

pub(crate) fn indexed_fields(row: &DynamicMessage, prefix: &str) -> Vec<(String, Option<f64>, Option<String>)> {
    let mut values = Vec::new();
    for field in row.descriptor().fields() {
        if field.is_list() || field.is_map() {
            continue;
        }
        let path = if prefix.is_empty() { field.name().to_owned() } else { format!("{prefix}.{}", field.name()) };
        let value = row.get_field(&field);
        if let Value::Message(message) = value.as_ref() {
            values.extend(indexed_fields(message, &path));
            continue;
        }
        let number = match value.as_ref() {
            Value::F64(value) => Some(*value),
            Value::F32(value) => Some(*value as f64),
            Value::I64(value) => Some(*value as f64),
            Value::U64(value) => Some(*value as f64),
            Value::I32(value) => Some(*value as f64),
            Value::U32(value) => Some(*value as f64),
            Value::Bool(value) => Some(u8::from(*value) as f64),
            _ => None,
        }
        .filter(|value| value.is_finite());
        let label = match value.as_ref() {
            Value::String(value) => Some(value.clone()),
            Value::EnumNumber(value) => Some(value.to_string()),
            _ => None,
        };
        values.push((path, number, label));
    }
    values
}

/* === Tests === */
