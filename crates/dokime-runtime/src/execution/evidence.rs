/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::collections::BTreeMap;
use std::ops::Deref;
// Third Party Imports
use harmos::signal::Series;

// Local Imports
use dokime_domain::model::identity::Identifier;
use crate::telemetry::TelemetrySample;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, Default, serde::Serialize)]
pub struct ExecutionEvidence {
    pub(crate) samples: BTreeMap<String, Vec<TelemetrySample>>,
    pub(crate) subscribed_records: Vec<crate::records::Record>,
    pub(crate) steps: Vec<Identifier>,
    pub(crate) attempts: Vec<super::TestStepAttempt>,
    pub(crate) gaps: Vec<crate::telemetry::LeaseGap>,
}

// The rows of one stream a criterion reads, in capture order. It reads as a
// slice of rows (`iter`, `len`, `last`) and turns one numeric field into a
// Harmos `Series`, which answers the aggregate vocabulary over it:
// `ctx.samples::<Samples>().column(|row| row.voltage_v).median()`.
#[derive(Clone, Debug, PartialEq)]
pub struct Samples<Row> {
    rows: Vec<Row>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl ExecutionEvidence {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_sample(&mut self, sample: TelemetrySample) {
        self.samples
            .entry(sample.stream.to_string())
            .or_default()
            .push(sample);
    }

    pub fn record_gaps(&mut self, gaps: Vec<crate::telemetry::LeaseGap>) {
        self.gaps = gaps;
    }

    pub fn record_step(&mut self, step: Identifier) {
        self.steps.push(step);
    }
}

impl<Row> Samples<Row> {
    pub(crate) fn new(rows: Vec<Row>) -> Self {
        Self { rows }
    }

    // One numeric field of every row; a non-finite value is skipped, as every
    // aggregate skips it, so `count` says how many values were measured.
    pub fn column(&self, field: impl FnMut(&Row) -> f64) -> Series<'static> {
        self.rows.iter().map(field).collect()
    }

    pub fn into_rows(self) -> Vec<Row> {
        self.rows
    }
}

impl<Row> Deref for Samples<Row> {
    type Target = [Row];

    fn deref(&self) -> &[Row] {
        &self.rows
    }
}

impl<Row> IntoIterator for Samples<Row> {
    type Item = Row;
    type IntoIter = std::vec::IntoIter<Row>;

    fn into_iter(self) -> Self::IntoIter {
        self.rows.into_iter()
    }
}

impl<'a, Row> IntoIterator for &'a Samples<Row> {
    type Item = &'a Row;
    type IntoIter = std::slice::Iter<'a, Row>;

    fn into_iter(self) -> Self::IntoIter {
        self.rows.iter()
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
