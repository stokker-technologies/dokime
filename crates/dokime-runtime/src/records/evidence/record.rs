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
use crate::core::{
    ArtifactId, Identifier, RecordId, RecordScope, TelemetrySampleId, TelemetryStreamId, Timestamp,
    Value,
};
use crate::records::{RecordErrorDetail, RecordSeverity};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[harmos::record(id = "dokime.record", version = 2)]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    pub record_id: RecordId,
    pub timestamp: Timestamp,
    pub received_at: Timestamp,
    pub record_type: Identifier,
    pub schema_version: u32,
    pub source: String,
    pub severity: RecordSeverity,
    pub message: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<RecordErrorDetail>,
    #[serde(default)]
    pub details: BTreeMap<String, Value>,
    pub scope: Option<RecordScope>,
    pub related_sample_id: Option<TelemetrySampleId>,
    pub related_stream_id: Option<TelemetryStreamId>,
    pub related_artifact_id: Option<ArtifactId>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl harmos::MemorySize for Record {
    fn memory_size(&self) -> usize {
        let fields = [
            (
                harmos::MemorySize::memory_size(&self.record_id),
                std::mem::size_of_val(&self.record_id),
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
                harmos::MemorySize::memory_size(&self.record_type),
                std::mem::size_of_val(&self.record_type),
            ),
            (
                harmos::MemorySize::memory_size(&self.source),
                std::mem::size_of_val(&self.source),
            ),
            (
                harmos::MemorySize::memory_size(&self.message),
                std::mem::size_of_val(&self.message),
            ),
            (
                harmos::MemorySize::memory_size(&self.error),
                std::mem::size_of_val(&self.error),
            ),
            (
                harmos::MemorySize::memory_size(&self.details),
                std::mem::size_of_val(&self.details),
            ),
            (
                harmos::MemorySize::memory_size(&self.scope),
                std::mem::size_of_val(&self.scope),
            ),
            (
                harmos::MemorySize::memory_size(&self.related_sample_id),
                std::mem::size_of_val(&self.related_sample_id),
            ),
            (
                harmos::MemorySize::memory_size(&self.related_stream_id),
                std::mem::size_of_val(&self.related_stream_id),
            ),
            (
                harmos::MemorySize::memory_size(&self.related_artifact_id),
                std::mem::size_of_val(&self.related_artifact_id),
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

impl Record {
    pub fn new(
        timestamp: Timestamp,
        record_type: Identifier,
        schema_version: u32,
        source: impl Into<String>,
        severity: RecordSeverity,
    ) -> Self {
        Self {
            record_id: crate::model::identity::new_id::<RecordId>(),
            received_at: timestamp.clone(),
            timestamp,
            record_type,
            schema_version,
            source: source.into(),
            severity,
            message: None,
            error: None,
            details: BTreeMap::new(),
            scope: None,
            related_sample_id: None,
            related_stream_id: None,
            related_artifact_id: None,
        }
    }

    pub fn detail<T: serde::de::DeserializeOwned>(&self, name: &str) -> crate::Result<T> {
        let value = self
            .details
            .get(name)
            .ok_or_else(|| crate::Error::RecordDetail {
                record: self.record_type.to_string(),
                field: name.to_owned(),
                reason: "no such detail is carried".to_owned(),
            })?;
        value
            .to_plain_json()
            .map_err(|error| error.to_string())
            .and_then(|json| serde_json::from_value(json).map_err(|error| error.to_string()))
            .map_err(|_| crate::Error::RecordDetail {
                record: self.record_type.to_string(),
                field: name.to_owned(),
                reason: "is not the type it was read as".to_owned(),
            })
    }

    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
