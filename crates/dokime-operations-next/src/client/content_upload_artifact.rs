/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Upload an Immutable Artifact

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use prost::Message;

// Local Imports
use crate::{Error, Result, client::Context};
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.content.upload_artifact";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct Staged {
    store: dokime_storage::Store,
    identifier: String,
}

/* === Enums === */

pub enum Input {
    Request,
    Item(v1::UploadArtifactFrame),
    Cancel,
    Finish,
}

/* === Traits === */

/* === Implementations === */

impl Drop for Staged {
    fn drop(&mut self) {
        let _ = self.store.abort_artifact(&self.identifier);
    }
}

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::ContentUploadArtifactRequest, v1::ArtifactMetadata>(ID, settings.mutation_timeout)
}
pub async fn handle<I: prost::Message + Default>(
    context: &Context, mut input: tonic::Streaming<I>, decode: fn(I) -> Option<Input>,
) -> Result<v1::ArtifactMetadata> {
    super::complete(ID, context.settings.mutation_timeout, async {
        let first = input.message().await.map_err(|error| Error::Invalid(error.to_string()))?;
        if !matches!(first.and_then(decode), Some(Input::Request)) {
            return Err(Error::Invalid("Upload Artifact: Send One Request Before the Header".into()));
        }
        let header = input.message().await.map_err(|error| Error::Invalid(error.to_string()))?.and_then(decode);
        let Some(Input::Item(v1::UploadArtifactFrame {
            payload:
                Some(v1::UploadArtifactFramePayload {
                    kind: Some(v1::upload_artifact_frame_payload::Kind::Header(mut header)),
                }),
        })) = header
        else {
            return Err(Error::Invalid("Upload Artifact: Send Its Header First".into()));
        };
        header.filename = header.filename.trim().to_owned();
        header.content_type = header.content_type.filter(|mime| !mime.is_empty());
        dokime_storage::validate_artifact_name(&header.filename).map_err(|error| Error::Invalid(error.to_string()))?;
        if header.content_type.as_ref().is_some_and(|mime| mime.len() > 256 || mime.chars().any(char::is_control))
            || header.size_bytes > i64::MAX as u64
        {
            return Err(Error::Invalid("Upload Artifact: Supply a Valid Media Type and File Length".into()));
        }
        let scope = header.scope.as_ref().ok_or_else(|| Error::Invalid("Upload Artifact: Select Its Execution and Cycle".into()))?;
        let execution = scope.execution_id.as_deref().ok_or_else(|| Error::Invalid("Upload Artifact: Select Its Execution".into()))?;
        let store = context.host.store();
        let cut = store.metadata(execution)?;
        if scope.cycle_id.as_deref() != Some(&cut.execution.cycle) || scope.case_id.is_some() {
            return Err(Error::Invalid("Upload Artifact: Use the Execution's Cycle and Frozen Origin Without a Planned Case".into()));
        }
        let identifier = uuid::Uuid::now_v7().to_string();
        let now = context.host.now_ns();
        let metadata = v1::ArtifactMetadata {
            artifact_id: identifier.clone(),
            upload_id: identifier.clone(),
            scope: header.scope.clone(),
            filename: header.filename,
            content_type: header.content_type,
            size_bytes: header.size_bytes,
            sha256: header.sha256.unwrap_or_default(),
            created_at: Some(prost_types::Timestamp {
                seconds: now.div_euclid(1_000_000_000),
                nanos: now.rem_euclid(1_000_000_000) as i32,
            }),
            ..Default::default()
        };
        store.begin_artifact(metadata)?;
        let _staged = Staged {
            store: store.clone(),
            identifier: identifier.clone(),
        };
        let maximum = super::content_transfer::maximum(context, 0)?;
        let mut offset = 0;
        let mut completed = None;
        loop {
            let Some(frame) = input.message().await.map_err(|error| Error::Invalid(error.to_string()))? else {
                break;
            };
            let value =
                decode(frame).ok_or_else(|| Error::Invalid("Upload Artifact: Send a Chunk, Completion, Finish or Cancel Frame".into()))?;
            match value {
                Input::Cancel => return Err(Error::Cancelled("Operation Cancelled by the Client".into())),
                Input::Finish => break,
                Input::Item(frame) if completed.is_none() => match frame.payload.and_then(|payload| payload.kind) {
                    Some(v1::upload_artifact_frame_payload::Kind::Chunk(chunk)) => {
                        if chunk.offset != offset
                            || chunk.data.is_empty()
                            || chunk.data.len() > maximum
                            || offset + chunk.data.len() as u64 > header.size_bytes
                        {
                            return Err(Error::Invalid(
                                "Upload Artifact: Send Contiguous Chunks Within Its Declared Length and 64 KiB Bound".into(),
                            ));
                        }
                        store.append_content(&identifier, offset, &chunk.data)?;
                        offset += chunk.data.len() as u64;
                    }
                    Some(v1::upload_artifact_frame_payload::Kind::Complete(complete)) => {
                        if complete.size_bytes != offset || offset != header.size_bytes {
                            return Err(Error::Invalid("Complete Artifact: Match the Declared Length".into()));
                        }
                        completed = Some(complete);
                    }
                    _ => return Err(Error::Invalid("Upload Artifact: Send Only Chunks Then One Digest Completion".into())),
                },
                _ => return Err(Error::Invalid("Upload Artifact: Finish Input After Its Digest Completion".into())),
            }
        }
        let completed =
            completed.ok_or_else(|| Error::Invalid("Upload Artifact: Send an Explicit Digest Completion Before Finishing".into()))?;
        let metadata = store.finish_artifact(&identifier, completed.size_bytes, &completed.sha256)?;
        if metadata.encoded_len() > context.settings.message_bytes {
            return Err(Error::Invalid("Upload Artifact: Shorten Its Metadata to Fit the Message Budget".into()));
        }
        Ok(metadata)
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
