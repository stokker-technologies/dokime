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
use crate::core::{Identifier, RecordScope, TelemetrySampleId, Timestamp, Value};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TelemetrySample {
    pub sample_id: TelemetrySampleId,
    pub stream: Identifier,
    pub schema_version: u32,
    pub sequence: u64,
    pub timestamp: Timestamp,
    pub received_at: Timestamp,
    pub source: String,
    pub scope: Option<RecordScope>,
    pub values: BTreeMap<String, Value>,
    pub summary: BTreeMap<String, Value>,
    pub metadata: BTreeMap<String, Value>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl harmos::MemorySize for TelemetrySample {
    fn memory_size(&self) -> usize {
        let fields = [
            (
                harmos::MemorySize::memory_size(&self.sample_id),
                std::mem::size_of_val(&self.sample_id),
            ),
            (
                harmos::MemorySize::memory_size(&self.stream),
                std::mem::size_of_val(&self.stream),
            ),
            (
                harmos::MemorySize::memory_size(&self.timestamp),
                std::mem::size_of_val(&self.timestamp),
            ),
            (
                harmos::MemorySize::memory_size(&self.received_at),
                std::mem::size_of_val(&self.received_at),
            ),
            (
                harmos::MemorySize::memory_size(&self.source),
                std::mem::size_of_val(&self.source),
            ),
            (
                harmos::MemorySize::memory_size(&self.scope),
                std::mem::size_of_val(&self.scope),
            ),
            (
                harmos::MemorySize::memory_size(&self.values),
                std::mem::size_of_val(&self.values),
            ),
            (
                harmos::MemorySize::memory_size(&self.summary),
                std::mem::size_of_val(&self.summary),
            ),
            (
                harmos::MemorySize::memory_size(&self.metadata),
                std::mem::size_of_val(&self.metadata),
            ),
        ];
        fields
            .into_iter()
            .fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
                if owned == usize::MAX {
                    usize::MAX
                } else {
                    bytes.saturating_add(owned.saturating_sub(inline))
                }
            })
    }
}

impl TelemetrySample {
    pub fn new(
        stream: Identifier,
        sequence: u64,
        timestamp: Timestamp,
        source: impl Into<String>,
    ) -> Self {
        // Belt: `Streams::published_as` is the SDK's own stamping site, but a
        // host- or prototype-produced sample can reach this constructor
        // without ever passing through it. Seeding the "timestamp" entry here
        // means a caller that only ever inserts its own columns afterward
        // still ends up with one, agreeing with `sample.timestamp` above.
        let mut values = BTreeMap::new();
        values
            .entry("timestamp".to_owned())
            .or_insert_with(|| Value::from(timestamp.clone()));

        Self {
            sample_id: TelemetrySampleId::new(),
            stream,
            schema_version: 1,
            sequence,
            received_at: timestamp.clone(),
            timestamp,
            source: source.into(),
            scope: None,
            values,
            summary: BTreeMap::new(),
            metadata: BTreeMap::new(),
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
