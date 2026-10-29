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

// Local Imports
use crate::storage::{Error, Result, records::ScopeRow, values};
use crate::telemetry::TelemetrySample;
use dokime_domain::model::identity::Identifier;
use dokime_domain::model::identity::TelemetrySampleId;
use dokime_domain::model::time::Timestamp;

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

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Sample {
    pub fn from_sample(sample: &TelemetrySample) -> Result<Self> {
        sample.timestamp.to_datetime()?;
        sample.received_at.to_datetime()?;
        if sample.schema_version == 0 {
            return Err(error("telemetry schema version must be positive"));
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

    pub fn into_sample(self) -> Result<TelemetrySample> {
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
        let values = values::decode_owned(self.values)?;
        let summary = values::decode_owned(self.summary)?;
        let metadata = values::decode_owned(self.metadata)?;
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

pub(super) fn copy_text(source: &str) -> Result<String> {
    let mut text = String::new();
    text.try_reserve_exact(source.len()).map_err(|_| error("telemetry text allocation failed"))?;
    text.push_str(source);
    Ok(text)
}

/* === Private Functions === */

fn error(reason: &'static str) -> Error {
    Error::Invalid(reason)
}

/* === Tests === */
