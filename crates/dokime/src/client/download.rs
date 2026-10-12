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
use std::path::{Path, PathBuf};
// Third Party Imports
use sha2::{Digest, Sha256};
use tokio::io::AsyncWriteExt;
// Local Imports
use super::{Client, wire};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Debug)]
pub struct Download<T> {
    pub path: PathBuf,
    pub size_bytes: u64,
    pub sha256: String,
    pub metadata: T,
}

struct Partial(Option<PathBuf>);

/* === Enums === */
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Client(#[from] super::Error),
    #[error("invalid download: {0}")]
    Invalid(&'static str),
    #[error("download filesystem: {0}")]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Publication(#[from] dokime_filesystem::Error),
    #[error("download publication task: {0}")]
    Task(#[from] tokio::task::JoinError),
}

enum Frame<T> {
    Header(T, Option<(u64, String)>),
    Chunk(u64, Vec<u8>),
    Complete(u64, String),
}

/* === Traits === */

/* === Implementations === */
impl Drop for Partial {
    fn drop(&mut self) {
        if let Some(path) = &self.0 {
            let _ = std::fs::remove_file(path);
        }
    }
}

impl Client {
    pub async fn download_artifact(
        &self,
        request: wire::DownloadArtifactRequest,
        destination: PathBuf,
    ) -> Result<Download<wire::ArtifactMetadata>, Error> {
        let identity = request.artifact_id.clone();
        let stream = self.content_download_artifact(request).await?;
        let result = receive(
            || stream.message(),
            destination,
            |frame| {
                use wire::artifact_frame::Payload;
                Ok(match frame.payload.ok_or(Error::Invalid("empty frame"))? {
                    Payload::Metadata(metadata) => {
                        if metadata.artifact_id != identity {
                            return Err(Error::Invalid("download identity changed"));
                        }
                        let expected = Some((metadata.size_bytes, metadata.sha256.clone()));
                        Frame::Header(metadata, expected)
                    }
                    Payload::Chunk(chunk) => Frame::Chunk(chunk.offset, chunk.data),
                    Payload::Complete(done) => Frame::Complete(done.size_bytes, done.sha256),
                })
            },
        )
        .await;
        stream.close().await;
        result
    }

    pub async fn download_cycle_bundle(
        &self,
        request: wire::DownloadCycleBundleRequest,
        destination: PathBuf,
    ) -> Result<Download<wire::CycleBundleMetadata>, Error> {
        let identity = request.cycle_id.clone();
        let stream = self.content_download_cycle_bundle(request).await?;
        let result = receive(
            || stream.message(),
            destination,
            |frame| {
                use wire::cycle_bundle_frame::Payload;
                Ok(match frame.payload.ok_or(Error::Invalid("empty frame"))? {
                    Payload::Metadata(metadata) => {
                        if metadata.cycle_id != identity {
                            return Err(Error::Invalid("download identity changed"));
                        }
                        let expected = Some((metadata.size_bytes, metadata.sha256.clone()));
                        Frame::Header(metadata, expected)
                    }
                    Payload::Chunk(chunk) => Frame::Chunk(chunk.offset, chunk.data),
                    Payload::Complete(done) => Frame::Complete(done.size_bytes, done.sha256),
                })
            },
        )
        .await;
        stream.close().await;
        result
    }

    pub async fn download_execution_file(
        &self,
        request: wire::DownloadExecutionFileRequest,
        destination: PathBuf,
    ) -> Result<Download<wire::ExecutionFileDownloadMetadata>, Error> {
        let identity = request.relative_path.clone();
        let stream = self.content_download_execution_file(request).await?;
        let result = receive(
            || stream.message(),
            destination,
            |frame| {
                use wire::execution_file_download_frame::Payload;
                Ok(match frame.payload.ok_or(Error::Invalid("empty frame"))? {
                    Payload::Metadata(metadata) => {
                        if metadata.relative_path != identity {
                            return Err(Error::Invalid("download identity changed"));
                        }
                        let expected = None;
                        Frame::Header(metadata, expected)
                    }
                    Payload::Chunk(chunk) => Frame::Chunk(chunk.offset, chunk.data),
                    Payload::Complete(done) => Frame::Complete(done.size_bytes, done.sha256),
                })
            },
        )
        .await;
        stream.close().await;
        result
    }

    pub async fn download_checkpoint_artifact(
        &self,
        request: wire::DownloadCheckpointArtifactRequest,
        destination: PathBuf,
        expected: (u64, String),
    ) -> Result<Download<wire::ArtifactMetadata>, Error> {
        let id = request.artifact_id.clone();
        let stream = self.history_download_checkpoint_artifact(request).await?;
        let result = receive(
            || stream.message(),
            destination,
            |frame| {
                use wire::artifact_frame::Payload;
                Ok(match frame.payload.ok_or(Error::Invalid("empty frame"))? {
                    Payload::Metadata(metadata) => {
                        if metadata.artifact_id != id
                            || metadata.size_bytes != expected.0
                            || metadata.sha256 != expected.1
                        {
                            return Err(Error::Invalid("artifact identity or inventory changed"));
                        }
                        Frame::Header(metadata, None)
                    }
                    Payload::Chunk(chunk) => Frame::Chunk(chunk.offset, chunk.data),
                    Payload::Complete(done) => {
                        if done.size_bytes != expected.0 || done.sha256 != expected.1 {
                            return Err(Error::Invalid("artifact differs from inventory"));
                        }
                        Frame::Complete(done.size_bytes, done.sha256)
                    }
                })
            },
        )
        .await;
        stream.close().await;
        result
    }

    pub async fn download_cycle_report(
        &self,
        request: wire::DownloadCycleReportRequest,
        destination: PathBuf,
    ) -> Result<Download<wire::ReportMetadata>, Error> {
        let id = request.cycle_id.clone();
        let stream = self.reports_download_cycle_report(request).await?;
        let result = receive(
            || stream.message(),
            destination,
            |frame| {
                use wire::report_html_frame::Payload;
                Ok(match frame.payload.ok_or(Error::Invalid("empty frame"))? {
                    Payload::Report(metadata) => {
                        if metadata.test_cycle_id != id {
                            return Err(Error::Invalid("report belongs to another cycle"));
                        }
                        Frame::Header(metadata, None)
                    }
                    Payload::Chunk(chunk) => Frame::Chunk(chunk.offset, chunk.data),
                    Payload::Complete(done) => Frame::Complete(done.size_bytes, done.sha256),
                })
            },
        )
        .await;
        stream.close().await;
        result
    }
}

/* === Public Functions === */

/* === Private Functions === */
async fn receive<T, M, F: std::future::Future<Output = super::Result<Option<T>>>>(
    next: impl FnMut() -> F,
    destination: PathBuf,
    frame: impl Fn(T) -> Result<Frame<M>, Error>,
) -> Result<Download<M>, Error> {
    let filename = destination
        .file_name()
        .ok_or(Error::Invalid("destination has no filename"))?;
    let mut partial_name = filename.to_os_string();
    partial_name.push(".part");
    let partial = destination.with_file_name(partial_name);
    let mut cleanup = Partial(None);
    let mut output = tokio::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&partial)
        .await?;
    cleanup.0 = Some(partial.clone());
    let (offset, sha256, metadata) = consume(next, &mut output, frame).await?;
    output.flush().await?;
    output.sync_all().await?;
    drop(output);

    // Native publication is owned once started; cancellation cannot interrupt a rename.
    cleanup.0 = None;
    let target = destination.clone();
    tokio::task::spawn_blocking(move || publish(&partial, &target)).await??;
    Ok(Download {
        path: destination,
        size_bytes: offset,
        sha256,
        metadata,
    })
}

async fn consume<T, M, F: std::future::Future<Output = super::Result<Option<T>>>>(
    mut next: impl FnMut() -> F,
    mut output: impl tokio::io::AsyncWrite + Unpin,
    frame: impl Fn(T) -> Result<Frame<M>, Error>,
) -> Result<(u64, String, M), Error> {
    let mut metadata = None;
    let mut expected = None;
    let mut digest = Sha256::new();
    let mut offset = 0u64;
    let mut complete = None;
    while let Some(message) = next().await? {
        if complete.is_some() {
            return Err(Error::Invalid("frames followed completion"));
        }
        match frame(message)? {
            Frame::Header(header, inventory) if metadata.is_none() => {
                metadata = Some(header);
                expected = inventory;
            }
            Frame::Chunk(position, bytes) if metadata.is_some() => {
                if position != offset || bytes.is_empty() {
                    return Err(Error::Invalid("chunks are not contiguous"));
                }
                offset = offset
                    .checked_add(bytes.len() as u64)
                    .ok_or(Error::Invalid("file size overflow"))?;
                output.write_all(&bytes).await?;
                digest.update(&bytes);
            }
            Frame::Complete(size, hash) if metadata.is_some() => {
                let actual = digest
                    .clone()
                    .finalize()
                    .iter()
                    .map(|byte| format!("{byte:02x}"))
                    .collect::<String>();
                if size != offset
                    || hash != actual
                    || expected
                        .as_ref()
                        .is_some_and(|(length, digest)| *length != size || digest != &hash)
                {
                    return Err(Error::Invalid("size or SHA-256 mismatch"));
                }
                complete = Some(hash);
            }
            _ => return Err(Error::Invalid("expected one header before file data")),
        }
    }
    let sha256 = complete.ok_or(Error::Invalid("stream ended without completion"))?;
    let metadata = metadata.ok_or(Error::Invalid("stream ended without metadata"))?;
    Ok((offset, sha256, metadata))
}

fn publish(source: &Path, destination: &Path) -> Result<(), Error> {
    dokime_filesystem::publication::replace(source, destination)?;
    Ok(())
}

/* === Tests === */
