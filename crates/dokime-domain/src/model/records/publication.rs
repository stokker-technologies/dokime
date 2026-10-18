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
use crate::model::execution::RecordScope;
use crate::model::identity::ArtifactId;
use crate::model::identity::Identifier;
use crate::model::identity::RecordId;
use crate::model::identity::TelemetrySampleId;
use crate::model::identity::TelemetryStreamId;
use crate::model::records::RecordSeverity;
use crate::model::records::error::RecordErrorDetail;
use crate::model::records::evidence::Record;
use crate::model::records::product::ProductRecordDraft;
use crate::model::time::Timestamp;
use crate::model::values::Value;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordPublishRequest {
    pub record_type: Identifier,
    #[serde(default)]
    pub timestamp: Option<Timestamp>,
    pub severity: RecordSeverity,
    pub message: Option<String>,
    #[serde(default)]
    pub error: Option<RecordErrorDetail>,
    #[serde(default)]
    pub details: BTreeMap<String, Value>,
    pub scope: Option<RecordScope>,
    pub related_sample_id: Option<TelemetrySampleId>,
    pub related_stream_id: Option<TelemetryStreamId>,
    pub related_artifact_id: Option<ArtifactId>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublishReceipt {
    pub status: PublishStatus,
    pub accepted: u64,
    pub dropped: u64,
    pub stream_cursors: BTreeMap<String, u64>,
    pub revision: u64,
    pub message: Option<String>,
}

/* === Enums === */
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PublishStatus {
    Accepted,
    Dropped,
}

/* === Traits === */

/* === Implementations === */
impl RecordPublishRequest {
    pub fn new(record_type: Identifier, severity: RecordSeverity) -> Self {
        Self {
            record_type,
            timestamp: None,
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

    pub fn into_record(self, record_id: RecordId, schema_version: u32, timestamp: Timestamp, source: impl Into<String>) -> Record {
        Record {
            record_id,
            received_at: timestamp.clone(),
            timestamp: self.timestamp.unwrap_or(timestamp),
            record_type: self.record_type,
            schema_version,
            source: source.into(),
            severity: self.severity,
            message: self.message,
            error: self.error,
            details: self.details,
            scope: self.scope,
            related_sample_id: self.related_sample_id,
            related_stream_id: self.related_stream_id,
            related_artifact_id: self.related_artifact_id,
        }
    }
}

impl From<ProductRecordDraft> for RecordPublishRequest {
    fn from(draft: ProductRecordDraft) -> Self {
        Self {
            record_type: draft.record_type,
            timestamp: None,
            severity: draft.severity,
            message: draft.message,
            error: None,
            details: draft.details,
            scope: draft.scope,
            related_sample_id: None,
            related_stream_id: None,
            related_artifact_id: None,
        }
    }
}

impl PublishReceipt {
    pub fn accepted(accepted: u64, stream_cursors: BTreeMap<String, u64>, revision: u64) -> Self {
        Self { status: PublishStatus::Accepted, accepted, dropped: 0, stream_cursors, revision, message: None }
    }

    pub fn dropped(dropped: u64, revision: u64, message: impl Into<String>) -> Self {
        Self {
            status: PublishStatus::Dropped,
            accepted: 0,
            dropped,
            stream_cursors: BTreeMap::new(),
            revision,
            message: Some(message.into()),
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
