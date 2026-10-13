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

/* === Re-Exports === */
pub use sample::Sample;

/* === Imports === */
// Standard Library Imports
use std::num::NonZeroUsize;
// Third Party Imports
use harmos::{MemorySize, Message, storage::SchemaKey};
// Local Imports
use super::{Error, Result, records::ScopeRow, values};
use crate::core::{Identifier, SessionId};
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

#[harmos::message]
#[derive(Clone, PartialEq)]
pub struct TelemetryRow {
    epoch: String,
    ordinal: u64,
    sample_id: String,
    stream: String,
    schema_version: u32,
    sequence: u64,
    timestamp: String,
    received_at: String,
    source: String,
    scope: Option<ScopeRow>,
    values: Vec<values::Cell>,
    summary: Vec<values::Cell>,
    metadata: Vec<values::Cell>,
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

impl TelemetryRow {
    pub fn encoding_memory(entry: &TelemetryEntry) -> Result<usize> {
        let sample = Sample::encoding_memory(&entry.sample)?;
        if sample == usize::MAX {
            return Ok(usize::MAX);
        }
        Ok(std::mem::size_of::<Self>()
            .saturating_add(entry.epoch.as_str().len())
            .saturating_add(sample))
    }

    pub fn from_entry(entry: &TelemetryEntry) -> Result<Self> {
        if Self::encoding_memory(entry)? == usize::MAX {
            return Err(error("telemetry native encoding memory overflow"));
        }
        let sample = Sample::from_sample(&entry.sample)?;
        Ok(Self {
            epoch: sample::copy_text(entry.epoch.as_str())?,
            ordinal: entry.ordinal,
            sample_id: sample.sample_id,
            stream: sample.stream,
            schema_version: sample.schema_version,
            sequence: sample.sequence,
            timestamp: sample.timestamp,
            received_at: sample.received_at,
            source: sample.source,
            scope: sample.scope,
            values: sample.values,
            summary: sample.summary,
            metadata: sample.metadata,
        })
    }

    pub fn decoding_memory(bytes: &[u8]) -> Result<usize> {
        let native = <Self as Message>::decoding_memory(bytes)?;
        native
            .checked_add(std::mem::size_of::<Vec<u8>>())
            .and_then(|bound| bound.checked_add(bytes.len()))
            .ok_or_else(|| error("telemetry decoding memory overflows"))
    }

    pub fn from_bytes(bytes: &[u8], max_bytes: usize) -> Result<Self> {
        if bytes.len() > max_bytes {
            return Err(error("telemetry capture frame exceeds byte limit"));
        }
        let row = Self::decode(bytes)?;
        let length = row.encoded_len();
        if length > bytes.len() {
            return Err(error("telemetry capture frame is not canonical protobuf"));
        }
        let mut canonical = Vec::new();
        canonical
            .try_reserve_exact(length)
            .map_err(|_| error("telemetry canonical allocation failed"))?;
        if canonical.capacity() > length {
            return Err(error("telemetry canonical capacity exceeds admission"));
        }
        row.encode_into(&mut canonical);
        if canonical != bytes {
            return Err(error("telemetry capture frame is not canonical protobuf"));
        }
        Ok(row)
    }

    pub fn projection_memory(&self) -> Result<usize> {
        let sample = sample::projection_memory(sample::Fields {
            strings: [
                &self.sample_id,
                &self.stream,
                &self.timestamp,
                &self.received_at,
                &self.source,
            ],
            maps: [&self.values, &self.summary, &self.metadata],
            scope: self.scope.as_ref(),
        })?;
        if sample == usize::MAX {
            return Ok(usize::MAX);
        }
        Ok(std::mem::size_of::<TelemetryEntry>()
            .saturating_add(self.epoch.capacity())
            .saturating_add(sample.saturating_sub(std::mem::size_of::<TelemetrySample>())))
    }

    pub fn into_entry(self, allowance: NonZeroUsize) -> Result<TelemetryEntry> {
        let bound = self.projection_memory()?;
        if bound == usize::MAX || bound > allowance.get() {
            return Err(error("telemetry projection exceeds memory allowance"));
        }
        if self.epoch.bytes().any(|byte| byte.is_ascii_uppercase()) {
            return Err(error("noncanonical telemetry producer identity"));
        }
        let epoch = SessionId::from_uuid(self.epoch)?;
        let sample = Sample {
            sample_id: self.sample_id,
            stream: self.stream,
            schema_version: self.schema_version,
            sequence: self.sequence,
            timestamp: self.timestamp,
            received_at: self.received_at,
            source: self.source,
            scope: self.scope,
            values: self.values,
            summary: self.summary,
            metadata: self.metadata,
        }
        .into_sample(allowance)?;
        Ok(TelemetryEntry {
            epoch,
            ordinal: self.ordinal,
            sample,
        })
    }
}

/* === Public Functions === */
pub fn schema() -> SchemaKey {
    SchemaKey {
        payload_encoding: harmos::storage::PayloadEncoding::PostcardV1,
        id: Identifier::new("dokime.capture.telemetry").unwrap(),
        version: 1.try_into().unwrap(),
    }
}

pub fn encode(entry: &TelemetryEntry, max_bytes: usize) -> Result<Vec<u8>> {
    let row = TelemetryRow::from_entry(entry)?;
    if row.encoded_len() > max_bytes {
        return Err(error("telemetry capture frame exceeds byte limit"));
    }
    Ok(row.encode_to_vec())
}

/* === Private Functions === */
fn error(reason: &'static str) -> Error {
    Error::Invalid(reason)
}

/* === Tests === */
