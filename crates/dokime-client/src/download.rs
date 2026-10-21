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
use super::{Client, contracts};

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

#[derive(Clone, Copy)]
struct Policy {
    overwrite: bool,
    maximum_bytes: u64,
}

/* === Enums === */

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("download request failed")]
    Request(#[source] super::Error),

    #[error(transparent)]
    Client(#[from] super::Error),

    #[error("invalid download: {0}")]
    Invalid(&'static str),

    #[error("recording receipt persistence failed")]
    Receipt(#[source] Box<dyn std::error::Error + Send + Sync>),

    #[error("download filesystem: {0}")]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Publication(#[from] harmos::filesystem::Error),

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
    pub async fn download_evidence(
        &self, request: contracts::content::DownloadEvidence, destination: PathBuf, overwrite: bool,
    ) -> Result<Download<dokime_domain::model::exports::EvidenceExportMetadata>, Error> {
        use dokime_domain::model::exports::{
            EVIDENCE_EXPORT_MAX_BYTES, EVIDENCE_EXPORT_SCHEMA_VERSION, EvidenceExportPayload, EvidenceSubject,
        };
        validate_destination(&destination, overwrite)?;
        let subject = match request.subject.ok_or(Error::Invalid("evidence subject is required"))? {
            EvidenceSubject::Execution(id) => EvidenceSubject::Execution(
                dokime_domain::model::identity::TestExecutionId::from_uuid(&id)
                    .map_err(|_| Error::Invalid("invalid execution UUID"))?
                    .to_string(),
            ),
            EvidenceSubject::Cycle(id) => EvidenceSubject::Cycle(
                dokime_domain::model::identity::TestCycleId::from_uuid(&id).map_err(|_| Error::Invalid("invalid cycle UUID"))?.to_string(),
            ),
        };
        let maximum_bytes = self
            .backend()
            .limits
            .max_evidence_bytes
            .filter(|limit| *limit > 0)
            .ok_or(Error::Invalid("backend did not declare evidence export capacity"))?
            .min(EVIDENCE_EXPORT_MAX_BYTES);
        let stream = self
            .content_download_evidence(contracts::content::DownloadEvidence { subject: Some(subject.clone()) })
            .await
            .map_err(Error::Request)?;
        let result = receive_with(
            || stream.message(),
            destination,
            |frame| {
                Ok(match frame.payload.ok_or(Error::Invalid("empty evidence frame"))? {
                    EvidenceExportPayload::Metadata(metadata) => {
                        if metadata.subject.as_ref() != Some(&subject) || metadata.schema_version != EVIDENCE_EXPORT_SCHEMA_VERSION {
                            return Err(Error::Invalid("evidence subject or schema changed"));
                        }
                        if metadata.sha256.len() != 64
                            || !metadata.sha256.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
                        {
                            return Err(Error::Invalid("evidence digest is invalid"));
                        }
                        let expected = Some((metadata.size_bytes, metadata.sha256.clone()));
                        Frame::Header(metadata, expected)
                    }
                    EvidenceExportPayload::Chunk(chunk) => Frame::Chunk(chunk.offset, chunk.data),
                    EvidenceExportPayload::Complete(done) => Frame::Complete(done.size_bytes, done.sha256),
                })
            },
            Policy { overwrite, maximum_bytes },
        )
        .await;
        stream.close().await;
        result
    }

    pub async fn download_artifact(
        &self, request: contracts::content::DownloadArtifact, destination: PathBuf,
    ) -> Result<Download<dokime_domain::model::content::wire::ArtifactMetadata>, Error> {
        let identity = request.artifact_id.clone();
        let stream = self.content_download_artifact(request).await.map_err(Error::Request)?;
        let result = receive(
            || stream.message(),
            destination,
            |frame| {
                use dokime_domain::model::content::wire::ArtifactFramePayload as Payload;
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

    pub async fn download_execution_file(
        &self, request: contracts::content::DownloadExecutionFile, destination: PathBuf,
    ) -> Result<Download<dokime_domain::model::content::wire::ExecutionFileDownloadMetadata>, Error> {
        let identity = request.relative_path.clone();
        let stream = self.content_download_execution_file(request).await.map_err(Error::Request)?;
        let result = receive(
            || stream.message(),
            destination,
            |frame| {
                use dokime_domain::model::content::wire::ExecutionFileDownloadFramePayload as Payload;
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
        &self, request: contracts::history::DownloadCheckpointArtifact, destination: PathBuf, expected: (u64, String),
    ) -> Result<Download<dokime_domain::model::content::wire::ArtifactMetadata>, Error> {
        let id = request.artifact_id.clone();
        let stream = self.history_download_checkpoint_artifact(request).await.map_err(Error::Request)?;
        let result = receive(
            || stream.message(),
            destination,
            |frame| {
                use dokime_domain::model::content::wire::ArtifactFramePayload as Payload;
                Ok(match frame.payload.ok_or(Error::Invalid("empty frame"))? {
                    Payload::Metadata(metadata) => {
                        if metadata.artifact_id != id || metadata.size_bytes != expected.0 || metadata.sha256 != expected.1 {
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
        &self, request: contracts::reports::DownloadCycleReport, destination: PathBuf,
    ) -> Result<Download<dokime_domain::model::reports::wire::ReportMetadata>, Error> {
        let id = request.cycle_id.clone();
        let identifier = request.identifier.clone();
        let stream = self.reports_download_cycle_report(request).await.map_err(Error::Request)?;
        let result = receive(
            || stream.message(),
            destination,
            |frame| {
                use dokime_domain::model::reports::wire::ReportHtmlFramePayload as Payload;
                Ok(match frame.payload.ok_or(Error::Invalid("empty frame"))? {
                    Payload::Report(metadata) => {
                        if metadata.test_cycle_id != id || metadata.identifier != identifier || metadata.test_execution_id.is_some() {
                            return Err(Error::Invalid("report belongs to another cycle"));
                        }
                        Frame::Header(*metadata, None)
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

    pub async fn download_execution_report(
        &self, request: contracts::reports::DownloadExecutionReport, destination: PathBuf,
    ) -> Result<Download<dokime_domain::model::reports::wire::ReportMetadata>, Error> {
        let id = request.execution_id.clone();
        let stream = self.reports_download_execution_report(request).await.map_err(Error::Request)?;
        let result = receive(
            || stream.message(),
            destination,
            |frame| {
                use dokime_domain::model::reports::wire::ReportHtmlFramePayload as Payload;
                Ok(match frame.payload.ok_or(Error::Invalid("empty frame"))? {
                    Payload::Report(metadata) => {
                        if metadata.test_execution_id.as_deref() != Some(id.as_str()) || metadata.identifier != "execution.report" {
                            return Err(Error::Invalid("report belongs to another execution"));
                        }
                        Frame::Header(*metadata, None)
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

pub fn validate_destination(destination: &Path, overwrite: bool) -> Result<(), Error> {
    if destination.file_name().is_none() {
        return Err(Error::Invalid("destination has no filename"));
    }
    let parent = destination.parent().filter(|path| !path.as_os_str().is_empty()).unwrap_or_else(|| Path::new("."));
    if !std::fs::metadata(parent)?.is_dir() {
        return Err(Error::Invalid("destination parent is not a directory"));
    }
    match std::fs::symlink_metadata(destination) {
        Ok(_) if !overwrite => {
            return Err(std::io::Error::new(std::io::ErrorKind::AlreadyExists, "destination exists; select overwrite explicitly").into());
        }
        Ok(metadata) if !metadata.is_file() || metadata.file_type().is_symlink() => {
            return Err(Error::Invalid("destination is not a regular file"));
        }
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    let mut name = destination.file_name().expect("validated filename").to_os_string();
    name.push(".part");
    match std::fs::symlink_metadata(destination.with_file_name(name)) {
        Ok(_) => Err(std::io::Error::new(std::io::ErrorKind::AlreadyExists, "download staging file already exists").into()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

/* === Private Functions === */

async fn receive<T, M, F: std::future::Future<Output = super::Result<Option<T>>>>(
    next: impl FnMut() -> F, destination: PathBuf, frame: impl Fn(T) -> Result<Frame<M>, Error>,
) -> Result<Download<M>, Error> {
    receive_with(next, destination, frame, Policy { overwrite: true, maximum_bytes: u64::MAX }).await
}

async fn receive_with<T, M, F: std::future::Future<Output = super::Result<Option<T>>>>(
    next: impl FnMut() -> F, destination: PathBuf, frame: impl Fn(T) -> Result<Frame<M>, Error>, policy: Policy,
) -> Result<Download<M>, Error> {
    let filename = destination.file_name().ok_or(Error::Invalid("destination has no filename"))?;
    let mut partial_name = filename.to_os_string();
    partial_name.push(".part");
    let partial = destination.with_file_name(partial_name);
    let staging = partial.clone();
    let (file, mut cleanup) = tokio::task::spawn_blocking(move || {
        let file = std::fs::OpenOptions::new().write(true).create_new(true).open(&staging)?;
        Ok::<_, std::io::Error>((file, Partial(Some(staging))))
    })
    .await??;
    let mut output = tokio::fs::File::from_std(file);
    let (offset, sha256, metadata) = consume(next, &mut output, frame, policy.maximum_bytes).await?;
    output.flush().await?;
    output.sync_all().await?;
    drop(output);

    // Native publication is owned once started; cancellation cannot interrupt a rename.
    cleanup.0 = None;
    let target = destination.clone();
    tokio::task::spawn_blocking(move || publish(&partial, &target, policy.overwrite)).await??;
    Ok(Download { path: destination, size_bytes: offset, sha256, metadata })
}

async fn consume<T, M, F: std::future::Future<Output = super::Result<Option<T>>>>(
    mut next: impl FnMut() -> F, mut output: impl tokio::io::AsyncWrite + Unpin, frame: impl Fn(T) -> Result<Frame<M>, Error>,
    maximum_bytes: u64,
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
                if inventory.as_ref().is_some_and(|(size, _)| *size > maximum_bytes) {
                    return Err(Error::Invalid("download exceeds its byte limit"));
                }
                metadata = Some(header);
                expected = inventory;
            }
            Frame::Chunk(position, bytes) if metadata.is_some() => {
                if position != offset || bytes.is_empty() {
                    return Err(Error::Invalid("chunks are not contiguous"));
                }
                offset = offset
                    .checked_add(bytes.len() as u64)
                    .filter(|size| *size <= maximum_bytes)
                    .ok_or(Error::Invalid("download exceeds its byte limit"))?;
                if expected.as_ref().is_some_and(|(size, _)| offset > *size) {
                    return Err(Error::Invalid("download exceeds its declared size"));
                }
                output.write_all(&bytes).await?;
                digest.update(&bytes);
            }
            Frame::Complete(size, hash) if metadata.is_some() => {
                let actual = digest.clone().finalize().iter().map(|byte| format!("{byte:02x}")).collect::<String>();
                if size != offset || hash != actual || expected.as_ref().is_some_and(|(length, digest)| *length != size || digest != &hash)
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

fn publish(source: &Path, destination: &Path, overwrite: bool) -> Result<(), Error> {
    let _cleanup = Partial(Some(source.to_owned()));
    if overwrite {
        harmos::filesystem::publication::replace(source, destination)?;
    } else {
        harmos::filesystem::publication::install(source, destination)?;
    }
    Ok(())
}

/* === Tests === */
