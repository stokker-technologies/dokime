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
    let mut accumulator = Accumulator::with_settings(Agg::Of(Aggregate::Median), QueryAccumulatorSettings { memory_bytes });
    let mut position = 0;
    let input = info_span!("plot.order_statistics.input").entered();
    read(&mut |value| {
        position += 1;
        accumulator.push(position, &value)?;
        Ok(())
    })?;
    drop(input);
    let completion = info_span!("plot.order_statistics.finish").entered();
    let values = accumulator.finish_ranks(statistics, &mut || Ok(()))?;
    drop(completion);
    Ok(values)
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
