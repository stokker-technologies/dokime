/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
pub(super) mod artifacts;
mod execution;
pub(super) mod files;
mod follow;
mod inventory;
mod list;
mod upload;
pub(super) mod window;

/* === Re-Exports === */
pub(super) use upload::{artifact_scope, content_failure};

/* === Imports === */
// Standard Library Imports
// Third Party Imports
use tonic::Status;
// Local Imports
use dokime_runtime::{
    RuntimeHandle,
    artifacts::{ArtifactRecord, ArtifactStatus},
};

/* === Constants === */
pub(super) const MAX_CONTENT_BYTES: u64 = 2 * 1024 * 1024 * 1024;
const MAX_TRANSFER_CHUNK_BYTES: usize = 64 * 1024;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
/* === Enums === */
/* === Traits === */
/* === Implementations === */

/* === Public Functions === */
pub(super) fn register(runtime: &RuntimeHandle, maximum: usize) -> Result<(), crate::grpc::Error> {
    artifacts::register(runtime, maximum)
}

pub(super) fn transfer_chunk_bytes(max_message_bytes: usize) -> usize {
    use dokime_contract::operations::content as wire;
    // Reserve Both The Wire Envelope And The Retained Stream Frame.
    let chunk = wire::ArtifactChunk {
        offset: u64::MAX,
        data: Vec::new(),
    };
    let overhead = [
        transfer_frame_memory(wire::ArtifactFrame {
            payload: Some(wire::ArtifactFramePayload::Chunk(chunk.clone())),
        }),
        transfer_frame_memory(wire::UploadArtifactFrame {
            payload: Some(wire::UploadArtifactFramePayload::Chunk(chunk.clone())),
        }),
        transfer_frame_memory(wire::ExecutionFileDownloadFrame {
            payload: Some(wire::ExecutionFileDownloadFramePayload::Chunk(
                chunk.clone(),
            )),
        }),
        transfer_frame_memory(wire::ExecutionFileFollowFrame {
            payload: Some(wire::ExecutionFileFollowFramePayload::Chunk(chunk.clone())),
        }),
        transfer_frame_memory(wire::AppearanceAssetFrame {
            payload: Some(wire::AppearanceAssetFramePayload::Chunk(chunk.clone())),
        }),
    ]
    .into_iter()
    .fold(32, usize::max);
    MAX_TRANSFER_CHUNK_BYTES.min(max_message_bytes.saturating_sub(overhead))
}

pub(super) fn metadata_memory(record: &ArtifactRecord) -> Result<usize, Status> {
    artifact_digest(record)?;
    if record.status == ArtifactStatus::Staged {
        return Err(Status::data_loss("invalid stored artifact metadata"));
    }
    let mut bytes = std::mem::size_of::<dokime_contract::operations::content::ArtifactMetadata>()
        .saturating_add(record.artifact_id.as_str().len())
        .saturating_add(record.upload_id.as_str().len())
        .saturating_add(record.filename.len())
        .saturating_add(record.content_type.as_ref().map_or(0, String::len))
        .saturating_add(64);
    for id in [
        record.scope.test_cycle_id.as_ref().map(|id| id.as_str()),
        record.scope.test_case_id.as_ref().map(|id| id.as_str()),
        record
            .scope
            .test_execution_id
            .as_ref()
            .map(|id| id.as_str()),
    ]
    .into_iter()
    .flatten()
    {
        bytes = bytes.saturating_add(id.len());
    }
    if bytes == usize::MAX {
        return Err(Status::resource_exhausted(
            "artifact metadata construction overflows",
        ));
    }
    Ok(bytes)
}

pub(super) fn metadata(
    record: &ArtifactRecord,
) -> Result<dokime_contract::operations::content::ArtifactMetadata, Status> {
    metadata_memory(record)?;
    let invalid = || Status::data_loss("invalid stored artifact metadata");
    let sha256 = artifact_digest(record)?.to_owned();
    let created = time::OffsetDateTime::parse(
        record.created_at.as_str(),
        &time::format_description::well_known::Rfc3339,
    )
    .map_err(|_| invalid())?;
    Ok(dokime_contract::operations::content::ArtifactMetadata {
        artifact_id: record.artifact_id.as_str().to_owned(),
        upload_id: record.upload_id.as_str().to_owned(),
        scope: Some(dokime_contract::operations::content::ArtifactScope {
            cycle_id: record
                .scope
                .test_cycle_id
                .as_ref()
                .map(|id| id.as_str().to_owned()),
            case_id: record
                .scope
                .test_case_id
                .as_ref()
                .map(|id| id.as_str().to_owned()),
            execution_id: record
                .scope
                .test_execution_id
                .as_ref()
                .map(|id| id.as_str().to_owned()),
        }),
        filename: record.filename.clone(),
        content_type: record.content_type.clone(),
        size_bytes: record.size_bytes,
        sha256,
        state: match record.status {
            ArtifactStatus::Complete => {
                dokime_contract::operations::content::ArtifactState::Complete
            }
            ArtifactStatus::Cancelled => {
                dokime_contract::operations::content::ArtifactState::Cancelled
            }
            ArtifactStatus::Rejected => {
                dokime_contract::operations::content::ArtifactState::Rejected
            }
            ArtifactStatus::Staged => return Err(invalid()),
        },
        created_at: Some(dokime_contract::operations::common::Timestamp {
            seconds: created.unix_timestamp(),
            nanos: created.nanosecond() as i32,
        }),
    })
}

/* === Private Functions === */
fn transfer_frame_memory<T: harmos::Protobuf>(frame: T) -> usize {
    let retained = std::mem::size_of::<T>()
        .saturating_add(std::mem::size_of::<harmos::operations::StreamItem<T>>());
    let decoded = T::decoding_memory(&frame.encode_to_vec())
        .unwrap_or(usize::MAX)
        .saturating_add(std::mem::size_of::<harmos::grpc::InputEnvelope<T, T>>());
    retained.max(decoded)
}

fn artifact_digest(record: &ArtifactRecord) -> Result<&str, Status> {
    record
        .sha256
        .as_deref()
        .filter(|digest| {
            digest.len() == 64
                && digest
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        })
        .ok_or_else(|| Status::data_loss("invalid stored artifact metadata"))
}

/* === Tests === */
