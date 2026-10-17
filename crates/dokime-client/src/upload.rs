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
use super::{Client, contracts};

/* === Constants === */

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
        &self, source: &Path, mut header: contracts::content::UploadArtifactHeader,
    ) -> Result<contracts::content::ArtifactMetadata, Error> {
        let file = tokio::fs::File::open(source).await?;
        header.size_bytes = file.metadata().await?.len();
        header.sha256 = None;
        let chunk_bytes = self.request_limit().saturating_sub(64).min(1024 * 1024);
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
            || metadata.state != contracts::content::ArtifactState::Complete
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
    mut source: impl AsyncRead + Unpin, header: contracts::content::UploadArtifactHeader, chunk_bytes: usize,
    mut send: impl FnMut(contracts::content::UploadArtifactFrame) -> F,
) -> Result<contracts::content::ArtifactCompletion, Error> {
    use contracts::content::UploadArtifactFramePayload as Payload;
    let expected = header.size_bytes;
    send(contracts::content::UploadArtifactFrame { payload: Some(Payload::Header(header)) }).await?;
    let mut digest = Sha256::new();
    let mut offset = 0u64;
    let mut buffer = vec![0; chunk_bytes];
    loop {
        let count = source.read(&mut buffer).await?;
        if count == 0 {
            break;
        }
        let next = offset.checked_add(count as u64).ok_or(Error::Invalid("file size overflow"))?;
        if next > expected {
            return Err(Error::Invalid("source length changed during upload"));
        }
        let data = buffer[..count].to_vec();
        digest.update(&data);
        send(contracts::content::UploadArtifactFrame { payload: Some(Payload::Chunk(contracts::content::ArtifactChunk { offset, data })) })
            .await?;
        offset = next;
    }
    if offset != expected {
        return Err(Error::Invalid("source length changed during upload"));
    }
    let completion = contracts::content::ArtifactCompletion {
        size_bytes: offset,
        sha256: digest.finalize().iter().map(|byte| format!("{byte:02x}")).collect::<String>(),
    };
    send(contracts::content::UploadArtifactFrame { payload: Some(Payload::Complete(completion.clone())) }).await?;
    Ok(completion)
}

/* === Tests === */
