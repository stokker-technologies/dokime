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

use super::resources::Files;
use super::{FilesUploadsAppendOptions, FilesUploadsCreateOptions, FilesUploadsDeleteOptions, FilesUploadsPublishOptions};
use crate::{FileMetadata, FileReference};
use sha2::{Digest, Sha256};
use std::{fs::File, io::Read, path::Path};
use tonic::Status;
use uuid::Uuid;

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
        let mut file =
            File::open(path.as_ref()).map_err(|error| Status::invalid_argument(format!("Upload File: Open Its Path: {error}")))?;
        let length = file.metadata().map_err(|error| Status::invalid_argument(format!("Upload File: Inspect Its Length: {error}")))?.len();
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
