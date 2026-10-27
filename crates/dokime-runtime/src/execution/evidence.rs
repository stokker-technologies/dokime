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
use super::evidence_rows::EvidenceRows;

// Third Party Imports
use harmos::signal::Series;

// Local Imports
use crate::telemetry::TelemetrySample;
use dokime_domain::model::identity::Identifier;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Debug, Default, serde::Serialize)]
pub struct ExecutionEvidence {
    pub(crate) samples: EvidenceRows<TelemetrySample>,
    pub(crate) subscribed_records: EvidenceRows<dokime_domain::model::records::evidence::Record>,
    pub(crate) steps: Vec<Identifier>,
    pub(crate) attempts: Vec<dokime_domain::model::execution::result::TestStepAttempt>,
    pub(crate) gaps: Vec<crate::telemetry::LeaseGap>,
}

// Criteria Iterate Saved Rows In Capture Order; Explicit Column Collection
// Uses A Finite Workspace For The Harmos Aggregate Vocabulary.
#[derive(Clone, Debug)]
pub struct Samples<Row> {
    rows: EvidenceRows<TelemetrySample>,
    decode: fn(&std::collections::BTreeMap<String, dokime_domain::model::values::Value>) -> Result<Row, crate::telemetry::FieldError>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl ExecutionEvidence {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_sample(&mut self, sample: TelemetrySample) {
        self.samples.push(sample.sample_id.as_str(), &sample).expect("execution evidence persistence failed");
    }

    pub fn record_gaps(&mut self, gaps: Vec<crate::telemetry::LeaseGap>) {
        self.gaps = gaps;
    }

    pub fn record_step(&mut self, step: Identifier) {
        self.steps.push(step);
    }
}

impl<Row> Samples<Row> {
    pub(crate) fn new(
        rows: EvidenceRows<TelemetrySample>,
        decode: fn(&std::collections::BTreeMap<String, dokime_domain::model::values::Value>) -> Result<Row, crate::telemetry::FieldError>,
    ) -> Self {
        Self { rows, decode }
    }

    pub fn iter(&self) -> impl Iterator<Item = Row> + '_ {
        self.rows.iter().filter_map(|sample| match (self.decode)(&sample.values) {
            Ok(row) => Some(row),
            Err(error) => {
                tracing::warn!(%error, "captured telemetry row shape differs from its declaration");
                None
            }
        })
    }

    pub fn len(&self) -> usize {
        self.iter().count()
    }
    pub fn is_empty(&self) -> bool {
        self.iter().next().is_none()
    }
    pub fn last(&self) -> Option<Row> {
        self.iter().last()
    }
    pub fn first(&self) -> Option<Row> {
        self.iter().next()
    }

    pub fn column(&self, mut field: impl FnMut(&Row) -> f64) -> Series<'static> {
        let mut values = Vec::new();
        let count = self.rows.len();
        assert!(count <= 64 * 1024 * 1024 / std::mem::size_of::<f64>(), "criterion column exceeds bounded workspace");
        values.try_reserve_exact(count).expect("criterion column allocation failed");
        values.extend(self.iter().map(|row| field(&row)));
        values.into()
    }

    pub fn into_rows(self) -> Vec<Row> {
        let maximum = 64 * 1024 * 1024 / std::mem::size_of::<Row>().max(1);
        assert!(self.rows.len() <= maximum, "criterion row collection exceeds bounded workspace");
        let mut bytes = 0usize;
        for row in self.rows.iter() {
            bytes = bytes
                .checked_add(crate::execution::evidence_rows::encoded_bytes(&row))
                .filter(|bytes| *bytes <= 64 * 1024 * 1024)
                .expect("criterion row collection exceeds bounded workspace");
        }
        self.iter().collect()
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
