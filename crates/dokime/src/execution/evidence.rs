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
// Third Party Imports

// Local Imports
use crate::core::Identifier;
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

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
