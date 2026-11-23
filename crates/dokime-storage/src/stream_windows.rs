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
use arrow_array::{Array, BooleanArray, Int64Array, RecordBatch, StructArray, UInt64Array};
use harmos_query::transform::{FieldPath, FieldPathExecution};
use prost_reflect::{DynamicMessage, ReflectMessage, Value};

// Local Imports
use crate::telemetry::aggregates::{self, Numeric, Values};
use crate::telemetry::previews::{Bucket, Point};
use crate::{Error, Result, Store};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct SampleWindow<'a> {
    pub execution: &'a str,
    pub streams: &'a [String],

    /// Row Fields to Read; Empty Reads Every Field.
    pub fields: &'a [String],

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

/// Fixed-Width Display Bins of a Window Read From Its Rows.
pub(crate) struct WindowBins {
    start_ns: i64,
    width: i128,
    bins: Vec<Option<Bucket>>,
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
    pub scaled_squares: f64,
    pub scale: f64,
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

    #[tracing::instrument(name = "plot.bounds", skip_all)]
    pub fn numeric_bounds(&self, execution: &str, stream: &str, field: &str, through: u64) -> Result<Option<(i64, i64)>> {
        let window = NumericWindow {
            execution,
            stream,
            field,
            through,
            after: 0,
            start_ns: i64::MIN,
            end_ns: i64::MAX,
            bins: NonZeroUsize::MIN,
        };
        if let Some(envelope) = self.envelope(window)? {
            return Ok(envelope.bounds());
        }
        let mut bounds = None::<(i64, i64)>;
        self.numeric_values(window, |_, time, _| {
            bounds = Some(bounds.map_or((time, time), |(start, end)| (start.min(time), end.max(time))));
            Ok(())
        })?;
        Ok(bounds)
    }

    /// Read a Window's Numeric Column Batch by Batch; Booleans Read as Zero and One.
    pub(crate) fn numeric_batches(
        &self, window: NumericWindow<'_>, mut read: impl FnMut(&RecordBatch, &Numeric<'_>) -> Result<()>,
    ) -> Result<()> {
        let path = FieldPath::from(format!("values.{}", window.field).as_str());
        self.evaluated_batches(window, &[window.field.into()], |batch| {
            if let Ok(column) = aggregates::numeric(&batch, window.field) {
                read(&batch, &column)?;
            } else if let Some(flags) = path.array(&batch).ok().and_then(|array| array.as_any().downcast_ref::<BooleanArray>().cloned()) {
                let values = (0..flags.len()).map(|row| u64::from(flags.value(row))).collect::<Vec<_>>();
                read(
                    &batch,
                    &Numeric {
                        values: Values::Unsigned(&values),
                        nulls: flags.logical_nulls(),
                    },
                )?;
            }
            Ok(true)
        })
    }

    /// Visit a Window's Valid Finite Values With Their Positions and Source Times, in Row Order.
    pub(crate) fn numeric_values(&self, window: NumericWindow<'_>, mut read: impl FnMut(u64, i64, f64) -> Result<()>) -> Result<()> {
        self.numeric_batches(window, |batch, column| visit(batch, column, &mut read))
    }

    pub fn numeric_bins(&self, window: NumericWindow<'_>) -> Result<Vec<NumericBin>> {
        if let Some(bins) = self.preview_bins(window)? {
            return Ok(bins);
        }
        let mut bins = WindowBins::new(&window)?;
        self.numeric_values(window, |position, time, value| bins.push(position, time, value))?;
        Ok(bins.finish())
    }
}

impl WindowBins {
    pub(crate) fn new(window: &NumericWindow<'_>) -> Result<Self> {
        if window.end_ns < window.start_ns {
            return Err(Error::Invalid("Read Plot Window: Choose Increasing Bounds".into()));
        }
        let span = (i128::from(window.end_ns) - i128::from(window.start_ns) + 1).max(1);
        Ok(Self {
            start_ns: window.start_ns,
            width: ((span + window.bins.get() as i128 - 1) / window.bins.get() as i128).max(1),
            bins: vec![None; window.bins.get()],
        })
    }

    /// Merge One Value Into Its Bin; Values Outside the Window Belong to No Bin.
    pub(crate) fn push(&mut self, position: u64, time: i64, value: f64) -> Result<()> {
        let point = Point { time: time.into(), position, value };
        let index = usize::try_from((i128::from(time) - i128::from(self.start_ns)) / self.width).ok();
        match index.and_then(|index| self.bins.get_mut(index)) {
            Some(Some(bin)) => bin.merge(Bucket::new(point)),
            Some(empty) => {
                *empty = Some(Bucket::new(point));
                Ok(())
            }
            None => Ok(()),
        }
    }

    pub(crate) fn finish(self) -> Vec<NumericBin> {
        self.bins.iter().flatten().map(Bucket::bin).collect()
    }
}

/* === Public Functions === */

/// Visit One Batch's Valid Finite Values With Their Positions and Source Times, in Row Order.
pub(crate) fn visit(batch: &RecordBatch, column: &Numeric<'_>, mut read: impl FnMut(u64, i64, f64) -> Result<()>) -> Result<()> {
    let metadata = batch.column(0).as_any().downcast_ref::<StructArray>().ok_or(Error::Corrupt("sample metadata differs"))?;
    let positions = metadata.column_by_name("position").unwrap().as_any().downcast_ref::<UInt64Array>().unwrap();
    let times = metadata.column_by_name("source_ns").unwrap().as_any().downcast_ref::<Int64Array>().unwrap();
    for row in (0..batch.num_rows()).filter(|row| column.valid(*row)) {
        let value = match column.values {
            Values::Signed(values) => values[row] as f64,
            Values::Unsigned(values) => values[row] as f64,
            Values::Float(values) => values[row],
        };
        if value.is_finite() {
            read(positions.value(row), times.value(row), value)?;
        }
    }
    Ok(())
}

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
