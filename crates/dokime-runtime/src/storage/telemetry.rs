/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

mod sample;
mod typed;

/* === Re-Exports === */

pub use sample::Sample;
#[cfg(test)]
pub(crate) use typed::PAYLOAD_PASSES;
pub use typed::{CodecCache, TypedCodec, codec, decode_positioned, decode_values, stored_codecs_where};
pub(crate) use typed::{Signal, index_metadata, numeric_metadata, selected_schema, signals};

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos::MemorySize;

// Local Imports
use crate::telemetry::TelemetrySample;
use dokime_domain::model::identity::SessionId;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TelemetryEntry {
    pub epoch: SessionId,
    pub ordinal: u64,
    pub sample: TelemetrySample,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl MemorySize for TelemetryEntry {
    fn memory_size(&self) -> usize {
        let epoch = self.epoch.memory_size();
        let sample = self.sample.memory_size();
        if epoch == usize::MAX || sample == usize::MAX {
            return usize::MAX;
        }
        std::mem::size_of::<Self>()
            .saturating_add(epoch.saturating_sub(std::mem::size_of_val(&self.epoch)))
            .saturating_add(sample.saturating_sub(std::mem::size_of_val(&self.sample)))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
