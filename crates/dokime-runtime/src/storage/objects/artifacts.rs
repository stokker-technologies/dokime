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
use dokime_domain::model::execution::RecordScope;
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
impl Content {
    pub(super) fn is_artifact(&self) -> bool {
        self.kind == "artifact"
    }

    pub(super) fn encoding_memory(value: &ExecutionOutputContent) -> Result<usize> {
        let ExecutionOutputContent::Artifact { record, metadata } = value else {
            return Ok(std::mem::size_of::<Self>().saturating_add("file".len()));
        };
        let record = Record::encoding_memory(record)?;
        let metadata = metadata::Row::encoding_memory(metadata)?;
        if [record, metadata].contains(&usize::MAX) {
            return Ok(usize::MAX);
        }
        Ok(std::mem::size_of::<Self>()
            .saturating_add("artifact".len())
            .saturating_add(record.saturating_sub(std::mem::size_of::<Record>()))
            .saturating_add(metadata.saturating_sub(std::mem::size_of::<metadata::Row>())))
    }

    pub(super) fn projection_memory(&self) -> Result<usize> {
        match (self.kind.as_str(), &self.record, &self.metadata) {
            ("file", None, None) => Ok(std::mem::size_of::<ExecutionOutputContent>()),
            ("artifact", Some(record), Some(metadata)) => {
                let record = record.projection_memory()?;
                let metadata = metadata.projection_memory()?;
                if [record, metadata].contains(&usize::MAX) {
                    return Ok(usize::MAX);
                }
                Ok(std::mem::size_of::<ExecutionOutputContent>()
                    .saturating_add(record.saturating_sub(std::mem::size_of::<Box<ArtifactRecord>>()))
                    .saturating_add(metadata.saturating_sub(std::mem::size_of::<dokime_domain::model::identity::Metadata>())))
            }
            _ => Err(Error::Invalid("invalid saved output content descriptor")),
        }
    }
}

impl TryFrom<&ExecutionOutputContent> for Content {
    type Error = Error;

    fn try_from(value: &ExecutionOutputContent) -> Result<Self> {
        if Self::encoding_memory(value)? == usize::MAX {
            return Err(Error::Invalid("saved output content encoding memory overflow"));
        }
        match value {
            ExecutionOutputContent::File => Ok(Self { kind: "file".into(), record: None, metadata: None }),
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
        if value.projection_memory()? == usize::MAX {
            return Err(Error::Invalid("saved output content projection memory overflow"));
        }
        match (value.kind.as_str(), value.record, value.metadata) {
            ("file", None, None) => Ok(Self::File),
            ("artifact", Some(record), Some(metadata)) => {
                Ok(Self::Artifact { record: Box::new(record.try_into()?), metadata: metadata.try_into()? })
            }
            _ => Err(Error::Invalid("invalid saved output content descriptor")),
        }
    }
}

impl Record {
    fn encoding_memory(value: &ArtifactRecord) -> Result<usize> {
        value.created_at.to_datetime()?;
        let scope = ScopeRow::encoding_memory(&value.scope);
        if scope == usize::MAX {
            return Ok(usize::MAX);
        }
        let fields = [
            value.artifact_id.as_str().len(),
            value.upload_id.as_str().len(),
            scope.saturating_sub(std::mem::size_of::<ScopeRow>()),
            value.filename.len(),
            value.content_type.as_deref().map_or(0, str::len),
            value.sha256.as_deref().map_or(0, str::len),
            status(&value.status).len(),
            value.created_at.as_str().len(),
        ];
        Ok(fields.into_iter().fold(std::mem::size_of::<Self>(), usize::saturating_add))
    }

    fn projection_memory(&self) -> Result<usize> {
        let scope = self.scope.as_ref().ok_or(Error::Invalid("saved artifact scope is missing"))?.projection_memory();
        if scope == usize::MAX {
            return Ok(usize::MAX);
        }
        let fields = [
            self.artifact_id.capacity(),
            self.upload_id.capacity(),
            scope.saturating_sub(std::mem::size_of::<RecordScope>()),
            self.filename.capacity(),
            self.content_type.as_ref().map_or(0, String::capacity),
            self.sha256.as_ref().map_or(0, String::capacity),
            self.created_at.capacity(),
        ];
        Ok(fields.into_iter().fold(std::mem::size_of::<ArtifactRecord>(), usize::saturating_add))
    }
}

impl TryFrom<&ArtifactRecord> for Record {
    type Error = Error;

    fn try_from(value: &ArtifactRecord) -> Result<Self> {
        if Self::encoding_memory(value)? == usize::MAX {
            return Err(Error::Invalid("saved artifact encoding memory overflow"));
        }
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
        if value.projection_memory()? == usize::MAX {
            return Err(Error::Invalid("saved artifact projection memory overflow"));
        }
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
