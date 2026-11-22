/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Fenced Artifact Uploads

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::io::Read;

// Third Party Imports
use sha2::{Digest, Sha256};

// Local Imports
use crate::client::resources::Files;
use crate::client::{
    CallOptions, FilesUploadsAppendOptions, FilesUploadsCreateOptions, FilesUploadsDeleteOptions, FilesUploadsPublishOptions,
};
use crate::templates::Context;
use crate::wire::host::files::{
    UploadsAppendRequest, UploadsAppendResponse, UploadsCreateRequest, UploadsCreateResponse, UploadsDeleteRequest, UploadsPublishRequest,
    UploadsPublishResponse,
};
use crate::{Error, FileMetadata, FileReference, Result, Upload};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

pub(crate) enum Uploads<'a> {
    Client { files: &'a Files, context: CallOptions },
    Template(&'a Context),
}

/* === Traits === */

/* === Implementations === */

impl Uploads<'_> {
    async fn create(&self, metadata: FileMetadata, request_id: String) -> Result<UploadsCreateResponse> {
        match self {
            Self::Client { files, context } => Ok(files
                .uploads
                .create_with(
                    metadata,
                    FilesUploadsCreateOptions {
                        request_id: Some(request_id),
                        context: context.clone(),
                    },
                )
                .await?),
            Self::Template(context) => context.operation_with(UploadsCreateRequest { metadata: Some(metadata), request_id }, None).await,
        }
    }
    async fn append(&self, upload: Upload, offset: u64, data: Vec<u8>, request_id: String) -> Result<UploadsAppendResponse> {
        match self {
            Self::Client { files, context } => Ok(files
                .uploads
                .append_with(
                    upload,
                    offset,
                    data,
                    FilesUploadsAppendOptions {
                        request_id: Some(request_id),
                        context: context.clone(),
                    },
                )
                .await?),
            Self::Template(context) => {
                context
                    .operation_with(
                        UploadsAppendRequest {
                            upload: Some(upload),
                            offset,
                            data,
                            request_id,
                        },
                        None,
                    )
                    .await
            }
        }
    }
    async fn publish(&self, upload: Upload, length: u64, sha256: Vec<u8>, request_id: String) -> Result<UploadsPublishResponse> {
        match self {
            Self::Client { files, context } => Ok(files
                .uploads
                .publish_with(
                    upload,
                    length,
                    sha256,
                    FilesUploadsPublishOptions {
                        request_id: Some(request_id),
                        context: context.clone(),
                    },
                )
                .await?),
            Self::Template(context) => {
                context
                    .operation_with(
                        UploadsPublishRequest {
                            upload: Some(upload),
                            length,
                            sha256,
                            request_id,
                        },
                        None,
                    )
                    .await
            }
        }
    }
    async fn abandon(&self, upload: Upload, request_id: String) -> Result<()> {
        match self {
            Self::Client { files, context } => {
                files
                    .uploads
                    .delete_with(
                        upload,
                        FilesUploadsDeleteOptions {
                            request_id: Some(request_id),
                            context: context.clone(),
                        },
                    )
                    .await?;
            }
            Self::Template(context) => {
                context.operation_with(UploadsDeleteRequest { upload: Some(upload), request_id }, None).await?;
            }
        }
        Ok(())
    }
}

/* === Public Functions === */

pub(crate) async fn upload(endpoint: Uploads<'_>, mut reader: impl Read, length: u64, request_id: &str) -> Result<FileReference> {
    let created = endpoint
        .create(
            FileMetadata {
                expected_length: Some(length),
                ..Default::default()
            },
            format!("{request_id}/create"),
        )
        .await?;
    let mut upload = created.upload.ok_or_else(|| invalid("Return the Created Upload Handle"))?;
    let mut published = false;
    let work = async {
        let chunk = usize::try_from(created.chunk_bytes)
            .ok()
            .filter(|chunk| *chunk > 0 && (*chunk as u64) < created.message_bytes)
            .ok_or_else(|| invalid("Return Valid Transfer Limits"))?;
        let mut buffer = vec![0; chunk];
        let mut digest = Sha256::new();
        let mut offset = 0_u64;
        loop {
            let size =
                reader.read(&mut buffer).map_err(|error| Error::Invalid(format!("Upload Artifact: Read the Source Bytes: {error}")))?;
            if size == 0 {
                break;
            }
            let reply = endpoint.append(upload.clone(), offset, buffer[..size].to_vec(), format!("{request_id}/append/{offset}")).await?;
            let acknowledged = reply.upload.ok_or_else(|| invalid("Return the Acknowledged Upload Handle"))?;
            offset += size as u64;
            if acknowledged.accepted_length != offset
                || acknowledged.id != upload.id
                || acknowledged.history != upload.history
                || acknowledged.writer_generation != upload.writer_generation
            {
                return Err(invalid("Preserve the Upload Identity, Fence and Accepted Offset"));
            }
            upload = acknowledged;
            digest.update(&buffer[..size]);
        }
        if offset != length {
            return Err(invalid("Keep the Source Length Stable During Upload"));
        }
        let sha256 = digest.finalize().to_vec();
        let reply = endpoint.publish(upload.clone(), offset, sha256.clone(), format!("{request_id}/publish")).await?;
        published = true;
        let file = reply.upload.ok_or_else(|| invalid("Return the Published File Reference"))?;
        if file.length != offset || file.sha256 != sha256 {
            return Err(invalid("Return the Published Length and SHA-256 Without Changes"));
        }
        Ok(file)
    }
    .await;
    if let Err(error) = work {
        if !published && let Err(cleanup) = endpoint.abandon(upload, format!("{request_id}/abandon")).await {
            return Err(Error::Invalid(format!("Upload Artifact: {error}; Abandon Unpublished Upload: {cleanup}")));
        }
        return Err(error);
    }
    work
}

/* === Private Functions === */

fn invalid(action: &str) -> Error {
    Error::Invalid(format!("Upload Artifact: {action}"))
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use prost::Message;
    use sha2::{Digest, Sha256};

    use crate::context::{Context, Execution, Local, LocalHost};
    use crate::operation::FutureResult;
    use crate::wire::host::content::{ArtifactsCreateRequest, ArtifactsCreateResponse};
    use crate::wire::host::files::{
        UploadsAppendRequest, UploadsAppendResponse, UploadsCreateRequest, UploadsCreateResponse, UploadsDeleteRequest,
        UploadsDeleteResponse, UploadsPublishRequest, UploadsPublishResponse,
    };
    use crate::{ArtifactMetadata, Error, FileReference, Origin, Result, TestCase, Upload};

    #[derive(Default)]
    struct State {
        bytes: Vec<u8>,
        identities: Vec<String>,
        abandoned: bool,
        attached: bool,
    }

    struct Host {
        state: Mutex<State>,
        corrupt_fence: bool,
        corrupt_digest: bool,
    }

    impl LocalHost for Host {
        fn call<'a>(&'a self, operation: &'static str, request: Vec<u8>) -> FutureResult<'a, Vec<u8>> {
            Box::pin(async move {
                let mut state = self.state.lock().unwrap();
                let decode = |error| Error::Invalid(format!("Decode Upload Fixture: {error}"));
                match operation {
                    "dokime.host.v1.files.Uploads/Create" => {
                        let request = UploadsCreateRequest::decode(request.as_slice()).map_err(decode)?;
                        state.identities.push(request.request_id);
                        Ok(UploadsCreateResponse {
                            upload: Some(Upload {
                                id: "owned".into(),
                                writer_generation: 7,
                                ..Default::default()
                            }),
                            chunk_bytes: 3,
                            message_bytes: 16,
                            ..Default::default()
                        }
                        .encode_to_vec())
                    }
                    "dokime.host.v1.files.Uploads/Append" => {
                        let request = UploadsAppendRequest::decode(request.as_slice()).map_err(decode)?;
                        assert_eq!(request.offset, state.bytes.len() as u64);
                        state.identities.push(request.request_id);
                        state.bytes.extend(request.data);
                        let mut upload = request.upload.unwrap();
                        upload.accepted_length = state.bytes.len() as u64;
                        if self.corrupt_fence {
                            upload.writer_generation += 1;
                        }
                        Ok(UploadsAppendResponse {
                            upload: Some(upload),
                            ..Default::default()
                        }
                        .encode_to_vec())
                    }
                    "dokime.host.v1.files.Uploads/Publish" => {
                        let request = UploadsPublishRequest::decode(request.as_slice()).map_err(decode)?;
                        assert_eq!(request.length, state.bytes.len() as u64);
                        assert_eq!(request.sha256, Sha256::digest(&state.bytes).as_slice());
                        state.identities.push(request.request_id);
                        let mut sha256 = request.sha256;
                        if self.corrupt_digest {
                            sha256[0] ^= 1;
                        }
                        Ok(UploadsPublishResponse {
                            upload: Some(FileReference {
                                length: request.length,
                                sha256,
                                ..Default::default()
                            }),
                            ..Default::default()
                        }
                        .encode_to_vec())
                    }
                    "dokime.host.v1.files.Uploads/Delete" => {
                        let request = UploadsDeleteRequest::decode(request.as_slice()).map_err(decode)?;
                        state.identities.push(request.request_id);
                        state.abandoned = true;
                        Ok(UploadsDeleteResponse::default().encode_to_vec())
                    }
                    "dokime.host.v1.content.Artifacts/Create" => {
                        let request = ArtifactsCreateRequest::decode(request.as_slice()).map_err(decode)?;
                        assert!(request.scope.is_none());
                        assert_eq!(request.file.unwrap().sha256, Sha256::digest(&state.bytes).as_slice());
                        state.identities.push(request.request_id);
                        state.attached = true;
                        Ok(ArtifactsCreateResponse {
                            artifact: Some(ArtifactMetadata {
                                filename: request.filename,
                                size_bytes: state.bytes.len() as u64,
                                ..Default::default()
                            }),
                            ..Default::default()
                        }
                        .encode_to_vec())
                    }
                    _ => panic!("Unexpected Upload Operation: {operation}"),
                }
            })
        }
    }

    #[tokio::test]
    async fn template_artifacts_share_fenced_uploads_and_never_abandon_published_files() -> Result<()> {
        for (corrupt_fence, corrupt_digest) in [(false, false), (true, false), (false, true)] {
            let host = Arc::new(Host {
                state: Mutex::new(State::default()),
                corrupt_fence,
                corrupt_digest,
            });
            let context = Context::local(
                Local {
                    host: host.clone(),
                    scope: "execution".into(),
                    attempt: 1,
                },
                Execution::new(TestCase::default(), Origin::default()),
                Arc::new(()),
            );
            let bytes = b"capture-data".to_vec();
            let result = context.artifact("capture.bin", bytes.clone(), "application/octet-stream").await;
            let state = host.state.lock().unwrap();
            if corrupt_fence || corrupt_digest {
                assert!(result.is_err());
                assert!(!state.attached);
                assert_eq!(state.abandoned, corrupt_fence);
                continue;
            }
            assert_eq!(result?.size_bytes, bytes.len() as u64);
            assert_eq!(state.bytes, bytes);
            assert!(state.attached);
            assert!(!state.abandoned);
            let prefix = state.identities[0].strip_suffix("/create").unwrap();
            let suffixes = state.identities.iter().map(|identity| identity.strip_prefix(prefix).unwrap()).collect::<Vec<_>>();
            assert_eq!(suffixes, ["/create", "/append/0", "/append/3", "/append/6", "/append/9", "/publish", "/attach"]);
        }
        Ok(())
    }
}
