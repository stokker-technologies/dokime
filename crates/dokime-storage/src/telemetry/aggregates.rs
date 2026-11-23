/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Library Numeric Results and Product Envelope Persistence

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

// Third Party Imports
use arrow_array::{Array, Float64Array, Int64Array, RecordBatch, StructArray, UInt64Array};
use arrow_buffer::NullBuffer;
use arrow_schema::{DataType, Fields};
use harmos_query::config::QueryAccumulatorSettings;
use harmos_query::scalar::Agg;
use harmos_query::transform::{FieldPath, FieldPathExecution};
use harmos_query::{Accumulator, Aggregate};
use rusqlite::Connection;
use tracing::info_span;

// Local Imports
use super::config::PreviewSettings;
use super::previews::{self, Envelope};
use super::summary::Running;
use crate::scalar::ScalarValue;
use crate::{Error, NumericWindow, Result, Store};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

pub(crate) type SummaryReads = BTreeMap<SummarySelection, Arc<NumericRead>>;

/* === Structs === */

/// A Numeric Selection's Count, Extremes, Capture-Order Edges, Total, Mean and Population Spread.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct NumericSummary {
    pub count: u64,
    pub min: ScalarValue,
    pub max: ScalarValue,
    pub first: ScalarValue,
    pub last: ScalarValue,
    pub sum: ScalarValue,
    pub average: ScalarValue,
    pub standard_deviation: f64,
    pub rms: f64,
}

#[derive(Default)]
pub(crate) struct NumericRead {
    value: Mutex<Option<NumericSummary>>,
}

/// An Unprojected Numeric Selection at One Saved Cut; Display Resolution Is Not Part of Its Statistics.
#[derive(Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct SummarySelection {
    execution: String,
    stream: String,
    field: String,
    after: u64,
    through: u64,
    start_ns: i64,
    end_ns: i64,
}

pub(crate) struct Numeric<'a> {
    pub(crate) values: Values<'a>,
    pub(crate) nulls: Option<NullBuffer>,
}

/// Values Ranked Exactly by the Library for Medians and Percentiles.
pub(crate) struct Ranked {
    accumulator: Accumulator,
    position: u64,
    /// Finite Floats Selected Directly While Every Value Is One and They Fit the Budget; Others Go to the Library.
    floats: Option<Vec<f64>>,
    capacity: usize,
}

/// Stored Envelopes Advanced During File Publication; One of an Earlier Layout Is Left As Saved.
pub(super) struct Statistics {
    settings: PreviewSettings,
    previews: BTreeMap<String, Option<Envelope>>,
    touched: BTreeMap<String, u64>,
}

/* === Enums === */

pub(crate) enum Values<'a> {
    Signed(&'a [i64]),
    Unsigned(&'a [u64]),
    Float(&'a [f64]),
}

/* === Traits === */

/* === Implementations === */

impl Numeric<'_> {
    pub(crate) fn valid(&self, row: usize) -> bool {
        self.nulls.as_ref().is_none_or(|nulls| nulls.is_valid(row))
    }
}

impl Ranked {
    pub(crate) fn new(memory_bytes: usize) -> Self {
        Self {
            accumulator: Accumulator::with_settings(Agg::Of(Aggregate::Median), QueryAccumulatorSettings { memory_bytes }),
            position: 0,
            floats: Some(Vec::new()),
            capacity: memory_bytes / size_of::<f64>(),
        }
    }

    pub(crate) fn push(&mut self, value: &ScalarValue) -> Result<()> {
        self.position += 1;
        if let (Some(floats), ScalarValue::Float(number)) = (&mut self.floats, value)
            && floats.len() < self.capacity
        {
            // A Nonfinite Sample Is a Missing Measurement, as in the Library.
            if number.is_finite() {
                floats.push(*number);
            }
            return Ok(());
        }
        for (position, number) in self.floats.take().into_iter().flatten().enumerate() {
            self.accumulator.push(position as u64 + 1, &ScalarValue::Float(number))?;
        }
        Ok(self.accumulator.push(self.position, value)?)
    }

    /// Push One Column's Valid Values in Row Order; the Library Treats Nonfinite Values as Missing.
    pub(crate) fn extend(&mut self, values: &Values<'_>, valid: impl Fn(usize) -> bool) -> Result<()> {
        match values {
            Values::Signed(values) => {
                (0..values.len()).filter(|row| valid(*row)).try_for_each(|row| self.push(&ScalarValue::Signed(values[row])))
            }
            Values::Unsigned(values) => {
                (0..values.len()).filter(|row| valid(*row)).try_for_each(|row| self.push(&ScalarValue::Unsigned(values[row])))
            }
            Values::Float(values) => match &mut self.floats {
                Some(floats) if floats.len() + values.len() <= self.capacity => {
                    self.position += values.len() as u64;
                    floats.extend((0..values.len()).filter(|row| valid(*row) && values[*row].is_finite()).map(|row| values[row]));
                    Ok(())
                }
                _ => (0..values.len()).filter(|row| valid(*row)).try_for_each(|row| self.push(&ScalarValue::Float(values[row]))),
            },
        }
    }

    /// The Library's Exact Ranks; Floats Are Selected in Linear Time With the Library's Interpolation.
    pub(crate) fn finish(self, statistics: &[Aggregate]) -> Result<Vec<ScalarValue>> {
        let _completion = info_span!("plot.order_statistics.finish").entered();
        let Some(mut floats) = self.floats else {
            return Ok(self.accumulator.finish_ranks(statistics, &mut || Ok(()))?);
        };
        statistics
            .iter()
            .map(|statistic| {
                let rank = match statistic {
                    Aggregate::Median => 50,
                    Aggregate::Percentile(rank @ 0..=100) => *rank,
                    _ => return Err(Error::Data("select median or percentile ranks in 0..=100")),
                };
                if floats.is_empty() {
                    return Ok(ScalarValue::Missing);
                }
                let numerator = (floats.len() - 1) as u128 * u128::from(rank);
                let lower = (numerator / 100) as usize;
                let (_, low, above) = floats.select_nth_unstable_by(lower, f64::total_cmp);
                let low = *low;
                let high = if numerator % 100 == 0 { low } else { above.iter().copied().min_by(f64::total_cmp).unwrap_or(low) };
                let fraction = (numerator % 100) as f64 / 100.0;
                let answer = if low == high {
                    low
                } else if low.signum() == high.signum() {
                    low + (high - low) * fraction
                } else {
                    low * (1.0 - fraction) + high * fraction
                };
                Ok(ScalarValue::Float(answer))
            })
            .collect()
    }
}

impl From<NumericWindow<'_>> for SummarySelection {
    fn from(window: NumericWindow<'_>) -> Self {
        Self {
            execution: window.execution.into(),
            stream: window.stream.into(),
            field: window.field.into(),
            after: window.after,
            through: window.through,
            start_ns: window.start_ns,
            end_ns: window.end_ns,
        }
    }
}

impl Statistics {
    pub(super) fn load(connection: &Connection, id: &str, settings: PreviewSettings) -> Result<Self> {
        Ok(Self {
            settings,
            previews: previews::load(connection, id)?,
            touched: BTreeMap::new(),
        })
    }

    pub(super) fn advance(&mut self, batch: &RecordBatch, last: u64) -> Result<()> {
        for field in numeric_fields(batch)? {
            let Some(envelope) = self.previews.entry(field.clone()).or_insert_with(|| Some(Envelope::empty(self.settings))) else {
                continue;
            };
            previews::extend(envelope, batch, &field, self.settings.bucket_count.get())?;
            if !envelope.buckets.is_empty() {
                self.touched.insert(field, last);
            }
        }
        Ok(())
    }

    pub(super) fn save(&self, connection: &Connection, id: &str) -> Result<()> {
        for (field, through) in &self.touched {
            if let Some(envelope) = &self.previews[field] {
                previews::save(connection, id, field, *through, envelope)?;
            }
        }
        Ok(())
    }
}

impl Store {
    /// A Selection's Summary: the Field's Incremental Statistics When It Selects Every Value, Otherwise One Pass.
    pub(crate) fn numeric_summary(&self, window: NumericWindow<'_>) -> Result<NumericSummary> {
        self.check_read()?;
        if let Some(envelope) = self.envelope(window)? {
            return envelope.summary.finish();
        }
        // Only Finite Read Sessions Share Results; Live Tails and Projected Selections Are Recomputed.
        let summaries = if self.has_projection(window)? { None } else { self.summaries.as_ref() };
        let selection = SummarySelection::from(window);
        let read = summaries
            .map(|summaries| -> Result<_> {
                let mut summaries = summaries.lock().map_err(|_| Error::Data("start a new read after the numeric summary lock failed"))?;
                Ok(summaries.entry(selection).or_insert_with(|| Arc::new(NumericRead::default())).clone())
            })
            .transpose()?;
        let mut saved = read
            .as_ref()
            .map(|read| read.value.lock().map_err(|_| Error::Data("start a new read after its numeric calculation failed")))
            .transpose()?;
        self.check_read()?;
        if let Some(summary) = saved.as_ref().and_then(|saved| saved.as_ref()) {
            return Ok(summary.clone());
        }
        let mut running = Running::default();
        self.numeric_batches(window, |_, column| {
            running.extend(&column.values, |row| column.valid(row));
            Ok(())
        })?;
        let summary = running.finish()?;
        if let Some(saved) = saved.as_mut() {
            **saved = Some(summary.clone());
        }
        Ok(summary)
    }

    pub(crate) fn ranks(&self, window: NumericWindow<'_>, percentiles: &[u8]) -> Result<BTreeMap<u8, ScalarValue>> {
        let statistics = percentiles.iter().copied().map(Aggregate::Percentile).collect::<Vec<_>>();
        let values = rank_values(
            |push| {
                self.evaluated_batches(window, &[window.field.into()], |batch| {
                    let field = FieldPath::from(format!("values.{}", window.field).as_str());
                    for row in 0..batch.num_rows() {
                        push(field.value(&batch, row)?)?;
                    }
                    Ok(true)
                })
            },
            &statistics,
            self.storage_settings()?.query_bytes.get(),
        )?;
        Ok(percentiles
            .iter()
            .copied()
            .zip(values)
            .filter(|(_, value)| !matches!(value, ScalarValue::Missing | ScalarValue::Null))
            .collect())
    }
}

/* === Public Functions === */

/* === Private Functions === */

pub(crate) fn rank_values(
    read: impl FnOnce(&mut dyn FnMut(ScalarValue) -> Result<()>) -> Result<()>, statistics: &[Aggregate], memory_bytes: usize,
) -> Result<Vec<ScalarValue>> {
    let mut ranked = Ranked::new(memory_bytes);
    let input = info_span!("plot.order_statistics.input").entered();
    read(&mut |value| ranked.push(&value))?;
    drop(input);
    ranked.finish(statistics)
}

pub(super) fn numeric_fields(batch: &RecordBatch) -> Result<Vec<String>> {
    let values = batch.column(1).as_any().downcast_ref::<StructArray>().ok_or(Error::Corrupt("summary values differ"))?;
    let mut fields = Vec::new();
    scalar_fields(values.fields(), "", &mut fields);
    Ok(fields)
}

/// Resolve a Numeric Value Column With the Validity of Every Enclosing Structure Folded In.
pub(crate) fn numeric<'a>(batch: &'a RecordBatch, path: &str) -> Result<Numeric<'a>> {
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
    use super::Ranked;
    use crate::scalar::ScalarValue;
    use harmos_query::Aggregate;

    #[test]
    fn selected_float_ranks_equal_the_library_ranks() {
        let statistics = [Aggregate::Median, Aggregate::Percentile(5), Aggregate::Percentile(95), Aggregate::Percentile(99)];
        let mut state = 0x9e37_79b9_7f4a_7c15_u64;
        for count in [1usize, 2, 7, 64, 1001] {
            let values = (0..count)
                .map(|index| {
                    state = state.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1_442_695_040_888_963_407);
                    match index % 9 {
                        0 => f64::NAN,
                        1 => 0.0,
                        _ => ((state >> 11) as f64 / (1u64 << 53) as f64 - 0.4) * 1e3,
                    }
                })
                .collect::<Vec<_>>();
            // A Budget of No Floats Ranks Every Value Through the Library.
            let (mut selected, mut library) = (Ranked::new(1 << 20), Ranked::new(0));
            for value in &values {
                selected.push(&ScalarValue::Float(*value)).unwrap();
                library.push(&ScalarValue::Float(*value)).unwrap();
            }
            assert!(selected.floats.is_some() && library.floats.is_none());
            assert_eq!(selected.finish(&statistics).unwrap(), library.finish(&statistics).unwrap(), "{count} Values");
        }
    }
}
