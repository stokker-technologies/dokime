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
pub(super) mod evidence;
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
use dokime::{
    RuntimeHandle,
    artifacts::{ArtifactRecord, ArtifactStatus},
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) fn register(runtime: &RuntimeHandle, maximum: usize) -> Result<(), crate::grpc::Error> {
    evidence::register(runtime, maximum)?;
    artifacts::register(runtime, maximum)
}

pub(super) fn download_chunk_bytes(runtime: &RuntimeHandle, maximum: usize) -> Result<usize, Status> {
    let bytes = runtime.storage().info().map_err(super::bundle::native_failure)?.file_batch_bytes;
    let bytes = usize::try_from(bytes).map_err(|_| Status::resource_exhausted("native file batch size overflows"))?;
    Ok(bytes.min(maximum).min(1_048_576))
}

pub(super) fn transfer_chunk_bytes(runtime: &RuntimeHandle, maximum: usize) -> Result<usize, Status> {
    use dokime::domain::model::{content::wire as content, plugins::wire as plugins};
    let cap = download_chunk_bytes(runtime, maximum)?;
    let artifact = content::UploadArtifactFrame {
        payload: Some(content::UploadArtifactFramePayload::Chunk(content::ArtifactChunk { offset: u64::MAX, data: Vec::new() })),
    };
    let plugin = plugins::InstallPluginFrame {
        frame: Some(plugins::InstallPluginFrameFrame::Chunk(plugins::InstallPluginChunk { offset: u64::MAX, data: Vec::new() })),
    };
    let artifact = input_allowance::<dokime::operations::content::UploadArtifact>(
        &artifact,
        &["payload", "Chunk", "field_0", "data"],
        cap,
        maximum,
    )?;
    let plugin =
        input_allowance::<dokime::operations::plugins::Install>(&plugin, &["frame", "Chunk", "field_0", "data"], cap, maximum)?;
    Ok(artifact.min(plugin))
}

pub(super) fn output_chunk_bytes<T: dokime::harmos::domain::Portable>(template: &T, cap: usize, maximum: usize) -> Result<usize, Status> {
    dokime::harmos::grpc::output_payload_limit(template, &["payload", "Chunk", "field_0", "data"], cap, maximum)
        .map_err(|error| Status::internal(error.to_string()))
}

pub(super) fn output_frame_len<T: dokime::harmos::domain::Portable>(frame: &T) -> Result<usize, Status> {
    dokime::harmos::grpc::output_item_len(frame).map_err(|error| Status::internal(error.to_string()))
}

pub(super) fn encoding_failure(source: Status) -> dokime::harmos::Error {
    if source.code() == tonic::Code::ResourceExhausted {
        return dokime::harmos::Error::ResourceExhausted { resource: "gRPC frame encoding" };
    }
    dokime::harmos::Error::Backend {
        operation: "encode gRPC frame",
        kind: dokime::harmos::BackendFailure::Permanent,
        source: Box::new(source),
    }
}

pub(super) fn metadata(record: &ArtifactRecord) -> Result<dokime::domain::model::content::wire::ArtifactMetadata, Status> {
    if record.status == ArtifactStatus::Staged {
        return Err(Status::data_loss("invalid stored artifact metadata"));
    }
    let invalid = || Status::data_loss("invalid stored artifact metadata");
    let sha256 = artifact_digest(record)?.to_owned();
    let created =
        time::OffsetDateTime::parse(record.created_at.as_str(), &time::format_description::well_known::Rfc3339).map_err(|_| invalid())?;
    Ok(dokime::domain::model::content::wire::ArtifactMetadata {
        artifact_id: record.artifact_id.as_str().to_owned(),
        upload_id: record.upload_id.as_str().to_owned(),
        scope: Some(dokime::domain::model::content::wire::ArtifactScope {
            cycle_id: record.scope.test_cycle_id.as_ref().map(|id| id.as_str().to_owned()),
            case_id: record.scope.test_case_id.as_ref().map(|id| id.as_str().to_owned()),
            execution_id: record.scope.test_execution_id.as_ref().map(|id| id.as_str().to_owned()),
        }),
        filename: record.filename.clone(),
        content_type: record.content_type.clone(),
        size_bytes: record.size_bytes,
        sha256,
        state: match record.status {
            ArtifactStatus::Complete => dokime::domain::model::content::wire::ArtifactState::Complete,
            ArtifactStatus::Cancelled => dokime::domain::model::content::wire::ArtifactState::Cancelled,
            ArtifactStatus::Rejected => dokime::domain::model::content::wire::ArtifactState::Rejected,
            ArtifactStatus::Staged => return Err(invalid()),
        },
        created_at: Some(dokime::domain::model::identity::wire::Timestamp {
            seconds: created.unix_timestamp(),
            nanos: created.nanosecond() as i32,
        }),
    })
}

/* === Private Functions === */

fn input_allowance<O: dokime::harmos::operations::OperationDeclaration>(
    template: &<O::Input as dokime::harmos::operations::Direction>::Item, path: &[&str], cap: usize, maximum: usize,
) -> Result<usize, Status> {
    dokime::harmos::grpc::input_payload_limit::<O>(template, path, cap, maximum).map_err(|error| Status::internal(error.to_string()))
}

fn artifact_digest(record: &ArtifactRecord) -> Result<&str, Status> {
    record
        .sha256
        .as_deref()
        .filter(|digest| digest.len() == 64 && digest.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)))
        .ok_or_else(|| Status::data_loss("invalid stored artifact metadata"))
}

/* === Tests === */
