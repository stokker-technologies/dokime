/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Plugin Host File Transfers

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

use crate::{Error, Resources, Result};
use dokime_protocol::FILE_DESCRIPTOR_SET;
use dokime_protocol::{client, context};
use dokime_storage::CONTENT_CHUNK_BYTES;
use harmos_protocol::v1::{ChildRequest, OperationContract};
use prost::Message;
use prost_types::FileDescriptorSet;

/* === Constants === */

const CHUNK_BYTES: usize = 4 * 1024 * 1024;
const MESSAGE_BYTES: u64 = 8 * 1024 * 1024;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declarations() -> Result<Vec<OperationContract>> {
    let types = FileDescriptorSet::decode(FILE_DESCRIPTOR_SET)?;
    Ok(vec![
        crate::declaration::<context::FilesGetRequest, context::FilesGetResponse>("dokime.plugin.host.v1.Files/Get", types.clone())?,
        crate::declaration::<context::FilesReadRequest, context::FilesReadResponse>("dokime.plugin.host.v1.Files/Read", types.clone())?,
        crate::declaration::<context::FilesDeleteRequest, context::FilesDeleteResponse>(
            "dokime.plugin.host.v1.Files/Delete",
            types.clone(),
        )?,
        crate::declaration::<context::files::UploadsCreateRequest, context::files::UploadsCreateResponse>(
            "dokime.plugin.host.v1.files.Uploads/Create",
            types.clone(),
        )?,
        crate::declaration::<context::files::UploadsGetRequest, context::files::UploadsGetResponse>(
            "dokime.plugin.host.v1.files.Uploads/Get",
            types.clone(),
        )?,
        crate::declaration::<context::files::UploadsResumeRequest, context::files::UploadsResumeResponse>(
            "dokime.plugin.host.v1.files.Uploads/Resume",
            types.clone(),
        )?,
        crate::declaration::<context::files::UploadsAppendRequest, context::files::UploadsAppendResponse>(
            "dokime.plugin.host.v1.files.Uploads/Append",
            types.clone(),
        )?,
        crate::declaration::<context::files::UploadsPublishRequest, context::files::UploadsPublishResponse>(
            "dokime.plugin.host.v1.files.Uploads/Publish",
            types.clone(),
        )?,
        crate::declaration::<context::files::UploadsDeleteRequest, context::files::UploadsDeleteResponse>(
            "dokime.plugin.host.v1.files.Uploads/Delete",
            types.clone(),
        )?,
    ])
}

pub async fn dispatch(resources: &Resources, request: &ChildRequest) -> Result<Vec<u8>> {
    let payload = request.payload.as_slice();
    resources.check_action_scope(&resources.execution, &request.operation)?;
    match request.operation.as_str() {
        "dokime.plugin.host.v1.Files/Get" => {
            let request = context::FilesGetRequest::decode(payload)?;
            let selected = request.file.as_ref().ok_or_else(|| Error::Invalid("Access File: Supply Its Reference".into()))?;
            let id = &selected.metadata.as_ref().ok_or_else(|| Error::Invalid("Access File: Supply Its Identity".into()))?.id;
            let entry = resources.store.content(id)?;
            resources.check_scope(&entry.execution)?;
            let cut = resources.store.file_read_cut()?;
            if request.at.as_ref().is_some_and(|at| at != &cut) {
                return Err(Error::Precondition("Read File: Resynchronize Its Current Cut".into()));
            }
            let file = resources.store.file_reference(id)?;
            Ok(context::FilesGetResponse {
                file: Some(file),
                chunk_bytes: CHUNK_BYTES as u64,
                message_bytes: MESSAGE_BYTES,
                resolved_at: Some(cut),
            }
            .encode_to_vec())
        }
        "dokime.plugin.host.v1.Files/Read" => {
            let request = context::FilesReadRequest::decode(payload)?;
            let selected = request.file.as_ref().ok_or_else(|| Error::Invalid("Access File: Supply Its Reference".into()))?;
            let id = &selected.metadata.as_ref().ok_or_else(|| Error::Invalid("Access File: Supply Its Identity".into()))?.id;
            let entry = resources.store.content(id)?;
            resources.check_scope(&entry.execution)?;
            let cut = resources.store.file_read_cut()?;
            if request.at.as_ref().is_some_and(|at| at != &cut) {
                return Err(Error::Precondition("Read File: Resynchronize Its Current Cut".into()));
            }
            let maximum = if request.max_bytes == 0 {
                CHUNK_BYTES
            } else {
                usize::try_from(request.max_bytes).map_err(|_| Error::Bound("Read File: Respect Its Chunk Bound".into()))?
            };
            if maximum > CHUNK_BYTES || request.offset > selected.length || selected.length > entry.length {
                return Err(Error::Bound("Read File: Stay Within Its Selected Length and Chunk Bound".into()));
            }
            let mut data = Vec::new();
            let length = maximum.min((selected.length - request.offset) as usize);
            while data.len() < length {
                let bytes = resources.store.content_window(
                    id,
                    request.offset + data.len() as u64,
                    (length - data.len()).min(CONTENT_CHUNK_BYTES),
                )?;
                if bytes.is_empty() {
                    return Err(Error::Precondition("Read File: Restore Its Missing Content".into()));
                }
                data.extend(bytes);
            }
            Ok(context::FilesReadResponse {
                data,
                total_length: selected.length,
                sha256: selected.sha256.clone(),
                resolved_at: Some(cut),
            }
            .encode_to_vec())
        }
        "dokime.plugin.host.v1.Files/Delete" => {
            let request = context::FilesDeleteRequest::decode(payload)?;
            let selected = request.file.as_ref().ok_or_else(|| Error::Invalid("Access File: Supply Its Reference".into()))?;
            let id = &selected.metadata.as_ref().ok_or_else(|| Error::Invalid("Access File: Supply Its Identity".into()))?.id;
            let entry = resources.store.content(id)?;
            resources.check_scope(&entry.execution)?;
            let value = client::FilesDeleteRequest::decode(request.encode_to_vec().as_slice())?;
            Ok(resources.store.delete_file(value)?.encode_to_vec())
        }
        "dokime.plugin.host.v1.files.Uploads/Create" => {
            let request = context::files::UploadsCreateRequest::decode(payload)?;
            let mut value = client::files::UploadsCreateRequest::decode(request.encode_to_vec().as_slice())?;
            let metadata = value.metadata.as_mut().ok_or_else(|| Error::Invalid("Create Upload: Supply Metadata".into()))?;
            if metadata.scope.as_ref().is_none_or(|scope| scope.is_empty()) {
                metadata.scope = Some(resources.execution.clone());
            }
            resources.check_scope(metadata.scope.as_deref().unwrap_or_default())?;
            Ok(resources.store.create_file_upload(value, CHUNK_BYTES as u64, MESSAGE_BYTES)?.encode_to_vec())
        }
        "dokime.plugin.host.v1.files.Uploads/Get" => {
            let request = context::files::UploadsGetRequest::decode(payload)?;
            let file = request.file.as_ref().ok_or_else(|| Error::Invalid("Get Upload: Supply Its Identity".into()))?;
            let (metadata, production, generation) = resources.store.file_upload(file)?;
            resources.check_scope(metadata.scope.as_deref().unwrap_or_default())?;
            let cut = resources.store.file_read_cut()?;
            if request.at.as_ref().is_some_and(|at| at != &cut) {
                return Err(Error::Precondition("Get Upload: Resynchronize Its Current Durable Progress".into()));
            }
            Ok(context::files::UploadsGetResponse {
                accepted_length: production.length,
                durable_length: production.length,
                production: Some(production),
                writer_generation: generation,
                chunk_bytes: CHUNK_BYTES as u64,
                message_bytes: MESSAGE_BYTES,
                resolved_at: Some(cut),
            }
            .encode_to_vec())
        }
        "dokime.plugin.host.v1.files.Uploads/Resume" => {
            let request = context::files::UploadsResumeRequest::decode(payload)?;
            let session = request.session.as_ref().ok_or_else(|| Error::Invalid("Write Upload: Supply Its Session".into()))?;
            let metadata = resources
                .store
                .file_upload_metadata(&session.id)?
                .ok_or_else(|| Error::NotFound("Write Upload: Select an Existing Upload".into()))?;
            resources.check_scope(metadata.scope.as_deref().unwrap_or_default())?;
            let value = client::files::UploadsResumeRequest::decode(request.encode_to_vec().as_slice())?;
            Ok(resources.store.resume_file_upload(value, CHUNK_BYTES as u64, MESSAGE_BYTES)?.encode_to_vec())
        }
        "dokime.plugin.host.v1.files.Uploads/Append" => {
            let request = context::files::UploadsAppendRequest::decode(payload)?;
            let session = request.session.as_ref().ok_or_else(|| Error::Invalid("Write Upload: Supply Its Session".into()))?;
            let metadata = resources
                .store
                .file_upload_metadata(&session.id)?
                .ok_or_else(|| Error::NotFound("Write Upload: Select an Existing Upload".into()))?;
            resources.check_scope(metadata.scope.as_deref().unwrap_or_default())?;
            let value = client::files::UploadsAppendRequest::decode(request.encode_to_vec().as_slice())?;
            Ok(resources.store.append_file_upload(value, CHUNK_BYTES)?.encode_to_vec())
        }
        "dokime.plugin.host.v1.files.Uploads/Publish" => {
            let request = context::files::UploadsPublishRequest::decode(payload)?;
            let session = request.session.as_ref().ok_or_else(|| Error::Invalid("Write Upload: Supply Its Session".into()))?;
            let metadata = resources
                .store
                .file_upload_metadata(&session.id)?
                .ok_or_else(|| Error::NotFound("Write Upload: Select an Existing Upload".into()))?;
            resources.check_scope(metadata.scope.as_deref().unwrap_or_default())?;
            let value = client::files::UploadsPublishRequest::decode(request.encode_to_vec().as_slice())?;
            Ok(resources.store.publish_file_upload(value)?.encode_to_vec())
        }
        "dokime.plugin.host.v1.files.Uploads/Delete" => {
            let request = context::files::UploadsDeleteRequest::decode(payload)?;
            let session = request.session.as_ref().ok_or_else(|| Error::Invalid("Write Upload: Supply Its Session".into()))?;
            let metadata = resources
                .store
                .file_upload_metadata(&session.id)?
                .ok_or_else(|| Error::NotFound("Write Upload: Select an Existing Upload".into()))?;
            resources.check_scope(metadata.scope.as_deref().unwrap_or_default())?;
            let value = client::files::UploadsDeleteRequest::decode(request.encode_to_vec().as_slice())?;
            Ok(resources.store.delete_file_upload(value)?.encode_to_vec())
        }
        _ => Err(Error::Invalid("Call File Operation: Select a Declared Unary Method".into())),
    }
}

/* === Private Functions === */

/* === Tests === */
