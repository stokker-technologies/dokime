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
use std::{collections::BTreeMap, num::NonZeroUsize};

// Third Party Imports

// Local Imports
use dokime_domain::model::identity::Identifier;
use dokime_domain::model::identity::TelemetrySampleId;
use dokime_domain::model::time::Timestamp;
use dokime_domain::model::values::Value;
use crate::storage::{Error, Result, records::ScopeRow, values};
use crate::telemetry::TelemetrySample;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[harmos::message]
#[derive(Clone, PartialEq)]
pub struct Sample {
    pub(super) sample_id: String,
    pub(super) stream: String,
    pub(super) schema_version: u32,
    pub(super) sequence: u64,
    pub(super) timestamp: String,
    pub(super) received_at: String,
    pub(super) source: String,
    pub(super) scope: Option<ScopeRow>,
    pub(super) values: Vec<values::Cell>,
    pub(super) summary: Vec<values::Cell>,
    pub(super) metadata: Vec<values::Cell>,
}
pub(super) struct Fields<'a> {
    pub(super) strings: [&'a String; 5],
    pub(super) maps: [&'a [values::Cell]; 3],
    pub(super) scope: Option<&'a ScopeRow>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Sample {
    pub fn encoding_memory(sample: &TelemetrySample) -> Result<usize> {
        sample.timestamp.to_datetime()?;
        sample.received_at.to_datetime()?;
        if sample.schema_version == 0 {
            return Err(error("telemetry schema version must be positive"));
        }
        let mut bytes = [
            sample.sample_id.as_str(),
            sample.stream.as_str(),
            sample.timestamp.as_str(),
            sample.received_at.as_str(),
            sample.source.as_str(),
        ]
        .into_iter()
        .fold(std::mem::size_of::<Self>(), |bytes, text| {
            bytes.saturating_add(text.len())
        });
        for map in [&sample.values, &sample.summary, &sample.metadata] {
            let native = values::encoding_memory(map)?;
            if native == usize::MAX {
                return Ok(usize::MAX);
            }
            bytes = bytes
                .saturating_add(native.saturating_sub(std::mem::size_of::<Vec<values::Cell>>()));
        }
        if let Some(scope) = &sample.scope {
            let native = ScopeRow::encoding_memory(scope);
            if native == usize::MAX {
                return Ok(usize::MAX);
            }
            bytes = bytes.saturating_add(native.saturating_sub(std::mem::size_of::<ScopeRow>()));
        }
        Ok(bytes)
    }
    pub fn from_sample(sample: &TelemetrySample) -> Result<Self> {
        if Self::encoding_memory(sample)? == usize::MAX {
            return Err(error("telemetry sample encoding memory overflows"));
        }
        Ok(Self {
            sample_id: copy_text(sample.sample_id.as_str())?,
            stream: copy_text(sample.stream.as_str())?,
            schema_version: sample.schema_version,
            sequence: sample.sequence,
            timestamp: copy_text(sample.timestamp.as_str())?,
            received_at: copy_text(sample.received_at.as_str())?,
            source: copy_text(sample.source.as_str())?,
            scope: sample.scope.as_ref().map(ScopeRow::from),
            values: values::encode(&sample.values)?,
            summary: values::encode(&sample.summary)?,
            metadata: values::encode(&sample.metadata)?,
        })
    }
    pub fn projection_memory(&self) -> Result<usize> {
        projection_memory(Fields {
            strings: [
                &self.sample_id,
                &self.stream,
                &self.timestamp,
                &self.received_at,
                &self.source,
            ],
            maps: [&self.values, &self.summary, &self.metadata],
            scope: self.scope.as_ref(),
        })
    }
    pub fn into_sample(self, allowance: NonZeroUsize) -> Result<TelemetrySample> {
        let bound = self.projection_memory()?;
        if bound == usize::MAX || bound > allowance.get() {
            return Err(error(
                "telemetry sample projection exceeds memory allowance",
            ));
        }
        if self.schema_version == 0 {
            return Err(error("telemetry schema version must be positive"));
        }
        if self.sample_id.bytes().any(|byte| byte.is_ascii_uppercase()) {
            return Err(error("noncanonical telemetry sample identity"));
        }
        let timestamp = Timestamp::new(self.timestamp);
        let received_at = Timestamp::new(self.received_at);
        timestamp.to_datetime()?;
        received_at.to_datetime()?;
        let sample_id = TelemetrySampleId::from_uuid(self.sample_id)?;
        let stream = Identifier::new(self.stream)?;
        let scope = self.scope.map(TryInto::try_into).transpose()?;
        let values = values::decode_owned(self.values, allowance)?;
        let summary = values::decode_owned(self.summary, allowance)?;
        let metadata = values::decode_owned(self.metadata, allowance)?;
        Ok(TelemetrySample {
            sample_id,
            stream,
            schema_version: self.schema_version,
            sequence: self.sequence,
            timestamp,
            received_at,
            source: self.source,
            scope,
            values,
            summary,
            metadata,
            storage_payload: None,
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */
pub(super) fn projection_memory(fields: Fields<'_>) -> Result<usize> {
    let mut bytes = fields
        .strings
        .into_iter()
        .fold(std::mem::size_of::<TelemetrySample>(), |bytes, text| {
            bytes.saturating_add(text.capacity())
        });
    for cells in fields.maps {
        let owned = values::owned_memory(cells)?;
        if owned == usize::MAX {
            return Ok(usize::MAX);
        }
        bytes = bytes
            .saturating_add(owned.saturating_sub(std::mem::size_of::<BTreeMap<String, Value>>()));
    }
    if let Some(scope) = fields.scope {
        bytes = bytes.saturating_add(
            scope
                .projection_memory()
                .saturating_sub(std::mem::size_of::<dokime_domain::model::execution::RecordScope>()),
        );
    }
    Ok(bytes)
}
pub(super) fn copy_text(source: &str) -> Result<String> {
    let mut text = String::new();
    text.try_reserve_exact(source.len())
        .map_err(|_| error("telemetry text allocation failed"))?;
    if text.capacity() > source.len() {
        return Err(error("telemetry text capacity exceeds admission"));
    }
    text.push_str(source);
    Ok(text)
}
fn error(reason: &'static str) -> Error {
    Error::Invalid(reason)
}

/* === Tests === */
