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
use super::metadata;
use crate::artifacts::{ArtifactRecord, ArtifactStatus};
use crate::storage::{Error, Result, records::ScopeRow};
use dokime_domain::model::execution::saved::ExecutionOutputContent;
use dokime_domain::model::identity::ArtifactId;
use dokime_domain::model::identity::UploadId;
use dokime_domain::model::time::Timestamp;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[harmos::message]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Content {
    kind: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    record: Option<Record>,

    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<metadata::Row>,
}

#[harmos::message]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Record {
    artifact_id: String,
    upload_id: String,
    scope: Option<ScopeRow>,
    filename: String,
    content_type: Option<String>,
    size_bytes: u64,
    sha256: Option<String>,
    status: String,
    created_at: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl TryFrom<&ExecutionOutputContent> for Content {
    type Error = Error;

    fn try_from(value: &ExecutionOutputContent) -> Result<Self> {
        match value {
            ExecutionOutputContent::File => Ok(Self {
                kind: "file".into(),
                record: None,
                metadata: None,
            }),
            ExecutionOutputContent::Artifact { record, metadata } => Ok(Self {
                kind: "artifact".into(),
                record: Some(Record::try_from(record.as_ref())?),
                metadata: Some(metadata::Row::try_from(metadata)?),
            }),
        }
    }
}

impl TryFrom<Content> for ExecutionOutputContent {
    type Error = Error;

    fn try_from(value: Content) -> Result<Self> {
        match (value.kind.as_str(), value.record, value.metadata) {
            ("file", None, None) => Ok(Self::File),
            ("artifact", Some(record), Some(metadata)) => Ok(Self::Artifact {
                record: Box::new(record.try_into()?),
                metadata: metadata.try_into()?,
            }),
            _ => Err(Error::Invalid("invalid saved output content descriptor")),
        }
    }
}

impl TryFrom<&ArtifactRecord> for Record {
    type Error = Error;

    fn try_from(value: &ArtifactRecord) -> Result<Self> {
        value.created_at.to_datetime()?;
        Ok(Self {
            artifact_id: value.artifact_id.as_str().to_owned(),
            upload_id: value.upload_id.as_str().to_owned(),
            scope: Some(ScopeRow::from(&value.scope)),
            filename: value.filename.as_str().to_owned(),
            content_type: value.content_type.as_deref().map(str::to_owned),
            size_bytes: value.size_bytes,
            sha256: value.sha256.as_deref().map(str::to_owned),
            status: status(&value.status).to_owned(),
            created_at: value.created_at.as_str().to_owned(),
        })
    }
}

impl TryFrom<Record> for ArtifactRecord {
    type Error = Error;

    fn try_from(value: Record) -> Result<Self> {
        if [&value.artifact_id, &value.upload_id].into_iter().any(|id| id.bytes().any(|byte| byte.is_ascii_uppercase())) {
            return Err(Error::Invalid("noncanonical saved artifact identity"));
        }
        let created_at = Timestamp::new(value.created_at);
        created_at.to_datetime()?;
        let status = match value.status.as_str() {
            "staged" => ArtifactStatus::Staged,
            "complete" => ArtifactStatus::Complete,
            "cancelled" => ArtifactStatus::Cancelled,
            "rejected" => ArtifactStatus::Rejected,
            _ => return Err(Error::Invalid("invalid saved artifact status")),
        };
        Ok(Self {
            artifact_id: ArtifactId::from_uuid(value.artifact_id)?,
            upload_id: UploadId::from_uuid(value.upload_id)?,
            scope: value.scope.ok_or(Error::Invalid("saved artifact scope is missing"))?.try_into()?,
            filename: value.filename,
            content_type: value.content_type,
            size_bytes: value.size_bytes,
            sha256: value.sha256,
            status,
            created_at,
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn status(value: &ArtifactStatus) -> &'static str {
    match value {
        ArtifactStatus::Staged => "staged",
        ArtifactStatus::Complete => "complete",
        ArtifactStatus::Cancelled => "cancelled",
        ArtifactStatus::Rejected => "rejected",
    }
}

/* === Tests === */
