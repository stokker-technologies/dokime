/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// File Upload Convenience

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    fs::File,
    io::{Cursor, Read},
    path::Path,
};

// Third Party Imports
use sha2::{Digest, Sha256};
use tonic::Status;
use uuid::Uuid;

// Local Imports
use super::resources::Files;
use super::{CallOptions, FilesUploadsAppendOptions, FilesUploadsCreateOptions, FilesUploadsDeleteOptions, FilesUploadsPublishOptions};
use crate::{FileMetadata, FileReference};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Files {
    /// Upload a Local File with Bounded Chunks and Verified Publication.
    pub async fn upload(&self, path: impl AsRef<Path>) -> Result<FileReference, Status> {
        let file = File::open(path.as_ref()).map_err(|error| Status::invalid_argument(format!("Upload File: Open Its Path: {error}")))?;
        let length = file.metadata().map_err(|error| Status::invalid_argument(format!("Upload File: Inspect Its Length: {error}")))?.len();
        self.upload_reader(file, length, CallOptions::default()).await
    }

    pub(crate) async fn upload_bytes(&self, data: Vec<u8>, context: CallOptions) -> Result<FileReference, Status> {
        let length = data.len() as u64;
        self.upload_reader(Cursor::new(data), length, context).await
    }

    async fn upload_reader(&self, mut file: impl Read, length: u64, context: CallOptions) -> Result<FileReference, Status> {
        let request_id = Uuid::now_v7().to_string();
        let created = self
            .uploads
            .create_with(
                FileMetadata {
                    expected_length: Some(length),
                    ..Default::default()
                },
                FilesUploadsCreateOptions {
                    request_id: Some(format!("{request_id}/create")),
                    context: context.clone(),
                    ..Default::default()
                },
            )
            .await?;
        let upload = created.upload.ok_or_else(|| Status::data_loss("Upload File: Missing Upload"))?;
        let work = async {
            let chunk = usize::try_from(created.chunk_bytes)
                .ok()
                .filter(|chunk| *chunk > 0 && (*chunk as u64) < created.message_bytes)
                .ok_or_else(|| Status::data_loss("Upload File: Invalid Transfer Limits"))?;
            let mut buffer = vec![0; chunk];
            let mut digest = Sha256::new();
            let mut offset = 0u64;
            loop {
                let size =
                    file.read(&mut buffer).map_err(|error| Status::invalid_argument(format!("Upload File: Read Its Path: {error}")))?;
                if size == 0 {
                    break;
                }
                let reply = self
                    .uploads
                    .append_with(
                        upload.clone(),
                        offset,
                        buffer[..size].to_vec(),
                        FilesUploadsAppendOptions {
                            request_id: Some(format!("{request_id}/append/{offset}")),
                            context: context.clone(),
                            ..Default::default()
                        },
                    )
                    .await?;
                offset += size as u64;
                if reply.upload.as_ref().is_none_or(|upload| upload.accepted_length != offset) {
                    return Err(Status::data_loss("Upload File: Invalid Accepted Offset"));
                }
                digest.update(&buffer[..size]);
            }
            let digest = digest.finalize().to_vec();
            let reply = self
                .uploads
                .publish_with(
                    upload.clone(),
                    offset,
                    digest.clone(),
                    FilesUploadsPublishOptions {
                        request_id: Some(format!("{request_id}/publish")),
                        context: context.clone(),
                        ..Default::default()
                    },
                )
                .await?;
            let reference = reply.upload.ok_or_else(|| Status::data_loss("Upload File: Missing Published Reference"))?;
            if reference.length != offset || reference.sha256 != digest {
                return Err(Status::data_loss("Upload File: Published Length or SHA-256 Changed"));
            }
            Ok(reference)
        }
        .await;
        if work.is_err() {
            self.uploads
                .delete_with(
                    upload,
                    FilesUploadsDeleteOptions {
                        request_id: Some(format!("{request_id}/abandon")),
                        context,
                        ..Default::default()
                    },
                )
                .await
                .map_err(|error| Status::unknown(format!("Abandon File Upload After {work:?}: {error}")))?;
        }
        work
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
