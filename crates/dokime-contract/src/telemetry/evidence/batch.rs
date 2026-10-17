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
use super::TelemetrySample;
use crate::core::{Identifier, TelemetryBatchId};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TelemetryBatch {
    pub batch_id: TelemetryBatchId,
    pub stream: Identifier,
    pub schema_version: u32,
    pub samples: Vec<TelemetrySample>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl harmos::MemorySize for TelemetryBatch {
    fn memory_size(&self) -> usize {
        let fields = [
            (
                harmos::MemorySize::memory_size(&self.batch_id),
                std::mem::size_of_val(&self.batch_id),
            ),
            (
                harmos::MemorySize::memory_size(&self.stream),
                std::mem::size_of_val(&self.stream),
            ),
            (
                harmos::MemorySize::memory_size(&self.samples),
                std::mem::size_of_val(&self.samples),
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

impl TelemetryBatch {
    pub fn new(batch_id: TelemetryBatchId, stream: Identifier, samples: Vec<TelemetrySample>) -> Self {
        Self {
            batch_id,
            stream,
            schema_version: 1,
            samples,
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
