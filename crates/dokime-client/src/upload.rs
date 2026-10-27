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
use std::{future::Future, path::Path};

// Third Party Imports
use sha2::{Digest, Sha256};
use tokio::io::{AsyncRead, AsyncReadExt};

// Local Imports
use super::Client;

/* === Constants === */

const PLUGIN_CHUNK_BYTES: u64 = 65_536;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Client(#[from] super::Error),

    #[error("upload filesystem: {0}")]
    Io(#[from] std::io::Error),

    #[error("invalid upload: {0}")]
    Invalid(&'static str),
}

/* === Traits === */

/* === Implementations === */

impl Client {
    pub async fn upload_artifact_file(
        &self, source: &Path, mut header: dokime_domain::model::content::wire::UploadArtifactHeader,
    ) -> Result<dokime_domain::model::content::wire::ArtifactMetadata, Error> {
        let file = tokio::fs::File::open(source).await?;
        header.size_bytes = file.metadata().await?.len();
        header.sha256 = None;
        let template = dokime_domain::model::content::wire::UploadArtifactFrame {
            payload: Some(dokime_domain::model::content::wire::UploadArtifactFramePayload::Chunk(
                dokime_domain::model::content::wire::ArtifactChunk { offset: header.size_bytes, data: Vec::new() },
            )),
        };
        let chunk_bytes = super::plugins::chunk_bytes::<dokime_operations::content::UploadArtifact>(
            self.backend().limits.max_chunk_bytes,
            self.request_limit(),
            PLUGIN_CHUNK_BYTES as usize,
            &template,
            &["payload", "Chunk", "field_0", "data"],
        )?;
        if chunk_bytes == 0 {
            return Err(Error::Invalid("negotiated frame limit cannot carry upload data"));
        }
        let upload = self.content_upload_artifact_open()?;
        let produce = async {
            let completion = feed(file, header, chunk_bytes, |frame| upload.send(frame)).await?;
            upload.end_input().await?;
            Ok::<_, Error>(completion)
        };
        let response = async { upload.response().await.map_err(Error::from) };
        // Both futures belong to this call; there is no detached file feeder.
        let result = tokio::try_join!(biased; response, produce);
        upload.close().await;
        let (metadata, completion) = result?;
        if metadata.artifact_id.is_empty()
            || metadata.state != dokime_domain::model::content::wire::ArtifactState::Complete
            || metadata.size_bytes != completion.size_bytes
            || metadata.sha256 != completion.sha256
        {
            return Err(Error::Invalid("server receipt differs from uploaded content"));
        }
        Ok(metadata)
    }
}

/* === Public Functions === */

/* === Private Functions === */

async fn feed<F: Future<Output = super::Result<()>>>(
    mut source: impl AsyncRead + Unpin, header: dokime_domain::model::content::wire::UploadArtifactHeader, chunk_bytes: usize,
    mut send: impl FnMut(dokime_domain::model::content::wire::UploadArtifactFrame) -> F,
) -> Result<dokime_domain::model::content::wire::ArtifactCompletion, Error> {
    use dokime_domain::model::content::wire::UploadArtifactFramePayload as Payload;
    let expected = header.size_bytes;
    send(dokime_domain::model::content::wire::UploadArtifactFrame { payload: Some(Payload::Header(header)) }).await?;
    let mut digest = Sha256::new();
    let mut offset = 0u64;
    while offset < expected {
        let remaining = usize::try_from(expected - offset).unwrap_or(usize::MAX);
        let mut data = vec![0; chunk_bytes.min(remaining)];
        let count = source.read(&mut data).await?;
        if count == 0 {
            break;
        }
        let next = offset.checked_add(count as u64).ok_or(Error::Invalid("file size overflow"))?;
        if next > expected {
            return Err(Error::Invalid("source length changed during upload"));
        }
        data.truncate(count);
        digest.update(&data);
        send(dokime_domain::model::content::wire::UploadArtifactFrame {
            payload: Some(Payload::Chunk(dokime_domain::model::content::wire::ArtifactChunk { offset, data })),
        })
        .await?;
        offset = next;
    }
    if offset != expected {
        return Err(Error::Invalid("source length changed during upload"));
    }
    let mut probe = [0u8; 1];
    if source.read(&mut probe).await? != 0 {
        return Err(Error::Invalid("source length changed during upload"));
    }
    let completion = dokime_domain::model::content::wire::ArtifactCompletion {
        size_bytes: offset,
        sha256: digest.finalize().iter().map(|byte| format!("{byte:02x}")).collect::<String>(),
    };
    send(dokime_domain::model::content::wire::UploadArtifactFrame { payload: Some(Payload::Complete(completion.clone())) }).await?;
    Ok(completion)
}

/* === Tests === */
