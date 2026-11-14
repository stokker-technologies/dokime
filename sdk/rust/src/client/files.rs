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
use dokime_protocol::client::files::{UploadsAppendRequest, UploadsCreateRequest, UploadsDeleteRequest, UploadsPublishRequest};
use harmos_protocol::storage::{FileMetadata, FileReference};
use sha2::{Digest, Sha256};
use std::{fs::File, io::Read, path::Path};
use tonic::{Request, Status};
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
            .create(Request::new(UploadsCreateRequest {
                metadata: Some(FileMetadata {
                    expected_length: Some(length),
                    ..Default::default()
                }),
                request_id: format!("{request_id}/create"),
            }))
            .await?
            .into_inner();
        let session = created
            .production
            .and_then(|production| production.session)
            .ok_or_else(|| Status::data_loss("Upload File: Missing Writer Session"))?;
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
                    .append(Request::new(UploadsAppendRequest {
                        session: Some(session.clone()),
                        offset,
                        data: buffer[..size].to_vec(),
                        request_id: format!("{request_id}/append/{offset}"),
                    }))
                    .await?
                    .into_inner();
                offset += size as u64;
                if reply.production.as_ref().is_none_or(|production| production.length != offset) {
                    return Err(Status::data_loss("Upload File: Invalid Accepted Offset"));
                }
                digest.update(&buffer[..size]);
            }
            let digest = digest.finalize().to_vec();
            let reply = self
                .uploads
                .publish(Request::new(UploadsPublishRequest {
                    session: Some(session.clone()),
                    length: offset,
                    sha256: digest.clone(),
                    request_id: format!("{request_id}/publish"),
                }))
                .await?
                .into_inner();
            let reference = reply.file.ok_or_else(|| Status::data_loss("Upload File: Missing Published Reference"))?;
            if reference.length != offset || reference.sha256 != digest {
                return Err(Status::data_loss("Upload File: Published Length or SHA-256 Changed"));
            }
            Ok(reference)
        }
        .await;
        if work.is_err() {
            self.uploads
                .delete(Request::new(UploadsDeleteRequest {
                    session: Some(session),
                    request_id: format!("{request_id}/abandon"),
                }))
                .await
                .map_err(|error| Status::unknown(format!("Abandon File Upload After {work:?}: {error}")))?;
        }
        work
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
