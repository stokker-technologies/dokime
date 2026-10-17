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
pub use typed::{
    TypedCodec, codec, decode_batch, decode_stream_values, decode_values, encode_batch,
};

/* === Imports === */
// Standard Library Imports
// Third Party Imports
use harmos::MemorySize;
// Local Imports

use crate::core::SessionId;
use crate::telemetry::TelemetrySample;

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
