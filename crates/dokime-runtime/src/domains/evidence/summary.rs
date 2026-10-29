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

// Third Party Imports
use serde::{Deserialize, Serialize};

// Local Imports
use crate::telemetry::TelemetrySample;
use dokime_domain::model::identity::Identifier;
use dokime_domain::model::records::evidence::RecordEntry;
use dokime_domain::model::time::Timestamp;

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

impl harmos::MemorySize for RecordSummary {
    fn memory_size(&self) -> usize {
        let inline = std::mem::size_of::<Self>();
        [
            self.record_type.memory_size().saturating_sub(std::mem::size_of_val(&self.record_type)),
            self.source.memory_size().saturating_sub(std::mem::size_of_val(&self.source)),
            self.updated_at.memory_size().saturating_sub(std::mem::size_of_val(&self.updated_at)),
        ]
        .into_iter()
        .fold(inline, usize::saturating_add)
    }
}

impl EvidenceSummaries {
    /// Groups Evidence In Stable Source Order.
    pub(crate) fn retained(
        records: &crate::storage::ReadRows<RecordEntry>, telemetry: &[TelemetrySample],
    ) -> crate::runtime::Result<harmos::storage::ReadBatch<Self>> {
        // Sort Borrowed Rows Before Allocating The Grouped Summaries.
        let mut record_rows = summary_slots(records.len())?;
        record_rows.extend(records.iter().enumerate());
        record_rows.sort_unstable_by(|(li, left), (ri, right)| {
            left.record_type.as_str().cmp(right.record_type.as_str()).then(left.source.cmp(&right.source)).then(li.cmp(ri))
        });
        let mut sample_rows = summary_slots(telemetry.len())?;
        sample_rows.extend(telemetry.iter().enumerate());
        sample_rows.sort_unstable_by(|(li, left), (ri, right)| {
            left.stream.as_str().cmp(right.stream.as_str()).then(left.source.cmp(&right.source)).then(li.cmp(ri))
        });
        let mut record_count = 0usize;
        let mut sample_count = 0usize;

        for (index, (_, record)) in record_rows.iter().enumerate() {
            if record_rows.get(index + 1).is_some_and(|(_, next)| next.record_type == record.record_type && next.source == record.source) {
                continue;
            }
            record_count = record_count.checked_add(1).ok_or_else(summary_memory)?;
        }
        for (index, (_, sample)) in sample_rows.iter().enumerate() {
            if sample_rows.get(index + 1).is_some_and(|(_, next)| next.stream == sample.stream && next.source == sample.source) {
                continue;
            }
            sample_count = sample_count.checked_add(1).ok_or_else(summary_memory)?;
        }
        harmos::storage::ReadBatch::new(()).try_map(|_| {
            let mut result = Self {
                records: summary_slots(record_count)?,
                telemetry_streams: summary_slots(sample_count)?,
            };
            let mut count = 0u64;
            for (index, (_, record)) in record_rows.iter().enumerate() {
                count = count.checked_add(1).ok_or_else(summary_memory)?;
                if record_rows
                    .get(index + 1)
                    .is_some_and(|(_, next)| next.record_type == record.record_type && next.source == record.source)
                {
                    continue;
                }
                result.records.push(RecordSummary {
                    record_type: record.record_type.clone(),
                    source: record.source.clone(),
                    total: count,
                    latest_sequence: record.sequence,
                    updated_at: record.timestamp.clone(),
                });
                count = 0;
            }
            for (index, (_, sample)) in sample_rows.iter().enumerate() {
                count = count.checked_add(1).ok_or_else(summary_memory)?;
                if sample_rows.get(index + 1).is_some_and(|(_, next)| next.stream == sample.stream && next.source == sample.source) {
                    continue;
                }
                result.telemetry_streams.push(TelemetryStreamSummary {
                    stream: sample.stream.clone(),
                    source: sample.source.clone(),
                    total: count,
                    latest_sequence: sample.sequence,
                    updated_at: sample.timestamp.clone(),
                });
                count = 0;
            }
            Ok(result)
        })
    }

    pub fn record_telemetry_sample(&mut self, sample: &TelemetrySample) {
        let index = match self
            .telemetry_streams
            .binary_search_by(|summary| summary.stream.as_str().cmp(sample.stream.as_str()).then(summary.source.cmp(&sample.source)))
        {
            Ok(index) => index,
            Err(index) => {
                self.telemetry_streams.insert(
                    index,
                    TelemetryStreamSummary {
                        stream: sample.stream.clone(),
                        source: sample.source.clone(),
                        total: 0,
                        latest_sequence: 0,
                        updated_at: sample.timestamp.clone(),
                    },
                );
                index
            }
        };
        let entry = &mut self.telemetry_streams[index];
        entry.total += 1;
        entry.latest_sequence = sample.sequence;
        entry.updated_at.clone_from(&sample.timestamp);
    }
}
/* === Public Functions === */

/* === Private Functions === */

fn summary_memory() -> crate::runtime::Error {
    harmos::Error::ResourceExhausted {
        resource: "evidence summary construction",
    }
    .into()
}

fn summary_slots<T>(count: usize) -> crate::runtime::Result<Vec<T>> {
    let mut rows = Vec::new();
    rows.try_reserve_exact(count).map_err(|_| summary_memory())?;
    Ok(rows)
}

/* === Tests === */
