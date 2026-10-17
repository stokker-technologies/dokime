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
use serde::{Deserialize, Serialize};
// Local Imports
use crate::records::RecordEntry;
use crate::core::{Identifier, Timestamp};
use crate::telemetry::TelemetrySample;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceSummaries {
    pub records: Vec<RecordSummary>,
    pub telemetry_streams: Vec<TelemetryStreamSummary>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordSummary {
    pub record_type: Identifier,
    pub source: String,
    pub total: u64,
    pub latest_sequence: u64,
    pub updated_at: Timestamp,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TelemetryStreamSummary {
    pub stream: Identifier,
    pub source: String,
    pub total: u64,
    pub latest_sequence: u64,
    pub updated_at: Timestamp,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl EvidenceSummaries {
    pub fn from_records<'a>(
        records: impl IntoIterator<Item = &'a RecordEntry>,
        telemetry: impl IntoIterator<Item = &'a TelemetrySample>,
    ) -> Self {
        let mut summaries = Self::default();
        for record in records {
            summaries.record_record(record);
        }
        for sample in telemetry {
            summaries.record_telemetry_sample(sample);
        }

        summaries
    }

    pub fn record_record(&mut self, record: &RecordEntry) {
        let mut summaries = self
            .records
            .iter()
            .cloned()
            .map(|summary| {
                (
                    (summary.record_type.to_string(), summary.source.clone()),
                    summary,
                )
            })
            .collect::<BTreeMap<_, _>>();
        let key = (record.record_type.to_string(), record.source.clone());
        let entry = summaries.entry(key).or_insert_with(|| RecordSummary {
            record_type: record.record_type.clone(),
            source: record.source.clone(),
            total: 0,
            latest_sequence: 0,
            updated_at: record.timestamp.clone(),
        });

        entry.total += 1;
        entry.latest_sequence = record.sequence;
        entry.updated_at = record.timestamp.clone();
        self.records = summaries.into_values().collect();
    }

    pub fn record_telemetry_sample(&mut self, sample: &TelemetrySample) {
        let mut summaries = self
            .telemetry_streams
            .iter()
            .cloned()
            .map(|summary| {
                (
                    (summary.stream.to_string(), summary.source.clone()),
                    summary,
                )
            })
            .collect::<BTreeMap<_, _>>();
        let key = (sample.stream.to_string(), sample.source.clone());
        let entry = summaries
            .entry(key)
            .or_insert_with(|| TelemetryStreamSummary {
                stream: sample.stream.clone(),
                source: sample.source.clone(),
                total: 0,
                latest_sequence: 0,
                updated_at: sample.timestamp.clone(),
            });

        entry.total += 1;
        entry.latest_sequence = sample.sequence;
        entry.updated_at = sample.timestamp.clone();
        self.telemetry_streams = summaries.into_values().collect();
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
